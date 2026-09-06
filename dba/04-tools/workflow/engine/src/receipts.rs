//! The append-only decision-receipt store: `.codeos/06-workflow/decisions.jsonl`.
//!
//! A receipt records that a workflow decision occurred and which governed inputs it was bound to.
//! It is evidence of progression, never a governing artifact. A receipt stops establishing its
//! checkpoint the moment any binding no longer matches its input's current hash.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The closed set of receipt-bearing decisions. A value outside this set is rejected on write.
pub const RECEIPT_CHECKPOINTS: &[&str] = &[
    "initial_product_preview", // bootstrap B4
    "early_preview",           // feature F4
    "reconciliation",          // feature F6
    "final_ux_validation",     // feature F8
    "acceptance",              // feature F9
    "operation_route",         // operation O2
    "no_action_closure",       // operation O5 (no_action branch only)
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub workflow: String,
    pub subject: String,
    pub checkpoint: String,
    pub result: String,
    /// Named governed inputs -> their content hash at decision time.
    pub bindings: BTreeMap<String, String>,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rationale: Option<String>,
    /// The observation statement itself, for an `operation_route` / `no_action_closure` receipt.
    /// Its hash is `bindings["observation"]`; storing the text keeps it from being lost, since an
    /// Operational Observation has no other durable home.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub observation: Option<String>,
}

impl Receipt {
    /// Only the closure receipt carries the "why no action" rationale; the O2 route classification
    /// does not.
    pub fn requires_rationale(&self) -> bool {
        self.checkpoint == "no_action_closure"
    }
}

/// Why a receipt does or does not currently establish its checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub enum ReceiptState {
    /// No receipt found for this workflow/subject/checkpoint.
    Absent,
    /// A receipt is present and all bindings still match.
    Current(Receipt),
    /// A receipt is present but a binding drifted. Carries (binding_name, was, now).
    Stale {
        receipt: Receipt,
        binding: String,
        was: String,
        now: String,
    },
}

pub struct ReceiptStore {
    path: PathBuf,
}

impl ReceiptStore {
    pub fn at(codeos_dir: &Path) -> Self {
        Self {
            path: codeos_dir.join("06-workflow").join("decisions.jsonl"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn all(&self) -> Result<Vec<Receipt>> {
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
            let r: Receipt = serde_json::from_str(line)
                .with_context(|| format!("{}:{}: malformed receipt", self.path.display(), i + 1))?;
            out.push(r);
        }
        Ok(out)
    }

    /// The most recent receipt for this workflow/subject/checkpoint, by file order (append-only).
    pub fn latest(
        &self,
        workflow: &str,
        subject: &str,
        checkpoint: &str,
    ) -> Result<Option<Receipt>> {
        Ok(self.all()?.into_iter().rfind(|r| {
            r.workflow == workflow && r.subject == subject && r.checkpoint == checkpoint
        }))
    }

    /// Append a receipt. Fails closed on an out-of-set checkpoint or a missing required rationale.
    pub fn append(&self, receipt: &Receipt) -> Result<()> {
        if !RECEIPT_CHECKPOINTS.contains(&receipt.checkpoint.as_str()) {
            anyhow::bail!(
                "'{}' is not a receipt-bearing checkpoint; the closed set is {:?}",
                receipt.checkpoint,
                RECEIPT_CHECKPOINTS
            );
        }
        if receipt.requires_rationale()
            && receipt
                .rationale
                .as_deref()
                .map(str::trim)
                .unwrap_or("")
                .is_empty()
        {
            anyhow::bail!(
                "checkpoint '{}' requires a non-empty rationale",
                receipt.checkpoint
            );
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        let line = serde_json::to_string(receipt)?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("open {}", self.path.display()))?;
        writeln!(f, "{line}").with_context(|| format!("append to {}", self.path.display()))?;
        Ok(())
    }
}

/// Resolve a receipt against the current hashes of its bindings.
pub fn evaluate(receipt: Option<Receipt>, current: &BTreeMap<String, String>) -> ReceiptState {
    let Some(receipt) = receipt else {
        return ReceiptState::Absent;
    };
    match crate::hashing::binding_drift(&receipt.bindings, current) {
        Some((binding, was, now)) => ReceiptState::Stale {
            binding,
            was,
            now,
            receipt,
        },
        None => ReceiptState::Current(receipt),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bindings(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn receipt(cp: &str, b: &[(&str, &str)]) -> Receipt {
        Receipt {
            workflow: "feature".into(),
            subject: "F-0001".into(),
            checkpoint: cp.into(),
            result: "direction_confirmed".into(),
            bindings: bindings(b),
            timestamp: "2026-09-05T00:00:00Z".into(),
            rationale: None,
            observation: None,
        }
    }

    #[test]
    fn append_rejects_out_of_set_checkpoint() {
        let dir = tempfile::tempdir().unwrap();
        let store = ReceiptStore::at(dir.path());
        let mut r = receipt("smoke", &[]);
        r.checkpoint = "smoke".into();
        assert!(store.append(&r).is_err());
    }

    #[test]
    fn append_requires_rationale_for_no_action() {
        let dir = tempfile::tempdir().unwrap();
        let store = ReceiptStore::at(dir.path());
        let mut r = receipt("no_action_closure", &[("observation", "abc")]);
        r.result = "no_action".into();
        assert!(store.append(&r).is_err(), "missing rationale must fail");
        r.rationale = Some("deliberately not acted on: cosmetic only".into());
        assert!(store.append(&r).is_ok());
    }

    #[test]
    fn latest_returns_most_recent_and_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let store = ReceiptStore::at(dir.path());
        let mut a = receipt("early_preview", &[("contract", "v1")]);
        a.result = "implementation_or_ux_refinement_required".into();
        store.append(&a).unwrap();
        let mut b = receipt("early_preview", &[("contract", "v2")]);
        b.result = "direction_confirmed".into();
        store.append(&b).unwrap();
        let got = store
            .latest("feature", "F-0001", "early_preview")
            .unwrap()
            .unwrap();
        assert_eq!(got.result, "direction_confirmed");
        assert_eq!(got.bindings.get("contract").unwrap(), "v2");
    }

    #[test]
    fn evaluate_detects_absent_current_and_stale() {
        let r = receipt(
            "early_preview",
            &[("contract", "hash-a"), ("intent", "hash-x")],
        );
        assert_eq!(evaluate(None, &bindings(&[])), ReceiptState::Absent);
        assert!(matches!(
            evaluate(
                Some(r.clone()),
                &bindings(&[("contract", "hash-a"), ("intent", "hash-x")])
            ),
            ReceiptState::Current(_)
        ));
        match evaluate(
            Some(r),
            &bindings(&[("contract", "hash-B"), ("intent", "hash-x")]),
        ) {
            ReceiptState::Stale {
                binding, was, now, ..
            } => {
                assert_eq!(binding, "contract");
                assert_eq!(was, "hash-a");
                assert_eq!(now, "hash-B");
            }
            other => panic!("expected Stale, got {other:?}"),
        }
    }
}
