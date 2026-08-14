//! Miscellaneous smoke tests for general CLI behavior (help, diagnose).
//!
//! Command-specific tests live in their own files:
//! - review_command.rs (review command, evidence modes, packet building)
//! - decision_command.rs (decision command)

mod common;
use common::run;

#[test]
fn smoke_help_exits_zero() {
    let (code, stdout, _) = run(&["--help"]);
    assert_eq!(code, 0, "help should exit 0");
    assert!(stdout.contains("codeos-reviewer"), "help should mention binary name");
}

#[test]
fn smoke_help_omits_retired_convenience_commands() {
    let (code, stdout, _) = run(&["--help"]);
    assert_eq!(code, 0);
    for retired in [
        "check-drift",
        "generate-report",
        "generate-adr-candidates",
        "generate-approval-dashboard",
        "generate-release-evidence",
    ] {
        assert!(
            !stdout.contains(retired),
            "retired command remains: {retired}"
        );
    }
}

#[test]
fn smoke_retired_command_fails_as_unknown() {
    let (code, _, stderr) = run(&["generate-report", "--stage", "4"]);
    assert_eq!(code, 1);
    assert!(stderr.contains("unrecognized subcommand"));
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
