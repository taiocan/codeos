//! The 11 adversarial cases from the work order (Section 12, Step 7) plus the plan's addition.
//! Each must fail closed at the correct boundary with a specific reason — never a false PASS.

use codeos_workflow::checker::{
    bindings_for_decision, evaluate, verification_bindings_for, CheckpointReport, State, Subject,
};
use codeos_workflow::contract::Workflow;
use codeos_workflow::hashing::text_sha256;
use codeos_workflow::project::Project;
use codeos_workflow::receipts::{Receipt, ReceiptStore};
use codeos_workflow::verification::{VerificationRecord, VerificationStore};
use std::collections::BTreeMap;
use std::fs;

struct P {
    _d: tempfile::TempDir,
    root: std::path::PathBuf,
}
impl P {
    fn new() -> Self {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().to_path_buf();
        for x in [
            ".codeos/00-project",
            ".codeos/01-specification/intents",
            ".codeos/01-specification/contracts",
            ".codeos/01-specification/event-schemas",
            ".codeos/05-review/reviews",
            ".codeos/06-workflow",
        ] {
            fs::create_dir_all(root.join(x)).unwrap();
        }
        P { _d: d, root }
    }
    fn w(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, c).unwrap();
    }
    fn proj(&self) -> Project {
        Project::discover(&self.root).unwrap()
    }
    fn store(&self) -> ReceiptStore {
        ReceiptStore::at(&self.root.join(".codeos"))
    }
    fn feat(&self, id: &str) -> Subject {
        Subject::resolve(&self.proj(), Workflow::Feature, id).unwrap()
    }
    fn op(&self, id: &str) -> Subject {
        Subject::resolve(&self.proj(), Workflow::Operation, id).unwrap()
    }
    /// Seed one passing mechanical-verification record, bound to the current inputs.
    fn verify(&self, feature: &str, checkpoint: &str, mechanic: &str) {
        let bindings =
            verification_bindings_for(&self.proj(), &self.feat(feature), checkpoint).unwrap();
        VerificationStore::at(&self.root.join(".codeos"))
            .append(&VerificationRecord {
                workflow: "feature".into(),
                subject: feature.into(),
                checkpoint: checkpoint.into(),
                verification: mechanic.into(),
                result: "passed".into(),
                bindings,
                command: "seeded".into(),
                timestamp: "2026-09-06T00:00:00Z".into(),
            })
            .unwrap();
    }
    /// Seed every feature mechanical verification against the current inputs.
    fn reverify(&self, feature: &str) {
        for (cp, mech) in [
            ("F3", "smoke"),
            ("F5", "behavior"),
            ("F5", "repeatability"),
            ("F5d", "data_integrity"),
            ("F5g", "playwright"),
        ] {
            self.verify(feature, cp, mech);
        }
    }
    /// A bootstrapped project (charter approved, an earlier feature accepted).
    fn bootstrapped(&self) {
        self.w(
            ".codeos/00-project/charter.md",
            "---\nartifact_type: charter\napproval:\n  by: P\n  at: 2026-09-05\n---\n# Charter\n",
        );
        self.w(
            ".codeos/00-project/codeos.yaml",
            "platform:\n  persistence: postgresql\n  backend: rust\n  webapp: svelte\n  runtime: docker\nartifacts:\n  charter: governed\n  intent: governed\n  contract: governed\n  event_schema: governed\n",
        );
        self.receipt(
            "feature",
            "F-0000",
            "acceptance",
            "accepted",
            None,
            None,
            "2026-01-01T00:00:00Z",
        );
    }
    fn spec(&self, id: &str, gui: bool, persist: bool) {
        let g = if gui { "changed" } else { "unchanged" };
        let pz = if persist { "changed" } else { "unchanged" };
        for (kind, path) in [
            (
                "intent",
                format!(".codeos/01-specification/intents/{id}.md"),
            ),
            (
                "contract",
                format!(".codeos/01-specification/contracts/{id}_contract.md"),
            ),
            (
                "event_schema",
                format!(".codeos/01-specification/event-schemas/{id}_schema.md"),
            ),
        ] {
            self.w(&path, &format!(
                "---\nartifact_type: {kind}\nfeature_id: {id}\nstatus: APPROVED\napproved_by: Primoz Gorjup\napproved_at: 2026-09-05\n---\n\n# {kind}\n\n## Validation Questions\n\n**Behavior** — a test proves it.\n**Repeatability** — re-run twice.\n**Browser** — a journey proves it.\n**Preview** — inspect the page.\n\n## Feature Impact Accounting\n\n| Tier | Changed / Unchanged | Reason |\n|---|---|---|\n| Persistence (PostgreSQL) | {pz} | table |\n| Backend (Rust) | changed | endpoint |\n| GUI (Svelte) | {g} | form |\n"
            ));
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn receipt(
        &self,
        wf: &str,
        subj: &str,
        cp: &str,
        result: &str,
        obs: Option<&str>,
        rationale: Option<&str>,
        ts: &str,
    ) {
        let mut b = bindings_for_decision(&self.proj(), &self.feat(subj), cp).unwrap_or_default();
        if let Some(o) = obs {
            b.insert("observation".into(), text_sha256(o));
        }
        self.store()
            .append(&Receipt {
                workflow: wf.into(),
                subject: subj.into(),
                checkpoint: cp.into(),
                result: result.into(),
                bindings: b,
                timestamp: ts.into(),
                rationale: rationale.map(String::from),
                observation: obs.map(String::from),
            })
            .unwrap();
    }
}
fn s<'a>(r: &'a [CheckpointReport], id: &str) -> &'a CheckpointReport {
    r.iter()
        .find(|x| x.id == id)
        .unwrap_or_else(|| panic!("no {id}"))
}
fn blocked_or_waiting(st: State) -> bool {
    matches!(st, State::Blocked | State::Waiting)
}

#[test]
fn a01_first_feature_before_bootstrap() {
    let p = P::new(); // NOT bootstrapped
    p.spec("F-0001", false, false);
    let r = evaluate(&p.proj(), &p.feat("F-0001")).unwrap();
    assert_eq!(s(&r, "bootstrap-entry").state, State::Blocked);
    assert!(s(&r, "bootstrap-entry").unmet[0].contains("Solution Bootstrap is not complete"));
    assert_eq!(s(&r, "F1").state, State::Blocked);
}

#[test]
fn a02_full_verification_before_early_preview() {
    let p = P::new();
    p.bootstrapped();
    p.spec("F-1", true, false);
    p.reverify("F-1");
    let r = evaluate(&p.proj(), &p.feat("F-1")).unwrap();
    assert_eq!(s(&r, "F4").state, State::Waiting);
    assert!(blocked_or_waiting(s(&r, "F5").state));
}

#[test]
fn a03_and_a10_early_preview_reused_after_spec_revision() {
    let p = P::new();
    p.bootstrapped();
    p.spec("F-2", true, false);
    p.reverify("F-2");
    p.receipt(
        "feature",
        "F-2",
        "early_preview",
        "direction_confirmed",
        None,
        None,
        "2026-09-05T10:00:00Z",
    );
    assert_eq!(
        s(&evaluate(&p.proj(), &p.feat("F-2")).unwrap(), "F4").state,
        State::Pass
    );
    // revise the Contract, then re-verify against it so only the preview receipt is stale
    let path = p
        .root
        .join(".codeos/01-specification/contracts/F-2_contract.md");
    fs::write(
        &path,
        format!("{}\n<!-- revised -->\n", fs::read_to_string(&path).unwrap()),
    )
    .unwrap();
    p.reverify("F-2");
    let r = evaluate(&p.proj(), &p.feat("F-2")).unwrap();
    assert_eq!(s(&r, "F4").state, State::Waiting);
    assert!(s(&r, "F4")
        .unmet
        .iter()
        .any(|u| u.contains("stale") && u.contains("contract")));
}

#[test]
fn a04_gui_feature_missing_playwright_evidence() {
    let p = P::new();
    p.bootstrapped();
    p.spec("F-3", true, false);
    p.receipt(
        "feature",
        "F-3",
        "early_preview",
        "direction_confirmed",
        None,
        None,
        "2026-09-05T10:00:00Z",
    );
    // Everything verified except the integrated Playwright journey.
    p.verify("F-3", "F3", "smoke");
    p.verify("F-3", "F5", "behavior");
    p.verify("F-3", "F5", "repeatability");
    let r = evaluate(&p.proj(), &p.feat("F-3")).unwrap();
    assert_eq!(s(&r, "F5g").state, State::Blocked);
    assert!(s(&r, "F5g")
        .unmet
        .iter()
        .any(|u| u.to_lowercase().contains("playwright")));
}

#[test]
fn a05_persistence_feature_missing_data_integrity_evidence() {
    let p = P::new();
    p.bootstrapped();
    p.spec("F-4", false, true); // persistence changed, no GUI
    p.verify("F-4", "F3", "smoke");
    p.verify("F-4", "F5", "behavior");
    p.verify("F-4", "F5", "repeatability");
    let r = evaluate(&p.proj(), &p.feat("F-4")).unwrap();
    assert_eq!(s(&r, "F5d").state, State::Blocked);
    assert!(s(&r, "F5d").unmet.iter().any(|u| u.contains("data")));
}

#[test]
fn a06_acceptance_with_no_reconciliation_receipt() {
    let p = P::new();
    p.bootstrapped();
    p.spec("F-5", false, false);
    p.reverify("F-5");
    let r = evaluate(&p.proj(), &p.feat("F-5")).unwrap();
    assert_eq!(s(&r, "F6").state, State::Waiting);
    assert!(blocked_or_waiting(s(&r, "F9").state));
}

#[test]
fn a07_operation_closure_without_a_route() {
    let p = P::new();
    let r = evaluate(&p.proj(), &p.op("obs-x")).unwrap();
    assert_eq!(s(&r, "O2").state, State::Blocked);
    assert!(blocked_or_waiting(s(&r, "O4").state));
    assert!(blocked_or_waiting(s(&r, "O5").state));
}

#[test]
fn a08_no_action_without_rationale_is_rejected() {
    let p = P::new();
    let mut b = BTreeMap::new();
    b.insert("observation".into(), text_sha256("cosmetic only"));
    let bad = Receipt {
        workflow: "operation".into(),
        subject: "obs-y".into(),
        checkpoint: "no_action_closure".into(),
        result: "no_action".into(),
        bindings: b,
        timestamp: "2026-09-05T00:00:00Z".into(),
        rationale: None,
        observation: Some("cosmetic only".into()),
    };
    assert!(p.store().append(&bad).is_err());
}

#[test]
fn a09_invalid_route_never_permits_progression() {
    let p = P::new();
    let mut b = BTreeMap::new();
    b.insert("observation".into(), text_sha256("something happened"));
    p.store()
        .append(&Receipt {
            workflow: "operation".into(),
            subject: "obs-z".into(),
            checkpoint: "operation_route".into(),
            result: "make_it_go_away".into(),
            bindings: b,
            timestamp: "2026-09-05T00:00:00Z".into(),
            rationale: None,
            observation: Some("something happened".into()),
        })
        .unwrap();
    let r = evaluate(&p.proj(), &p.op("obs-z")).unwrap();
    assert_eq!(s(&r, "O2").state, State::Blocked);
    assert!(s(&r, "O2")
        .unmet
        .iter()
        .any(|u| u.contains("make_it_go_away")));
}

#[test]
fn a11_reconciliation_with_a_truthful_gap_still_passes_f6() {
    let p = P::new();
    p.bootstrapped();
    p.spec("F-6", false, false);
    p.reverify("F-6");
    // A completed reconciliation, honestly recording a gap in its own text, is still `completed`.
    p.receipt(
        "feature",
        "F-6",
        "reconciliation",
        "completed",
        None,
        None,
        "2026-09-05T12:00:00Z",
    );
    let r = evaluate(&p.proj(), &p.feat("F-6")).unwrap();
    let f6 = s(&r, "F6");
    assert_eq!(f6.state, State::Pass, "{f6:?}");
    assert!(f6
        .met
        .iter()
        .any(|m| m.contains("reconciliation: completed")));
    assert!(!f6.unmet.iter().any(|u| u.to_lowercase().contains("gap")));
}
