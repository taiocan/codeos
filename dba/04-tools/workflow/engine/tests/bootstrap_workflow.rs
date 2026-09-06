//! Bootstrap B1–B5 and the bootstrap-entry gate: a solution's first feature is mechanically
//! blocked until Bootstrap completes.

use codeos_workflow::checker::{
    bindings_for_decision, evaluate, verification_bindings_for, State, Subject,
};
use codeos_workflow::contract::Workflow;
use codeos_workflow::project::Project;
use codeos_workflow::receipts::{Receipt, ReceiptStore};
use codeos_workflow::verification::{VerificationRecord, VerificationStore};
use std::fs;

struct Fx {
    _d: tempfile::TempDir,
    root: std::path::PathBuf,
}

impl Fx {
    fn new() -> Self {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().to_path_buf();
        for p in [
            ".codeos/00-project",
            ".codeos/01-specification/intents",
            ".codeos/01-specification/contracts",
            ".codeos/01-specification/event-schemas",
            ".codeos/06-workflow",
        ] {
            fs::create_dir_all(root.join(p)).unwrap();
        }
        Fx { _d: d, root }
    }
    fn w(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, c).unwrap();
    }
    fn project(&self) -> Project {
        Project::discover(&self.root).unwrap()
    }
    fn boot(&self) -> Subject {
        Subject::resolve(&self.project(), Workflow::Bootstrap, "solution").unwrap()
    }
    /// Seed a passing B3 baseline verification record, bound to the current codeos.yaml.
    fn verify_b3(&self) {
        let bindings = verification_bindings_for(&self.project(), &self.boot(), "B3").unwrap();
        VerificationStore::at(&self.root.join(".codeos"))
            .append(&VerificationRecord {
                workflow: "bootstrap".into(),
                subject: "solution".into(),
                checkpoint: "B3".into(),
                verification: "baseline".into(),
                result: "passed".into(),
                bindings,
                command: "seeded".into(),
                timestamp: "2026-09-06T00:00:00Z".into(),
            })
            .unwrap();
    }
    fn record_ipp(&self) {
        let bindings =
            bindings_for_decision(&self.project(), &self.boot(), "initial_product_preview")
                .unwrap();
        ReceiptStore::at(&self.root.join(".codeos"))
            .append(&Receipt {
                workflow: "bootstrap".into(),
                subject: "solution".into(),
                checkpoint: "initial_product_preview".into(),
                result: "direction_confirmed".into(),
                bindings,
                timestamp: "2026-09-06T00:00:00Z".into(),
                rationale: None,
                observation: None,
            })
            .unwrap();
    }
}

fn charter_approved() -> &'static str {
    "---\nartifact_type: charter\napproval:\n  by: Primoz Gorjup\n  at: 2026-09-05\n---\n\n# Solution Charter\n"
}

fn codeos_yaml() -> &'static str {
    "platform:\n  persistence: postgresql\n  backend: rust\n  webapp: svelte\n  runtime: docker\n\nartifacts:\n  charter: governed\n  intent: governed\n  contract: governed\n  event_schema: governed\n"
}

fn st<'a>(
    r: &'a [codeos_workflow::checker::CheckpointReport],
    id: &str,
) -> &'a codeos_workflow::checker::CheckpointReport {
    r.iter()
        .find(|x| x.id == id)
        .unwrap_or_else(|| panic!("no {id}"))
}

#[test]
fn bootstrap_blocks_at_b3_then_b4_then_completes() {
    let fx = Fx::new();
    fx.w(".codeos/00-project/charter.md", charter_approved());
    fx.w(".codeos/00-project/codeos.yaml", codeos_yaml());

    // B1/B2 pass; B3 has no verification record yet.
    let r = evaluate(&fx.project(), &fx.boot()).unwrap();
    assert_eq!(st(&r, "B1").state, State::Pass);
    assert_eq!(st(&r, "B2").state, State::Pass);
    assert_eq!(st(&r, "B3").state, State::Blocked);
    assert!(st(&r, "B3")
        .unmet
        .iter()
        .any(|u| u.contains("no verification record")));

    // B3 verified: B4 is WAITING for the Initial Product Preview.
    fx.verify_b3();
    let r = evaluate(&fx.project(), &fx.boot()).unwrap();
    assert_eq!(st(&r, "B3").state, State::Pass);
    assert_eq!(st(&r, "B4").state, State::Waiting);
    assert_eq!(st(&r, "B5").state, State::Blocked);

    // Record the Initial Product Preview -> B4/B5 pass.
    fx.record_ipp();
    let r = evaluate(&fx.project(), &fx.boot()).unwrap();
    assert_eq!(st(&r, "B4").state, State::Pass);
    assert_eq!(st(&r, "B5").state, State::Pass);
}

#[test]
fn first_feature_is_blocked_until_bootstrap_completes() {
    let fx = Fx::new();
    // Charter present but NOT approved -> Bootstrap B1 fails.
    fx.w(
        ".codeos/00-project/charter.md",
        "---\nartifact_type: charter\napproval: null\n---\n# Charter\n",
    );
    fx.w(".codeos/00-project/codeos.yaml", codeos_yaml());

    let approved = |kind: &str| {
        format!(
        "---\nartifact_type: {kind}\nfeature_id: F-0001\nstatus: APPROVED\napproved_by: Primoz Gorjup\napproved_at: 2026-09-05\n---\n\n# {kind}\n\n## Validation Questions\n\n**Behavior** — proven by a test.\n**Repeatability** — re-run twice.\n**Browser** — not applicable.\n**Preview** — inspect the page.\n\n## Feature Impact Accounting\n\n| Tier | Changed / Unchanged | Reason |\n|---|---|---|\n| Persistence (PostgreSQL) | unchanged | n/a |\n| Backend (Rust) | changed | endpoint |\n| GUI (Svelte) | unchanged | n/a |\n"
    )
    };
    fx.w(
        ".codeos/01-specification/intents/F-0001.md",
        &approved("intent"),
    );
    fx.w(
        ".codeos/01-specification/contracts/F-0001_contract.md",
        &approved("contract"),
    );
    fx.w(
        ".codeos/01-specification/event-schemas/F-0001_schema.md",
        &approved("event_schema"),
    );

    let subj = Subject::resolve(&fx.project(), Workflow::Feature, "F-0001").unwrap();
    let r = evaluate(&fx.project(), &subj).unwrap();
    let gate = st(&r, "bootstrap-entry");
    assert_eq!(gate.state, State::Blocked);
    assert!(gate
        .unmet
        .iter()
        .any(|u| u.contains("Solution Bootstrap is not complete")));
    assert_eq!(st(&r, "F1").state, State::Blocked);

    // Approve the Charter, verify B3, record the Initial Product Preview -> gate opens.
    fx.w(".codeos/00-project/charter.md", charter_approved());
    fx.verify_b3();
    fx.record_ipp();

    let r = evaluate(&fx.project(), &subj).unwrap();
    assert_eq!(st(&r, "bootstrap-entry").state, State::Pass);
    assert_eq!(st(&r, "F1").state, State::Pass, "{:?}", st(&r, "F1"));
}

/// Regression: implementing the first feature mutates the working tree, but B3's verification
/// record is bound to codeos.yaml only and B4's receipt to the Charter and codeos.yaml — so
/// Bootstrap stays complete and the bootstrap-entry gate stays open. (Binding either to
/// implementation state would deadlock the very first feature: F1–F9 can never pass, so no
/// acceptance receipt can ever satisfy the gate's bypass.)
#[test]
fn implementing_the_first_feature_does_not_reblock_bootstrap() {
    let fx = Fx::new();
    fx.w(".codeos/00-project/charter.md", charter_approved());
    fx.w(".codeos/00-project/codeos.yaml", codeos_yaml());
    fx.w("backend/src/lib.rs", "// skeleton\n");
    fx.verify_b3();
    fx.record_ipp();

    let b3_bindings = verification_bindings_for(&fx.project(), &fx.boot(), "B3").unwrap();
    assert!(
        !b3_bindings.contains_key("implementation_state"),
        "B3 must not bind to working-tree state: {b3_bindings:?}"
    );

    let r = evaluate(&fx.project(), &fx.boot()).unwrap();
    for id in ["B1", "B2", "B3", "B4", "B5"] {
        assert_eq!(st(&r, id).state, State::Pass, "{id}: {:?}", st(&r, id));
    }

    // Implement the first feature: the working tree changes under every implementation path.
    fx.w("backend/src/notes.rs", "pub fn create() {}\n");
    fx.w("web/src/routes/+page.svelte", "<h1>notes</h1>\n");
    fx.w("migrations/0002_notes.sql", "create table notes ();\n");

    let r = evaluate(&fx.project(), &fx.boot()).unwrap();
    for id in ["B1", "B2", "B3", "B4", "B5"] {
        assert_eq!(
            st(&r, id).state,
            State::Pass,
            "after impl {id}: {:?}",
            st(&r, id)
        );
    }
    let subj = Subject::resolve(&fx.project(), Workflow::Feature, "F-0001").unwrap();
    let r = evaluate(&fx.project(), &subj).unwrap();
    assert_eq!(st(&r, "bootstrap-entry").state, State::Pass);
}
