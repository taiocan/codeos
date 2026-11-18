//! Feature Workflow v14 (VALIDITY WO-02): a change invalidates only the decisions whose own
//! dependency set changed. Runs under DBA-35; DBA-34 is the contrast.

mod fai_fixture;

use codeos_workflow::checker::decision_state;
use codeos_workflow::receipts::{Receipt, ReceiptState, ReceiptStore};
use fai_fixture::*;
use std::fs;

const SCOPES: &str = "    provider: [backend]\n";

/// Whether the latest receipt is current. Under v13, F8 and F9 cannot be evaluated without a
/// unique current Candidate Bundle, which also means not current.
fn current(fx: &Fixture, feature: &str, checkpoint: &str) -> bool {
    matches!(
        decision_state(&fx.project(), &fx.subject(feature), checkpoint),
        Ok(ReceiptState::Current(_))
    )
}

fn commit(fx: &Fixture, message: &str) {
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", message, "--allow-empty"]);
}

/// F-0001 accepted under DBA-35 (`v14`) or under DBA-34.
fn accepted(v14: bool) -> Fixture {
    let fx = if v14 {
        Fixture::under("DBA-35", SCOPES)
    } else {
        Fixture::under("DBA-34", SCOPES)
    };
    assert_eq!(
        fx.project().feature_workflow_version().unwrap(),
        if v14 { 14 } else { 13 }
    );
    fx.write(
        "claim-scopes.json",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    fx.spec("F-0001", ADVANCE, ROUTE);
    commit(&fx, "spec");
    fx.check("F-0001");
    let evidence = fx.evidence("F-0001");
    fx.accept("F-0001", Some(&evidence));
    assert!(current(&fx, "F-0001", "acceptance"));
    fx
}

fn edit(fx: &Fixture, rel: &str, from: &str, to: &str) {
    let path = fx.root.join(rel);
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains(from), "{rel} has no '{from}'");
    fs::write(&path, text.replacen(from, to, 1)).unwrap();
}

const CONTRACT: &str = ".codeos/01-specification/contracts/F-0001_contract.md";
const INTENT: &str = ".codeos/01-specification/intents/F-0001.md";

/// R1: acceptance binds governed meaning, not implementation bytes. R2: Current Validity still
/// follows the implementation.
#[test]
fn a_runtime_change_keeps_acceptance_and_stales_only_current_validity() {
    for v14 in [true, false] {
        let fx = accepted(v14);
        std::thread::sleep(std::time::Duration::from_millis(1100));
        fx.write("backend/Cargo.lock", "# rustls 0.23.46\n");
        commit(&fx, "runtime change");
        assert_eq!(current(&fx, "F-0001", "acceptance"), v14, "v14={v14}");
        assert!(
            !codeos_workflow::revalidation::current_acceptance_valid(&fx.project(), "F-0001")
                .unwrap(),
            "Current Validity follows the implementation (v14={v14})"
        );
        assert!(
            !current(&fx, "F-0001", "reconciliation"),
            "F6 needs revalidation first"
        );
    }
}

/// R1/R2: a PASSED Post-Acceptance Dependency Revalidation satisfies F6 without a new receipt.
#[test]
fn a_passed_revalidation_satisfies_reconciliation_without_a_new_f6() {
    let fx = accepted(true);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fx.write("backend/Cargo.lock", "# rustls 0.23.46\n");
    commit(&fx, "trigger");
    run_checks_revalidation(&fx);
    fx.seed("F-0001", "candidate", "project_qualification");
    let (bundle, _) = codeos_workflow::candidate::prepare(
        &fx.project(),
        &fx.subject("F-0001"),
        &[],
        "post_acceptance_revalidation",
        Some("HEAD"),
    )
    .unwrap();
    assert!(
        !current(&fx, "F-0001", "reconciliation"),
        "not before the review passes"
    );
    fx.write(
        ".codeos/05-review/reviews/codex/2026-11-18T000000Z-F-0001-stage-8-test.md",
        &format!("---\nreview_id: test\nreviewed:\n  source: codex\n  feature: F-0001\n  stage: 8\n  review_commit: abc\n  reviewed_packet_sha256: packet\n  effective_concern: NO OBJECTION\n  candidate_bundle: {bundle}\n  candidate_outcome: PASSED\n---\n"),
    );
    assert!(current(&fx, "F-0001", "reconciliation"));
    assert!(current(&fx, "F-0001", "acceptance"));
    assert!(
        codeos_workflow::revalidation::current_acceptance_valid(&fx.project(), "F-0001").unwrap()
    );
}

/// R4: structured fields compare by their canonical parsed form.
#[test]
fn a_yaml_quoting_change_stales_nothing() {
    for v14 in [true, false] {
        let fx = accepted(v14);
        let bundle = codeos_workflow::candidate::unique_current(&fx.project(), "F-0001").unwrap();
        edit(
            &fx,
            INTENT,
            "semantic_change: initial feature specification",
            "semantic_change: \"initial feature specification\"",
        );
        assert_eq!(current(&fx, "F-0001", "acceptance"), v14, "v14={v14}");
        if v14 {
            // Verifications and the bundle depend on meaning too, so nothing re-runs.
            assert_eq!(fx.evidence_current("F-0001"), Ok(()));
            assert_eq!(
                codeos_workflow::candidate::unique_current(&fx.project(), "F-0001").unwrap(),
                bundle
            );
        }
    }
}

/// R4: prose compares byte for byte, unless a Semantic Equivalence Receipt covers the exact delta.
#[test]
fn a_prose_change_needs_an_exact_semantic_equivalence_receipt() {
    let fx = accepted(true);
    let project = fx.project();
    let path = fx.root.join(CONTRACT);
    let before = codeos_workflow::validity::spec_meaning(&path)
        .unwrap()
        .unwrap();
    edit(
        &fx,
        CONTRACT,
        "| label | Heading |",
        "| label | The heading |",
    );
    assert!(
        !current(&fx, "F-0001", "acceptance"),
        "prose is compared byte for byte"
    );

    let unbound = "0".repeat(64);
    assert!(
        codeos_workflow::validity::equivalence_bindings(&project, "F-0001", "contract", &unbound)
            .is_err(),
        "the receipt must name an identity a decision depended on"
    );
    let bindings =
        codeos_workflow::validity::equivalence_bindings(&project, "F-0001", "contract", &before)
            .unwrap();
    ReceiptStore::at(&fx.root.join(".codeos"))
        .append(&Receipt {
            workflow: "feature".into(),
            subject: "F-0001".into(),
            checkpoint: "semantic_equivalence".into(),
            result: "equivalent".into(),
            bindings,
            timestamp: "2026-11-18T00:00:00Z".into(),
            rationale: Some("wording only".into()),
            observation: None,
        })
        .unwrap();
    assert!(
        current(&fx, "F-0001", "acceptance"),
        "the receipt covers this exact delta"
    );

    edit(
        &fx,
        CONTRACT,
        "| label | The heading |",
        "| label | Another heading |",
    );
    assert!(
        !current(&fx, "F-0001", "acceptance"),
        "a further change is not covered"
    );
}

/// R3: a Candidate Bundle cites content, so a re-run with identical content keeps it current.
#[test]
fn a_rerun_record_with_identical_content_keeps_the_bundle() {
    for v14 in [true, false] {
        let fx = accepted(v14);
        let before = codeos_workflow::candidate::unique_current(&fx.project(), "F-0001").unwrap();
        assert!(before.is_some());
        std::thread::sleep(std::time::Duration::from_millis(1100));
        fx.seed("F-0001", "F3", "smoke");
        let after = codeos_workflow::candidate::unique_current(&fx.project(), "F-0001").unwrap();
        assert_eq!(after == before, v14, "v14={v14}");
    }
}

/// R5: a v14 bundle and F9 do not depend on the bytes of codeos.yaml.
#[test]
fn a_non_semantic_configuration_edit_stales_neither_acceptance_nor_the_bundle() {
    for v14 in [true, false] {
        let fx = accepted(v14);
        let yaml = fx.root.join(".codeos/00-project/codeos.yaml");
        let text = fs::read_to_string(&yaml).unwrap();
        fs::write(&yaml, format!("# a comment\n{text}")).unwrap();
        assert_eq!(current(&fx, "F-0001", "acceptance"), v14, "v14={v14}");
    }
}

/// F4 binds the Intent's meaning, its judged claims and the judged product, not the Contract.
#[test]
fn an_early_preview_ignores_contract_prose_but_not_intent_prose() {
    let fx = Fixture::under("DBA-35", SCOPES);
    fx.spec("F-0001", ADVANCE, ROUTE);
    commit(&fx, "spec");
    fx.receipt("F-0001", "early_preview", "direction_confirmed", None);
    assert!(current(&fx, "F-0001", "early_preview"));
    edit(
        &fx,
        CONTRACT,
        "| label | Heading |",
        "| label | The heading |",
    );
    assert!(current(&fx, "F-0001", "early_preview"));
    edit(&fx, INTENT, "# Intent: F-0001", "# Intent: F-0001, revised");
    assert!(!current(&fx, "F-0001", "early_preview"));
}

/// Compatibility: a receipt recorded before v14 is compared as recorded after adoption.
#[test]
fn a_receipt_recorded_before_v14_stays_current_after_adoption() {
    let fx = Fixture::under("DBA-34", SCOPES);
    fx.spec("F-0001", ADVANCE, ROUTE);
    commit(&fx, "spec");
    fx.receipt("F-0001", "early_preview", "direction_confirmed", None);
    edit(
        &fx,
        ".codeos/00-project/codeos.yaml",
        "dba_configuration: DBA-34",
        "dba_configuration: DBA-35",
    );
    assert_eq!(fx.project().feature_workflow_version().unwrap(), 14);
    assert!(current(&fx, "F-0001", "early_preview"));
}
