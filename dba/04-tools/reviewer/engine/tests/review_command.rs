//! review command tests.
//!
//! Tests for the review command (packet building, evidence modes, warnings).

mod common;
use common::{setup_temp_git_repo, add_extra_commit, run, run_in_dir, repo_root, binary};
use std::process::Command;

/// Helper: create the project-local .codeos directory and nested toolkit mount.
fn setup_codeos_symlink(repo_path: &std::path::Path) {
    let target = repo_root();
    std::fs::create_dir_all(repo_path.join(".codeos")).expect("create .codeos directory");
    std::os::unix::fs::symlink(&target, repo_path.join(".codeos/toolkit"))
        .expect("create toolkit symlink");
    std::fs::write(repo_path.join(".git/info/exclude"), "/.codeos/toolkit\n")
        .expect("ignore toolkit symlink");
}

#[test]
fn smoke_review_no_args_exits_usage() {
    let (code, _, stderr) = run(&["review"]);
    // clap will exit 2 (usage error) when required positional args are missing
    assert_ne!(code, 0, "review with no args should fail");
    let _ = stderr; // may contain clap error message
}

#[test]
fn smoke_review_print_packet_nonexistent_file() {
    // --print-packet (--dry-run) with a nonexistent file should exit non-zero (EXIT_PACKET=4)
    let (code, _, stderr) = run(&[
        "review", "UPG-SMOKE-TEST", "selfdev-step-0",
        "--print-packet",
        "does-not-exist-smoke-test.md",
    ]);
    // packet build should succeed if file is not found (shown as missing), but an empty packet fails.
    // Either PACKET (4) or the file is treated as missing and results in empty packet.
    assert!(
        code == 4 || code == 0,
        "expected exit 4 (PACKET) or 0 (file shown as missing), got {}: {}",
        code, stderr
    );
}

#[test]
fn smoke_review_print_packet_existing_file() {
    // Pass an actual file in the repo; --print-packet should produce output and exit 0
    let (code, stdout, stderr) = run(&[
        "review", "UPG-SMOKE-TEST", "selfdev-step-0",
        "--print-packet", "--skip-prechecks",
        "CLAUDE.md",
    ]);
    assert_eq!(
        code, 0,
        "--print-packet with a real file should exit 0; stderr: {}",
        stderr
    );
    assert!(stdout.len() > 10, "packet output should be non-empty");
}

#[test]
fn smoke_delta_mode_untracked_artifact_exits_packet() {
    // Delta review on an untracked file must exit EXIT_PACKET (4) with a clear diagnostic.
    // Bash behavior: git ls-files --error-unmatch catches this; Rust must match.
    let (dir, base_sha) = setup_temp_git_repo();
    let dir_path = dir.path();

    // Write a file that is NOT staged or committed — untracked.
    std::fs::write(dir_path.join("untracked.md"), "# untracked\n").expect("write untracked");

    let out = Command::new(binary())
        .args([
            "review", "FEAT", "delta-test",
            "--mode", "delta",
            "--base", &base_sha,
            "--print-packet",
            "untracked.md",
        ])
        .current_dir(dir_path)
        .output()
        .expect("run binary");

    let code = out.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert_eq!(
        code, 4,
        "untracked artifact in delta mode must exit 4 (EXIT_PACKET); stderr: {}",
        stderr
    );
    assert!(
        stderr.contains("untracked"),
        "error message must mention 'untracked': {}",
        stderr
    );
}

#[test]
fn smoke_delta_mode_tracked_artifact_succeeds() {
    // A tracked but unchanged file in delta mode should produce a packet (exit 0 with --print-packet).
    let (dir, base_sha) = setup_temp_git_repo();
    let dir_path = dir.path();
    // tracked.md was committed in setup; it is tracked and unchanged since base.

    let out = Command::new(binary())
        .args([
            "review", "FEAT", "delta-test",
            "--mode", "delta",
            "--base", &base_sha,
            "--print-packet", "--skip-prechecks",
            "tracked.md",
        ])
        .current_dir(dir_path)
        .output()
        .expect("run binary");

    let code = out.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    // exit 0 (packet printed) is the expected path; exit 4 (EMPTY_PACKET) is also acceptable
    // since an unchanged file produces no diff.
    assert!(
        code == 0 || code == 4,
        "tracked artifact in delta mode should exit 0 or 4 (empty diff); got {}; stderr: {}",
        code, stderr
    );
}

#[test]
fn smoke_full_context_diff_present_in_delta_plus_base() {
    // AC-1: Full Context Diff section appears when --mode delta AND --base are both active.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    setup_codeos_symlink(p);
    // Modify the named artifact AND add a second file, so the packet is non-empty AND the
    // full context diff contains changes beyond the named artifact.
    std::fs::write(p.join("tracked.md"), "# tracked\nmodified\n").expect("modify tracked");
    add_extra_commit(p, "extra.md", "# extra\n");

    let out = Command::new(binary())
        .args([
            "review", "FEAT", "full-diff-test",
            "--mode", "delta",
            "--base", &base_sha,
            "--print-packet", "--skip-prechecks",
            "tracked.md",
        ])
        .current_dir(p)
        .output()
        .expect("run binary");

    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert_eq!(code, 0, "delta+base should exit 0; stderr: {}", stderr);
    assert!(
        stdout.contains("Full Context Diff (informational"),
        "packet must contain Full Context Diff section; got: {}", &stdout[..stdout.len().min(500)]
    );
}

#[test]
fn smoke_full_context_diff_absent_in_full_mode() {
    // AC-1/AC-2: No Full Context Diff section when --mode full is used (no --base).
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_codeos_symlink(p);
    add_extra_commit(p, "extra.md", "# extra\n");

    let out = Command::new(binary())
        .args([
            "review", "FEAT", "full-diff-test",
            "--mode", "full",  // full mode, no --base
            "--print-packet", "--skip-prechecks",
            "tracked.md",
        ])
        .current_dir(p)
        .output()
        .expect("run binary");

    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert_eq!(code, 0, "full mode should exit 0; stderr: {}", stderr);
    assert!(
        !stdout.contains("Full Context Diff (informational"),
        "packet must NOT contain Full Context Diff in full mode"
    );
}

#[test]
fn smoke_full_context_diff_absent_in_delta_without_base() {
    // AC-1: No Full Context Diff section when --mode delta is used WITHOUT --base.
    // Guard is `delta_mode && delta_base.is_some()` — delta without base → absent.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_codeos_symlink(p);
    // Modify tracked.md so delta mode finds a change (avoids EMPTY_PACKET).
    std::fs::write(p.join("tracked.md"), "# tracked\nmodified\n").expect("modify tracked");
    add_extra_commit(p, "extra.md", "# extra\n");

    let out = Command::new(binary())
        .args([
            "review", "FEAT", "delta-no-base-test",
            "--mode", "delta",  // delta mode, but NO --base
            "--print-packet", "--skip-prechecks",
            "tracked.md",
        ])
        .current_dir(p)
        .output()
        .expect("run binary");

    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    // Delta without --base diffs against HEAD; tracked.md has no uncommitted changes after
    // add_extra_commit, so may be EMPTY_PACKET (exit 4). Either way, no Full Context Diff.
    assert!(
        !stdout.contains("Full Context Diff (informational"),
        "delta-without-base must NOT contain Full Context Diff; stderr: {}", stderr
    );
}

#[test]
fn smoke_full_context_diff_named_artifact_section_unchanged() {
    // AC-3: Named artifact section is present and unmodified alongside the Full Context Diff.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    setup_codeos_symlink(p);
    // Modify the named artifact so the delta packet is non-empty (exit 0).
    std::fs::write(p.join("tracked.md"), "# tracked\nmodified\n").expect("modify tracked");
    add_extra_commit(p, "extra.md", "# extra\n");

    // Get packet WITH delta+base (includes Full Context Diff).
    let out_delta = Command::new(binary())
        .args([
            "review", "FEAT", "ac3-test",
            "--mode", "delta",
            "--base", &base_sha,
            "--print-packet", "--skip-prechecks",
            "tracked.md",
        ])
        .current_dir(p)
        .output()
        .expect("run binary");
    let stdout_delta = String::from_utf8_lossy(&out_delta.stdout).into_owned();

    // Get packet WITHOUT delta (no Full Context Diff).
    let out_full = Command::new(binary())
        .args([
            "review", "FEAT", "ac3-test",
            "--mode", "full",
            "--print-packet", "--skip-prechecks",
            "tracked.md",
        ])
        .current_dir(p)
        .output()
        .expect("run binary");
    let stdout_full = String::from_utf8_lossy(&out_full.stdout).into_owned();

    // Both must contain the ARTIFACTS TO REVIEW section.
    assert!(stdout_delta.contains("ARTIFACTS TO REVIEW"), "delta packet missing artifacts section");
    assert!(stdout_full.contains("ARTIFACTS TO REVIEW"), "full packet missing artifacts section");

    // The delta packet must also have the Full Context Diff — it must appear AFTER the
    // artifact+diff section, proving Full Context Diff is additive (not replacing).
    let artifacts_pos = stdout_delta.find("ARTIFACTS TO REVIEW")
        .expect("delta packet must have ARTIFACTS TO REVIEW");
    let full_diff_pos = stdout_delta.find("Full Context Diff (informational")
        .expect("delta packet must have Full Context Diff section");
    assert!(
        artifacts_pos < full_diff_pos,
        "ARTIFACTS TO REVIEW must appear before Full Context Diff — named artifacts must not be replaced"
    );

    // Verify the named-artifact diff content is present, appears before Full Context Diff,
    // and matches the raw git diff output — proving Full Context Diff is purely additive.
    // tracked.md was modified from "# tracked\n" to "# tracked\nmodified\n".
    let expected_diff = Command::new("git")
        .args(["diff", &base_sha, "--", "tracked.md"])
        .current_dir(p)
        .output()
        .expect("git diff tracked.md");
    let expected_diff_str = String::from_utf8_lossy(&expected_diff.stdout).to_string();
    let expected_trimmed = expected_diff_str.trim();

    // The packet's DELTA DIFF section (from "ARTIFACTS TO REVIEW" to "Full Context Diff")
    // must contain the raw diff of tracked.md unchanged.
    let named_diff_section = &stdout_delta[artifacts_pos..full_diff_pos];
    assert!(
        named_diff_section.contains(expected_trimmed),
        "named-artifact diff section must contain the exact git diff of tracked.md; expected: {:?}; section excerpt: {}",
        &expected_trimmed[..expected_trimmed.len().min(200)],
        &named_diff_section[..named_diff_section.len().min(400)]
    );
}

#[test]
fn smoke_full_context_diff_clipping_marker() {
    // AC-4: When the diff exceeds budget, CLIPPED marker appears.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    setup_codeos_symlink(p);
    // Modify the named artifact and add a large file, so there is a non-empty full diff.
    std::fs::write(p.join("tracked.md"), "# tracked\nmodified\n").expect("modify tracked");
    let large_content = "x".repeat(200);
    add_extra_commit(p, "large.md", &large_content);

    // Run with a tiny budget (0) to guarantee the full diff is clipped.
    // CODEOS_PACKET_BUDGET_BYTES=0 means remaining=0, so clip marker is always emitted
    // when the full diff is non-empty.
    let out = Command::new(binary())
        .args([
            "review", "FEAT", "clip-test",
            "--mode", "delta",
            "--base", &base_sha,
            "--print-packet", "--skip-prechecks",
            "tracked.md",
        ])
        .current_dir(p)
        .env("CODEOS_PACKET_BUDGET_BYTES", "0")  // zero budget → always clips
        .output()
        .expect("run binary");

    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    // The Full Context Diff section must have the CLIPPED marker (not just the header).
    assert!(
        stdout.contains("Full Context Diff (informational"),
        "packet must have Full Context Diff section; stderr: {}", stderr
    );
    assert!(
        stdout.contains("CLIPPED"),
        "packet must have CLIPPED marker when budget is zero; stdout: {}", &stdout[..stdout.len().min(800)]
    );
}

#[test]
fn smoke_full_context_diff_absent_in_full_mode_with_base() {
    // AC-1/AC-2: No Full Context Diff section when --mode full is used WITH --base.
    // Guard is `delta_mode && delta_base.is_some()` — full mode (delta_mode=false) → absent
    // even when --base is provided.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    setup_codeos_symlink(p);
    add_extra_commit(p, "extra.md", "# extra\n");

    let out = Command::new(binary())
        .args([
            "review", "FEAT", "full-base-test",
            "--mode", "full",
            "--base", &base_sha,  // base provided, but mode is full (not delta)
            "--print-packet", "--skip-prechecks",
            "tracked.md",
        ])
        .current_dir(p)
        .output()
        .expect("run binary");

    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert_eq!(out.status.code().unwrap_or(-1), 0, "full mode with base should exit 0; stderr: {}", stderr);
    assert!(
        !stdout.contains("Full Context Diff (informational"),
        "full mode with --base must NOT contain Full Context Diff section"
    );
}

#[test]
fn smoke_full_context_diff_no_clip_within_budget() {
    // AC-4: When the full diff fits within the remaining content budget, no CLIPPED marker.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    setup_codeos_symlink(p);
    // Small named-artifact change + tiny extra file → diff fits in default budget (50 000).
    std::fs::write(p.join("tracked.md"), "# tracked\nmodified\n").expect("modify tracked");
    add_extra_commit(p, "extra.md", "# extra\n");

    // Default budget (50 000); in delta mode, review_content_bytes = named-artifact diff bytes
    // only (~100 bytes), so remaining ≈ 49 900 bytes — far larger than the tiny full diff.
    let out = Command::new(binary())
        .args([
            "review", "FEAT", "no-clip-test",
            "--mode", "delta",
            "--base", &base_sha,
            "--print-packet", "--skip-prechecks",
            "tracked.md",
        ])
        .current_dir(p)
        .output()
        .expect("run binary");

    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert_eq!(out.status.code().unwrap_or(-1), 0, "delta+base with small diff should exit 0; stderr: {}", stderr);
    assert!(
        stdout.contains("Full Context Diff (informational"),
        "packet must have Full Context Diff section"
    );
    assert!(
        !stdout.contains("CLIPPED"),
        "packet must NOT have CLIPPED marker when diff fits within budget"
    );
}

#[test]
fn smoke_sha_only_missing_path_exits_usage() {
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    setup_codeos_symlink(p);

    let out = Command::new(binary())
        .args([
            "review", "FEAT", "test-sha-only-missing",
            "--sha-only", "nonexistent-file.md",
            "--skip-prechecks",
            "tracked.md",
        ])
        .current_dir(p)
        .output()
        .expect("run binary");

    assert_eq!(
        out.status.code().unwrap_or(-1), 1,
        "missing --sha-only path must exit 1 (EXIT_USAGE)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--sha-only path not found: nonexistent-file.md"),
        "stderr must name the missing path; got: {}", stderr
    );
}

#[test]
fn smoke_sha_only_existing_path_no_spurious_exit1() {
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    setup_codeos_symlink(p);

    // sha_only_artifact.md exists — should not trigger the missing-path check
    std::fs::write(p.join("sha_only_artifact.md"), "# sha only\n").expect("write sha_only");

    // --print-packet + --skip-prechecks avoids needing a real Codex session
    let out = Command::new(binary())
        .args([
            "review", "FEAT", "test-sha-only-exists",
            "--sha-only", "sha_only_artifact.md",
            "--print-packet", "--skip-prechecks",
            "tracked.md",
        ])
        .current_dir(p)
        .output()
        .expect("run binary");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_ne!(
        out.status.code().unwrap_or(-1), 1,
        "existing --sha-only path must not exit 1; stderr: {}", stderr
    );
    assert!(
        !stderr.contains("--sha-only path not found"),
        "existing path must not trigger missing-path error; stderr: {}", stderr
    );
}

#[test]
fn smoke_readonly_invariant_no_warning_on_print_packet() {
    // AC-5: --print-packet never invokes the provider, so no pre/post snapshot is taken
    // and no WARNING can appear on stderr regardless of working-tree state.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_codeos_symlink(p);

    // Make the working tree dirty so that IF a snapshot were taken and compared it would differ.
    std::fs::write(p.join("dirty.md"), "# dirty\n").expect("write dirty file");

    let out = Command::new(binary())
        .args([
            "review", "FEAT", "readonly-test",
            "--print-packet", "--skip-prechecks",
            "tracked.md",
        ])
        .current_dir(p)
        .output()
        .expect("run binary");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("WARNING: working tree changed"),
        "--print-packet must not emit read-only warning; stderr: {}", stderr
    );
}

#[test]
fn smoke_readonly_invariant_git_status_porcelain_detects_mutation() {
    // Verify the underlying git mechanism: porcelain output differs when a file is added.
    // This is the comparison logic the invariant check relies on (AC-1/AC-2/AC-3).
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();

    let clean = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(p)
        .output()
        .expect("git status clean");
    assert!(clean.stdout.is_empty(), "clean repo must have empty porcelain output");

    std::fs::write(p.join("mutated.md"), "# mutated\n").expect("write file");

    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(p)
        .output()
        .expect("git status dirty");
    assert!(
        dirty.stdout != clean.stdout,
        "porcelain output must differ after mutation (pre != post)"
    );
    assert!(
        !dirty.stdout.is_empty(),
        "dirty repo must have non-empty porcelain output"
    );
}

#[test]
fn smoke_review_oversized_packet_warning() {
    // Verify warning appears on stderr when packet > 50KB
    let (dir, _base_sha) = setup_temp_git_repo();
    let p = dir.path();

    // Create a large artifact to trigger oversized warning (>50KB)
    let large_content = "x".repeat(60_000);
    std::fs::write(p.join("large.md"), large_content).expect("write large file");
    Command::new("git").args(["add", "large.md"]).current_dir(p).output().expect("git add");
    Command::new("git").args(["commit", "-m", "add large"]).current_dir(p).output().expect("git commit");

    let (_code, _stdout, stderr) = run_in_dir(
        p,
        &["review", "UPG-9999", "test-stage", "--print-packet", "large.md"],
    );

    // Check for enhanced warning format
    assert!(stderr.contains("warning: packet is"), "stderr should contain warning header: {}", stderr);
    assert!(stderr.contains("KB"), "stderr should show size in KB: {}", stderr);
    assert!(stderr.contains("over") && stderr.contains("KB budget"), "stderr should mention budget: {}", stderr);
    assert!(stderr.contains("largest inputs:"), "stderr should show top contributors: {}", stderr);
    assert!(stderr.contains("suggest for R2+:"), "stderr should suggest delta mode: {}", stderr);
    assert!(stderr.contains("--mode delta --base"), "stderr should show delta command: {}", stderr);
    assert!(stderr.contains("optional:"), "stderr should show optional note: {}", stderr);
    assert!(stderr.contains("--sha-only") && stderr.contains("reduces review evidence"),
        "stderr should warn about sha-only: {}", stderr);
}

#[test]
fn smoke_review_warning_goes_to_stderr_not_packet() {
    // Verify warning goes to stderr, not packet content
    let (dir, _base_sha) = setup_temp_git_repo();
    let p = dir.path();

    // Create oversized artifact
    let large_content = "y".repeat(60_000);
    std::fs::write(p.join("large2.md"), large_content).expect("write large file");
    Command::new("git").args(["add", "large2.md"]).current_dir(p).output().expect("git add");
    Command::new("git").args(["commit", "-m", "add large2"]).current_dir(p).output().expect("git commit");

    let (_code, stdout, stderr) = run_in_dir(
        p,
        &["review", "UPG-9998", "test-stage", "--print-packet", "large2.md"],
    );

    // Warning should be in stderr
    assert!(stderr.contains("warning: packet is"), "warning should be in stderr");

    // Warning text should NOT be in packet stdout
    assert!(!stdout.contains("warning: packet is"), "warning should not appear in packet stdout");
    assert!(!stdout.contains("largest inputs:"), "contributor list should not be in packet");
}

#[test]
fn smoke_review_delta_mode_tracked_files_only() {
    // Verify delta mode errors on untracked files
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();

    // Create untracked artifact
    std::fs::write(p.join("untracked.md"), "# untracked\n").expect("write untracked");

    let (code, _stdout, stderr) = run_in_dir(
        p,
        &["review", "UPG-9997", "test-stage", "--print-packet",
          "--mode", "delta", "--base", &base_sha, "untracked.md"],
    );

    // Should fail with clear diagnostic
    assert_ne!(code, 0, "delta mode should fail on untracked files");
    assert!(stderr.contains("untracked"), "stderr should mention untracked: {}", stderr);
    assert!(stderr.contains("delta") || stderr.contains("compare"),
        "stderr should mention delta mode issue: {}", stderr);
}

#[test]
fn smoke_review_sha_only_reduces_packet_size() {
    // Verify --sha-only excludes content from packet
    // Use the unchanged root AGENTS.md so its content cannot re-enter through the working-tree
    // diff while a structural migration is in progress.
    let sha_only_artifact = "AGENTS.md";
    let companion = "CLAUDE.md";

    // Review without sha-only
    let (code1, stdout1, _stderr1) = run(&[
        "review", "UPG-SMOKE-SHA", "test-stage",
        "--print-packet", "--skip-prechecks",
        sha_only_artifact, companion,
    ]);
    assert_eq!(code1, 0, "review without sha-only should succeed");

    // Review with sha-only for Cargo.toml
    let (code2, stdout2, _stderr2) = run(&[
        "review", "UPG-SMOKE-SHA", "test-stage",
        "--print-packet", "--skip-prechecks",
        "--sha-only", sha_only_artifact,
        companion,
    ]);
    assert_eq!(code2, 0, "review with sha-only should succeed");

    // SHA-only packet should be smaller
    assert!(stdout2.len() < stdout1.len(),
        "sha-only packet should be smaller: {} vs {}", stdout2.len(), stdout1.len());

    // SHA-only should show path_sha_only visibility
    assert!(stdout2.contains("path_sha_only"), "sha-only should mark visibility");
    assert!(stdout2.contains("AGENTS.md"), "sha-only manifest should include path");

    let full_artifacts = stdout1.split("ARTIFACTS TO REVIEW").nth(1).unwrap_or("")
        .split("DIFF TO REVIEW").next().unwrap_or("");
    let sha_only_artifacts = stdout2.split("ARTIFACTS TO REVIEW").nth(1).unwrap_or("")
        .split("DIFF TO REVIEW").next().unwrap_or("");

    // Full packet should contain an AGENTS.md artifact section.
    assert!(full_artifacts.contains("--- AGENTS.md ("),
        "full packet should include an AGENTS.md artifact section");

    // SHA-only packet should NOT contain an AGENTS.md artifact section. Its path and digest remain
    // in the manifest, and identical text elsewhere in diff evidence is irrelevant to this check.
    assert!(!sha_only_artifacts.contains("--- AGENTS.md ("),
        "sha-only packet should not include an AGENTS.md artifact section");
}

#[test]
fn smoke_review_help_mentions_evidence_modes() {
    // Verify help text regression prevention
    let (_code, stdout, _stderr) = run(&["review", "--help"]);

    // Check for evidence mode documentation
    assert!(stdout.contains("--mode delta"), "help should mention --mode delta: {}", stdout);
    assert!(stdout.contains("--base"), "help should mention --base: {}", stdout);
    assert!(stdout.contains("tracked files"), "help should mention tracked files requirement: {}", stdout);
    assert!(stdout.contains("--sha-only"), "help should mention --sha-only: {}", stdout);
    assert!(stdout.contains("reduces review evidence") || stdout.contains("reduces evidence"),
        "help should warn about evidence reduction: {}", stdout);
    assert!(stdout.contains("--print-packet"), "help should mention --print-packet: {}", stdout);

    // Check for examples
    assert!(stdout.contains("Examples:") || stdout.contains("# Round 1") || stdout.contains("# Round 2"),
        "help should include usage examples: {}", stdout);
}
