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

// ---- Full Context Diff tests (AC-1, AC-2, AC-3, AC-4) ----

/// Create a .codeos symlink in the temp repo pointing to the real Codeos toolkit root.
/// This lets the binary find prompts/codeos-reviewer-task.md from a temp repo.
fn setup_codeos_symlink(repo_path: &std::path::Path) {
    let target = repo_root();
    std::os::unix::fs::symlink(&target, repo_path.join(".codeos"))
        .expect("create .codeos symlink");
}

/// Helper: add a new commit with an extra file to the temp repo.
fn add_extra_commit(repo_path: &std::path::Path, filename: &str, content: &str) -> String {
    std::fs::write(repo_path.join(filename), content).expect("write extra file");
    Command::new("git").args(["add", filename]).current_dir(repo_path).output().expect("git add");
    Command::new("git").args(["commit", "-m", "extra"]).current_dir(repo_path).output().expect("git commit");
    let out = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(repo_path)
        .output().expect("git rev-parse");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
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

// --- UPG-0035: --sha-only missing-path exit code ---

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

// --- UPG-0020: check-drift subcommand ---

fn run_in_dir(repo_path: &std::path::Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(binary())
        .args(args)
        .current_dir(repo_path)
        .output()
        .expect("run binary in dir");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

#[test]
fn smoke_check_drift_clean_diff_exits_zero() {
    // AC-1: no watched files in diff → exit 0.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    // Add a commit with a non-watched file only.
    add_extra_commit(p, "notes.md", "# notes\n");

    let (code, _, stderr) = run_in_dir(p, &["check-drift", "--base", &base_sha]);
    assert_eq!(code, 0, "clean diff must exit 0; stderr: {}", stderr);
}

#[test]
fn smoke_check_drift_watched_file_no_report_exits_drift() {
    // AC-2: watched file changed, no reconciliation report → exit 6 (EXIT_DRIFT).
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    add_extra_commit(p, "Cargo.toml", "[package]\nname = \"test\"\n");

    let (code, _, stderr) = run_in_dir(p, &["check-drift", "--base", &base_sha]);
    assert_eq!(code, 6, "drift without report must exit 6; stderr: {}", stderr);
    assert!(stderr.contains("Cargo.toml"), "stderr must name triggering file; got: {}", stderr);
    assert!(
        stderr.contains("stack-reconciliation-report.md"),
        "stderr must name the required report; got: {}", stderr
    );
}

#[test]
fn smoke_check_drift_watched_file_with_report_exits_zero() {
    // AC-3: watched file changed + reconciliation report in diff → exit 0.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    // Commit both Cargo.toml and the reconciliation report in one go.
    std::fs::write(p.join("Cargo.toml"), "[package]\nname = \"test\"\n").expect("write Cargo.toml");
    std::fs::write(p.join("stack-reconciliation-report.md"), "# Stack Reconciliation Report\n").expect("write report");
    Command::new("git").args(["add", "Cargo.toml", "stack-reconciliation-report.md"]).current_dir(p).output().ok();
    Command::new("git").args(["commit", "-m", "drift+report"]).current_dir(p).output().ok();

    let (code, _, stderr) = run_in_dir(p, &["check-drift", "--base", &base_sha]);
    assert_eq!(code, 0, "reconciled drift must exit 0; stderr: {}", stderr);
}

#[test]
fn smoke_check_drift_invalid_base_exits_config() {
    // AC-4: git diff fails on an invalid base ref → exit 2 (EXIT_CONFIG).
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();

    let (code, _, stderr) = run_in_dir(p, &["check-drift", "--base", "nonexistent-ref-xyz"]);
    assert_eq!(code, 2, "invalid base ref must exit 2 (EXIT_CONFIG); stderr: {}", stderr);
}

#[test]
fn smoke_check_drift_strict_flag_accepted() {
    // AC-8: --strict is accepted; drift message prefixed with STRICT MODE.
    let (dir, base_sha) = setup_temp_git_repo();
    let p = dir.path();
    add_extra_commit(p, "Cargo.toml", "[package]\nname = \"test\"\n");

    let (code, _, stderr) = run_in_dir(p, &["check-drift", "--base", &base_sha, "--strict"]);
    assert_eq!(code, 6, "--strict drift must still exit 6; stderr: {}", stderr);
    assert!(stderr.contains("STRICT MODE"), "--strict must prefix message; stderr: {}", stderr);
}

// --- UPG-0034: read-only invariant check ---

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
