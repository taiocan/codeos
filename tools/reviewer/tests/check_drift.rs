//! check-drift command tests.
//!
//! Tests for the stack/dependency drift detection command.

mod common;
use common::{setup_temp_git_repo, add_extra_commit, run_in_dir};
use std::process::Command;

#[test]
fn smoke_check_drift_clean_diff_exits_zero() {
    // AC-1: no watched files in diff → exit 0.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    // Add a commit with a non-watched file only.
    add_extra_commit(p, "notes.md", "# notes\n");

    let (code, _, stderr) = run_in_dir(p, &["check-drift", "--base", &base_sha]);
    assert_eq!(code, 0, "clean diff must exit 0; stderr: {}", stderr);
}

#[test]
fn smoke_check_drift_watched_file_no_report_exits_drift() {
    // AC-2: watched file changed, no reconciliation report → exit 6 (EXIT_DRIFT).
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    add_extra_commit(p, "Cargo.toml", "[package]\nname = \"test\"\n");

    let (code, _, stderr) = run_in_dir(p, &["check-drift", "--base", &base_sha]);
    assert_eq!(code, 6, "drift without report must exit 6; stderr: {}", stderr);
    assert!(stderr.contains("Cargo.toml"), "stderr must name triggering file; got: {}", stderr);
    assert!(
        stderr.contains("stack-reconciliation-report.md"),
        "stderr must name the required report; got: {}", stderr
    );
}

#[test]
fn smoke_check_drift_watched_file_with_report_exits_zero() {
    // AC-3: watched file changed + reconciliation report in diff → exit 0.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    // Commit both Cargo.toml and the reconciliation report in one go.
    std::fs::write(p.join("Cargo.toml"), "[package]\nname = \"test\"\n").expect("write Cargo.toml");
    std::fs::write(p.join("stack-reconciliation-report.md"), "# Stack Reconciliation Report\n").expect("write report");
    Command::new("git").args(["add", "Cargo.toml", "stack-reconciliation-report.md"]).current_dir(p).output().ok();
    Command::new("git").args(["commit", "-m", "drift+report"]).current_dir(p).output().ok();

    let (code, _, stderr) = run_in_dir(p, &["check-drift", "--base", &base_sha]);
    assert_eq!(code, 0, "reconciled drift must exit 0; stderr: {}", stderr);
}

#[test]
fn smoke_check_drift_invalid_base_exits_config() {
    // AC-4: git diff fails on an invalid base ref → exit 2 (EXIT_CONFIG).
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();

    let (code, _, stderr) = run_in_dir(p, &["check-drift", "--base", "nonexistent-ref-xyz"]);
    assert_eq!(code, 2, "invalid base ref must exit 2 (EXIT_CONFIG); stderr: {}", stderr);
}

#[test]
fn smoke_check_drift_strict_flag_accepted() {
    // AC-8: --strict is accepted; drift message prefixed with STRICT MODE.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    add_extra_commit(p, "Cargo.toml", "[package]\nname = \"test\"\n");

    let (code, _, stderr) = run_in_dir(p, &["check-drift", "--base", &base_sha, "--strict"]);
    assert_eq!(code, 6, "--strict drift must still exit 6; stderr: {}", stderr);
    assert!(stderr.contains("STRICT MODE"), "--strict must prefix message; stderr: {}", stderr);
}
