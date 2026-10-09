//! Review Input Manifest (Reviewer Tool Contract v17): the single derived identity graph for one
//! review. Every identity here is computed from its canonical artifact; nothing is transcribed.

use crate::config::Config;
use crate::packet::{sha256_file, sha256_str, ReviewBasis, ReviewPacket};
use codeos_workflow::checker::Subject;
use codeos_workflow::contract::Workflow;
use codeos_workflow::project::Project;
use codeos_workflow::receipts::Receipt;
use codeos_workflow::verification::VerificationRecord;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CONTRACT: &str = "dba/04-tools/reviewer/contract/v17.md";
pub const CONTRACT_V18: &str = "dba/04-tools/reviewer/contract/v18.md";
pub const CONTRACT_V19: &str = "dba/04-tools/reviewer/contract/v19.md";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReviewInputManifest {
    pub feature: String,
    pub stage: String,
    /// The review identity this manifest was derived for; it names the review operation.
    pub operation: String,
    /// The predecessor review a bounded operation targets; `None` for an ordinary review.
    pub lineage_target: Option<String>,
    pub semantic_revision: Option<u64>,
    pub configuration: Option<String>,
    pub implementation: Option<String>,
    /// DBA-23's immutable F6-to-F9 candidate identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_bundle: Option<String>,
    /// Canonical governed path -> SHA-256, or `None` when the artifact is absent.
    pub governed_context: BTreeMap<String, Option<String>>,
    /// `<checkpoint>/<verification>` -> SHA-256 of the latest record line, or `None` when absent.
    pub verification: BTreeMap<String, Option<String>>,
    /// The production evidence F6 selected, as recorded by its reconciliation receipt.
    pub production_evidence: Option<String>,
    /// The runtime event evidence bound by the F7 runtime-event verification.
    pub runtime_evidence: Option<String>,
    pub required_inputs: BTreeSet<String>,
    /// The subset of required inputs CodeOS derived itself, as opposed to caller-declared ones.
    #[serde(default)]
    pub derived_inputs: BTreeSet<String>,
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
    /// Architecture Scopes could not be read, so the applicable scope is unknown.
    pub scope_error: Option<String>,
}

pub fn selected(contract: Option<&str>) -> bool {
    matches!(contract, Some(CONTRACT) | Some(CONTRACT_V18) | Some(CONTRACT_V19))
}

pub fn candidate_lifecycle(contract: Option<&str>) -> bool {
    matches!(contract, Some(CONTRACT_V18) | Some(CONTRACT_V19))
}

/// Derive the manifest from canonical artifacts. It precedes packet construction, which takes its
/// required inputs from here; `seal` later binds it to the final packet.
pub fn derive(
    cfg: &Config,
    feature: &str,
    stage: &str,
    declared: &[String],
    basis: &ReviewBasis,
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
    match crate::cmd::inspect_architecture_scopes::owning_scope_path(root, feature) {
        Ok(Some(scope)) if !governed.contains(&scope) => governed.push(scope),
        Ok(_) => {}
        Err(error) => manifest.derived.scope_error = Some(error),
    }
    for path in governed {
        manifest.governed_context.insert(path.clone(), hash(&path));
        manifest.derived_inputs.insert(path.clone());
        manifest.required_inputs.insert(path);
    }

    let project = match Project::discover(root) {
        Ok(project) => project,
        Err(error) => {
            manifest.derived.project_error = Some(error.to_string());
            return manifest;
        }
    };
    if candidate_lifecycle(crate::config::resolve_reviewer_tool_contract(cfg).ok().as_deref())
        && stage == "8"
    {
        match codeos_workflow::candidate::unique_current(&project, feature) {
            Ok(Some(id)) => {
                let path = format!(".codeos/06-workflow/candidates/{id}.json");
                manifest.candidate_bundle = Some(id);
                manifest.derived_inputs.insert(path.clone());
                manifest.required_inputs.insert(path);
            }
            Ok(None) => manifest.derived.project_error = Some(
                "no unique current Candidate Bundle; run codeos-workflow prepare-candidate".into(),
            ),
            Err(error) => manifest.derived.project_error = Some(error.to_string()),
        }
    }
    let implementation_paths = project.implementation_paths();
    manifest.implementation =
        codeos_workflow::hashing::working_tree_state(&project.root, &implementation_paths).ok();
    for path in changed_paths(root, basis) {
        if implementation_paths
            .iter()
            .any(|scope| path == *scope || path.starts_with(&format!("{scope}/")))
        {
            manifest.derived_inputs.insert(path.clone());
            manifest.required_inputs.insert(path);
        }
    }

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
        let path = format!(".codeos/06-workflow/reality-evidence/{selected}.json");
        manifest.derived_inputs.insert(path.clone());
        manifest.required_inputs.insert(path);
    }

    let store = std::fs::read_to_string(root.join(".codeos/06-workflow/verifications.jsonl"))
        .unwrap_or_default();
    let subject = Subject::resolve(&project, Workflow::Feature, feature);
    let mut required = subject
        .as_ref()
        .ok()
        .and_then(|subject| {
            codeos_workflow::checker::required_verification_records(&project, subject).ok()
        })
        .unwrap_or_default();
    // Runtime-event verification is optional evidence; when recorded it is bound like the rest.
    let has_runtime_events = store.lines().any(|line| {
        serde_json::from_str::<VerificationRecord>(line).is_ok_and(|record| {
            record.subject == feature && record.verification == "runtime_event_schema"
        })
    });
    if has_runtime_events {
        required.push(("F7", "runtime_event_schema"));
    }
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
        manifest.derived_inputs.insert(path.clone());
        manifest.required_inputs.insert(path.clone());
    }
    // A production-source record references its evidence by identity; the evidence itself is the
    // inspectable required input.
    if let Some(referenced) = manifest.referenced_production_evidence() {
        let path = format!(".codeos/06-workflow/reality-evidence/{referenced}.json");
        manifest.derived_inputs.insert(path.clone());
        manifest.required_inputs.insert(path);
    }
    manifest
}

/// The event-set identity of the latest passing F7 runtime-event verification for `feature`.
pub fn current_event_evidence(root: &std::path::Path, feature: &str) -> Option<String> {
    let store =
        std::fs::read_to_string(root.join(".codeos/06-workflow/verifications.jsonl")).ok()?;
    store.lines().rev().find_map(|line| {
        let record: VerificationRecord = serde_json::from_str(line).ok()?;
        (record.workflow == "feature"
            && record.subject == feature
            && record.verification == "runtime_event_schema"
            && record.result == "passed")
            .then(|| {
                record
                    .details?
                    .get("event_set_sha256")?
                    .as_str()
                    .map(str::to_string)
            })
            .flatten()
    })
}

/// Every path in the implementation delta: the bounded delta from the reviewed snapshot, or the
/// working-tree change from `--base`/HEAD plus untracked files.
fn changed_paths(root: &std::path::Path, basis: &ReviewBasis) -> BTreeSet<String> {
    let root = root.to_string_lossy();
    match basis {
        ReviewBasis::Bounded { delta, .. } => delta
            .changed_paths
            .union(&delta.new_paths)
            .cloned()
            .collect(),
        ReviewBasis::Ordinary { base } => {
            let mut paths: BTreeSet<String> =
                crate::packet::diff_changed_paths(base.as_deref(), &root)
                    .unwrap_or_default()
                    .into_iter()
                    .collect();
            paths.extend(crate::packet::git_untracked_files(&root).unwrap_or_default());
            paths
        }
    }
}

impl ReviewInputManifest {
    /// The production evidence identity the F5 production-source record references.
    pub fn referenced_production_evidence(&self) -> Option<String> {
        let (_, record) = self.derived.records.get("F5/production_source_evidence")?;
        record
            .details
            .as_ref()?
            .get("evidence_sha256")?
            .as_str()
            .map(str::to_string)
    }

    /// (CodeOS-derived required bytes, caller-declared bytes) counted toward the packet budget.
    pub fn size_split(&self, packet: &ReviewPacket) -> (u64, u64) {
        packet
            .budget_contributors
            .iter()
            .fold((0, 0), |(derived, declared), (path, bytes)| {
                if path.starts_with('(') || self.derived_inputs.contains(path) {
                    (derived + bytes, declared)
                } else {
                    (derived, declared + bytes)
                }
            })
    }

    pub fn print_size(&self, packet: &ReviewPacket) {
        let (derived, declared) = self.size_split(packet);
        let kb = |bytes: u64| bytes.div_ceil(1024);
        eprintln!("  required inputs (derived): {} KB", kb(derived));
        eprintln!("  declared inputs:           {} KB", kb(declared));
        eprintln!(
            "  total:                     {} KB",
            kb(packet.review_content_bytes)
        );
        eprintln!(
            "  budget:                    {} KB",
            kb(packet.budget_bytes)
        );
    }

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
