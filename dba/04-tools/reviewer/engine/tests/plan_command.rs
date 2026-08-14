//! plan command tests (UPG-0045).
//!
//! `plan` previews what `review` would send — resolved artifacts, evidence mode, packet size
//! vs. budget — without invoking Codex or writing anything. These tests cover: normal output,
//! EMPTY_PACKET reporting, delta mode, sha-only mode, a missing-artifact/precheck-failure case,
//! oversized-packet warning content and no-Codex/no-mutation behavior.

mod common;
use common::{add_extra_commit, binary, repo_root, run_in_dir, setup_temp_git_repo};
use std::process::Command;

/// Create the project-local .codeos directory and toolkit mount, so `packet::build()`
/// can find `dba/03-prompts/review/codeos-reviewer-task.md` via `toolkit_root`. Duplicated from
/// `review_command.rs` (private there, not shared) rather than promoting it into
/// `tests/common/mod.rs`, to keep this change's touched-file scope to this new test file only.
fn setup_codeos_symlink(repo_path: &std::path::Path) {
    let target = repo_root();
    std::fs::create_dir_all(repo_path.join(".codeos")).expect("create .codeos directory");
    std::os::unix::fs::symlink(&target, repo_path.join(".codeos/toolkit"))
        .expect("create toolkit symlink");
    std::fs::write(repo_path.join(".git/info/exclude"), "/.codeos/toolkit\n")
        .expect("ignore toolkit symlink");
}

#[test]
fn smoke_plan_full_mode_basic() {
    let (dir, _base_sha) = setup_temp_git_repo();
    setup_codeos_symlink(dir.path());
    let (code, stdout, stderr) = run_in_dir(
        dir.path(),
        &["plan", "UPG-SMOKE-TEST", "selfdev-step-3", "tracked.md"],
    );
    assert_eq!(
        code, 0,
        "plan on a real tracked artifact should exit 0; stderr: {}",
        stderr
    );
    assert!(stdout.contains("review plan: UPG-SMOKE-TEST selfdev-step-3"));
    assert!(stdout.contains("mode: full"));
    assert!(stdout.contains("coverage: FULL_COVERAGE"));
    assert!(stdout.contains("tracked.md"));
    assert!(stdout.contains("review_content_bytes:"));
    assert!(stdout.contains("estimated_review_tokens:"));
}

#[test]
fn smoke_plan_missing_artifact_exits_packet() {
    let (dir, _base_sha) = setup_temp_git_repo();
    let (code, _stdout, stderr) = run_in_dir(
        dir.path(),
        &[
            "plan",
            "UPG-SMOKE-TEST",
            "selfdev-step-3",
            "does-not-exist.md",
        ],
    );
    assert_eq!(
        code, 4,
        "missing artifact must exit EXIT_PACKET (4); stderr: {}",
        stderr
    );
    assert!(
        stderr.contains("does not resolve"),
        "stderr should explain the missing artifact: {}",
        stderr
    );
}

#[test]
fn smoke_plan_empty_packet_delta_mode_no_diff() {
    // Delta mode against HEAD with no working-tree changes to the artifact -> EMPTY_PACKET.
    let (dir, base_sha) = setup_temp_git_repo();
    setup_codeos_symlink(dir.path());
    let (code, stdout, stderr) = run_in_dir(
        dir.path(),
        &[
            "plan",
            "UPG-SMOKE-TEST",
            "selfdev-step-3",
            "--base",
            &base_sha,
            "tracked.md",
        ],
    );
    assert_eq!(
        code, 4,
        "EMPTY_PACKET must exit EXIT_PACKET (4); stderr: {}",
        stderr
    );
    assert!(
        stdout.contains("EMPTY_PACKET"),
        "plan should report EMPTY_PACKET: {}",
        stdout
    );
}

#[test]
fn smoke_plan_delta_mode_reports_changed_file() {
    let (dir, base_sha) = setup_temp_git_repo();
    setup_codeos_symlink(dir.path());
    // Real change since base_sha.
    add_extra_commit(dir.path(), "tracked.md", "# tracked\nchanged\n");

    let (code, stdout, stderr) = run_in_dir(
        dir.path(),
        &[
            "plan",
            "UPG-SMOKE-TEST",
            "selfdev-step-3",
            "--base",
            &base_sha,
            "tracked.md",
        ],
    );
    assert_eq!(
        code, 0,
        "delta plan with a real change should exit 0; stderr: {}",
        stderr
    );
    assert!(stdout.contains("mode: delta"));
    assert!(stdout.contains(&format!("base: {}", base_sha)));
    assert!(
        stdout.contains("delta_diff"),
        "changed artifact should be reported as delta_diff: {}",
        stdout
    );
}

#[test]
fn smoke_plan_sha_only_mode() {
    let (dir, _base_sha) = setup_temp_git_repo();
    setup_codeos_symlink(dir.path());
    std::fs::write(dir.path().join("other.md"), "# other\ncontent here\n").expect("write other.md");
    Command::new("git")
        .args(["add", "other.md"])
        .current_dir(dir.path())
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "add other"])
        .current_dir(dir.path())
        .output()
        .expect("git commit");

    let (code, stdout, stderr) = run_in_dir(
        dir.path(),
        &[
            "plan",
            "UPG-SMOKE-TEST",
            "selfdev-step-3",
            "--sha-only",
            "tracked.md",
            "other.md",
        ],
    );
    assert_eq!(code, 0, "sha-only plan should exit 0; stderr: {}", stderr);
    assert!(
        stdout.contains("tracked.md (path_sha_only"),
        "sha-only artifact should be reported as path_sha_only: {}",
        stdout
    );
    assert!(
        stdout.contains("other.md (shown"),
        "positional artifact should still be shown in full: {}",
        stdout
    );
}

#[test]
fn smoke_plan_oversized_packet_warning_content() {
    // A very small budget guarantees the real repo's own packet.rs source exceeds it.
    let out = Command::new(binary())
        .args([
            "plan",
            "UPG-SMOKE-TEST",
            "selfdev-step-3",
            "--skip-prechecks",
            "dba/04-tools/reviewer/engine/src/packet.rs",
        ])
        .current_dir(common::repo_root())
        .env("CODEOS_PACKET_BUDGET_BYTES", "1000")
        .output()
        .expect("run plan with tiny budget");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let code = out.status.code().unwrap_or(-1);
    assert_eq!(
        code, 0,
        "over-budget full-coverage plan should still exit 0"
    );
    assert!(
        stdout.contains("WARNING: packet is"),
        "over-budget plan must include a WARNING line: {}",
        stdout
    );
    assert!(
        stdout.contains("largest inputs:"),
        "over-budget plan must rank contributors: {}",
        stdout
    );
    assert!(
        stdout.contains("--base <last-review-commit>"),
        "over-budget plan must suggest the exact delta-mode command: {}",
        stdout
    );
}

#[test]
fn smoke_plan_never_invokes_codex_or_mutates_tree() {
    let (dir, _base_sha) = setup_temp_git_repo();
    setup_codeos_symlink(dir.path());
    // Snapshot AFTER the symlink setup, so the pre-existing untracked `.codeos` entry is
    // present in both `before` and `after` — the assertion below is about what `plan` itself
    // changes, not about the test fixture's own setup.
    let before = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir.path())
        .output()
        .expect("git status before")
        .stdout;

    let (code, stdout, stderr) = run_in_dir(
        dir.path(),
        &["plan", "UPG-SMOKE-TEST", "selfdev-step-3", "tracked.md"],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);

    let after = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir.path())
        .output()
        .expect("git status after")
        .stdout;
    assert_eq!(before, after, "plan must not change the working tree");

    // `review` only prints "review logged:" after a real Codex invocation + log append;
    // `plan` must never reach that code path.
    assert!(
        !stdout.contains("review logged:"),
        "plan must never invoke/log a real review: {}",
        stdout
    );

    assert!(
        !dir.path().join(".codeos/05-review/reviews").exists(),
        "plan must not create durable review records"
    );
    assert!(
        !dir.path().join(".codeos-state").exists(),
        "plan must not create operational state"
    );
}

#[test]
fn smoke_plan_idempotent_output() {
    let (dir, _base_sha) = setup_temp_git_repo();
    setup_codeos_symlink(dir.path());

    let (code1, stdout1, stderr1) = run_in_dir(
        dir.path(),
        &["plan", "UPG-SMOKE-TEST", "selfdev-step-3", "tracked.md"],
    );
    assert_eq!(code1, 0, "stderr: {}", stderr1);

    let (code2, stdout2, stderr2) = run_in_dir(
        dir.path(),
        &["plan", "UPG-SMOKE-TEST", "selfdev-step-3", "tracked.md"],
    );
    assert_eq!(code2, 0, "stderr: {}", stderr2);

    // plan's summary embeds no generation timestamp (unlike the full packet's own
    // `PACKET MANIFEST` section), so two runs with unchanged repo state must be byte-identical.
    assert_eq!(
        stdout1, stdout2,
        "plan output must be idempotent across repeated runs"
    );
}
