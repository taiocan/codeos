use crate::config::Config;
use crate::packet::{sha256_file, sha256_str, ReviewPacket};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const INVALID_APPLICABILITY: &str = "INVALID_REVIEW_INPUT_LINEAGE";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicabilityRecord {
    pub feature: String,
    pub stage: String,
    pub semantic_revision: u64,
    pub review_id: String,
    pub assessment_sha256: String,
    pub packet_sha256: String,
    pub f6_selected_production_evidence: String,
    pub reviewed_production_evidence: String,
    pub canonical_implementation_identity: String,
    pub claimed_implementation_identities: Vec<String>,
    pub reason_codes: Vec<String>,
}

impl ApplicabilityRecord {
    pub fn sha256(&self) -> Result<String> {
        Ok(sha256_str(&serde_json::to_string(self)?))
    }
}

#[derive(Debug, Deserialize)]
struct Receipt {
    workflow: String,
    subject: String,
    checkpoint: String,
    bindings: std::collections::BTreeMap<String, String>,
    timestamp: String,
}

fn block<'a>(log: &'a str, review_id: &str) -> Result<&'a str> {
    let marker = format!("Review ID: {review_id}");
    let (_, tail) = log
        .split_once(&marker)
        .with_context(|| format!("review {review_id} does not exist"))?;
    Ok(tail.split("\n## ").next().unwrap_or(tail))
}

fn line<'a>(block: &'a str, prefix: &str) -> Result<&'a str> {
    block
        .lines()
        .find_map(|value| value.strip_prefix(prefix))
        .with_context(|| format!("review record omits {prefix}"))
}

fn reference(block: &str, prefix: &str) -> Result<(PathBuf, String)> {
    let value = line(block, prefix)?;
    let (path, hash) = value
        .rsplit_once(" (sha256:")
        .with_context(|| format!("malformed {prefix}"))?;
    Ok((PathBuf::from(path), hash.trim_end_matches(')').to_string()))
}

fn frontmatter(path: &Path) -> Result<Value> {
    let text = std::fs::read_to_string(path)?;
    let yaml = text
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---"))
        .map(|(value, _)| value)
        .context("assessment has no YAML frontmatter")?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(yaml)?;
    Ok(serde_json::to_value(yaml)?)
}

fn review_timestamp(log: &str, review_id: &str) -> Result<String> {
    let marker = format!("Review ID: {review_id}");
    let before = log.split_once(&marker).context("review missing")?.0;
    let heading = before
        .lines()
        .rev()
        .find(|value| value.starts_with("## "))
        .context("review heading missing")?;
    Ok(heading
        .trim_start_matches("## ")
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string())
}

fn selected_f6(repo_root: &Path, feature: &str, before: &str) -> Result<String> {
    let path = repo_root.join(".codeos/06-workflow/decisions.jsonl");
    let mut selected = None;
    for raw in std::fs::read_to_string(path)?.lines() {
        let receipt: Receipt = serde_json::from_str(raw)?;
        if receipt.workflow == "feature"
            && receipt.subject == feature
            && receipt.checkpoint == "reconciliation"
            && receipt.timestamp.as_str() <= before
        {
            selected = receipt.bindings.get("production_evidence").cloned();
        }
    }
    selected.context("no authoritative pre-execution F6 production-evidence selection")
}

fn artifact_values(reviewed: &Value) -> Vec<(String, String)> {
    reviewed
        .get("artifacts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            Some((
                item.get("path")?.as_str()?.to_string(),
                item.get("sha256")?.as_str()?.to_string(),
            ))
        })
        .collect()
}

pub fn audit(
    cfg: &Config,
    feature: &str,
    stage: &str,
    review_id: &str,
) -> Result<ApplicabilityRecord> {
    anyhow::ensure!(stage == "8", "invalid-lineage recovery is Stage 8 only");
    anyhow::ensure!(
        matches!(
            crate::config::resolve_reviewer_tool_contract(cfg)?.as_str(),
            "dba/04-tools/reviewer/contract/v14.md"
                | "dba/04-tools/reviewer/contract/v15.md"
                | "dba/04-tools/reviewer/contract/v16.md"
        ),
        "invalid-lineage recovery requires Reviewer Tool Contract v14"
    );
    let log_text = std::fs::read_to_string(&cfg.review_log)?;
    anyhow::ensure!(
        !log_text.contains(&format!(
            "Review applicability target: {review_id}\nApplicability: {INVALID_APPLICABILITY}"
        )),
        "review already has an invalid-applicability record"
    );
    let record = block(&log_text, review_id)?;
    anyhow::ensure!(
        record.contains("Parse status: OK   Assessment status: COMPLETE"),
        "target assessment is not structurally complete"
    );
    anyhow::ensure!(
        record.contains("PROGRESSION PERMITTED"),
        "only a progression-permitting assessment may use invalid-lineage recovery"
    );
    let semantic_revision = review_id
        .split("__SR")
        .nth(1)
        .and_then(|value| value.split("__").next())
        .and_then(|value| value.parse::<u64>().ok())
        .context("target review has no semantic revision")?;
    let (assessment_path, assessment_sha256) = reference(record, "Full assessment: ")?;
    let (packet_path, packet_sha256) = reference(record, "Reviewed packet: ")?;
    anyhow::ensure!(
        sha256_file(assessment_path.to_string_lossy().as_ref())? == assessment_sha256,
        "assessment hash mismatch"
    );
    anyhow::ensure!(
        sha256_file(packet_path.to_string_lossy().as_ref())? == packet_sha256,
        "packet hash mismatch"
    );
    let assessment = frontmatter(&assessment_path)?;
    let reviewed = assessment
        .get("reviewed")
        .context("assessment omits reviewed inputs")?;
    anyhow::ensure!(
        reviewed.get("feature").and_then(Value::as_str) == Some(feature),
        "review belongs to another feature"
    );
    let review_time = review_timestamp(&log_text, review_id)?;
    let f6_selected = selected_f6(&cfg.repo_root, feature, &review_time)?;
    let artifacts = artifact_values(reviewed);
    let reviewed_production = artifacts
        .iter()
        .find(|(path, hash)| path == &format!(".codeos/06-workflow/reality-evidence/{hash}.json"))
        .map(|(_, hash)| hash.clone())
        .context("review packet contains no canonical production evidence")?;
    let production_path = cfg.repo_root.join(format!(
        ".codeos/06-workflow/reality-evidence/{reviewed_production}.json"
    ));
    anyhow::ensure!(
        sha256_file(production_path.to_string_lossy().as_ref())? == reviewed_production,
        "reviewed production evidence hash mismatch"
    );
    let production: Value = serde_json::from_slice(&std::fs::read(&production_path)?)?;
    let canonical_implementation = production
        .get("implementation_sha256")
        .and_then(Value::as_str)
        .context("production evidence omits implementation identity")?
        .to_string();
    let mut claimed = BTreeSet::new();
    for (path, hash) in &artifacts {
        if !path.starts_with("evidence/review/") || !path.ends_with(".json") {
            continue;
        }
        let full = cfg.repo_root.join(path);
        if !full.is_file() || sha256_file(full.to_string_lossy().as_ref())? != *hash {
            continue;
        }
        let value: Value = serde_json::from_slice(&std::fs::read(full)?)?;
        if let Some(identity) = value
            .pointer("/bindings/implementation_state")
            .and_then(Value::as_str)
        {
            claimed.insert(identity.to_string());
        }
    }
    let mut reasons = Vec::new();
    if reviewed_production != f6_selected {
        reasons.push("F6_EVIDENCE_MISMATCH".to_string());
    }
    if claimed
        .iter()
        .any(|identity| identity != &canonical_implementation)
    {
        reasons.push("UNSUPPORTED_IMPLEMENTATION_IDENTITY".to_string());
    }
    anyhow::ensure!(
        !reasons.is_empty(),
        "review input lineage is mechanically coherent"
    );
    Ok(ApplicabilityRecord {
        feature: feature.to_string(),
        stage: stage.to_string(),
        semantic_revision,
        review_id: review_id.to_string(),
        assessment_sha256,
        packet_sha256,
        f6_selected_production_evidence: f6_selected,
        reviewed_production_evidence: reviewed_production,
        canonical_implementation_identity: canonical_implementation,
        claimed_implementation_identities: claimed.into_iter().collect(),
        reason_codes: reasons,
    })
}

pub fn append_audit(cfg: &Config, record: &ApplicabilityRecord) -> Result<String> {
    let record_json = serde_json::to_string(record)?;
    let sha = sha256_str(&record_json);
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let entry = format!(
        "\n## {ts} REVIEW APPLICABILITY — {} — Stage {}\nReview applicability target: {}\nApplicability: {INVALID_APPLICABILITY}\nApplicability record SHA-256: {sha}\nApplicability record JSON: {record_json}\nSemantic revision: {}\nAssessment SHA-256: {}\nReviewed packet SHA-256: {}\nF6-selected production evidence: {}\nReviewed production evidence: {}\nCanonical implementation identity: {}\nClaimed implementation identities: {}\nReason codes: {}\n",
        record.feature,
        record.stage,
        record.review_id,
        record.semantic_revision,
        record.assessment_sha256,
        record.packet_sha256,
        record.f6_selected_production_evidence,
        record.reviewed_production_evidence,
        record.canonical_implementation_identity,
        record.claimed_implementation_identities.join(", "),
        record.reason_codes.join(", "),
    );
    crate::log::append_raw(&cfg.review_log, &entry)?;
    Ok(sha)
}

pub fn applicability_record(log: &str, review_id: &str) -> Result<(String, String)> {
    let marker = format!("Review applicability target: {review_id}");
    let (_, tail) = log
        .split_once(&marker)
        .context("no invalid-applicability record exists")?;
    let record = tail.split("\n## ").next().unwrap_or(tail);
    anyhow::ensure!(
        record
            .lines()
            .any(|line| line == format!("Applicability: {INVALID_APPLICABILITY}")),
        "target review is not invalidated"
    );
    let sha = line(record, "Applicability record SHA-256: ")?.to_string();
    let raw = line(record, "Applicability record JSON: ")?;
    anyhow::ensure!(sha256_str(raw) == sha, "applicability record hash mismatch");
    let parsed: ApplicabilityRecord = serde_json::from_str(raw)?;
    anyhow::ensure!(
        parsed.review_id == review_id,
        "applicability record targets another review"
    );
    anyhow::ensure!(
        parsed.sha256()? == sha,
        "applicability record is not canonical"
    );
    Ok((record.to_string(), sha))
}

pub fn resolve_authorization(
    log: &str,
    feature: &str,
    stage: &str,
    review_id: &str,
) -> Result<(String, String)> {
    let (record, record_sha) = applicability_record(log, review_id)?;
    let blocks = log.split("\n## ").collect::<Vec<_>>();
    let auth = blocks
        .iter()
        .rev()
        .find(|block| {
            block
                .lines()
                .any(|line| line == "Decision: AUTHORIZE_LINEAGE_REPLACEMENT")
                && block
                    .lines()
                    .any(|line| line == format!("Lineage replacement after: {review_id}"))
                && block
                    .lines()
                    .any(|line| line == format!("Applicability record SHA-256: {record_sha}"))
        })
        .context("no valid lineage-replacement authorization exists")?;
    anyhow::ensure!(
        auth.contains(&format!("— {feature} — Stage {stage}")),
        "authorization belongs to another feature or stage"
    );
    let replacement_prefix = format!("Review ID: REV__{feature}__{stage}__");
    anyhow::ensure!(
        !log.lines().any(|line| line.starts_with(&replacement_prefix)
            && line.ends_with("__LINEAGE-REPLACEMENT-1")),
        "lineage replacement already consumed"
    );
    let after_auth = log
        .split_once(auth)
        .map(|(_, tail)| tail)
        .unwrap_or_default();
    anyhow::ensure!(
        !after_auth.split("\n## ").any(|block| block
            .contains(&format!("— {feature} — Stage {stage}"))
            && block.lines().any(|line| line == "Decision: STOP")),
        "a later STOP denies lineage replacement"
    );
    Ok((record, auth.to_string()))
}

pub fn replacement_id(feature: &str, revision: u64) -> String {
    format!("REV__{feature}__8__SR{revision}__LINEAGE-REPLACEMENT-1")
}

pub fn validate_current_lineage(
    cfg: &Config,
    feature: &str,
    packet: &ReviewPacket,
) -> Result<Option<String>> {
    let decisions =
        std::fs::read_to_string(cfg.repo_root.join(".codeos/06-workflow/decisions.jsonl"))?;
    let mut selected = None;
    for raw in decisions.lines() {
        let receipt: Receipt = serde_json::from_str(raw)?;
        if receipt.workflow == "feature"
            && receipt.subject == feature
            && receipt.checkpoint == "reconciliation"
        {
            selected = receipt.bindings.get("production_evidence").cloned();
        }
    }
    let has_production_artifact = packet.artifacts.iter().any(|artifact| {
        artifact
            .path
            .starts_with(".codeos/06-workflow/reality-evidence/")
    });
    let contract = crate::config::resolve_reviewer_tool_contract(cfg)?;
    if contract == "dba/04-tools/reviewer/contract/v16.md" {
        let canonical_config = sha256_file(
            cfg.repo_root
                .join(".codeos/00-project/codeos.yaml")
                .to_string_lossy()
                .as_ref(),
        )?;
        let mut supplied = Vec::new();
        for artifact in packet.artifacts.iter().filter(|item| {
            item.path.starts_with("evidence/review/") && item.path.ends_with(".json")
        }) {
            let value: Value =
                serde_json::from_slice(&std::fs::read(cfg.repo_root.join(&artifact.path))?)?;
            supplied.push(value);
        }
        validate_v16_supplied_inputs(
            &canonical_config,
            &supplied,
            packet.artifacts.iter().any(|a| {
                a.path == ".codeos/06-workflow/verifications.jsonl" && a.visibility == "shown"
            }),
        )?;
    }
    if selected.is_none() && !has_production_artifact {
        return Ok(None);
    }
    let selected =
        selected.context("reality-dependent Stage 8 review requires an explicit F6 selection")?;
    let expected_path = format!(".codeos/06-workflow/reality-evidence/{selected}.json");
    let artifact = packet
        .artifacts
        .iter()
        .find(|artifact| artifact.path == expected_path && artifact.sha256 == selected)
        .context("replacement packet does not contain the exact F6-selected production evidence")?;
    anyhow::ensure!(
        artifact.visibility != "missing",
        "selected production evidence is unavailable"
    );
    let evidence_path = cfg.repo_root.join(&expected_path);
    anyhow::ensure!(
        sha256_file(evidence_path.to_string_lossy().as_ref())? == selected,
        "selected production evidence hash mismatch"
    );
    let evidence: Value = serde_json::from_slice(&std::fs::read(evidence_path)?)?;
    let implementation = evidence
        .get("implementation_sha256")
        .and_then(Value::as_str)
        .context("selected production evidence omits implementation identity")?;
    for artifact in packet
        .artifacts
        .iter()
        .filter(|item| item.path.starts_with("evidence/review/") && item.path.ends_with(".json"))
    {
        let value: Value =
            serde_json::from_slice(&std::fs::read(cfg.repo_root.join(&artifact.path))?)?;
        if let Some(claimed) = value
            .pointer("/bindings/implementation_state")
            .and_then(Value::as_str)
        {
            anyhow::ensure!(
                claimed == implementation,
                "review evidence asserts an unsupported implementation identity"
            );
        }
        if let Some(claimed) = value
            .pointer("/verification/production_source/evidence_sha256")
            .and_then(Value::as_str)
        {
            anyhow::ensure!(
                claimed == selected,
                "review evidence conflicts with the F6-selected production evidence"
            );
        }
    }
    Ok(Some(selected))
}

fn validate_v16_supplied_inputs(
    canonical_config: &str,
    supplied: &[Value],
    has_underlying_verifications: bool,
) -> Result<()> {
    let mut claims_verification_summary = false;
    for value in supplied {
        if let Some(claimed) = value
            .pointer("/bindings/codeos_yaml")
            .and_then(Value::as_str)
        {
            anyhow::ensure!(
                claimed == canonical_config,
                "review evidence codeos.yaml identity is not the canonical artifact hash"
            );
        }
        claims_verification_summary |= value.get("verification").is_some();
    }
    anyhow::ensure!(
        !claims_verification_summary || has_underlying_verifications,
        "summary-only verification evidence is prohibited; include underlying inspectable verification records"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture() -> (tempfile::TempDir, Config, String) {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".codeos/00-project")).unwrap();
        fs::create_dir_all(dir.path().join(".codeos/05-review/reviews/codex/packets")).unwrap();
        fs::create_dir_all(dir.path().join(".codeos/06-workflow/reality-evidence")).unwrap();
        fs::create_dir_all(dir.path().join("evidence/review")).unwrap();
        let toolkit = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .unwrap()
            .to_path_buf();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&toolkit, dir.path().join(".codeos/toolkit")).unwrap();
        fs::write(
            dir.path().join(".codeos/00-project/codeos.yaml"),
            "codeos_schema: 2\ndba_configuration: DBA-19\n",
        )
        .unwrap();
        let selected = "a".repeat(64);
        let reviewed_body = serde_json::json!({
            "implementation_sha256": "canonical-implementation",
            "governed_inputs": {"intent":"i","contract":"c","event_schema":"e"}
        });
        let reviewed_bytes = serde_json::to_vec(&reviewed_body).unwrap();
        let reviewed = sha256_str(std::str::from_utf8(&reviewed_bytes).unwrap());
        fs::write(
            dir.path().join(format!(
                ".codeos/06-workflow/reality-evidence/{reviewed}.json"
            )),
            &reviewed_bytes,
        )
        .unwrap();
        let manifest = serde_json::json!({"bindings":{"implementation_state":"unsupported"}});
        let manifest_text = serde_json::to_string(&manifest).unwrap();
        let manifest_sha = sha256_str(&manifest_text);
        fs::write(
            dir.path().join("evidence/review/manifest.json"),
            manifest_text,
        )
        .unwrap();
        let packet_path = dir
            .path()
            .join(".codeos/05-review/reviews/codex/packets/p.packet.txt");
        fs::write(&packet_path, "packet").unwrap();
        let assessment_path = dir.path().join(".codeos/05-review/reviews/codex/a.md");
        fs::write(
            &assessment_path,
            format!("---\nreviewed:\n  feature: F-1\n  artifacts:\n    - path: .codeos/06-workflow/reality-evidence/{reviewed}.json\n      sha256: {reviewed}\n    - path: evidence/review/manifest.json\n      sha256: {manifest_sha}\n---\nassessment\n"),
        )
        .unwrap();
        fs::write(
            dir.path().join(".codeos/06-workflow/decisions.jsonl"),
            format!("{{\"workflow\":\"feature\",\"subject\":\"F-1\",\"checkpoint\":\"reconciliation\",\"bindings\":{{\"production_evidence\":\"{selected}\"}},\"timestamp\":\"2026-01-01T00:00:00Z\"}}\n"),
        )
        .unwrap();
        let assessment_sha = sha256_file(assessment_path.to_string_lossy().as_ref()).unwrap();
        let packet_sha = sha256_file(packet_path.to_string_lossy().as_ref()).unwrap();
        let review_id = "REV__F-1__8__SR2__TERMINAL-CLOSURE-1".to_string();
        let log = format!("# log\n\n## 2026-01-02T00:00:00Z TERMINAL REVIEW CLOSURE — F-1 — Stage 8\nReview ID: {review_id}\nEffective concern: TERMINAL REVIEW CLOSED — PROGRESSION PERMITTED\nParse status: OK   Assessment status: COMPLETE\nFull assessment: {} (sha256:{assessment_sha})\nReviewed packet: {} (sha256:{packet_sha})\n", assessment_path.display(), packet_path.display());
        let log_path = dir.path().join(".codeos/05-review/reviews/review-log.md");
        fs::write(&log_path, log).unwrap();
        let cfg = Config {
            reasoning_effort: "high".into(),
            packet_budget_mode: crate::config::PacketBudgetMode::Fail,
            repo_root: dir.path().to_path_buf(),
            toolkit_root: toolkit,
            review_log: log_path,
            codex_dir: dir.path().join(".codeos/05-review/reviews/codex"),
            state_dir: dir.path().join(".codeos-state"),
        };
        (dir, cfg, review_id)
    }

    #[test]
    fn audit_proves_pre_execution_f6_and_implementation_conflicts() {
        let (_dir, cfg, id) = fixture();
        let record = audit(&cfg, "F-1", "8", &id).unwrap();
        assert_eq!(
            record.reason_codes,
            vec![
                "F6_EVIDENCE_MISMATCH",
                "UNSUPPORTED_IMPLEMENTATION_IDENTITY"
            ]
        );
        assert_eq!(
            record.canonical_implementation_identity,
            "canonical-implementation"
        );
    }

    #[test]
    fn audit_record_is_single_use_and_replacement_id_is_not_r4() {
        let (_dir, cfg, id) = fixture();
        let record = audit(&cfg, "F-1", "8", &id).unwrap();
        append_audit(&cfg, &record).unwrap();
        assert!(audit(&cfg, "F-1", "8", &id).is_err());
        assert_eq!(
            replacement_id("F-1", 2),
            "REV__F-1__8__SR2__LINEAGE-REPLACEMENT-1"
        );
        let log = fs::read_to_string(&cfg.review_log).unwrap();
        let (_, stored_sha) = applicability_record(&log, &id).unwrap();
        assert_eq!(stored_sha, record.sha256().unwrap());
    }

    #[test]
    fn applicability_record_rejects_tampered_immutable_json() {
        let (_dir, cfg, id) = fixture();
        let record = audit(&cfg, "F-1", "8", &id).unwrap();
        append_audit(&cfg, &record).unwrap();
        let log = fs::read_to_string(&cfg.review_log).unwrap();
        let tampered = log.replace(
            "\"review_id\":\"REV__F-1__8__SR2__TERMINAL-CLOSURE-1\"",
            "\"review_id\":\"REV__F-1__8__SR2__R3\"",
        );
        assert!(applicability_record(&tampered, &id).is_err());
    }

    #[test]
    fn v16_rejects_transcribed_configuration_hash_before_review_capacity() {
        let supplied = vec![serde_json::json!({
            "bindings": {"codeos_yaml": "manually-expanded-wrong-hash"}
        })];
        let error = validate_v16_supplied_inputs("canonical-hash", &supplied, true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("not the canonical artifact hash"));
    }

    #[test]
    fn v16_rejects_summary_only_verification_before_review_capacity() {
        let supplied = vec![serde_json::json!({
            "bindings": {"codeos_yaml": "canonical-hash"},
            "verification": {"browser": {"result": "passed"}}
        })];
        let error = validate_v16_supplied_inputs("canonical-hash", &supplied, false)
            .unwrap_err()
            .to_string();
        assert!(error.contains("summary-only verification evidence"));
    }
}
