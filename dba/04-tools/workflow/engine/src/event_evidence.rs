//! Canonical, run-bounded runtime-event evidence.
//!
//! The verification store remains the authority. This module captures only bytes appended while
//! one caller-supplied verification command executes, validates those exact bytes, stores them
//! content-addressably, and returns a record bound to the current feature inputs.

use crate::checker::{verification_bindings_for_mechanic, Subject};
use crate::hashing;
use crate::project::Project;
use crate::verification::{VerificationRecord, VerificationStore};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct VerifyEventArgs<'a> {
    pub subject: &'a Subject,
    pub event_log: &'a str,
    pub validator: &'a str,
    pub command: &'a [String],
}

struct LockGuard(PathBuf);

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn project_file(project: &Project, supplied: &str, purpose: &str) -> Result<PathBuf> {
    let relative = Path::new(supplied);
    anyhow::ensure!(
        !supplied.trim().is_empty()
            && !relative.is_absolute()
            && !relative
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir)),
        "{purpose} must be a non-traversing project-relative path"
    );
    let path = project.root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(path)
}

fn acquire_lock(project: &Project) -> Result<LockGuard> {
    let path = project.codeos_dir().join("06-workflow/event-evidence.lock");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .with_context(|| {
            format!(
                "another bounded event-evidence run is active (lock: {})",
                path.display()
            )
        })?;
    Ok(LockGuard(path))
}

fn parse_events(bytes: &[u8]) -> Result<Vec<Value>> {
    anyhow::ensure!(!bytes.is_empty(), "verification appended no runtime events");
    anyhow::ensure!(
        bytes.ends_with(b"\n"),
        "bounded runtime-event evidence ends with an incomplete JSONL record"
    );
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .enumerate()
        .map(|(index, line)| {
            let value: Value = serde_json::from_slice(line)
                .with_context(|| format!("runtime event {} is malformed JSON", index + 1))?;
            anyhow::ensure!(
                value.is_object(),
                "runtime event {} is not an object",
                index + 1
            );
            Ok(value)
        })
        .collect()
}

fn bounded_append<'a>(before: &[u8], after: &'a [u8]) -> Result<&'a [u8]> {
    anyhow::ensure!(
        after.len() >= before.len() && after[..before.len()] == *before,
        "event log was truncated or its pre-run prefix changed during verification"
    );
    Ok(&after[before.len()..])
}

fn correlation_ids(events: &[Value]) -> Result<Vec<String>> {
    let mut ids = BTreeSet::new();
    for (index, event) in events.iter().enumerate() {
        let value = event
            .get("correlation_id")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .with_context(|| format!("runtime event {} has no correlation_id", index + 1))?;
        ids.insert(value.to_string());
    }
    Ok(ids.into_iter().collect())
}

fn semantic_hash(events: &[Value]) -> Result<String> {
    let mut correlations = BTreeMap::<String, usize>::new();
    let mut next_correlation = 1usize;
    let normalized: Vec<Value> = events
        .iter()
        .map(|event| {
            let mut event = event.clone();
            let object = event
                .as_object_mut()
                .expect("parse_events guarantees objects");
            object.remove("event_id");
            object.remove("timestamp");
            if let Some(correlation) = object.get("correlation_id").and_then(Value::as_str) {
                let ordinal = *correlations
                    .entry(correlation.to_string())
                    .or_insert_with(|| {
                        let current = next_correlation;
                        next_correlation += 1;
                        current
                    });
                object.insert(
                    "correlation_id".into(),
                    Value::String(format!("correlation-{ordinal}")),
                );
            }
            event
        })
        .collect();
    Ok(hashing::bytes_sha256(&serde_json::to_vec(&normalized)?))
}

fn validate_with_jq(validator: &Path, bytes: &[u8], event_count: usize) -> Result<Value> {
    let mut child = Command::new("jq")
        .args(["-s", "-f"])
        .arg(validator)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("start jq validator {}", validator.display()))?;
    child
        .stdin
        .take()
        .context("open jq stdin")?
        .write_all(bytes)?;
    let output = child.wait_with_output()?;
    anyhow::ensure!(
        output.status.success(),
        "event-schema validator failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    let result: Value = serde_json::from_slice(&output.stdout)
        .context("event-schema validator did not return one JSON result")?;
    anyhow::ensure!(
        result.get("total").and_then(Value::as_u64) == Some(event_count as u64),
        "event-schema validator result does not account for the exact bounded event set"
    );
    let invalid = result
        .get("invalid_events")
        .and_then(Value::as_array)
        .context("event-schema validator result omits invalid_events")?;
    anyhow::ensure!(
        invalid.is_empty(),
        "bounded event set contains schema-invalid events"
    );
    let prohibited = result
        .pointer("/privacy_check/prohibited_payload_keys_found")
        .and_then(Value::as_array)
        .context("event-schema validator result omits prohibited payload-field findings")?;
    anyhow::ensure!(
        prohibited.is_empty(),
        "bounded event set contains prohibited payload fields"
    );
    Ok(result)
}

pub fn verify(project: &Project, args: VerifyEventArgs<'_>) -> Result<VerificationRecord> {
    anyhow::ensure!(
        args.subject.workflow.slug() == "feature",
        "bounded runtime-event evidence is supported only for a feature"
    );
    anyhow::ensure!(
        !args.command.is_empty(),
        "verification command is required after --"
    );
    let event_log = project_file(project, args.event_log, "event log")?;
    let validator = project_file(project, args.validator, "event-schema validator")?;
    anyhow::ensure!(validator.is_file(), "event-schema validator does not exist");
    let _lock = acquire_lock(project)?;

    let before = std::fs::read(&event_log).unwrap_or_default();
    anyhow::ensure!(
        before.is_empty() || before.ends_with(b"\n"),
        "event log has an incomplete pre-existing JSONL record"
    );
    let output = Command::new(&args.command[0])
        .args(&args.command[1..])
        .current_dir(&project.root)
        .output()
        .with_context(|| format!("execute verification command {}", args.command.join(" ")))?;
    anyhow::ensure!(
        output.status.success(),
        "verification command failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );

    let after = std::fs::read(&event_log)
        .with_context(|| format!("read event log after verification: {}", event_log.display()))?;
    let bounded = bounded_append(&before, &after)?;
    let events = parse_events(bounded)?;
    let correlations = correlation_ids(&events)?;
    let validator_result = validate_with_jq(&validator, bounded, events.len())?;
    let event_set_sha256 = hashing::bytes_sha256(bounded);
    let semantic_sha256 = semantic_hash(&events)?;

    let evidence_dir = project.codeos_dir().join("06-workflow/event-evidence");
    std::fs::create_dir_all(&evidence_dir)?;
    let evidence_path = evidence_dir.join(format!("{event_set_sha256}.jsonl"));
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&evidence_path)
    {
        Ok(mut file) => file.write_all(bounded)?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            anyhow::ensure!(
                std::fs::read(&evidence_path)? == bounded,
                "content-addressed event evidence path contains different bytes"
            );
        }
        Err(error) => return Err(error.into()),
    }

    let mut bindings =
        verification_bindings_for_mechanic(project, args.subject, "F5g", "playwright")?;
    bindings.insert(
        "event_evidence_validator".into(),
        hashing::file_sha256(&validator)?.context("hash event-schema validator")?,
    );
    let relative_evidence = evidence_path
        .strip_prefix(&project.root)
        .unwrap_or(&evidence_path)
        .to_string_lossy()
        .into_owned();
    let record = VerificationRecord {
        workflow: "feature".into(),
        subject: args.subject.id.clone(),
        checkpoint: "F7".into(),
        verification: "runtime_event_schema".into(),
        result: "passed".into(),
        bindings,
        command: args.command.join(" "),
        timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        details: Some(serde_json::json!({
            "schema": 1,
            "event_evidence": relative_evidence,
            "event_set_sha256": event_set_sha256,
            "semantic_sha256": semantic_sha256,
            "event_count": events.len(),
            "correlation_ids": correlations,
            "event_schema_conformance": "passed",
            "validator_result": validator_result,
        })),
    };
    VerificationStore::at(&project.codeos_dir()).append(&record)?;
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verification::{evaluate, RecordState};

    fn event(id: &str, correlation: &str, kind: &str) -> Value {
        serde_json::json!({
            "event_id": id,
            "event_type": kind,
            "timestamp": 1,
            "correlation_id": correlation,
            "source_module": "x",
            "payload": {}
        })
    }

    #[test]
    fn deterministic_semantics_ignore_generated_envelope_fields() {
        let first = vec![
            event("id-a", "run-a", "Started"),
            event("id-b", "run-a", "Done"),
        ];
        let second = vec![
            event("id-c", "run-b", "Started"),
            event("id-d", "run-b", "Done"),
        ];
        assert_eq!(
            semantic_hash(&first).unwrap(),
            semantic_hash(&second).unwrap()
        );
    }

    #[test]
    fn malformed_and_incomplete_jsonl_fail_closed() {
        assert!(parse_events(br#"{"event_type":"x"}"#).is_err());
        assert!(parse_events(b"not-json\n").is_err());
        assert!(parse_events(b"").is_err());
    }

    #[test]
    fn correlation_identity_is_required_and_scoped_to_the_bounded_set() {
        assert!(correlation_ids(&[serde_json::json!({"event_type":"x"})]).is_err());
        assert_eq!(
            correlation_ids(&[event("a", "run-2", "x"), event("b", "run-1", "y")]).unwrap(),
            vec!["run-1", "run-2"]
        );
    }

    #[test]
    fn pre_run_events_cannot_enter_the_bounded_set() {
        let before = b"{\"correlation_id\":\"other-run\"}\n";
        let after = b"{\"correlation_id\":\"other-run\"}\n{\"correlation_id\":\"this-run\"}\n";
        assert_eq!(
            bounded_append(before, after).unwrap(),
            b"{\"correlation_id\":\"this-run\"}\n"
        );
        assert!(bounded_append(before, b"rewritten\n").is_err());
    }

    fn jq_filter(dir: &Path) -> PathBuf {
        let path = dir.join("validate.jq");
        std::fs::write(
            &path,
            r#"{
              total: length,
              invalid_events: [.[] | select(.event_type != "Allowed")],
              privacy_check: {prohibited_payload_keys_found: [.[] | .payload | keys[] | select(. == "secret")]}
            }"#,
        )
        .unwrap();
        path
    }

    #[test]
    fn schema_valid_events_pass_and_invalid_events_fail() {
        let dir = tempfile::tempdir().unwrap();
        let validator = jq_filter(dir.path());
        let valid = b"{\"event_type\":\"Allowed\",\"payload\":{}}\n";
        assert!(validate_with_jq(&validator, valid, 1).is_ok());
        let invalid = b"{\"event_type\":\"Unknown\",\"payload\":{}}\n";
        assert!(validate_with_jq(&validator, invalid, 1).is_err());
    }

    #[test]
    fn prohibited_payload_and_incomplete_validator_results_fail() {
        let dir = tempfile::tempdir().unwrap();
        let validator = jq_filter(dir.path());
        let prohibited = b"{\"event_type\":\"Allowed\",\"payload\":{\"secret\":true}}\n";
        assert!(validate_with_jq(&validator, prohibited, 1).is_err());
        let valid = b"{\"event_type\":\"Allowed\",\"payload\":{}}\n";
        assert!(validate_with_jq(&validator, valid, 2).is_err());
    }

    #[test]
    fn implementation_and_governed_input_mismatch_make_evidence_stale() {
        let record = VerificationRecord {
            workflow: "feature".into(),
            subject: "F-1".into(),
            checkpoint: "F7".into(),
            verification: "runtime_event_schema".into(),
            result: "passed".into(),
            bindings: BTreeMap::from([
                ("implementation_state".into(), "implementation-a".into()),
                ("event_schema".into(), "schema-a".into()),
            ]),
            command: "verify".into(),
            timestamp: "now".into(),
            details: None,
        };
        let implementation_b = BTreeMap::from([
            ("implementation_state".into(), "implementation-b".into()),
            ("event_schema".into(), "schema-a".into()),
        ]);
        let schema_b = BTreeMap::from([
            ("implementation_state".into(), "implementation-a".into()),
            ("event_schema".into(), "schema-b".into()),
        ]);
        assert!(
            matches!(evaluate(Some(record.clone()), &implementation_b), RecordState::Stale { binding, .. } if binding == "implementation_state")
        );
        assert!(
            matches!(evaluate(Some(record), &schema_b), RecordState::Stale { binding, .. } if binding == "event_schema")
        );
    }
}
