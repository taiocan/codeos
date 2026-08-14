//! Behavioral tests for the reviewer-to-Codex boundary. No real Codex process is invoked.

mod common;
use common::{repo_root, run_in_dir, run_with_fake_codex, setup_fake_codex, setup_temp_git_repo};

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
    assert!(invoked.contains("--sandbox read-only"), "{invoked}");

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
fn saved_session_is_resumed_and_fresh_bypasses_it() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let fake = setup_fake_codex();

    let (first, _, stderr) =
        run_with_fake_codex(repo.path(), &fake, &review_args("UPG-SESSION"), "success");
    assert_eq!(first, 0, "{stderr}");
    let session = repo
        .path()
        .join(".codeos-state/codex-sessions/UPG-SESSION.json");
    assert!(session.is_file());

    let (second, _, stderr) =
        run_with_fake_codex(repo.path(), &fake, &review_args("UPG-SESSION"), "success");
    assert_eq!(second, 0, "{stderr}");
    let resumed_args = std::fs::read_to_string(&fake.args_log).unwrap();
    assert!(
        resumed_args.contains("exec resume --json"),
        "{resumed_args}"
    );
    assert!(resumed_args.contains("fake-session"), "{resumed_args}");
    assert!(
        resumed_args.contains("sandbox_mode=read-only"),
        "{resumed_args}"
    );

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
    assert!(!fresh_args.contains(" resume "), "{fresh_args}");
}

#[test]
fn external_mutation_is_reported_without_gaining_authority() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let fake = setup_fake_codex();
    let (code, _, stderr) = run_with_fake_codex(
        repo.path(),
        &fake,
        &review_args("UPG-READONLY"),
        "mutate",
    );
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

fn value_after(text: &str, marker: &str) -> Option<u64> {
    let rest = &text[text.find(marker)? + marker.len()..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}
