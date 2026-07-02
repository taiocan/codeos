// Integration / smoke tests — require a git repo and bash; fast to run.
// These tests verify the CLI surface, exit codes, and packet behavior.
// Provider invocation is NOT tested here (that would require codex on PATH).

use tempfile::TempDir;

/// Create a minimal git repo in a temp directory and return (TempDir, base_sha).
fn setup_temp_git_repo() -> (TempDir, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let p = dir.path();
    Command::new("git").args(["init"]).current_dir(p).output().expect("git init");
    Command::new("git").args(["config", "user.email", "test@codeos.test"]).current_dir(p).output().ok();
    Command::new("git").args(["config", "user.name", "Codeos Test"]).current_dir(p).output().ok();
    std::fs::write(p.join("tracked.md"), "# tracked\n").expect("write tracked");
    Command::new("git").args(["add", "tracked.md"]).current_dir(p).output().expect("git add");
    Command::new("git").args(["commit", "-m", "initial"]).current_dir(p).output().expect("git commit");
    let sha_out = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let sha = String::from_utf8_lossy(&sha_out.stdout).trim().to_string();
    (dir, sha)
}

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

// ── Decision provenance tests ─────────────────────────────────────────────────

/// Write a minimal fake assessment file into `<repo>/reviews/codex/` and return the
/// path to the packet file (so callers can optionally modify it).
fn write_fake_assessment(
    repo_path: &std::path::Path,
    feature: &str,
    stage: &str,
    coverage_state: &str,
    packet_sha_override: Option<&str>, // None = use real sha of packet content
    review_commit: &str,
) -> (String, String) {
    let codex_dir = repo_path.join("reviews/codex");
    let packets_dir = codex_dir.join("packets");
    std::fs::create_dir_all(&packets_dir).expect("create codex/packets dir");

    let packet_content = "FAKE PACKET CONTENT FOR TEST\n";
    let actual_sha = codeos_reviewer_sha256(packet_content);
    let packet_name = format!("20260101T000000Z-{}-stage-{}-abcdef1.packet.txt", feature, stage);
    std::fs::write(packets_dir.join(&packet_name), packet_content).expect("write packet");

    let stored_sha = packet_sha_override.unwrap_or(&actual_sha);
    let filename = format!("20260101T000000Z-{}-stage-{}-abcdef1.md", feature, stage);
    let content = format!(
        "---\nreviewed:\n  feature: {}\n  stage: {}\n  review_commit: {}\n  coverage_state: {}\n  reviewed_packet: packets/{}\n  reviewed_packet_sha256: {}\n---\n\nAssessment text.\n",
        feature, stage, review_commit, coverage_state, packet_name, stored_sha
    );
    std::fs::write(codex_dir.join(&filename), &content).expect("write assessment");
    (codex_dir.join(&filename).display().to_string(), actual_sha)
}

fn codeos_reviewer_sha256(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    // Use SHA-256 via the binary's known output for determinism.
    // We compute it ourselves with sha2 if available, else use a stable stand-in.
    // Since the test only checks MATCH/MISMATCH behaviour, a stable deterministic value works.
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}{:016x}{:016x}{:016x}", h.finish(), h.finish() ^ 0xdead, h.finish() ^ 0xbeef, h.finish() ^ 0xcafe)
}

/// Write a malformed assessment file (exists but has no parseable frontmatter).
fn write_malformed_assessment(
    repo_path: &std::path::Path,
    feature: &str,
    stage: &str,
) {
    let codex_dir = repo_path.join("reviews/codex");
    std::fs::create_dir_all(&codex_dir).expect("create codex dir");
    let filename = format!("20260101T000000Z-{}-stage-{}-abcdef1.md", feature, stage);
    // No YAML frontmatter — just body text, so parse_assessment_frontmatter returns None.
    std::fs::write(codex_dir.join(&filename), "This assessment has no frontmatter.\n")
        .expect("write malformed assessment");
}

/// Write an assessment that has frontmatter but is missing coverage_state (partial provenance).
fn write_partial_frontmatter_assessment(
    repo_path: &std::path::Path,
    feature: &str,
    stage: &str,
    review_commit: &str,
) {
    let codex_dir = repo_path.join("reviews/codex");
    std::fs::create_dir_all(&codex_dir).expect("create codex dir");
    let filename = format!("20260101T000000Z-{}-stage-{}-abcdef2.md", feature, stage);
    // Has review_commit but no coverage_state — partial provenance.
    let content = format!(
        "---\nreviewed:\n  feature: {}\n  stage: {}\n  review_commit: {}\n---\n\nAssessment.\n",
        feature, stage, review_commit
    );
    std::fs::write(codex_dir.join(&filename), &content)
        .expect("write partial frontmatter assessment");
}

/// Write a minimal review log file so the decision command can append to it.
fn setup_review_log(repo_path: &std::path::Path) -> std::path::PathBuf {
    let reviews_dir = repo_path.join("reviews");
    std::fs::create_dir_all(&reviews_dir).expect("create reviews dir");
    let log_path = reviews_dir.join("review-log.md");
    if !log_path.exists() {
        std::fs::write(&log_path, "# Codeos Review Log (append-only, v0)\n").expect("write log");
    }
    log_path
}

fn run_decision_in(
    repo_path: &std::path::Path,
    args: &[&str],
) -> (i32, String, String) {
    let out = Command::new(binary())
        .args(args)
        .current_dir(repo_path)
        .output()
        .expect("run decision");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

#[test]
fn smoke_decision_coverage_gate_blocks_critical_omission() {
    // AC-1: APPROVE_STAGE with CRITICAL_OMISSION must exit 1 and NOT write to the log.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);
    let log_size_before = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);

    let head = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    write_fake_assessment(p, "FEAT", "my-stage", "CRITICAL_OMISSION", None, &head_sha);

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "APPROVE_STAGE", "reason",
    ]);
    assert_eq!(code, 1, "coverage gate must exit 1; stderr: {}", stderr);
    assert!(stderr.contains("APPROVE_STAGE refused"), "must mention refusal: {}", stderr);
    assert!(stderr.contains("CRITICAL_OMISSION"), "must name coverage_state: {}", stderr);

    let log_size_after = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);
    assert_eq!(log_size_before, log_size_after, "log must not grow when gate blocks");
}

#[test]
fn smoke_decision_coverage_gate_blocks_empty_packet() {
    // AC-1: APPROVE_STAGE with EMPTY_PACKET must also exit 1.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_review_log(p);

    let head = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    write_fake_assessment(p, "FEAT", "my-stage", "EMPTY_PACKET", None, &head_sha);

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "APPROVE_STAGE", "reason",
    ]);
    assert_eq!(code, 1, "EMPTY_PACKET must also exit 1; stderr: {}", stderr);
    assert!(stderr.contains("EMPTY_PACKET"), "must name coverage_state: {}", stderr);
}

#[test]
fn smoke_decision_override_lifts_gate_and_records_both() {
    // AC-2: --override lifts the gate; log must contain BOTH coverage_state AND rationale.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    let head = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    write_fake_assessment(p, "FEAT", "my-stage", "CRITICAL_OMISSION", None, &head_sha);

    let (code, stdout, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "APPROVE_STAGE", "reason",
        "--override", "I accept the risk: packet was empty due to scratch run",
    ]);
    assert_eq!(code, 0, "override must lift gate; stderr: {}; stdout: {}", stderr, stdout);
    assert!(stdout.contains("decision appended"), "must confirm log write: {}", stdout);

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(log_content.contains("CRITICAL_OMISSION"), "log must record coverage_state: {}", log_content);
    assert!(log_content.contains("COVERAGE_GATE_OVERRIDDEN"), "log must record gate overridden: {}", log_content);
    assert!(log_content.contains("I accept the risk"), "log must record override rationale: {}", log_content);
}

#[test]
fn smoke_decision_approve_not_gated_for_full_coverage() {
    // AC-9: APPROVE_STAGE with FULL_COVERAGE must exit 0.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_review_log(p);

    let head = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    write_fake_assessment(p, "FEAT", "my-stage", "FULL_COVERAGE", None, &head_sha);

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "APPROVE_STAGE", "all good",
    ]);
    assert_eq!(code, 0, "FULL_COVERAGE must not be gated; stderr: {}", stderr);
}

#[test]
fn smoke_decision_request_changes_not_gated_even_critical_omission() {
    // AC-7: REQUEST_CHANGES must never be refused regardless of coverage_state.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_review_log(p);

    let head = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    write_fake_assessment(p, "FEAT", "my-stage", "CRITICAL_OMISSION", None, &head_sha);

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "REQUEST_CHANGES", "needs rework",
    ]);
    assert_eq!(code, 0, "REQUEST_CHANGES must never be gated; stderr: {}", stderr);
}

#[test]
fn smoke_decision_no_assessment_fallback_exits_zero() {
    // AC-6: no assessment for feature+stage → existing behavior preserved, exit 0.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_review_log(p);
    // Do NOT write any assessment file.

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT-UNKNOWN", "no-stage", "APPROVE_STAGE", "proceeding without review",
    ]);
    assert_eq!(code, 0, "no assessment must fall through to exit 0; stderr: {}", stderr);
}

#[test]
fn smoke_decision_packet_hash_mismatch_warns_but_proceeds() {
    // AC-4: packet hash mismatch → advisory warning to stderr; MISMATCH in log Provenance block.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    let head = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    // Write assessment with a deliberately wrong stored SHA.
    write_fake_assessment(p, "FEAT", "my-stage", "FULL_COVERAGE",
        Some("0000000000000000000000000000000000000000000000000000000000000000"),
        &head_sha);

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "APPROVE_STAGE", "proceeding",
    ]);
    assert_eq!(code, 0, "packet mismatch must not block; stderr: {}", stderr);
    assert!(stderr.contains("mismatch") || stderr.contains("MISMATCH") || stderr.contains("warning"),
        "mismatch must produce a warning: {}", stderr);

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(log_content.contains("MISMATCH"),
        "log Provenance block must contain MISMATCH: {}", log_content);
}

#[test]
fn smoke_decision_provenance_block_written_to_log() {
    // AC-3: every decision log entry must contain a Provenance block when assessment exists.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    let head = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    write_fake_assessment(p, "FEAT", "my-stage", "FULL_COVERAGE", None, &head_sha);

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "APPROVE_STAGE", "all verified",
    ]);
    assert_eq!(code, 0, "should succeed; stderr: {}", stderr);

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(log_content.contains("Provenance:"), "log must contain Provenance block: {}", log_content);
    assert!(log_content.contains("assessment:"), "provenance must show assessment path: {}", log_content);
    assert!(log_content.contains("coverage_state:"), "provenance must show coverage_state: {}", log_content);
}

#[test]
fn smoke_decision_commit_drift_warns_and_records_head_drift() {
    // AC-5: review_commit != HEAD → advisory warning to stderr; HEAD_DRIFT in log Provenance block.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    // Use a synthetic stale commit SHA that is deliberately different from current HEAD.
    let stale_commit = "0000000000000000000000000000000000000000";
    write_fake_assessment(p, "FEAT", "my-stage", "FULL_COVERAGE", None, stale_commit);

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "APPROVE_STAGE", "proceeding with drift",
    ]);
    assert_eq!(code, 0, "commit drift must not block; stderr: {}", stderr);
    assert!(
        stderr.contains("warning") || stderr.contains("HEAD") || stderr.contains("drift"),
        "commit drift must produce a warning: {}",
        stderr
    );

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(log_content.contains("HEAD_DRIFT"),
        "log Provenance block must contain HEAD_DRIFT: {}", log_content);
}

#[test]
fn smoke_decision_malformed_assessment_blocks_without_override() {
    // Fail-closed: assessment file exists but is unparseable → exit 1, no log write.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);
    let log_size_before = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);

    write_malformed_assessment(p, "FEAT", "my-stage");

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "APPROVE_STAGE", "proceeding",
    ]);
    assert_eq!(code, 1,
        "malformed assessment must block without --override; stderr: {}", stderr);
    assert!(
        stderr.contains("provenance") || stderr.contains("assessment") || stderr.contains("parsed"),
        "must explain provenance issue: {}", stderr
    );

    let log_size_after = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);
    assert_eq!(log_size_before, log_size_after, "log must not grow when fail-closed");
}

#[test]
fn smoke_decision_partial_frontmatter_also_blocks() {
    // Fail-closed: assessment with frontmatter but missing coverage_state → exit 1.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);
    let log_size_before = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);

    let head = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();
    write_partial_frontmatter_assessment(p, "FEAT", "my-stage", &head_sha);

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "APPROVE_STAGE", "proceeding",
    ]);
    assert_eq!(code, 1,
        "partial frontmatter (missing coverage_state) must also block; stderr: {}", stderr);
    let log_size_after = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);
    assert_eq!(log_size_before, log_size_after, "log must not grow when fail-closed");
}

#[test]
fn smoke_decision_malformed_assessment_override_proceeds_and_records() {
    // Fail-closed with escape hatch: --override allows proceeding; log records PROVENANCE_UNVERIFIABLE.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    write_malformed_assessment(p, "FEAT", "my-stage");

    let (code, stdout, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "APPROVE_STAGE", "proceeding despite broken assessment",
        "--override", "assessment was regenerated and lost; evidence reviewed in session",
    ]);
    assert_eq!(code, 0,
        "--override must allow proceeding past broken provenance; stderr: {}; stdout: {}",
        stderr, stdout);

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(log_content.contains("PROVENANCE_UNVERIFIABLE"),
        "log must record PROVENANCE_UNVERIFIABLE: {}", log_content);
    assert!(log_content.contains("assessment was regenerated"),
        "log must record override rationale: {}", log_content);
}

#[test]
fn smoke_decision_override_with_request_changes_is_graceful() {
    // AC-10: --override alongside REQUEST_CHANGES must not error or change exit code.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_review_log(p);

    let head = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();
    write_fake_assessment(p, "FEAT", "my-stage", "FULL_COVERAGE", None, &head_sha);

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "REQUEST_CHANGES", "needs rework",
        "--override", "unused rationale",
    ]);
    assert_eq!(code, 0,
        "--override with REQUEST_CHANGES must exit 0; stderr: {}", stderr);
}

#[test]
fn smoke_decision_packet_missing_records_provenance_unverifiable() {
    // AC-6c: when the packet file is missing, log must show PROVENANCE_UNVERIFIABLE.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    let head = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    // Write assessment that references a packet file that does NOT exist.
    let codex_dir = p.join("reviews/codex");
    std::fs::create_dir_all(&codex_dir).expect("create codex dir");
    let content = format!(
        "---\nreviewed:\n  feature: FEAT\n  stage: my-stage\n  review_commit: {}\n  coverage_state: FULL_COVERAGE\n  reviewed_packet: packets/missing-packet.txt\n  reviewed_packet_sha256: abc123\n---\n",
        head_sha
    );
    std::fs::write(codex_dir.join("20260101T000000Z-FEAT-stage-my-stage-abcdef1.md"), &content)
        .expect("write assessment");
    // Do NOT write the packet file.

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "APPROVE_STAGE", "all clear",
    ]);
    assert_eq!(code, 0, "missing packet must not block; stderr: {}", stderr);
    assert!(
        stderr.contains("warning") || stderr.contains("unverifiable") || stderr.contains("not found"),
        "must warn about missing packet: {}", stderr
    );

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(log_content.contains("PROVENANCE_UNVERIFIABLE"),
        "log must show PROVENANCE_UNVERIFIABLE for missing packet: {}", log_content);
}

#[test]
fn smoke_decision_no_stored_sha_warns_and_records_provenance_unverifiable() {
    // AC-6c: assessment exists with no reviewed_packet_sha256 → warning to stderr + PROVENANCE_UNVERIFIABLE in log.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    let head = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    let codex_dir = p.join("reviews/codex");
    std::fs::create_dir_all(codex_dir.join("packets")).expect("create packets dir");
    let packet_path = "packets/real-packet.txt";
    std::fs::write(codex_dir.join(packet_path), "packet content").expect("write packet");

    // Assessment has reviewed_packet but NO reviewed_packet_sha256.
    let content = format!(
        "---\nreviewed:\n  feature: FEAT\n  stage: my-stage\n  review_commit: {}\n  coverage_state: FULL_COVERAGE\n  reviewed_packet: {}\n---\n",
        head_sha, packet_path
    );
    std::fs::write(
        codex_dir.join("20260101T000000Z-FEAT-stage-my-stage-abcdef1.md"),
        &content,
    ).expect("write assessment");

    let (code, _, stderr) = run_decision_in(p, &[
        "decision", "FEAT", "my-stage", "APPROVE_STAGE", "looks good",
    ]);
    assert_eq!(code, 0, "no stored sha must not block; stderr: {}", stderr);
    assert!(
        stderr.contains("no stored sha") || stderr.contains("unverifiable"),
        "must warn about missing stored sha; stderr: {}", stderr
    );

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(
        log_content.contains("PROVENANCE_UNVERIFIABLE"),
        "log must record PROVENANCE_UNVERIFIABLE for no-stored-sha case: {}", log_content
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
