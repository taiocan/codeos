//! Behavioral tests for the reviewer-to-Codex boundary. No real Codex process is invoked.

mod common;
use common::{
    repo_root, run_in_dir, run_with_fake_codex, run_with_fake_codex_env, setup_fake_codex,
    setup_temp_git_repo,
};

fn setup_codeos_symlink(repo_path: &std::path::Path) {
    std::fs::create_dir_all(repo_path.join(".codeos")).unwrap();
    std::os::unix::fs::symlink(repo_root(), repo_path.join(".codeos/toolkit")).unwrap();
    std::fs::write(repo_path.join(".git/info/exclude"), "/.codeos/toolkit\n").unwrap();
}

fn review_args<'a>(feature: &'a str) -> [&'a str; 4] {
    ["review", feature, "selfdev-step-1", "tracked.md"]
}

#[test]
fn plan_and_review_prepare_equivalent_packets() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let fake = setup_fake_codex();

    let (plan_code, plan_stdout, plan_stderr) = run_in_dir(
        repo.path(),
        &["plan", "UPG-PARITY", "selfdev-step-1", "tracked.md"],
    );
    assert_eq!(plan_code, 0, "{plan_stderr}");

    let (review_code, review_stdout, review_stderr) =
        run_with_fake_codex(repo.path(), &fake, &review_args("UPG-PARITY"), "success");
    assert_eq!(review_code, 0, "{review_stderr}");
    assert!(review_stdout.contains("review logged:"), "{review_stdout}");
    let invoked = std::fs::read_to_string(&fake.args_log).unwrap();
    for required in [
        "--ephemeral",
        "--ignore-user-config",
        "--ignore-rules",
        "--strict-config",
        "--skip-git-repo-check",
        "project_doc_max_bytes=0",
        "default_permissions=\"codeos-review\"",
        "permissions.codeos-review.filesystem=",
        "permissions.codeos-review.network.enabled=false",
        "shell_environment_policy.inherit=\"none\"",
    ] {
        assert!(invoked.contains(required), "missing {required}: {invoked}");
    }
    assert!(!invoked.contains("--sandbox"), "{invoked}");
    assert!(
        !invoked.contains(repo.path().to_str().unwrap()),
        "{invoked}"
    );

    let captured = std::fs::read_to_string(&fake.packet_log).unwrap();
    assert!(captured.contains("Evidence coverage:      FULL_COVERAGE"));
    assert!(plan_stdout.contains("coverage: FULL_COVERAGE"));
    let planned_bytes = value_after(&plan_stdout, "review_content_bytes: ").unwrap();
    let packet_bytes = value_after(&captured, "review_content_bytes: ").unwrap();
    assert_eq!(planned_bytes, packet_bytes);

    let packets = repo.path().join(".codeos/05-review/reviews/codex/packets");
    let saved = std::fs::read_dir(packets)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(captured, std::fs::read_to_string(saved).unwrap());
}

#[test]
fn every_review_is_fresh_ephemeral_and_saved_sessions_are_ignored() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let fake = setup_fake_codex();

    let sessions = repo.path().join(".codeos-state/codex-sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    let historical = sessions.join("UPG-SESSION.json");
    std::fs::write(
        &historical,
        r#"{"feature":"UPG-SESSION","session_id":"old-session","codex_version":"old","created_at":"old"}"#,
    )
    .unwrap();

    let (first, _, stderr) =
        run_with_fake_codex(repo.path(), &fake, &review_args("UPG-SESSION"), "success");
    assert_eq!(first, 0, "{stderr}");
    let first_args = std::fs::read_to_string(&fake.args_log).unwrap();
    assert!(first_args.contains("--ephemeral"), "{first_args}");
    assert!(!first_args.contains("resume"), "{first_args}");
    assert!(!first_args.contains("old-session"), "{first_args}");

    let (second, _, stderr) =
        run_with_fake_codex(repo.path(), &fake, &review_args("UPG-SESSION"), "success");
    assert_eq!(second, 0, "{stderr}");
    let second_args = std::fs::read_to_string(&fake.args_log).unwrap();
    assert!(second_args.contains("--ephemeral"), "{second_args}");
    assert!(!second_args.contains("resume"), "{second_args}");

    let (fresh, _, stderr) = run_with_fake_codex(
        repo.path(),
        &fake,
        &[
            "review",
            "UPG-SESSION",
            "selfdev-step-1",
            "--fresh",
            "tracked.md",
        ],
        "success",
    );
    assert_eq!(fresh, 0, "{stderr}");
    let fresh_args = std::fs::read_to_string(&fake.args_log).unwrap();
    assert!(fresh_args.contains("--ephemeral"), "{fresh_args}");
    assert!(!fresh_args.contains("resume"), "{fresh_args}");
    assert_eq!(
        std::fs::read_to_string(&historical).unwrap(),
        r#"{"feature":"UPG-SESSION","session_id":"old-session","codex_version":"old","created_at":"old"}"#
    );
}

#[test]
fn external_mutation_is_reported_without_gaining_authority() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let fake = setup_fake_codex();
    let (code, _, stderr) =
        run_with_fake_codex(repo.path(), &fake, &review_args("UPG-READONLY"), "mutate");
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stderr.contains("working tree changed during review"),
        "{stderr}"
    );
}

#[test]
fn codex_failure_and_malformed_jsonl_fail_before_records() {
    for (feature, mode, expected) in [
        ("UPG-FAIL", "failure", "codex exec failed"),
        ("UPG-MALFORMED", "malformed", "malformed Codex JSONL"),
    ] {
        let (repo, _) = setup_temp_git_repo();
        setup_codeos_symlink(repo.path());
        let fake = setup_fake_codex();
        let (code, _, stderr) =
            run_with_fake_codex(repo.path(), &fake, &review_args(feature), mode);
        assert_eq!(code, 3, "{stderr}");
        assert!(stderr.contains(expected), "{stderr}");
        assert!(!repo.path().join(".codeos/05-review/reviews").exists());
    }
}

#[test]
fn ineffective_isolation_fails_before_model_or_records() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let fake = setup_fake_codex();

    let (code, _, stderr) = run_with_fake_codex(
        repo.path(),
        &fake,
        &review_args("UPG-ISOLATION-FAIL"),
        "isolation_failure",
    );

    assert_eq!(code, 3, "{stderr}");
    assert!(stderr.contains("isolation preflight failed"), "{stderr}");
    assert!(!fake.args_log.exists(), "model-bearing exec must not start");
    assert!(!fake.packet_log.exists(), "packet must not reach Codex");
    assert!(!repo.path().join(".codeos/05-review/reviews").exists());
}

#[test]
fn over_budget_default_refuses_before_codex_session_or_records() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let fake = setup_fake_codex();

    let (code, _, stderr) = run_with_fake_codex_env(
        repo.path(),
        &fake,
        &review_args("UPG-BUDGET-REFUSAL"),
        "success",
        &[("CODEOS_PACKET_BUDGET_BYTES", "1")],
    );

    assert_eq!(code, 4, "{stderr}");
    assert!(
        stderr.contains("oversized Codex invocation refused"),
        "{stderr}"
    );
    assert!(!fake.args_log.exists(), "Codex exec must not start");
    assert!(!fake.packet_log.exists(), "no packet may be sent to Codex");
    assert!(!repo.path().join(".codeos-state/codex-sessions").exists());
    assert!(!repo.path().join(".codeos/05-review/reviews").exists());
}

#[test]
fn warn_override_permits_an_intentional_over_budget_codex_invocation() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let fake = setup_fake_codex();

    let (code, stdout, stderr) = run_with_fake_codex_env(
        repo.path(),
        &fake,
        &review_args("UPG-BUDGET-OVERRIDE"),
        "success",
        &[
            ("CODEOS_PACKET_BUDGET_BYTES", "1"),
            ("CODEOS_PACKET_BUDGET_MODE", "warn"),
        ],
    );

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("review logged:"), "{stdout}");
    assert!(
        fake.args_log.is_file(),
        "operator override must reach Codex exec"
    );
    assert!(!repo.path().join(".codeos-state/codex-sessions").exists());
    assert!(repo.path().join(".codeos/05-review/reviews").is_dir());
}

fn value_after(text: &str, marker: &str) -> Option<u64> {
    let rest = &text[text.find(marker)? + marker.len()..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}
