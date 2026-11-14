//! Feature retirement (Workflow Governance v5, Operation & Learning v3): an accepted feature leaves
//! the governed set only through one human retirement receipt. Runs under DBA-30.

mod fai_fixture;

use codeos_workflow::monitor::ProjectStatus;
use codeos_workflow::observations::{ClosureEvidence, ObservationStore};
use codeos_workflow::retirement;
use fai_fixture::Fixture;
use std::fs;
use std::process::Command;

const PRESERVE: &str = "mode: preserve\nreason: secondary capability";

fn candidate() -> Fixture {
    let fx = Fixture::under("DBA-30", "    provider: [backend]\n");
    fx.spec("F-0002", PRESERVE, "");
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "F-0002"]);
    accept(&fx, "F-0002");
    fx
}

fn accept(fx: &Fixture, feature: &str) {
    let line = format!(
        r#"{{"workflow":"feature","subject":"{feature}","checkpoint":"acceptance","result":"accepted","bindings":{{}},"timestamp":"2026-11-01T00:00:00Z"}}"#
    );
    let path = fx.root.join(".codeos/06-workflow/decisions.jsonl");
    let text = fs::read_to_string(&path).unwrap_or_default();
    fs::write(&path, format!("{text}{line}\n")).unwrap();
}

fn route(fx: &Fixture, feature: &str) -> String {
    let store = ObservationStore::at(&fx.root.join(".codeos"));
    let observation = store
        .observe(format!("{feature} is no longer used"), vec![])
        .unwrap();
    store
        .route_observation(
            &fx.project(),
            &observation.id,
            "feature_retirement",
            Some("feature".into()),
            Some(feature.into()),
        )
        .unwrap();
    observation.id
}

fn decide(fx: &Fixture, feature: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
        .args([
            "decide",
            "--workflow",
            "feature",
            "--subject",
            feature,
            "--checkpoint",
            "retirement",
            "--result",
            "retired",
            "--rationale",
            "no user reaches it",
        ])
        .current_dir(&fx.root)
        .output()
        .unwrap()
}

fn remove(fx: &Fixture, feature: &str) {
    for path in retirement::specification_artifacts(&fx.project(), feature) {
        fs::remove_file(path).unwrap();
    }
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "retire"]);
}

fn retirement_blockers(fx: &Fixture) -> Vec<String> {
    ProjectStatus::derive(&fx.project())
        .unwrap()
        .attention
        .into_iter()
        .filter(|item| item.checkpoint.as_deref() == Some("RETIREMENT"))
        .map(|item| item.issue.unwrap())
        .collect()
}

fn refusal(fx: &Fixture, feature: &str) -> String {
    retirement::decision_bindings(&fx.project(), feature)
        .unwrap_err()
        .to_string()
}

fn receipt_identity(fx: &Fixture, checkpoint: &str) -> String {
    let decisions =
        fs::read_to_string(fx.root.join(".codeos/06-workflow/decisions.jsonl")).unwrap();
    let line = decisions
        .lines()
        .find(|line| line.contains(&format!(r#""checkpoint":"{checkpoint}""#)))
        .unwrap();
    codeos_workflow::hashing::text_sha256(line)
}

fn close(fx: &Fixture, observation: &str, receipt: String) -> anyhow::Result<()> {
    let head = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&fx.root)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    ObservationStore::at(&fx.root.join(".codeos"))
        .close(
            &fx.project(),
            observation,
            vec![
                ClosureEvidence {
                    kind: "retirement_receipt".into(),
                    reference: receipt,
                },
                ClosureEvidence {
                    kind: "commit".into(),
                    reference: head.trim().into(),
                },
            ],
            None,
        )
        .map(|_| ())
}

#[test]
fn a_routed_retirement_removes_the_feature_and_keeps_its_history() {
    let fx = candidate();
    let observation = route(&fx, "F-0002");
    let output = decide(&fx, "F-0002");
    assert!(output.status.success(), "{output:?}");

    // Recorded but not yet removed: the artifacts block, and the ID cannot be reused.
    let blockers = retirement_blockers(&fx);
    assert_eq!(blockers.len(), 1, "{blockers:?}");
    assert!(blockers[0].contains("never reused"));

    remove(&fx, "F-0002");
    let status = ProjectStatus::derive(&fx.project()).unwrap();
    assert!(retirement_blockers(&fx).is_empty());
    assert_eq!(status.retired_features.len(), 1);
    assert_eq!(status.retired_features[0].id, "F-0002");
    assert!(status.features.iter().all(|f| f.id != "F-0002"));
    let decisions =
        fs::read_to_string(fx.root.join(".codeos/06-workflow/decisions.jsonl")).unwrap();
    assert!(decisions.contains(r#""subject":"F-0002","checkpoint":"acceptance""#));

    close(&fx, &observation, receipt_identity(&fx, "retirement")).unwrap();
}

#[test]
fn retirement_is_refused_without_a_route_or_while_something_depends_on_it() {
    let fx = candidate();
    assert!(refusal(&fx, "F-0002").contains("no open feature_retirement observation"));
    route(&fx, "F-0002");

    fx.write(
        ".codeos/02-architecture/scopes/core.md",
        "---\nartifact_type: architecture\nfeatures: [F-0002]\napproval:\n  by: Human\n  at: 2026-11-01\nreader_model: engineer\n---\n\n## Structure\n",
    );
    let why = refusal(&fx, "F-0002");
    assert!(
        why.contains("Architecture Scope still names F-0002"),
        "{why}"
    );
    fs::remove_file(fx.root.join(".codeos/02-architecture/scopes/core.md")).unwrap();

    fx.spec(
        "F-0003",
        PRESERVE,
        "| summary | Summary | inherited | F-0002/label | as F-0002 | as F-0002 | none | simulated |\n",
    );
    assert!(refusal(&fx, "F-0002").contains("F-0003 inherits a claim from F-0002"));
}

#[test]
fn a_journey_advancing_feature_cannot_be_retired() {
    let fx = candidate();
    fx.spec("F-0004", fai_fixture::ADVANCE, "");
    accept(&fx, "F-0004");
    route(&fx, "F-0004");
    assert!(refusal(&fx, "F-0004").contains("advanced the Product Journey Frontier"));
}

#[test]
fn an_accepted_feature_deleted_without_a_retirement_decision_blocks() {
    let fx = candidate();
    remove(&fx, "F-0002");
    let blockers = retirement_blockers(&fx);
    assert_eq!(blockers.len(), 1, "{blockers:?}");
    assert!(blockers[0].contains("removed without a retirement decision"));
    assert!(ProjectStatus::derive(&fx.project()).unwrap().is_blocked());
}

#[test]
fn dba_29_neither_routes_retirement_nor_reports_it() {
    let fx = Fixture::under("DBA-29", "    provider: [backend]\n");
    fx.spec("F-0002", PRESERVE, "");
    accept(&fx, "F-0002");
    let store = ObservationStore::at(&fx.root.join(".codeos"));
    let observation = store.observe("F-0002 is unused".into(), vec![]).unwrap();
    let err = store
        .route_observation(
            &fx.project(),
            &observation.id,
            "feature_retirement",
            Some("feature".into()),
            Some("F-0002".into()),
        )
        .unwrap_err();
    assert!(err.to_string().contains("invalid operation route"));
    remove(&fx, "F-0002");
    assert!(retirement_blockers(&fx).is_empty());
}

#[test]
fn a_feature_is_retired_once() {
    let fx = candidate();
    route(&fx, "F-0002");
    assert!(decide(&fx, "F-0002").status.success());
    assert!(refusal(&fx, "F-0002").contains("F-0002 is already retired"));
}

#[test]
fn closure_requires_removal_and_a_later_retirement_receipt_for_the_target() {
    let fx = candidate();
    let observation = route(&fx, "F-0002");
    assert!(decide(&fx, "F-0002").status.success());
    let receipt = receipt_identity(&fx, "retirement");

    let why = close(&fx, &observation, receipt.clone())
        .unwrap_err()
        .to_string();
    assert!(why.contains("Specification Package to be removed"), "{why}");

    remove(&fx, "F-0002");
    let why = close(&fx, &observation, receipt_identity(&fx, "acceptance"))
        .unwrap_err()
        .to_string();
    assert!(
        why.contains("absent, too early, or targets another subject"),
        "{why}"
    );

    // Receipts record whole seconds; a route in a later second makes the receipt too early.
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let late = route(&fx, "F-0002");
    let why = close(&fx, &late, receipt.clone()).unwrap_err().to_string();
    assert!(
        why.contains("absent, too early, or targets another subject"),
        "{why}"
    );

    close(&fx, &observation, receipt).unwrap();
}
