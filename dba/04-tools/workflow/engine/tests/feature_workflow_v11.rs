//! FAI acceptance cases for Feature Workflow v11 (DBA-27) and v12 (DBA-28): production-source evidence binds the
//! Evidence Scopes it observed and its verifier, and a claim may inherit upstream production truth.
//! A fake verifier produces real evidence through `check`; the other mechanics are seeded or stubbed.

mod fai_fixture;

use codeos_workflow::checker::{run_checks, verification_bindings_for_mechanic, State};
use codeos_workflow::receipts::ReceiptStore;
use codeos_workflow::verification::{VerificationRecord, VerificationStore};
use fai_fixture::*;
use std::fs;
use std::process::Command;

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
    assert!(bound.contains_key("product_state") && !bound.contains_key("implementation_state"));

    fx.write("backend/tests/route.rs", "#[test]\nfn route_again() {}\n");
    assert_eq!(fx.evidence_current("F-0001"), Ok(()));
    assert!(fx.f6_current("F-0001"));
    // The fixture-mode behavior record still binds the tests, so F5 re-runs it; no provider run.
    assert_ne!(fx.state("F-0001", "F5").0, State::Pass);
}

/// Feature Workflow v12 (DBA-28): delivered state excludes test-runner configuration. v11 (DBA-27)
/// keeps binding it as issued.
#[test]
fn a_test_runner_configuration_change_leaves_evidence_and_f6_current() {
    let routed_web = |configuration| {
        let fx = routed_under(
            configuration,
            "    provider: [backend]\n    web: [web]\n",
            r#"{"F-0001": {"route": ["provider", "web"]}}"#,
        );
        let evidence = fx.evidence("F-0001");
        fx.receipt("F-0001", "reconciliation", "completed", Some(&evidence));
        fx
    };
    let as_issued = routed_web("DBA-27");
    as_issued.write(
        "web/playwright.config.ts",
        "export default { workers: 1 };\n",
    );
    assert_eq!(
        as_issued.evidence_current("F-0001"),
        Err("evidence_scope_state".into())
    );
    assert!(!as_issued.f6_current("F-0001"));

    let fx = routed_web("DBA-28");
    let bound = &ReceiptStore::at(&fx.root.join(".codeos"))
        .latest("feature", "F-0001", "reconciliation")
        .unwrap()
        .unwrap()
        .bindings;
    assert!(bound.contains_key("delivered_state") && !bound.contains_key("product_state"));
    let product = fx.project().product_state().unwrap();

    fx.write(
        "web/playwright.config.ts",
        "export default { workers: 1 };\n",
    );
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

/// The one F6 selection rule every consumer applies (0514192): v11 and v12 production evidence
/// carries no implementation identity and stays selectable while the current production-source
/// verification verified it; a change to a scope it observed makes it stale.
#[test]
fn f6_selected_evidence_is_current_by_its_own_format() {
    for configuration in ["DBA-27", "DBA-28"] {
        let fx = routed_under(
            configuration,
            "    provider: [backend]\n    web: [web]\n",
            r#"{"F-0001": {"route": ["provider"]}}"#,
        );
        let evidence = fx.evidence("F-0001");
        let stored: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(fx.root.join(format!(
                ".codeos/06-workflow/reality-evidence/{evidence}.json"
            )))
            .unwrap(),
        )
        .unwrap();
        assert!(stored.get("implementation_sha256").is_none());
        let selectable = |fx: &Fixture| {
            codeos_workflow::checker::validate_production_evidence_selection(
                &fx.project(),
                &fx.subject("F-0001"),
                &evidence,
            )
            .map_err(|error| error.to_string())
        };
        assert_eq!(selectable(&fx), Ok(()), "{configuration}");
        fx.write("web/src/page.svelte", "<h1>Compare departures</h1>\n");
        assert_eq!(selectable(&fx), Ok(()), "{configuration}");
        fx.write("backend/src/lib.rs", "pub fn route() { /* changed */ }\n");
        assert!(
            selectable(&fx)
                .unwrap_err()
                .contains("'evidence_scope_state' changed"),
            "{configuration}"
        );
    }
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

    run_checks(&fx.project(), &fx.subject("F-0001"), None).unwrap();
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

/// Feature Workflow v12 (DBA-28). Under v11 (DBA-27) as issued, an acceptance from before Candidate
/// Bundles has no evidence to inherit.
#[test]
fn an_inherited_claim_follows_the_upstream_revalidation_evidence() {
    let (fx, _, _) = revalidated_upstream("DBA-27", true);
    fx.spec(
        "F-0002",
        "mode: preserve\nreason: summarizes the route",
        "| summary | Route summary | inherited | F-0001/route | as F-0001 | as F-0001 | none | simulated |\n",
    );
    let resolved =
        codeos_workflow::revalidation::inherited_evidence(&fx.project(), &fx.subject("F-0002"))
            .unwrap();
    assert_eq!(
        resolved[0].1.as_ref().err().map(String::as_str),
        Some("F-0001 has no accepted Candidate Bundle")
    );

    for legacy in [false, true] {
        let (fx, accepted, revalidated) = revalidated_upstream("DBA-28", legacy);
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

/// DQA WO-01C: `check --revalidation --plan` serializes the plan `check --revalidation` executes.
/// It runs no verifier, writes nothing, and names exactly the verifications the executor then runs.
#[test]
fn the_revalidation_plan_is_read_only_and_is_what_check_executes() {
    let fx = routed_under(
        "DBA-28",
        "    provider: [backend]\n",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    let accepted = fx.evidence("F-0001");
    fx.accept("F-0001", Some(&accepted));
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fx.write("backend/Cargo.lock", "# rustls 0.23.46\n");
    fx.write(
        "scripts/verify.sh",
        &VERIFIER.replace(
            "set -euo pipefail\n",
            "set -euo pipefail\ntouch verifier-ran\n",
        ),
    );
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "trigger"]);
    fx.seed("F-0001", "F3", "smoke");
    fx.seed("F-0001", "F3", "product_journey");

    let before = codeos_files(&fx.root);
    let output = Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
        .args(["--project", fx.root.to_str().unwrap()])
        .args(["check", "--workflow", "feature", "--subject", "F-0001"])
        .args(["--revalidation", "--trigger", "HEAD", "--plan"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(
        !fx.root.join("verifier-ran").exists(),
        "the plan ran the verifier"
    );
    assert_eq!(
        codeos_files(&fx.root),
        before,
        "the plan wrote a governed record"
    );

    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let item = |name: &str| {
        plan["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["item"] == name)
            .unwrap()
            .clone()
    };
    assert_eq!(item("smoke")["action"], "reuse");
    assert_eq!(item("smoke")["record"].as_str().map(str::len), Some(64));
    let evidence = item("production_source_evidence");
    assert_eq!(evidence["action"], "rerun");
    assert!(
        evidence["reason"].as_str().unwrap().starts_with("changed:"),
        "{evidence}"
    );
    let planned: Vec<String> = plan["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["action"] == "rerun")
        .map(|i| {
            format!(
                "{} / {}",
                i["checkpoint"].as_str().unwrap(),
                i["item"].as_str().unwrap()
            )
        })
        .collect();

    let executed: Vec<String> = run_checks_revalidation(&fx)
        .iter()
        .map(|line| line.split(':').next().unwrap().to_string())
        .collect();
    assert_eq!(executed, planned);
    assert!(fx.root.join("verifier-ran").exists());
}
