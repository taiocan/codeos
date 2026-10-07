//! Live predicates over canonical artifacts, plus reviewer-record lookup and a command runner.
//! Nothing here is a receipt: these are conditions the tool evaluates now, or durable records it
//! reads. A historical exit code is never trusted on its own.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

/// Split a Markdown file into (yaml frontmatter, body). Frontmatter is the block between the first
/// two `---` lines when the file starts with `---`.
pub fn split_frontmatter(text: &str) -> (Option<&str>, &str) {
    let rest = match text.strip_prefix("---\n") {
        Some(r) => r,
        None => return (None, text),
    };
    match rest.find("\n---\n") {
        Some(end) => (Some(&rest[..end]), &rest[end + 5..]),
        None => (None, text),
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct SpecFrontmatter {
    pub status: Option<String>,
    pub approved_by: Option<String>,
    pub approved_at: Option<serde_yaml::Value>,
}

/// True when an Intent/Contract/Event-Schema records the joint package approval.
pub fn spec_artifact_approved(path: &Path) -> Result<bool> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(false);
    };
    let (Some(fm), _) = split_frontmatter(&text) else {
        return Ok(false);
    };
    let fm: SpecFrontmatter = serde_yaml::from_str(fm)
        .with_context(|| format!("parse frontmatter of {}", path.display()))?;
    let approved = fm.status.as_deref() == Some("APPROVED")
        && fm.approved_by.as_deref().map(str::trim).unwrap_or("").len() > 3
        && fm
            .approved_at
            .as_ref()
            .map(|v| {
                !matches!(v, serde_yaml::Value::Null)
                    && !v.as_str().map(str::trim).unwrap_or("x").is_empty()
            })
            .unwrap_or(false);
    Ok(approved)
}

/// True when a Charter or Architecture Scope records `approval.by` and `approval.at`.
pub fn approval_recorded(path: &Path) -> Result<bool> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(false);
    };
    let (Some(fm), _) = split_frontmatter(&text) else {
        return Ok(false);
    };
    #[derive(Deserialize)]
    struct Fm {
        approval: Option<serde_yaml::Value>,
    }
    let fm: Fm = serde_yaml::from_str(fm).unwrap_or(Fm { approval: None });
    let Some(serde_yaml::Value::Mapping(m)) = fm.approval else {
        return Ok(false);
    };
    let non_empty = |k: &str| {
        m.get(serde_yaml::Value::String(k.into()))
            .and_then(|v| {
                v.as_str()
                    .map(|s| s.trim().to_string())
                    .or_else(|| Some(v_to_string(v)))
            })
            .map(|s| !s.is_empty() && s != "~" && s != "null")
            .unwrap_or(false)
    };
    Ok(non_empty("by") && non_empty("at"))
}

fn v_to_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

/// Does the file contain a `## <heading>` section that has real content and no unfilled
/// `[placeholder]` token? An unfilled placeholder is bracketed text not immediately followed by
/// `(` (which would make it a Markdown link) and not just digits or `#`.
pub fn section_filled(path: &Path, heading: &str) -> Result<bool> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(false);
    };
    let want = format!("## {heading}");
    let mut found = false;
    let mut in_section = false;
    let mut in_comment = false;
    let mut saw_content = false;

    for line in text.lines() {
        let t = line.trim();
        if !in_comment && t.starts_with("## ") {
            if in_section {
                break;
            }
            in_section = t == want;
            found = found || in_section;
            continue;
        }
        if !in_section {
            continue;
        }
        if in_comment {
            if t.contains("-->") {
                in_comment = false;
            }
            continue;
        }
        if t.starts_with("<!--") {
            if !t.contains("-->") {
                in_comment = true;
            }
            continue;
        }
        if t.is_empty() || t.chars().all(|c| c == '|' || c == '-' || c == ' ') {
            continue;
        }
        if has_unfilled_placeholder(t) {
            return Ok(false);
        }
        saw_content = true;
    }
    Ok(found && saw_content)
}

fn has_unfilled_placeholder(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            if let Some(rel) = line[i + 1..].find(']') {
                let inner = &line[i + 1..i + 1 + rel];
                let after = line.as_bytes().get(i + 1 + rel + 1).copied();
                let is_link = after == Some(b'(');
                let trivial = inner.is_empty()
                    || inner
                        .chars()
                        .all(|c| c.is_ascii_digit() || c == '#' || c == '-' || c == 'O');
                if !is_link && !trivial {
                    return true;
                }
                i = i + 1 + rel + 1;
                continue;
            }
        }
        i += 1;
    }
    false
}

/// A reviewer assessment record, parsed from its frontmatter.
#[derive(Debug, Clone)]
pub struct ReviewRecord {
    pub review_id: String,
    pub review_commit: String,
    pub reviewed_packet_sha256: String,
    pub effective_concern: String,
    pub production_evidence: Option<String>,
    pub artifacts: Vec<ReviewArtifact>,
    /// The Review Input Manifest sidecar (Reviewer Tool Contract v17), present only when its packet
    /// hash equals the review's recorded `reviewed_packet_sha256`.
    pub manifest_sidecar: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewArtifact {
    pub path: String,
    pub sha256: String,
    pub visibility: String,
}

/// The most recent reviewer record for `feature` at `stage`, by filename timestamp order.
/// Load the v17 manifest sidecar written beside a review's packet, bound to that exact packet.
fn manifest_sidecar(assessment: &Path, reviewed: Option<&serde_yaml::Value>) -> Option<serde_json::Value> {
    let field = |name: &str| reviewed?.get(name)?.as_str().map(str::to_string);
    let packet = assessment.parent()?.join(field("reviewed_packet")?);
    let sidecar: serde_json::Value =
        serde_json::from_slice(&std::fs::read(format!("{}.meta.json", packet.display())).ok()?)
            .ok()?;
    (sidecar.get("manifest").is_some()
        && sidecar.get("content_sha256").and_then(|v| v.as_str())
            == field("reviewed_packet_sha256").as_deref())
    .then_some(sidecar)
}

/// For a review carrying a v17 manifest, every identity it recorded that differs from the identity
/// derived from current state. `None` when the review has no manifest, so its HEAD binding applies.
pub fn review_identity_drift(
    root: &Path,
    implementation_paths: &[String],
    record: &ReviewRecord,
) -> Result<Option<Vec<String>>> {
    let Some(sidecar) = &record.manifest_sidecar else {
        return Ok(None);
    };
    let manifest = &sidecar["manifest"];
    let mut drift = Vec::new();
    let file_sha = |rel: &str| crate::hashing::file_sha256(&root.join(rel)).ok().flatten();

    if manifest["implementation"].as_str()
        != Some(crate::hashing::working_tree_state(root, implementation_paths)?.as_str())
    {
        drift.push("implementation".to_string());
    }
    if manifest["configuration"].as_str() != file_sha(".codeos/00-project/codeos.yaml").as_deref() {
        drift.push(".codeos/00-project/codeos.yaml".to_string());
    }
    for artifact in sidecar["artifacts"].as_array().into_iter().flatten() {
        let (Some(path), Some(sha)) = (artifact["path"].as_str(), artifact["sha256"].as_str()) else {
            continue;
        };
        if !sha.is_empty() && file_sha(path).as_deref() != Some(sha) {
            drift.push(path.to_string());
        }
    }
    let store = std::fs::read_to_string(root.join(".codeos/06-workflow/verifications.jsonl"))
        .unwrap_or_default();
    for (key, recorded) in manifest["verification"].as_object().into_iter().flatten() {
        let Some((checkpoint, verification)) = key.split_once('/') else {
            continue;
        };
        let current = store.lines().rev().find_map(|line| {
            let record: serde_json::Value = serde_json::from_str(line).ok()?;
            (record["workflow"] == "feature"
                && record["subject"].as_str() == manifest["feature"].as_str()
                && record["checkpoint"] == checkpoint
                && record["verification"] == verification)
                .then(|| crate::hashing::bytes_sha256(line.as_bytes()))
        });
        if recorded.as_str() != current.as_deref() {
            drift.push(key.clone());
        }
    }
    Ok(Some(drift))
}

pub fn latest_review_record(
    dir: &Path,
    feature: &str,
    stage: &str,
) -> Result<Option<ReviewRecord>> {
    if !dir.is_dir() {
        return Ok(None);
    }
    fn collect_markdown(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                collect_markdown(&entry.path(), files)?;
            } else if file_type.is_file() && entry.path().extension().is_some_and(|x| x == "md") {
                files.push(entry.path());
            }
        }
        Ok(())
    }

    let mut candidates = Vec::new();
    collect_markdown(dir, &mut candidates)?;
    candidates.sort_by(|a, b| a.file_name().cmp(&b.file_name()).then_with(|| a.cmp(b)));
    let mut latest_name = None;
    let mut latest = None;
    for path in candidates.into_iter().rev() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let (Some(fm), _) = split_frontmatter(&text) else {
            continue;
        };
        let doc: serde_yaml::Value = match serde_yaml::from_str(fm) {
            Ok(d) => d,
            Err(_) => continue,
        };
        if doc
            .get("counts_as_semantic_closure")
            .and_then(|value| value.as_bool())
            == Some(false)
        {
            continue;
        }
        let reviewed = doc.get("reviewed");
        let f = reviewed
            .and_then(|r| r.get("feature"))
            .and_then(|v| v.as_str());
        let s = reviewed.and_then(|r| r.get("stage"));
        let stage_matches = s.and_then(|v| v.as_str()) == Some(stage)
            || s.and_then(|v| v.as_u64()).map(|v| v.to_string()) == Some(stage.to_string());
        if f != Some(feature) || !stage_matches {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if latest_name.as_deref() == Some(name) {
            anyhow::bail!(
                "ambiguous reviewer records for {feature} at stage {stage}: duplicate canonical filename {name}"
            );
        }
        if latest.is_some() {
            break;
        }
        latest_name = Some(name.to_string());
        let artifacts = reviewed
            .and_then(|r| r.get("artifacts"))
            .and_then(|v| v.as_sequence())
            .map(|artifacts| {
                artifacts
                    .iter()
                    .filter_map(|artifact| {
                        Some(ReviewArtifact {
                            path: artifact.get("path")?.as_str()?.to_string(),
                            sha256: artifact.get("sha256")?.as_str()?.to_string(),
                            visibility: artifact.get("visibility")?.as_str()?.to_string(),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let production_evidence = artifacts.iter().find_map(|artifact| {
            let identity = artifact
                .path
                .strip_prefix(".codeos/06-workflow/reality-evidence/")?
                .strip_suffix(".json")?;
            (identity.len() == 64 && artifact.sha256 == identity && artifact.visibility == "shown")
                .then(|| identity.to_string())
        });
        latest = Some(ReviewRecord {
            review_id: doc
                .get("review_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            review_commit: reviewed
                .and_then(|r| r.get("review_commit"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            reviewed_packet_sha256: reviewed
                .and_then(|r| r.get("reviewed_packet_sha256"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            effective_concern: reviewed
                .and_then(|r| r.get("effective_concern"))
                .or_else(|| doc.get("effective_concern"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            production_evidence,
            artifacts,
            manifest_sidecar: manifest_sidecar(&path, reviewed),
        });
    }
    Ok(latest)
}

/// A DBA-18 assessability correction is an inspectable derivative, not replacement production
/// evidence. Accept its lineage only when the immutable retry record contains the selected F6
/// source, the superseded evidence, and the corrected evidence, and every on-disk identity and
/// implementation binding agrees.
pub fn review_binds_production_evidence(
    repo_root: &Path,
    review: &ReviewRecord,
    selected: &str,
) -> Result<bool> {
    if review.production_evidence.as_deref() == Some(selected) {
        return Ok(true);
    }
    if review.review_id.ends_with("__TERMINAL-CLOSURE-1") {
        let selected_path = format!(".codeos/06-workflow/reality-evidence/{selected}.json");
        let Some(selected_artifact) = review
            .artifacts
            .iter()
            .find(|artifact| artifact.path == selected_path && artifact.sha256 == selected)
        else {
            return Ok(false);
        };
        if selected_artifact.visibility != "path_sha_only"
            || crate::hashing::file_sha256(&repo_root.join(&selected_artifact.path))?.as_deref()
                != Some(selected)
        {
            return Ok(false);
        }
        let selected_json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(repo_root.join(&selected_artifact.path))?)?;
        let Some(selected_implementation) = selected_json
            .get("implementation_sha256")
            .and_then(|value| value.as_str())
        else {
            return Ok(false);
        };
        let selected_governed = selected_json.get("governed_inputs");
        for artifact in review.artifacts.iter().filter(|artifact| {
            matches!(artifact.visibility.as_str(), "shown" | "added_file")
                && artifact.path.starts_with("evidence/review/")
        }) {
            let path = repo_root.join(&artifact.path);
            if crate::hashing::file_sha256(&path)?.as_deref() != Some(&artifact.sha256) {
                continue;
            }
            let evidence: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
            let bindings = evidence.get("bindings");
            let production = evidence
                .get("verification")
                .and_then(|value| value.get("production_source"));
            let workflow = evidence
                .get("verification")
                .and_then(|value| value.get("workflow"));
            if bindings
                .and_then(|value| value.get("implementation_state"))
                .and_then(|value| value.as_str())
                == Some(selected_implementation)
                && production
                    .and_then(|value| value.get("evidence_sha256"))
                    .and_then(|value| value.as_str())
                    == Some(selected)
                && workflow
                    .and_then(|value| value.get("selected_production_evidence"))
                    .and_then(|value| value.as_str())
                    == Some(selected)
                && selected_governed.is_some_and(|governed| {
                    ["intent", "contract", "event_schema"].iter().all(|key| {
                        governed.get(key).and_then(|value| value.as_str())
                            == bindings
                                .and_then(|value| value.get(key))
                                .and_then(|value| value.as_str())
                    })
                })
            {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    if !review.review_id.ends_with("__TERMINAL-CLOSURE-RETRY-1") {
        return Ok(false);
    }
    let selected_path = format!(".codeos/06-workflow/reality-evidence/{selected}.json");
    let selected_artifact = review
        .artifacts
        .iter()
        .find(|artifact| artifact.path == selected_path && artifact.sha256 == selected);
    let Some(selected_artifact) = selected_artifact else {
        return Ok(false);
    };
    if selected_artifact.visibility != "path_sha_only"
        || crate::hashing::file_sha256(&repo_root.join(&selected_artifact.path))?.as_deref()
            != Some(selected)
    {
        return Ok(false);
    }
    let selected_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(repo_root.join(&selected_artifact.path))?)?;
    let selected_implementation = selected_json
        .get("implementation_sha256")
        .and_then(|value| value.as_str());
    let selected_governed = selected_json.get("governed_inputs");
    let Some(selected_implementation) = selected_implementation else {
        return Ok(false);
    };

    for correction_artifact in review
        .artifacts
        .iter()
        .filter(|artifact| artifact.visibility == "assessability_correction")
    {
        let correction_path = repo_root.join(&correction_artifact.path);
        if crate::hashing::file_sha256(&correction_path)?.as_deref()
            != Some(correction_artifact.sha256.as_str())
        {
            continue;
        }
        let correction: serde_json::Value =
            serde_json::from_slice(&std::fs::read(correction_path)?)?;
        if correction
            .get("classification")
            .and_then(|value| value.as_str())
            != Some("ASSESSABILITY_CORRECTION")
            || correction
                .get("production_evidence_sha256")
                .and_then(|value| value.as_str())
                != Some(selected)
            || correction
                .get("implementation_sha256")
                .and_then(|value| value.as_str())
                != Some(selected_implementation)
        {
            continue;
        }
        let Some(superseded_sha) = correction
            .get("superseded_review_evidence_sha256")
            .and_then(|value| value.as_str())
        else {
            continue;
        };
        let Some(superseded_artifact) = review
            .artifacts
            .iter()
            .find(|artifact| artifact.sha256 == superseded_sha && artifact.visibility == "shown")
        else {
            continue;
        };
        let superseded_path = repo_root.join(&superseded_artifact.path);
        if crate::hashing::file_sha256(&superseded_path)?.as_deref() != Some(superseded_sha) {
            continue;
        }
        let superseded: serde_json::Value =
            serde_json::from_slice(&std::fs::read(superseded_path)?)?;
        if superseded
            .get("implementation_sha256")
            .and_then(|value| value.as_str())
            != Some(selected_implementation)
            || superseded
                .pointer("/production/evidence_sha256")
                .and_then(|value| value.as_str())
                != Some(selected)
        {
            continue;
        }
        // Governed scope remains owned by the selected canonical evidence. A correction may not
        // introduce an alternative governed-input set.
        if correction
            .get("governed_inputs")
            .is_some_and(|value| Some(value) != selected_governed)
            || superseded
                .get("governed_inputs")
                .is_some_and(|value| Some(value) != selected_governed)
        {
            continue;
        }
        return Ok(true);
    }
    Ok(false)
}

pub fn evidence_reassessment_history_permits_progression(
    log_path: &Path,
    feature: &str,
    stage: &str,
    latest_review_id: &str,
) -> Result<bool> {
    let Ok(content) = std::fs::read_to_string(log_path) else {
        return Ok(false);
    };
    let prefix = format!("REV__{feature}__{stage}__EVIDENCE-REASSESSMENT-");
    if !latest_review_id.starts_with(&prefix) {
        return Ok(false);
    }
    let legacy = format!("REV__{feature}__{stage}__INFRA-REVALIDATION-1");
    let mut current = latest_review_id.to_string();
    let mut seen = std::collections::BTreeSet::new();
    let mut expected_findings: Option<String> = None;
    loop {
        if !seen.insert(current.clone()) {
            return Ok(false);
        }
        let Some((_, tail)) = content.split_once(&format!("Review ID: {current}")) else {
            return Ok(false);
        };
        let block = tail.split("\n## ").next().unwrap_or(tail);
        let legacy_mapping_defect = current == latest_review_id
            && block
                .lines()
                .any(|line| line == "Evidence loop state: CLOSED — SUBSTANTIVE")
            && block
                .lines()
                .any(|line| line == "Codex concern: NO OBJECTION")
            && block
                .lines()
                .any(|line| line == "Effective concern: CHANGES ADVISED")
            && block
                .lines()
                .any(|line| line == "Parse status: OK   Assessment status: COMPLETE");
        if block
            .lines()
            .any(|line| line == "Evidence loop state: CLOSED — SUBSTANTIVE")
            && !legacy_mapping_defect
        {
            return Ok(false);
        }
        let Some(findings) = block
            .lines()
            .find_map(|line| line.strip_prefix("Open evidence findings: "))
        else {
            return Ok(false);
        };
        if let Some(expected) = &expected_findings {
            if expected != findings {
                return Ok(false);
            }
        } else {
            expected_findings = Some(findings.to_string());
        }
        let Some(previous) = block
            .lines()
            .find_map(|line| line.strip_prefix("Evidence reassessment after: "))
        else {
            return Ok(false);
        };
        if current == latest_review_id
            && !block
                .lines()
                .any(|line| line == "Evidence loop state: CLOSED — EVIDENCE SATISFIED")
            && !legacy_mapping_defect
        {
            return Ok(false);
        }
        if previous == legacy {
            let Some((_, legacy_tail)) = content.split_once(&format!("Review ID: {legacy}")) else {
                return Ok(false);
            };
            let legacy_block = legacy_tail.split("\n## ").next().unwrap_or(legacy_tail);
            return Ok(legacy_block
                .lines()
                .any(|line| line == "Effective concern: DO NOT ADVANCE")
                && legacy_block
                    .lines()
                    .find_map(|line| line.strip_prefix("Authorized findings: "))
                    == expected_findings.as_deref());
        }
        if !previous.starts_with(&prefix) {
            return Ok(false);
        }
        current = previous.to_string();
    }
}

/// A review outcome permits progression unless it is an unresolved "DO NOT ADVANCE".
pub fn review_permits_progression(concern: &str) -> bool {
    !concern.trim().eq_ignore_ascii_case("DO NOT ADVANCE")
}

pub fn terminal_review_closure_history_permits_progression(
    log_path: &Path,
    feature: &str,
    stage: &str,
    closure_id: &str,
) -> Result<bool> {
    let Ok(content) = std::fs::read_to_string(log_path) else {
        return Ok(false);
    };
    let normal = closure_id.ends_with("__TERMINAL-CLOSURE-1");
    let legacy_retry = closure_id.ends_with("__TERMINAL-CLOSURE-RETRY-1");
    if !closure_id.starts_with(&format!("REV__{feature}__{stage}__"))
        || (!normal && !legacy_retry)
        || content.matches(&format!("Review ID: {closure_id}")).count() != 1
    {
        return Ok(false);
    }
    let Some((before_closure, closure_tail)) =
        content.split_once(&format!("Review ID: {closure_id}"))
    else {
        return Ok(false);
    };
    let closure = closure_tail.split("\n## ").next().unwrap_or(closure_tail);
    let structurally_complete = closure
        .lines()
        .any(|line| line == "Parse status: OK   Assessment status: COMPLETE");
    let explicitly_closed = closure
        .lines()
        .any(|line| line == "Effective concern: TERMINAL REVIEW CLOSED — PROGRESSION PERMITTED");
    let legacy_nonblocking_coverage_mapping = legacy_retry
        && closure
            .lines()
            .any(|line| line == "Evidence change classification: ASSESSABILITY_CORRECTION")
        && closure
            .lines()
            .any(|line| line == "Closure semantic disposition: OPEN")
        && closure
            .lines()
            .any(|line| line == "Codex concern: NO OBJECTION")
        && closure
            .lines()
            .any(|line| line == "Effective concern: CHANGES ADVISED")
        && closure.lines().any(|line| {
            line.starts_with("Coverage: SECRET_REDACTION")
                || line.starts_with("Coverage: PARTIAL_COVERAGE")
        });
    if !structurally_complete || (!explicitly_closed && !legacy_nonblocking_coverage_mapping) {
        return Ok(false);
    }
    let Some(terminal_id) = closure
        .lines()
        .find_map(|line| line.strip_prefix("Terminal closure after: "))
    else {
        return Ok(false);
    };
    if !terminal_id.ends_with("__REENTRY-1") {
        return Ok(false);
    }
    if legacy_retry {
        let legacy_id = closure_id
            .trim_end_matches("__TERMINAL-CLOSURE-RETRY-1")
            .to_string()
            + "__TERMINAL-CLOSURE-1";
        if before_closure
            .matches(&format!("Review ID: {legacy_id}"))
            .count()
            != 1
        {
            return Ok(false);
        }
        let Some((_, legacy_tail)) = before_closure.split_once(&format!("Review ID: {legacy_id}"))
        else {
            return Ok(false);
        };
        let legacy = legacy_tail.split("\n## ").next().unwrap_or(legacy_tail);
        if !(legacy.contains("Parse status: FAILED")
            || legacy.contains("Assessment status: INCOMPLETE"))
            || legacy.contains("Closure semantic disposition:")
        {
            return Ok(false);
        }
    }
    let Some((_, terminal_tail)) = before_closure.split_once(&format!("Review ID: {terminal_id}"))
    else {
        return Ok(false);
    };
    let terminal = terminal_tail.split("\n## ").next().unwrap_or(terminal_tail);
    if !terminal
        .lines()
        .any(|line| line == "Effective concern: DO NOT ADVANCE")
        && !terminal
            .lines()
            .any(|line| line.contains("Assessment status: INCOMPLETE"))
    {
        return Ok(false);
    }
    let authorization = before_closure.rsplit("\n## ").find(|block| {
        block
            .lines()
            .any(|line| line == "Decision: AUTHORIZE_TERMINAL_REVIEW_CLOSURE")
            && block
                .lines()
                .any(|line| line == format!("Terminal closure after: {terminal_id}"))
    });
    let Some(authorization) = authorization else {
        return Ok(false);
    };
    let auth_scope = authorization
        .lines()
        .find_map(|line| line.strip_prefix("Terminal closure scope SHA-256: "));
    let closure_scope = closure
        .lines()
        .find_map(|line| line.strip_prefix("Terminal closure scope SHA-256: "));
    let conditions = closure
        .lines()
        .find_map(|line| line.strip_prefix("Authorized conditions: "));
    Ok(auth_scope.is_some() && auth_scope == closure_scope && conditions.is_some())
}

pub fn lineage_replacement_history_permits_progression(
    log_path: &Path,
    feature: &str,
    stage: &str,
    replacement_id: &str,
) -> Result<bool> {
    let Ok(content) = std::fs::read_to_string(log_path) else {
        return Ok(false);
    };
    if !replacement_id.starts_with(&format!("REV__{feature}__{stage}__"))
        || !replacement_id.ends_with("__LINEAGE-REPLACEMENT-1")
        || content
            .matches(&format!("Review ID: {replacement_id}"))
            .count()
            != 1
    {
        return Ok(false);
    }
    let (_, tail) = content
        .split_once(&format!("Review ID: {replacement_id}"))
        .context("replacement record missing")?;
    let replacement = tail.split("\n## ").next().unwrap_or(tail);
    if !replacement
        .lines()
        .any(|line| line == "Parse status: OK   Assessment status: COMPLETE")
        || !replacement.lines().any(|line| {
            line == "Effective concern: LINEAGE REPLACEMENT CLOSED — PROGRESSION PERMITTED"
        })
    {
        return Ok(false);
    }
    let target = content.split("\n## ").find_map(|block| {
        if block
            .lines()
            .any(|line| line == "Decision: AUTHORIZE_LINEAGE_REPLACEMENT")
        {
            block
                .lines()
                .find_map(|line| line.strip_prefix("Lineage replacement after: "))
        } else {
            None
        }
    });
    let Some(target) = target else {
        return Ok(false);
    };
    Ok(content.contains(&format!(
        "Review applicability target: {target}\nApplicability: INVALID_REVIEW_INPUT_LINEAGE"
    )) && content.contains(&format!("Review ID: {target}"))
        && content.contains("Applicability record SHA-256:"))
}

pub fn lineage_replacement_correction_history_permits_progression(
    log_path: &Path,
    feature: &str,
    stage: &str,
    correction_id: &str,
) -> Result<bool> {
    let Ok(content) = std::fs::read_to_string(log_path) else {
        return Ok(false);
    };
    if !correction_id.starts_with(&format!("REV__{feature}__{stage}__"))
        || !correction_id.ends_with("__LINEAGE-REPLACEMENT-CORRECTION-1")
        || content
            .matches(&format!("Review ID: {correction_id}"))
            .count()
            != 1
    {
        return Ok(false);
    }
    let (_, tail) = content
        .split_once(&format!("Review ID: {correction_id}"))
        .context("correction assessment record missing")?;
    let correction = tail.split("\n## ").next().unwrap_or(tail);
    if !correction
        .lines()
        .any(|line| line == "Parse status: OK   Assessment status: COMPLETE")
        || !correction.lines().any(|line| {
            line == "Effective concern: LINEAGE REPLACEMENT CORRECTION CLOSED — PROGRESSION PERMITTED"
        })
    {
        return Ok(false);
    }
    let failed_id = correction_id.trim_end_matches("-CORRECTION-1").to_string() + "-1";
    if !content.contains(&format!("Review ID: {failed_id}"))
        || !content.contains(&format!(
            "Correction eligibility target: {failed_id}\nEligibility: ASSESSMENT_INPUT_CORRECTION"
        ))
        || !content.contains(&format!("Assessment input correction after: {failed_id}"))
        || !content.contains("Correction record SHA-256:")
    {
        return Ok(false);
    }
    Ok(true)
}

/// Validate the immutable composite history required by Review Policy v5. This is deliberately
/// stricter than treating the closure assessment as a replacement for REENTRY-1.
pub fn evidence_closure_history_permits_progression(
    log_path: &Path,
    feature: &str,
    stage: &str,
) -> Result<bool> {
    let Ok(content) = std::fs::read_to_string(log_path) else {
        return Ok(false);
    };
    let ordinary_suffix = format!(" REVIEW — {feature} — Stage {stage}");
    if content
        .lines()
        .filter(|l| l.starts_with("## ") && l.ends_with(&ordinary_suffix))
        .count()
        < 3
    {
        return Ok(false);
    }
    let reentry_id = format!("REV__{feature}__{stage}__REENTRY-1");
    let closure_id = format!("REV__{feature}__{stage}__EVIDENCE-CLOSURE-1");
    let Some((_, after_reentry)) = content.split_once(&format!("Review ID: {reentry_id}")) else {
        return Ok(false);
    };
    let reentry_block = after_reentry.split("\n## ").next().unwrap_or(after_reentry);
    if !reentry_block
        .lines()
        .any(|l| l == "Effective concern: DO NOT ADVANCE")
    {
        return Ok(false);
    }
    let Some((before_closure, after_closure)) =
        content.split_once(&format!("Review ID: {closure_id}"))
    else {
        return Ok(false);
    };
    let closure_block = after_closure.split("\n## ").next().unwrap_or(after_closure);
    if !closure_block
        .lines()
        .any(|l| l == "Effective concern: EVIDENCE CLOSED — PROGRESSION PERMITTED")
        || !closure_block
            .lines()
            .any(|l| l == format!("Evidence closure after: {reentry_id}"))
    {
        return Ok(false);
    }
    let authorization = before_closure.rsplit("\n## ").find(|block| {
        block
            .lines()
            .any(|l| l == "Decision: AUTHORIZE_EVIDENCE_CLOSURE")
            && block
                .lines()
                .any(|l| l == format!("Evidence closure after: {reentry_id}"))
    });
    let Some(auth) = authorization else {
        return Ok(false);
    };
    let auth_findings = auth
        .lines()
        .find_map(|l| l.strip_prefix("Authorized findings: "));
    let closure_findings = closure_block
        .lines()
        .find_map(|l| l.strip_prefix("Authorized findings: "));
    Ok(auth_findings.is_some() && auth_findings == closure_findings)
}

pub fn infrastructure_revalidation_history_permits_progression(
    log_path: &Path,
    feature: &str,
    stage: &str,
) -> Result<bool> {
    let Ok(content) = std::fs::read_to_string(log_path) else {
        return Ok(false);
    };
    let reentry_id = format!("REV__{feature}__{stage}__REENTRY-1");
    let closure_id = format!("REV__{feature}__{stage}__EVIDENCE-CLOSURE-1");
    let terminal_id = format!("REV__{feature}__{stage}__INFRA-REVALIDATION-1");
    let block_after = |id: &str| {
        content
            .split_once(&format!("Review ID: {id}"))
            .map(|(_, tail)| tail.split("\n## ").next().unwrap_or(tail))
    };
    if !block_after(&reentry_id).is_some_and(|block| {
        block
            .lines()
            .any(|line| line == "Effective concern: DO NOT ADVANCE")
    }) || !block_after(&closure_id).is_some_and(|block| {
        block
            .lines()
            .any(|line| line == "Effective concern: DO NOT ADVANCE")
    }) {
        return Ok(false);
    }
    let Some((before_terminal, terminal_tail)) =
        content.split_once(&format!("Review ID: {terminal_id}"))
    else {
        return Ok(false);
    };
    let terminal = terminal_tail.split("\n## ").next().unwrap_or(terminal_tail);
    if !terminal.lines().any(|line| {
        line == "Effective concern: INFRASTRUCTURE EVIDENCE REVALIDATED — PROGRESSION PERMITTED"
    }) || !terminal
        .lines()
        .any(|line| line == format!("Infrastructure revalidation after: {closure_id}"))
    {
        return Ok(false);
    }
    let terminal_findings = terminal
        .lines()
        .find_map(|line| line.strip_prefix("Authorized findings: "));
    let terminal_evidence = terminal
        .lines()
        .find_map(|line| line.strip_prefix("Infrastructure repair evidence: "));
    let authorization = before_terminal.rsplit("\n## ").find(|block| {
        block
            .lines()
            .any(|line| line == "Decision: AUTHORIZE_INFRASTRUCTURE_REVALIDATION")
            && block
                .lines()
                .any(|line| line == format!("Infrastructure revalidation after: {closure_id}"))
    });
    let Some(authorization) = authorization else {
        return Ok(false);
    };
    Ok(terminal_findings.is_some()
        && terminal_findings
            == authorization
                .lines()
                .find_map(|line| line.strip_prefix("Authorized findings: "))
        && terminal_evidence.is_some()
        && terminal_evidence
            == authorization
                .lines()
                .find_map(|line| line.strip_prefix("Infrastructure repair evidence: ")))
}

/// Result of live-executing a verification command.
pub struct CommandOutcome {
    pub ok: bool,
    /// The command line that was run (empty when nothing could be executed).
    pub command: String,
    pub summary: String,
    pub details: Option<serde_json::Value>,
}

/// Run a shell command from `cwd`. Used only by `check`, never by `status`/`next`.
pub fn run(cwd: &Path, program: &str, args: &[&str]) -> CommandOutcome {
    run_env(cwd, program, args, &[])
}

/// As `run`, with extra environment variables.
pub fn run_env(cwd: &Path, program: &str, args: &[&str], env: &[(&str, &str)]) -> CommandOutcome {
    let mut cmd = Command::new(program);
    cmd.args(args).current_dir(cwd);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let command = format!("{program} {}", args.join(" "));
    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Prefer the most informative test-harness lines (a `test result:` that recorded
            // work, or any failure line) over the raw final lines, which for `cargo test` are
            // an empty trailing doctest summary.
            let salient: Vec<&str> = stdout
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    (t.starts_with("test result:") && !t.contains("0 passed; 0 failed"))
                        || t.starts_with("FAILED")
                        || t.contains("panicked")
                        || t.ends_with("FAILED")
                })
                .collect();
            let tail: String = if salient.is_empty() {
                stdout
                    .lines()
                    .rev()
                    .take(3)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join(" | ")
            } else {
                salient
                    .iter()
                    .rev()
                    .take(3)
                    .rev()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" | ")
            };
            CommandOutcome {
                ok: out.status.success(),
                summary: format!(
                    "{command} -> {}{}",
                    if out.status.success() { "ok" } else { "FAILED" },
                    if tail.is_empty() {
                        String::new()
                    } else {
                        format!(" ({tail})")
                    }
                ),
                command,
                details: None,
            }
        }
        Err(e) => CommandOutcome {
            ok: false,
            summary: format!("{command}: {e}"),
            command,
            details: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn frontmatter_split() {
        let (fm, body) = split_frontmatter("---\na: 1\n---\nhello\n");
        assert_eq!(fm.unwrap().trim(), "a: 1");
        assert_eq!(body.trim(), "hello");
        let (fm, _) = split_frontmatter("no frontmatter here");
        assert!(fm.is_none());
    }

    #[test]
    fn spec_approval_requires_all_three_fields() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("c.md");
        fs::write(&p, "---\nstatus: DRAFT\napproved_by:\napproved_at:\n---\nx").unwrap();
        assert!(!spec_artifact_approved(&p).unwrap());
        fs::write(
            &p,
            "---\nstatus: APPROVED\napproved_by: Primoz Gorjup\napproved_at: 2026-09-05\n---\nx",
        )
        .unwrap();
        assert!(spec_artifact_approved(&p).unwrap());
    }

    #[test]
    fn charter_approval_detection() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("charter.md");
        fs::write(&p, "---\napproval: null\n---\nx").unwrap();
        assert!(!approval_recorded(&p).unwrap());
        fs::write(
            &p,
            "---\napproval:\n  by: Primoz Gorjup\n  at: 2026-09-05\n---\nx",
        )
        .unwrap();
        assert!(approval_recorded(&p).unwrap());
    }

    #[test]
    fn review_progression_rule() {
        assert!(review_permits_progression("NO OBJECTION"));
        assert!(review_permits_progression("CHANGES ADVISED"));
        assert!(!review_permits_progression("DO NOT ADVANCE"));
    }

    fn correction_lineage_fixture() -> (tempfile::TempDir, ReviewRecord, String) {
        let dir = tempfile::tempdir().unwrap();
        let selected_body = serde_json::json!({
            "implementation_sha256": "implementation",
            "governed_inputs": {"intent": "intent", "contract": "contract"}
        });
        let selected_bytes = serde_json::to_vec(&selected_body).unwrap();
        let selected = crate::hashing::bytes_sha256(&selected_bytes);
        let selected_path = format!(".codeos/06-workflow/reality-evidence/{selected}.json");
        fs::create_dir_all(dir.path().join(".codeos/06-workflow/reality-evidence")).unwrap();
        fs::write(dir.path().join(&selected_path), &selected_bytes).unwrap();

        let superseded = serde_json::json!({
            "implementation_sha256": "implementation",
            "production": {"evidence_sha256": selected}
        });
        let superseded_bytes = serde_json::to_vec(&superseded).unwrap();
        let superseded_sha = crate::hashing::bytes_sha256(&superseded_bytes);
        let superseded_path = "evidence/review/compact.json";
        fs::create_dir_all(dir.path().join("evidence/review")).unwrap();
        fs::write(dir.path().join(superseded_path), &superseded_bytes).unwrap();

        let correction = serde_json::json!({
            "classification": "ASSESSABILITY_CORRECTION",
            "implementation_sha256": "implementation",
            "production_evidence_sha256": selected,
            "superseded_review_evidence_sha256": superseded_sha
        });
        let correction_bytes = serde_json::to_vec(&correction).unwrap();
        let correction_sha = crate::hashing::bytes_sha256(&correction_bytes);
        let correction_path = "evidence/review/correction.json";
        fs::write(dir.path().join(correction_path), &correction_bytes).unwrap();

        let record = ReviewRecord {
            review_id: "REV__F-1__8__SR2__TERMINAL-CLOSURE-RETRY-1".into(),
            review_commit: "commit".into(),
            reviewed_packet_sha256: "packet".into(),
            effective_concern: "TERMINAL REVIEW CLOSED — PROGRESSION PERMITTED".into(),
            production_evidence: None,
            manifest_sidecar: None,
            artifacts: vec![
                ReviewArtifact {
                    path: selected_path,
                    sha256: selected.clone(),
                    visibility: "path_sha_only".into(),
                },
                ReviewArtifact {
                    path: superseded_path.into(),
                    sha256: superseded_sha,
                    visibility: "shown".into(),
                },
                ReviewArtifact {
                    path: correction_path.into(),
                    sha256: correction_sha,
                    visibility: "assessability_correction".into(),
                },
            ],
        };
        (dir, record, selected)
    }

    #[test]
    fn production_lineage_accepts_direct_and_authorized_correction_bindings() {
        let (dir, mut record, selected) = correction_lineage_fixture();
        assert!(review_binds_production_evidence(dir.path(), &record, &selected).unwrap());
        record.production_evidence = Some(selected.clone());
        record.artifacts.clear();
        assert!(review_binds_production_evidence(dir.path(), &record, &selected).unwrap());
    }

    #[test]
    fn production_lineage_rejects_unrelated_or_broken_supersession() {
        let (dir, mut record, selected) = correction_lineage_fixture();
        let correction = record
            .artifacts
            .iter_mut()
            .find(|artifact| artifact.visibility == "assessability_correction")
            .unwrap();
        correction.sha256 = "0".repeat(64);
        assert!(!review_binds_production_evidence(dir.path(), &record, &selected).unwrap());

        let (dir, mut record, selected) = correction_lineage_fixture();
        record
            .artifacts
            .iter_mut()
            .find(|artifact| artifact.visibility == "shown")
            .unwrap()
            .sha256 = "1".repeat(64);
        assert!(!review_binds_production_evidence(dir.path(), &record, &selected).unwrap());
    }

    #[test]
    fn production_lineage_rejects_changed_implementation_or_governed_scope() {
        let (dir, record, selected) = correction_lineage_fixture();
        let correction_path = dir.path().join("evidence/review/correction.json");
        let mut correction: serde_json::Value =
            serde_json::from_slice(&fs::read(&correction_path).unwrap()).unwrap();
        correction["implementation_sha256"] = "changed".into();
        fs::write(&correction_path, serde_json::to_vec(&correction).unwrap()).unwrap();
        assert!(!review_binds_production_evidence(dir.path(), &record, &selected).unwrap());

        let (dir, mut record, selected) = correction_lineage_fixture();
        let correction_path = dir.path().join("evidence/review/correction.json");
        let mut correction: serde_json::Value =
            serde_json::from_slice(&fs::read(&correction_path).unwrap()).unwrap();
        correction["governed_inputs"] = serde_json::json!({"intent": "different"});
        let bytes = serde_json::to_vec(&correction).unwrap();
        fs::write(&correction_path, &bytes).unwrap();
        record
            .artifacts
            .iter_mut()
            .find(|artifact| artifact.visibility == "assessability_correction")
            .unwrap()
            .sha256 = crate::hashing::bytes_sha256(&bytes);
        assert!(!review_binds_production_evidence(dir.path(), &record, &selected).unwrap());
    }

    fn review_record(feature: &str, stage: &str, id: &str) -> String {
        format!(
            "---\nreview_id: {id}\nreviewed:\n  feature: {feature}\n  stage: \"{stage}\"\n  review_commit: abc\n  reviewed_packet_sha256: packet\neffective_concern: NO OBJECTION\n---\n"
        )
    }

    fn canonical_review_record(feature: &str, stage: u64, id: &str) -> String {
        format!(
            "---\nreview_id: {id}\nreviewed:\n  feature: {feature}\n  stage: {stage}\n  review_commit: abc\n  reviewed_packet_sha256: packet\n  effective_concern: NO OBJECTION\n---\n"
        )
    }

    #[test]
    fn review_record_is_found_at_root_or_canonical_codex_location() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("20260101T000000Z-F-ROOT-stage-8-abc.md"),
            review_record("F-ROOT", "8", "REV__F-ROOT__8__R1"),
        )
        .unwrap();
        let codex = dir.path().join("codex");
        fs::create_dir(&codex).unwrap();
        fs::write(
            codex.join("20260101T000001Z-F-NESTED-stage-8-abc.md"),
            canonical_review_record("F-NESTED", 8, "REV__F-NESTED__8__R1"),
        )
        .unwrap();

        assert!(latest_review_record(dir.path(), "F-ROOT", "8")
            .unwrap()
            .is_some());
        assert!(latest_review_record(dir.path(), "F-NESTED", "8")
            .unwrap()
            .is_some());
    }

    #[test]
    fn reentry_record_uses_normal_canonical_discovery() {
        let dir = tempfile::tempdir().unwrap();
        let codex = dir.path().join("codex");
        fs::create_dir(&codex).unwrap();
        fs::write(
            codex.join("20260101T000001Z-F-0004-stage-8-abc.md"),
            canonical_review_record("F-0004", 8, "REV__F-0004__8__REENTRY-1"),
        )
        .unwrap();
        let record = latest_review_record(dir.path(), "F-0004", "8")
            .unwrap()
            .unwrap();
        assert_eq!(record.effective_concern, "NO OBJECTION");
    }

    #[test]
    fn evidence_reassessment_requires_closed_chain_back_to_negative_terminal_record() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("review-log.md");
        std::fs::write(&log, "## t INFRASTRUCTURE REVALIDATION — F-1 — Stage 8\nReview ID: REV__F-1__8__INFRA-REVALIDATION-1\nAuthorized findings: runtime-evidence\nEffective concern: DO NOT ADVANCE\n\n## t EVIDENCE REASSESSMENT — F-1 — Stage 8\nReview ID: REV__F-1__8__EVIDENCE-REASSESSMENT-1\nEvidence reassessment after: REV__F-1__8__INFRA-REVALIDATION-1\nOpen evidence findings: runtime-evidence\nEvidence loop state: CLOSED — EVIDENCE SATISFIED\nEffective concern: NO OBJECTION\n").unwrap();
        assert!(evidence_reassessment_history_permits_progression(
            &log,
            "F-1",
            "8",
            "REV__F-1__8__EVIDENCE-REASSESSMENT-1"
        )
        .unwrap());

        let text = std::fs::read_to_string(&log)
            .unwrap()
            .replace("CLOSED — EVIDENCE SATISFIED", "CLOSED — SUBSTANTIVE");
        std::fs::write(&log, text).unwrap();
        assert!(!evidence_reassessment_history_permits_progression(
            &log,
            "F-1",
            "8",
            "REV__F-1__8__EVIDENCE-REASSESSMENT-1"
        )
        .unwrap());

        std::fs::write(&log, "## t INFRASTRUCTURE REVALIDATION — F-1 — Stage 8\nReview ID: REV__F-1__8__INFRA-REVALIDATION-1\nAuthorized findings: runtime-evidence\nEffective concern: DO NOT ADVANCE\n\n## t EVIDENCE REASSESSMENT — F-1 — Stage 8\nReview ID: REV__F-1__8__EVIDENCE-REASSESSMENT-1\nEvidence reassessment after: REV__F-1__8__INFRA-REVALIDATION-1\nOpen evidence findings: runtime-evidence\nEvidence loop state: CLOSED — SUBSTANTIVE\nCodex concern: NO OBJECTION\nEffective concern: CHANGES ADVISED\nParse status: OK   Assessment status: COMPLETE\n").unwrap();
        assert!(evidence_reassessment_history_permits_progression(
            &log,
            "F-1",
            "8",
            "REV__F-1__8__EVIDENCE-REASSESSMENT-1"
        )
        .unwrap());
    }

    #[test]
    fn irrelevant_files_are_ignored_and_duplicate_canonical_names_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("notes.md"), "not a review").unwrap();
        fs::write(dir.path().join("packet.txt"), "not a review").unwrap();
        assert!(latest_review_record(dir.path(), "F-0004", "8")
            .unwrap()
            .is_none());

        let name = "20260101T000001Z-F-0004-stage-8-abc.md";
        fs::write(
            dir.path().join(name),
            review_record("F-0004", "8", "REV__F-0004__8__R3"),
        )
        .unwrap();
        let codex = dir.path().join("codex");
        fs::create_dir(&codex).unwrap();
        fs::write(
            codex.join(name),
            review_record("F-0004", "8", "REV__F-0004__8__REENTRY-1"),
        )
        .unwrap();
        let error = latest_review_record(dir.path(), "F-0004", "8").unwrap_err();
        assert!(error.to_string().contains("ambiguous reviewer records"));
    }

    #[test]
    fn evidence_closure_requires_the_complete_composite_chain() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("review-log.md");
        let text = "## t REVIEW — F-1 — Stage 8\n## t REVIEW — F-1 — Stage 8\n## t REVIEW — F-1 — Stage 8\nReview ID: REV__F-1__8__REENTRY-1\nEffective concern: DO NOT ADVANCE\n## t HUMAN DECISION — F-1 — Stage 8\nDecision: AUTHORIZE_EVIDENCE_CLOSURE\nEvidence closure after: REV__F-1__8__REENTRY-1\nAuthorized findings: runtime-evidence\n## t EVIDENCE CLOSURE REVIEW — F-1 — Stage 8\nReview ID: REV__F-1__8__EVIDENCE-CLOSURE-1\nEvidence closure after: REV__F-1__8__REENTRY-1\nAuthorized findings: runtime-evidence\nEffective concern: EVIDENCE CLOSED — PROGRESSION PERMITTED\n";
        fs::write(&log, text).unwrap();
        assert!(evidence_closure_history_permits_progression(&log, "F-1", "8").unwrap());
        fs::write(
            &log,
            text.replace(
                "Authorized findings: runtime-evidence\nEffective concern",
                "Authorized findings: other\nEffective concern",
            ),
        )
        .unwrap();
        assert!(!evidence_closure_history_permits_progression(&log, "F-1", "8").unwrap());
    }

    #[test]
    fn terminal_closure_requires_exact_authorized_composite_history() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("review-log.md");
        let terminal = "REV__F-1__8__SR2__REENTRY-1";
        let closure = "REV__F-1__8__SR2__TERMINAL-CLOSURE-1";
        std::fs::write(&log, format!(
            "## t RE-ENTRY REVIEW — F-1 — Stage 8\nReview ID: {terminal}\nEffective concern: DO NOT ADVANCE\nParse status: OK   Assessment status: COMPLETE\n\n## t HUMAN DECISION — F-1 — Stage 8\nDecision: AUTHORIZE_TERMINAL_REVIEW_CLOSURE\nTerminal closure after: {terminal}\nTerminal closure scope SHA-256: abc\n\n## t TERMINAL REVIEW CLOSURE — F-1 — Stage 8\nReview ID: {closure}\nTerminal closure after: {terminal}\nTerminal closure scope SHA-256: abc\nAuthorized conditions: C1\nEffective concern: TERMINAL REVIEW CLOSED — PROGRESSION PERMITTED\nParse status: OK   Assessment status: COMPLETE\n"
        )).unwrap();
        assert!(
            terminal_review_closure_history_permits_progression(&log, "F-1", "8", closure).unwrap()
        );
        let bad = std::fs::read_to_string(&log).unwrap().replace(
            "Terminal closure scope SHA-256: abc\nAuthorized conditions",
            "Terminal closure scope SHA-256: different\nAuthorized conditions",
        );
        std::fs::write(&log, bad).unwrap();
        assert!(
            !terminal_review_closure_history_permits_progression(&log, "F-1", "8", closure)
                .unwrap()
        );
    }

    #[test]
    fn structural_closure_failure_is_not_a_semantic_review_record() {
        let dir = tempfile::tempdir().unwrap();
        let codex = dir.path().join("codex");
        fs::create_dir(&codex).unwrap();
        fs::write(
            codex.join("2026-01-01T000000Z-F-1-stage-8-abc.md"),
            review_record("F-1", "8", "REV__F-1__8__SR2__REENTRY-1"),
        )
        .unwrap();
        fs::write(
            codex.join("2026-01-01T000001Z-F-1-stage-8-abc.md"),
            "---\nreview_id: REV__F-1__8__SR2__TERMINAL-CLOSURE-FAILURE-1\ncounts_as_semantic_closure: false\nreviewed:\n  feature: F-1\n  stage: 8\n  review_commit: abc\n  reviewed_packet_sha256: failed-packet\n  effective_concern: DO NOT ADVANCE\n---\n",
        )
        .unwrap();
        let latest = latest_review_record(dir.path(), "F-1", "8")
            .unwrap()
            .unwrap();
        assert_eq!(latest.review_id, "REV__F-1__8__SR2__REENTRY-1");
    }

    #[test]
    fn legacy_retry_requires_a_preserved_malformed_v12_closure() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("review-log.md");
        let terminal = "REV__F-1__8__SR2__REENTRY-1";
        let legacy = "REV__F-1__8__SR2__TERMINAL-CLOSURE-1";
        let retry = "REV__F-1__8__SR2__TERMINAL-CLOSURE-RETRY-1";
        let text = format!(
            "## t RE-ENTRY REVIEW — F-1 — Stage 8\nReview ID: {terminal}\nEffective concern: DO NOT ADVANCE\n\n## t HUMAN DECISION — F-1 — Stage 8\nDecision: AUTHORIZE_TERMINAL_REVIEW_CLOSURE\nTerminal closure after: {terminal}\nTerminal closure scope SHA-256: abc\n\n## t TERMINAL REVIEW CLOSURE — F-1 — Stage 8\nReview ID: {legacy}\nParse status: FAILED   Assessment status: INCOMPLETE\n\n## t TERMINAL REVIEW CLOSURE — F-1 — Stage 8\nReview ID: {retry}\nTerminal closure after: {terminal}\nTerminal closure scope SHA-256: abc\nAuthorized conditions: C1\nEffective concern: TERMINAL REVIEW CLOSED — PROGRESSION PERMITTED\nParse status: OK   Assessment status: COMPLETE\n"
        );
        fs::write(&log, &text).unwrap();
        assert!(
            terminal_review_closure_history_permits_progression(&log, "F-1", "8", retry).unwrap()
        );
        fs::write(&log, text.replace("Parse status: FAILED   Assessment status: INCOMPLETE", "Parse status: OK   Assessment status: COMPLETE\nClosure semantic disposition: OPEN")).unwrap();
        assert!(
            !terminal_review_closure_history_permits_progression(&log, "F-1", "8", retry).unwrap()
        );
    }

    #[test]
    fn legacy_retry_derives_closed_when_only_nonblocking_coverage_floor_caused_open_mapping() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("review-log.md");
        let terminal = "REV__F-1__8__SR2__REENTRY-1";
        let legacy = "REV__F-1__8__SR2__TERMINAL-CLOSURE-1";
        let retry = "REV__F-1__8__SR2__TERMINAL-CLOSURE-RETRY-1";
        let text = format!(
            "## t RE-ENTRY REVIEW — F-1 — Stage 8\nReview ID: {terminal}\nEffective concern: DO NOT ADVANCE\n\n## t HUMAN DECISION — F-1 — Stage 8\nDecision: AUTHORIZE_TERMINAL_REVIEW_CLOSURE\nTerminal closure after: {terminal}\nTerminal closure scope SHA-256: abc\n\n## t TERMINAL REVIEW CLOSURE — F-1 — Stage 8\nReview ID: {legacy}\nParse status: FAILED   Assessment status: INCOMPLETE\n\n## t TERMINAL REVIEW CLOSURE — F-1 — Stage 8\nReview ID: {retry}\nTerminal closure after: {terminal}\nTerminal closure scope SHA-256: abc\nAuthorized conditions: C1, C2, C3\nEvidence change classification: ASSESSABILITY_CORRECTION\nClosure semantic disposition: OPEN\nCodex concern: NO OBJECTION\nEffective concern: CHANGES ADVISED\nParse status: OK   Assessment status: COMPLETE\nCoverage: SECRET_REDACTION; redactions: 1\n"
        );
        fs::write(&log, &text).unwrap();
        assert!(
            terminal_review_closure_history_permits_progression(&log, "F-1", "8", retry).unwrap()
        );
        fs::write(
            &log,
            text.replace("Coverage: SECRET_REDACTION", "Coverage: CRITICAL_OMISSION"),
        )
        .unwrap();
        assert!(
            !terminal_review_closure_history_permits_progression(&log, "F-1", "8", retry).unwrap()
        );
    }

    #[test]
    fn lineage_replacement_correction_requires_the_complete_immutable_chain() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("review-log.md");
        let failed = "REV__F-2__8__SR3__LINEAGE-REPLACEMENT-1";
        let correction = "REV__F-2__8__SR3__LINEAGE-REPLACEMENT-CORRECTION-1";
        let text = format!(
            "## t REVIEW — F-2 — Stage 8\nReview ID: {failed}\nEffective concern: DO NOT ADVANCE\nParse status: OK   Assessment status: COMPLETE\n\n## t ASSESSMENT INPUT CORRECTION ELIGIBILITY — F-2 — Stage 8\nCorrection eligibility target: {failed}\nEligibility: ASSESSMENT_INPUT_CORRECTION\nCorrection record SHA-256: abc\n\n## t HUMAN DECISION — F-2 — Stage 8\nDecision: AUTHORIZE_ASSESSMENT_INPUT_CORRECTION\nAssessment input correction after: {failed}\nCorrection record SHA-256: abc\n\n## t REVIEW — F-2 — Stage 8\nReview ID: {correction}\nEffective concern: LINEAGE REPLACEMENT CORRECTION CLOSED — PROGRESSION PERMITTED\nParse status: OK   Assessment status: COMPLETE\n"
        );
        fs::write(&log, &text).unwrap();
        assert!(lineage_replacement_correction_history_permits_progression(
            &log, "F-2", "8", correction
        )
        .unwrap());
        fs::write(
            &log,
            text.replace(
                "Eligibility: ASSESSMENT_INPUT_CORRECTION",
                "Eligibility: DENIED",
            ),
        )
        .unwrap();
        assert!(!lineage_replacement_correction_history_permits_progression(
            &log, "F-2", "8", correction
        )
        .unwrap());
    }
}
