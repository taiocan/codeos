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
    assert!(
        stdout.contains("codeos-reviewer"),
        "help should mention binary name"
    );
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
        "stage-start",
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
    assert!(
        stdout.contains("reviewer:") && stdout.contains("codex"),
        "diagnose should identify Codex: {stdout}"
    );
}

#[test]
fn smoke_diagnose_with_feature_and_stage() {
    let (code, stdout, _) = run(&["diagnose", "UPG-9999", "selfdev-step-1"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("UPG-9999"), "diagnose should echo feature");
}

#[test]
fn smoke_provider_override_is_retired() {
    let (code, _, stderr) = run(&["--provider", "nonexistent-provider-xyz", "diagnose"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("unexpected argument"), "{stderr}");
}

#[test]
fn smoke_diagnose_shows_reasoning_source() {
    let (code, stdout, _) = run(&["diagnose"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("reasoning_effort:"), "{stdout}");
}
