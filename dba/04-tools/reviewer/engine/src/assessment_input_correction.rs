use crate::config::Config;
use crate::packet::{sha256_file, sha256_str, ReviewPacket};
use anyhow::{Context, Result};
use codeos_workflow::receipts::{Receipt, ReceiptStore};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const ELIGIBILITY: &str = "ASSESSMENT_INPUT_CORRECTION";
const REQUIRED: &[&str] = &[
    "smoke",
    "product_journey",
    "behavior",
    "repeatability",
    "playwright",
    "runtime_event_schema",
    "production_source_evidence",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationIdentity {
    pub verification: String,
    pub checkpoint: String,
    pub timestamp: String,
    pub record_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionRecord {
    pub feature: String,
    pub stage: String,
    pub semantic_revision: u64,
    pub failed_review_id: String,
    pub failed_assessment_sha256: String,
    pub original_packet_sha256: String,
    pub failed_assessment_timestamp: String,
    pub incorrect_codeos_yaml_sha256: String,
    pub canonical_codeos_yaml_sha256: String,
    pub implementation_identity: String,
    pub implementation_artifacts: BTreeMap<String, String>,
    pub governed_inputs: BTreeMap<String, String>,
    pub architecture_identity: String,
    pub f6_selected_production_evidence: String,
    pub runtime_evidence_sha256: String,
    pub omitted_verification_records: Vec<VerificationIdentity>,
    pub reason_codes: Vec<String>,
}

fn block<'a>(log: &'a str, review_id: &str) -> Result<&'a str> {
    let marker = format!("Review ID: {review_id}");
    let (_, tail) = log.split_once(&marker).context("failed review is absent")?;
    Ok(tail.split("\n## ").next().unwrap_or(tail))
}

fn line<'a>(value: &'a str, prefix: &str) -> Result<&'a str> {
    value
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .with_context(|| format!("record omits {prefix}"))
}

fn reference(value: &str, prefix: &str) -> Result<(PathBuf, String)> {
    let raw = line(value, prefix)?;
    let (path, hash) = raw
        .rsplit_once(" (sha256:")
        .context("malformed reference")?;
    Ok((PathBuf::from(path), hash.trim_end_matches(')').to_string()))
}

fn frontmatter(path: &Path) -> Result<Value> {
    let text = std::fs::read_to_string(path)?;
    let yaml = text
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---"))
        .map(|(value, _)| value)
        .context("assessment has no frontmatter")?;
    Ok(serde_json::to_value(serde_yaml::from_str::<
        serde_yaml::Value,
    >(yaml)?)?)
}

fn review_timestamp(log: &str, review_id: &str) -> Result<String> {
    let before = log
        .split_once(&format!("Review ID: {review_id}"))
        .context("failed review is absent")?
        .0;
    let heading = before
        .lines()
        .rev()
        .find(|line| line.starts_with("## "))
        .context("review heading is absent")?;
    Ok(heading
        .trim_start_matches("## ")
        .split_whitespace()
        .next()
        .unwrap_or("")
        .into())
}

fn artifact_hash(reviewed: &Value, path: &str) -> Result<String> {
    reviewed
        .get("artifacts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|item| item.get("path").and_then(Value::as_str) == Some(path))
        .and_then(|item| item.get("sha256"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .with_context(|| format!("failed packet omits {path}"))
}

fn immutable_packet_artifact(packet: &str, path: &str, expected_sha: &str) -> Result<Vec<u8>> {
    let marker = format!("  --- {path} (sha256: {expected_sha}, visibility: shown) ---\n");
    let (_, tail) = packet
        .split_once(&marker)
        .with_context(|| format!("immutable packet omits shown artifact {path}"))?;
    let body = tail.split("\n  --- ").next().unwrap_or(tail);
    let normalized = body
        .lines()
        .map(|line| line.strip_prefix("    ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");
    let trimmed = normalized.trim_end_matches('\n');
    for candidate in [trimmed.to_string(), format!("{trimmed}\n")] {
        if sha256_str(&candidate) == expected_sha {
            return Ok(candidate.into_bytes());
        }
    }
    anyhow::bail!("immutable packet artifact identity mismatch")
}

fn latest_f6(repo: &Path, feature: &str, before: &str) -> Result<Receipt> {
    ReceiptStore::at(&repo.join(".codeos"))
        .latest_as_of("feature", feature, "reconciliation", before)?
        .context("no pre-assessment F6 selection")
}

fn verification_records(
    repo: &Path,
    feature: &str,
    implementation: &str,
    governed: &BTreeMap<String, String>,
    before: &str,
) -> Result<(Vec<VerificationIdentity>, String, String)> {
    let text = std::fs::read_to_string(repo.join(".codeos/06-workflow/verifications.jsonl"))?;
    let mut found: BTreeMap<String, (VerificationIdentity, String, Value)> = BTreeMap::new();
    for raw in text.lines().filter(|line| !line.trim().is_empty()) {
        let value: Value = serde_json::from_str(raw)?;
        if value.get("workflow").and_then(Value::as_str) != Some("feature")
            || value.get("subject").and_then(Value::as_str) != Some(feature)
            || value.get("result").and_then(Value::as_str) != Some("passed")
        {
            continue;
        }
        let Some(name) = value.get("verification").and_then(Value::as_str) else {
            continue;
        };
        if !REQUIRED.contains(&name) {
            continue;
        }
        let timestamp = value.get("timestamp").and_then(Value::as_str).unwrap_or("");
        if timestamp > before {
            continue;
        }
        let bindings = value
            .get("bindings")
            .and_then(Value::as_object)
            .context("verification omits bindings")?;
        if bindings.get("implementation_state").and_then(Value::as_str) != Some(implementation) {
            continue;
        }
        if governed.iter().any(|(key, expected)| {
            bindings.get(key).and_then(Value::as_str) != Some(expected.as_str())
        }) {
            continue;
        }
        let identity = VerificationIdentity {
            verification: name.into(),
            checkpoint: value
                .get("checkpoint")
                .and_then(Value::as_str)
                .unwrap_or("")
                .into(),
            timestamp: timestamp.into(),
            record_sha256: sha256_str(raw),
        };
        found.insert(name.into(), (identity, raw.into(), value));
    }
    let missing = REQUIRED
        .iter()
        .filter(|name| !found.contains_key(**name))
        .copied()
        .collect::<Vec<_>>();
    anyhow::ensure!(
        missing.is_empty(),
        "required pre-assessment verification is absent: {}",
        missing.join(", ")
    );
    let runtime = found["runtime_event_schema"]
        .2
        .pointer("/details/event_set_sha256")
        .and_then(Value::as_str)
        .context("runtime verification omits event evidence identity")?
        .to_string();
    let raw = REQUIRED
        .iter()
        .map(|name| found[*name].1.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    Ok((
        REQUIRED.iter().map(|name| found[*name].0.clone()).collect(),
        runtime,
        raw,
    ))
}

pub fn audit(
    cfg: &Config,
    feature: &str,
    stage: &str,
    review_id: &str,
) -> Result<CorrectionRecord> {
    anyhow::ensure!(stage == "8", "assessment input correction is Stage 8 only");
    anyhow::ensure!(
        matches!(
            crate::config::resolve_reviewer_tool_contract(cfg)?.as_str(),
            "dba/04-tools/reviewer/contract/v16.md" | "dba/04-tools/reviewer/contract/v17.md"
        ),
        "assessment input correction requires Reviewer Tool Contract v16 or later"
    );
    anyhow::ensure!(
        review_id.ends_with("__LINEAGE-REPLACEMENT-1"),
        "target is not LINEAGE-REPLACEMENT-1"
    );
    let log = std::fs::read_to_string(&cfg.review_log)?;
    anyhow::ensure!(
        !log.contains(&format!("Correction eligibility target: {review_id}")),
        "correction eligibility already recorded"
    );
    let failed = block(&log, review_id)?;
    anyhow::ensure!(
        failed.contains("Parse status: OK   Assessment status: COMPLETE")
            && failed.contains("Effective concern: DO NOT ADVANCE"),
        "target is not a complete failed assessment"
    );
    let (assessment_path, assessment_sha) = reference(failed, "Full assessment: ")?;
    let (packet_path, packet_sha) = reference(failed, "Reviewed packet: ")?;
    anyhow::ensure!(
        sha256_file(assessment_path.to_string_lossy().as_ref())? == assessment_sha,
        "failed assessment identity mismatch"
    );
    anyhow::ensure!(
        sha256_file(packet_path.to_string_lossy().as_ref())? == packet_sha,
        "original packet identity mismatch"
    );
    let assessment = frontmatter(&assessment_path)?;
    let findings = assessment
        .get("findings")
        .and_then(Value::as_array)
        .context("assessment has no findings")?;
    anyhow::ensure!(
        findings.len() == 2,
        "assessment contains findings beyond the two correctable input defects"
    );
    let mut configuration_input = false;
    let mut omitted_verification = false;
    for finding in findings {
        let summary = finding.get("summary").and_then(Value::as_str).unwrap_or("");
        let evidence = finding
            .get("evidence")
            .and_then(Value::as_str)
            .unwrap_or("");
        let kind = finding
            .get("blocker_kind")
            .and_then(Value::as_str)
            .unwrap_or("");
        if summary.contains("configuration input")
            || evidence.contains("codeos.yaml")
            || evidence.contains("codeos_yaml")
        {
            configuration_input = true;
        } else if kind == "EVIDENCE_INSUFFICIENT"
            && (summary.contains("verification") || evidence.contains("verifications.jsonl"))
        {
            omitted_verification = true;
        } else {
            anyhow::bail!("assessment contains a substantive or unrecognized blocker");
        }
    }
    anyhow::ensure!(
        configuration_input && omitted_verification,
        "assessment blockers do not establish both permitted input-defect classes"
    );
    let reviewed = assessment
        .get("reviewed")
        .context("assessment omits reviewed inputs")?;
    anyhow::ensure!(
        reviewed.get("feature").and_then(Value::as_str) == Some(feature),
        "assessment belongs to another feature"
    );
    let timestamp = review_timestamp(&log, review_id)?;
    let revision = review_id
        .split("__SR")
        .nth(1)
        .and_then(|v| v.split("__").next())
        .and_then(|v| v.parse().ok())
        .context("review id omits semantic revision")?;
    let manifest_path = reviewed
        .get("artifacts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|a| a.get("path").and_then(Value::as_str))
        .find(|p| p.starts_with("evidence/review/") && p.ends_with(".json"))
        .context("failed packet has no review evidence manifest")?;
    let manifest_sha = artifact_hash(reviewed, manifest_path)?;
    let packet_bytes = std::fs::read_to_string(&packet_path)?;
    let manifest: Value = serde_json::from_slice(&immutable_packet_artifact(
        &packet_bytes,
        manifest_path,
        &manifest_sha,
    )?)?;
    let incorrect = manifest
        .pointer("/bindings/codeos_yaml")
        .and_then(Value::as_str)
        .context("review manifest omits codeos_yaml identity")?
        .to_string();
    let f6 = latest_f6(&cfg.repo_root, feature, &timestamp)?;
    let canonical = f6
        .bindings
        .get("codeos_yaml")
        .context("pre-assessment F6 receipt omits canonical codeos.yaml identity")?
        .clone();
    anyhow::ensure!(
        incorrect != canonical,
        "configuration identity was not objectively incorrect"
    );
    let implementation = manifest
        .pointer("/bindings/implementation_state")
        .and_then(Value::as_str)
        .context("review manifest omits implementation identity")?
        .to_string();
    let governed = ["intent", "contract", "event_schema"]
        .into_iter()
        .map(|key| {
            Ok((
                key.to_string(),
                manifest
                    .pointer(&format!("/bindings/{key}"))
                    .and_then(Value::as_str)
                    .with_context(|| format!("manifest omits {key}"))?
                    .to_string(),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let architecture = artifact_hash(
        reviewed,
        ".codeos/02-architecture/scopes/trip-forecasting.md",
    )?;
    let implementation_artifacts = reviewed
        .get("artifacts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let path = item.get("path")?.as_str()?;
            (path.starts_with("backend/")
                || path.starts_with("web/")
                || path.starts_with("provider-boundary/")
                || path == "docker-compose.yml")
                .then(|| Some((path.to_string(), item.get("sha256")?.as_str()?.to_string())))?
        })
        .collect::<BTreeMap<_, _>>();
    anyhow::ensure!(
        !implementation_artifacts.is_empty(),
        "failed assessment omits implementation surface"
    );
    let production = f6
        .bindings
        .get("production_evidence")
        .context("F6 omits production evidence")?
        .clone();
    let prod: Value = serde_json::from_slice(&std::fs::read(cfg.repo_root.join(format!(
        ".codeos/06-workflow/reality-evidence/{production}.json"
    )))?)?;
    anyhow::ensure!(
        prod.get("implementation_sha256").and_then(Value::as_str) == Some(&implementation),
        "production evidence implementation binding conflicts"
    );
    let (records, runtime, _) = verification_records(
        &cfg.repo_root,
        feature,
        &implementation,
        &governed,
        &timestamp,
    )?;
    anyhow::ensure!(
        !packet_bytes.contains("UNDERLYING INSPECTABLE VERIFICATION RECORDS"),
        "verification output was not omitted from original packet"
    );
    Ok(CorrectionRecord {
        feature: feature.into(),
        stage: stage.into(),
        semantic_revision: revision,
        failed_review_id: review_id.into(),
        failed_assessment_sha256: assessment_sha,
        original_packet_sha256: packet_sha,
        failed_assessment_timestamp: timestamp,
        incorrect_codeos_yaml_sha256: incorrect,
        canonical_codeos_yaml_sha256: canonical,
        implementation_identity: implementation,
        implementation_artifacts,
        governed_inputs: governed,
        architecture_identity: architecture,
        f6_selected_production_evidence: production,
        runtime_evidence_sha256: runtime,
        omitted_verification_records: records,
        reason_codes: vec![
            "INCORRECT_CANONICAL_IDENTITY".into(),
            "OMITTED_PREEXISTING_VERIFICATION_OUTPUT".into(),
        ],
    })
}

pub fn append_audit(cfg: &Config, record: &CorrectionRecord) -> Result<String> {
    let json = serde_json::to_string(record)?;
    let sha = sha256_str(&json);
    let entry = format!("\n## {} ASSESSMENT INPUT CORRECTION ELIGIBILITY — {} — Stage {}\nCorrection eligibility target: {}\nEligibility: {ELIGIBILITY}\nCorrection record SHA-256: {sha}\nCorrection record JSON: {json}\nFailed assessment SHA-256: {}\nOriginal packet SHA-256: {}\nCanonical codeos.yaml SHA-256: {}\nPre-existing verification record identities: {}\nReason codes: {}\n", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"), record.feature, record.stage, record.failed_review_id, record.failed_assessment_sha256, record.original_packet_sha256, record.canonical_codeos_yaml_sha256, record.omitted_verification_records.iter().map(|v| v.record_sha256.as_str()).collect::<Vec<_>>().join(", "), record.reason_codes.join(", "));
    crate::log::append_raw(&cfg.review_log, &entry)?;
    Ok(sha)
}

pub fn correction_record(log: &str, failed_id: &str) -> Result<(CorrectionRecord, String)> {
    let marker = format!("Correction eligibility target: {failed_id}\nEligibility: {ELIGIBILITY}");
    let before = log
        .split_once(&marker)
        .context("eligible correction record is absent")?
        .0;
    let heading = before.rsplit("\n## ").next().unwrap_or(before);
    let full = format!(
        "{heading}{marker}{}",
        log.split_once(&marker)
            .unwrap()
            .1
            .split("\n## ")
            .next()
            .unwrap_or("")
    );
    let json = line(&full, "Correction record JSON: ")?;
    let sha = line(&full, "Correction record SHA-256: ")?.to_string();
    anyhow::ensure!(
        sha256_str(json) == sha,
        "correction record identity mismatch"
    );
    Ok((serde_json::from_str(json)?, sha))
}

pub fn resolve_authorization(log: &str, failed_id: &str) -> Result<CorrectionRecord> {
    let (record, sha) = correction_record(log, failed_id)?;
    let authorization = log
        .split("\n## ")
        .find(|block| {
            block.contains("Decision: AUTHORIZE_ASSESSMENT_INPUT_CORRECTION")
                && block.contains(&format!("Assessment input correction after: {failed_id}"))
                && block.contains(&format!("Correction record SHA-256: {sha}"))
        })
        .context("explicit correction authorization is absent")?;
    anyhow::ensure!(
        !authorization.contains("Decision: STOP"),
        "correction authorization stopped"
    );
    anyhow::ensure!(
        !log.contains("__LINEAGE-REPLACEMENT-CORRECTION-1"),
        "correction assessment already consumed"
    );
    Ok(record)
}

pub fn preflight(cfg: &Config, record: &CorrectionRecord, packet: &ReviewPacket) -> Result<String> {
    let fresh = audit_without_duplicate(
        cfg,
        &record.feature,
        &record.stage,
        &record.failed_review_id,
    )?;
    anyhow::ensure!(
        serde_json::to_string(&fresh)? == serde_json::to_string(record)?,
        "correction inputs or unchanged state drifted after eligibility audit"
    );
    let packet_artifacts = packet
        .artifacts
        .iter()
        .map(|artifact| (artifact.path.as_str(), artifact.sha256.as_str()))
        .collect::<BTreeMap<_, _>>();
    for (path, expected) in &record.implementation_artifacts {
        anyhow::ensure!(
            packet_artifacts.get(path.as_str()).copied() == Some(expected.as_str()),
            "implementation changed after failed assessment: {path}"
        );
    }
    anyhow::ensure!(
        packet_artifacts
            .get(".codeos/02-architecture/scopes/trip-forecasting.md")
            .copied()
            == Some(record.architecture_identity.as_str()),
        "architecture changed after failed assessment"
    );
    for (name, path) in [
        (
            "intent",
            format!(".codeos/01-specification/intents/{}.md", record.feature),
        ),
        (
            "contract",
            format!(
                ".codeos/01-specification/contracts/{}_contract.md",
                record.feature
            ),
        ),
        (
            "event_schema",
            format!(
                ".codeos/01-specification/event-schemas/{}_schema.md",
                record.feature
            ),
        ),
    ] {
        anyhow::ensure!(
            packet_artifacts.get(path.as_str()).copied()
                == record.governed_inputs.get(name).map(String::as_str),
            "governed meaning changed after failed assessment: {name}"
        );
    }
    let current_f6 = latest_f6(&cfg.repo_root, &record.feature, "9999-12-31T23:59:59Z")?;
    anyhow::ensure!(
        current_f6.bindings.get("production_evidence")
            == Some(&record.f6_selected_production_evidence),
        "F6 production-evidence selection changed after failed assessment"
    );
    anyhow::ensure!(
        packet_artifacts.iter().any(|(path, hash)| {
            path.starts_with(".codeos/06-workflow/event-evidence/")
                && **hash == record.runtime_evidence_sha256
        }),
        "corrected packet omits the bound pre-assessment runtime evidence"
    );
    let (_, _, raw) = verification_records(
        &cfg.repo_root,
        &record.feature,
        &record.implementation_identity,
        &record.governed_inputs,
        &record.failed_assessment_timestamp,
    )?;
    let paths = packet
        .artifacts
        .iter()
        .map(|a| a.path.as_str())
        .collect::<BTreeSet<_>>();
    anyhow::ensure!(
        paths.contains(
            &format!(
                ".codeos/06-workflow/reality-evidence/{}.json",
                record.f6_selected_production_evidence
            )
            .as_str()
        ),
        "corrected packet omits F6-selected production evidence"
    );
    Ok(raw)
}

fn audit_without_duplicate(
    cfg: &Config,
    feature: &str,
    stage: &str,
    review_id: &str,
) -> Result<CorrectionRecord> {
    let log = std::fs::read_to_string(&cfg.review_log)?;
    let marker = format!("Correction eligibility target: {review_id}");
    let stripped = if let Some((before, after)) = log.split_once(&marker) {
        let start = before.rfind("\n## ").unwrap_or(before.len());
        let remainder = after
            .split_once("\n## ")
            .map(|(_, rest)| format!("\n## {rest}"))
            .unwrap_or_default();
        format!("{}{}", &before[..start], remainder)
    } else {
        log.clone()
    };
    let temp = tempfile::NamedTempFile::new()?;
    std::fs::write(temp.path(), &stripped)?;
    let mut cfg2 = cfg.clone();
    cfg2.review_log = temp.path().to_path_buf();
    audit(&cfg2, feature, stage, review_id)
}

pub fn correction_id(feature: &str, revision: u64) -> String {
    format!("REV__{feature}__8__SR{revision}__LINEAGE-REPLACEMENT-CORRECTION-1")
}
