//! SPERF WO-01: a golden `status --json` report and the command's evaluation counts.
//!
//! The fixture mirrors JamFree's shape: a DBA-26 project with a Cargo backend below the root, a
//! web tree, one current accepted feature, one stale accepted feature and one in progress. The
//! golden report is the semantic identity reference for the evaluation snapshot (SPERF WO-02).
//! Regenerate it only for an intended report change: `UPDATE_GOLDEN=1 cargo test --test
//! status_snapshot`.

use codeos_workflow::checker::{
    bindings_for_decision, verification_bindings_for_mechanic, Subject,
};
use codeos_workflow::contract::Workflow;
use codeos_workflow::project::Project;
use codeos_workflow::receipts::{Receipt, ReceiptStore};
use codeos_workflow::verification::{VerificationRecord, VerificationStore};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// JamFree's DBA-26 project configuration, without its reality verification command.
const CODEOS_YAML: &str = "codeos_schema: 2\ndba_configuration: DBA-26\nplatform:\n  persistence: postgresql\n  backend: rust\n  webapp: svelte\n  runtime: docker\nmechanics:\n  delivery:\n    vertical_slice: always\n    early_gui_preview: always_when_gui\n  validation:\n    smoke: always\n    product_journey: always\n    behavior: always\n    repeatability: always\n    data_integrity: always_when_persistence\n    playwright: always_when_gui\n    human_ux: always_when_gui\n    production_source_evidence: when_reality_dependent\n    project_qualification: always\n  communication:\n    summary_first: always\n    reader_oriented_writing: always\n    terminology_consistency: always\nartifacts:\n  charter: governed\n  intent: governed\n  contract: governed\n  event_schema: governed\n  architecture: governed\n  module_design_note: nongoverned\n  learning_register: nongoverned\n  review_file: nongoverned\n  feature_decomposition: nongoverned\n  project_terminology: nongoverned\n  user_workflow_map: nongoverned\n  refinement: nongoverned\n";

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    fn write(&self, rel: &str, content: &str) {
        let path = self.root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn project(&self) -> Project {
        Project::discover(&self.root).unwrap()
    }

    fn subject(&self, id: &str) -> Subject {
        let workflow = if id == "solution" {
            Workflow::Bootstrap
        } else {
            Workflow::Feature
        };
        Subject::resolve(&self.project(), workflow, id).unwrap()
    }

    fn receipt(&self, feature: &str, checkpoint: &str, result: &str) {
        let subject = self.subject(feature);
        let bindings = bindings_for_decision(&self.project(), &subject, checkpoint).unwrap();
        ReceiptStore::at(&self.root.join(".codeos"))
            .append(&Receipt {
                workflow: subject.workflow.slug().into(),
                subject: feature.into(),
                checkpoint: checkpoint.into(),
                result: result.into(),
                bindings,
                timestamp: "2026-10-16T00:00:00Z".into(),
                rationale: None,
                observation: None,
            })
            .unwrap();
    }

    /// Walk one feature through F1–F9 with seeded passing records, as `check`, `decide` and
    /// `prepare-candidate` would leave them.
    fn accept(&self, feature: &str) {
        let subject = self.subject(feature);
        for (checkpoint, mechanic) in
            codeos_workflow::checker::required_verification_records(&self.project(), &subject)
                .unwrap()
        {
            if checkpoint == "F3" {
                self.verify(feature, checkpoint, mechanic);
            }
        }
        self.receipt(feature, "early_preview", "direction_confirmed");
        for (checkpoint, mechanic) in
            codeos_workflow::checker::required_verification_records(&self.project(), &subject)
                .unwrap()
        {
            if checkpoint != "F3" {
                self.verify(feature, checkpoint, mechanic);
            }
        }
        self.receipt(feature, "reconciliation", "completed");
        self.verify(feature, "candidate", "project_qualification");
        codeos_workflow::candidate::prepare(&self.project(), &subject, &[], "ordinary", None)
            .unwrap();
        self.receipt(feature, "final_ux_validation", "accepted");
        self.receipt(feature, "acceptance", "accepted");
    }

    fn verify(&self, feature: &str, checkpoint: &str, mechanic: &str) {
        let subject = self.subject(feature);
        let bindings =
            verification_bindings_for_mechanic(&self.project(), &subject, checkpoint, mechanic)
                .unwrap();
        VerificationStore::at(&self.root.join(".codeos"))
            .append(&VerificationRecord {
                workflow: subject.workflow.slug().into(),
                subject: feature.into(),
                checkpoint: checkpoint.into(),
                verification: mechanic.into(),
                result: "passed".into(),
                bindings,
                command: "seeded".into(),
                timestamp: "2026-10-16T00:00:00Z".into(),
                details: (mechanic == "product_journey").then(
                    || serde_json::json!({ "observed_surface": format!("{feature}-surface") }),
                ),
            })
            .unwrap();
    }

    fn git(&self, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .env("GIT_AUTHOR_DATE", "2026-10-16T00:00:00Z")
            .env("GIT_COMMITTER_DATE", "2026-10-16T00:00:00Z")
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    }
}

fn spec(fx: &Fixture, feature: &str, contribution: &str) {
    let front = |kind: &str| {
        format!("---\nartifact_type: {kind}\nfeature_id: {feature}\nstatus: APPROVED\nsemantic_revision: 1\nsemantic_change: initial feature specification\napproved_by: Primoz Gorjup\napproved_at: 2026-10-16\n---\n\n")
    };
    fx.write(
        &format!(".codeos/01-specification/intents/{feature}.md"),
        &format!(
            "{}# Intent: {feature} — Golden {feature}\n",
            front("intent")
        ),
    );
    fx.write(
        &format!(".codeos/01-specification/contracts/{feature}_contract.md"),
        &format!("{}# Behavioral Contract: {feature}\n\nThis feature has a GUI-visible outcome.\n\n## Validation Questions\n\n**Behavior** — a real integration test proves it.\n**Repeatability** — the same test re-run twice both pass.\n**Browser** — a Playwright journey proves the visible outcome.\n**Preview** — the human inspects the running page.\n\n## Feature Impact Accounting\n\n| Tier | Changed / Unchanged | Reason |\n|---|---|---|\n| Persistence (PostgreSQL) | unchanged | none |\n| Backend (Rust) | changed | new endpoint |\n| GUI (Svelte) | changed | new form |\n\n## Product Journey Contribution\n{contribution}\n\n## Reality Basis\n\n| Claim ID | Claim | Classification | Source | Conditions | Currency | Unavailable | Fixture |\n|---|---|---|---|---|---|---|---|\n| label | Heading | not-reality-dependent | n/a | n/a | n/a | n/a | n/a |\n", front("contract")),
    );
    fx.write(
        &format!(".codeos/01-specification/event-schemas/{feature}_schema.md"),
        &format!(
            "{}# Event Schema: {feature}\n\n## Observation Mode\n\n**Mode:** `external-observation`\n",
            front("event_schema")
        ),
    );
}

fn fixture() -> Fixture {
    // A fixed directory name keeps the reported project name stable.
    let dir = tempfile::tempdir().unwrap();
    let fx = Fixture {
        root: dir.path().canonicalize().unwrap().join("golden"),
        _dir: dir,
    };
    let toolkit = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap();
    fs::create_dir_all(fx.root.join(".codeos")).unwrap();
    std::os::unix::fs::symlink(toolkit, fx.root.join(".codeos/toolkit")).unwrap();
    fx.write(".gitignore", ".codeos/toolkit\nbackend/target\n");
    fx.write(".codeos/00-project/codeos.yaml", CODEOS_YAML);
    fx.write(
        ".codeos/00-project/charter.md",
        "---\nartifact_type: charter\napproval:\n  by: Human\n  at: 2026-10-16\n---\n\n## Primary Product Journey\njourney_id: primary\nlegitimate_entry: product home\ninitial_frontier: entry\ntarget_outcome: useful result\nverification_route: product_journey\n",
    );
    fx.write(
        "backend/Cargo.toml",
        "[package]\nname = \"golden\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    fx.write("backend/src/lib.rs", "pub fn forecast() {}\n");
    fx.write("web/src/routes/+page.svelte", "<h1>Compare</h1>\n");
    fx.write("docker-compose.yml", "services: {}\n");
    fs::create_dir_all(fx.root.join(".codeos/06-workflow")).unwrap();

    // Bootstrap is complete; F-0001 is accepted and stays current.
    spec(
        &fx,
        "F-0001",
        "mode: advance\nprevious_frontier: entry\nproposed_frontier: decided\ntransition: the driver compares departure times",
    );
    fx.git(&["init", "-q"]);
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-q", "-m", "fixture"]);
    fx.verify("solution", "B3", "baseline");
    fx.receipt("solution", "initial_product_preview", "direction_confirmed");
    fx.accept("F-0001");

    // F-0002 is accepted, then its Contract changes: stale.
    spec(
        &fx,
        "F-0002",
        "mode: preserve\nreason: refines the decision",
    );
    fx.accept("F-0002");
    let contract = fx
        .root
        .join(".codeos/01-specification/contracts/F-0002_contract.md");
    let text = fs::read_to_string(&contract).unwrap();
    fs::write(&contract, text.replace("new form", "revised form")).unwrap();

    // F-0003 is in progress with one verification.
    spec(
        &fx,
        "F-0003",
        "mode: preserve\nreason: refines the decision again",
    );
    fx.verify("F-0003", "F3", "smoke");
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-q", "-m", "features"]);
    fx
}

#[test]
fn status_report_matches_the_golden_reference_and_counts_its_work() {
    let fx = fixture();
    let counts_path = fx.root.join("counts.json");
    let output = Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
        .arg("--project")
        .arg(&fx.root)
        .args(["status", "--json"])
        .env(codeos_workflow::instrument::COUNTS_ENV, &counts_path)
        .output()
        .unwrap();
    let report = String::from_utf8(output.stdout)
        .unwrap()
        .replace(fx.root.to_str().unwrap(), "<root>");
    let report: Value = serde_json::from_str(&report)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&output.stderr)));

    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/status.json");
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        fs::create_dir_all(golden.parent().unwrap()).unwrap();
        fs::write(
            &golden,
            serde_json::to_string_pretty(&report).unwrap() + "\n",
        )
        .unwrap();
    }
    let expected: Value = serde_json::from_str(&fs::read_to_string(&golden).unwrap()).unwrap();
    assert_eq!(
        report, expected,
        "status --json differs from tests/golden/status.json"
    );

    let counts: Value = serde_json::from_str(&fs::read_to_string(&counts_path).unwrap()).unwrap();
    eprintln!("{counts:#}");
    let count = |key: &str| counts["counts"][key].as_u64().unwrap_or(0);
    assert_eq!(count("configuration_resolution"), 1);
    assert!(count("cargo_metadata_resolved") <= 1);
    assert_eq!(count("store_parse:verifications"), 1);
    assert_eq!(count("store_parse:decisions"), 1);
    for kind in ["working_tree_state", "product_state"] {
        assert_eq!(
            count(&format!("binding:{kind}")),
            counts["distinct_bindings"][kind].as_u64().unwrap_or(0),
            "{kind} is computed more than once per distinct input"
        );
    }
}

/// `status --json` counts, written outside the project, and the Git status it leaves behind.
fn status_counts(fx: &Fixture) -> (Value, String) {
    let counts = tempfile::NamedTempFile::new().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
        .arg("--project")
        .arg(&fx.root)
        .args(["status", "--json"])
        .env(codeos_workflow::instrument::COUNTS_ENV, counts.path())
        .output()
        .unwrap();
    assert!(!output.stdout.is_empty());
    let tree = Command::new("git")
        .args([
            "status",
            "--porcelain",
            "--untracked-files=all",
            "--ignored",
        ])
        .current_dir(&fx.root)
        .output()
        .unwrap()
        .stdout;
    (
        serde_json::from_str(&fs::read_to_string(counts.path()).unwrap()).unwrap(),
        String::from_utf8(tree).unwrap(),
    )
}

#[test]
fn status_work_does_not_grow_with_checkpoints_and_persists_nothing() {
    let fx = fixture();
    let tree = |fx: &Fixture| {
        String::from_utf8(
            Command::new("git")
                .args([
                    "status",
                    "--porcelain",
                    "--untracked-files=all",
                    "--ignored",
                ])
                .current_dir(&fx.root)
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
    };
    let before = tree(&fx);
    let (small, after) = status_counts(&fx);
    assert_eq!(before, after, "status wrote into the project");

    // Two more in-progress features add checkpoints to evaluate, but no new Git work.
    for feature in ["F-0004", "F-0005"] {
        spec(&fx, feature, "mode: preserve\nreason: another refinement");
        fx.verify(feature, "F3", "smoke");
    }
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-q", "-m", "more features"]);
    let (large, _) = status_counts(&fx);
    for key in ["process:git", "configuration_resolution", "process:cargo"] {
        assert_eq!(
            large["counts"][key], small["counts"][key],
            "{key} grows with checkpoint count"
        );
    }
}
