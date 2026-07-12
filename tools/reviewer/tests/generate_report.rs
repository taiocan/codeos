//! generate-report command tests.
//!
//! Tests for the generate-report subcommand (Stage 4-6 approval-dashboard report generation).

mod common;
use common::{setup_temp_git_repo, add_extra_commit, run_in_dir, binary, repo_root};
use std::process::Command;

fn read_template() -> String {
    std::fs::read_to_string(repo_root().join("templates/stage-4-6-report.md"))
        .expect("read stage-4-6-report.md template")
}

/// Slice out the `## Stage <n> ...` section of the template, up to the next stage header (or EOF).
fn template_section(stage: &str) -> String {
    let content = read_template();
    let marker = format!("## Stage {}", stage);
    let start = content.find(&marker).expect("find stage marker");
    let rest = &content[start..];
    // Find the next stage marker or EOF
    let end = rest[marker.len()..]
        .find("\n## Stage ")
        .map(|i| i + marker.len())
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

fn extract_field_labels(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.starts_with("**") && line.contains("**:"))
        .map(|line| {
            line.split("**:")
                .next()
                .unwrap_or("")
                .trim_start_matches("**")
                .trim()
                .to_string()
        })
        .collect()
}

#[test]
fn smoke_generate_report_stage4_field_coverage() {
    // AC-1 / AC-16: every field from the Stage 4 template section appears in the generated
    // output, in the same order, including the nested "Approved artifacts used:" sub-items.
    let (dir, _sha) = setup_temp_git_repo();
    let (code, stdout, stderr) = run_in_dir(dir.path(), &["generate-report", "--stage", "4"]);
    assert_eq!(code, 0, "stage 4 must exit 0; stderr: {}", stderr);

    let expected = extract_field_labels(&template_section("4"));
    let actual = extract_field_labels(&stdout);
    assert_eq!(actual, expected, "stage 4 field labels must match template order/coverage");
}

#[test]
fn smoke_generate_report_stage5_field_coverage() {
    // AC-2 / AC-16.
    let (dir, _sha) = setup_temp_git_repo();
    let (code, stdout, stderr) = run_in_dir(dir.path(), &["generate-report", "--stage", "5"]);
    assert_eq!(code, 0, "stage 5 must exit 0; stderr: {}", stderr);

    let expected = extract_field_labels(&template_section("5"));
    let actual = extract_field_labels(&stdout);
    assert_eq!(actual, expected, "stage 5 field labels must match template order/coverage");
}

#[test]
fn smoke_generate_report_stage6_field_coverage() {
    // AC-3 / AC-16: includes the nested "Raw logs committed:" sub-items.
    let (dir, _sha) = setup_temp_git_repo();
    let (code, stdout, stderr) = run_in_dir(dir.path(), &["generate-report", "--stage", "6"]);
    assert_eq!(code, 0, "stage 6 must exit 0; stderr: {}", stderr);

    let expected = extract_field_labels(&template_section("6"));
    let actual = extract_field_labels(&stdout);
    assert_eq!(actual, expected, "stage 6 field labels must match template order/coverage");
}

#[test]
fn smoke_generate_report_preamble_present() {
    // AC-6: every generated report opens with the [INFERRED]/[FILL] warning block.
    let (dir, _sha) = setup_temp_git_repo();
    let (code, stdout, stderr) = run_in_dir(dir.path(), &["generate-report", "--stage", "4"]);
    assert_eq!(code, 0, "stderr: {}", stderr);

    let mut lines = stdout.lines();
    assert_eq!(
        lines.next().unwrap_or(""),
        "> [INFERRED] fields were populated automatically from CLI arguments, git, test output, or"
    );
    assert_eq!(
        lines.next().unwrap_or(""),
        "> events — verify before submitting. [FILL] fields require human or model authorship."
    );
}

#[test]
fn smoke_generate_report_feature_flag_inferred() {
    // AC-7 / AC-4: --feature populates "Feature:" as "<id> [INFERRED]"; omitted -> "[FILL]".
    let (dir, _sha) = setup_temp_git_repo();

    let (code, stdout, stderr) =
        run_in_dir(dir.path(), &["generate-report", "--stage", "4", "--feature", "UPG-9999"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Feature: UPG-9999 [INFERRED]"), "stdout: {}", stdout);

    let (code, stdout, stderr) = run_in_dir(dir.path(), &["generate-report", "--stage", "4"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Feature: [FILL]"), "stdout: {}", stdout);
}

#[test]
fn smoke_generate_report_base_populates_files_changed() {
    // AC-8(a) / AC-4: --base <known-commit> with a subsequent commit populates Files changed.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    add_extra_commit(p, "changed.md", "# changed\n");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-report", "--stage", "4", "--base", &base_sha]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Files changed: [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("changed.md"), "stdout: {}", stdout);
}

#[test]
fn smoke_generate_report_base_zero_diff() {
    // AC-8(b): --base HEAD (no diff) -> "(none) [INFERRED]", exit 0, no error.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-report", "--stage", "4", "--base", "HEAD"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Files changed: (none) [INFERRED]"), "stdout: {}", stdout);
    assert!(stderr.is_empty(), "no error expected for zero-diff; stderr: {}", stderr);
}

#[test]
fn smoke_generate_report_base_invalid_ref() {
    // AC-8(c): --base nonexistent-ref -> Files changed [FILL], error on stderr, exit 0.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-report", "--stage", "4", "--base", "nonexistent-ref-xyz"]);
    assert_eq!(code, 0, "invalid base must not fail the whole command; stderr: {}", stderr);
    assert!(stdout.contains("Files changed: [FILL]"), "stdout: {}", stdout);
    assert!(!stderr.is_empty(), "expected an error on stderr for invalid base ref");
}

#[test]
fn smoke_generate_report_test_output_valid_summary() {
    // AC-9(a) / AC-4: a fixture with a valid cargo test summary line populates all four counts.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("test-output.txt");
    std::fs::write(
        &fixture,
        "running 10 tests\n...\ntest result: ok. 7 passed; 2 failed; 1 ignored; 0 measured; 0 filtered out\n",
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-report", "--stage", "5", "--test-output", fixture.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Tests run: 9 [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("Tests passed: 7 [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("Tests failed: 2 [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("Tests skipped: 1 [INFERRED]"), "stdout: {}", stdout);
}

#[test]
fn smoke_generate_report_test_output_no_matching_line() {
    // AC-9(b): a fixture with no matching summary line falls back to [FILL] for all four,
    // with no error (graceful fallback, distinct from a read error).
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("test-output.txt");
    std::fs::write(&fixture, "no summary here\n").expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-report", "--stage", "5", "--test-output", fixture.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Tests run: [FILL]"), "stdout: {}", stdout);
    assert!(stdout.contains("Tests passed: [FILL]"), "stdout: {}", stdout);
    assert!(stdout.contains("Tests failed: [FILL]"), "stdout: {}", stdout);
    assert!(stdout.contains("Tests skipped: [FILL]"), "stdout: {}", stdout);
    assert!(stderr.is_empty(), "no error expected for non-matching content; stderr: {}", stderr);
}

#[test]
fn smoke_generate_report_test_output_missing_file() {
    // AC-9(c): a nonexistent --test-output path -> [FILL] fallback + error on stderr.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-report", "--stage", "5", "--test-output", "does-not-exist.txt"],
    );
    assert_eq!(code, 0, "missing file must not fail the whole command; stderr: {}", stderr);
    assert!(stdout.contains("Tests run: [FILL]"), "stdout: {}", stdout);
    assert!(!stderr.is_empty(), "expected an error on stderr for missing test-output file");
}

#[test]
fn smoke_generate_report_events_fixture() {
    // AC-10(a) / AC-4: a readable --events file populates Events captured with its line count.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("events.jsonl");
    std::fs::write(&fixture, "{}\n{}\n{}\n").expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-report", "--stage", "6", "--events", fixture.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Events captured: 3 [INFERRED]"), "stdout: {}", stdout);
}

#[test]
fn smoke_generate_report_events_missing_file() {
    // AC-10(b): a nonexistent --events path -> [FILL] fallback + error on stderr.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-report", "--stage", "6", "--events", "does-not-exist.jsonl"],
    );
    assert_eq!(code, 0, "missing events file must not fail the whole command; stderr: {}", stderr);
    assert!(stdout.contains("Events captured: [FILL]"), "stdout: {}", stdout);
    assert!(!stderr.is_empty(), "expected an error on stderr for missing events file");
}

#[test]
fn smoke_generate_report_all_fill_without_optional_inputs() {
    // AC-5: with no optional inputs, every field the tool cannot infer is [FILL] — never
    // blank, including the "Raw logs committed:" / "Approved artifacts used:" parent labels
    // for nested sub-item groups (they have no value of their own to infer either).
    let (dir, _sha) = setup_temp_git_repo();
    let (code, stdout, stderr) = run_in_dir(dir.path(), &["generate-report", "--stage", "6"]);
    assert_eq!(code, 0, "stderr: {}", stderr);

    let mut checked_parent_labels = 0;
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('>') {
            continue;
        }
        if trimmed == "Raw logs committed: [FILL]" {
            checked_parent_labels += 1;
        }
        if let Some(idx) = line.find(':') {
            let value = line[idx + 1..].trim();
            assert!(!value.is_empty(), "field must not be blank; line: {}", line);
        }
    }
    assert_eq!(checked_parent_labels, 1, "Raw logs committed: must carry [FILL], not be blank");
}

#[test]
fn smoke_generate_report_test_output_unrecognized_status() {
    // AC-9: a summary line whose status is neither "ok" nor "FAILED" (case-sensitive) does
    // not match the stated regex and must fall back to [FILL], same as no matching line.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("test-output.txt");
    std::fs::write(
        &fixture,
        "test result: WEIRD. 7 passed; 2 failed; 1 ignored; 0 measured; 0 filtered out\n",
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-report", "--stage", "5", "--test-output", fixture.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Tests run: [FILL]"), "stdout: {}", stdout);
    assert!(stdout.contains("Tests passed: [FILL]"), "stdout: {}", stdout);
    assert!(stdout.contains("Tests failed: [FILL]"), "stdout: {}", stdout);
    assert!(stdout.contains("Tests skipped: [FILL]"), "stdout: {}", stdout);
}

#[test]
fn smoke_generate_report_stdout_only_no_stderr_on_success() {
    // AC-11: on a clean run, stderr is empty; all content goes to stdout.
    let (dir, _sha) = setup_temp_git_repo();
    let (code, stdout, stderr) = run_in_dir(dir.path(), &["generate-report", "--stage", "4"]);
    assert_eq!(code, 0);
    assert!(stderr.is_empty(), "clean run must have empty stderr; got: {}", stderr);
    assert!(!stdout.is_empty());
}

#[test]
fn smoke_generate_report_stdout_still_written_on_partial_error() {
    // AC-11: a bad --events path still yields a full report on stdout with [FILL] fallback,
    // while the error goes to stderr only.
    let (dir, _sha) = setup_temp_git_repo();
    let (code, stdout, stderr) = run_in_dir(
        dir.path(),
        &["generate-report", "--stage", "6", "--events", "does-not-exist.jsonl"],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("## Stage 6 Runtime Evidence Report"), "stdout: {}", stdout);
    assert!(!stderr.is_empty());
}

#[test]
fn smoke_generate_report_exit_zero_all_stages() {
    // AC-12: all valid invocations exit 0.
    let (dir, _sha) = setup_temp_git_repo();
    for stage in ["4", "5", "6"] {
        let (code, _, stderr) = run_in_dir(dir.path(), &["generate-report", "--stage", stage]);
        assert_eq!(code, 0, "stage {} must exit 0; stderr: {}", stage, stderr);
    }
}

#[test]
fn smoke_generate_report_invalid_stage_exits_usage() {
    // AC-13: an out-of-range --stage value exits 1 (EXIT_USAGE) with a message on stderr.
    let (dir, _sha) = setup_temp_git_repo();
    let (code, _, stderr) = run_in_dir(dir.path(), &["generate-report", "--stage", "7"]);
    assert_eq!(code, 1, "invalid stage must exit 1 (EXIT_USAGE); stderr: {}", stderr);
    assert!(!stderr.is_empty(), "expected a usage message on stderr");
}

#[test]
fn smoke_generate_report_no_provider_config_required() {
    // AC-14: generate-report dispatches before config::resolve() and needs no provider config
    // (this temp repo has no .codeos symlink or reviewer.toml set up at all).
    let (dir, _sha) = setup_temp_git_repo();
    let (code, _, stderr) = run_in_dir(dir.path(), &["generate-report", "--stage", "4"]);
    assert_eq!(code, 0, "must succeed without provider config; stderr: {}", stderr);
}

#[test]
fn smoke_generate_report_deterministic_output() {
    // AC-15: identical inputs produce byte-for-byte identical stdout across two invocations.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    add_extra_commit(p, "changed.md", "# changed\n");

    let args = ["generate-report", "--stage", "4", "--feature", "UPG-9999", "--base", base_sha.as_str()];
    let (code1, stdout1, _) = run_in_dir(p, &args);
    let (code2, stdout2, _) = run_in_dir(p, &args);
    assert_eq!(code1, 0);
    assert_eq!(code2, 0);
    assert_eq!(stdout1, stdout2, "output must be deterministic for identical inputs");
}

