//! Executable proof of DBA-17 closure and DBA-18 structural-failure recovery.
//! No real provider or downstream project is used.

mod common;

use codeos_workflow::evidence::terminal_review_closure_history_permits_progression;
use common::{
    binary, repo_root, run_in_dir, run_with_fake_codex, setup_fake_codex, setup_temp_git_repo,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

const FEATURE: &str = "FEAT";
const REVISION: u64 = 2;
const R3: &str = "REV__FEAT__8__SR2__R3";
const REENTRY: &str = "REV__FEAT__8__SR2__REENTRY-1";
const CLOSURE: &str = "REV__FEAT__8__SR2__TERMINAL-CLOSURE-1";
const FAILURE_1: &str = "REV__FEAT__8__SR2__TERMINAL-CLOSURE-FAILURE-1";

fn sha256(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}

fn write(path: &Path, value: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, value).unwrap();
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn commit(repo: &Path, path: &str, content: &str, message: &str) {
    write(&repo.join(path), content);
    git(repo, &["add", path]);
    git(repo, &["commit", "-m", message]);
}

fn setup_project(repo: &Path, dba: &str) -> Vec<String> {
    std::fs::create_dir_all(repo.join(".codeos/00-project")).unwrap();
    std::os::unix::fs::symlink(repo_root(), repo.join(".codeos/toolkit")).unwrap();
    write(
        &repo.join(".git/info/exclude"),
        "/.codeos/toolkit\n/.codeos/05-review/\n",
    );
    write(
        &repo.join(".codeos/00-project/codeos.yaml"),
        &format!("codeos_schema: 2\ndba_configuration: {dba}\n"),
    );

    let artifacts = vec![
        ".codeos/00-project/charter.md".to_string(),
        ".codeos/01-specification/intents/FEAT.md".to_string(),
        ".codeos/01-specification/contracts/FEAT_contract.md".to_string(),
        ".codeos/01-specification/event-schemas/FEAT_schema.md".to_string(),
        "tracked.md".to_string(),
    ];
    write(
        &repo.join(&artifacts[0]),
        "---\nartifact_type: charter\napproval:\n  by: Human\n  at: 2026-09-24\n---\n# Charter\n",
    );
    for (path, kind) in [
        (&artifacts[1], "intent"),
        (&artifacts[2], "contract"),
        (&artifacts[3], "event_schema"),
    ] {
        write(
            &repo.join(path),
            &format!(
                "---\nartifact_type: {kind}\nfeature_id: {FEATURE}\nsemantic_revision: {REVISION}\nsemantic_revision_from: 1\nsemantic_change: repair closure fixture\nstatus: APPROVED\napproved_by: Human\napproved_at: 2026-09-24\n---\n# {kind}\n"
            ),
        );
    }
    git(
        repo,
        &["add", ".codeos/00-project", ".codeos/01-specification"],
    );
    git(repo, &["commit", "-m", "add approved package"]);
    artifacts
}

fn review(
    repo: &Path,
    fake: &common::FakeCodex,
    artifacts: &[String],
    mode: &str,
    special: &[&str],
) -> String {
    let mut args = vec!["review", FEATURE, "8", "--skip-prechecks"];
    args.extend_from_slice(special);
    args.extend(artifacts.iter().map(String::as_str));
    let (code, stdout, stderr) = run_with_fake_codex(repo, fake, &args, mode);
    assert_eq!(code, 0, "review failed:\n{stdout}\n{stderr}");
    stdout
}

fn decision(repo: &Path, args: &[&str]) {
    let output = Command::new(binary())
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "decision failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn record_block<'a>(log: &'a str, review_id: &str) -> &'a str {
    log.split_once(&format!("Review ID: {review_id}"))
        .unwrap()
        .1
        .split("\n## ")
        .next()
        .unwrap()
}

fn line_value<'a>(block: &'a str, prefix: &str) -> &'a str {
    block
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .unwrap()
}

fn referenced_path(block: &str, prefix: &str) -> PathBuf {
    PathBuf::from(
        line_value(block, prefix)
            .rsplit_once(" (sha256:")
            .unwrap()
            .0,
    )
}

#[test]
fn structural_failure_retries_exact_payload_through_the_complete_executable_chain() {
    let (temp, _) = setup_temp_git_repo();
    let repo = temp.path();
    let fake = setup_fake_codex();
    let artifacts = setup_project(repo, "DBA-18");

    review(repo, &fake, &artifacts, "success", &[]);
    commit(
        repo,
        "tracked.md",
        "# tracked\nround two\n",
        "prepare round two",
    );
    review(repo, &fake, &artifacts, "success", &[]);
    commit(
        repo,
        "tracked.md",
        "# tracked\nround three\n",
        "prepare round three",
    );
    review(repo, &fake, &artifacts, "blocking", &[]);

    decision(
        repo,
        &[
            "decision",
            FEATURE,
            "8",
            "AUTHORIZE_REENTRY",
            "One bounded repair review.",
            "--after-review",
            R3,
        ],
    );
    commit(
        repo,
        "tracked.md",
        "# tracked\nre-entry candidate\n",
        "prepare re-entry candidate",
    );
    review(
        repo,
        &fake,
        &artifacts,
        "blocking",
        &["--reentry-after", R3],
    );

    let log_path = repo.join(".codeos/05-review/reviews/review-log.md");
    let log = std::fs::read_to_string(&log_path).unwrap();
    let terminal = record_block(&log, REENTRY);
    let assessment_path = referenced_path(terminal, "Full assessment: ");
    let assessment = std::fs::read_to_string(&assessment_path).unwrap();
    let assessment_yaml: Value = serde_yaml::from_str(
        assessment
            .strip_prefix("---\n")
            .unwrap()
            .split_once("\n---")
            .unwrap()
            .0,
    )
    .unwrap();
    let finding_id = assessment_yaml["findings"][0]["finding_id"]
        .as_str()
        .unwrap_or_else(|| panic!("terminal assessment has no parsed finding:\n{assessment}"));
    let assessment_sha = sha256(&assessment);
    let control = tempfile::tempdir().unwrap();
    let scope_path = control.path().join("scope.json");
    write(
        &scope_path,
        &serde_json::json!({
            "terminal_review_id": REENTRY,
            "assessment_sha256": assessment_sha,
            "conditions": [{
                "id": "C1",
                "description": "Repair the terminal implementation defect",
                "source": {"kind": "parsed_finding", "finding_id": finding_id}
            }]
        })
        .to_string(),
    );
    decision(
        repo,
        &[
            "decision",
            FEATURE,
            "8",
            "AUTHORIZE_TERMINAL_REVIEW_CLOSURE",
            "Assess only the repaired terminal condition.",
            "--after-review",
            REENTRY,
            "--closure-scope",
            scope_path.to_str().unwrap(),
        ],
    );

    let original_review = line_value(terminal, "Base: ")
        .split("  Review: ")
        .nth(1)
        .unwrap()
        .split("  Branch:")
        .next()
        .unwrap();
    let original_diff = line_value(terminal, "Diff-hash: ");
    commit(
        repo,
        "tracked.md",
        "# tracked\nrepaired implementation\n",
        "repair terminal condition",
    );
    commit(
        repo,
        "fresh-evidence.txt",
        "fresh proof for repaired implementation\n",
        "record fresh repair evidence",
    );
    write(
        &repo.join("unrelated-untracked.txt"),
        "must not enter a bounded closure packet\n",
    );

    let packet_path = control.path().join("closure.packet.txt");
    let mut plan_args = vec![
        "plan",
        FEATURE,
        "8",
        "--skip-prechecks",
        "--emit-packet",
        packet_path.to_str().unwrap(),
        "--terminal-review-closure-after",
        REENTRY,
    ];
    let mut closure_artifacts = artifacts.clone();
    closure_artifacts.push("fresh-evidence.txt".into());
    plan_args.extend(closure_artifacts.iter().map(String::as_str));
    let (code, _, stderr) = run_in_dir(repo, &plan_args);
    assert_eq!(code, 0, "closure plan failed: {stderr}");
    let closure_packet = std::fs::read_to_string(&packet_path).unwrap();
    assert!(!closure_packet.contains("unrelated-untracked.txt"));
    assert!(closure_packet.contains("DELTA DIFF"));
    assert!(closure_packet.contains("fresh-evidence.txt (mode: added_file"));
    let sidecar: Value = serde_json::from_str(
        &std::fs::read_to_string(format!("{}.meta.json", packet_path.display())).unwrap(),
    )
    .unwrap();
    let evidence_sha = sidecar["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|artifact| artifact["path"] == "fresh-evidence.txt")
        .unwrap()["sha256"]
        .as_str()
        .unwrap();
    let manifest_path = control.path().join("repair-manifest.json");
    write(
        &manifest_path,
        &serde_json::json!({
            "terminal_review_id": REENTRY,
            "original_implementation_identity": format!("{original_review}:{original_diff}"),
            "current_implementation_identity": format!(
                "{}:{}",
                sidecar["review_sha"].as_str().unwrap(),
                sidecar["diff_hash"].as_str().unwrap()
            ),
            "changed_paths": ["fresh-evidence.txt", "tracked.md"],
            "evidence_identities": [evidence_sha],
            "links": [{
                "condition_id": "C1",
                "changed_paths": ["fresh-evidence.txt", "tracked.md"],
                "evidence_identities": [evidence_sha],
                "rationale": "The implementation repair and its fresh evidence close C1."
            }]
        })
        .to_string(),
    );
    review(
        repo,
        &fake,
        &closure_artifacts,
        "structural_failure",
        &[
            "--terminal-review-closure-after",
            REENTRY,
            "--repair-manifest",
            manifest_path.to_str().unwrap(),
        ],
    );

    let failed_log = std::fs::read_to_string(&log_path).unwrap();
    let failed = record_block(&failed_log, FAILURE_1);
    assert!(failed.contains("Closure execution status: STRUCTURAL FAILURE"));
    assert!(failed.contains("Retry status: ONE BOUNDED BRIDGE RETRY AVAILABLE"));
    assert!(!failed_log.contains(&format!("Review ID: {CLOSURE}")));

    let (code, _, stderr) = run_with_fake_codex(
        repo,
        &fake,
        &[
            "review",
            FEATURE,
            "8",
            "--retry-terminal-review-closure-after",
            FAILURE_1,
            "tracked.md",
        ],
        "success",
    );
    assert_ne!(code, 0, "retry accepted replacement review inputs");
    assert!(
        stderr.contains("accepts no general replacement"),
        "{stderr}"
    );

    let (code, stdout, stderr) = run_with_fake_codex(
        repo,
        &fake,
        &[
            "review",
            FEATURE,
            "8",
            "--retry-terminal-review-closure-after",
            FAILURE_1,
        ],
        "success",
    );
    assert_eq!(code, 0, "retry failed:\n{stdout}\n{stderr}");

    let final_log = std::fs::read_to_string(&log_path).unwrap();
    for required in [
        R3,
        "Decision: AUTHORIZE_REENTRY",
        REENTRY,
        "Decision: AUTHORIZE_TERMINAL_REVIEW_CLOSURE",
        FAILURE_1,
        CLOSURE,
        "Effective concern: TERMINAL REVIEW CLOSED — PROGRESSION PERMITTED",
    ] {
        assert!(
            final_log.contains(required),
            "missing {required}:\n{final_log}"
        );
    }
    assert!(
        terminal_review_closure_history_permits_progression(&log_path, FEATURE, "8", CLOSURE)
            .unwrap(),
        "the exact composite history consumed by F7 did not permit progression"
    );
    let (code, _, stderr) = run_with_fake_codex(
        repo,
        &fake,
        &[
            "review",
            FEATURE,
            "8",
            "--retry-terminal-review-closure-after",
            FAILURE_1,
        ],
        "success",
    );
    assert_ne!(code, 0, "a second retry unexpectedly succeeded");
    assert!(stderr.contains("already been consumed"), "{stderr}");
}
