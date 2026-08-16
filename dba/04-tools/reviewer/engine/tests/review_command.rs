//! Reviewer interface tests: evidence validation, path safety, and the deliberately small CLI.

mod common;
use common::{add_extra_commit, binary, repo_root, run, run_in_dir, setup_temp_git_repo};
use std::process::Command;

fn setup_codeos_symlink(repo_path: &std::path::Path) {
    std::fs::create_dir_all(repo_path.join(".codeos")).expect("create .codeos directory");
    std::os::unix::fs::symlink(repo_root(), repo_path.join(".codeos/toolkit"))
        .expect("create toolkit symlink");
    std::fs::write(repo_path.join(".git/info/exclude"), "/.codeos/toolkit\n")
        .expect("ignore toolkit symlink");
}

#[test]
fn review_requires_evidence_arguments() {
    let (code, _, stderr) = run(&["review"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("required"));
}

#[test]
fn plan_rejects_missing_artifact() {
    let (dir, _) = setup_temp_git_repo();
    let (code, _, stderr) = run_in_dir(
        dir.path(),
        &["plan", "UPG-SMOKE", "selfdev-step-1", "missing.md"],
    );
    assert_eq!(code, 4);
    assert!(stderr.contains("does not resolve"), "{stderr}");
}

#[test]
fn delta_rejects_untracked_artifact() {
    let (dir, base) = setup_temp_git_repo();
    setup_codeos_symlink(dir.path());
    std::fs::write(dir.path().join("untracked.md"), "# untracked\n").unwrap();

    let (code, _, stderr) = run_in_dir(
        dir.path(),
        &[
            "plan",
            "UPG-SMOKE",
            "selfdev-step-1",
            "--base",
            &base,
            "untracked.md",
        ],
    );
    assert_eq!(code, 4);
    assert!(stderr.contains("untracked"), "{stderr}");
}

#[test]
fn base_ref_is_resolved_to_a_commit() {
    let (dir, base) = setup_temp_git_repo();
    setup_codeos_symlink(dir.path());
    add_extra_commit(dir.path(), "tracked.md", "# tracked\nchanged\n");

    let (code, stdout, stderr) = run_in_dir(
        dir.path(),
        &[
            "plan",
            "UPG-SMOKE",
            "selfdev-step-1",
            "--base",
            "HEAD~1",
            "tracked.md",
        ],
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains(&format!("base: {base}")), "{stdout}");
    assert!(stdout.contains("delta_diff"), "{stdout}");
}

#[test]
fn sha_only_and_shown_artifacts_remain_distinct() {
    let (dir, _) = setup_temp_git_repo();
    setup_codeos_symlink(dir.path());
    std::fs::write(dir.path().join("other.md"), "# other\n").unwrap();
    Command::new("git")
        .args(["add", "other.md"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "other"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let (code, stdout, stderr) = run_in_dir(
        dir.path(),
        &[
            "plan",
            "UPG-SMOKE",
            "selfdev-step-1",
            "--sha-only",
            "tracked.md",
            "other.md",
        ],
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("tracked.md (path_sha_only"), "{stdout}");
    assert!(stdout.contains("other.md (shown"), "{stdout}");
}

#[test]
fn resolved_internal_symlink_is_allowed() {
    let (dir, _) = setup_temp_git_repo();
    setup_codeos_symlink(dir.path());
    std::os::unix::fs::symlink("tracked.md", dir.path().join("alias.md")).unwrap();

    let (code, stdout, stderr) = run_in_dir(
        dir.path(),
        &["plan", "UPG-SMOKE", "selfdev-step-1", "alias.md"],
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("tracked.md (shown"), "{stdout}");
}

#[test]
fn resolved_external_symlink_is_rejected() {
    let (dir, _) = setup_temp_git_repo();
    std::os::unix::fs::symlink("/etc/hosts", dir.path().join("escape.md")).unwrap();

    let (code, _, stderr) = run_in_dir(
        dir.path(),
        &["plan", "UPG-SMOKE", "selfdev-step-1", "escape.md"],
    );
    assert_eq!(code, 4);
    assert!(stderr.contains("outside the repository"), "{stderr}");
}

#[test]
fn directory_artifact_is_rejected() {
    let (dir, _) = setup_temp_git_repo();
    std::fs::create_dir(dir.path().join("evidence")).unwrap();
    let (code, _, stderr) = run_in_dir(
        dir.path(),
        &["plan", "UPG-SMOKE", "selfdev-step-1", "evidence"],
    );
    assert_eq!(code, 4);
    assert!(stderr.contains("regular file"), "{stderr}");
}

#[test]
fn identifiers_reject_path_syntax() {
    let (dir, _) = setup_temp_git_repo();
    let (code, _, stderr) = run_in_dir(
        dir.path(),
        &["plan", "../UPG-SMOKE", "selfdev-step-1", "tracked.md"],
    );
    assert_eq!(code, 1);
    assert!(stderr.contains("feature must start"), "{stderr}");
}

#[test]
fn removed_review_flags_are_unknown() {
    for flag in ["--mode", "--print-packet", "--dry-run"] {
        let out = Command::new(binary())
            .args(["review", "UPG-SMOKE", "selfdev-step-1", flag])
            .current_dir(repo_root())
            .output()
            .unwrap();
        assert!(
            !out.status.success(),
            "retired flag remained accepted: {flag}"
        );
        assert!(String::from_utf8_lossy(&out.stderr).contains("unexpected argument"));
    }
}

#[test]
fn review_help_exposes_only_current_evidence_controls() {
    let (code, stdout, stderr) = run(&["review", "--help"]);
    assert_eq!(code, 0, "{stderr}");
    for current in ["--base", "--sha-only", "--guard-clean", "--fresh"] {
        assert!(
            stdout.contains(current),
            "missing current option {current}: {stdout}"
        );
    }
    assert!(
        stdout.contains("<WORKFLOW_OR_STAGE>"),
        "review target is still presented as stage-only: {stdout}"
    );
    for retired in ["--mode", "--print-packet", "--dry-run", "--provider"] {
        assert!(
            !stdout.contains(retired),
            "retired option remains in help: {retired}"
        );
    }
}
