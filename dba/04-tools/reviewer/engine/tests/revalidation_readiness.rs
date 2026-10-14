//! Cross-boundary conformance for Feature Workflow v8 post-acceptance revalidation of a
//! reality-dependent feature: accepted feature → implementation change → fresh production evidence
//! → `prepare-candidate --revalidation` (workflow) → Stage 8 readiness (reviewer) PASS, with no new
//! F6 receipt. The workflow and the reviewer each encoded the F6 selection; this test binds them.

mod common;

use codeos_workflow::checker::{verification_bindings_for_mechanic, Subject};
use codeos_workflow::contract::Workflow;
use codeos_workflow::project::Project;
use codeos_workflow::receipts::{Receipt, ReceiptStore};
use codeos_workflow::verification::{VerificationRecord, VerificationStore};
use common::{repo_root, run_with_fake_codex, setup_fake_codex};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

const FEATURE: &str = "F-0001";

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn git(root: &Path, args: &[&str], date: &str) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn commit(root: &Path, message: &str, date: &str) -> String {
    git(root, &["add", "-A"], date);
    git(root, &["commit", "-q", "-m", message], date);
    git(root, &["rev-parse", "HEAD"], date)
}

fn spec(kind: &str, body: &str) -> String {
    format!(
        "---\nartifact_type: {kind}\nfeature_id: {FEATURE}\nsemantic_revision: 1\nsemantic_change: initial feature specification\nstatus: APPROVED\napproved_by: Human\napproved_at: 2026-10-01\n---\n\n# {kind}\n{body}"
    )
}

/// One production-source evidence file bound to the current implementation, and its record.
fn production_evidence(project: &Project, subject: &Subject, timestamp: &str) -> String {
    let implementation = codeos_workflow::hashing::working_tree_state(
        &project.root,
        &project.implementation_paths(),
    )
    .unwrap();
    let bytes = serde_json::to_vec(
        &serde_json::json!({ "implementation_sha256": implementation, "observations": [] }),
    )
    .unwrap();
    let id = sha256(&bytes);
    write(
        &project.root,
        &format!(".codeos/06-workflow/reality-evidence/{id}.json"),
        std::str::from_utf8(&bytes).unwrap(),
    );
    record(
        project,
        subject,
        "F5",
        "production_source_evidence",
        timestamp,
        serde_json::json!({ "evidence_sha256": id }),
    );
    id
}

fn record(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    verification: &str,
    timestamp: &str,
    details: serde_json::Value,
) {
    VerificationStore::at(&project.codeos_dir())
        .append(&VerificationRecord {
            workflow: "feature".into(),
            subject: FEATURE.into(),
            checkpoint: checkpoint.into(),
            verification: verification.into(),
            result: "passed".into(),
            bindings: verification_bindings_for_mechanic(
                project,
                subject,
                checkpoint,
                verification,
            )
            .unwrap(),
            command: "fixture".into(),
            timestamp: timestamp.into(),
            details: (!details.is_null()).then_some(details),
        })
        .unwrap();
}

/// Every verification revalidation requires, current for the working tree. Test-based records bind
/// inspectable per-test evidence, as `check` records it.
fn verify_all(project: &Project, timestamp: &str) -> String {
    let subject = Subject::resolve(project, Workflow::Feature, FEATURE).unwrap();
    let mut production = String::new();
    for (checkpoint, verification) in
        codeos_workflow::revalidation::required_verifications(project, &subject).unwrap()
    {
        if verification == "production_source_evidence" {
            production = production_evidence(project, &subject, timestamp);
            continue;
        }
        let details = if codeos_workflow::test_evidence::VERIFICATIONS.contains(&verification) {
            let stored = format!("{{\"schema\":1,\"verification\":\"{verification}\"}}");
            let id = sha256(stored.as_bytes());
            write(
                &project.root,
                &format!("{}/{id}.json", codeos_workflow::test_evidence::DIR),
                &stored,
            );
            serde_json::json!({ codeos_workflow::test_evidence::DETAILS_KEY: id })
        } else {
            serde_json::Value::Null
        };
        record(
            project,
            &subject,
            checkpoint,
            verification,
            timestamp,
            details,
        );
    }
    record(
        project,
        &subject,
        "candidate",
        "project_qualification",
        timestamp,
        serde_json::Value::Null,
    );
    production
}

fn receipt(project: &Project, checkpoint: &str, result: &str, bindings: BTreeMap<String, String>) {
    ReceiptStore::at(&project.codeos_dir())
        .append(&Receipt {
            workflow: "feature".into(),
            subject: FEATURE.into(),
            checkpoint: checkpoint.into(),
            result: result.into(),
            bindings,
            timestamp: "2026-10-02T12:00:00Z".into(),
            rationale: None,
            observation: None,
        })
        .unwrap();
}

fn reconciliations(root: &Path) -> usize {
    std::fs::read_to_string(root.join(".codeos/06-workflow/decisions.jsonl"))
        .unwrap()
        .lines()
        .filter(|line| line.contains("\"reconciliation\""))
        .count()
}

#[test]
fn reality_dependent_revalidation_candidate_is_ready_without_a_new_f6_receipt() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"], "2026-10-01T10:00:00Z");
    std::fs::create_dir_all(root.join(".codeos")).unwrap();
    std::os::unix::fs::symlink(repo_root(), root.join(".codeos/toolkit")).unwrap();
    write(
        root,
        ".git/info/exclude",
        "/.codeos/toolkit\n/.codeos/05-review/\n",
    );
    write(
        root,
        ".codeos/00-project/codeos.yaml",
        "codeos_schema: 2\ndba_configuration: DBA-26\n",
    );
    write(
        root,
        ".codeos/00-project/charter.md",
        "---\nartifact_type: charter\napproval:\n  by: Human\n  at: 2026-10-01\n---\n\n## Primary Product Journey\njourney_id: primary\nlegitimate_entry: product home\ninitial_frontier: entry\ntarget_outcome: useful result\nverification_route: product_journey\n",
    );
    write(
        root,
        &format!(".codeos/01-specification/intents/{FEATURE}.md"),
        &spec("intent", ""),
    );
    write(
        root,
        &format!(".codeos/01-specification/contracts/{FEATURE}_contract.md"),
        &spec(
            "contract",
            "\n## Feature Impact Accounting\n\n| Tier | Changed / Unchanged | Reason |\n|---|---|---|\n| Backend (Rust) | changed | forecast |\n\n## Reality Basis\n\n| Claim ID | Claim | Classification | Source | Conditions | Currency | Unavailable | Fixture |\n|---|---|---|---|---|---|---|---|\n| route | real route | reality-dependent | provider | supported | observed | unavailable | test-only |\n\n## Product Journey Contribution\nmode: advance\nprevious_frontier: entry\nproposed_frontier: decided\ntransition: the driver compares departure times\n",
        ),
    );
    write(
        root,
        &format!(".codeos/01-specification/event-schemas/{FEATURE}_schema.md"),
        &spec("event_schema", "\n**Mode:** `external-observation`\n"),
    );
    write(root, "src/lib.rs", "pub fn forecast() {}\n");
    commit(
        root,
        "approved package and implementation",
        "2026-10-01T10:00:00Z",
    );

    // Accepted: verified, reconciled with selected production evidence, accepted.
    let project = Project::discover(root).unwrap();
    let accepted_evidence = verify_all(&project, "2026-10-01T12:00:00Z");
    let f6: BTreeMap<String, String> = [
        (
            "implementation_state",
            codeos_workflow::hashing::working_tree_state(root, &project.implementation_paths())
                .unwrap(),
        ),
        (
            "codeos_yaml",
            sha256(&std::fs::read(project.codeos_yaml()).unwrap()),
        ),
        ("production_evidence", accepted_evidence.clone()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    receipt(&project, "reconciliation", "completed", f6);
    // A legacy acceptance, as JamFree F-0002 and F-0003 have: bound to the Specification Package.
    let subject = Subject::resolve(&project, Workflow::Feature, FEATURE).unwrap();
    let acceptance = [
        ("intent", project.intent(FEATURE)),
        ("contract", project.contract(FEATURE)),
        ("event_schema", project.event_schema(FEATURE)),
    ]
    .into_iter()
    .map(|(k, path)| (k.to_string(), sha256(&std::fs::read(path).unwrap())))
    .collect();
    receipt(&project, "acceptance", "accepted", acceptance);
    commit(root, "accept", "2026-10-02T12:00:00Z");

    // A dependency-style implementation change, then fresh evidence for it.
    write(
        root,
        "src/lib.rs",
        "pub fn forecast() { /* patched transport */ }\n",
    );
    let trigger = commit(root, "patch transport", "2026-10-03T10:00:00Z");
    let fresh_evidence = verify_all(&project, "2026-10-03T12:00:00Z");
    assert_ne!(fresh_evidence, accepted_evidence);

    let (id, bundle) = codeos_workflow::candidate::prepare(
        &project,
        &subject,
        &[],
        "post_acceptance_revalidation",
        Some(&trigger),
    )
    .unwrap();
    assert_eq!(
        bundle.production_evidence.as_deref(),
        Some(fresh_evidence.as_str())
    );
    commit(
        root,
        "revalidation evidence and candidate",
        "2026-10-03T13:00:00Z",
    );

    let fake = setup_fake_codex();
    let intent = format!(".codeos/01-specification/intents/{FEATURE}.md");
    let (_, stdout, stderr) =
        run_with_fake_codex(root, &fake, &["plan", FEATURE, "8", &intent], "success");
    assert!(
        stdout.contains(&id) && stdout.contains("FULL_COVERAGE"),
        "{stdout}\n{stderr}"
    );
    assert!(
        stdout.contains("review_readiness: PASS"),
        "{stdout}\n{stderr}"
    );
    assert!(stdout.contains(&format!("reality-evidence/{fresh_evidence}.json")));
    assert_eq!(
        reconciliations(root),
        1,
        "revalidation creates no F6 receipt"
    );
}
