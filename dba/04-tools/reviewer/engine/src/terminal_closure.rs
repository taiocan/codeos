#![allow(clippy::items_after_test_module)]

use crate::packet::{sha256_file, sha256_str, ArtifactEntry, CoverageState, ReviewPacket};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClosureScope {
    pub terminal_review_id: String,
    pub assessment_sha256: String,
    pub conditions: Vec<ClosureCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClosureCondition {
    pub id: String,
    pub description: String,
    pub source: ConditionSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ConditionSource {
    ParsedFinding { finding_id: String },
    AssessmentExcerpt { excerpt: String, sha256: String },
    IncompleteReason { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepairManifest {
    pub terminal_review_id: String,
    pub original_implementation_identity: String,
    pub current_implementation_identity: String,
    pub changed_paths: Vec<String>,
    pub evidence_identities: Vec<String>,
    pub links: Vec<RepairLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepairLink {
    pub condition_id: String,
    pub changed_paths: Vec<String>,
    pub evidence_identities: Vec<String>,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssessabilityCorrection {
    pub classification: String,
    pub failed_review_id: String,
    pub failed_assessment_sha256: String,
    pub closure_scope_sha256: String,
    pub implementation_identity: String,
    pub deficiency_excerpt: String,
    pub deficiency_sha256: String,
    pub corrections: Vec<CorrectedEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorrectedEvidence {
    pub path: String,
    pub sha256: String,
    pub supersedes_path: String,
    pub supersedes_sha256: String,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub struct ValidatedScope {
    pub scope: ClosureScope,
    pub raw: String,
    pub sha256: String,
    pub assessment_path: PathBuf,
    pub original_implementation_identity: String,
    pub original_packet_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ValidatedRepair {
    pub raw: String,
    pub sha256: String,
    pub condition_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TerminalClosureExecution {
    pub scope: ValidatedScope,
    pub repair: ValidatedRepair,
    pub decision_timestamp: String,
    pub reason: String,
    pub attempt: u8,
    pub legacy_retry: bool,
    pub input_sha256: String,
    pub assessability_correction_sha256: Option<String>,
}

pub struct TerminalClosureRetry {
    pub packet: ReviewPacket,
    pub execution: TerminalClosureExecution,
}

fn record_block<'a>(content: &'a str, review_id: &str) -> Result<&'a str> {
    let marker = format!("Review ID: {review_id}");
    let (_, tail) = content
        .split_once(&marker)
        .with_context(|| format!("terminal review {review_id} does not exist"))?;
    Ok(tail.split("\n## ").next().unwrap_or(tail))
}

fn parenthesized_reference(line: &str, prefix: &str) -> Result<(PathBuf, String)> {
    let value = line
        .strip_prefix(prefix)
        .with_context(|| format!("terminal record omits {prefix}"))?;
    let (path, hash) = value
        .rsplit_once(" (sha256:")
        .with_context(|| format!("malformed {prefix} reference"))?;
    Ok((PathBuf::from(path), hash.trim_end_matches(')').to_string()))
}

fn parsed_assessment(assessment: &str) -> Result<serde_yaml::Value> {
    let frontmatter = assessment
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---"))
        .map(|(frontmatter, _)| frontmatter)
        .context("terminal assessment has no readable YAML frontmatter")?;
    match serde_yaml::from_str(frontmatter) {
        Ok(value) => Ok(value),
        Err(original_error) => {
            recover_historical_effective_concern_note(frontmatter).with_context(|| {
                format!("terminal assessment frontmatter is malformed: {original_error}")
            })
        }
    }
}

/// Recover the one historical writer defect without changing the immutable assessment. The
/// recovery is deliberately field-specific and reconstructs YAML only in memory. Anything other
/// than one exact, non-empty plain-scalar line containing the known `: ` trigger stays closed.
fn recover_historical_effective_concern_note(frontmatter: &str) -> Result<serde_yaml::Value> {
    const PREFIX: &str = "  effective_concern_note: ";
    let candidates = frontmatter
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("effective_concern_note:"))
        .collect::<Vec<_>>();
    anyhow::ensure!(
        candidates.len() == 1,
        "historical effective_concern_note recovery requires exactly one field"
    );
    let (line_index, line) = candidates[0];
    let value = line
        .strip_prefix(PREFIX)
        .context("historical effective_concern_note has an ambiguous field boundary")?;
    anyhow::ensure!(
        !value.is_empty()
            && value.contains(": ")
            && !value.contains('\t')
            && !value.contains(" #")
            && !matches!(
                value.chars().next(),
                Some(
                    '"' | '\''
                        | '['
                        | ']'
                        | '{'
                        | '}'
                        | '|'
                        | '>'
                        | '&'
                        | '*'
                        | '!'
                        | '%'
                        | '@'
                        | '`'
                )
            ),
        "historical effective_concern_note value is missing or ambiguous"
    );

    let mut repaired = frontmatter.lines().map(str::to_string).collect::<Vec<_>>();
    repaired[line_index] = format!(
        "{PREFIX}{}",
        serde_json::to_string(value).context("quote historical effective_concern_note")?
    );
    let yaml: serde_yaml::Value = serde_yaml::from_str(&repaired.join("\n"))
        .context("frontmatter remains malformed after bounded historical recovery")?;
    anyhow::ensure!(
        yaml.get("reviewed")
            .and_then(|reviewed| reviewed.get("effective_concern_note"))
            .and_then(serde_yaml::Value::as_str)
            == Some(value),
        "historical effective_concern_note did not round-trip exactly"
    );
    Ok(yaml)
}

pub fn load_and_validate_scope(
    log_path: &Path,
    feature: &str,
    stage: &str,
    terminal_review_id: &str,
    current_semantic_revision: Option<u64>,
    scope_path: &Path,
) -> Result<ValidatedScope> {
    anyhow::ensure!(
        stage == "8",
        "Terminal Review Closure is available only at Stage 8"
    );
    let expected = match current_semantic_revision {
        Some(revision) => format!("REV__{feature}__8__SR{revision}__REENTRY-1"),
        None => format!("REV__{feature}__8__REENTRY-1"),
    };
    anyhow::ensure!(
        terminal_review_id == expected,
        "terminal review does not belong to the current package review cycle"
    );
    let log = std::fs::read_to_string(log_path)?;
    let block = record_block(&log, terminal_review_id)?;
    anyhow::ensure!(
        block
            .lines()
            .any(|line| line == "Effective concern: DO NOT ADVANCE")
            || block.contains("Assessment status: INCOMPLETE"),
        "terminal review is neither negative nor incomplete"
    );
    let closure_id = match current_semantic_revision {
        Some(revision) => format!("REV__{feature}__8__SR{revision}__TERMINAL-CLOSURE-1"),
        None => format!("REV__{feature}__8__TERMINAL-CLOSURE-1"),
    };
    anyhow::ensure!(
        !log.contains(&format!("Review ID: {closure_id}")),
        "the Terminal Review Closure for this terminal review has already been consumed"
    );
    let assessment_line = block
        .lines()
        .find(|line| line.starts_with("Full assessment: "))
        .context("terminal review omits its full assessment")?;
    let (assessment_path, recorded_assessment_sha) =
        parenthesized_reference(assessment_line, "Full assessment: ")?;
    let actual_assessment_sha = sha256_file(
        assessment_path
            .to_str()
            .context("assessment path is not valid Unicode")?,
    )?;
    anyhow::ensure!(
        actual_assessment_sha == recorded_assessment_sha,
        "terminal assessment content no longer matches its immutable record"
    );
    let assessment = std::fs::read_to_string(&assessment_path)?;
    let yaml = parsed_assessment(&assessment)?;

    let raw = std::fs::read_to_string(scope_path)?;
    let scope: ClosureScope = serde_json::from_str(&raw).context("invalid closure scope JSON")?;
    anyhow::ensure!(
        scope.terminal_review_id == terminal_review_id,
        "closure scope targets another terminal review"
    );
    anyhow::ensure!(
        scope.assessment_sha256 == actual_assessment_sha,
        "closure scope assessment identity mismatch"
    );
    anyhow::ensure!(
        !scope.conditions.is_empty(),
        "closure scope has no conditions"
    );

    let findings = yaml
        .get("findings")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let parsed_ids = findings
        .iter()
        .filter_map(|finding| finding.get("finding_id")?.as_str().map(str::to_string))
        .collect::<BTreeSet<_>>();
    let blocking_ids = findings
        .iter()
        .filter(|finding| {
            finding
                .get("classification")
                .and_then(serde_yaml::Value::as_str)
                == Some("IN-SCOPE BLOCKER")
        })
        .filter_map(|finding| finding.get("finding_id")?.as_str().map(str::to_string))
        .collect::<BTreeSet<_>>();
    let incomplete_reason = block.lines().find_map(|line| {
        line.strip_prefix("INCOMPLETE — ")
            .and_then(|value| value.strip_suffix(" — advancement prohibited on this record."))
    });
    let mut condition_ids = BTreeSet::new();
    let mut sourced_findings = BTreeSet::new();
    for condition in &scope.conditions {
        anyhow::ensure!(
            !condition.id.trim().is_empty() && condition_ids.insert(condition.id.clone()),
            "closure condition IDs must be non-empty and unique"
        );
        anyhow::ensure!(
            !condition.description.trim().is_empty(),
            "closure condition description is empty"
        );
        match &condition.source {
            ConditionSource::ParsedFinding { finding_id } => {
                anyhow::ensure!(
                    parsed_ids.contains(finding_id),
                    "unknown parsed finding {finding_id}"
                );
                sourced_findings.insert(finding_id.clone());
            }
            ConditionSource::AssessmentExcerpt { excerpt, sha256 } => {
                anyhow::ensure!(!excerpt.trim().is_empty(), "assessment excerpt is empty");
                anyhow::ensure!(
                    assessment.contains(excerpt),
                    "assessment excerpt is not present in the immutable assessment"
                );
                anyhow::ensure!(
                    sha256_str(excerpt) == *sha256,
                    "assessment excerpt SHA-256 mismatch"
                );
            }
            ConditionSource::IncompleteReason { reason } => {
                anyhow::ensure!(
                    incomplete_reason == Some(reason.as_str()),
                    "condition does not cite the exact recorded incomplete reason"
                );
            }
        }
    }
    anyhow::ensure!(
        blocking_ids.is_subset(&sourced_findings),
        "closure scope omits one or more parsed blocking findings"
    );

    let base_line = block
        .lines()
        .find(|line| line.starts_with("Base: "))
        .context("terminal review omits implementation binding")?;
    let original_review_commit = base_line
        .split("  Review: ")
        .nth(1)
        .and_then(|value| value.split("  Branch:").next())
        .context("terminal review has malformed implementation binding")?
        .to_string();
    let original_diff_hash = block
        .lines()
        .find_map(|line| line.strip_prefix("Diff-hash: "))
        .context("terminal review omits implementation diff identity")?;
    let packet_line = block
        .lines()
        .find(|line| line.starts_with("Reviewed packet: "))
        .context("terminal review omits its reviewed packet")?;
    let (original_packet_path, _) = parenthesized_reference(packet_line, "Reviewed packet: ")?;
    let canonical_raw = serde_json::to_string(&scope)?;
    Ok(ValidatedScope {
        scope,
        sha256: sha256_str(&canonical_raw),
        raw: canonical_raw,
        assessment_path,
        original_implementation_identity: format!("{original_review_commit}:{original_diff_hash}"),
        original_packet_path,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_execution(
    log_path: &Path,
    repo_root: &Path,
    feature: &str,
    stage: &str,
    terminal_review_id: &str,
    current_semantic_revision: Option<u64>,
    packet: &ReviewPacket,
    manifest_path: &Path,
) -> Result<TerminalClosureExecution> {
    let log = std::fs::read_to_string(log_path)?;
    let marker = format!("Review ID: {terminal_review_id}");
    let (_, tail) = log
        .split_once(&marker)
        .context("terminal review does not exist")?;
    let suffix = format!(" HUMAN DECISION — {feature} — Stage {stage}");
    let mut authorization = None;
    let mut stopped = false;
    for block in tail.split("\n## ").skip(1) {
        let mut lines = block.lines();
        let heading = lines.next().unwrap_or_default();
        if heading.ends_with(&suffix) {
            let values = lines.collect::<Vec<_>>();
            let decision = values
                .iter()
                .find_map(|line| line.strip_prefix("Decision: "));
            if decision == Some("STOP") {
                stopped = true;
            } else if decision == Some("AUTHORIZE_TERMINAL_REVIEW_CLOSURE")
                && values.iter().any(|line| {
                    line.strip_prefix("Terminal closure after: ") == Some(terminal_review_id)
                })
            {
                let scope_json = values
                    .iter()
                    .find_map(|line| line.strip_prefix("Terminal closure scope JSON: "))
                    .context("authorization omits its canonical closure scope")?;
                let timestamp = heading
                    .strip_suffix(&suffix)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                let reason = values
                    .iter()
                    .find_map(|line| line.strip_prefix("Reason/next: "))
                    .unwrap_or_default()
                    .to_string();
                authorization = Some((scope_json.to_string(), timestamp, reason));
                stopped = false;
            }
        }
    }
    anyhow::ensure!(
        !stopped,
        "a later STOP denies Terminal Review Closure execution"
    );
    let (scope_json, decision_timestamp, reason) = authorization
        .context("no Terminal Review Closure authorization follows the terminal review")?;
    let temp = tempfile::NamedTempFile::new()?;
    std::fs::write(temp.path(), scope_json)?;
    let scope = load_and_validate_scope(
        log_path,
        feature,
        stage,
        terminal_review_id,
        current_semantic_revision,
        temp.path(),
    )?;
    let repair = validate_repair_manifest(repo_root, packet, &scope, manifest_path)?;
    Ok(TerminalClosureExecution {
        scope,
        repair,
        decision_timestamp,
        reason,
        attempt: 1,
        legacy_retry: false,
        input_sha256: String::new(),
        assessability_correction_sha256: None,
    })
}

fn line_value<'a>(block: &'a str, prefix: &str) -> Result<&'a str> {
    block
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .with_context(|| format!("closure record omits {prefix}"))
}

fn coverage_state(value: &str) -> Result<CoverageState> {
    match value {
        "FULL_COVERAGE" => Ok(CoverageState::FullCoverage),
        "PARTIAL_COVERAGE" => Ok(CoverageState::PartialCoverage),
        "SECRET_REDACTION" => Ok(CoverageState::SecretRedaction),
        "CRITICAL_OMISSION" => Ok(CoverageState::CriticalOmission),
        "EMPTY_PACKET" => Ok(CoverageState::EmptyPacket),
        _ => anyhow::bail!("closure assessment has unknown coverage state"),
    }
}

fn packet_from_record(
    repo_root: &Path,
    block: &str,
    feature: &str,
    stage: &str,
    failed_review_id: &str,
    scope_sha: &str,
    correction_path: Option<&Path>,
) -> Result<(ReviewPacket, PathBuf, String, Option<String>)> {
    let packet_line = block
        .lines()
        .find(|line| line.starts_with("Reviewed packet: "))
        .context("closure failure omits its reviewed packet")?;
    let (packet_path, recorded_packet_sha) =
        parenthesized_reference(packet_line, "Reviewed packet: ")?;
    let packet_content = std::fs::read_to_string(&packet_path)?;
    anyhow::ensure!(
        sha256_str(&packet_content) == recorded_packet_sha,
        "saved closure payload no longer matches its immutable record"
    );

    let assessment_line = block
        .lines()
        .find(|line| line.starts_with("Full assessment: "))
        .context("closure failure omits its assessment")?;
    let (assessment_path, recorded_assessment_sha) =
        parenthesized_reference(assessment_line, "Full assessment: ")?;
    anyhow::ensure!(
        sha256_file(assessment_path.to_string_lossy().as_ref())? == recorded_assessment_sha,
        "closure failure assessment no longer matches its immutable record"
    );
    let assessment = std::fs::read_to_string(&assessment_path)?;
    let yaml = parsed_assessment(&assessment)?;
    let reviewed = yaml
        .get("reviewed")
        .context("closure assessment omits reviewed bindings")?;
    anyhow::ensure!(
        reviewed.get("feature").and_then(serde_yaml::Value::as_str) == Some(feature),
        "closure failure belongs to another feature"
    );
    let recorded_stage = reviewed
        .get("stage")
        .context("closure assessment omits stage")?;
    anyhow::ensure!(
        recorded_stage.as_str() == Some(stage)
            || recorded_stage.as_u64().map(|value| value.to_string()) == Some(stage.to_string()),
        "closure failure belongs to another stage"
    );
    let mut artifacts = reviewed
        .get("artifacts")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|item| ArtifactEntry {
            path: item
                .get("path")
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or_default()
                .to_string(),
            sha256: item
                .get("sha256")
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or_default()
                .to_string(),
            visibility: item
                .get("visibility")
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or_default()
                .to_string(),
            bytes: 0,
        })
        .collect::<Vec<_>>();

    let correction = if let Some(path) = correction_path {
        let raw = std::fs::read_to_string(path)?;
        let value: AssessabilityCorrection =
            serde_json::from_str(&raw).context("invalid assessability correction JSON")?;
        anyhow::ensure!(
            value.classification == "ASSESSABILITY_CORRECTION",
            "evidence correction classification must be ASSESSABILITY_CORRECTION"
        );
        anyhow::ensure!(
            value.failed_review_id == failed_review_id,
            "assessability correction targets another failed closure"
        );
        anyhow::ensure!(
            value.failed_assessment_sha256 == recorded_assessment_sha,
            "assessability correction assessment identity mismatch"
        );
        anyhow::ensure!(
            value.closure_scope_sha256 == scope_sha,
            "assessability correction expands or changes closure scope"
        );
        let implementation_identity = format!(
            "{}:{}",
            reviewed
                .get("review_commit")
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or_default(),
            reviewed
                .get("diff_hash")
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or_default()
        );
        anyhow::ensure!(
            value.implementation_identity == implementation_identity,
            "assessability correction implementation identity mismatch"
        );
        anyhow::ensure!(
            !value.deficiency_excerpt.trim().is_empty()
                && assessment.contains(&value.deficiency_excerpt)
                && sha256_str(&value.deficiency_excerpt) == value.deficiency_sha256,
            "assessability correction does not cite an exact deficiency from the failed assessment"
        );
        anyhow::ensure!(
            !value.corrections.is_empty(),
            "assessability correction supplies no corrected evidence"
        );
        let artifact_hashes = artifacts
            .iter()
            .map(|artifact| (artifact.path.as_str(), artifact.sha256.as_str()))
            .collect::<BTreeMap<_, _>>();
        let mut paths = BTreeSet::new();
        for item in &value.corrections {
            anyhow::ensure!(
                !item.rationale.trim().is_empty() && paths.insert(item.path.clone()),
                "assessability correction paths must be unique and have a rationale"
            );
            anyhow::ensure!(
                item.path.starts_with("evidence/")
                    || item.path.starts_with(".codeos/06-workflow/event-evidence/")
                    || item
                        .path
                        .starts_with(".codeos/06-workflow/reality-evidence/"),
                "assessability correction may contain evidence paths only"
            );
            anyhow::ensure!(
                artifact_hashes.get(item.supersedes_path.as_str())
                    == Some(&item.supersedes_sha256.as_str()),
                "corrected evidence is not bound to a superseded packet artifact"
            );
            let absolute = repo_root.join(&item.path);
            anyhow::ensure!(
                absolute.is_file()
                    && sha256_file(absolute.to_string_lossy().as_ref())? == item.sha256,
                "corrected evidence identity mismatch at {}",
                item.path
            );
        }
        let raw_sha = sha256_str(&raw);
        Some((value, raw, raw_sha))
    } else {
        None
    };

    let corrected_sources = correction
        .as_ref()
        .map(|(value, _, _)| {
            value
                .corrections
                .iter()
                .map(|item| item.supersedes_path.as_str())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    for artifact in &artifacts {
        if artifact.sha256.is_empty()
            || matches!(
                artifact.path.as_str(),
                ".codeos/codeos.yaml" | ".codeos/00-project/codeos.yaml"
            )
            || artifact.visibility == "missing"
            || corrected_sources.contains(artifact.path.as_str())
        {
            continue;
        }
        let path = Path::new(&artifact.path);
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        anyhow::ensure!(
            absolute.is_file()
                && sha256_file(absolute.to_string_lossy().as_ref())? == artifact.sha256,
            "closure retry input changed at {}",
            artifact.path
        );
    }

    let string = |key: &str| {
        reviewed
            .get(key)
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let mut final_content = packet_content;
    let correction_sha = correction.as_ref().map(|(_, _, sha)| sha.clone());
    if let Some((value, raw, _)) = correction {
        final_content.push_str(&format!(
            "\n\nASSESSABILITY CORRECTION\nImmutable predecessor packet SHA-256: {recorded_packet_sha}\nCorrection manifest:\n{raw}\nCorrected evidence follows. Assess only whether it remedies the cited evidence deficiency for the unchanged original closure scope. Do not reopen or expand that scope.\n"
        ));
        for item in value.corrections {
            let raw_evidence = std::fs::read_to_string(repo_root.join(&item.path))?;
            let (sanitized, hits) = crate::precheck::redact_secrets(&raw_evidence);
            anyhow::ensure!(hits == 0, "corrected evidence contains secret-like values");
            final_content.push_str(&format!(
                "\n--- {} (ASSESSABILITY_CORRECTION; sha256: {}; supersedes {} @ {}) ---\n{}\n",
                item.path, item.sha256, item.supersedes_path, item.supersedes_sha256, sanitized
            ));
            artifacts.push(ArtifactEntry {
                path: item.path,
                sha256: item.sha256,
                visibility: "assessability_correction".to_string(),
                bytes: raw_evidence.len() as u64,
            });
        }
    }
    let packet = ReviewPacket::from_immutable_record(
        feature.to_string(),
        stage.to_string(),
        string("branch"),
        string("review_commit"),
        string("base_commit"),
        string("diff_hash"),
        coverage_state(
            reviewed
                .get("coverage_state")
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or_default(),
        )?,
        reviewed
            .get("workspace_dirty")
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(false),
        reviewed
            .get("redaction_count")
            .and_then(serde_yaml::Value::as_u64)
            .unwrap_or(0) as usize,
        reviewed
            .get("secret_redaction")
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(false),
        artifacts,
        Vec::new(),
        final_content,
    );
    Ok((packet, packet_path, recorded_packet_sha, correction_sha))
}

pub fn resolve_retry_execution(
    log_path: &Path,
    repo_root: &Path,
    feature: &str,
    stage: &str,
    failed_review_id: &str,
    current_semantic_revision: Option<u64>,
    correction_path: Option<&Path>,
) -> Result<TerminalClosureRetry> {
    anyhow::ensure!(
        stage == "8",
        "Terminal Review Closure retry is available only at Stage 8"
    );
    let failure_one = format_terminal_failure_id(feature, current_semantic_revision, 1);
    let legacy_id = format_terminal_closure_id(feature, current_semantic_revision);
    let legacy_retry = failed_review_id == legacy_id;
    anyhow::ensure!(
        failed_review_id == failure_one || legacy_retry,
        "retry must target FAILURE-1 or an eligible legacy malformed closure"
    );
    let log = std::fs::read_to_string(log_path)?;
    anyhow::ensure!(
        log.matches(&format!("Review ID: {failed_review_id}"))
            .count()
            == 1,
        "closure failure identity is missing or ambiguous"
    );
    let block = record_block(&log, failed_review_id)?;
    anyhow::ensure!(
        block.contains("Parse status: FAILED") || block.contains("Assessment status: INCOMPLETE"),
        "a structurally complete closure assessment is final and cannot be retried"
    );
    if !legacy_retry {
        anyhow::ensure!(
            block
                .lines()
                .any(|line| line == "Closure execution status: STRUCTURAL FAILURE"),
            "v13 retry target is not a structural execution failure"
        );
    }
    let semantic_id = format_terminal_closure_id(feature, current_semantic_revision);
    let bridge_id = format_legacy_retry_id(feature, current_semantic_revision);
    let failure_two = format_terminal_failure_id(feature, current_semantic_revision, 2);
    anyhow::ensure!(
        !log.contains(&format!("Review ID: {failure_two}"))
            && !log.contains(&format!("Review ID: {bridge_id}"))
            && (legacy_retry || !log.contains(&format!("Review ID: {semantic_id}"))),
        "Terminal Review Closure retry capacity has already been consumed"
    );

    let terminal_review_id = line_value(block, "Terminal closure after: ")?.to_string();
    let expected_terminal = match current_semantic_revision {
        Some(revision) => format!("REV__{feature}__8__SR{revision}__REENTRY-1"),
        None => format!("REV__{feature}__8__REENTRY-1"),
    };
    anyhow::ensure!(
        terminal_review_id == expected_terminal,
        "closure failure belongs to another package cycle"
    );
    let scope_sha = line_value(block, "Terminal closure scope SHA-256: ")?.to_string();
    let repair_sha = line_value(block, "Repair manifest SHA-256: ")?.to_string();
    let conditions = line_value(block, "Authorized conditions: ")?
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    anyhow::ensure!(
        !conditions.is_empty(),
        "closure failure has no authorized conditions"
    );
    let authorization = line_value(block, "Human authorization: ")?
        .strip_suffix(" HUMAN DECISION")
        .unwrap_or_default()
        .to_string();
    let reason = line_value(block, "Authorized scope: ")?.to_string();
    let before = log
        .split_once(&format!("Review ID: {failed_review_id}"))
        .map(|(value, _)| value)
        .unwrap_or_default();
    anyhow::ensure!(
        before.contains("Decision: AUTHORIZE_TERMINAL_REVIEW_CLOSURE")
            && before.contains(&format!("Terminal closure after: {terminal_review_id}"))
            && before.contains(&format!("Terminal closure scope SHA-256: {scope_sha}")),
        "closure failure lacks its valid original human authorization"
    );

    let (packet, packet_path, packet_sha, correction_sha) = packet_from_record(
        repo_root,
        block,
        feature,
        stage,
        failed_review_id,
        &scope_sha,
        correction_path,
    )?;
    if legacy_retry {
        anyhow::ensure!(
            packet.content().contains("dba_configuration: DBA-17"),
            "legacy closure payload does not prove DBA-17 as its historical authority"
        );
    }
    let input_sha256 =
        sha256_str(&format!(
        "{feature}\n{stage}\n{}\n{terminal_review_id}\n{scope_sha}\n{repair_sha}\n{packet_sha}\n{}",
        current_semantic_revision.map(|value| value.to_string()).unwrap_or_else(|| "0".into()),
        conditions.join(",")
    ));
    if let Ok(recorded) = line_value(block, "Closure input SHA-256: ") {
        anyhow::ensure!(
            recorded == input_sha256,
            "closure retry input identity mismatch"
        );
    } else {
        anyhow::ensure!(legacy_retry, "v13 closure failure omits its input identity");
    }
    let original_identity = format!("{}:{}", packet.review_sha, packet.diff_hash);
    let execution = TerminalClosureExecution {
        scope: ValidatedScope {
            scope: ClosureScope {
                terminal_review_id,
                assessment_sha256: String::new(),
                conditions: Vec::new(),
            },
            raw: String::new(),
            sha256: scope_sha,
            assessment_path: PathBuf::new(),
            original_implementation_identity: original_identity,
            original_packet_path: packet_path,
        },
        repair: ValidatedRepair {
            raw: String::new(),
            sha256: repair_sha,
            condition_ids: conditions,
        },
        decision_timestamp: authorization,
        reason,
        attempt: 2,
        legacy_retry,
        input_sha256,
        assessability_correction_sha256: correction_sha,
    };
    Ok(TerminalClosureRetry { packet, execution })
}

pub fn format_terminal_closure_id(feature: &str, semantic_revision: Option<u64>) -> String {
    match semantic_revision {
        Some(revision) => format!("REV__{feature}__8__SR{revision}__TERMINAL-CLOSURE-1"),
        None => format!("REV__{feature}__8__TERMINAL-CLOSURE-1"),
    }
}

pub fn format_terminal_failure_id(
    feature: &str,
    semantic_revision: Option<u64>,
    attempt: u8,
) -> String {
    match semantic_revision {
        Some(revision) => {
            format!("REV__{feature}__8__SR{revision}__TERMINAL-CLOSURE-FAILURE-{attempt}")
        }
        None => format!("REV__{feature}__8__TERMINAL-CLOSURE-FAILURE-{attempt}"),
    }
}

pub fn format_legacy_retry_id(feature: &str, semantic_revision: Option<u64>) -> String {
    match semantic_revision {
        Some(revision) => {
            format!("REV__{feature}__8__SR{revision}__TERMINAL-CLOSURE-RETRY-1")
        }
        None => format!("REV__{feature}__8__TERMINAL-CLOSURE-RETRY-1"),
    }
}

pub fn closure_input_sha(
    feature: &str,
    stage: &str,
    semantic_revision: Option<u64>,
    execution: &TerminalClosureExecution,
    packet_sha: &str,
) -> String {
    let base = format!(
        "{feature}\n{stage}\n{}\n{}\n{}\n{}\n{packet_sha}\n{}",
        semantic_revision
            .map(|value| value.to_string())
            .unwrap_or_else(|| "0".into()),
        execution.scope.scope.terminal_review_id,
        execution.scope.sha256,
        execution.repair.sha256,
        execution.repair.condition_ids.join(",")
    );
    match execution.assessability_correction_sha256.as_deref() {
        Some(correction) => sha256_str(&format!("{base}\n{correction}")),
        None => sha256_str(&base),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    const HISTORICAL_SR3_FRONTMATTER: &str = "---\nreview_id: REV__F-0001__8__SR3__REENTRY-1\nfindings: []\nunparsed_findings_count: 1\nreviewed:\n  feature: F-0001\n  stage: 8\n  coverage_state: FULL_COVERAGE\n  codex_concern: CHANGES ADVISED\n  effective_concern: DO NOT ADVANCE\n  effective_concern_note: raised to DO NOT ADVANCE: assessment INCOMPLETE — 1 finding block(s) were malformed and could not be recorded\n  evidence: B\n  parse_status: FAILED\n  assessment_status: INCOMPLETE\n  incomplete_reason: \"1 finding block(s) were malformed and could not be recorded\"\n---\n\nHistorical body remains immutable.\n";

    #[test]
    fn historical_sr3_note_is_recovered_exactly_without_changing_bytes() {
        let frontmatter = HISTORICAL_SR3_FRONTMATTER
            .strip_prefix("---\n")
            .unwrap()
            .split_once("\n---")
            .unwrap()
            .0;
        let parse_error = serde_yaml::from_str::<serde_yaml::Value>(frontmatter)
            .unwrap_err()
            .to_string();
        assert!(
            parse_error.contains("mapping values are not allowed"),
            "unexpected exact parse failure: {parse_error}"
        );

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("immutable-assessment.md");
        std::fs::write(&path, HISTORICAL_SR3_FRONTMATTER).unwrap();
        let before = std::fs::read(&path).unwrap();
        let before_sha = sha256_file(path.to_string_lossy().as_ref()).unwrap();
        let parsed = parsed_assessment(HISTORICAL_SR3_FRONTMATTER).unwrap();
        assert_eq!(
            parsed["reviewed"]["effective_concern_note"].as_str(),
            Some(
                "raised to DO NOT ADVANCE: assessment INCOMPLETE — 1 finding block(s) were malformed and could not be recorded"
            )
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(
            sha256_file(path.to_string_lossy().as_ref()).unwrap(),
            before_sha
        );
    }

    #[test]
    fn historical_recovery_rejects_ambiguous_missing_and_duplicate_fields() {
        let ambiguous = HISTORICAL_SR3_FRONTMATTER.replace(
            "assessment INCOMPLETE — 1 finding",
            "assessment INCOMPLETE # comment: 1 finding",
        );
        assert!(parsed_assessment(&ambiguous).is_err());

        let missing = HISTORICAL_SR3_FRONTMATTER.replace(
            "  effective_concern_note: raised to DO NOT ADVANCE: assessment INCOMPLETE — 1 finding block(s) were malformed and could not be recorded\n",
            "  another_broken_field: still: malformed\n",
        );
        assert!(parsed_assessment(&missing).is_err());

        let duplicate = HISTORICAL_SR3_FRONTMATTER.replace(
            "  evidence: B\n",
            "  effective_concern_note: conflicting: duplicate value\n  evidence: B\n",
        );
        assert!(parsed_assessment(&duplicate).is_err());
    }

    fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf, String) {
        let dir = tempfile::tempdir().unwrap();
        let assessment = dir.path().join("assessment.md");
        let assessment_text = "---\nfindings:\n  - finding_id: TERM__F1\n    classification: IN-SCOPE BLOCKER\nassessment_status: INCOMPLETE\n---\n\nFinding: broken output\nExact ambiguous-looking but bounded concern.\n";
        std::fs::write(&assessment, assessment_text).unwrap();
        let assessment_sha = hex::encode(Sha256::digest(assessment_text.as_bytes()));
        let packet = dir.path().join("packet.txt");
        std::fs::write(&packet, "original packet").unwrap();
        let packet_sha = hex::encode(Sha256::digest(b"original packet"));
        let log = dir.path().join("review-log.md");
        std::fs::write(
            &log,
            format!(
                "# log\n## t RE-ENTRY REVIEW — FEAT — Stage 8\nReview ID: REV__FEAT__8__SR2__REENTRY-1\nBase: base  Review: current  Branch: main\nDiff-hash: original-diff\nEffective concern: DO NOT ADVANCE\nParse status: FAILED   Assessment status: INCOMPLETE\nINCOMPLETE — one malformed finding — advancement prohibited on this record.\nFull assessment: {} (sha256:{assessment_sha})\nReviewed packet: {} (sha256:{packet_sha})\n",
                assessment.display(),
                packet.display()
            ),
        )
        .unwrap();
        (dir, log, assessment, assessment_sha)
    }

    #[test]
    fn malformed_terminal_scope_uses_only_verified_immutable_sources() {
        let (dir, log, _, assessment_sha) = fixture();
        let excerpt = "Exact ambiguous-looking but bounded concern.";
        let scope = dir.path().join("scope.json");
        std::fs::write(
            &scope,
            serde_json::json!({
                "terminal_review_id": "REV__FEAT__8__SR2__REENTRY-1",
                "assessment_sha256": assessment_sha,
                "conditions": [
                    {"id":"C1","description":"Repair parsed blocker","source":{"kind":"parsed_finding","finding_id":"TERM__F1"}},
                    {"id":"C2","description":"Bound readable concern","source":{"kind":"assessment_excerpt","excerpt":excerpt,"sha256":sha256_str(excerpt)}},
                    {"id":"C3","description":"Produce complete assessment","source":{"kind":"incomplete_reason","reason":"one malformed finding"}}
                ]
            })
            .to_string(),
        )
        .unwrap();
        load_and_validate_scope(
            &log,
            "FEAT",
            "8",
            "REV__FEAT__8__SR2__REENTRY-1",
            Some(2),
            &scope,
        )
        .unwrap();
    }

    #[test]
    fn altered_excerpt_and_omitted_parsed_blocker_fail_closed() {
        let (dir, log, _, assessment_sha) = fixture();
        let scope = dir.path().join("scope.json");
        std::fs::write(
            &scope,
            serde_json::json!({
                "terminal_review_id": "REV__FEAT__8__SR2__REENTRY-1",
                "assessment_sha256": assessment_sha,
                "conditions": [{"id":"C1","description":"invented","source":{"kind":"assessment_excerpt","excerpt":"not in assessment","sha256":sha256_str("not in assessment")}}]
            }).to_string(),
        ).unwrap();
        assert!(load_and_validate_scope(
            &log,
            "FEAT",
            "8",
            "REV__FEAT__8__SR2__REENTRY-1",
            Some(2),
            &scope
        )
        .is_err());
    }

    #[test]
    fn closure_identity_is_package_bound_and_legacy_deterministic() {
        assert_eq!(
            format_terminal_closure_id("FEAT", Some(2)),
            "REV__FEAT__8__SR2__TERMINAL-CLOSURE-1"
        );
        assert_eq!(
            format_terminal_closure_id("FEAT", None),
            "REV__FEAT__8__TERMINAL-CLOSURE-1"
        );
    }

    #[test]
    fn repair_manifest_requires_complete_path_links_and_fresh_bound_evidence() {
        use crate::packet::{build, PacketBuildOptions};
        use std::process::Command;

        let repo = tempfile::tempdir().unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.email", "test@example.invalid"],
            vec!["config", "user.name", "Test"],
        ] {
            assert!(Command::new("git")
                .args(args)
                .current_dir(repo.path())
                .status()
                .unwrap()
                .success());
        }
        std::fs::write(repo.path().join("implementation.txt"), "before").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(repo.path())
            .status()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "base"])
            .current_dir(repo.path())
            .status()
            .unwrap();
        let head = String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(repo.path())
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();
        std::fs::write(repo.path().join("implementation.txt"), "after").unwrap();
        std::fs::write(repo.path().join("fresh-evidence.txt"), "fresh proof").unwrap();
        let evidence_path = repo.path().join("fresh-evidence.txt");
        let implementation_path = repo.path().join("implementation.txt");
        let toolkit_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .unwrap();
        let mut packet = build(&PacketBuildOptions {
            feature: "FEAT".into(),
            stage: "8".into(),
            artifacts: vec![
                implementation_path.display().to_string(),
                evidence_path.display().to_string(),
            ],
            sha_only_paths: vec![],
            basis: crate::packet::ReviewBasis::Ordinary { base: None },
            repo_root: repo.path().display().to_string(),
            toolkit_root: toolkit_root.display().to_string(),
            scope: None,
        })
        .unwrap();
        packet.bounded_changed_paths = Some(BTreeSet::from([
            "fresh-evidence.txt".to_string(),
            "implementation.txt".to_string(),
        ]));
        let evidence_sha = sha256_file(evidence_path.to_str().unwrap()).unwrap();
        let original_packet_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(original_packet_file.path(), "old evidence identity").unwrap();
        let scope = ValidatedScope {
            scope: ClosureScope {
                terminal_review_id: "REV__FEAT__8__SR2__REENTRY-1".into(),
                assessment_sha256: "a".repeat(64),
                conditions: vec![ClosureCondition {
                    id: "C1".into(),
                    description: "repair".into(),
                    source: ConditionSource::IncompleteReason {
                        reason: "repair".into(),
                    },
                }],
            },
            raw: "{}".into(),
            sha256: sha256_str("{}"),
            assessment_path: repo.path().join("assessment"),
            original_implementation_identity: format!("{head}:original-diff"),
            original_packet_path: original_packet_file.path().to_path_buf(),
        };
        let manifest_file = tempfile::NamedTempFile::new().unwrap();
        let manifest = serde_json::json!({
            "terminal_review_id":"REV__FEAT__8__SR2__REENTRY-1",
            "original_implementation_identity":scope.original_implementation_identity,
            "current_implementation_identity":format!("{}:{}", packet.review_sha, packet.diff_hash),
            "changed_paths":["fresh-evidence.txt","implementation.txt"],
            "evidence_identities":[evidence_sha],
            "links":[{"condition_id":"C1","changed_paths":["fresh-evidence.txt","implementation.txt"],"evidence_identities":[evidence_sha],"rationale":"repair and prove C1"}]
        });
        std::fs::write(manifest_file.path(), manifest.to_string()).unwrap();
        validate_repair_manifest(repo.path(), &packet, &scope, manifest_file.path()).unwrap();

        let mut incomplete = manifest;
        incomplete["links"][0]["changed_paths"] = serde_json::json!(["implementation.txt"]);
        std::fs::write(manifest_file.path(), incomplete.to_string()).unwrap();
        assert!(
            validate_repair_manifest(repo.path(), &packet, &scope, manifest_file.path()).is_err()
        );
    }

    #[test]
    fn dirty_terminal_snapshot_allows_only_post_review_repair_paths() {
        use crate::packet::{build, PacketBuildOptions};
        use std::process::Command;

        let repo = tempfile::tempdir().unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.email", "test@example.invalid"],
            vec!["config", "user.name", "Test"],
        ] {
            assert!(Command::new("git")
                .args(args)
                .current_dir(repo.path())
                .status()
                .unwrap()
                .success());
        }
        let governed = repo.path().join(".codeos/01-specification/intents/FEAT.md");
        std::fs::create_dir_all(governed.parent().unwrap()).unwrap();
        std::fs::write(&governed, "approved baseline\n").unwrap();
        std::fs::write(repo.path().join("implementation.txt"), "baseline\n").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(repo.path())
            .status()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "base"])
            .current_dir(repo.path())
            .status()
            .unwrap();

        std::fs::write(&governed, "approved reviewed meaning\n").unwrap();
        std::fs::write(
            repo.path().join("implementation.txt"),
            "reviewed implementation\n",
        )
        .unwrap();
        std::fs::write(
            repo.path().join("historical-evidence.txt"),
            "reviewed proof\n",
        )
        .unwrap();
        let toolkit_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .unwrap();
        let original = build(&PacketBuildOptions {
            feature: "FEAT".into(),
            stage: "8".into(),
            artifacts: vec![
                governed.display().to_string(),
                repo.path().join("implementation.txt").display().to_string(),
                repo.path()
                    .join("historical-evidence.txt")
                    .display()
                    .to_string(),
            ],
            sha_only_paths: vec![],
            basis: crate::packet::ReviewBasis::Ordinary { base: None },
            repo_root: repo.path().display().to_string(),
            toolkit_root: toolkit_root.display().to_string(),
            scope: None,
        })
        .unwrap();
        let control = tempfile::tempdir().unwrap();
        let original_packet = control.path().join("terminal.packet.txt");
        std::fs::write(&original_packet, original.content()).unwrap();
        let scope = ValidatedScope {
            scope: ClosureScope {
                terminal_review_id: "REV__FEAT__8__SR2__REENTRY-1".into(),
                assessment_sha256: "a".repeat(64),
                conditions: vec![ClosureCondition {
                    id: "C1".into(),
                    description: "repair".into(),
                    source: ConditionSource::IncompleteReason {
                        reason: "repair".into(),
                    },
                }],
            },
            raw: "{}".into(),
            sha256: sha256_str("{}"),
            assessment_path: repo.path().join("assessment"),
            original_implementation_identity: format!(
                "{}:{}",
                original.review_sha, original.diff_hash
            ),
            original_packet_path: original_packet,
        };

        std::fs::write(repo.path().join("implementation.txt"), "repaired\n").unwrap();
        let evidence = repo.path().join("fresh-evidence.txt");
        std::fs::write(&evidence, "fresh proof\n").unwrap();
        let mut current = build(&PacketBuildOptions {
            feature: "FEAT".into(),
            stage: "8".into(),
            artifacts: vec![
                governed.display().to_string(),
                repo.path().join("implementation.txt").display().to_string(),
                evidence.display().to_string(),
            ],
            sha_only_paths: vec![],
            basis: crate::packet::ReviewBasis::Ordinary { base: None },
            repo_root: repo.path().display().to_string(),
            toolkit_root: toolkit_root.display().to_string(),
            scope: None,
        })
        .unwrap();
        current.bounded_changed_paths = Some(BTreeSet::from([
            "fresh-evidence.txt".to_string(),
            "implementation.txt".to_string(),
        ]));
        let evidence_sha = sha256_file(evidence.to_str().unwrap()).unwrap();
        let manifest_path = control.path().join("repair-manifest.json");
        let manifest = serde_json::json!({
            "terminal_review_id":"REV__FEAT__8__SR2__REENTRY-1",
            "original_implementation_identity":scope.original_implementation_identity,
            "current_implementation_identity":format!("{}:{}", current.review_sha, current.diff_hash),
            "changed_paths":["fresh-evidence.txt","implementation.txt"],
            "evidence_identities":[evidence_sha],
            "links":[{"condition_id":"C1","changed_paths":["fresh-evidence.txt","implementation.txt"],"evidence_identities":[evidence_sha],"rationale":"repair and proof"}]
        });
        std::fs::write(&manifest_path, manifest.to_string()).unwrap();
        validate_repair_manifest(repo.path(), &current, &scope, &manifest_path).unwrap();

        std::fs::write(&governed, "post-review governed drift\n").unwrap();
        let mut drifted = build(&PacketBuildOptions {
            feature: "FEAT".into(),
            stage: "8".into(),
            artifacts: vec![
                governed.display().to_string(),
                repo.path().join("implementation.txt").display().to_string(),
                evidence.display().to_string(),
            ],
            sha_only_paths: vec![],
            basis: crate::packet::ReviewBasis::Ordinary { base: None },
            repo_root: repo.path().display().to_string(),
            toolkit_root: toolkit_root.display().to_string(),
            scope: None,
        })
        .unwrap();
        drifted.bounded_changed_paths = Some(BTreeSet::from([
            ".codeos/01-specification/intents/FEAT.md".to_string(),
            "fresh-evidence.txt".to_string(),
            "implementation.txt".to_string(),
        ]));
        let mut drift_manifest = manifest;
        drift_manifest["current_implementation_identity"] =
            serde_json::json!(format!("{}:{}", drifted.review_sha, drifted.diff_hash));
        std::fs::write(&manifest_path, drift_manifest.to_string()).unwrap();
        let error = validate_repair_manifest(repo.path(), &drifted, &scope, &manifest_path)
            .unwrap_err()
            .to_string();
        assert!(error.contains("governed-meaning or architecture drift"));
    }

    #[test]
    fn complete_open_closure_is_final_and_not_retryable() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("review-log.md");
        let closure = format_terminal_closure_id("FEAT", Some(2));
        std::fs::write(
            &log,
            format!(
                "## t TERMINAL REVIEW CLOSURE — FEAT — Stage 8\nReview ID: {closure}\nParse status: OK   Assessment status: COMPLETE\nClosure semantic disposition: OPEN\n"
            ),
        )
        .unwrap();
        let error = resolve_retry_execution(&log, dir.path(), "FEAT", "8", &closure, Some(2), None)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("structurally complete closure assessment is final"));
    }

    #[test]
    fn malformed_retry_exhausts_capacity_without_creating_semantic_closure() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("review-log.md");
        let failure_one = format_terminal_failure_id("FEAT", Some(2), 1);
        let failure_two = format_terminal_failure_id("FEAT", Some(2), 2);
        std::fs::write(
            &log,
            format!(
                "## t TERMINAL REVIEW CLOSURE EXECUTION FAILURE — FEAT — Stage 8\nReview ID: {failure_one}\nClosure execution status: STRUCTURAL FAILURE\nParse status: FAILED   Assessment status: INCOMPLETE\n\n## t TERMINAL REVIEW CLOSURE EXECUTION FAILURE — FEAT — Stage 8\nReview ID: {failure_two}\nClosure execution status: STRUCTURAL FAILURE\nParse status: FAILED   Assessment status: INCOMPLETE\n"
            ),
        )
        .unwrap();
        let error =
            resolve_retry_execution(&log, dir.path(), "FEAT", "8", &failure_one, Some(2), None)
                .err()
                .unwrap()
                .to_string();
        assert!(error.contains("retry capacity has already been consumed"));
        assert!(!std::fs::read_to_string(log)
            .unwrap()
            .contains("Closure semantic disposition:"));
    }

    #[test]
    fn legacy_v12_bridge_requires_valid_authorization_and_unchanged_payload_bindings() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("tracked.md");
        std::fs::write(&artifact, "reviewed bytes\n").unwrap();
        let artifact_sha = sha256_file(artifact.to_str().unwrap()).unwrap();
        let packet = dir.path().join("closure.packet.txt");
        std::fs::write(
            &packet,
            "dba_configuration: DBA-17\nimmutable closure payload\n",
        )
        .unwrap();
        let packet_sha = sha256_file(packet.to_str().unwrap()).unwrap();
        let assessment = dir.path().join("closure-assessment.md");
        let assessment_text = format!(
            "---\nreview_id: REV__FEAT__8__SR2__TERMINAL-CLOSURE-1\nfindings: []\nunparsed_findings_count: 1\nreviewed:\n  feature: FEAT\n  stage: 8\n  branch: main\n  base_commit: base\n  review_commit: review\n  artifacts:\n    - path: {}\n      sha256: {artifact_sha}\n      visibility: full\n  diff_hash: diff\n  coverage_state: FULL_COVERAGE\n  workspace_dirty: false\n  redaction_count: 0\n  secret_redaction: false\n  reviewed_packet_sha256: {packet_sha}\nparse_status: FAILED\nassessment_status: INCOMPLETE\n---\n",
            artifact.display()
        );
        std::fs::write(&assessment, &assessment_text).unwrap();
        let assessment_sha = sha256_file(assessment.to_str().unwrap()).unwrap();
        let terminal = "REV__FEAT__8__SR2__REENTRY-1";
        let legacy = format_terminal_closure_id("FEAT", Some(2));
        let log = dir.path().join("review-log.md");
        std::fs::write(
            &log,
            format!(
                "## t RE-ENTRY REVIEW — FEAT — Stage 8\nReview ID: {terminal}\nEffective concern: DO NOT ADVANCE\n\n## t HUMAN DECISION — FEAT — Stage 8\nDecision: AUTHORIZE_TERMINAL_REVIEW_CLOSURE\nTerminal closure after: {terminal}\nTerminal closure scope SHA-256: scope\n\n## t TERMINAL REVIEW CLOSURE — FEAT — Stage 8\nReview ID: {legacy}\nTerminal closure after: {terminal}\nHuman authorization: t HUMAN DECISION\nAuthorized scope: exact scope\nTerminal closure scope SHA-256: scope\nRepair manifest SHA-256: repair\nAuthorized conditions: C1\nParse status: FAILED   Assessment status: INCOMPLETE\nFull assessment: {} (sha256:{assessment_sha})\nReviewed packet: {} (sha256:{packet_sha})\n",
                assessment.display(),
                packet.display()
            ),
        )
        .unwrap();
        let retry =
            resolve_retry_execution(&log, dir.path(), "FEAT", "8", &legacy, Some(2), None).unwrap();
        assert!(retry.execution.legacy_retry);
        assert_eq!(
            retry.packet.content(),
            "dba_configuration: DBA-17\nimmutable closure payload\n"
        );

        std::fs::write(&artifact, "changed after malformed closure\n").unwrap();
        let error = resolve_retry_execution(&log, dir.path(), "FEAT", "8", &legacy, Some(2), None)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("closure retry input changed"), "{error}");
    }

    #[test]
    fn assessability_correction_is_evidence_only_and_bound_to_the_failed_attempt() {
        let dir = tempfile::tempdir().unwrap();
        let evidence = dir.path().join("evidence/original.json");
        let corrected = dir.path().join("evidence/corrected.json");
        let implementation = dir.path().join("tracked.md");
        let governed = dir.path().join(".codeos/00-project/charter.md");
        std::fs::create_dir_all(evidence.parent().unwrap()).unwrap();
        std::fs::create_dir_all(governed.parent().unwrap()).unwrap();
        std::fs::write(&evidence, "{\"geometry\":\"hash-only\"}\n").unwrap();
        std::fs::write(&corrected, "{\"geometry\":[[14.5,46.0],[11.5,48.1]]}\n").unwrap();
        std::fs::write(&implementation, "unchanged implementation\n").unwrap();
        std::fs::write(&governed, "unchanged charter\n").unwrap();
        let evidence_sha = sha256_file(evidence.to_str().unwrap()).unwrap();
        let corrected_sha = sha256_file(corrected.to_str().unwrap()).unwrap();
        let implementation_sha = sha256_file(implementation.to_str().unwrap()).unwrap();
        let governed_sha = sha256_file(governed.to_str().unwrap()).unwrap();
        let packet = dir.path().join("closure.packet.txt");
        std::fs::write(
            &packet,
            "dba_configuration: DBA-17\nimmutable closure payload\n",
        )
        .unwrap();
        let packet_sha = sha256_file(packet.to_str().unwrap()).unwrap();
        let deficiency = "Required evidence: actual sanitized geometry is not inspectable.";
        let assessment = dir.path().join("closure-assessment.md");
        let assessment_text = format!(
            "---\nreview_id: REV__FEAT__8__SR2__TERMINAL-CLOSURE-1\nfindings: []\nunparsed_findings_count: 1\nreviewed:\n  feature: FEAT\n  stage: 8\n  branch: main\n  base_commit: base\n  review_commit: review\n  artifacts:\n    - path: evidence/original.json\n      sha256: {evidence_sha}\n      visibility: shown\n    - path: tracked.md\n      sha256: {implementation_sha}\n      visibility: shown\n    - path: .codeos/00-project/charter.md\n      sha256: {governed_sha}\n      visibility: shown\n  diff_hash: diff\n  coverage_state: FULL_COVERAGE\n  workspace_dirty: true\n  redaction_count: 0\n  secret_redaction: false\n  reviewed_packet_sha256: {packet_sha}\nparse_status: FAILED\nassessment_status: INCOMPLETE\n---\n{deficiency}\n"
        );
        std::fs::write(&assessment, &assessment_text).unwrap();
        let assessment_sha = sha256_file(assessment.to_str().unwrap()).unwrap();
        let terminal = "REV__FEAT__8__SR2__REENTRY-1";
        let legacy = format_terminal_closure_id("FEAT", Some(2));
        let log = dir.path().join("review-log.md");
        std::fs::write(
            &log,
            format!(
                "## t RE-ENTRY REVIEW — FEAT — Stage 8\nReview ID: {terminal}\nEffective concern: DO NOT ADVANCE\n\n## t HUMAN DECISION — FEAT — Stage 8\nDecision: AUTHORIZE_TERMINAL_REVIEW_CLOSURE\nTerminal closure after: {terminal}\nTerminal closure scope SHA-256: scope\n\n## t TERMINAL REVIEW CLOSURE — FEAT — Stage 8\nReview ID: {legacy}\nTerminal closure after: {terminal}\nHuman authorization: t HUMAN DECISION\nAuthorized scope: exact scope\nTerminal closure scope SHA-256: scope\nRepair manifest SHA-256: repair\nAuthorized conditions: C1, C2, C3\nParse status: FAILED   Assessment status: INCOMPLETE\nFull assessment: {} (sha256:{assessment_sha})\nReviewed packet: {} (sha256:{packet_sha})\n",
                assessment.display(),
                packet.display()
            ),
        )
        .unwrap();
        let correction = dir.path().join("correction.json");
        let write_correction = |scope: &str, supersedes_sha: &str| {
            std::fs::write(
                &correction,
                serde_json::json!({
                    "classification": "ASSESSABILITY_CORRECTION",
                    "failed_review_id": legacy,
                    "failed_assessment_sha256": assessment_sha,
                    "closure_scope_sha256": scope,
                    "implementation_identity": "review:diff",
                    "deficiency_excerpt": deficiency,
                    "deficiency_sha256": sha256_str(deficiency),
                    "corrections": [{
                        "path": "evidence/corrected.json",
                        "sha256": corrected_sha,
                        "supersedes_path": "evidence/original.json",
                        "supersedes_sha256": supersedes_sha,
                        "rationale": "Expose the coordinates omitted by the original evidence."
                    }]
                })
                .to_string(),
            )
            .unwrap();
        };
        write_correction("scope", &evidence_sha);
        let history_before = std::fs::read(&log).unwrap();
        let retry = resolve_retry_execution(
            &log,
            dir.path(),
            "FEAT",
            "8",
            &legacy,
            Some(2),
            Some(&correction),
        )
        .unwrap();
        assert!(retry.execution.legacy_retry);
        assert!(retry.execution.assessability_correction_sha256.is_some());
        assert!(retry.packet.content().contains("ASSESSABILITY CORRECTION"));
        assert!(retry.packet.content().contains("[[14.5,46.0],[11.5,48.1]]"));
        assert_eq!(std::fs::read(&log).unwrap(), history_before);

        std::fs::write(&implementation, "changed implementation\n").unwrap();
        assert!(resolve_retry_execution(
            &log,
            dir.path(),
            "FEAT",
            "8",
            &legacy,
            Some(2),
            Some(&correction)
        )
        .err()
        .unwrap()
        .to_string()
        .contains("input changed"));
        std::fs::write(&implementation, "unchanged implementation\n").unwrap();
        std::fs::write(&governed, "changed charter\n").unwrap();
        assert!(resolve_retry_execution(
            &log,
            dir.path(),
            "FEAT",
            "8",
            &legacy,
            Some(2),
            Some(&correction)
        )
        .err()
        .unwrap()
        .to_string()
        .contains("input changed"));
        std::fs::write(&governed, "unchanged charter\n").unwrap();

        write_correction("expanded-scope", &evidence_sha);
        assert!(resolve_retry_execution(
            &log,
            dir.path(),
            "FEAT",
            "8",
            &legacy,
            Some(2),
            Some(&correction)
        )
        .err()
        .unwrap()
        .to_string()
        .contains("closure scope"));
        write_correction("scope", "unrelated-evidence-identity");
        assert!(resolve_retry_execution(
            &log,
            dir.path(),
            "FEAT",
            "8",
            &legacy,
            Some(2),
            Some(&correction)
        )
        .err()
        .unwrap()
        .to_string()
        .contains("superseded packet artifact"));
    }
}

pub fn validate_repair_manifest(
    _repo_root: &Path,
    packet: &ReviewPacket,
    scope: &ValidatedScope,
    manifest_path: &Path,
) -> Result<ValidatedRepair> {
    let raw = std::fs::read_to_string(manifest_path)?;
    let manifest: RepairManifest =
        serde_json::from_str(&raw).context("invalid repair manifest JSON")?;
    anyhow::ensure!(
        manifest.terminal_review_id == scope.scope.terminal_review_id,
        "repair manifest targets another terminal review"
    );
    anyhow::ensure!(
        manifest.original_implementation_identity == scope.original_implementation_identity,
        "repair manifest original implementation mismatch"
    );
    anyhow::ensure!(
        manifest.current_implementation_identity
            == format!("{}:{}", packet.review_sha, packet.diff_hash),
        "repair manifest current implementation mismatch"
    );

    let expected_conditions = scope
        .scope
        .conditions
        .iter()
        .map(|condition| condition.id.clone())
        .collect::<BTreeSet<_>>();
    let linked_conditions = manifest
        .links
        .iter()
        .map(|link| link.condition_id.clone())
        .collect::<BTreeSet<_>>();
    anyhow::ensure!(
        expected_conditions == linked_conditions,
        "repair manifest does not account for the exact authorized condition set"
    );
    anyhow::ensure!(
        manifest
            .links
            .iter()
            .all(|link| !link.rationale.trim().is_empty()),
        "repair manifest contains an empty rationale"
    );

    let repository_paths = packet
        .bounded_changed_paths
        .clone()
        .context("Terminal Review Closure packet has no resolved bounded delta")?;
    anyhow::ensure!(
        !repository_paths.iter().any(|path| {
            path == ".codeos/00-project/charter.md"
                || path.starts_with(".codeos/01-specification/")
                || path.starts_with(".codeos/02-architecture/")
        }),
        "Terminal Review Closure cannot carry governed-meaning or architecture drift"
    );
    let actual_paths = repository_paths
        .into_iter()
        .filter(|path| !path.starts_with(".codeos/") && !path.starts_with(".codeos-state/"))
        .collect::<BTreeSet<_>>();
    let declared_paths = manifest
        .changed_paths
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    anyhow::ensure!(
        actual_paths == declared_paths,
        "repair manifest changed paths do not match the repository delta: actual={actual_paths:?} declared={declared_paths:?}"
    );
    let linked_paths = manifest
        .links
        .iter()
        .flat_map(|link| link.changed_paths.iter().cloned())
        .collect::<BTreeSet<_>>();
    let evidence = manifest
        .evidence_identities
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let linked_evidence = manifest
        .links
        .iter()
        .flat_map(|link| link.evidence_identities.iter().cloned())
        .collect::<BTreeSet<_>>();
    anyhow::ensure!(
        linked_paths == declared_paths,
        "every changed path must be linked to an authorized condition"
    );
    anyhow::ensure!(
        linked_evidence == evidence,
        "every evidence identity must be linked to an authorized condition"
    );
    let current_hashes = packet
        .artifacts
        .iter()
        .map(|artifact| artifact.sha256.clone())
        .collect::<BTreeSet<_>>();
    anyhow::ensure!(
        evidence
            .iter()
            .all(|identity| current_hashes.contains(identity)),
        "repair evidence identity is not bound to the current review packet"
    );
    if !declared_paths.is_empty() {
        anyhow::ensure!(
            !evidence.is_empty(),
            "implementation repair requires fresh evidence"
        );
        let original_packet = std::fs::read_to_string(&scope.original_packet_path)?;
        anyhow::ensure!(
            evidence
                .iter()
                .all(|identity| !original_packet.contains(identity)),
            "implementation repair reused pre-repair evidence"
        );
    }
    Ok(ValidatedRepair {
        sha256: sha256_str(&raw),
        raw,
        condition_ids: expected_conditions.into_iter().collect(),
    })
}
