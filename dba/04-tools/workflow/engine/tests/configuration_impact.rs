//! VALIDITY WO-04 (Rule W4): a consumer binds the `codeos.yaml` sections it reads and the versions
//! of the contracts it applies, never the whole file. Runs under DBA-36; DBA-35 is the contrast.

mod fai_fixture;

use codeos_workflow::checker::{
    bindings_for_decision, decision_state, qualification_bindings, verification_bindings_for,
    verification_bindings_for_mechanic, Subject,
};
use codeos_workflow::contract::Workflow;
use codeos_workflow::receipts::{Receipt, ReceiptState, ReceiptStore};
use codeos_workflow::verification::{evaluate, RecordState, VerificationStore};
use fai_fixture::*;
use std::fs;

const SCOPES: &str = "    provider: [backend]\n";

/// F-0001 accepted, plus a B4 receipt, under DBA-36 (`w4`) or DBA-35.
fn accepted(w4: bool) -> Fixture {
    let fx = if w4 {
        Fixture::under("DBA-36", SCOPES)
    } else {
        Fixture::under("DBA-35", SCOPES)
    };
    assert_eq!(
        fx.project().feature_workflow_version().unwrap(),
        if w4 { 15 } else { 14 }
    );
    fx.write(
        "claim-scopes.json",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    fx.spec("F-0001", ADVANCE, ROUTE);
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "spec"]);
    fx.check("F-0001");
    let evidence = fx.evidence("F-0001");
    fx.accept("F-0001", Some(&evidence));
    let bootstrap = solution(&fx);
    ReceiptStore::at(&fx.root.join(".codeos"))
        .append(&Receipt {
            workflow: "bootstrap".into(),
            subject: "solution".into(),
            checkpoint: "initial_product_preview".into(),
            result: "direction_confirmed".into(),
            bindings: bindings_for_decision(&fx.project(), &bootstrap, "initial_product_preview")
                .unwrap(),
            timestamp: "2026-11-18T00:00:00Z".into(),
            rationale: None,
            observation: None,
        })
        .unwrap();
    fx
}

fn solution(fx: &Fixture) -> Subject {
    Subject::resolve(&fx.project(), Workflow::Bootstrap, "solution").unwrap()
}

/// What each consumer depends on now: B3, the journey's execution identity.
fn identities(fx: &Fixture) -> (std::collections::BTreeMap<String, String>, String) {
    let project = fx.project();
    let b3 = verification_bindings_for(&project, &solution(fx), "B3").unwrap();
    let journey = verification_bindings_for_mechanic(
        &project,
        &fx.subject("F-0001"),
        "F3",
        "product_journey",
    )
    .unwrap()["execution_identity"]
        .clone();
    (b3, journey)
}

fn current(state: ReceiptState) -> bool {
    matches!(state, ReceiptState::Current(_))
}

fn qualification_current(fx: &Fixture) -> bool {
    let project = fx.project();
    let record = VerificationStore::at(&fx.root.join(".codeos"))
        .latest("feature", "F-0001", "candidate", "project_qualification")
        .unwrap();
    let want = qualification_bindings(&project, &fx.subject("F-0001"), "candidate", &[]).unwrap();
    matches!(evaluate(record, &want), RecordState::Current(_))
}

fn edit_yaml(fx: &Fixture, f: impl Fn(&str) -> String) {
    let path = fx.root.join(".codeos/00-project/codeos.yaml");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, f(&text)).unwrap();
}

/// The DBA-34 → DBA-35 case: an edit to codeos.yaml that touches no section a consumer reads stales
/// nothing.
#[test]
fn an_edit_outside_the_read_sections_stales_no_consumer() {
    for w4 in [true, false] {
        let fx = accepted(w4);
        let before = identities(&fx);
        assert!(qualification_current(&fx));
        edit_yaml(&fx, |text| format!("# adopted elsewhere\n{text}"));
        let after = identities(&fx);
        assert_eq!(after.0 == before.0, w4, "B3 (w4={w4})");
        assert_eq!(after.1 == before.1, w4, "execution identity (w4={w4})");
        assert_eq!(qualification_current(&fx), w4, "qualification (w4={w4})");
        let project = fx.project();
        assert_eq!(
            current(decision_state(&project, &fx.subject("F-0001"), "reconciliation").unwrap()),
            w4,
            "F6 (w4={w4})"
        );
        assert_eq!(
            current(decision_state(&project, &solution(&fx), "initial_product_preview").unwrap()),
            w4,
            "B4 (w4={w4})"
        );
    }
}

/// A change to a section a consumer reads still stales it.
#[test]
fn a_platform_change_stales_b3_b4_and_execution_identities() {
    let fx = accepted(true);
    let before = identities(&fx);
    edit_yaml(&fx, |text| {
        text.replace(
            "codeos_schema: 2\n",
            "codeos_schema: 2\nplatform:\n  backend: rust\n",
        )
    });
    let after = identities(&fx);
    assert_ne!(after.0, before.0, "B3 reads platform");
    assert_ne!(after.1, before.1, "execution reads platform");
    assert!(!current(
        decision_state(&fx.project(), &solution(&fx), "initial_product_preview").unwrap()
    ));
    // Qualification is a shared verification: it reaches `platform` through its execution
    // identity, once, not through a direct binding.
    assert!(!qualification_current(&fx));
}

/// Sections compare by parsed value: quoting and key order change nothing.
#[test]
fn a_reality_verification_change_stales_execution_but_quoting_does_not() {
    let fx = accepted(true);
    let before = identities(&fx).1;
    edit_yaml(&fx, |text| {
        text.replace(
            "  command: scripts/verify.sh",
            "  command: \"scripts/verify.sh\"",
        )
    });
    assert_eq!(identities(&fx).1, before, "quoting is not a change");
    edit_yaml(&fx, |text| {
        text.replace(
            "  command: \"scripts/verify.sh\"",
            "  command: scripts/verify-v2.sh",
        )
    });
    assert_ne!(identities(&fx).1, before);
}

/// Adopting DBA-36: an F6 receipt recorded under DBA-35 (v14) stays current, because v15 drops
/// `codeos.yaml` from its dependency set; B4 recorded before Bootstrap v2 is evaluated as recorded.
#[test]
fn adopting_dba_36_keeps_v14_f6_and_re_asks_legacy_b4_once() {
    let fx = Fixture::under("DBA-35", SCOPES);
    fx.write(
        "claim-scopes.json",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    fx.spec("F-0001", ADVANCE, ROUTE);
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "spec"]);
    fx.check("F-0001");
    let evidence = fx.evidence("F-0001");
    fx.accept("F-0001", Some(&evidence));
    ReceiptStore::at(&fx.root.join(".codeos"))
        .append(&Receipt {
            workflow: "bootstrap".into(),
            subject: "solution".into(),
            checkpoint: "initial_product_preview".into(),
            result: "direction_confirmed".into(),
            bindings: bindings_for_decision(
                &fx.project(),
                &solution(&fx),
                "initial_product_preview",
            )
            .unwrap(),
            timestamp: "2026-11-18T00:00:00Z".into(),
            rationale: None,
            observation: None,
        })
        .unwrap();
    edit_yaml(&fx, |text| {
        text.replace("dba_configuration: DBA-35", "dba_configuration: DBA-36")
    });
    let project = fx.project();
    assert_eq!(project.feature_workflow_version().unwrap(), 15);
    assert!(current(
        decision_state(&project, &fx.subject("F-0001"), "reconciliation").unwrap()
    ));
    assert!(!current(
        decision_state(&project, &solution(&fx), "initial_product_preview").unwrap()
    ));
}
