//! The append-only mechanical-verification store: `.codeos/06-workflow/verifications.jsonl`.
//!
//! A verification record is evidence that a mechanical verification actually executed and passed
//! against a specific state. Only `check` writes one, and only on success. It makes no adequacy
//! judgment — it does not claim the behavior is correct, only that the named verification ran and
//! passed against the bound inputs. It stops establishing its checkpoint the moment any bound input
//! no longer matches its current hash.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationRecord {
    pub workflow: String,
    pub subject: String,
    /// The checkpoint this verification belongs to, e.g. `F3`, `F5`, `B3`.
    pub checkpoint: String,
    /// The verification identity, e.g. `smoke`, `behavior`, `baseline`.
    pub verification: String,
    /// Always `passed` — a failing verification is never persisted.
    pub result: String,
    /// Named governed inputs / implementation state -> content hash at execution time.
    pub bindings: BTreeMap<String, String>,
    /// The command line actually executed.
    pub command: String,
    pub timestamp: String,
}

/// Why a verification record does or does not currently establish its checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub enum RecordState {
    Absent,
    Current(VerificationRecord),
    Stale {
        record: VerificationRecord,
        binding: String,
        was: String,
        now: String,
    },
}

pub struct VerificationStore {
    path: PathBuf,
}

impl VerificationStore {
    pub fn at(codeos_dir: &Path) -> Self {
        Self {
            path: codeos_dir.join("06-workflow").join("verifications.jsonl"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn all(&self) -> Result<Vec<VerificationRecord>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let text = std::fs::read_to_string(&self.path)
            .with_context(|| format!("read {}", self.path.display()))?;
        let mut out = Vec::new();
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            out.push(serde_json::from_str(line).with_context(|| {
                format!(
                    "{}:{}: malformed verification record",
                    self.path.display(),
                    i + 1
                )
            })?);
        }
        Ok(out)
    }

    /// The most recent record for this workflow/subject/checkpoint/verification, by file order.
    pub fn latest(
        &self,
        workflow: &str,
        subject: &str,
        checkpoint: &str,
        verification: &str,
    ) -> Result<Option<VerificationRecord>> {
        Ok(self.all()?.into_iter().rfind(|r| {
            r.workflow == workflow
                && r.subject == subject
                && r.checkpoint == checkpoint
                && r.verification == verification
        }))
    }

    pub fn append(&self, record: &VerificationRecord) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        let line = serde_json::to_string(record)?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("open {}", self.path.display()))?;
        writeln!(f, "{line}").with_context(|| format!("append to {}", self.path.display()))?;
        Ok(())
    }
}

/// Resolve a verification record against the current hashes of its bindings.
pub fn evaluate(
    record: Option<VerificationRecord>,
    current: &BTreeMap<String, String>,
) -> RecordState {
    let Some(record) = record else {
        return RecordState::Absent;
    };
    match crate::hashing::binding_drift(&record.bindings, current) {
        Some((binding, was, now)) => RecordState::Stale {
            record,
            binding,
            was,
            now,
        },
        None => RecordState::Current(record),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn rec(cp: &str, verification: &str, bindings: &[(&str, &str)]) -> VerificationRecord {
        VerificationRecord {
            workflow: "feature".into(),
            subject: "F-0001".into(),
            checkpoint: cp.into(),
            verification: verification.into(),
            result: "passed".into(),
            bindings: b(bindings),
            command: "cargo test".into(),
            timestamp: "2026-09-06T00:00:00Z".into(),
        }
    }

    #[test]
    fn append_latest_roundtrips_by_checkpoint_and_verification() {
        let dir = tempfile::tempdir().unwrap();
        let store = VerificationStore::at(dir.path());
        store
            .append(&rec("F5", "behavior", &[("contract", "v1")]))
            .unwrap();
        store
            .append(&rec("F5", "repeatability", &[("contract", "v1")]))
            .unwrap();
        store
            .append(&rec("F5", "behavior", &[("contract", "v2")]))
            .unwrap();
        let got = store
            .latest("feature", "F-0001", "F5", "behavior")
            .unwrap()
            .unwrap();
        assert_eq!(got.bindings.get("contract").unwrap(), "v2");
        assert!(store
            .latest("feature", "F-0001", "F5", "repeatability")
            .unwrap()
            .is_some());
        assert!(store
            .latest("feature", "F-0001", "F5", "playwright")
            .unwrap()
            .is_none());
    }

    #[test]
    fn evaluate_detects_absent_current_and_stale() {
        assert_eq!(evaluate(None, &b(&[])), RecordState::Absent);
        let r = rec("F3", "smoke", &[("contract", "hash-a"), ("impl", "hash-x")]);
        assert!(matches!(
            evaluate(
                Some(r.clone()),
                &b(&[("contract", "hash-a"), ("impl", "hash-x")])
            ),
            RecordState::Current(_)
        ));
        match evaluate(Some(r), &b(&[("contract", "hash-a"), ("impl", "hash-Y")])) {
            RecordState::Stale { binding, now, .. } => {
                assert_eq!(binding, "impl");
                assert_eq!(now, "hash-Y");
            }
            other => panic!("expected Stale, got {other:?}"),
        }
    }
}
