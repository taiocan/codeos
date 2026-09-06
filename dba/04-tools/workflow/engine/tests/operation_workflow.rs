//! Operation & Learning O1–O5: every route classified by a human, the tool never selecting one,
//! closure derived or (for no_action) recorded.

use codeos_workflow::checker::{evaluate, State, Subject};
use codeos_workflow::contract::{Workflow, OPERATION_ROUTES};
use codeos_workflow::hashing::text_sha256;
use codeos_workflow::project::Project;
use codeos_workflow::receipts::{Receipt, ReceiptStore};
use std::collections::BTreeMap;
use std::fs;

struct Fx {
    _d: tempfile::TempDir,
    root: std::path::PathBuf,
}
impl Fx {
    fn new() -> Self {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().to_path_buf();
        fs::create_dir_all(root.join(".codeos/06-workflow")).unwrap();
        Fx { _d: d, root }
    }
    fn project(&self) -> Project {
        Project::discover(&self.root).unwrap()
    }
    fn store(&self) -> ReceiptStore {
        ReceiptStore::at(&self.root.join(".codeos"))
    }
    fn subj(&self, id: &str) -> Subject {
        Subject::resolve(&self.project(), Workflow::Operation, id).unwrap()
    }
    fn route(&self, subject: &str, route: &str, obs: &str) {
        let mut b = BTreeMap::new();
        b.insert("observation".into(), text_sha256(obs));
        self.store()
            .append(&Receipt {
                workflow: "operation".into(),
                subject: subject.into(),
                checkpoint: "operation_route".into(),
                result: route.into(),
                bindings: b,
                timestamp: "2026-09-05T10:00:00Z".into(),
                rationale: None,
                observation: Some(obs.into()),
            })
            .unwrap();
    }
    fn accept(&self, feature: &str, ts: &str) {
        self.store()
            .append(&Receipt {
                workflow: "feature".into(),
                subject: feature.into(),
                checkpoint: "acceptance".into(),
                result: "accepted".into(),
                bindings: BTreeMap::new(),
                timestamp: ts.into(),
                rationale: None,
                observation: None,
            })
            .unwrap();
    }
}
fn st<'a>(
    r: &'a [codeos_workflow::checker::CheckpointReport],
    id: &str,
) -> &'a codeos_workflow::checker::CheckpointReport {
    r.iter().find(|x| x.id == id).unwrap()
}

#[test]
fn without_a_route_o1_and_o2_wait_and_the_tool_names_no_route() {
    let fx = Fx::new();
    let r = evaluate(&fx.project(), &fx.subj("obs-slow-search")).unwrap();
    assert_eq!(st(&r, "O1").state, State::Waiting);
    assert_eq!(st(&r, "O2").state, State::Blocked);
    // The report contains no route name — it only asks the human to record one.
    let joined = format!("{r:?}");
    for route in OPERATION_ROUTES {
        // "no_action" appears only in the suggested command template, never as a chosen result.
        if *route != "no_action" {
            assert!(
                !joined.contains(&format!("result: \"{route}\"")),
                "{route} leaked as a decision"
            );
        }
    }
}

#[test]
fn no_action_route_needs_an_explicit_closure_receipt_then_closes() {
    let fx = Fx::new();
    let obs = "cosmetic: the footer year is stale; not worth a change now";
    fx.route("obs-footer-year", "no_action", obs);

    let r = evaluate(&fx.project(), &fx.subj("obs-footer-year")).unwrap();
    assert_eq!(st(&r, "O1").state, State::Pass);
    assert_eq!(st(&r, "O2").state, State::Pass);
    assert_eq!(st(&r, "O4").state, State::Waiting);
    assert!(st(&r, "O4")
        .unmet
        .iter()
        .any(|u| u.contains("no closure receipt")));

    // Missing rationale must fail closed.
    let mut b = BTreeMap::new();
    b.insert("observation".into(), text_sha256(obs));
    let closure = Receipt {
        workflow: "operation".into(),
        subject: "obs-footer-year".into(),
        checkpoint: "no_action_closure".into(),
        result: "no_action".into(),
        bindings: b.clone(),
        timestamp: "2026-09-05T11:00:00Z".into(),
        rationale: None,
        observation: Some(obs.into()),
    };
    assert!(
        fx.store().append(&closure).is_err(),
        "closure without rationale must be rejected"
    );

    let closure = Receipt {
        rationale: Some("purely cosmetic; revisit at next UI pass".into()),
        ..closure
    };
    fx.store().append(&closure).unwrap();
    let r = evaluate(&fx.project(), &fx.subj("obs-footer-year")).unwrap();
    assert_eq!(st(&r, "O4").state, State::Pass);
    assert_eq!(st(&r, "O5").state, State::Pass);
}

#[test]
fn new_feature_route_closes_on_the_new_feature_acceptance() {
    let fx = Fx::new();
    fx.route("F-0002", "new_feature", "users want CSV export of notes");
    let r = evaluate(&fx.project(), &fx.subj("F-0002")).unwrap();
    assert_eq!(st(&r, "O4").state, State::Blocked);

    fx.accept("F-0002", "2026-09-06T09:00:00Z"); // after the route decision
    let r = evaluate(&fx.project(), &fx.subj("F-0002")).unwrap();
    assert_eq!(st(&r, "O4").state, State::Pass);
    assert_eq!(st(&r, "O5").state, State::Pass);
}

#[test]
fn implementation_defect_needs_a_re_acceptance_after_the_route() {
    let fx = Fx::new();
    fx.accept("F-0001", "2026-09-04T00:00:00Z"); // the ORIGINAL acceptance, before the observation
    fx.route(
        "F-0001",
        "implementation_defect",
        "empty note is accepted in production",
    );
    let r = evaluate(&fx.project(), &fx.subj("F-0001")).unwrap();
    // the pre-existing acceptance does not count — it predates the route decision
    assert_eq!(st(&r, "O4").state, State::Blocked, "{:?}", st(&r, "O4"));

    fx.accept("F-0001", "2026-09-07T00:00:00Z"); // re-acceptance after the fix
    let r = evaluate(&fx.project(), &fx.subj("F-0001")).unwrap();
    assert_eq!(st(&r, "O4").state, State::Pass);
}

#[test]
fn charter_and_architecture_routes_wait_for_a_human_linked_resolution() {
    let fx = Fx::new();
    for route in ["charter_change", "architecture_reassessment"] {
        fx.route(
            &format!("obs-{route}"),
            route,
            "external regulation changed",
        );
        let r = evaluate(&fx.project(), &fx.subj(&format!("obs-{route}"))).unwrap();
        let o4 = st(&r, "O4");
        assert_eq!(o4.state, State::Waiting, "{route}: {o4:?}");
        assert!(o4.unmet.iter().any(|u| u.contains("not automatic")));
    }
}
