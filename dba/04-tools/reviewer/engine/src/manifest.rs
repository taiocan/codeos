//! Review Input Manifest (Reviewer Tool Contract v17): the single derived identity graph for one
//! review. Every identity here is computed from its canonical artifact; nothing is transcribed.

use crate::config::Config;
use crate::packet::{sha256_file, sha256_str, ReviewPacket};
use codeos_workflow::checker::Subject;
use codeos_workflow::contract::Workflow;
use codeos_workflow::project::Project;
use codeos_workflow::receipts::Receipt;
use codeos_workflow::verification::VerificationRecord;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CONTRACT: &str = "dba/04-tools/reviewer/contract/v17.md";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReviewInputManifest {
    pub feature: String,
    pub stage: String,
    /// The review identity this manifest was derived for; it names the review operation.
    pub operation: String,
    pub semantic_revision: Option<u64>,
    pub configuration: Option<String>,
    pub implementation: Option<String>,
    /// Canonical governed path -> SHA-256, or `None` when the artifact is absent.
    pub governed_context: BTreeMap<String, Option<String>>,
    /// `<checkpoint>/<verification>` -> SHA-256 of the latest record line, or `None` when absent.
    pub verification: BTreeMap<String, Option<String>>,
    /// The production evidence F6 selected, as recorded by its reconciliation receipt.
    pub production_evidence: Option<String>,
    /// The runtime event evidence bound by the F7 runtime-event verification.
    pub runtime_evidence: Option<String>,
    pub required_inputs: BTreeSet<String>,
    /// Content identity of everything the reviewer receives; set once the packet is final.
    pub reviewed_snapshot: String,
    #[serde(skip)]
    pub(crate) derived: Derived,
}

/// Canonical material the Review Readiness Gate inspects but the manifest only identifies.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Derived {
    pub f6_receipt: Option<Receipt>,
    /// `<checkpoint>/<verification>` -> (exact stored line, parsed record).
    pub records: BTreeMap<String, (String, VerificationRecord)>,
    pub project_error: Option<String>,
}

pub fn selected(contract: Option<&str>) -> bool {
    contract == Some(CONTRACT)
}

/// Derive the manifest from canonical artifacts. It precedes packet construction, which takes its
/// required inputs from here; `seal` later binds it to the final packet.
pub fn derive(
    cfg: &Config,
    feature: &str,
    stage: &str,
    declared: &[String],
) -> ReviewInputManifest {
    let root = &cfg.repo_root;
    let hash = |rel: &str| {
        let path = root.join(rel);
        path.is_file()
            .then(|| sha256_file(path.to_string_lossy().as_ref()).ok())
            .flatten()
    };
    let mut manifest = ReviewInputManifest {
        feature: feature.to_string(),
        stage: stage.to_string(),
        configuration: hash(".codeos/00-project/codeos.yaml"),
        ..Default::default()
    };
    manifest.required_inputs = declared.iter().cloned().collect();
    if stage != "8" {
        return manifest;
    }

    manifest.semantic_revision = crate::cmd::review::approved_stage8_cycle(cfg, feature)
        .ok()
        .flatten();
    let mut governed = vec![
        ".codeos/00-project/charter.md".to_string(),
        format!(".codeos/01-specification/intents/{feature}.md"),
        format!(".codeos/01-specification/contracts/{feature}_contract.md"),
        format!(".codeos/01-specification/event-schemas/{feature}_schema.md"),
    ];
    governed.extend(
        declared
            .iter()
            .filter(|path| path.starts_with(".codeos/02-architecture/scopes/"))
            .cloned(),
    );
    for path in governed {
        manifest.governed_context.insert(path.clone(), hash(&path));
        manifest.required_inputs.insert(path);
    }

    let project = match Project::discover(root) {
        Ok(project) => project,
        Err(error) => {
            manifest.derived.project_error = Some(error.to_string());
            return manifest;
        }
    };
    manifest.implementation = codeos_workflow::hashing::working_tree_state(
        &project.root,
        &project.implementation_paths(),
    )
    .ok();

    let decisions = std::fs::read_to_string(root.join(".codeos/06-workflow/decisions.jsonl"))
        .unwrap_or_default();
    manifest.derived.f6_receipt = decisions
        .lines()
        .filter_map(|line| serde_json::from_str::<Receipt>(line).ok())
        .rfind(|receipt| {
            receipt.workflow == "feature"
                && receipt.subject == feature
                && receipt.checkpoint == "reconciliation"
        });
    manifest.production_evidence = manifest
        .derived
        .f6_receipt
        .as_ref()
        .and_then(|receipt| receipt.bindings.get("production_evidence").cloned());
    if let Some(selected) = &manifest.production_evidence {
        manifest.required_inputs.insert(format!(
            ".codeos/06-workflow/reality-evidence/{selected}.json"
        ));
    }

    let store = std::fs::read_to_string(root.join(".codeos/06-workflow/verifications.jsonl"))
        .unwrap_or_default();
    let subject = Subject::resolve(&project, Workflow::Feature, feature);
    let required = subject
        .as_ref()
        .map(codeos_workflow::revalidation::required_verifications)
        .unwrap_or_default();
    for (checkpoint, verification) in required {
        let key = format!("{checkpoint}/{verification}");
        let latest = store.lines().rev().find_map(|line| {
            let record: VerificationRecord = serde_json::from_str(line).ok()?;
            (record.workflow == "feature"
                && record.subject == feature
                && record.checkpoint == checkpoint
                && record.verification == verification)
                .then(|| (line.to_string(), record))
        });
        manifest.verification.insert(
            key.clone(),
            latest.as_ref().map(|(line, _)| sha256_str(line)),
        );
        if let Some(entry) = latest {
            manifest.derived.records.insert(key, entry);
        }
    }
    manifest.runtime_evidence = manifest
        .derived
        .records
        .get("F7/runtime_event_schema")
        .and_then(|(_, record)| record.details.as_ref())
        .and_then(|details| details.get("event_evidence"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    if let Some(path) = &manifest.runtime_evidence {
        manifest.required_inputs.insert(path.clone());
    }
    manifest
}

impl ReviewInputManifest {
    /// Required verification record lines the packet does not already carry verbatim.
    pub fn records_missing_from(&self, packet: &ReviewPacket) -> Vec<String> {
        self.derived
            .records
            .values()
            .map(|(line, _)| line.clone())
            .filter(|line| !packet.content().contains(line.as_str()))
            .collect()
    }

    /// Required inputs that exist on disk; packet construction includes each in full.
    pub fn existing_required_inputs(&self, repo_root: &std::path::Path) -> Vec<String> {
        self.required_inputs
            .iter()
            .filter(|path| repo_root.join(path).is_file())
            .cloned()
            .collect()
    }

    /// Bind the manifest to its review operation and the exact bytes the reviewer will receive.
    pub fn seal(&mut self, operation: &str, packet: &ReviewPacket) {
        self.operation = operation.to_string();
        let mut identity = format!(
            "review_commit={}\ndiff={}\npacket={}\n",
            packet.review_sha,
            packet.diff_hash,
            sha256_str(packet.content())
        );
        for entry in &packet.artifacts {
            identity.push_str(&format!("{}={}\n", entry.path, entry.sha256));
        }
        self.reviewed_snapshot = sha256_str(&identity);
    }
}
