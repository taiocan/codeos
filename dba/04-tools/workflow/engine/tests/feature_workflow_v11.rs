//! FAI acceptance cases for Feature Workflow v11 (DBA-27): production-source evidence binds the
//! Evidence Scopes it observed and its verifier, and a claim may inherit upstream production truth.
//! A fake verifier produces real evidence through `check`; the other mechanics are seeded or stubbed.

use codeos_workflow::checker::{
    bindings_for_decision, evaluate, run_checks, verification_bindings_for_mechanic, State, Subject,
};
use codeos_workflow::contract::Workflow;
use codeos_workflow::project::Project;
use codeos_workflow::receipts::{Receipt, ReceiptStore};
use codeos_workflow::verification::{VerificationRecord, VerificationStore};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const VERIFIER: &str = r#"#!/usr/bin/env bash
set -euo pipefail
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

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    fn new(scopes: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let fx = Fixture {
            root: dir.path().canonicalize().unwrap(),
            _dir: dir,
        };
        let toolkit = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .unwrap();
        fs::create_dir_all(fx.root.join(".codeos/06-workflow")).unwrap();
        std::os::unix::fs::symlink(toolkit, fx.root.join(".codeos/toolkit")).unwrap();
        fx.write(".gitignore", ".codeos/toolkit\nbackend/target\n");
        fx.write(
            ".codeos/00-project/codeos.yaml",
            &format!("codeos_schema: 2\ndba_configuration: DBA-27\nreality_verification:\n  command: scripts/verify.sh\n  scopes:\n{scopes}"),
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

    fn write(&self, rel: &str, content: &str) {
        let path = self.root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn git(&self, args: &[&str]) {
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

    fn project(&self) -> Project {
        Project::discover(&self.root).unwrap()
    }

    fn subject(&self, feature: &str) -> Subject {
        Subject::resolve(&self.project(), Workflow::Feature, feature).unwrap()
    }

    /// A non-GUI Specification Package; `reality` holds extra Reality Basis rows.
    fn spec(&self, feature: &str, contribution: &str, reality: &str) {
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

    fn seed(&self, feature: &str, checkpoint: &str, mechanic: &str) {
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

    fn receipt(&self, feature: &str, checkpoint: &str, result: &str, evidence: Option<&str>) {
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

    /// Seed F3, then let `check` run F5 with the fake verifier.
    fn check(&self, feature: &str) -> Vec<String> {
        self.seed(feature, "F3", "smoke");
        self.seed(feature, "F3", "product_journey");
        run_checks(&self.project(), &self.subject(feature)).unwrap()
    }

    fn state(&self, feature: &str, checkpoint: &str) -> (State, Vec<String>) {
        let reports = evaluate(&self.project(), &self.subject(feature)).unwrap();
        let report = reports.into_iter().find(|r| r.id == checkpoint).unwrap();
        (report.state, report.unmet)
    }

    /// Whether the latest production-source evidence record is current (Absent, Failed and Stale
    /// are not), with the binding that drifted.
    fn evidence_current(&self, feature: &str) -> Result<(), String> {
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

    /// Whether the latest F6 receipt still matches the inputs it recorded.
    fn f6_current(&self, feature: &str) -> bool {
        let receipt = ReceiptStore::at(&self.root.join(".codeos"))
            .latest("feature", feature, "reconciliation")
            .unwrap()
            .unwrap();
        let now = codeos_workflow::checker::bindings_for_evaluation(
            &self.project(),
            &self.subject(feature),
            "reconciliation",
        )
        .unwrap();
        receipt
            .bindings
            .iter()
            .filter(|(key, _)| key.as_str() != "production_evidence")
            .all(|(key, value)| now.get(key) == Some(value))
    }

    fn evidence(&self, feature: &str) -> String {
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
    fn accept(&self, feature: &str, evidence: Option<&str>) {
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

const ROUTE: &str = "| route | Road route | reality-dependent | provider | roads | now | unavailable | test-only |\n";
const ADVANCE: &str = "mode: advance\nprevious_frontier: entry\nproposed_frontier: decided\ntransition: the driver compares routes";

fn routed(scopes: &str, claim_scopes: &str) -> Fixture {
    let fx = Fixture::new(scopes);
    fx.write("claim-scopes.json", claim_scopes);
    fx.spec("F-0001", ADVANCE, ROUTE);
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "spec"]);
    let log = fx.check("F-0001");
    assert_eq!(fx.state("F-0001", "F5").0, State::Pass, "{log:?}");
    fx
}

#[test]
fn a_test_only_change_leaves_production_evidence_and_f6_current() {
    let fx = routed(
        "    provider: [backend]\n",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    let evidence = fx.evidence("F-0001");
    fx.receipt("F-0001", "reconciliation", "completed", Some(&evidence));
    let bound = &ReceiptStore::at(&fx.root.join(".codeos"))
        .latest("feature", "F-0001", "reconciliation")
        .unwrap()
        .unwrap()
        .bindings;
    assert!(bound.contains_key("delivered_state") && !bound.contains_key("implementation_state"));

    fx.write("backend/tests/route.rs", "#[test]\nfn route_again() {}\n");
    assert_eq!(fx.evidence_current("F-0001"), Ok(()));
    assert!(fx.f6_current("F-0001"));
    // The fixture-mode behavior record still binds the tests, so F5 re-runs it; no provider run.
    assert_ne!(fx.state("F-0001", "F5").0, State::Pass);
}

#[test]
fn a_test_runner_configuration_change_leaves_evidence_and_f6_current() {
    let fx = routed(
        "    provider: [backend]\n    web: [web]\n",
        r#"{"F-0001": {"route": ["provider", "web"]}}"#,
    );
    let evidence = fx.evidence("F-0001");
    fx.receipt("F-0001", "reconciliation", "completed", Some(&evidence));
    let product = fx.project().product_state().unwrap();

    fx.write("web/playwright.config.ts", "export default { workers: 1 };\n");
    assert_eq!(fx.evidence_current("F-0001"), Ok(()));
    assert!(fx.f6_current("F-0001"));
    // F4 and F8 keep their product state, which still counts the configuration.
    assert_ne!(fx.project().product_state().unwrap(), product);

    // Build configuration is delivered.
    fx.write("web/vite.config.ts", "export default {};\n");
    assert_eq!(
        fx.evidence_current("F-0001"),
        Err("evidence_scope_state".into())
    );
    assert!(!fx.f6_current("F-0001"));
}

#[test]
fn a_shared_ui_change_stales_only_evidence_that_observed_it() {
    let provider_only = routed(
        "    provider: [backend]\n    web: [web]\n",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    provider_only.write("web/src/page.svelte", "<h1>Compare departures</h1>\n");
    assert_eq!(provider_only.evidence_current("F-0001"), Ok(()));

    let observed_ui = routed(
        "    provider: [backend]\n    web: [web]\n",
        r#"{"F-0001": {"route": ["provider", "web"]}}"#,
    );
    observed_ui.write("web/src/page.svelte", "<h1>Compare departures</h1>\n");
    assert_eq!(
        observed_ui.evidence_current("F-0001"),
        Err("evidence_scope_state".into())
    );

    // A claim that names no scope observed everything.
    let unscoped = routed("    provider: [backend]\n", "{}");
    unscoped.write("web/src/page.svelte", "<h1>Compare departures</h1>\n");
    assert_eq!(
        unscoped.evidence_current("F-0001"),
        Err("evidence_scope_state".into())
    );
}

#[test]
fn a_verifier_change_stales_evidence_and_check_regenerates_it() {
    let fx = routed(
        "    provider: [backend]\n",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    let first = fx.evidence("F-0001");
    fx.write("scripts/verify.sh", &format!("{VERIFIER}# observes more\n"));
    assert_eq!(
        fx.evidence_current("F-0001"),
        Err("reality_verifier".into())
    );

    run_checks(&fx.project(), &fx.subject("F-0001")).unwrap();
    assert_eq!(fx.state("F-0001", "F5").0, State::Pass);
    assert_ne!(fx.evidence("F-0001"), first);
}

#[test]
fn a_verifier_change_does_not_stale_an_accepted_feature() {
    let fx = routed(
        "    provider: [backend]\n",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    let evidence = fx.evidence("F-0001");
    fx.accept("F-0001", Some(&evidence));
    let project = fx.project();
    assert!(codeos_workflow::revalidation::current_acceptance_valid(&project, "F-0001").unwrap());
    fx.write("scripts/verify.sh", &format!("{VERIFIER}# observes more\n"));
    let project = fx.project();
    assert!(codeos_workflow::revalidation::current_acceptance_valid(&project, "F-0001").unwrap());
}

#[test]
fn a_runtime_dependency_change_still_stales_the_evidence() {
    // The scope names only the source directory; the package lockfile is still bound.
    let fx = routed(
        "    provider: [backend/src]\n",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    fx.write("backend/Cargo.lock", "# rustls 0.23.46\n");
    assert_eq!(
        fx.evidence_current("F-0001"),
        Err("evidence_scope_state".into())
    );
}

#[test]
fn an_undeclared_evidence_scope_fails_closed() {
    let fx = Fixture::new("    provider: [backend]\n");
    fx.write(
        "claim-scopes.json",
        r#"{"F-0001": {"route": ["everything-else"]}}"#,
    );
    fx.spec("F-0001", ADVANCE, ROUTE);
    let log = fx.check("F-0001");
    assert!(
        log.iter()
            .any(|line| line.contains("undeclared or duplicated Evidence Scope")),
        "{log:?}"
    );
    assert!(VerificationStore::at(&fx.root.join(".codeos"))
        .latest("feature", "F-0001", "F5", "production_source_evidence")
        .unwrap()
        .is_none());
}

#[test]
fn an_inherited_claim_follows_upstream_evidence_without_its_own_run() {
    let fx = routed(
        "    provider: [backend]\n",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    let evidence = fx.evidence("F-0001");
    fx.accept("F-0001", Some(&evidence));
    fx.spec(
        "F-0002",
        "mode: preserve\nreason: summarizes the route",
        "| summary | Route summary | inherited | F-0001/route | as F-0001 | as F-0001 | none | simulated |\n",
    );
    let log = fx.check("F-0002");
    assert!(
        !log.iter().any(|l| l.contains("production_source_evidence")),
        "{log:?}"
    );
    let (state, _) = fx.state("F-0002", "F5");
    assert_eq!(state, State::Pass);
    assert!(VerificationStore::at(&fx.root.join(".codeos"))
        .latest("feature", "F-0002", "F5", "production_source_evidence")
        .unwrap()
        .is_none());

    fx.accept("F-0002", None);
    let project = fx.project();
    let bundle = codeos_workflow::candidate::unique_current(&project, "F-0002")
        .unwrap()
        .unwrap();
    let inherited = codeos_workflow::candidate::CandidateStore::at(&project.codeos_dir())
        .read(&bundle)
        .unwrap()
        .inherited_evidence;
    assert_eq!(inherited["summary"].evidence, evidence);
    assert!(codeos_workflow::revalidation::current_acceptance_valid(&project, "F-0002").unwrap());

    // Upstream loses validity: the inherited claim propagates it.
    let upstream = fx
        .root
        .join(".codeos/01-specification/contracts/F-0001_contract.md");
    let text = fs::read_to_string(&upstream).unwrap();
    fs::write(&upstream, text.replace("Road route", "Road route, revised")).unwrap();
    let project = fx.project();
    let assessment =
        codeos_workflow::revalidation::acceptance_assessment(&project, "F-0002").unwrap();
    assert!(!assessment.current);
    assert_eq!(assessment.reason, Some("UPSTREAM_REVALIDATION_REQUIRED"));
}

#[test]
fn a_changed_observed_surface_still_stales_f4() {
    let fx = Fixture::new("    provider: [backend]\n");
    let gui = |feature: &str| {
        fx.spec(feature, ADVANCE, "");
        let path = fx.root.join(format!(
            ".codeos/01-specification/contracts/{feature}_contract.md"
        ));
        let text = fs::read_to_string(&path).unwrap();
        fs::write(
            &path,
            text.replace(
                "| GUI (Svelte) | unchanged | none |",
                "| GUI (Svelte) | changed | form |",
            ),
        )
        .unwrap();
    };
    gui("F-0003");
    let observe = |surface: &str| {
        let subject = fx.subject("F-0003");
        let bindings =
            verification_bindings_for_mechanic(&fx.project(), &subject, "F3", "product_journey")
                .unwrap();
        VerificationStore::at(&fx.root.join(".codeos"))
            .append(&VerificationRecord {
                workflow: "feature".into(),
                subject: "F-0003".into(),
                checkpoint: "F3".into(),
                verification: "product_journey".into(),
                result: "passed".into(),
                bindings,
                command: "seeded".into(),
                timestamp: "2026-10-16T00:00:00Z".into(),
                details: Some(serde_json::json!({ "observed_surface": surface })),
            })
            .unwrap();
    };
    fx.seed("F-0003", "F3", "smoke");
    observe("surface-1");
    fx.receipt("F-0003", "early_preview", "direction_confirmed", None);
    assert_eq!(fx.state("F-0003", "F4").0, State::Pass);
    observe("surface-2");
    let (state, unmet) = fx.state("F-0003", "F4");
    assert_eq!(state, State::Waiting);
    assert!(
        unmet.iter().any(|u| u.contains("observed_surface")),
        "{unmet:?}"
    );
}

/// F-0001 accepted, then revalidated after a change with new production evidence. `legacy` drops
/// the Candidate Bundle from the acceptance receipt, as for an acceptance made before bundles.
fn revalidated_upstream(legacy: bool) -> (Fixture, String, String) {
    let fx = routed(
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
        last["bindings"].as_object_mut().unwrap().remove("candidate_bundle");
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

fn run_checks_revalidation(fx: &Fixture) -> Vec<String> {
    fx.seed("F-0001", "F3", "smoke");
    fx.seed("F-0001", "F3", "product_journey");
    codeos_workflow::checker::run_revalidation_checks(&fx.project(), &fx.subject("F-0001"), "HEAD")
        .unwrap()
}

#[test]
fn an_inherited_claim_follows_the_upstream_revalidation_evidence() {
    for legacy in [false, true] {
        let (fx, accepted, revalidated) = revalidated_upstream(legacy);
        fx.spec(
            "F-0002",
            "mode: preserve\nreason: summarizes the route",
            "| summary | Route summary | inherited | F-0001/route | as F-0001 | as F-0001 | none | simulated |\n",
        );
        let project = fx.project();
        let resolved =
            codeos_workflow::revalidation::inherited_evidence(&project, &fx.subject("F-0002"))
                .unwrap();
        let evidence = resolved[0].1.as_ref().unwrap().evidence.clone();
        assert_eq!(evidence, revalidated, "legacy={legacy}");
        assert_ne!(evidence, accepted);
    }
}
