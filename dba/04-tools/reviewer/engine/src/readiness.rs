//! Review Readiness Gate (Review Policy v15, Reviewer Tool Contract v17). Runs before every
//! reviewer invocation; any issue is `REVIEW_NOT_READY` and consumes no review capacity.

use crate::config::Config;
use crate::manifest::ReviewInputManifest;
use crate::packet::{sha256_file, ReviewPacket};
use serde_json::Value;
use std::path::Path;

/// Declared in contract order, which is also report order: environment and toolchain causes come
/// before the input defects they can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    if crate::manifest::candidate_lifecycle(
        crate::config::resolve_reviewer_tool_contract(cfg)
            .ok()
            .as_deref(),
    ) && manifest.stage == "8"
        && manifest.candidate_bundle.is_none()
    {
        readiness.fail(
            Reason::RequiredInputMissing,
            "candidate_bundle",
            manifest.derived.project_error.clone().unwrap_or_else(|| {
                "no unique current Candidate Bundle; run codeos-workflow prepare-candidate".into()
            }),
        );
    }
    if manifest.stage == "8" {
        evidence(cfg, manifest, &claims, &mut readiness);
    }
    inspectability(manifest, packet, caller_sha_only, &mut readiness);
    coverage(packet, &mut readiness);
    readiness.issues.sort_by_key(|issue| issue.reason);
    readiness
}

fn environment(readiness: &mut Readiness) {
    let codex = match crate::codex::resolve_codex_executable() {
        Ok(codex) => codex,
        Err(error) => {
            readiness.fail(
                Reason::ExecutionEnvironmentBlocked,
                "codex",
                error.to_string(),
            );
            return;
        }
    };
    // Local credential probe; it makes no model request.
    let authenticated = std::process::Command::new(&codex)
        .args(["login", "status"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if !authenticated {
        readiness.fail(
            Reason::ExecutionEnvironmentBlocked,
            "codex credentials",
            "`codex login status` reports no usable login",
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
        // Feature Workflow v11 production evidence binds its Evidence Scopes and verifier; the
        // workflow derives their current values. Everything else binds the implementation state.
        let implementation_current = if record.bindings.contains_key("evidence_scope_state") {
            manifest
                .derived
                .scoped_bindings
                .get(key)
                .is_some_and(|now| {
                    ["evidence_scope_state", "reality_verifier"]
                        .iter()
                        .all(|binding| record.bindings.get(*binding) == now.get(*binding))
                })
        } else {
            record
                .bindings
                .get("implementation_state")
                .map(String::as_str)
                == current
        };
        if !implementation_current {
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

    test_evidence(cfg, manifest, readiness);

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
    // A revalidation keeps the historical F6 receipt; only its selected evidence must be current.
    // A v11 F6 receipt binds the non-test product state in place of the implementation state.
    let f6_bound = match receipt.bindings.get("product_state") {
        Some(bound) if !receipt.bindings.contains_key("implementation_state") => {
            Some(bound.as_str()) == manifest.derived.product_state.as_deref()
        }
        _ => {
            receipt
                .bindings
                .get("implementation_state")
                .map(String::as_str)
                == current
        }
    };
    if !manifest.derived.revalidation && !f6_bound {
        readiness.fail(
            Reason::EvidenceStale,
            "F6",
            "reconciliation is bound to a different implementation",
        );
    }
    if let Some(bound) = receipt
        .bindings
        .get("codeos_yaml")
        .filter(|_| !manifest.derived.revalidation)
    {
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
    if let Some(referenced) = manifest.referenced_production_evidence() {
        let path = cfg.repo_root.join(format!(
            ".codeos/06-workflow/reality-evidence/{referenced}.json"
        ));
        match sha256_file(path.to_string_lossy().as_ref()) {
            Err(_) => readiness.fail(
                Reason::RequiredInputMissing,
                "F5/production_source_evidence",
                "referenced production evidence is absent",
            ),
            Ok(actual) if actual != referenced => readiness.fail(
                Reason::F6SelectedEvidenceMismatch,
                "F5/production_source_evidence",
                "referenced production evidence content does not match its identity",
            ),
            Ok(_) => {}
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

/// A new candidate assessment needs what each test-based verification checked, not only that it
/// passed: each such record must bind per-test evidence whose content matches its identity.
fn test_evidence(cfg: &Config, manifest: &ReviewInputManifest, readiness: &mut Readiness) {
    if manifest.candidate_bundle.is_some() {
        for (key, identity) in manifest.test_evidence() {
            let Some(identity) = identity else {
                readiness.fail(
                    Reason::RequiredInputNotInspectable,
                    &key,
                    "verification record binds no per-test evidence; re-run codeos-workflow check",
                );
                continue;
            };
            let path = cfg.repo_root.join(format!(
                "{}/{identity}.json",
                codeos_workflow::test_evidence::DIR
            ));
            match sha256_file(path.to_string_lossy().as_ref()) {
                Ok(actual) if actual == identity => {}
                Ok(_) => readiness.fail(
                    Reason::RequiredInputNotInspectable,
                    &key,
                    "per-test evidence content does not match its identity",
                ),
                Err(_) => readiness.fail(
                    Reason::RequiredInputNotInspectable,
                    &key,
                    "per-test evidence is absent",
                ),
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
    if let Some(error) = &manifest.derived.scope_error {
        readiness.fail(
            Reason::RequiredInputNotInspectable,
            ".codeos/02-architecture/scopes",
            format!("the applicable Architecture Scope cannot be determined: {error}"),
        );
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

/// The packet is scoped to required inputs, so any remaining exclusion or omission is a required
/// change the reviewer could not see. Redaction keeps its own coverage treatment.
fn coverage(packet: &ReviewPacket, readiness: &mut Readiness) {
    use crate::packet::CoverageState;
    if matches!(
        packet.coverage_state,
        CoverageState::PartialCoverage | CoverageState::CriticalOmission
    ) {
        for (path, reason, _) in &packet.excluded_paths {
            if reason.contains("redacted") {
                continue;
            }
            readiness.fail(
                Reason::RequiredInputNotInspectable,
                path,
                format!("required change is not in the packet ({reason})"),
            );
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use codeos_workflow::verification::VerificationRecord;

    fn record(verification: &str, evidence: Option<&str>) -> (String, VerificationRecord) {
        let record = VerificationRecord {
            workflow: "feature".into(),
            subject: "F-1".into(),
            checkpoint: "F5".into(),
            verification: verification.into(),
            result: "passed".into(),
            bindings: Default::default(),
            command: "fixture".into(),
            timestamp: "2026-10-10T00:00:00Z".into(),
            details: evidence
                .map(|sha| serde_json::json!({ codeos_workflow::test_evidence::DETAILS_KEY: sha })),
        };
        (serde_json::to_string(&record).unwrap(), record)
    }

    #[test]
    fn only_a_revalidation_candidate_keeps_its_historical_f6_receipt() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = Config {
            reasoning_effort: String::new(),
            reviewer_model: String::new(),
            codex_cli_version: String::new(),
            packet_budget_mode: crate::config::PacketBudgetMode::Fail,
            repo_root: dir.path().into(),
            toolkit_root: dir.path().into(),
            review_log: dir.path().join("log"),
            codex_dir: dir.path().into(),
            state_dir: dir.path().into(),
        };
        let mut manifest = ReviewInputManifest {
            implementation: Some("now".into()),
            configuration: Some("config-now".into()),
            ..Default::default()
        };
        manifest.derived.f6_receipt = Some(codeos_workflow::receipts::Receipt {
            workflow: "feature".into(),
            subject: "F-1".into(),
            checkpoint: "reconciliation".into(),
            result: "completed".into(),
            bindings: [
                ("implementation_state".to_string(), "before".to_string()),
                ("codeos_yaml".to_string(), "config-before".to_string()),
            ]
            .into(),
            timestamp: "2026-10-01T00:00:00Z".into(),
            rationale: None,
            observation: None,
        });
        let f6_issues = |manifest: &ReviewInputManifest| {
            let mut readiness = Readiness::default();
            evidence(&cfg, manifest, &[], &mut readiness);
            readiness
                .issues
                .into_iter()
                .filter(|issue| issue.subject == "F6")
                .count()
        };
        // An ordinary candidate still needs F6 reconciled against the current state.
        assert_eq!(f6_issues(&manifest), 2);
        manifest.derived.revalidation = true;
        assert_eq!(f6_issues(&manifest), 0);
    }

    #[test]
    fn v11_evidence_and_f6_are_judged_by_their_scoped_bindings() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = Config {
            reasoning_effort: String::new(),
            reviewer_model: String::new(),
            codex_cli_version: String::new(),
            packet_budget_mode: crate::config::PacketBudgetMode::Fail,
            repo_root: dir.path().into(),
            toolkit_root: dir.path().into(),
            review_log: dir.path().join("log"),
            codex_dir: dir.path().into(),
            state_dir: dir.path().into(),
        };
        let key = "F5/production_source_evidence".to_string();
        let scoped: std::collections::BTreeMap<String, String> = [
            ("evidence_scope_state".to_string(), "scope-1".to_string()),
            ("reality_verifier".to_string(), "verifier-1".to_string()),
        ]
        .into();
        let record = VerificationRecord {
            workflow: "feature".into(),
            subject: "F-1".into(),
            checkpoint: "F5".into(),
            verification: "production_source_evidence".into(),
            result: "passed".into(),
            bindings: scoped.clone(),
            command: "fixture".into(),
            timestamp: "2026-10-16T00:00:00Z".into(),
            details: None,
        };
        let mut manifest = ReviewInputManifest {
            // The implementation moved; scoped evidence does not depend on it.
            implementation: Some("implementation-2".into()),
            ..Default::default()
        };
        manifest
            .verification
            .insert(key.clone(), Some("line".into()));
        manifest
            .derived
            .records
            .insert(key.clone(), (String::new(), record));
        manifest.derived.scoped_bindings.insert(key.clone(), scoped);
        manifest.derived.product_state = Some("product-1".into());
        manifest.derived.f6_receipt = Some(codeos_workflow::receipts::Receipt {
            workflow: "feature".into(),
            subject: "F-1".into(),
            checkpoint: "reconciliation".into(),
            result: "completed".into(),
            bindings: [("product_state".to_string(), "product-1".to_string())].into(),
            timestamp: "2026-10-16T00:00:00Z".into(),
            rationale: None,
            observation: None,
        });
        let stale = |manifest: &ReviewInputManifest| {
            let mut readiness = Readiness::default();
            evidence(&cfg, manifest, &[], &mut readiness);
            readiness
                .issues
                .into_iter()
                .filter(|issue| issue.reason == Reason::EvidenceStale)
                .map(|issue| issue.subject)
                .collect::<Vec<_>>()
        };
        assert!(stale(&manifest).is_empty(), "{:?}", stale(&manifest));

        // The verifier changed, and so did the product F6 judged.
        manifest
            .derived
            .scoped_bindings
            .get_mut(&key)
            .unwrap()
            .insert("reality_verifier".into(), "verifier-2".into());
        manifest.derived.product_state = Some("product-2".into());
        assert_eq!(stale(&manifest), vec![key, "F6".to_string()]);
    }

    #[test]
    fn candidate_assessment_requires_bound_per_test_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = Config {
            reasoning_effort: String::new(),
            reviewer_model: String::new(),
            codex_cli_version: String::new(),
            packet_budget_mode: crate::config::PacketBudgetMode::Fail,
            repo_root: dir.path().into(),
            toolkit_root: dir.path().into(),
            review_log: dir.path().join("log"),
            codex_dir: dir.path().into(),
            state_dir: dir.path().into(),
        };
        let stored = b"{\"schema\":1}";
        let identity = crate::packet::sha256_str(std::str::from_utf8(stored).unwrap());
        let store = dir.path().join(codeos_workflow::test_evidence::DIR);
        std::fs::create_dir_all(&store).unwrap();
        std::fs::write(store.join(format!("{identity}.json")), stored).unwrap();
        let other = "0".repeat(64);
        std::fs::write(store.join(format!("{other}.json")), b"tampered").unwrap();

        let mut manifest = ReviewInputManifest::default();
        for (key, entry) in [
            ("F5/behavior", record("behavior", Some(&identity))),
            ("F5/repeatability", record("repeatability", None)),
            ("F5g/playwright", record("playwright", Some(&other))),
            ("F3/smoke", record("smoke", None)),
        ] {
            manifest.derived.records.insert(key.into(), entry);
        }
        let issues = |manifest: &ReviewInputManifest| {
            let mut readiness = Readiness::default();
            test_evidence(&cfg, manifest, &mut readiness);
            readiness
                .issues
                .into_iter()
                .map(|issue| (issue.reason, issue.subject))
                .collect::<Vec<_>>()
        };
        // Legacy assessments without a Candidate Bundle are unaffected.
        assert!(issues(&manifest).is_empty());
        manifest.candidate_bundle = Some("bundle".into());
        assert_eq!(
            issues(&manifest),
            [
                (
                    Reason::RequiredInputNotInspectable,
                    "F5/repeatability".into()
                ),
                (Reason::RequiredInputNotInspectable, "F5g/playwright".into()),
            ]
        );
    }
}
