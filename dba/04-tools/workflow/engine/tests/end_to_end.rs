//! One reproducible pass of the full DBA-6 flow against a fixture: Solution Bootstrap → Feature
//! Development F1–F9 → Operation & Learning O1–O5, each reaching all-PASS. This is the pilot
//! captured as a test — the real end-to-end pilot runs the same sequence against a live Docker
//! stack, real PostgreSQL, and a real Playwright journey.

use codeos_workflow::checker::{
    bindings_for_decision, evaluate, verification_bindings_for, State, Subject,
};
use codeos_workflow::contract::Workflow;
use codeos_workflow::hashing::text_sha256;
use codeos_workflow::project::Project;
use codeos_workflow::receipts::{Receipt, ReceiptStore};
use codeos_workflow::verification::{VerificationRecord, VerificationStore};
use std::fs;

struct Pilot {
    _d: tempfile::TempDir,
    root: std::path::PathBuf,
}

impl Pilot {
    fn new() -> Self {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().to_path_buf();
        for p in [
            ".codeos/00-project",
            ".codeos/01-specification/intents",
            ".codeos/01-specification/contracts",
            ".codeos/01-specification/event-schemas",
            ".codeos/05-review/reviews",
            ".codeos/06-workflow",
            "backend/src",
            "web/src",
        ] {
            fs::create_dir_all(root.join(p)).unwrap();
        }
        Pilot { _d: d, root }
    }
    fn w(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, c).unwrap();
    }
    fn project(&self) -> Project {
        Project::discover(&self.root).unwrap()
    }
    fn subject(&self, wf: Workflow, id: &str) -> Subject {
        Subject::resolve(&self.project(), wf, id).unwrap()
    }
    fn verify(&self, wf: Workflow, subj: &str, checkpoint: &str, mechanic: &str) {
        let bindings =
            verification_bindings_for(&self.project(), &self.subject(wf, subj), checkpoint)
                .unwrap();
        VerificationStore::at(&self.root.join(".codeos"))
            .append(&VerificationRecord {
                workflow: wf.slug().into(),
                subject: subj.into(),
                checkpoint: checkpoint.into(),
                verification: mechanic.into(),
                result: "passed".into(),
                bindings,
                command: "pilot".into(),
                timestamp: "2026-09-06T10:00:00Z".into(),
            })
            .unwrap();
    }
    fn decide(&self, wf: Workflow, subj: &str, cp: &str, result: &str) {
        self.decide_full(wf, subj, cp, result, None, None);
    }
    fn decide_full(
        &self,
        wf: Workflow,
        subj: &str,
        cp: &str,
        result: &str,
        observation: Option<&str>,
        rationale: Option<&str>,
    ) {
        let sub = self.subject(wf, subj);
        let mut bindings = bindings_for_decision(&self.project(), &sub, cp).unwrap_or_default();
        if let Some(o) = observation {
            bindings.insert("observation".into(), text_sha256(o));
        }
        ReceiptStore::at(&self.root.join(".codeos"))
            .append(&Receipt {
                workflow: wf.slug().into(),
                subject: subj.into(),
                checkpoint: cp.into(),
                result: result.into(),
                bindings,
                timestamp: "2026-09-06T10:00:00Z".into(),
                rationale: rationale.map(String::from),
                observation: observation.map(String::from),
            })
            .unwrap();
    }
    fn all_pass(
        &self,
        wf: Workflow,
        subj: &str,
    ) -> Vec<codeos_workflow::checker::CheckpointReport> {
        let r = evaluate(&self.project(), &self.subject(wf, subj)).unwrap();
        for c in &r {
            assert!(
                matches!(c.state, State::Pass | State::NotApplicable),
                "{}/{} checkpoint {} is {:?}: {:?}",
                wf.slug(),
                subj,
                c.id,
                c.state,
                c.unmet
            );
        }
        r
    }
}

const CODEOS_YAML: &str = "platform:\n  persistence: postgresql\n  backend: rust\n  webapp: svelte\n  runtime: docker\nartifacts:\n  charter: governed\n  intent: governed\n  contract: governed\n  event_schema: governed\n";

fn approved(kind: &str, gui_row: &str, persist_row: &str) -> String {
    format!(
        "---\nartifact_type: {kind}\nfeature_id: F-0001\nstatus: APPROVED\napproved_by: Primoz Gorjup\napproved_at: 2026-09-06\n---\n\n# {kind}: F-0001\n\nThis feature has a GUI-visible outcome.\n\n## Validation Questions\n\n**Behavior** — an integration test proves it.\n**Repeatability** — the same test re-run twice both pass.\n**Browser** — a Playwright journey proves the visible outcome.\n**Preview** — the human inspects the running page.\n\n## Feature Impact Accounting\n\n| Tier | Changed / Unchanged | Reason |\n|---|---|---|\n| Persistence (PostgreSQL) | {persist_row} | notes table |\n| Backend (Rust) | changed | new endpoint |\n| GUI (Svelte) | {gui_row} | new form |\n"
    )
}

#[test]
fn bootstrap_then_feature_then_operation_all_reach_all_pass() {
    let p = Pilot::new();

    // --- Solution Bootstrap ------------------------------------------------
    p.w(
        ".codeos/00-project/charter.md",
        "---\nartifact_type: charter\napproval:\n  by: Primoz Gorjup\n  at: 2026-09-06\n---\n\n# Solution Charter\n",
    );
    p.w(".codeos/00-project/codeos.yaml", CODEOS_YAML);
    p.w("backend/src/lib.rs", "// skeleton\n");
    p.w("docker-compose.yml", "services: {}\n");

    p.verify(Workflow::Bootstrap, "solution", "B3", "baseline");
    p.decide(
        Workflow::Bootstrap,
        "solution",
        "initial_product_preview",
        "direction_confirmed",
    );
    p.all_pass(Workflow::Bootstrap, "solution");

    // --- Feature Development F1–F9 ---------------------------------------
    p.w(
        ".codeos/01-specification/intents/F-0001.md",
        &approved("intent", "changed", "changed"),
    );
    p.w(
        ".codeos/01-specification/contracts/F-0001_contract.md",
        &approved("contract", "changed", "changed"),
    );
    p.w(
        ".codeos/01-specification/event-schemas/F-0001_schema.md",
        &approved("event_schema", "changed", "changed"),
    );
    p.w("backend/src/notes.rs", "pub fn create() {}\n");
    p.w("web/src/routes/+page.svelte", "<h1>notes</h1>\n");

    // F3 smoke, then the Early Development Preview, then full verification.
    p.verify(Workflow::Feature, "F-0001", "F3", "smoke");
    p.decide(
        Workflow::Feature,
        "F-0001",
        "early_preview",
        "direction_confirmed",
    );
    p.verify(Workflow::Feature, "F-0001", "F5", "behavior");
    p.verify(Workflow::Feature, "F-0001", "F5", "repeatability");
    p.verify(Workflow::Feature, "F-0001", "F5d", "data_integrity");
    p.verify(Workflow::Feature, "F-0001", "F5g", "playwright");

    // F6 reconciliation (completed, gaps notwithstanding), F7 reviewer record, F8/F9 receipts.
    p.decide_full(
        Workflow::Feature,
        "F-0001",
        "reconciliation",
        "completed",
        None,
        None,
    );
    p.w(
        ".codeos/05-review/reviews/2026-09-06T100000Z-F-0001-stage-8-0000000.md",
        "---\nartifact_type: review_record\nreviewed:\n  feature: F-0001\n  stage: \"8\"\n  review_commit: \"\"\n  reviewed_packet_sha256: \"\"\n  diff_hash: \"\"\neffective_concern: NO OBJECTION\n---\n\n# Independent Review — F-0001, Stage 8\n\nNO OBJECTION.\n",
    );
    p.decide(
        Workflow::Feature,
        "F-0001",
        "final_ux_validation",
        "accepted",
    );
    p.decide(Workflow::Feature, "F-0001", "acceptance", "accepted");
    p.all_pass(Workflow::Feature, "F-0001");

    // --- Operation & Learning O1–O5 -----------------------------------
    let obs = "Footer shows a hard-coded year; cosmetic, not worth a change this cycle.";
    p.decide_full(
        Workflow::Operation,
        "obs-footer-year",
        "operation_route",
        "no_action",
        Some(obs),
        None,
    );
    p.decide_full(
        Workflow::Operation,
        "obs-footer-year",
        "no_action_closure",
        "no_action",
        Some(obs),
        Some("Deliberately not acted on: purely cosmetic; revisit at the next UI pass."),
    );
    p.all_pass(Workflow::Operation, "obs-footer-year");
}
