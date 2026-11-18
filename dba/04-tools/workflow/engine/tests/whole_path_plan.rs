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
