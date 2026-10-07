//! Executable conformance proof for Review Policy v15 and Reviewer Tool Contract v17. Every
//! refusal happens before the reviewer is invoked and writes no review record. No real provider
//! is used; the fake reviewer counts invocations.
//!
//! | v15 rule | v17 clause | Implementation | Tests |
//! |---|---|---|---|
//! | Readiness before every invocation | Readiness and coverage | `readiness::evaluate`, gate in `cmd/review.rs` | every `*_refused`, `*_not_ready*` test |
//! | Environment usable | `EXECUTION_ENVIRONMENT_BLOCKED` | `readiness::environment`, `docker_available` | `unavailable_reviewer_cli_*`, `missing_reviewer_credentials_*`, `unavailable_docker_*` |
//! | Toolchain usable | `REVIEWER_BUILD_MISMATCH` | `build_identity`, `readiness::toolchain` | `stale_reviewer_binary_is_refused` |
//! | Machine-owned identities | `CONFIGURATION_MISMATCH`, `IMPLEMENTATION_IDENTITY_INVALID`, supplied hashes are expectations | `manifest::derive`, `readiness::identities`, `cmd/decision.rs` | `transcribed_configuration_*`, `unsupported_implementation_*`, `supplied_event_evidence_*`, `authorization_record_hashes_*` |
//! | Evidence current and coherent | `EVIDENCE_STALE`, `F6_SELECTED_EVIDENCE_MISMATCH` | `readiness::evidence` | `implementation_changed_*`, `wrong_f6_*` |
//! | Required inputs inspectable | `REQUIRED_INPUT_MISSING`, `REQUIRED_INPUT_NOT_INSPECTABLE` | `readiness::inspectability`, prepare governed-context check | `missing_required_verification_*`, `missing_governed_context_*`, `caller_cannot_narrow_*`, `caller_cannot_omit_*`, `owning_architecture_scope_*`, `unreadable_architecture_scopes_*` |
//! | One manifest drives the packet | Review Input Manifest, incl. lineage target | `manifest.rs`, `packet::include_verification_records`, sidecar | `complete_current_evidence_*`, `owning_architecture_scope_*` |
//! | Scope-aware coverage and budget | `FULL_COVERAGE` over the required set | `PacketBuildOptions::scope`, `readiness::coverage` | `coverage_and_budget_are_scoped_*` |
//! | Reviewed-state snapshot | `SNAPSHOT_NOT_RECONSTRUCTIBLE`, snapshot deltas | `ReviewedSnapshot::full_file_preimages`, `--base` refusal | `stage8_base_commit_*`, `packet::tests::reviewed_untracked_full_file_*` |
//! | Bounded lineage | `LINEAGE_INVALID` | `refuse_lineage` | `absent_bounded_lineage_is_refused` |
//! | Execution failure is not semantic | `REVIEW_EXECUTION_INVALID`, two automatic re-executions | execution loop in `cmd/review.rs`, `log::append_execution_invalid` | `structurally_invalid_*`, `automatic_reexecution_*`, `invalid_execution_is_invisible_*` |
//! | Only development failures block | `DO NOT ADVANCE` unchanged | assessment parsing | `genuine_defects_still_block_progression` |
//! | v17 incorporates v16 | inherited mechanisms complete | bounded-basis required inputs, contract lists | `inherited_reentry_*`, `inherited_lineage_replacement_*`, `inherited_evidence_reassessment_*`, `lineage_replacement_completes_under_dba21` |
//! | F7 binds to reviewed identities | "what would be reviewed now" is the recorded manifest identity | `codeos_workflow::evidence::review_identity_drift` | `v17_review_binding_survives_*`, `pre_v17_review_records_keep_head_binding` |
//! | UX receipts bind the judged product (DBA-22+) | F4/F8 bind `product_state`; evidence keeps full binding | `checker::bindings_for_decision`, `hashing::product_state` | `ux_receipts_ignore_test_only_edits_*`, `existing_receipts_and_earlier_configurations_*` |
//! | Required evidence by identity; explicit budget | referenced evidence present and intact; size split; per-project budget | `ReviewInputManifest::referenced_production_evidence`, `print_size`, `config::packet_budget_bytes` | `referenced_production_evidence_*`, `packet_size_report_*`, `project_reviewer_toml_*` |
//! | End to end | manifest → gate → packet → review → record → workflow/decision | all of the above | `ready_review_is_consumed_by_workflow_and_decision` |

mod common;

use codeos_workflow::project::Project;
use common::{
    repo_root, run_in_dir_with_env, run_with_fake_codex, setup_fake_codex, setup_temp_git_repo,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

const FEATURE: &str = "FEAT";
const CANDIDATE: &str = "DBA-22";

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn write(path: &Path, value: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, value).unwrap();
}

fn file_sha(path: &Path) -> String {
    sha256(&std::fs::read(path).unwrap())
}

fn git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(output.status.success(), "git {}", args.join(" "));
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name == "target" {
            continue;
        }
        let (source, dest) = (entry.path(), to.join(&name));
        if source.is_dir() {
            copy_tree(&source, &dest);
        } else {
            std::fs::copy(&source, &dest).unwrap();
        }
    }
}

/// A private copy of the toolkit, so a test may change reviewer sources without touching the
/// real one. Projects bind the registered DBA-22 configuration.
fn candidate_toolkit() -> tempfile::TempDir {
    let toolkit = tempfile::tempdir().unwrap();
    copy_tree(&repo_root().join("dba"), &toolkit.path().join("dba"));
    for file in ["dba-system.md", "CLAUDE.md", "AGENTS.md"] {
        let source = repo_root().join(file);
        if source.is_file() {
            std::fs::copy(&source, toolkit.path().join(file)).unwrap();
        }
    }
    toolkit
}

struct Fixture {
    repo: tempfile::TempDir,
    _toolkit: tempfile::TempDir,
    toolkit: PathBuf,
}

impl Fixture {
    fn path(&self) -> &Path {
        self.repo.path()
    }
}

/// A Stage-8 project whose canonical evidence is complete and current.
fn ready_project(reality: bool) -> Fixture {
    let (repo, _) = setup_temp_git_repo();
    let toolkit = candidate_toolkit();
    let root = repo.path();
    std::fs::create_dir_all(root.join(".codeos")).unwrap();
    std::os::unix::fs::symlink(toolkit.path(), root.join(".codeos/toolkit")).unwrap();
    write(
        &root.join(".git/info/exclude"),
        "/.codeos/toolkit\n/.codeos/05-review/\n",
    );
    write(
        &root.join(".codeos/00-project/codeos.yaml"),
        &format!("codeos_schema: 2\ndba_configuration: {CANDIDATE}\n"),
    );
    write(
        &root.join(".codeos/00-project/charter.md"),
        "---\nartifact_type: charter\napproval:\n  by: Human\n  at: 2026-10-06\n---\n# Charter\n",
    );
    let header = |kind: &str| {
        format!(
            "---\nartifact_type: {kind}\nfeature_id: {FEATURE}\nsemantic_revision: 2\nsemantic_revision_from: 1\nsemantic_change: readiness fixture\nstatus: APPROVED\napproved_by: Human\napproved_at: 2026-10-06\n---\n# {kind}\n"
        )
    };
    let mut contract = header("contract");
    if reality {
        contract.push_str(
            "\n## Reality Basis\n\n| Claim ID | Claim | Classification | Source | Freshness | Coverage | Failure | Disclosure |\n|---|---|---|---|---|---|---|---|\n| C1 | live departures | reality-dependent | provider | 5m | route | unavailable | shown |\n",
        );
    }
    write(&root.join(intent()), &header("intent"));
    write(&root.join(contract_path()), &contract);
    write(&root.join(schema()), &header("event_schema"));
    write(&root.join("src/lib.rs"), "pub fn feature() {}\n");
    git(root, &["add", "."]);
    git(
        root,
        &["commit", "-m", "approved package and implementation"],
    );

    let fixture = Fixture {
        toolkit: toolkit.path().to_path_buf(),
        repo,
        _toolkit: toolkit,
    };
    record_current_evidence(&fixture, reality);
    git(fixture.path(), &["add", ".codeos/06-workflow"]);
    git(fixture.path(), &["commit", "-m", "record current evidence"]);
    fixture
}

fn intent() -> String {
    format!(".codeos/01-specification/intents/{FEATURE}.md")
}
fn contract_path() -> String {
    format!(".codeos/01-specification/contracts/{FEATURE}_contract.md")
}
fn schema() -> String {
    format!(".codeos/01-specification/event-schemas/{FEATURE}_schema.md")
}

fn implementation_state(root: &Path) -> String {
    let project = Project::discover(root).unwrap();
    codeos_workflow::hashing::working_tree_state(&project.root, &project.implementation_paths())
        .unwrap()
}

fn record_current_evidence(fixture: &Fixture, reality: bool) {
    record_evidence_with_events(fixture, reality, "{\"event\":\"feature_completed\"}\n");
}

fn record_evidence_with_events(fixture: &Fixture, reality: bool, events: &str) {
    let root = fixture.path();
    let implementation = implementation_state(root);
    let bindings = json!({
        "intent": file_sha(&root.join(intent())),
        "contract": file_sha(&root.join(contract_path())),
        "event_schema": file_sha(&root.join(schema())),
        "implementation_state": implementation,
    });
    let events_sha = sha256(events.as_bytes());
    let events_path = format!(".codeos/06-workflow/event-evidence/{events_sha}.jsonl");
    write(&root.join(&events_path), events);

    let mut required = vec![
        ("F3", "smoke", Value::Null),
        ("F3", "product_journey", Value::Null),
        ("F5", "behavior", Value::Null),
        ("F5", "repeatability", Value::Null),
        (
            "F7",
            "runtime_event_schema",
            json!({
                "event_evidence": events_path,
                "event_set_sha256": events_sha,
                "semantic_sha256": events_sha,
                "event_schema_conformance": "passed",
            }),
        ),
    ];
    let mut f6 = json!({
        "codeos_yaml": file_sha(&root.join(".codeos/00-project/codeos.yaml")),
        "implementation_state": implementation,
    });
    if reality {
        let evidence = json!({ "implementation_sha256": implementation, "observations": [] });
        let bytes = serde_json::to_vec(&evidence).unwrap();
        let selected = sha256(&bytes);
        let path = root.join(format!(
            ".codeos/06-workflow/reality-evidence/{selected}.json"
        ));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
        required.push((
            "F5",
            "production_source_evidence",
            json!({ "evidence_sha256": selected }),
        ));
        f6["production_evidence"] = json!(selected);
    }
    let mut store = String::new();
    for (checkpoint, verification, details) in required {
        let mut record = json!({
            "workflow": "feature", "subject": FEATURE, "checkpoint": checkpoint,
            "verification": verification, "result": "passed", "bindings": bindings,
            "command": "fixture", "timestamp": "2026-10-06T10:00:00Z",
        });
        if !details.is_null() {
            record["details"] = details;
        }
        store.push_str(&serde_json::to_string(&record).unwrap());
        store.push('\n');
    }
    write(
        &root.join(".codeos/06-workflow/verifications.jsonl"),
        &store,
    );
    let receipt = json!({
        "workflow": "feature", "subject": FEATURE, "checkpoint": "reconciliation",
        "result": "completed", "bindings": f6, "timestamp": "2026-10-06T10:05:00Z",
    });
    write(
        &root.join(".codeos/06-workflow/decisions.jsonl"),
        &format!("{}\n", serde_json::to_string(&receipt).unwrap()),
    );
}

fn review(fixture: &Fixture, fake: &common::FakeCodex, extra: &[&str]) -> (i32, String, String) {
    let mut args = vec!["review", FEATURE, "8", "--skip-prechecks"];
    args.extend_from_slice(extra);
    args.push("src/lib.rs");
    run_with_fake_codex(fixture.path(), fake, &args, "success")
}

fn invoked(fake: &common::FakeCodex) -> bool {
    fake.packet_log.exists()
}

fn assert_not_ready(
    fixture: &Fixture,
    result: (i32, String, String),
    code: &str,
    fake: &common::FakeCodex,
) {
    let (status, stdout, stderr) = result;
    assert_ne!(status, 0, "review unexpectedly passed:\n{stdout}\n{stderr}");
    assert!(
        stderr.contains(&format!("REVIEW_NOT_READY: {code}")),
        "expected {code}:\n{stderr}"
    );
    assert!(!invoked(fake), "reviewer must not be invoked");
    assert!(
        !fixture.path().join(".codeos/05-review/reviews").exists(),
        "no review record may be written"
    );
}

#[test]
fn complete_current_evidence_reaches_the_reviewer_with_machine_owned_inputs() {
    let fixture = ready_project(true);
    let fake = setup_fake_codex();
    let (status, stdout, stderr) = review(&fixture, &fake, &[]);
    assert_eq!(status, 0, "review failed:\n{stdout}\n{stderr}");
    assert!(invoked(&fake));

    let packet = std::fs::read_to_string(&fake.packet_log).unwrap();
    assert!(packet.contains("REQUIRED VERIFICATION RECORDS"));
    assert!(packet.contains("\"verification\":\"production_source_evidence\""));
    assert!(packet.contains(".codeos/06-workflow/reality-evidence/"));
    assert!(packet.contains(".codeos/06-workflow/event-evidence/"));

    let packets = fixture
        .path()
        .join(".codeos/05-review/reviews/codex/packets");
    let sidecar = std::fs::read_dir(&packets)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.to_string_lossy().ends_with(".packet.txt.meta.json"))
        .expect("Review Input Manifest sidecar");
    let manifest: Value = serde_json::from_str(&std::fs::read_to_string(sidecar).unwrap()).unwrap();
    let manifest = &manifest["manifest"];
    assert_eq!(manifest["operation"], "REV__FEAT__8__SR2__R1");
    assert_eq!(
        manifest["implementation"],
        implementation_state(fixture.path())
    );
    assert_eq!(manifest["verification"].as_object().unwrap().len(), 6);
    assert!(manifest["reviewed_snapshot"].as_str().unwrap().len() == 64);
}

#[test]
fn unavailable_reviewer_cli_is_an_environment_block() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    let result = run_in_dir_with_env(
        fixture.path(),
        &["review", FEATURE, "8", "--skip-prechecks", "src/lib.rs"],
        &[("PATH", "/usr/bin:/bin")],
    );
    assert_not_ready(&fixture, result, "EXECUTION_ENVIRONMENT_BLOCKED", &fake);
}

#[test]
fn stale_reviewer_binary_is_refused() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    let main = fixture
        .toolkit
        .join("dba/04-tools/reviewer/engine/src/main.rs");
    let mut source = std::fs::read_to_string(&main).unwrap();
    source.push_str("// newer source than the running binary\n");
    std::fs::write(&main, source).unwrap();
    assert_not_ready(
        &fixture,
        review(&fixture, &fake, &[]),
        "REVIEWER_BUILD_MISMATCH",
        &fake,
    );
}

#[test]
fn transcribed_configuration_identity_must_match_the_artifact() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    write(
        &fixture.path().join("evidence/review/claims.json"),
        &json!({ "bindings": { "codeos_yaml": "0".repeat(64) } }).to_string(),
    );
    let result = review(&fixture, &fake, &["evidence/review/claims.json"]);
    assert_not_ready(&fixture, result, "CONFIGURATION_MISMATCH", &fake);
}

#[test]
fn unsupported_implementation_identity_is_refused() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    write(
        &fixture.path().join("evidence/review/claims.json"),
        &json!({ "bindings": { "implementation_state": "f".repeat(64) } }).to_string(),
    );
    let result = review(&fixture, &fake, &["evidence/review/claims.json"]);
    assert_not_ready(&fixture, result, "IMPLEMENTATION_IDENTITY_INVALID", &fake);
}

#[test]
fn wrong_f6_production_evidence_binding_is_refused() {
    let fixture = ready_project(true);
    let fake = setup_fake_codex();
    let store = fixture
        .path()
        .join(".codeos/06-workflow/verifications.jsonl");
    let text = std::fs::read_to_string(&store).unwrap();
    let rebound: Vec<String> = text
        .lines()
        .map(|line| {
            let mut record: Value = serde_json::from_str(line).unwrap();
            if record["verification"] == "production_source_evidence" {
                record["details"]["evidence_sha256"] = json!("a".repeat(64));
            }
            serde_json::to_string(&record).unwrap()
        })
        .collect();
    std::fs::write(&store, rebound.join("\n") + "\n").unwrap();
    assert_not_ready(
        &fixture,
        review(&fixture, &fake, &[]),
        "F6_SELECTED_EVIDENCE_MISMATCH",
        &fake,
    );
}

#[test]
fn missing_required_verification_is_refused() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    let store = fixture
        .path()
        .join(".codeos/06-workflow/verifications.jsonl");
    let text = std::fs::read_to_string(&store).unwrap();
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| !line.contains("\"behavior\""))
        .collect();
    std::fs::write(&store, kept.join("\n") + "\n").unwrap();
    assert_not_ready(
        &fixture,
        review(&fixture, &fake, &[]),
        "REQUIRED_INPUT_MISSING",
        &fake,
    );
}

#[test]
fn implementation_changed_after_verification_is_stale() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    write(
        &fixture.path().join("src/lib.rs"),
        "pub fn feature() { changed() }\n",
    );
    assert_not_ready(
        &fixture,
        review(&fixture, &fake, &[]),
        "EVIDENCE_STALE",
        &fake,
    );
}

#[test]
fn caller_cannot_narrow_a_required_input_to_its_hash() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    let contract = contract_path();
    let result = review(&fixture, &fake, &["--sha-only", &contract]);
    assert_not_ready(&fixture, result, "REQUIRED_INPUT_NOT_INSPECTABLE", &fake);
}

/// Change the implementation (one modified, one new untracked file), then re-record current
/// evidence for that exact working tree, as the workflow would before Stage 8.
fn change_implementation(fixture: &Fixture, reality: bool) {
    let root = fixture.path();
    write(&root.join("src/lib.rs"), "pub fn feature() { helper() }\n");
    write(
        &root.join("src/new.rs"),
        "pub fn helper() { /* NEW-IMPLEMENTATION-MARKER */ }\n",
    );
    record_current_evidence(fixture, reality);
    git(root, &["add", ".codeos/06-workflow"]);
    git(
        root,
        &[
            "commit",
            "-m",
            "re-record evidence for the changed implementation",
        ],
    );
}

/// Material outside any review scope: old evidence, generated logs, unrelated notes.
fn add_unrelated_history(fixture: &Fixture) {
    let root = fixture.path();
    write(
        &root.join(".codeos/06-workflow/reality-evidence/historical.json"),
        "{\"marker\":\"UNRELATED-HISTORY-MARKER\"}\n",
    );
    write(
        &root.join("notes/unrelated.md"),
        &format!(
            "api_key=UNRELATEDSECRETVALUE\n{}",
            "UNRELATED-HISTORY-MARKER\n".repeat(6000)
        ),
    );
}

fn sidecar(fixture: &Fixture) -> Value {
    let packets = fixture
        .path()
        .join(".codeos/05-review/reviews/codex/packets");
    let path = std::fs::read_dir(&packets)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.to_string_lossy().ends_with(".packet.txt.meta.json"))
        .expect("Review Input Manifest sidecar");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn coverage_and_budget_are_scoped_to_required_inputs() {
    let fixture = ready_project(true);
    change_implementation(&fixture, true);
    add_unrelated_history(&fixture);
    let fake = setup_fake_codex();
    let (status, stdout, stderr) = review(&fixture, &fake, &[]);
    assert_eq!(status, 0, "review failed:\n{stdout}\n{stderr}");

    let packet = std::fs::read_to_string(&fake.packet_log).unwrap();
    assert!(
        packet.contains("NEW-IMPLEMENTATION-MARKER"),
        "new file must be reviewed"
    );
    assert!(
        packet.contains("helper()"),
        "modified file must be reviewed"
    );
    assert!(
        !packet.contains("UNRELATED-HISTORY-MARKER"),
        "unrelated history entered the packet"
    );

    let sidecar = sidecar(&fixture);
    assert_eq!(sidecar["coverage_state"], "FullCoverage");
    assert_eq!(
        sidecar["redaction_count"], 0,
        "unrelated secrets must not affect coverage"
    );
    let required = sidecar["manifest"]["required_inputs"].as_array().unwrap();
    assert!(required.iter().any(|path| path == "src/new.rs"));
    assert!(!required.iter().any(|path| path == "notes/unrelated.md"));
    // The unrelated 100+ KB note would exceed the default budget if it were counted.
    assert!(
        sidecar["review_content_bytes"].as_u64().unwrap()
            < sidecar["budget_bytes"].as_u64().unwrap()
    );
}

#[test]
fn caller_cannot_omit_a_changed_implementation_file() {
    let fixture = ready_project(false);
    change_implementation(&fixture, false);
    let fake = setup_fake_codex();
    let result = review(&fixture, &fake, &["--sha-only", "src/new.rs"]);
    assert_not_ready(&fixture, result, "REQUIRED_INPUT_NOT_INSPECTABLE", &fake);
}

fn current_event_sha(fixture: &Fixture) -> String {
    let store = std::fs::read_to_string(
        fixture
            .path()
            .join(".codeos/06-workflow/verifications.jsonl"),
    )
    .unwrap();
    store
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find_map(|record| {
            record["details"]["event_set_sha256"]
                .as_str()
                .map(str::to_string)
        })
        .unwrap()
}

fn reassessment(
    fixture: &Fixture,
    fake: &common::FakeCodex,
    sha: Option<&str>,
) -> (i32, String, String) {
    let mut extra = vec![
        "--evidence-reassessment-after",
        "REV__FEAT__8__SR2__R3",
        "--findings",
        "F1",
    ];
    if let Some(sha) = sha {
        extra.extend(["--event-evidence-sha", sha]);
    }
    review(fixture, fake, &extra)
}

#[test]
fn supplied_event_evidence_hash_is_only_an_expectation() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    let wrong = "e".repeat(64);
    assert_not_ready(
        &fixture,
        reassessment(&fixture, &fake, Some(&wrong)),
        "EVIDENCE_STALE",
        &fake,
    );

    // Omitted and correct hashes both pass identity derivation; this fixture then stops at its
    // deliberately absent predecessor, never at a missing-hash usage error.
    let current = current_event_sha(&fixture);
    for sha in [None, Some(current.as_str())] {
        let result = reassessment(&fixture, &fake, sha);
        assert!(
            !result
                .2
                .contains("requires --findings and --event-evidence-sha"),
            "{}",
            result.2
        );
        assert_not_ready(&fixture, result, "SNAPSHOT_NOT_RECONSTRUCTIBLE", &fake);
    }
}

#[test]
fn authorization_record_hashes_are_derived_under_v17() {
    let fixture = ready_project(false);
    write(
        &fixture
            .path()
            .join(".codeos/05-review/reviews/review-log.md"),
        "# Review log\n",
    );
    for (decision, flag) in [
        (
            "AUTHORIZE_LINEAGE_REPLACEMENT",
            "--applicability-record-sha",
        ),
        (
            "AUTHORIZE_ASSESSMENT_INPUT_CORRECTION",
            "--correction-record-sha",
        ),
    ] {
        let (status, _, stderr) = run_in_dir_with_env(
            fixture.path(),
            &[
                "decision",
                FEATURE,
                "8",
                decision,
                "bounded reason",
                "--after-review",
                "REV__FEAT__8__SR2__LINEAGE-REPLACEMENT-1",
            ],
            &[],
        );
        assert_ne!(status, 0);
        assert!(
            !stderr.contains(&format!("requires {flag}")),
            "{decision}: {stderr}"
        );
    }
}

#[test]
fn stage8_base_commit_delta_is_not_reviewed_state() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    let result = review(&fixture, &fake, &["--base", "HEAD"]);
    assert_not_ready(&fixture, result, "SNAPSHOT_NOT_RECONSTRUCTIBLE", &fake);
}

fn invocations(fake: &common::FakeCodex) -> usize {
    let mut count = fake.args_log.clone().into_os_string();
    count.push(".count");
    std::fs::read_to_string(PathBuf::from(count))
        .map(|text| text.lines().count())
        .unwrap_or(0)
}

fn review_log(fixture: &Fixture) -> String {
    std::fs::read_to_string(
        fixture
            .path()
            .join(".codeos/05-review/reviews/review-log.md"),
    )
    .unwrap_or_default()
}

fn review_mode(fixture: &Fixture, fake: &common::FakeCodex, mode: &str) -> (i32, String, String) {
    run_with_fake_codex(
        fixture.path(),
        fake,
        &["review", FEATURE, "8", "--skip-prechecks", "src/lib.rs"],
        mode,
    )
}

#[test]
fn structurally_invalid_responses_consume_no_review_capacity() {
    for mode in ["structural_failure", "malformed_finding"] {
        let fixture = ready_project(false);
        let fake = setup_fake_codex();
        let (status, stdout, stderr) = review_mode(&fixture, &fake, mode);
        assert_ne!(status, 0, "{mode}: {stdout}\n{stderr}");
        assert!(
            stderr.contains("REVIEW_EXECUTION_INVALID"),
            "{mode}: {stderr}"
        );
        assert_eq!(
            invocations(&fake),
            3,
            "{mode}: one execution plus two automatic re-executions"
        );

        let log = review_log(&fixture);
        assert!(
            log.contains("REVIEW EXECUTION INVALID — FEAT — Stage 8"),
            "{log}"
        );
        assert!(
            !log.contains("Review ID:"),
            "an invalid execution is not a review: {log}"
        );
        let codex = fixture.path().join(".codeos/05-review/reviews/codex");
        let assessments = std::fs::read_dir(&codex)
            .unwrap()
            .filter(|entry| {
                entry
                    .as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|e| e == "md")
            })
            .count();
        assert_eq!(assessments, 0, "no semantic assessment may be written");
        let preserved = std::fs::read_dir(codex.join("execution-invalid"))
            .unwrap()
            .count();
        assert_eq!(preserved, 4, "identical packet plus three raw attempts");

        // Capacity is unchanged: the next valid review is still round 1.
        let fake = setup_fake_codex();
        let (status, stdout, stderr) = review_mode(&fixture, &fake, "success");
        assert_eq!(status, 0, "{stdout}\n{stderr}");
        assert!(review_log(&fixture).contains("Review ID: REV__FEAT__8__SR2__R1"));
    }
}

#[test]
fn automatic_reexecution_recovers_with_identical_inputs() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    let (status, stdout, stderr) = review_mode(&fixture, &fake, "structural_once");
    assert_eq!(status, 0, "{stdout}\n{stderr}");
    assert_eq!(invocations(&fake), 2);
    let log = review_log(&fixture);
    assert!(log.contains("Review ID: REV__FEAT__8__SR2__R1"), "{log}");
    assert!(!log.contains("REVIEW EXECUTION INVALID"));
    let preserved = fixture
        .path()
        .join(".codeos/05-review/reviews/codex/execution-invalid");
    assert_eq!(
        std::fs::read_dir(preserved).unwrap().count(),
        2,
        "packet plus one attempt"
    );
}

fn assert_refused_with(
    fixture: &Fixture,
    result: (i32, String, String),
    code: &str,
    fake: &common::FakeCodex,
) {
    let (status, _, stderr) = result;
    assert_ne!(status, 0, "{stderr}");
    assert!(stderr.contains("REVIEW_NOT_READY"), "{stderr}");
    assert!(
        stderr.contains(&format!("  - {code}:")),
        "expected {code}:\n{stderr}"
    );
    assert!(!invoked(fake));
    assert!(!fixture.path().join(".codeos/05-review/reviews").exists());
}

#[test]
fn unavailable_docker_is_reported_as_the_environment_cause() {
    let fixture = ready_project(false);
    write(&fixture.path().join("docker-compose.yml"), "services: {}\n");
    let fake = setup_fake_codex();
    write(
        &fake.dir.path().join("docker"),
        "#!/usr/bin/env bash\nexit 1\n",
    );
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(
        fake.dir.path().join("docker"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    // The new compose file changes the implementation, so evidence must be refreshed with Docker.
    assert_not_ready(
        &fixture,
        review(&fixture, &fake, &[]),
        "EXECUTION_ENVIRONMENT_BLOCKED",
        &fake,
    );
}

#[test]
fn missing_governed_context_is_refused() {
    let fixture = ready_project(false);
    std::fs::remove_file(fixture.path().join(schema())).unwrap();
    let fake = setup_fake_codex();
    assert_refused_with(
        &fixture,
        review(&fixture, &fake, &[]),
        "REQUIRED_INPUT_MISSING",
        &fake,
    );
}

#[test]
fn absent_bounded_lineage_is_refused() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    let result = review(
        &fixture,
        &fake,
        &["--replace-invalid-lineage-after", "REV__FEAT__8__SR2__R3"],
    );
    assert_not_ready(&fixture, result, "LINEAGE_INVALID", &fake);
}

#[test]
fn genuine_defects_still_block_progression() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    let (status, stdout, stderr) = review_mode(&fixture, &fake, "blocking");
    assert_eq!(status, 0, "{stdout}\n{stderr}");
    let record = codeos_workflow::evidence::latest_review_record(
        &fixture.path().join(".codeos/05-review/reviews"),
        FEATURE,
        "8",
    )
    .unwrap()
    .expect("semantic review record");
    assert_eq!(record.review_id, "REV__FEAT__8__SR2__R1");
    assert_eq!(record.effective_concern, "DO NOT ADVANCE");
    assert!(!codeos_workflow::evidence::review_permits_progression(
        &record.effective_concern
    ));
}

/// End to end: project state -> manifest -> readiness PASS -> full-coverage packet -> reviewer ->
/// valid assessment -> record -> workflow reader and human decision consume it.
#[test]
fn ready_review_is_consumed_by_workflow_and_decision() {
    let fixture = ready_project(true);
    let fake = setup_fake_codex();
    let (status, stdout, stderr) = review_mode(&fixture, &fake, "success");
    assert_eq!(status, 0, "{stdout}\n{stderr}");
    assert_eq!(invocations(&fake), 1);
    assert_eq!(sidecar(&fixture)["coverage_state"], "FullCoverage");

    let record = codeos_workflow::evidence::latest_review_record(
        &fixture.path().join(".codeos/05-review/reviews"),
        FEATURE,
        "8",
    )
    .unwrap()
    .expect("semantic review record");
    assert_eq!(record.review_id, "REV__FEAT__8__SR2__R1");
    assert!(codeos_workflow::evidence::review_permits_progression(
        &record.effective_concern
    ));

    let (status, _, stderr) = run_in_dir_with_env(
        fixture.path(),
        &[
            "decision",
            FEATURE,
            "8",
            "APPROVE_STAGE",
            "ready review accepted",
        ],
        &[],
    );
    assert_eq!(status, 0, "{stderr}");
    assert!(review_log(&fixture).contains("APPROVE_STAGE"));
}

#[test]
fn invalid_execution_is_invisible_to_the_workflow() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    let _ = review_mode(&fixture, &fake, "structural_failure");
    let record = codeos_workflow::evidence::latest_review_record(
        &fixture.path().join(".codeos/05-review/reviews"),
        FEATURE,
        "8",
    )
    .unwrap();
    assert!(
        record.is_none(),
        "an invalid execution is not a review record"
    );
}

#[test]
fn owning_architecture_scope_is_required_and_included() {
    let fixture = ready_project(false);
    let scope = ".codeos/02-architecture/scopes/core.md";
    write(
        &fixture.path().join(scope),
        "---\nartifact_type: architecture\nreader_model: stable-topic\nfeatures:\n  - FEAT\napproval:\n  by: Human\n  at: 2026-10-06\n---\n# Core SCOPE-MARKER\n",
    );
    git(fixture.path(), &["add", scope]);
    git(
        fixture.path(),
        &["commit", "-m", "approve architecture scope"],
    );
    let fake = setup_fake_codex();
    let (status, stdout, stderr) = review(&fixture, &fake, &[]);
    assert_eq!(status, 0, "{stdout}\n{stderr}");
    assert!(std::fs::read_to_string(&fake.packet_log)
        .unwrap()
        .contains("SCOPE-MARKER"));
    let manifest = &sidecar(&fixture)["manifest"];
    assert!(
        manifest["governed_context"].get(scope).is_some(),
        "{manifest}"
    );
    assert!(manifest["lineage_target"].is_null());
}

#[test]
fn unreadable_architecture_scopes_are_refused() {
    let fixture = ready_project(false);
    write(
        &fixture
            .path()
            .join(".codeos/02-architecture/scopes/core.md"),
        "---\nfeatures: []\napproval: null\n---\n# Invalid scope\n",
    );
    let fake = setup_fake_codex();
    assert_refused_with(
        &fixture,
        review(&fixture, &fake, &[]),
        "REQUIRED_INPUT_NOT_INSPECTABLE",
        &fake,
    );
}

#[test]
fn missing_reviewer_credentials_are_an_environment_block() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    let result = common::run_with_fake_codex_env(
        fixture.path(),
        &fake,
        &["review", FEATURE, "8", "--skip-prechecks", "src/lib.rs"],
        "success",
        &[("CODEOS_FAKE_AUTH", "none")],
    );
    assert!(result.2.contains("codex credentials"), "{}", result.2);
    assert_not_ready(&fixture, result, "EXECUTION_ENVIRONMENT_BLOCKED", &fake);
}

fn governed_review(
    fixture: &Fixture,
    fake: &common::FakeCodex,
    mode: &str,
    extra: &[&str],
) -> (i32, String, String) {
    let (intent, contract, schema) = (intent(), contract_path(), schema());
    let mut args = vec!["review", FEATURE, "8", "--skip-prechecks"];
    args.extend_from_slice(extra);
    args.extend([
        ".codeos/00-project/charter.md",
        intent.as_str(),
        contract.as_str(),
        schema.as_str(),
        "src/lib.rs",
    ]);
    run_with_fake_codex(fixture.path(), fake, &args, mode)
}

fn decide(fixture: &Fixture, args: &[&str]) {
    let (status, stdout, stderr) = run_in_dir_with_env(fixture.path(), args, &[]);
    assert_eq!(status, 0, "decision failed:\n{stdout}\n{stderr}");
}

fn assert_ok(result: (i32, String, String), what: &str) {
    assert_eq!(result.0, 0, "{what} failed:\n{}\n{}", result.1, result.2);
}

/// Inherited v16 mechanism: bounded re-entry completes under v17 readiness and snapshot rules.
#[test]
fn inherited_reentry_completes_under_v17() {
    let fixture = ready_project(false);
    let fake = setup_fake_codex();
    for mode in ["success", "success", "blocking"] {
        assert_ok(
            governed_review(&fixture, &fake, mode, &[]),
            "ordinary round",
        );
    }
    decide(
        &fixture,
        &[
            "decision",
            FEATURE,
            "8",
            "AUTHORIZE_REENTRY",
            "One bounded repair review.",
            "--after-review",
            "REV__FEAT__8__SR2__R3",
        ],
    );
    change_implementation(&fixture, false);
    assert_ok(
        governed_review(
            &fixture,
            &fake,
            "success",
            &["--reentry-after", "REV__FEAT__8__SR2__R3"],
        ),
        "re-entry",
    );
    let log = review_log(&fixture);
    assert!(
        log.contains("Review ID: REV__FEAT__8__SR2__REENTRY-1"),
        "{log}"
    );
    // Records written within one second share a file-name timestamp, so assert on the record.
    let reentry = record_block(&log, "REV__FEAT__8__SR2__REENTRY-1");
    assert!(
        reentry.contains("Effective concern: NO OBJECTION"),
        "{reentry}"
    );
}

/// Rebind the project to another approved configuration and refresh the F6/verification records
/// that bind `codeos.yaml`, as an explicit human-authorized adoption would.
fn bind(fixture: &Fixture, configuration: &str, reality: bool) {
    write(
        &fixture.path().join(".codeos/00-project/codeos.yaml"),
        &format!("codeos_schema: 2\ndba_configuration: {configuration}\n"),
    );
    record_current_evidence(fixture, reality);
    git(
        fixture.path(),
        &["add", ".codeos/00-project", ".codeos/06-workflow"],
    );
    git(
        fixture.path(),
        &["commit", "-m", &format!("bind {configuration}")],
    );
}

fn record_block(log: &str, review_id: &str) -> String {
    log.split_once(&format!("Review ID: {review_id}"))
        .unwrap_or_else(|| panic!("missing {review_id}:\n{log}"))
        .1
        .split("\n## ")
        .next()
        .unwrap()
        .to_string()
}

fn first_finding_id(fixture: &Fixture, review_id: &str) -> (String, String) {
    let block = record_block(&review_log(fixture), review_id);
    let reference = block
        .lines()
        .find_map(|line| line.strip_prefix("Full assessment: "))
        .unwrap();
    let path = reference.rsplit_once(" (sha256:").unwrap().0;
    let text = std::fs::read_to_string(path).unwrap();
    let yaml: Value = serde_yaml::from_str(
        text.strip_prefix("---\n")
            .unwrap()
            .split_once("\n---")
            .unwrap()
            .0,
    )
    .unwrap();
    let id = yaml["findings"][0]["finding_id"]
        .as_str()
        .unwrap()
        .to_string();
    (id, sha256(text.as_bytes()))
}

/// Inherited v14 mechanism: a lineage replacement of a pre-v14 closure that cited production
/// evidence F6 never selected completes under v17 with derived authorization identity.
#[test]
fn inherited_lineage_replacement_completes_under_v17() {
    lineage_replacement_flow(CANDIDATE);
}

/// Conformance repair: a progression-permitting replacement is also recordable under DBA-21.
#[test]
fn lineage_replacement_completes_under_dba21() {
    lineage_replacement_flow("DBA-21");
}

fn lineage_replacement_flow(recovery: &str) {
    const R3: &str = "REV__FEAT__8__SR2__R3";
    const REENTRY: &str = "REV__FEAT__8__SR2__REENTRY-1";
    const CLOSURE: &str = "REV__FEAT__8__SR2__TERMINAL-CLOSURE-1";
    let fixture = ready_project(true);
    bind(&fixture, "DBA-18", true);
    let fake = setup_fake_codex();

    // History under DBA-18.
    for (round, mode) in ["success", "success", "blocking"].into_iter().enumerate() {
        write(
            &fixture.path().join("src/lib.rs"),
            &format!("pub fn feature() {{ round_{round}() }}\n"),
        );
        git(fixture.path(), &["commit", "-am", "next round"]);
        assert_ok(
            governed_review(&fixture, &fake, mode, &[]),
            "ordinary round",
        );
    }
    decide(
        &fixture,
        &[
            "decision",
            FEATURE,
            "8",
            "AUTHORIZE_REENTRY",
            "bounded repair",
            "--after-review",
            R3,
        ],
    );
    write(
        &fixture.path().join("src/lib.rs"),
        "pub fn feature() { candidate() }\n",
    );
    git(fixture.path(), &["commit", "-am", "re-entry candidate"]);
    assert_ok(
        governed_review(&fixture, &fake, "blocking", &["--reentry-after", R3]),
        "re-entry",
    );
    let (finding, assessment_sha) = first_finding_id(&fixture, REENTRY);
    let control = tempfile::tempdir().unwrap();
    let scope = control.path().join("scope.json");
    write(
        &scope,
        &json!({
            "terminal_review_id": REENTRY,
            "assessment_sha256": assessment_sha,
            "conditions": [{
                "id": "C1",
                "description": "Repair the terminal implementation defect",
                "source": {"kind": "parsed_finding", "finding_id": finding}
            }]
        })
        .to_string(),
    );
    decide(
        &fixture,
        &[
            "decision",
            FEATURE,
            "8",
            "AUTHORIZE_TERMINAL_REVIEW_CLOSURE",
            "repaired condition only",
            "--after-review",
            REENTRY,
            "--closure-scope",
            scope.to_str().unwrap(),
        ],
    );
    let reentry_block = record_block(&review_log(&fixture), REENTRY);
    let original = reentry_block
        .lines()
        .find_map(|line| line.strip_prefix("Base: "))
        .unwrap()
        .split("  Review: ")
        .nth(1)
        .unwrap()
        .split("  Branch:")
        .next()
        .unwrap()
        .to_string();
    let original_diff = reentry_block
        .lines()
        .find_map(|line| line.strip_prefix("Diff-hash: "))
        .unwrap()
        .to_string();

    change_implementation(&fixture, true);
    // The defect being recovered: the closure cites production evidence F6 never selected.
    let unselected = "{\"implementation_sha256\":\"unselected\"}\n";
    let unselected_path = format!(
        ".codeos/06-workflow/reality-evidence/{}.json",
        sha256(unselected.as_bytes())
    );
    write(&fixture.path().join(&unselected_path), unselected);
    git(fixture.path(), &["add", "."]);
    git(fixture.path(), &["commit", "-m", "repair and evidence"]);

    let (intent, contract, schema) = (intent(), contract_path(), schema());
    let closure_artifacts = [
        ".codeos/00-project/charter.md",
        intent.as_str(),
        contract.as_str(),
        schema.as_str(),
        "src/lib.rs",
        "src/new.rs",
        unselected_path.as_str(),
    ];
    let packet = control.path().join("closure.packet.txt");
    let mut plan = vec![
        "plan",
        FEATURE,
        "8",
        "--skip-prechecks",
        "--emit-packet",
        packet.to_str().unwrap(),
        "--terminal-review-closure-after",
        REENTRY,
    ];
    plan.extend(closure_artifacts);
    let (status, stdout, stderr) = run_in_dir_with_env(fixture.path(), &plan, &[]);
    assert_eq!(status, 0, "closure plan failed:\n{stdout}\n{stderr}");
    let sidecar: Value = serde_json::from_str(
        &std::fs::read_to_string(format!("{}.meta.json", packet.display())).unwrap(),
    )
    .unwrap();
    let artifacts = sidecar["artifacts"].as_array().unwrap();
    let changed: Vec<&str> = artifacts
        .iter()
        .filter(|a| matches!(a["visibility"].as_str(), Some("delta_diff" | "added_file")))
        .map(|a| a["path"].as_str().unwrap())
        .filter(|path| !path.starts_with(".codeos/"))
        .collect();
    let evidence: Vec<&str> = artifacts
        .iter()
        .filter(|a| {
            a["path"]
                .as_str()
                .unwrap()
                .starts_with(".codeos/06-workflow/")
        })
        .map(|a| a["sha256"].as_str().unwrap())
        .collect();
    let manifest = control.path().join("repair-manifest.json");
    write(
        &manifest,
        &json!({
            "terminal_review_id": REENTRY,
            "original_implementation_identity": format!("{original}:{original_diff}"),
            "current_implementation_identity": format!(
                "{}:{}",
                sidecar["review_sha"].as_str().unwrap(),
                sidecar["diff_hash"].as_str().unwrap()
            ),
            "changed_paths": changed,
            "evidence_identities": evidence,
            "links": [{
                "condition_id": "C1",
                "changed_paths": changed,
                "evidence_identities": evidence,
                "rationale": "The repair and its evidence close C1."
            }]
        })
        .to_string(),
    );
    let mut closure = vec![
        "review",
        FEATURE,
        "8",
        "--skip-prechecks",
        "--terminal-review-closure-after",
        REENTRY,
        "--repair-manifest",
        manifest.to_str().unwrap(),
    ];
    closure.extend(closure_artifacts);
    assert_ok(
        run_with_fake_codex(fixture.path(), &fake, &closure, "success"),
        "closure",
    );
    assert!(record_block(&review_log(&fixture), CLOSURE).contains("PROGRESSION PERMITTED"));

    // Recovery under the selected configuration.
    bind(&fixture, recovery, true);
    let (status, stdout, stderr) = run_in_dir_with_env(
        fixture.path(),
        &[
            "audit-review-applicability",
            FEATURE,
            "8",
            "--review-id",
            CLOSURE,
        ],
        &[],
    );
    assert_eq!(status, 0, "audit failed:\n{stdout}\n{stderr}");
    let record_sha = stdout
        .trim()
        .rsplit_once('(')
        .unwrap()
        .1
        .trim_end_matches(')')
        .to_string();
    let mut authorize = vec![
        "decision",
        FEATURE,
        "8",
        "AUTHORIZE_LINEAGE_REPLACEMENT",
        "replace invalid lineage",
        "--after-review",
        CLOSURE,
    ];
    if recovery != CANDIDATE {
        authorize.extend(["--applicability-record-sha", record_sha.as_str()]);
    }
    decide(&fixture, &authorize);
    let selected = format!(
        ".codeos/06-workflow/reality-evidence/{}.json",
        current_selection(&fixture)
    );
    let mut replace = vec!["--replace-invalid-lineage-after", CLOSURE];
    if recovery != CANDIDATE {
        // Earlier contracts require the caller to declare the F6-selected evidence.
        replace.push(selected.as_str());
    }
    assert_ok(
        governed_review(&fixture, &fake, "success", &replace),
        "lineage replacement",
    );
    let replacement = record_block(
        &review_log(&fixture),
        "REV__FEAT__8__SR2__LINEAGE-REPLACEMENT-1",
    );
    assert!(
        replacement.contains("LINEAGE REPLACEMENT CLOSED — PROGRESSION PERMITTED"),
        "{replacement}"
    );
}

fn current_selection(fixture: &Fixture) -> String {
    let decisions =
        std::fs::read_to_string(fixture.path().join(".codeos/06-workflow/decisions.jsonl"))
            .unwrap();
    decisions
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|receipt| {
            receipt["bindings"]["production_evidence"]
                .as_str()
                .map(str::to_string)
        })
        .last()
        .unwrap()
}

/// Inherited v9 mechanism: evidence reassessment continues DBA-13 infrastructure-revalidation
/// history and completes under v17 with the event-evidence identity derived, not transcribed.
#[test]
fn inherited_evidence_reassessment_completes_under_v17() {
    const R3: &str = "REV__FEAT__8__R3";
    const REENTRY: &str = "REV__FEAT__8__REENTRY-1";
    const CLOSURE: &str = "REV__FEAT__8__EVIDENCE-CLOSURE-1";
    const INFRA: &str = "REV__FEAT__8__INFRA-REVALIDATION-1";
    let fixture = ready_project(false);
    bind(&fixture, "DBA-13", false);
    let fake = setup_fake_codex();
    let step = |label: &str| {
        write(
            &fixture.path().join("src/lib.rs"),
            &format!("pub fn feature() {{ {label}() }}\n"),
        );
        git(fixture.path(), &["commit", "-am", label]);
    };

    // History under DBA-13.
    for (round, mode) in ["success", "success", "blocking"].into_iter().enumerate() {
        step(&format!("round_{round}"));
        assert_ok(
            governed_review(&fixture, &fake, mode, &[]),
            "ordinary round",
        );
    }
    decide(
        &fixture,
        &[
            "decision",
            FEATURE,
            "8",
            "AUTHORIZE_REENTRY",
            "bounded repair",
            "--after-review",
            R3,
        ],
    );
    step("reentry");
    assert_ok(
        governed_review(&fixture, &fake, "blocking", &["--reentry-after", R3]),
        "re-entry",
    );
    let (finding, _) = first_finding_id(&fixture, REENTRY);
    decide(
        &fixture,
        &[
            "decision",
            FEATURE,
            "8",
            "AUTHORIZE_EVIDENCE_CLOSURE",
            "evidence only",
            "--after-review",
            REENTRY,
            "--findings",
            &finding,
        ],
    );
    step("closure");
    assert_ok(
        governed_review(
            &fixture,
            &fake,
            "blocking",
            &["--evidence-closure-after", REENTRY, "--findings", &finding],
        ),
        "evidence closure",
    );
    record_current_evidence(&fixture, false);
    git(fixture.path(), &["add", ".codeos/06-workflow"]);
    git(
        fixture.path(),
        &["commit", "-m", "repaired evidence system"],
    );
    let infra_sha = current_event_sha(&fixture);
    decide(
        &fixture,
        &[
            "decision",
            FEATURE,
            "8",
            "AUTHORIZE_INFRASTRUCTURE_REVALIDATION",
            "evidence system repaired",
            "--after-review",
            CLOSURE,
            "--findings",
            &finding,
            "--event-evidence-sha",
            &infra_sha,
        ],
    );
    step("infra");
    assert_ok(
        governed_review(
            &fixture,
            &fake,
            "blocking",
            &[
                "--infrastructure-revalidation-after",
                CLOSURE,
                "--findings",
                &finding,
                "--event-evidence-sha",
                &infra_sha,
            ],
        ),
        "infrastructure revalidation",
    );
    assert!(review_log(&fixture).contains(&format!("Review ID: {INFRA}")));

    // Continuation under the candidate v17 configuration, with materially new evidence.
    write(
        &fixture.path().join(".codeos/00-project/codeos.yaml"),
        &format!("codeos_schema: 2\ndba_configuration: {CANDIDATE}\n"),
    );
    record_evidence_with_events(
        &fixture,
        false,
        "{\"event\":\"feature_completed\",\"run\":2}\n",
    );
    git(fixture.path(), &["add", ".codeos"]);
    git(
        fixture.path(),
        &["commit", "-m", "bind candidate with new runtime evidence"],
    );
    assert_ok(
        governed_review(
            &fixture,
            &fake,
            "success",
            &[
                "--evidence-reassessment-after",
                INFRA,
                "--findings",
                &finding,
            ],
        ),
        "evidence reassessment",
    );
    let log = review_log(&fixture);
    let reassessment = record_block(&log, "REV__FEAT__8__EVIDENCE-REASSESSMENT-1");
    assert!(
        reassessment.contains(&current_event_sha(&fixture)),
        "{reassessment}"
    );
    assert!(
        !reassessment.contains("Effective concern: DO NOT ADVANCE"),
        "{reassessment}"
    );
}

#[test]
fn runtime_event_verification_is_required_only_when_recorded() {
    let fixture = ready_project(false);
    let store = fixture
        .path()
        .join(".codeos/06-workflow/verifications.jsonl");
    let text = std::fs::read_to_string(&store).unwrap();
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| !line.contains("runtime_event_schema"))
        .collect();
    std::fs::write(&store, kept.join("\n") + "\n").unwrap();
    git(
        fixture.path(),
        &["commit", "-am", "external-observation feature"],
    );
    let fake = setup_fake_codex();
    let (status, stdout, stderr) = review(&fixture, &fake, &[]);
    assert_eq!(status, 0, "{stdout}\n{stderr}");
    let manifest = &sidecar(&fixture)["manifest"];
    assert!(
        manifest["verification"]
            .get("F7/runtime_event_schema")
            .is_none(),
        "{manifest}"
    );
}

#[test]
fn workflow_required_data_integrity_is_required_for_review() {
    let fixture = ready_project(false);
    let contract = fixture.path().join(contract_path());
    let mut text = std::fs::read_to_string(&contract).unwrap();
    text.push_str("\n## Feature Impact Accounting\n\n| Tier | Changed / Unchanged | Reason |\n|---|---|---|\n| Persistence (PostgreSQL) | changed | stores feature state |\n");
    std::fs::write(&contract, text).unwrap();
    record_current_evidence(&fixture, false);
    git(fixture.path(), &["add", "."]);
    git(fixture.path(), &["commit", "-m", "persistence feature"]);
    let fake = setup_fake_codex();
    let result = review(&fixture, &fake, &[]);
    assert!(result.2.contains("F5d/data_integrity"), "{}", result.2);
    assert_not_ready(&fixture, result, "REQUIRED_INPUT_MISSING", &fake);
}

fn reviewed_identity_drift(fixture: &Fixture) -> Option<Vec<String>> {
    let project = Project::discover(fixture.path()).unwrap();
    let record = codeos_workflow::evidence::latest_review_record(
        &fixture.path().join(".codeos/05-review/reviews"),
        FEATURE,
        "8",
    )
    .unwrap()
    .expect("semantic review record");
    codeos_workflow::evidence::review_identity_drift(
        &project.root,
        &project.implementation_paths(),
        &record,
    )
    .unwrap()
}

/// F7 binds a v17 review to its recorded identities, not to repository HEAD.
#[test]
fn v17_review_binding_survives_governance_only_commits() {
    let fixture = ready_project(true);
    let fake = setup_fake_codex();
    let (status, stdout, stderr) = review(&fixture, &fake, &[]);
    assert_eq!(status, 0, "{stdout}\n{stderr}");
    // Committing the review record and a later receipt moves HEAD but changes no reviewed identity.
    std::fs::write(
        fixture.path().join(".git/info/exclude"),
        "/.codeos/toolkit\n",
    )
    .unwrap();
    git(fixture.path(), &["add", ".codeos/05-review"]);
    git(fixture.path(), &["commit", "-m", "record review"]);
    let decisions = fixture.path().join(".codeos/06-workflow/decisions.jsonl");
    let mut text = std::fs::read_to_string(&decisions).unwrap();
    text.push_str("{\"workflow\":\"feature\",\"subject\":\"FEAT\",\"checkpoint\":\"final_ux_validation\",\"result\":\"accepted\",\"bindings\":{},\"timestamp\":\"2026-10-07T12:00:00Z\"}\n");
    std::fs::write(&decisions, text).unwrap();
    git(fixture.path(), &["commit", "-am", "record receipt"]);
    assert_eq!(reviewed_identity_drift(&fixture), Some(vec![]));

    // Any reviewed identity change is drift.
    write(
        &fixture.path().join("src/lib.rs"),
        "pub fn feature() { changed() }\n",
    );
    assert_eq!(
        reviewed_identity_drift(&fixture),
        Some(vec!["implementation".to_string(), "src/lib.rs".to_string()])
    );
}

/// Records without a v17 manifest keep the reviewed-commit (HEAD) binding.
#[test]
fn pre_v17_review_records_keep_head_binding() {
    let fixture = ready_project(false);
    bind(&fixture, "DBA-21", false);
    let fake = setup_fake_codex();
    assert_ok(
        governed_review(&fixture, &fake, "success", &[]),
        "DBA-21 review",
    );
    assert_eq!(reviewed_identity_drift(&fixture), None);
}

fn ux_bindings(
    fixture: &Fixture,
    checkpoint: &str,
) -> (
    std::collections::BTreeMap<String, String>,
    std::collections::BTreeMap<String, String>,
) {
    let project = Project::discover(fixture.path()).unwrap();
    let subject = codeos_workflow::checker::Subject::resolve(
        &project,
        codeos_workflow::contract::Workflow::Feature,
        FEATURE,
    )
    .unwrap();
    (
        codeos_workflow::checker::bindings_for_decision(&project, &subject, checkpoint).unwrap(),
        codeos_workflow::checker::bindings_for_evaluation(&project, &subject, checkpoint).unwrap(),
    )
}

fn f5_bindings(fixture: &Fixture) -> std::collections::BTreeMap<String, String> {
    let project = Project::discover(fixture.path()).unwrap();
    let subject = codeos_workflow::checker::Subject::resolve(
        &project,
        codeos_workflow::contract::Workflow::Feature,
        FEATURE,
    )
    .unwrap();
    codeos_workflow::checker::verification_bindings_for(&project, &subject, "F5").unwrap()
}

/// Under DBA-22, human UX receipts bind the judged product; test files still bind the evidence.
#[test]
fn ux_receipts_ignore_test_only_edits_but_evidence_does_not() {
    use codeos_workflow::hashing::binding_drift;
    let fixture = ready_project(false);
    for checkpoint in ["early_preview", "final_ux_validation"] {
        let (recorded, _) = ux_bindings(&fixture, checkpoint);
        assert!(
            recorded.contains_key("product_state"),
            "{checkpoint}: {recorded:?}"
        );
        assert!(
            !recorded.contains_key("implementation_state"),
            "{checkpoint}"
        );
    }
    let (preview, _) = ux_bindings(&fixture, "early_preview");
    let (ux, _) = ux_bindings(&fixture, "final_ux_validation");
    let evidence = f5_bindings(&fixture);

    write(
        &fixture.path().join("src/summary.spec.ts"),
        "test('x', () => {});\n",
    );
    write(
        &fixture.path().join("src/tests/journey.rs"),
        "#[test] fn j() {}\n",
    );
    assert_eq!(
        binding_drift(&preview, &ux_bindings(&fixture, "early_preview").1),
        None
    );
    assert_eq!(
        binding_drift(&ux, &ux_bindings(&fixture, "final_ux_validation").1),
        None
    );
    assert_eq!(
        binding_drift(&evidence, &f5_bindings(&fixture)).map(|drift| drift.0),
        Some("implementation_state".to_string()),
        "test-only edits must still stale verification evidence"
    );

    write(
        &fixture.path().join("src/lib.rs"),
        "pub fn feature() { changed() }\n",
    );
    assert_eq!(
        binding_drift(&preview, &ux_bindings(&fixture, "early_preview").1).map(|drift| drift.0),
        Some("product_state".to_string())
    );
}

/// Receipts recorded with full implementation binding keep it; earlier configurations are unchanged.
#[test]
fn existing_receipts_and_earlier_configurations_keep_full_binding() {
    use codeos_workflow::hashing::binding_drift;
    let fixture = ready_project(false);
    let (_, evaluation) = ux_bindings(&fixture, "early_preview");
    let mut legacy = evaluation.clone();
    legacy.remove("product_state");
    write(
        &fixture.path().join("src/summary.spec.ts"),
        "test('x', () => {});\n",
    );
    assert_eq!(
        binding_drift(&legacy, &ux_bindings(&fixture, "early_preview").1).map(|drift| drift.0),
        Some("implementation_state".to_string())
    );

    let earlier = ready_project(false);
    bind(&earlier, "DBA-21", false);
    let (recorded, _) = ux_bindings(&earlier, "early_preview");
    assert!(recorded.contains_key("implementation_state"));
    assert!(!recorded.contains_key("product_state"));
}

fn referenced_evidence(fixture: &Fixture) -> PathBuf {
    fixture.path().join(format!(
        ".codeos/06-workflow/reality-evidence/{}.json",
        current_selection(fixture)
    ))
}

/// Production-source records reference evidence by identity; the evidence itself must be present,
/// intact, and inspectable before review.
#[test]
fn referenced_production_evidence_must_be_present_and_intact() {
    let fixture = ready_project(true);
    std::fs::remove_file(referenced_evidence(&fixture)).unwrap();
    let fake = setup_fake_codex();
    let result = review(&fixture, &fake, &[]);
    assert!(
        result
            .2
            .contains("F5/production_source_evidence: referenced production evidence is absent"),
        "{}",
        result.2
    );
    assert_refused_with(&fixture, result, "REQUIRED_INPUT_MISSING", &fake);

    let fixture = ready_project(true);
    std::fs::write(referenced_evidence(&fixture), "{\"tampered\":true}\n").unwrap();
    let fake = setup_fake_codex();
    let result = review(&fixture, &fake, &[]);
    assert!(
        result
            .2
            .contains("referenced production evidence content does not match its identity"),
        "{}",
        result.2
    );
    assert_refused_with(&fixture, result, "F6_SELECTED_EVIDENCE_MISMATCH", &fake);
}

#[test]
fn packet_size_report_separates_derived_and_declared_inputs() {
    let fixture = ready_project(true);
    let (status, stdout, stderr) = run_in_dir_with_env(
        fixture.path(),
        &["plan", FEATURE, "8", "--skip-prechecks", "src/lib.rs"],
        &[],
    );
    assert_eq!(status, 0, "{stdout}\n{stderr}");
    for line in [
        "required inputs (derived):",
        "declared inputs:",
        "total:",
        "budget:",
    ] {
        assert!(stderr.contains(line), "missing {line}:\n{stderr}");
    }
}

#[test]
fn project_reviewer_toml_sets_the_packet_budget() {
    let fixture = ready_project(false);
    write(
        &fixture.path().join(".codeos/05-review/reviewer.toml"),
        "packet_budget_bytes = 204800\n",
    );
    let (status, stdout, stderr) = run_in_dir_with_env(
        fixture.path(),
        &["plan", FEATURE, "8", "--skip-prechecks", "src/lib.rs"],
        &[],
    );
    assert_eq!(status, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("budget 204800"), "{stdout}");
    assert!(
        stderr.contains("budget:                    200 KB"),
        "{stderr}"
    );
}
