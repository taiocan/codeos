use crate::packet::{sha256_file, sha256_str, ReviewPacket};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
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
    serde_yaml::from_str(frontmatter).context("terminal assessment frontmatter is malformed")
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
    })
}

pub fn format_terminal_closure_id(feature: &str, semantic_revision: Option<u64>) -> String {
    match semantic_revision {
        Some(revision) => format!("REV__{feature}__8__SR{revision}__TERMINAL-CLOSURE-1"),
        None => format!("REV__{feature}__8__TERMINAL-CLOSURE-1"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

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
        let packet = build(&PacketBuildOptions {
            feature: "FEAT".into(),
            stage: "8".into(),
            artifacts: vec![
                implementation_path.display().to_string(),
                evidence_path.display().to_string(),
            ],
            sha_only_paths: vec![],
            delta_mode: false,
            delta_base: None,
            repo_root: repo.path().display().to_string(),
            toolkit_root: toolkit_root.display().to_string(),
        })
        .unwrap();
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
}

pub fn validate_repair_manifest(
    repo_root: &Path,
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

    let output = std::process::Command::new("git")
        .args([
            "diff",
            "--name-only",
            scope
                .original_implementation_identity
                .split(':')
                .next()
                .unwrap_or_default(),
        ])
        .current_dir(repo_root)
        .output()?;
    anyhow::ensure!(
        output.status.success(),
        "could not derive repair changed paths"
    );
    let mut repository_paths = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let untracked = std::process::Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(repo_root)
        .output()?;
    anyhow::ensure!(
        untracked.status.success(),
        "could not derive untracked repair paths"
    );
    repository_paths.extend(
        String::from_utf8_lossy(&untracked.stdout)
            .lines()
            .map(str::to_string),
    );
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
        "repair manifest changed paths do not match the repository delta"
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
