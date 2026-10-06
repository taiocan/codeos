//! Review Readiness Gate (Review Policy v15, Reviewer Tool Contract v17). Runs before every
//! reviewer invocation; any issue is `REVIEW_NOT_READY` and consumes no review capacity.

use crate::config::Config;
use crate::manifest::ReviewInputManifest;
use crate::packet::{sha256_file, ReviewPacket};
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    ExecutionEnvironmentBlocked,
    ReviewerBuildMismatch,
    ConfigurationMismatch,
    ImplementationIdentityInvalid,
    F6SelectedEvidenceMismatch,
    EvidenceStale,
    RequiredInputMissing,
    RequiredInputNotInspectable,
    SnapshotNotReconstructible,
    LineageInvalid,
}

impl Reason {
    pub fn code(self) -> &'static str {
        match self {
            Self::ExecutionEnvironmentBlocked => "EXECUTION_ENVIRONMENT_BLOCKED",
            Self::ReviewerBuildMismatch => "REVIEWER_BUILD_MISMATCH",
            Self::ConfigurationMismatch => "CONFIGURATION_MISMATCH",
            Self::ImplementationIdentityInvalid => "IMPLEMENTATION_IDENTITY_INVALID",
            Self::F6SelectedEvidenceMismatch => "F6_SELECTED_EVIDENCE_MISMATCH",
            Self::EvidenceStale => "EVIDENCE_STALE",
            Self::RequiredInputMissing => "REQUIRED_INPUT_MISSING",
            Self::RequiredInputNotInspectable => "REQUIRED_INPUT_NOT_INSPECTABLE",
            Self::SnapshotNotReconstructible => "SNAPSHOT_NOT_RECONSTRUCTIBLE",
            Self::LineageInvalid => "LINEAGE_INVALID",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub reason: Reason,
    pub subject: String,
    pub detail: String,
}

#[derive(Debug, Default)]
pub struct Readiness {
    pub issues: Vec<Issue>,
}

impl Readiness {
    pub fn passes(&self) -> bool {
        self.issues.is_empty()
    }

    fn fail(&mut self, reason: Reason, subject: impl Into<String>, detail: impl Into<String>) {
        self.issues.push(Issue {
            reason,
            subject: subject.into(),
            detail: detail.into(),
        });
    }

    /// Prepare-time issues (bounded snapshot or lineage resolution) refused before a packet exists.
    pub fn refusal(reason: Reason, subject: &str, detail: &str) -> Self {
        let mut readiness = Self::default();
        readiness.fail(reason, subject, detail);
        readiness
    }

    pub fn print_plan(&self) {
        if self.passes() {
            println!("  review_readiness: PASS");
            return;
        }
        println!(
            "  review_readiness: REVIEW_NOT_READY: {}",
            self.issues[0].reason.code()
        );
        for issue in &self.issues {
            println!(
                "    - {}: {}: {}",
                issue.reason.code(),
                issue.subject,
                issue.detail
            );
        }
    }

    pub fn print_refusal(&self) {
        eprintln!("REVIEW_NOT_READY: {}", self.issues[0].reason.code());
        for issue in &self.issues {
            eprintln!(
                "  - {}: {}: {}",
                issue.reason.code(),
                issue.subject,
                issue.detail
            );
        }
        eprintln!("  The reviewer was not invoked. No review capacity was consumed.");
    }
}

/// Evaluate every readiness condition for one sealed manifest and its final packet.
pub fn evaluate(
    cfg: &Config,
    manifest: &ReviewInputManifest,
    packet: &ReviewPacket,
    caller_sha_only: &[String],
) -> Readiness {
    let mut readiness = Readiness::default();
    environment(&mut readiness);
    toolchain(cfg, &mut readiness);
    let claims = declared_json_claims(cfg, packet);
    identities(manifest, &claims, &mut readiness);
    if manifest.stage == "8" {
        evidence(cfg, manifest, &claims, &mut readiness);
    }
    inspectability(manifest, packet, caller_sha_only, &mut readiness);
    readiness
}

fn environment(readiness: &mut Readiness) {
    if let Err(error) = crate::codex::resolve_codex_executable() {
        readiness.fail(
            Reason::ExecutionEnvironmentBlocked,
            "codex",
            error.to_string(),
        );
    }
}

fn toolchain(cfg: &Config, readiness: &mut Readiness) {
    let engine = cfg.toolkit_root.join("dba/04-tools/reviewer/engine");
    match crate::build_identity::build_source_sha256(&engine) {
        Ok(current) if current == env!("CODEOS_REVIEWER_BUILD_SHA256") => {}
        Ok(_) => readiness.fail(
            Reason::ReviewerBuildMismatch,
            "codeos-reviewer",
            "the running binary was built from different reviewer sources; rebuild it",
        ),
        Err(error) => readiness.fail(
            Reason::ReviewerBuildMismatch,
            "codeos-reviewer",
            format!(
                "cannot read reviewer sources at {}: {error}",
                engine.display()
            ),
        ),
    }
}

/// Declared JSON evidence and the identities it claims.
fn declared_json_claims(cfg: &Config, packet: &ReviewPacket) -> Vec<(String, Value)> {
    packet
        .artifacts
        .iter()
        .filter(|entry| entry.path.ends_with(".json"))
        .filter(|entry| {
            !entry
                .path
                .starts_with(".codeos/06-workflow/reality-evidence/")
        })
        .filter_map(|entry| {
            let bytes = std::fs::read(cfg.repo_root.join(&entry.path)).ok()?;
            Some((entry.path.clone(), serde_json::from_slice(&bytes).ok()?))
        })
        .collect()
}

fn identities(
    manifest: &ReviewInputManifest,
    claims: &[(String, Value)],
    readiness: &mut Readiness,
) {
    for (path, value) in claims {
        if let Some(claimed) = value
            .pointer("/bindings/codeos_yaml")
            .and_then(Value::as_str)
        {
            if Some(claimed) != manifest.configuration.as_deref() {
                readiness.fail(
                    Reason::ConfigurationMismatch,
                    path,
                    "claimed codeos.yaml identity differs from the canonical artifact",
                );
            }
        }
        if manifest.stage != "8" {
            continue;
        }
        if let Some(claimed) = value
            .pointer("/bindings/implementation_state")
            .and_then(Value::as_str)
        {
            if Some(claimed) != manifest.implementation.as_deref() {
                readiness.fail(
                    Reason::ImplementationIdentityInvalid,
                    path,
                    "claimed implementation identity differs from the current implementation",
                );
            }
        }
    }
    if manifest.stage == "8" && manifest.implementation.is_none() {
        let detail = manifest
            .derived
            .project_error
            .clone()
            .unwrap_or_else(|| "implementation identity could not be derived".to_string());
        readiness.fail(
            Reason::ImplementationIdentityInvalid,
            "implementation",
            detail,
        );
    }
}

fn evidence(
    cfg: &Config,
    manifest: &ReviewInputManifest,
    claims: &[(String, Value)],
    readiness: &mut Readiness,
) {
    let current = manifest.implementation.as_deref();
    let governed = |key: &str| {
        let path = match key {
            "intent" => format!(".codeos/01-specification/intents/{}.md", manifest.feature),
            "contract" => format!(
                ".codeos/01-specification/contracts/{}_contract.md",
                manifest.feature
            ),
            "event_schema" => format!(
                ".codeos/01-specification/event-schemas/{}_schema.md",
                manifest.feature
            ),
            _ => return None,
        };
        manifest.governed_context.get(&path).cloned().flatten()
    };

    let mut environment_needed = false;
    for (key, sha) in &manifest.verification {
        let Some((_, record)) = manifest.derived.records.get(key) else {
            debug_assert!(sha.is_none());
            environment_needed = true;
            readiness.fail(
                Reason::RequiredInputMissing,
                key,
                "required verification record is absent",
            );
            continue;
        };
        if record.result != "passed" {
            environment_needed = true;
            readiness.fail(
                Reason::EvidenceStale,
                key,
                "latest verification did not pass",
            );
            continue;
        }
        if record
            .bindings
            .get("implementation_state")
            .map(String::as_str)
            != current
        {
            environment_needed = true;
            readiness.fail(
                Reason::EvidenceStale,
                key,
                "verification is bound to a different implementation",
            );
        }
        for binding in ["intent", "contract", "event_schema"] {
            if let Some(bound) = record.bindings.get(binding) {
                if Some(bound) != governed(binding).as_ref() {
                    readiness.fail(
                        Reason::EvidenceStale,
                        key,
                        format!("verification is bound to a different {binding}"),
                    );
                }
            }
        }
    }
    if environment_needed && !docker_available(cfg) {
        readiness.fail(
            Reason::ExecutionEnvironmentBlocked,
            "docker",
            "verification evidence must be refreshed but Docker is unavailable",
        );
    }

    let needs_production = manifest
        .verification
        .contains_key("F5/production_source_evidence");
    let Some(receipt) = manifest.derived.f6_receipt.as_ref() else {
        if needs_production {
            readiness.fail(
                Reason::RequiredInputMissing,
                "F6",
                "reality-dependent claims require an F6 reconciliation selection",
            );
        }
        return;
    };
    if receipt
        .bindings
        .get("implementation_state")
        .map(String::as_str)
        != current
    {
        readiness.fail(
            Reason::EvidenceStale,
            "F6",
            "reconciliation is bound to a different implementation",
        );
    }
    if let Some(bound) = receipt.bindings.get("codeos_yaml") {
        if Some(bound) != manifest.configuration.as_ref() {
            readiness.fail(
                Reason::EvidenceStale,
                "F6",
                "reconciliation is bound to a different codeos.yaml",
            );
        }
    }
    let Some(selected) = manifest.production_evidence.as_deref() else {
        if needs_production {
            readiness.fail(
                Reason::F6SelectedEvidenceMismatch,
                "F6",
                "reconciliation selects no production evidence",
            );
        }
        return;
    };
    let path = cfg.repo_root.join(format!(
        ".codeos/06-workflow/reality-evidence/{selected}.json"
    ));
    if let Ok(actual) = sha256_file(path.to_string_lossy().as_ref()) {
        if actual != selected {
            readiness.fail(
                Reason::F6SelectedEvidenceMismatch,
                "F6",
                "selected production evidence content does not match its identity",
            );
        }
        let bound = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            .and_then(|value| {
                value
                    .get("implementation_sha256")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            });
        if bound.as_deref() != current {
            readiness.fail(
                Reason::EvidenceStale,
                "F6",
                "selected production evidence is bound to a different implementation",
            );
        }
    }
    if let Some((_, record)) = manifest
        .derived
        .records
        .get("F5/production_source_evidence")
    {
        let verified = record
            .details
            .as_ref()
            .and_then(|details| details.get("evidence_sha256"))
            .and_then(Value::as_str);
        if verified != Some(selected) {
            readiness.fail(
                Reason::F6SelectedEvidenceMismatch,
                "F5/production_source_evidence",
                "verified production evidence differs from the F6 selection",
            );
        }
    }
    for (path, value) in claims {
        if let Some(claimed) = value
            .pointer("/verification/production_source/evidence_sha256")
            .and_then(Value::as_str)
        {
            if claimed != selected {
                readiness.fail(
                    Reason::F6SelectedEvidenceMismatch,
                    path,
                    "declared evidence cites production evidence F6 did not select",
                );
            }
        }
    }
}

fn inspectability(
    manifest: &ReviewInputManifest,
    packet: &ReviewPacket,
    caller_sha_only: &[String],
    readiness: &mut Readiness,
) {
    for path in &manifest.required_inputs {
        let Some(entry) = packet.artifacts.iter().find(|entry| &entry.path == path) else {
            readiness.fail(
                Reason::RequiredInputMissing,
                path,
                "required input is absent from the packet",
            );
            continue;
        };
        match entry.visibility.as_str() {
            "missing" => readiness.fail(Reason::RequiredInputMissing, path, "file is absent"),
            _ if entry.bytes == 0 => {
                readiness.fail(Reason::RequiredInputMissing, path, "file is empty")
            }
            "shown" | "shown_redacted" | "added_file" | "delta_diff" => {}
            // An unchanged file in a delta review was already reviewed; its identity suffices.
            "path_sha_only" if !caller_sha_only.contains(path) && !packet.base_sha.is_empty() => {}
            visibility => readiness.fail(
                Reason::RequiredInputNotInspectable,
                path,
                format!("content is not inspectable (visibility: {visibility})"),
            ),
        }
    }
    for line in manifest.records_missing_from(packet) {
        let key = serde_json::from_str::<Value>(&line)
            .ok()
            .map(|value| {
                format!(
                    "{}/{}",
                    value["checkpoint"].as_str().unwrap_or("?"),
                    value["verification"].as_str().unwrap_or("?")
                )
            })
            .unwrap_or_default();
        readiness.fail(
            Reason::RequiredInputNotInspectable,
            key,
            "required verification record content is not in the packet",
        );
    }
}

fn docker_available(cfg: &Config) -> bool {
    let compose = cfg.repo_root.join("docker-compose.yml");
    if !Path::new(&compose).is_file() {
        return true;
    }
    std::process::Command::new("docker")
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
