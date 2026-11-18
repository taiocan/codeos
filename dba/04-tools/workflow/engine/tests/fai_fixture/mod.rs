//! FAI fixtures shared by the Feature Workflow v11/v12 tests and the DQA qualification replay: a
//! project whose fake verifier produces real evidence through `check`, with the other mechanics
//! seeded or stubbed.
#![allow(dead_code)]

use codeos_workflow::checker::{
    bindings_for_decision, decision_state, evaluate, run_checks,
    verification_bindings_for_mechanic, State, Subject,
};
use codeos_workflow::contract::Workflow;
use codeos_workflow::execution::Environment;
use codeos_workflow::project::Project;
use codeos_workflow::receipts::{Receipt, ReceiptState, ReceiptStore};
use codeos_workflow::verification::{VerificationRecord, VerificationStore};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const VERIFIER: &str = r#"#!/usr/bin/env bash
set -euo pipefail
# Feature Workflow v13: the fake adapter's ledger records its one provider call.
[[ -z "${CODEOS_PROVIDER_LEDGER:-}" ]] || printf '{"schema":1,"attempt":1,"event":"attempt","at":"2026-10-16T00:00:00Z"}\n{"schema":1,"attempt":1,"event":"success","at":"2026-10-16T00:00:01Z"}\n' >> "$CODEOS_PROVIDER_LEDGER"
h=$(printf 'x' | sha256sum | cut -c1-64)
receipts='[{"provider_host":"provider.test","provider_path":"/route","request_sha256":"'$h'","response_sha256":"'$h'","provider_request_id":null,"requested_at":"2026-10-16T00:00:00Z","received_at":"2026-10-16T00:00:01Z","http_status":200}]'
visible='"ok"'
vs=$(printf '%s' "$visible" | sha256sum | cut -c1-64)
rs=$(printf '%s' "$receipts" | sha256sum | cut -c1-64)
jq -n --slurpfile d "$CODEOS_REALITY_DEFINITION_PATH" --slurpfile s claim-scopes.json \
  --argjson r "$receipts" --argjson v "$visible" --arg vs "$vs" --arg rs "$rs" '
  $d[0] as $d | {schema: $d.schema, subject: $d.subject, claims: $d.claims,
    source_kind: "production", provider: "provider.test", observed_at: "2026-10-16T00:00:00Z",
    governed_inputs: $d.governed_inputs, claim_scopes: ($s[0][$d.subject] // {}),
    observations: [{claim_ids: $d.claims, visible_result: $v, visible_result_sha256: $vs,
      provider_receipts_sha256: $rs, provider_request_response_receipts: $r, details: {}}]}' \
  > "$CODEOS_REALITY_EVIDENCE_PATH"
"#;

fn toolkit_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap()
}

pub struct Fixture {
    _dir: tempfile::TempDir,
    pub root: PathBuf,
}

impl Fixture {
    pub fn new(scopes: &str) -> Self {
        Self::under("DBA-27", scopes)
    }

    pub fn under(configuration: &str, scopes: &str) -> Self {
        let toolkit = toolkit_root();
        let dir = tempfile::tempdir().unwrap();
        let fx = Fixture {
            root: dir.path().canonicalize().unwrap(),
            _dir: dir,
        };
        fs::create_dir_all(fx.root.join(".codeos/06-workflow")).unwrap();
        std::os::unix::fs::symlink(toolkit, fx.root.join(".codeos/toolkit")).unwrap();
        fx.write(".gitignore", ".codeos/toolkit\nbackend/target\n");
        fx.write(
            ".codeos/00-project/codeos.yaml",
            &format!("codeos_schema: 2\ndba_configuration: {configuration}\nreality_verification:\n  command: scripts/verify.sh\n  scopes:\n{scopes}"),
        );
        fx.write(
            ".codeos/00-project/charter.md",
            "---\nartifact_type: charter\napproval:\n  by: Human\n  at: 2026-10-16\n---\n\n## Primary Product Journey\njourney_id: primary\nlegitimate_entry: product home\ninitial_frontier: entry\ntarget_outcome: useful result\nverification_route: product_journey\n",
        );
        fx.write(
            "backend/Cargo.toml",
            "[package]\nname = \"fai\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        fx.write("backend/Cargo.lock", "# rustls 0.23.45\n");
        fx.write("backend/src/lib.rs", "pub fn route() {}\n");
        fx.write("backend/tests/route.rs", "#[test]\nfn route() {}\n");
        fx.write("web/package.json", "{}\n");
        fx.write("web/src/page.svelte", "<h1>Compare</h1>\n");
        fx.write("docker-compose.yml", "services: {}\n");
        fx.write("scripts/verify.sh", VERIFIER);
        Command::new("chmod")
            .args(["+x", "scripts/verify.sh"])
            .current_dir(&fx.root)
            .status()
            .unwrap();
        fx.write("claim-scopes.json", "{}\n");
        fx.write(
            ".codeos/06-workflow/.verify-stub",
            "behavior=pass\nrepeatability=pass\n",
        );
        fx.git(&["init", "-q"]);
        fx.git(&["add", "-A"]);
        fx.git(&["commit", "-qm", "fixture"]);
        // Bootstrap: an accepted earlier feature satisfies the entry gate.
        ReceiptStore::at(&fx.root.join(".codeos"))
            .append(&Receipt {
                workflow: "feature".into(),
                subject: "F-0000".into(),
                checkpoint: "acceptance".into(),
                result: "accepted".into(),
                bindings: Default::default(),
                timestamp: "2026-10-01T00:00:00Z".into(),
                rationale: None,
                observation: None,
            })
            .unwrap();
        fx.spec("F-0000", "mode: preserve\nreason: earlier work", "");
        fx
    }

    pub fn write(&self, rel: &str, content: &str) {
        let path = self.root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    pub fn git(&self, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    }

    pub fn project(&self) -> Project {
        Project::discover(&self.root).unwrap()
    }

    pub fn subject(&self, feature: &str) -> Subject {
        Subject::resolve(&self.project(), Workflow::Feature, feature).unwrap()
    }

    /// A non-GUI Specification Package; `reality` holds extra Reality Basis rows.
    pub fn spec(&self, feature: &str, contribution: &str, reality: &str) {
        let front = |kind: &str| {
            format!("---\nartifact_type: {kind}\nfeature_id: {feature}\nstatus: APPROVED\nsemantic_revision: 1\nsemantic_change: initial feature specification\napproved_by: Human\napproved_at: 2026-10-16\n---\n\n")
        };
        self.write(
            &format!(".codeos/01-specification/intents/{feature}.md"),
            &format!("{}# Intent: {feature}\n", front("intent")),
        );
        self.write(
            &format!(".codeos/01-specification/contracts/{feature}_contract.md"),
            &format!("{}# Behavioral Contract: {feature}\n\n## Validation Questions\n\n**Behavior** — a real integration test proves it.\n**Repeatability** — the same test re-run twice both pass.\n\n## Feature Impact Accounting\n\n| Tier | Changed / Unchanged | Reason |\n|---|---|---|\n| Persistence (PostgreSQL) | unchanged | none |\n| Backend (Rust) | changed | route |\n| GUI (Svelte) | unchanged | none |\n\n## Product Journey Contribution\n{contribution}\n\n## Reality Basis\n\n| Claim ID | Claim | Classification | Source | Conditions | Currency | Unavailable | Fixture |\n|---|---|---|---|---|---|---|---|\n| label | Heading | not-reality-dependent | n/a | n/a | n/a | n/a | n/a |\n{reality}", front("contract")),
        );
        self.write(
            &format!(".codeos/01-specification/event-schemas/{feature}_schema.md"),
            &format!("{}# Event Schema: {feature}\n\n## Observation Mode\n\n**Mode:** `external-observation`\n", front("event_schema")),
        );
    }

    pub fn seed(&self, feature: &str, checkpoint: &str, mechanic: &str) {
        let subject = self.subject(feature);
        let bindings =
            verification_bindings_for_mechanic(&self.project(), &subject, checkpoint, mechanic)
                .unwrap();
        VerificationStore::at(&self.root.join(".codeos"))
            .append(&VerificationRecord {
                workflow: "feature".into(),
                subject: feature.into(),
                checkpoint: checkpoint.into(),
                verification: mechanic.into(),
                result: "passed".into(),
                bindings,
                command: "seeded".into(),
                timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                details: None,
            })
            .unwrap();
    }

    pub fn receipt(&self, feature: &str, checkpoint: &str, result: &str, evidence: Option<&str>) {
        let mut bindings =
            bindings_for_decision(&self.project(), &self.subject(feature), checkpoint).unwrap();
        if let Some(evidence) = evidence {
            bindings.insert("production_evidence".into(), evidence.into());
        }
        ReceiptStore::at(&self.root.join(".codeos"))
            .append(&Receipt {
                workflow: "feature".into(),
                subject: feature.into(),
                checkpoint: checkpoint.into(),
                result: result.into(),
                bindings,
                timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                rationale: Some("fixture".into()),
                observation: None,
            })
            .unwrap();
    }

    /// Seed F3, then let `check` run F5 with the fake verifier: under Feature Workflow v13, the
    /// deterministic run, the pre-candidate qualification, then the production run.
    pub fn check(&self, feature: &str) -> Vec<String> {
        self.seed(feature, "F3", "smoke");
        self.seed(feature, "F3", "product_journey");
        let run = |environment| run_checks(&self.project(), &self.subject(feature), environment);
        if !self.v13() {
            return run(None).unwrap();
        }
        let mut log = run(Some(Environment::Deterministic)).unwrap();
        self.seed(feature, "candidate", "project_qualification");
        log.extend(run(Some(Environment::Production)).unwrap());
        log
    }

    pub fn v13(&self) -> bool {
        self.project().feature_workflow_version().unwrap() >= 13
    }

    pub fn state(&self, feature: &str, checkpoint: &str) -> (State, Vec<String>) {
        let reports = evaluate(&self.project(), &self.subject(feature)).unwrap();
        let report = reports.into_iter().find(|r| r.id == checkpoint).unwrap();
        (report.state, report.unmet)
    }

    /// Whether the latest production-source evidence record is current (Absent, Failed and Stale
    /// are not), with the binding that drifted.
    pub fn evidence_current(&self, feature: &str) -> Result<(), String> {
        let project = self.project();
        let subject = self.subject(feature);
        let want = verification_bindings_for_mechanic(
            &project,
            &subject,
            "F5",
            "production_source_evidence",
        )
        .unwrap();
        let have = VerificationStore::at(&self.root.join(".codeos"))
            .latest("feature", feature, "F5", "production_source_evidence")
            .unwrap();
        match codeos_workflow::verification::evaluate(have, &want) {
            codeos_workflow::verification::RecordState::Current(_) => Ok(()),
            codeos_workflow::verification::RecordState::Stale { binding, .. } => Err(binding),
            _ => Err("absent or failed".into()),
        }
    }

    /// Whether the latest F6 receipt is current by the workflow's decision rule.
    pub fn f6_current(&self, feature: &str) -> bool {
        matches!(
            decision_state(&self.project(), &self.subject(feature), "reconciliation").unwrap(),
            ReceiptState::Current(_)
        )
    }

    pub fn evidence(&self, feature: &str) -> String {
        VerificationStore::at(&self.root.join(".codeos"))
            .latest("feature", feature, "F5", "production_source_evidence")
            .unwrap()
            .unwrap()
            .details
            .unwrap()["evidence_sha256"]
            .as_str()
            .unwrap()
            .to_string()
    }

    /// F6 through F9 for a feature whose F5 passes, selecting its production evidence if any.
    pub fn accept(&self, feature: &str, evidence: Option<&str>) {
        self.receipt(feature, "reconciliation", "completed", evidence);
        self.seed(feature, "candidate", "project_qualification");
        let project = self.project();
        codeos_workflow::candidate::prepare(
            &project,
            &self.subject(feature),
            &[],
            "ordinary",
            None,
        )
        .unwrap();
        self.receipt(feature, "acceptance", "accepted", None);
    }
}

pub const ROUTE: &str = "| route | Road route | reality-dependent | provider | roads | now | unavailable | test-only |\n";
pub const ADVANCE: &str = "mode: advance\nprevious_frontier: entry\nproposed_frontier: decided\ntransition: the driver compares routes";

pub fn routed(scopes: &str, claim_scopes: &str) -> Fixture {
    routed_under("DBA-27", scopes, claim_scopes)
}

pub fn routed_under(configuration: &str, scopes: &str, claim_scopes: &str) -> Fixture {
    let fx = Fixture::under(configuration, scopes);
    fx.write("claim-scopes.json", claim_scopes);
    fx.spec("F-0001", ADVANCE, ROUTE);
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "spec"]);
    let log = fx.check("F-0001");
    assert_eq!(fx.state("F-0001", "F5").0, State::Pass, "{log:?}");
    fx
}

/// F-0001 accepted, then revalidated after a change with new production evidence. `legacy` drops
/// the Candidate Bundle from the acceptance receipt, as for an acceptance made before bundles.
pub fn revalidated_upstream(configuration: &str, legacy: bool) -> (Fixture, String, String) {
    let fx = routed_under(
        configuration,
        "    provider: [backend]\n",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    let accepted = fx.evidence("F-0001");
    fx.accept("F-0001", Some(&accepted));
    if legacy {
        let path = fx.root.join(".codeos/06-workflow/decisions.jsonl");
        let text = fs::read_to_string(&path).unwrap();
        let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
        let mut last: serde_json::Value = serde_json::from_str(lines.last().unwrap()).unwrap();
        last["bindings"]
            .as_object_mut()
            .unwrap()
            .remove("candidate_bundle");
        *lines.last_mut().unwrap() = serde_json::to_string(&last).unwrap();
        fs::write(&path, lines.join("\n") + "\n").unwrap();
    }

    // The lockfile is always a bundle input, so the change stales the acceptance. The trigger must
    // be committed after the acceptance second.
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fx.write("backend/Cargo.lock", "# rustls 0.23.46\n");
    fx.write("scripts/verify.sh", &format!("{VERIFIER}# v2\n"));
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "trigger"]);
    let project = fx.project();
    assert!(!codeos_workflow::revalidation::current_acceptance_valid(&project, "F-0001").unwrap());
    let log = run_checks_revalidation(&fx);
    let revalidated = fx.evidence("F-0001");
    assert_ne!(revalidated, accepted, "{log:?}");
    fx.seed("F-0001", "candidate", "project_qualification");
    let (bundle, _) = codeos_workflow::candidate::prepare(
        &fx.project(),
        &fx.subject("F-0001"),
        &[],
        "post_acceptance_revalidation",
        Some("HEAD"),
    )
    .unwrap();
    fx.write(
        "review/F-0001-stage8.md",
        &format!("---\nreview_id: r1\nreviewed:\n  feature: F-0001\n  stage: 8\n  source: codex\n  candidate_bundle: {bundle}\n  candidate_outcome: PASSED\n---\n"),
    );
    let project = fx.project();
    assert!(codeos_workflow::revalidation::current_acceptance_valid(&project, "F-0001").unwrap());
    (fx, accepted, revalidated)
}

pub fn run_checks_revalidation(fx: &Fixture) -> Vec<String> {
    fx.seed("F-0001", "F3", "smoke");
    fx.seed("F-0001", "F3", "product_journey");
    let run = |environment| {
        codeos_workflow::checker::run_revalidation_checks(
            &fx.project(),
            &fx.subject("F-0001"),
            "HEAD",
            environment,
        )
        .unwrap()
    };
    if !fx.v13() {
        return run(None);
    }
    let mut log = run(Some(Environment::Deterministic));
    fx.seed("F-0001", "candidate", "project_qualification");
    log.extend(run(Some(Environment::Production)));
    log
}

/// Every file under `.codeos` (except the toolkit link) with its content, to prove a read-only run.
pub fn codeos_files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    pub fn walk(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.ends_with("toolkit") {
                continue;
            } else if path.is_dir() {
                walk(&path, out);
            } else {
                out.push((path.clone(), fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(&root.join(".codeos"), &mut out);
    out.sort();
    out
}

pub const EVENT_VALIDATOR: &str = r#"{
  total: length,
  invalid_events: [.[] | select(.event_type != "RouteShown")],
  privacy_check: {prohibited_payload_keys_found: [.[] | .payload | keys[] | select(. == "secret")]}
}"#;

impl Fixture {
    /// Put a feature in the `events` Observation Mode, with a validator and an empty event log.
    pub fn events_mode(&self, feature: &str) {
        let path = self.root.join(format!(
            ".codeos/01-specification/event-schemas/{feature}_schema.md"
        ));
        let text = fs::read_to_string(&path).unwrap();
        fs::write(
            &path,
            text.replace("**Mode:** `external-observation`", "**Mode:** `events`"),
        )
        .unwrap();
        self.write("events/validate.jq", EVENT_VALIDATOR);
        self.write("events/runtime_events.jsonl", "");
    }

    /// Record runtime-event evidence through `verify-events`, with one appended event.
    pub fn record_events(&self, feature: &str) {
        let command: Vec<String> = [
            "bash",
            "-c",
            r#"printf '{"event_id":"%s","event_type":"RouteShown","timestamp":1,"correlation_id":"run-%s","source_module":"route","payload":{}}\n' "$RANDOM" "$RANDOM" >> events/runtime_events.jsonl"#,
        ]
        .map(String::from)
        .to_vec();
        codeos_workflow::event_evidence::verify(
            &self.project(),
            codeos_workflow::event_evidence::VerifyEventArgs {
                subject: &self.subject(feature),
                event_log: "events/runtime_events.jsonl",
                validator: "events/validate.jq",
                command: &command,
            },
        )
        .unwrap();
    }
}
