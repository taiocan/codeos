//! End-to-end checks of the Feature Development contract against fixture evidence — no real
//! project, no network. Drives the library directly.

use codeos_workflow::checker::{evaluate, verification_bindings_for, Subject};
use codeos_workflow::contract::Workflow;
use codeos_workflow::project::Project;
use codeos_workflow::receipts::{Receipt, ReceiptStore};
use codeos_workflow::verification::{VerificationRecord, VerificationStore};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use codeos_workflow::checker::State;

struct Fixture {
    _dir: tempfile::TempDir,
    root: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        for d in [
            ".codeos/00-project",
            ".codeos/01-specification/intents",
            ".codeos/01-specification/contracts",
            ".codeos/01-specification/event-schemas",
            ".codeos/05-review/reviews",
            ".codeos/06-workflow",
        ] {
            fs::create_dir_all(root.join(d)).unwrap();
        }
        let fx = Fixture { _dir: dir, root };
        // These tests exercise Feature Development, not the bootstrap gate. Represent a project
        // where a prior feature was already accepted, so bootstrap-entry is satisfied.
        ReceiptStore::at(&fx.root.join(".codeos"))
            .append(&Receipt {
                workflow: "feature".into(),
                subject: "F-0000".into(),
                checkpoint: "acceptance".into(),
                result: "accepted".into(),
                bindings: BTreeMap::new(),
                timestamp: "2026-01-01T00:00:00Z".into(),
                rationale: None,
                observation: None,
            })
            .unwrap();
        fx
    }

    fn write(&self, rel: &str, content: &str) {
        let p = self.root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }

    fn project(&self) -> Project {
        Project::discover(&self.root).unwrap()
    }

    /// Make the fixture a git repo with one commit, so `working_tree_state` (HEAD + diff +
    /// untracked) actually moves when a tracked implementation file is edited.
    fn git_init_commit(&self) {
        let sh = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&self.root)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@t")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@t")
                .output()
                .unwrap();
        };
        sh(&["init", "-q"]);
        sh(&["add", "-A"]);
        sh(&["commit", "-q", "-m", "fixture"]);
    }

    fn subject(&self, id: &str) -> Subject {
        Subject::resolve(&self.project(), Workflow::Feature, id).unwrap()
    }

    /// Seed a passing mechanical-verification record for one (checkpoint, mechanic), bound to the
    /// current inputs — exactly what `check` would persist on success.
    fn verify(&self, feature: &str, checkpoint: &str, mechanic: &str) {
        let bindings =
            verification_bindings_for(&self.project(), &self.subject(feature), checkpoint).unwrap();
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

    fn state_of<'a>(
        reports: &'a [codeos_workflow::checker::CheckpointReport],
        id: &str,
    ) -> &'a codeos_workflow::checker::CheckpointReport {
        reports
            .iter()
            .find(|r| r.id == id)
            .unwrap_or_else(|| panic!("no checkpoint {id}"))
    }
}

fn approved_spec(feature: &str, gui: bool, persist: bool) -> (String, String, String) {
    let intent = format!(
        "---\nartifact_type: intent\nfeature_id: {feature}\nstatus: APPROVED\napproved_by: Primoz Gorjup\napproved_at: 2026-09-05\n---\n\n# Intent: {feature}\n"
    );
    let gui_row = if gui { "changed" } else { "unchanged" };
    let persist_row = if persist { "changed" } else { "unchanged" };
    let gui_marker = if gui {
        "This feature has a GUI-visible outcome."
    } else {
        ""
    };
    let contract = format!(
        "---\nartifact_type: contract\nfeature_id: {feature}\nstatus: APPROVED\napproved_by: Primoz Gorjup\napproved_at: 2026-09-05\n---\n\n# Behavioral Contract: {feature}\n\n{gui_marker}\n\n## Validation Questions\n\n**Behavior** — a real integration test proves it.\n**Repeatability** — the same test re-run twice both pass.\n**Browser** — a Playwright journey proves the visible outcome.\n**Preview** — the human inspects the running page.\n\n## Feature Impact Accounting\n\n| Tier | Changed / Unchanged | Reason |\n|---|---|---|\n| Persistence (PostgreSQL) | {persist_row} | notes table |\n| Backend (Rust) | changed | new endpoints |\n| GUI (Svelte) | {gui_row} | new form |\n"
    );
    let schema = format!(
        "---\nartifact_type: event_schema\nfeature_id: {feature}\nstatus: APPROVED\napproved_by: Primoz Gorjup\napproved_at: 2026-09-05\n---\n\n# Event Schema: {feature}\n"
    );
    (intent, contract, schema)
}

fn put_receipt(
    root: &Path,
    feature: &str,
    checkpoint: &str,
    result: &str,
    bindings: BTreeMap<String, String>,
) {
    let store = ReceiptStore::at(&root.join(".codeos"));
    store
        .append(&Receipt {
            workflow: "feature".into(),
            subject: feature.into(),
            checkpoint: checkpoint.into(),
            result: result.into(),
            bindings,
            timestamp: "2026-09-05T00:00:00Z".into(),
            rationale: None,
            observation: None,
        })
        .unwrap();
}

/// Compute a checkpoint's receipt bindings exactly as `decide` would, so a recorded receipt is
/// Current.
fn hashes(fx: &Fixture, feature: &str, checkpoint: &str) -> BTreeMap<String, String> {
    codeos_workflow::checker::bindings_for_decision(&fx.project(), &fx.subject(feature), checkpoint)
        .unwrap()
}

#[test]
fn draft_spec_blocks_f1_and_everything_after() {
    let fx = Fixture::new();
    fx.write(
        ".codeos/01-specification/intents/F-0001.md",
        "---\nstatus: DRAFT\napproved_by:\napproved_at:\n---\nx",
    );
    fx.write(
        ".codeos/01-specification/contracts/F-0001_contract.md",
        "---\nstatus: DRAFT\n---\nx",
    );
    fx.write(
        ".codeos/01-specification/event-schemas/F-0001_schema.md",
        "---\nstatus: DRAFT\n---\nx",
    );

    let reports = evaluate(&fx.project(), &fx.subject("F-0001")).unwrap();
    let f1 = Fixture::state_of(&reports, "F1");
    assert_eq!(f1.state, State::Blocked);
    assert!(f1
        .unmet
        .iter()
        .any(|u| u.contains("joint package approval")));
    assert_eq!(Fixture::state_of(&reports, "F5").state, State::Blocked);
}

#[test]
fn approved_spec_passes_f1_f2_then_blocks_on_missing_smoke_record() {
    let fx = Fixture::new();
    let (i, c, s) = approved_spec("F-0001", true, true);
    fx.write(".codeos/01-specification/intents/F-0001.md", &i);
    fx.write(".codeos/01-specification/contracts/F-0001_contract.md", &c);
    fx.write(
        ".codeos/01-specification/event-schemas/F-0001_schema.md",
        &s,
    );

    let reports = evaluate(&fx.project(), &fx.subject("F-0001")).unwrap();
    assert_eq!(Fixture::state_of(&reports, "F1").state, State::Pass);
    assert_eq!(Fixture::state_of(&reports, "F2").state, State::Pass);
    let f3 = Fixture::state_of(&reports, "F3");
    assert_eq!(f3.state, State::Blocked);
    assert!(f3
        .unmet
        .iter()
        .any(|u| u.contains("no verification record") && u.contains("codeos-workflow check")));
}

#[test]
fn f5_stays_blocked_until_early_preview_is_recorded_and_current() {
    let fx = Fixture::new();
    let (i, c, s) = approved_spec("F-0007", true, false);
    fx.write(".codeos/01-specification/intents/F-0007.md", &i);
    fx.write(".codeos/01-specification/contracts/F-0007_contract.md", &c);
    fx.write(
        ".codeos/01-specification/event-schemas/F-0007_schema.md",
        &s,
    );

    // Verification records present: F3 passes, F4 is WAITING for the preview, F5 blocked behind it.
    fx.reverify("F-0007");
    let r = evaluate(&fx.project(), &fx.subject("F-0007")).unwrap();
    assert_eq!(Fixture::state_of(&r, "F3").state, State::Pass, "{r:?}");
    assert_eq!(Fixture::state_of(&r, "F4").state, State::Waiting);
    assert_eq!(Fixture::state_of(&r, "F5").state, State::Blocked);

    // Record direction_confirmed, bound to current hashes.
    put_receipt(
        &fx.root,
        "F-0007",
        "early_preview",
        "direction_confirmed",
        hashes(&fx, "F-0007", "early_preview"),
    );
    let r = evaluate(&fx.project(), &fx.subject("F-0007")).unwrap();
    assert_eq!(Fixture::state_of(&r, "F4").state, State::Pass);
    assert_eq!(Fixture::state_of(&r, "F5").state, State::Pass);

    // Mutate the Contract, then re-run the mechanical verifications against the new contract so the
    // ONLY thing stale is the early-preview receipt.
    let contract_path = ".codeos/01-specification/contracts/F-0007_contract.md";
    fx.write(
        contract_path,
        &format!("{c}\n<!-- edited after preview -->\n"),
    );
    fx.reverify("F-0007");

    let r = evaluate(&fx.project(), &fx.subject("F-0007")).unwrap();
    let f4 = Fixture::state_of(&r, "F4");
    assert_eq!(f4.state, State::Waiting);
    assert!(
        f4.unmet
            .iter()
            .any(|u| u.contains("stale") && u.contains("contract")),
        "{:?}",
        f4.unmet
    );
    // F5, previously PASS, is blocked again behind the stale F4.
    assert_eq!(Fixture::state_of(&r, "F5").state, State::Blocked);
}

#[test]
fn a_stale_implementation_reblocks_full_verification() {
    // A verification record goes stale the moment the implementation it was run against drifts.
    let fx = Fixture::new();
    let (i, c, s) = approved_spec("F-0021", false, true);
    fx.write(".codeos/01-specification/intents/F-0021.md", &i);
    fx.write(".codeos/01-specification/contracts/F-0021_contract.md", &c);
    fx.write(
        ".codeos/01-specification/event-schemas/F-0021_schema.md",
        &s,
    );
    fx.write("backend/src/notes.rs", "// v1\n");
    fx.git_init_commit();
    fx.reverify("F-0021");
    put_receipt(
        &fx.root,
        "F-0021",
        "early_preview",
        "direction_confirmed",
        hashes(&fx, "F-0021", "early_preview"),
    );

    let r = evaluate(&fx.project(), &fx.subject("F-0021")).unwrap();
    assert_eq!(Fixture::state_of(&r, "F5").state, State::Pass);
    assert_eq!(Fixture::state_of(&r, "F5d").state, State::Pass);

    // Change the implementation without re-verifying: the earliest mechanical checkpoint bound to
    // implementation_state (F3) re-blocks, and everything after it blocks behind it.
    fx.write("backend/src/notes.rs", "// v2 — behaviour changed\n");
    let r = evaluate(&fx.project(), &fx.subject("F-0021")).unwrap();
    let f3 = Fixture::state_of(&r, "F3");
    assert_eq!(f3.state, State::Blocked, "{f3:?}");
    assert!(f3
        .unmet
        .iter()
        .any(|u| u.contains("stale") && u.contains("implementation_state")));
    assert_eq!(Fixture::state_of(&r, "F5").state, State::Blocked);
}

#[test]
fn reconciliation_completed_passes_even_though_gaps_may_remain() {
    // Guards the semantic fix: F6 records `completed`, never "no gaps".
    let fx = Fixture::new();
    let (i, c, s) = approved_spec("F-0009", false, true);
    fx.write(".codeos/01-specification/intents/F-0009.md", &i);
    fx.write(".codeos/01-specification/contracts/F-0009_contract.md", &c);
    fx.write(
        ".codeos/01-specification/event-schemas/F-0009_schema.md",
        &s,
    );
    fx.reverify("F-0009");

    put_receipt(
        &fx.root,
        "F-0009",
        "reconciliation",
        "completed",
        hashes(&fx, "F-0009", "reconciliation"),
    );

    let r = evaluate(&fx.project(), &fx.subject("F-0009")).unwrap();
    let f6 = Fixture::state_of(&r, "F6");
    assert_eq!(f6.state, State::Pass, "{f6:?}");
    assert!(f6
        .met
        .iter()
        .any(|m| m.contains("reconciliation: completed")));
    assert!(!f6.unmet.iter().any(|u| u.to_lowercase().contains("gap")));
}

#[test]
fn non_gui_feature_marks_f4_not_applicable() {
    let fx = Fixture::new();
    let (i, c, s) = approved_spec("F-0100", false, false);
    fx.write(".codeos/01-specification/intents/F-0100.md", &i);
    fx.write(".codeos/01-specification/contracts/F-0100_contract.md", &c);
    fx.write(
        ".codeos/01-specification/event-schemas/F-0100_schema.md",
        &s,
    );
    let r = evaluate(&fx.project(), &fx.subject("F-0100")).unwrap();
    assert_eq!(Fixture::state_of(&r, "F4").state, State::NotApplicable);
    assert_eq!(Fixture::state_of(&r, "F5g").state, State::NotApplicable);
}
