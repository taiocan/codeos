//! Feature Workflow v13 (DQA WO-03): a feature `check` starts only the verifications its stated
//! environment authorizes, and a production-source run is accounted from the adapter's ledger.
//! Runs under the DBA-29 candidate in a toolkit copy; never against a provider.

mod fai_fixture;

use codeos_workflow::checker::{run_checks, run_revalidation_checks, State};
use codeos_workflow::execution::Environment;
use codeos_workflow::verification::VerificationStore;
use fai_fixture::*;
use std::fs;

const PRODUCTION: Option<Environment> = Some(Environment::Production);
const DETERMINISTIC: Option<Environment> = Some(Environment::Deterministic);

/// A fake production verifier whose adapter ledger records `ledger` (lines with `$at` for a time)
/// in place of the fixture's one call, and which refuses to run outside the production environment.
fn verifier(ledger: &str, exit: u8) -> String {
    let base: String = VERIFIER
        .lines()
        .filter(|line| !line.contains("CODEOS_PROVIDER_LEDGER") && !line.starts_with("# Feature"))
        .map(|line| format!("{line}\n"))
        .collect();
    format!(
        "{base}[[ \"$CODEOS_EXECUTION_ENVIRONMENT\" == production ]] || exit 9\nat=2026-10-18T00:00:00Z\n{ledger}exit {exit}\n"
    )
}

const ONE_CALL: &str = "printf '{\"schema\":1,\"attempt\":1,\"event\":\"attempt\",\"at\":\"%s\"}\\n{\"schema\":1,\"attempt\":1,\"event\":\"success\",\"at\":\"%s\"}\\n' $at $at >> \"$CODEOS_PROVIDER_LEDGER\"\n";

/// A GUI F-0001 with a reality-dependent claim under the DBA-29 candidate, through F4, with F5 and
/// F5g stale and the deterministic mechanics stubbed to pass.
fn gui_feature() -> Fixture {
    let (toolkit, id) = candidate_toolkit();
    let fx = Fixture::under_toolkit(toolkit, id, "    provider: [backend]\n");
    fx.write(
        "claim-scopes.json",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    fx.write("scripts/verify.sh", &verifier(ONE_CALL, 0));
    fx.write(
        ".codeos/06-workflow/.verify-stub",
        "behavior=pass\nrepeatability=pass\nplaywright=pass\n",
    );
    fx.spec("F-0001", ADVANCE, ROUTE);
    let contract = fx
        .root
        .join(".codeos/01-specification/contracts/F-0001_contract.md");
    let text = fs::read_to_string(&contract).unwrap();
    fs::write(
        &contract,
        text.replace(
            "| GUI (Svelte) | unchanged | none |",
            "| GUI (Svelte) | changed | form |",
        ),
    )
    .unwrap();
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "spec"]);
    fx.seed("F-0001", "F3", "smoke");
    let subject = fx.subject("F-0001");
    let bindings = codeos_workflow::checker::verification_bindings_for_mechanic(
        &fx.project(),
        &subject,
        "F3",
        "product_journey",
    )
    .unwrap();
    VerificationStore::at(&fx.root.join(".codeos"))
        .append(&codeos_workflow::verification::VerificationRecord {
            workflow: "feature".into(),
            subject: "F-0001".into(),
            checkpoint: "F3".into(),
            verification: "product_journey".into(),
            result: "passed".into(),
            bindings,
            command: "seeded".into(),
            timestamp: "2026-10-18T00:00:00Z".into(),
            details: Some(serde_json::json!({ "observed_surface": "surface-1" })),
        })
        .unwrap();
    fx.receipt("F-0001", "early_preview", "direction_confirmed", None);
    assert_eq!(fx.state("F-0001", "F4").0, State::Pass);
    fx
}

fn record(fx: &Fixture, checkpoint: &str, mechanic: &str) -> Option<serde_json::Value> {
    VerificationStore::at(&fx.root.join(".codeos"))
        .latest("feature", "F-0001", checkpoint, mechanic)
        .unwrap()
        .map(|record| record.details.unwrap_or_default())
}

fn ledgers(fx: &Fixture) -> usize {
    fs::read_dir(fx.root.join(".codeos/06-workflow/provider-ledger"))
        .map(|dir| dir.count())
        .unwrap_or(0)
}

fn check(fx: &Fixture, environment: Option<Environment>) -> Vec<String> {
    run_checks(&fx.project(), &fx.subject("F-0001"), environment).unwrap()
}

/// Criteria 1–3: the F-0005 incident state, F5 and F5g both stale, under a production check.
#[test]
fn a_production_check_with_f5_and_f5g_stale_starts_nothing() {
    let fx = gui_feature();
    let log = check(&fx, PRODUCTION);
    assert!(
        log.iter().any(|line| line.contains("started nothing")
            && line.contains("F5/behavior")
            && line.contains("F5g/playwright")),
        "{log:?}"
    );
    for (checkpoint, mechanic) in [
        ("F5", "behavior"),
        ("F5", "production_source_evidence"),
        ("F5g", "playwright"),
    ] {
        assert!(
            record(&fx, checkpoint, mechanic).is_none(),
            "{checkpoint}/{mechanic}"
        );
    }
    assert_eq!(ledgers(&fx), 0, "no production run started");
}

/// Criterion 2: F5g completes in the deterministic environment while F5 waits on production.
#[test]
fn a_deterministic_check_completes_f5g_while_production_evidence_is_pending() {
    let fx = gui_feature();
    let log = check(&fx, None);
    for (checkpoint, mechanic) in [
        ("F5", "behavior"),
        ("F5", "repeatability"),
        ("F5g", "playwright"),
    ] {
        assert_eq!(
            record(&fx, checkpoint, mechanic).unwrap()["execution_environment"],
            "deterministic",
            "{checkpoint}/{mechanic}: {log:?}"
        );
    }
    assert!(record(&fx, "F5", "production_source_evidence").is_none());
    assert_eq!(ledgers(&fx), 0);
    assert!(
        log.last()
            .unwrap()
            .contains("qualify-project --mode acceptance --subject F-0001"),
        "{log:?}"
    );
}

/// Pre-candidate qualification orders before the production run; the run is then accounted and
/// reconciled with its receipts.
#[test]
fn a_production_run_waits_on_qualification_and_is_accounted() {
    let fx = gui_feature();
    check(&fx, DETERMINISTIC);
    let log = check(&fx, PRODUCTION);
    assert!(
        log.iter()
            .any(|line| line.contains("started nothing") && line.contains("qualify-project")),
        "{log:?}"
    );
    assert_eq!(ledgers(&fx), 0);

    fx.seed("F-0001", "candidate", "project_qualification");
    let log = check(&fx, None);
    assert!(
        log.last().unwrap().contains("--environment production"),
        "{log:?}"
    );
    let log = check(&fx, PRODUCTION);
    let details = record(&fx, "F5", "production_source_evidence").expect("production evidence");
    assert_eq!(details["execution_environment"], "production", "{log:?}");
    assert_eq!(
        details["provider_calls"],
        serde_json::json!({"attempted": 1, "successful": 1, "failed": 0, "unresolved": 0, "total": 1})
    );
    assert_eq!(fx.state("F-0001", "F5").0, State::Pass);
    assert_eq!(fx.state("F-0001", "F5g").0, State::Pass);
}

/// Criterion 4: one request that times out before the run aborts.
#[test]
fn a_timed_out_request_is_accounted_when_the_run_fails() {
    let fx = gui_feature();
    fx.write(
        "scripts/verify.sh",
        &verifier(
            "printf '{\"schema\":1,\"attempt\":1,\"event\":\"attempt\",\"at\":\"%s\"}\\n{\"schema\":1,\"attempt\":1,\"event\":\"failure\",\"at\":\"%s\"}\\n' $at $at >> \"$CODEOS_PROVIDER_LEDGER\"\n",
            1,
        ),
    );
    check(&fx, DETERMINISTIC);
    fx.seed("F-0001", "candidate", "project_qualification");
    let log = check(&fx, PRODUCTION);
    assert!(
        log.iter().any(|line| line
            .contains("[provider calls: attempted=1 successful=0 failed=1 unresolved=0 total=1]")),
        "{log:?}"
    );
    assert!(record(&fx, "F5", "production_source_evidence").is_none());
    let totals: Vec<_> = fs::read_dir(fx.root.join(".codeos/06-workflow/provider-ledger"))
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            path.to_string_lossy()
                .ends_with(".totals.json")
                .then_some(path)
        })
        .collect();
    assert_eq!(totals.len(), 1);
}

/// A run aborted before Codeos recorded its totals is counted by the next production check, with
/// the request in flight unresolved rather than guessed.
#[test]
fn an_aborted_run_is_reported_by_the_next_production_check() {
    let fx = gui_feature();
    fx.write(
        ".codeos/06-workflow/provider-ledger/F-0001-aborted.jsonl",
        "{\"schema\":1,\"attempt\":1,\"event\":\"attempt\",\"at\":\"2026-10-18T00:00:00Z\"}\n",
    );
    let log = check(&fx, PRODUCTION);
    assert!(
        log.iter().any(|line| line == "provider run F-0001-aborted ended without accounting: attempted=1 successful=0 failed=0 unresolved=1 total=1"),
        "{log:?}"
    );
    assert!(fx
        .root
        .join(".codeos/06-workflow/provider-ledger/F-0001-aborted.totals.json")
        .is_file());
    assert!(!check(&fx, PRODUCTION)
        .iter()
        .any(|line| line.contains("F-0001-aborted")));
}

#[test]
fn successful_calls_must_reconcile_with_the_provider_receipts() {
    let fx = gui_feature();
    fx.write(
        "scripts/verify.sh",
        &verifier(
            &format!(
                "{ONE_CALL}{}",
                ONE_CALL.replace("\"attempt\":1", "\"attempt\":2")
            ),
            0,
        ),
    );
    check(&fx, DETERMINISTIC);
    fx.seed("F-0001", "candidate", "project_qualification");
    let log = check(&fx, PRODUCTION);
    assert!(
        log.iter().any(|line| line
            .contains("2 successful provider calls do not reconcile with 1 provider receipts")),
        "{log:?}"
    );
    assert!(record(&fx, "F5", "production_source_evidence").is_none());
}

/// Revalidation reruns obey the same environments.
#[test]
fn a_production_revalidation_waits_on_its_deterministic_reruns() {
    let fx = gui_feature();
    check(&fx, DETERMINISTIC);
    fx.seed("F-0001", "candidate", "project_qualification");
    check(&fx, PRODUCTION);
    let evidence = fx.evidence("F-0001");
    fx.accept("F-0001", Some(&evidence));
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fx.write("backend/Cargo.lock", "# rustls 0.23.46\n");
    fx.write(
        ".codeos/06-workflow/.verify-stub",
        "smoke=pass\nproduct_journey=pass\nbehavior=pass\nrepeatability=pass\nplaywright=pass\n",
    );
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "trigger"]);
    let revalidate = |environment| {
        run_revalidation_checks(&fx.project(), &fx.subject("F-0001"), "HEAD", environment).unwrap()
    };
    let before = ledgers(&fx);
    let log = revalidate(PRODUCTION);
    assert!(
        log.iter().any(|line| line.contains("started nothing")
            && line.contains("--revalidation --trigger HEAD`")),
        "{log:?}"
    );
    assert_eq!(ledgers(&fx), before);
    let log = revalidate(DETERMINISTIC);
    assert!(log.last().unwrap().contains("qualify-project"), "{log:?}");
    assert_eq!(ledgers(&fx), before);
}

#[test]
fn an_environment_is_refused_before_feature_workflow_v13() {
    let fx = Fixture::under("DBA-28", "    provider: [backend]\n");
    fx.spec("F-0001", ADVANCE, ROUTE);
    let error = run_checks(&fx.project(), &fx.subject("F-0001"), DETERMINISTIC).unwrap_err();
    assert!(
        error.to_string().contains("Feature Workflow v13"),
        "{error}"
    );
}
