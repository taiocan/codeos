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

fn add_stage8_inputs(repo_path: &std::path::Path, feature: &str) -> Vec<String> {
    let paths = vec![
        ".codeos/00-project/charter.md".to_string(),
        format!(".codeos/01-specification/intents/{feature}.md"),
        format!(".codeos/01-specification/contracts/{feature}_contract.md"),
        format!(".codeos/01-specification/event-schemas/{feature}_schema.md"),
    ];
    for path in &paths {
        let absolute = repo_path.join(path);
        std::fs::create_dir_all(absolute.parent().unwrap()).unwrap();
        std::fs::write(&absolute, format!("# {path}\napproved content\n")).unwrap();
    }
    Command::new("git")
        .arg("add")
        .args(&paths)
        .current_dir(repo_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "add stage8 inputs"])
        .current_dir(repo_path)
        .output()
        .unwrap();
    paths
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
fn communication_context_does_not_change_review_content_budget() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let args = ["plan", "UPG-OUTPUT", "selfdev-step-1", "tracked.md"];

    let (first_code, first_stdout, first_stderr) = run_in_dir(repo.path(), &args);
    assert_eq!(first_code, 0, "{first_stderr}");
    let before = plan_value_after(&first_stdout, "review_content_bytes: ").unwrap();

    std::fs::create_dir_all(repo.path().join(".codeos/00-project")).unwrap();
    std::fs::write(
        repo.path().join(".codeos/00-project/terminology.md"),
        "# Project Terminology\nA large communication-only glossary entry.\n",
    )
    .unwrap();
    Command::new("git")
        .args(["add", ".codeos/00-project/terminology.md"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "add project terminology"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let (second_code, second_stdout, second_stderr) = run_in_dir(repo.path(), &args);
    assert_eq!(second_code, 0, "{second_stderr}");
    let after = plan_value_after(&second_stdout, "review_content_bytes: ").unwrap();
    assert_eq!(before, after, "communication context changed evidence budgeting");
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
fn stage8_plan_fails_with_exact_missing_canonical_inputs() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());

    let (code, stdout, stderr) = run_in_dir(
        repo.path(),
        &["plan", "F-0042", "8", "--skip-prechecks", "tracked.md"],
    );

    assert_eq!(code, 4, "{stderr}");
    assert!(stdout.contains("stage8_readiness: FAIL"), "{stdout}");
    for path in [
        ".codeos/00-project/charter.md",
        ".codeos/01-specification/intents/F-0042.md",
        ".codeos/01-specification/contracts/F-0042_contract.md",
        ".codeos/01-specification/event-schemas/F-0042_schema.md",
    ] {
        assert!(
            stdout.contains(path),
            "missing diagnostic for {path}: {stdout}"
        );
    }
}

#[test]
fn stage8_plan_passes_with_canonical_inputs_and_does_not_infer_undeclared_evidence() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let paths = add_stage8_inputs(repo.path(), "F-0042");
    let mut args = vec!["plan", "F-0042", "8"];
    args.extend(paths.iter().map(String::as_str));

    let (code, stdout, stderr) = run_in_dir(repo.path(), &args);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("stage8_readiness: PASS"), "{stdout}");
    assert!(stdout.contains("coverage: FULL_COVERAGE"), "{stdout}");
}

#[test]
fn stage8_secret_redaction_label_alone_does_not_fail_readiness() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let paths = add_stage8_inputs(repo.path(), "F-0042");
    std::fs::write(repo.path().join("evidence.md"), "TOKEN=abcdefghijk\n").unwrap();
    Command::new("git")
        .args(["add", "evidence.md"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "add redacted evidence"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let mut args = vec!["plan", "F-0042", "8", "evidence.md"];
    args.extend(paths.iter().map(String::as_str));

    let (code, stdout, stderr) = run_in_dir(repo.path(), &args);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("coverage: SECRET_REDACTION"), "{stdout}");
    assert!(stdout.contains("stage8_readiness: PASS"), "{stdout}");
}

#[test]
fn stage8_partial_coverage_label_alone_does_not_fail_readiness() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let paths = add_stage8_inputs(repo.path(), "F-0042");
    std::fs::create_dir_all(repo.path().join("events")).unwrap();
    std::fs::write(
        repo.path().join("events/runtime_events.jsonl"),
        "{\"event\":\"intentionally excluded untracked context\"}\n",
    )
    .unwrap();
    let mut args = vec!["plan", "F-0042", "8"];
    args.extend(paths.iter().map(String::as_str));

    let (code, stdout, stderr) = run_in_dir(repo.path(), &args);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("coverage: PARTIAL_COVERAGE"), "{stdout}");
    assert!(stdout.contains("stage8_readiness: PASS"), "{stdout}");
}

#[test]
fn stage8_empty_canonical_input_fails_with_corrective_action() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let paths = add_stage8_inputs(repo.path(), "F-0042");
    std::fs::write(repo.path().join(&paths[0]), "").unwrap();
    let mut args = vec!["plan", "F-0042", "8"];
    args.extend(paths.iter().map(String::as_str));

    let (code, stdout, stderr) = run_in_dir(repo.path(), &args);

    assert_eq!(code, 4, "{stderr}");
    assert!(stdout.contains("required canonical Stage-8 input is empty"), "{stdout}");
    assert!(stdout.contains("provide the populated approved artifact"), "{stdout}");
}

#[test]
fn stage8_canonical_hash_only_input_is_not_reviewable() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let paths = add_stage8_inputs(repo.path(), "F-0042");
    let charter = paths[0].as_str();
    let mut args = vec!["plan", "F-0042", "8", "--sha-only", charter];
    args.extend(paths[1..].iter().map(String::as_str));

    let (code, stdout, stderr) = run_in_dir(repo.path(), &args);

    assert_eq!(code, 4, "{stderr}");
    assert!(stdout.contains("stage8_readiness: FAIL"), "{stdout}");
    assert!(stdout.contains("visibility: path_sha_only"), "{stdout}");
}

#[test]
fn stage8_oversize_declared_evidence_fails_but_redaction_does_not() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let paths = add_stage8_inputs(repo.path(), "F-0042");
    std::fs::write(repo.path().join("oversize.md"), vec![b'x'; 256 * 1024 + 1]).unwrap();
    Command::new("git")
        .args(["add", "oversize.md"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "add oversize evidence"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let mut args = vec!["plan", "F-0042", "8", "oversize.md"];
    args.extend(paths.iter().map(String::as_str));

    let (code, stdout, stderr) = run_in_dir(repo.path(), &args);

    assert_eq!(code, 4, "{stderr}");
    assert!(stdout.contains("stage8_readiness: FAIL"), "{stdout}");
    assert!(stdout.contains("oversize_omitted"), "{stdout}");
    assert!(stdout.contains("oversize.md"), "{stdout}");
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
    assert!(
        stdout.contains("Codex review will refuse this packet"),
        "plan must explain the default invocation policy: {}",
        stdout
    );
}

#[test]
fn smoke_plan_warn_override_still_builds_and_reports_an_over_budget_packet() {
    // Spending policy never blocks packet construction: plan remains diagnostic under the
    // explicit operator override and reports that the corresponding Codex invocation is allowed.
    let out = Command::new(binary())
        .args([
            "plan",
            "UPG-SMOKE-TEST",
            "selfdev-step-3",
            "--skip-prechecks",
            "dba/04-tools/reviewer/engine/src/packet.rs",
        ])
        .current_dir(repo_root())
        .env("CODEOS_PACKET_BUDGET_BYTES", "1000")
        .env("CODEOS_PACKET_BUDGET_MODE", "warn")
        .output()
        .expect("run plan with tiny budget and warn override");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(code, 0, "plan must remain diagnostic: {}", stdout);
    assert!(
        stdout.contains("operator override active: CODEOS_PACKET_BUDGET_MODE=warn"),
        "plan must report the active override: {}",
        stdout
    );
}

#[test]
fn smoke_plan_events_log_artifact_warns_but_still_builds() {
    // The precheck (UPG-0074) fires on a whole runtime-event-log passed as a positional artifact
    // — the measured cause of the F-0004 packet's bloat — but it warns, it does not block: the
    // packet still builds and the command still exits 0.
    let (dir, _base_sha) = setup_temp_git_repo();
    setup_codeos_symlink(dir.path());
    std::fs::write(dir.path().join("runtime_events.jsonl"), "{}\n").expect("write events log");
    Command::new("git")
        .args(["add", "runtime_events.jsonl"])
        .current_dir(dir.path())
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "add events log"])
        .current_dir(dir.path())
        .output()
        .expect("git commit");

    let (code, _stdout, stderr) = run_in_dir(
        dir.path(),
        &[
            "plan",
            "UPG-SMOKE-TEST",
            "selfdev-step-3",
            "runtime_events.jsonl",
        ],
    );
    assert_eq!(code, 0, "the hygiene check must warn, not fail: {}", stderr);
    assert!(
        stderr.contains("looks like a full runtime event log"),
        "expected the artifact-hygiene warning, got: {}",
        stderr
    );
}

#[test]
fn smoke_plan_events_log_via_sha_only_does_not_warn() {
    // The same file passed via --sha-only, the tool's own suggested remedy, must not trigger the
    // warning — its content never reaches the precheck loop, which only reads `args.artifacts`.
    let (dir, _base_sha) = setup_temp_git_repo();
    setup_codeos_symlink(dir.path());
    std::fs::write(dir.path().join("runtime_events.jsonl"), "{}\n").expect("write events log");
    Command::new("git")
        .args(["add", "runtime_events.jsonl"])
        .current_dir(dir.path())
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "add events log"])
        .current_dir(dir.path())
        .output()
        .expect("git commit");

    let (code, _stdout, stderr) = run_in_dir(
        dir.path(),
        &[
            "plan",
            "UPG-SMOKE-TEST",
            "selfdev-step-3",
            "--sha-only",
            "runtime_events.jsonl",
            "tracked.md",
        ],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(
        !stderr.contains("looks like a full runtime event log"),
        "a --sha-only file must not trigger the artifact-hygiene warning: {}",
        stderr
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

fn plan_value_after(text: &str, marker: &str) -> Option<u64> {
    let rest = &text[text.find(marker)? + marker.len()..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}
