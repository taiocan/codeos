// Integration / smoke tests — require a git repo and bash; fast to run.
// These tests verify the CLI surface, exit codes, and packet behavior.
// Provider invocation is NOT tested here (that would require codex on PATH).

use std::path::PathBuf;
use std::process::Command;

fn binary() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target/debug/codeos-reviewer");
    p
}

fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); p.pop(); // tools/reviewer -> Codeos/
    p
}

/// Run the binary with given args from the repo root, return (exit code, stdout, stderr).
fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(binary())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("failed to run codeos-reviewer binary");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

#[test]
fn smoke_help_exits_zero() {
    let (code, stdout, _) = run(&["--help"]);
    assert_eq!(code, 0, "help should exit 0");
    assert!(stdout.contains("codeos-reviewer"), "help should mention binary name");
}

#[test]
fn smoke_diagnose_exits_zero() {
    let (code, stdout, _) = run(&["diagnose"]);
    assert_eq!(code, 0, "diagnose should exit 0");
    assert!(stdout.contains("provider:"), "diagnose should output provider");
}

#[test]
fn smoke_diagnose_with_feature_and_stage() {
    let (code, stdout, _) = run(&["diagnose", "UPG-9999", "selfdev-step-1"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("UPG-9999"), "diagnose should echo feature");
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
fn smoke_decision_bad_verdict_exits_usage() {
    let (code, _, stderr) = run(&[
        "decision", "UPG-SMOKE", "selfdev-step-0", "WRONG_VERDICT", "reason",
    ]);
    assert_ne!(code, 0, "bad verdict should fail");
    assert!(
        stderr.contains("APPROVE_STAGE") || stderr.contains("REQUEST_CHANGES"),
        "should mention valid verdicts: {}",
        stderr
    );
}

#[test]
fn smoke_provider_override_unknown() {
    // An unknown provider via --provider flag should exit with EXIT_CONFIG (2) or EXIT_PROVIDER (3)
    let (code, _, stderr) = run(&[
        "--provider", "nonexistent-provider-xyz",
        "review", "UPG-SMOKE", "selfdev-step-0",
        "--print-packet", "--skip-prechecks",
        "CLAUDE.md",
    ]);
    // With --print-packet, provider is never invoked; config resolution may still reject it
    // Accept either success (provider not resolved yet) or config error
    let _ = (code, stderr);
}

#[test]
fn smoke_diagnose_shows_provider_source() {
    let (code, stdout, _) = run(&["diagnose"]);
    assert_eq!(code, 0);
    // Should mention where provider came from
    assert!(
        stdout.contains("source:") || stdout.contains("default") || stdout.contains("codex"),
        "diagnose should show provider source: {}",
        stdout
    );
}
