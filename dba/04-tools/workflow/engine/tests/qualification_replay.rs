//! DQA WO-01D: the two lifecycle states release qualification replays because no consumer history
//! reproduces them, each observed only through the CLI. They hold the outcomes the human accepted
//! in FAI WO-04, so a candidate configuration that restores either DBA-27 residual fails.
//! `CODEOS_QUALIFICATION_CONFIGURATION` names the configuration under qualification (default
//! DBA-28, the current accepted behavior).

mod fai_fixture;

use fai_fixture::*;
use std::process::Command;

fn configuration() -> String {
    std::env::var("CODEOS_QUALIFICATION_CONFIGURATION").unwrap_or_else(|_| "DBA-28".into())
}

/// The `status --json` state of one feature checkpoint.
fn checkpoint(fx: &Fixture, feature: &str, id: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
        .args(["--project", fx.root.to_str().unwrap(), "status", "--json"])
        .output()
        .unwrap();
    let status: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let feature = status["features"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == feature)
        .unwrap_or_else(|| panic!("{feature} missing from status"));
    feature["checkpoints"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == id)
        .map(|c| c["status"].as_str().unwrap().to_owned())
        .unwrap_or_else(|| panic!("{id} missing: {feature}"))
}

/// Residual 1: an inherited claim follows an upstream accepted before Candidate Bundles through the
/// bundle of its passed revalidation.
#[test]
fn an_inherited_claim_follows_a_pre_bundle_upstream_through_its_revalidation() {
    let (fx, _, _) = revalidated_upstream(&configuration(), true);
    fx.spec(
        "F-0002",
        "mode: preserve\nreason: summarizes the route",
        "| summary | Route summary | inherited | F-0001/route | as F-0001 | as F-0001 | none | simulated |\n",
    );
    fx.check("F-0002");
    assert_eq!(checkpoint(&fx, "F-0002", "F5"), "COMPLETE");
}

/// Residual 2: after acceptance, a test-runner configuration change leaves web-scoped production
/// evidence current, so revalidation reuses it instead of a new production run.
#[test]
fn a_test_runner_configuration_change_reuses_scoped_production_evidence() {
    let fx = routed_under(
        &configuration(),
        "    provider: [backend]\n    web: [web]\n",
        r#"{"F-0001": {"route": ["provider", "web"]}}"#,
    );
    let evidence = fx.evidence("F-0001");
    fx.accept("F-0001", Some(&evidence));
    // The trigger must be committed after the acceptance second.
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fx.write(
        "web/playwright.config.ts",
        "export default { workers: 1 };\n",
    );
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "test-runner configuration"]);

    let output = Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
        .args(["--project", fx.root.to_str().unwrap()])
        .args(["check", "--workflow", "feature", "--subject", "F-0001"])
        .args(["--revalidation", "--trigger", "HEAD", "--plan"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let item = plan["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["item"] == "production_source_evidence")
        .unwrap();
    assert_eq!(item["action"], "reuse", "{item}");
}
