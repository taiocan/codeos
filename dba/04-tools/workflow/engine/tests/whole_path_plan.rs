//! VALIDITY WO-05: the whole-path plan lists exactly the human gates `check` requires, says which
//! can be consented to now, and never offers F8 or F9 up front.

mod fai_fixture;

use codeos_workflow::checker::{evaluate, whole_path_plan, Actor, State};
use fai_fixture::*;
use std::collections::BTreeSet;

fn gates(fx: &Fixture, consent: Option<&str>) -> BTreeSet<String> {
    whole_path_plan(&fx.project(), &fx.subject("F-0001"))
        .unwrap()
        .into_iter()
        .filter(|s| s.kind == "decision" && consent.is_none_or(|c| s.consent == c))
        .map(|s| s.checkpoint)
        .collect()
}

/// The human gates `check` waits on now: WAITING checkpoints whose actor is the human.
fn waiting_human(fx: &Fixture) -> BTreeSet<String> {
    evaluate(&fx.project(), &fx.subject("F-0001"))
        .unwrap()
        .into_iter()
        .filter(|r| r.state == State::Waiting && r.actor == Some(Actor::Human))
        .map(|r| r.id)
        .collect()
}

#[test]
fn the_plan_lists_the_gates_check_requires_and_offers_only_ready_ones() {
    let fx = Fixture::under("DBA-36", "    provider: [backend]\n");
    fx.write(
        "claim-scopes.json",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    fx.spec("F-0001", ADVANCE, ROUTE);
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "spec"]);

    // Before any verification: every remaining gate is listed, none can be consented to yet.
    assert_eq!(
        gates(&fx, None),
        BTreeSet::from(["F6".to_string(), "F9".to_string()])
    );
    assert_eq!(gates(&fx, Some("now")), waiting_human(&fx));
    assert!(gates(&fx, Some("now")).is_empty());

    // After the verifications pass, F6 is ready: the plan offers exactly what `check` waits on.
    fx.check("F-0001");
    assert_eq!(gates(&fx, Some("now")), waiting_human(&fx));
    assert_eq!(gates(&fx, Some("now")), BTreeSet::from(["F6".to_string()]));
    assert!(
        gates(&fx, Some("later")).contains("F9"),
        "F9 is never up front"
    );

    // Once accepted, no decision remains, and the plan names exactly the checkpoints `check`
    // still reports open (the fixture's acceptance skips the Stage 8 review).
    let evidence = fx.evidence("F-0001");
    fx.accept("F-0001", Some(&evidence));
    assert!(gates(&fx, None).is_empty());
    let planned: BTreeSet<_> = whole_path_plan(&fx.project(), &fx.subject("F-0001"))
        .unwrap()
        .into_iter()
        .map(|s| s.checkpoint)
        .collect();
    let open: BTreeSet<_> = evaluate(&fx.project(), &fx.subject("F-0001"))
        .unwrap()
        .into_iter()
        .filter(|r| !matches!(r.state, State::Pass | State::NotApplicable))
        .map(|r| r.id)
        .collect();
    assert_eq!(planned, open);
}

/// A GUI feature: the F4 and F8 gates show exactly the `observation:human` claims their
/// checkpoint asks the human to judge, never another checkpoint's or another feature's; a current
/// F4 is not offered again; F8 and F9 stay later.
#[test]
fn human_gates_carry_exactly_their_own_claims() {
    use codeos_workflow::session::{self, Outcome};
    use codeos_workflow::verification::{VerificationRecord, VerificationStore};
    let fx = Fixture::under("DBA-36", "    provider: [backend]\n");
    for feature in ["F-0001", "F-0002"] {
        fx.spec(feature, "mode: preserve\nreason: shows the page", "");
        let path = fx.root.join(format!(
            ".codeos/01-specification/contracts/{feature}_contract.md"
        ));
        let text = std::fs::read_to_string(&path).unwrap().replace(
            "| GUI (Svelte) | unchanged | none |",
            "| GUI (Svelte) | changed | page |",
        );
        let rows = if feature == "F-0001" {
            "| F-0001/readable | The page is readable. | F4 | observation:human |\n\
             | F-0001/final-readable | The final page is readable. | F8 | observation:human |\n\
             | F-0001/route-shown | The route is shown. | F5 | run:behavior |\n"
        } else {
            "| F-0002/clear | The return page is clear. | F4 | observation:human |\n"
        };
        std::fs::write(
            &path,
            format!("{text}\n## Validation Claims\n\n| Claim ID | Claim | Checkpoint | Evidence |\n|---|---|---|---|\n{rows}"),
        )
        .unwrap();
    }
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "specs"]);
    fx.seed("F-0001", "F3", "smoke");
    let bindings = codeos_workflow::checker::verification_bindings_for_mechanic(
        &fx.project(),
        &fx.subject("F-0001"),
        "F3",
        "product_journey",
    )
    .unwrap();
    VerificationStore::at(&fx.root.join(".codeos"))
        .append(&VerificationRecord {
            workflow: "feature".into(),
            subject: "F-0001".into(),
            checkpoint: "F3".into(),
            verification: "product_journey".into(),
            result: "passed".into(),
            bindings,
            command: "seeded".into(),
            timestamp: "2026-11-18T00:00:00Z".into(),
            details: Some(serde_json::json!({ "observed_surface": "surface-1" })),
        })
        .unwrap();

    let step = |checkpoint: &str| {
        whole_path_plan(&fx.project(), &fx.subject("F-0001"))
            .unwrap()
            .into_iter()
            .find(|s| s.kind == "decision" && s.checkpoint == checkpoint)
    };
    let f4 = step("F4").expect("F4 is a remaining gate");
    assert_eq!(f4.claims, vec!["F-0001/readable".to_string()]);
    assert_eq!(f4.consent, "now");
    assert_eq!(gates(&fx, Some("now")), waiting_human(&fx));
    assert!(
        f4.command
            .contains("--subject F-0001 --checkpoint early_preview"),
        "{}",
        f4.command
    );
    let f8 = step("F8").expect("F8 is a remaining gate");
    assert_eq!(f8.claims, vec!["F-0001/final-readable".to_string()]);
    assert_eq!(f8.consent, "later");
    assert_eq!(step("F9").expect("F9 remains").consent, "later");

    // Once a session records F4 with its claim, F4 is not offered again.
    session::record(
        &fx.project(),
        vec![Outcome {
            feature: "F-0001".into(),
            checkpoint: "early_preview".into(),
            result: "direction_confirmed".into(),
            claims: [("F-0001/readable".to_string(), "met".to_string())]
                .into_iter()
                .collect(),
        }],
        "seen surface-1",
    )
    .unwrap();
    assert!(step("F4").is_none());
    assert_eq!(step("F8").expect("F8 remains").consent, "later");
}
