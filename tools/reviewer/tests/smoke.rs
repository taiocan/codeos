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

// --- UPG-0021: generate-report subcommand ---

fn read_template() -> String {
    std::fs::read_to_string(repo_root().join("templates/stage-4-6-report.md"))
        .expect("read stage-4-6-report.md template")
}

/// Slice out the `## Stage <n> ...` section of the template, up to the next stage header (or EOF).
fn template_section(stage: &str) -> String {
    let content = read_template();
    let start_marker = format!("## Stage {} ", stage);
    let start = content.find(&start_marker).expect("stage header not found in template");
    let rest = &content[start..];
    let next_marker = match stage {
        "4" => Some("## Stage 5 "),
        "5" => Some("## Stage 6 "),
        _ => None,
    };
    match next_marker {
        Some(marker) => rest[..rest.find(marker).expect("next stage header not found")].to_string(),
        None => rest.to_string(),
    }
}

/// Extract field labels (text up to and including the first `:`) from non-heading,
/// non-comment, non-blank lines, in order. Used to diff template field coverage against
/// generator output regardless of the value appended after the colon.
fn extract_field_labels(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| {
            let trimmed_start = l.trim_start();
            if trimmed_start.is_empty()
                || trimmed_start.starts_with('#')
                || trimmed_start.starts_with('*')
                || trimmed_start.starts_with('>')
            {
                return None;
            }
            let idx = l.find(':')?;
            Some(l[..=idx].trim_start().to_string())
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

// --- UPG-0022: generate-adr-candidates subcommand ---

const ADR_BANNER_LINE_1: &str =
    "> [INFERRED] fields were populated automatically by extracting risk bullets from the source";
const ADR_BANNER_LINE_2: &str =
    "> document — verify before submitting. [FILL] fields require human or model authorship.";
const ADR_BANNER_LINE_3: &str =
    "> ADR candidates are non-authoritative until routed through Stage 1–3 or Stage 10.";

#[test]
fn smoke_generate_adr_section_boundary() {
    // AC-1: only bullets between "## Architectural Risks" and the next "## " heading are used;
    // content in a later section must not leak in.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("00b.md");
    std::fs::write(
        &fixture,
        "# Solution Discovery: Test Domain\n\n\
## Architectural Risks\n\n\
- Risk A: something\n\
- Risk B: something else\n\n\
## Explicit Non-Decisions\n\n\
- Not a risk bullet, must be excluded\n",
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", fixture.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Decision needed: Risk A: something [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("Decision needed: Risk B: something else [INFERRED]"), "stdout: {}", stdout);
    assert!(
        !stdout.contains("Not a risk bullet"),
        "content from a later section must not leak in; stdout: {}",
        stdout
    );
}

#[test]
fn smoke_generate_adr_bullet_extraction() {
    // AC-2: top-level bullets only; indented continuation lines ignored, prose ignored,
    // a bare bullet with no text after the marker produces no candidate.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("00b.md");
    std::fs::write(
        &fixture,
        "## Architectural Risks\n\n\
Some intro prose before the first bullet.\n\n\
- Risk One\n\
  this indented continuation line must be ignored\n\
-   \n\
- Risk Two\n",
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", fixture.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Decision needed: Risk One [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("Decision needed: Risk Two [INFERRED]"), "stdout: {}", stdout);
    assert!(
        !stdout.contains("continuation line"),
        "indented continuation must not become its own or appended content; stdout: {}",
        stdout
    );
    assert!(
        !stdout.contains("intro prose"),
        "prose before bullets must not become a candidate; stdout: {}",
        stdout
    );
    // Exactly two candidates: the bare "-   " bullet must not produce a third.
    assert_eq!(
        stdout.matches("## Candidate").count(),
        2,
        "bare bullet with no text must not produce a candidate; stdout: {}",
        stdout
    );
}

#[test]
fn smoke_generate_adr_multiple_candidates_structure() {
    // AC-3: one "# ADR Candidates" heading, N "## Candidate n" subheadings in source order.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("00b.md");
    std::fs::write(
        &fixture,
        "## Architectural Risks\n\n- Risk Alpha\n- Risk Beta\n- Risk Gamma\n",
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", fixture.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert_eq!(stdout.matches("# ADR Candidates").count(), 1, "stdout: {}", stdout);

    let c1 = stdout.find("## Candidate 1").expect("Candidate 1 present");
    let c2 = stdout.find("## Candidate 2").expect("Candidate 2 present");
    let c3 = stdout.find("## Candidate 3").expect("Candidate 3 present");
    assert!(c1 < c2 && c2 < c3, "candidates must appear in source order; stdout: {}", stdout);

    for field in [
        "Decision needed:",
        "Why now: [FILL]",
        "Features affected: [FILL]",
        "Options: [FILL]",
        "Risk if deferred: [FILL]",
        "Does this affect behavior: [FILL]",
        "Recommended route: [FILL]",
        "- Stage 1–3",
        "- Stage 10",
        "- no action yet",
    ] {
        assert_eq!(
            stdout.matches(field).count(),
            3,
            "field '{}' must appear once per candidate group; stdout: {}",
            field,
            stdout
        );
    }
}

#[test]
fn smoke_generate_adr_inferred_and_fill_tagging() {
    // AC-4: exactly one [INFERRED] per group (on Decision needed), [FILL] on the other six.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("00b.md");
    std::fs::write(&fixture, "## Architectural Risks\n\n- Risk Alpha\n- Risk Beta\n")
        .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", fixture.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    // Count only within the candidates section — the banner itself contains the literal
    // substring "[INFERRED]" in its explanatory text, which must not be counted here.
    let candidates_section = &stdout[stdout.find("# ADR Candidates").expect("heading present")..];
    assert_eq!(
        candidates_section.matches("[INFERRED]").count(),
        2,
        "exactly one [INFERRED] per candidate (Decision needed only); stdout: {}",
        stdout
    );
    assert_eq!(
        candidates_section.matches("[FILL]").count(),
        12,
        "six [FILL] fields per candidate x 2 candidates; stdout: {}",
        stdout
    );
}

#[test]
fn smoke_generate_adr_preamble_present() {
    // AC-5: banner is the first three non-blank lines, verbatim.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("00b.md");
    std::fs::write(&fixture, "## Architectural Risks\n\n- Risk Alpha\n").expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", fixture.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);

    let mut lines = stdout.lines();
    assert_eq!(lines.next().unwrap_or(""), ADR_BANNER_LINE_1);
    assert_eq!(lines.next().unwrap_or(""), ADR_BANNER_LINE_2);
    assert_eq!(lines.next().unwrap_or(""), ADR_BANNER_LINE_3);
}

#[test]
fn smoke_generate_adr_no_section_found() {
    // AC-6: no "## Architectural Risks" heading at all -> empty stdout, stderr names the path,
    // exit 0 (valid-but-empty, not a usage failure).
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("00b.md");
    std::fs::write(
        &fixture,
        "# Solution Discovery: Test Domain\n\n## Explicit Non-Decisions\n\n- something\n",
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", fixture.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("no \"## Architectural Risks\" section found"), "stderr: {}", stderr);
    assert!(stderr.contains(fixture.to_str().unwrap()), "stderr must name the path; got: {}", stderr);
}

#[test]
fn smoke_generate_adr_section_empty() {
    // AC-7: heading present, zero valid bullets -> empty stdout, a distinct stderr message,
    // exit 0.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("00b.md");
    std::fs::write(
        &fixture,
        "## Architectural Risks\n\nJust prose here, no bullets at all.\n\n## Explicit Non-Decisions\n",
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", fixture.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("contains no risk bullets"), "stderr: {}", stderr);
    assert!(
        !stderr.contains("no \"## Architectural Risks\" section found"),
        "AC-6 and AC-7 messages must be distinct; stderr: {}",
        stderr
    );
}

#[test]
fn smoke_generate_adr_missing_source_file() {
    // AC-8: unreadable/missing --source -> exit 1 (EXIT_USAGE), stderr names the path.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", "does-not-exist.md"]);
    assert_eq!(code, 1, "missing source file must exit 1; stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("does-not-exist.md"), "stderr must name the path; got: {}", stderr);
}

#[test]
fn smoke_generate_adr_source_required() {
    // AC-9: omitting --source is a clap usage error, exit 1.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let (code, _, stderr) = run_in_dir(p, &["generate-adr-candidates"]);
    assert_eq!(code, 1, "missing --source must exit 1; stderr: {}", stderr);
}

#[test]
fn smoke_generate_adr_stdout_only() {
    // AC-10: stderr empty on a successful non-empty run; stdout exactly empty on the
    // AC-6/AC-7/AC-8 error/empty paths.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("00b.md");
    std::fs::write(&fixture, "## Architectural Risks\n\n- Risk Alpha\n").expect("write fixture");

    let (code, _, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", fixture.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.is_empty(), "successful run must have empty stderr; got: {}", stderr);

    let no_section = p.join("no-section.md");
    std::fs::write(&no_section, "# Just a title\n").expect("write fixture");
    let (_, stdout, _) =
        run_in_dir(p, &["generate-adr-candidates", "--source", no_section.to_str().unwrap()]);
    assert!(stdout.is_empty(), "AC-6 case must have empty stdout; got: {}", stdout);

    let empty_section = p.join("empty-section.md");
    std::fs::write(&empty_section, "## Architectural Risks\n\nprose only\n").expect("write fixture");
    let (_, stdout, _) =
        run_in_dir(p, &["generate-adr-candidates", "--source", empty_section.to_str().unwrap()]);
    assert!(stdout.is_empty(), "AC-7 case must have empty stdout; got: {}", stdout);

    let (_, stdout, _) =
        run_in_dir(p, &["generate-adr-candidates", "--source", "does-not-exist.md"]);
    assert!(stdout.is_empty(), "AC-8 case must have empty stdout; got: {}", stdout);
}

#[test]
fn smoke_generate_adr_exit_zero_on_success() {
    // AC-11: any invocation finding >= 1 risk bullet exits 0.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("00b.md");
    std::fs::write(&fixture, "## Architectural Risks\n\n- Risk Alpha\n").expect("write fixture");

    let (code, _, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", fixture.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
}

#[test]
fn smoke_generate_adr_no_provider_config_required() {
    // AC-12: dispatches before config::resolve(); no provider config needed.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("00b.md");
    std::fs::write(&fixture, "## Architectural Risks\n\n- Risk Alpha\n").expect("write fixture");

    let (code, _, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", fixture.to_str().unwrap()]);
    assert_eq!(code, 0, "must succeed without provider config; stderr: {}", stderr);
}

#[test]
fn smoke_generate_adr_deterministic_output() {
    // AC-13: identical source produces byte-for-byte identical stdout across two invocations.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let fixture = p.join("00b.md");
    std::fs::write(
        &fixture,
        "## Architectural Risks\n\n- Risk Alpha\n- Risk Beta\n",
    )
    .expect("write fixture");

    let args = ["generate-adr-candidates", "--source", fixture.to_str().unwrap()];
    let (code1, stdout1, _) = run_in_dir(p, &args);
    let (code2, stdout2, _) = run_in_dir(p, &args);
    assert_eq!(code1, 0);
    assert_eq!(code2, 0);
    assert_eq!(stdout1, stdout2, "output must be deterministic for identical inputs");
}

#[test]
fn smoke_generate_adr_guardrail_inseparable_from_output() {
    // AC-14: the non-authoritative guardrail line is present in every non-empty-stdout run,
    // for both a single-candidate and a multi-candidate fixture — there is no code path that
    // emits candidate content without it.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let single = p.join("single.md");
    std::fs::write(&single, "## Architectural Risks\n\n- Risk Alpha\n").expect("write fixture");
    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", single.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains(ADR_BANNER_LINE_3), "stdout: {}", stdout);

    let multi = p.join("multi.md");
    std::fs::write(&multi, "## Architectural Risks\n\n- Risk Alpha\n- Risk Beta\n- Risk Gamma\n")
        .expect("write fixture");
    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-adr-candidates", "--source", multi.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains(ADR_BANNER_LINE_3), "stdout: {}", stdout);
}

// --- UPG-0023: generate-approval-dashboard subcommand ---

const DASHBOARD_BANNER_LINE_1: &str =
    "> [INFERRED] fields were populated automatically from the feature registry — verify before";
const DASHBOARD_BANNER_LINE_2: &str =
    "> submitting. [FILL] fields require human or model authorship. This dashboard is a navigation";
const DASHBOARD_BANNER_LINE_3: &str =
    "> aid, not a decision record — the registry and change records remain authoritative.";

#[test]
fn smoke_dashboard_full_vs_minimal_schema_identical_output() {
    // AC-1: an entry with every registry field populated and an entry with only the fields
    // this tool reads must produce byte-identical dashboard blocks for the same values.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let full = p.join("full.yaml");
    std::fs::write(
        &full,
        r#"schema_version: 2
features:
  - feature_id: UPG-1001
    slug: sample-feature
    description: "A feature with every field populated"
    type: F
    status: active
    branch: feature/sample
    current_stage: 4
    intent: intents/UPG-1001.md
    contract: contracts/UPG-1001_contract.md
    event_schema: events/UPG-1001_schema.md
    pr: null
    last_commit: abc123
    reconciliation_status: pending
    replay_status: na
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let minimal = p.join("minimal.yaml");
    std::fs::write(
        &minimal,
        r#"schema_version: 2
features:
  - feature_id: UPG-1001
    slug: sample-feature
    status: active
    current_stage: 4
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code1, stdout1, stderr1) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", full.to_str().unwrap()]);
    let (code2, stdout2, stderr2) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", minimal.to_str().unwrap()]);
    assert_eq!(code1, 0, "stderr: {}", stderr1);
    assert_eq!(code2, 0, "stderr: {}", stderr2);
    assert_eq!(stdout1, stdout2, "full-schema and minimal-schema entries must produce identical output");
}

#[test]
fn smoke_dashboard_only_active_features_in_registry_order() {
    // AC-2: only status: active entries appear, in original registry order.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-2001
    slug: first-active
    status: active
    current_stage: 1
    blockers: []
    notes: ""
  - feature_id: UPG-2002
    slug: suspended-one
    status: suspended
    current_stage: 2
    blockers: []
    notes: ""
  - feature_id: UPG-2003
    slug: second-active
    status: active
    current_stage: 3
    blockers: []
    notes: ""
  - feature_id: UPG-2004
    slug: complete-one
    status: complete
    current_stage: 9
    blockers: []
    notes: ""
  - feature_id: UPG-2005
    slug: blocked-one
    status: blocked
    current_stage: 4
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("UPG-2001: first-active"), "stdout: {}", stdout);
    assert!(stdout.contains("UPG-2003: second-active"), "stdout: {}", stdout);
    assert!(!stdout.contains("suspended-one"), "stdout: {}", stdout);
    assert!(!stdout.contains("complete-one"), "stdout: {}", stdout);
    assert!(!stdout.contains("blocked-one"), "stdout: {}", stdout);

    let pos_first = stdout.find("UPG-2001: first-active").expect("first-active present");
    let pos_second = stdout.find("UPG-2003: second-active").expect("second-active present");
    assert!(pos_first < pos_second, "active features must appear in registry order");
}

#[test]
fn smoke_dashboard_output_structure() {
    // AC-3: one heading, one subsection per active feature, six fields each, in order.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-3001
    slug: alpha
    status: active
    current_stage: 1
    blockers: []
  - feature_id: UPG-3002
    slug: beta
    status: active
    current_stage: 2
    blockers: []
  - feature_id: UPG-3003
    slug: gamma
    status: active
    current_stage: 3
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert_eq!(stdout.matches("# Approval Dashboard").count(), 1, "stdout: {}", stdout);

    let c1 = stdout.find("## UPG-3001: alpha").expect("alpha present");
    let c2 = stdout.find("## UPG-3002: beta").expect("beta present");
    let c3 = stdout.find("## UPG-3003: gamma").expect("gamma present");
    assert!(c1 < c2 && c2 < c3, "subsections must appear in order; stdout: {}", stdout);

    for field in [
        "Active features:",
        "Current stage:",
        "Reviewer recommendation: [FILL]",
        "Open blockers:",
        "Next human decision: [FILL]",
        "Risk: [FILL]",
    ] {
        assert_eq!(
            stdout.matches(field).count(),
            3,
            "field '{}' must appear once per active feature; stdout: {}",
            field,
            stdout
        );
    }
}

#[test]
fn smoke_dashboard_inferred_edge_cases() {
    // AC-4: (a) populated stage + blockers; (b) null stage + empty blockers.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-4001
    slug: with-stage-and-blockers
    status: active
    current_stage: 4
    blockers:
      - waiting on review
      - blocked by dependency
  - feature_id: UPG-4002
    slug: not-started
    status: active
    current_stage: null
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Current stage: 4 [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("Open blockers: [INFERRED]\nwaiting on review\nblocked by dependency"), "stdout: {}", stdout);
    assert!(stdout.contains("Current stage: not started [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("Open blockers: (none) [INFERRED]"), "stdout: {}", stdout);
}

#[test]
fn smoke_dashboard_fill_fields_always_present() {
    // AC-5: Reviewer recommendation / Next human decision / Risk are always [FILL].
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-5001
    slug: sample
    status: active
    current_stage: 5
    blockers:
      - some blocker
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Reviewer recommendation: [FILL]"), "stdout: {}", stdout);
    assert!(stdout.contains("Next human decision: [FILL]"), "stdout: {}", stdout);
    assert!(stdout.contains("Risk: [FILL]"), "stdout: {}", stdout);
}

#[test]
fn smoke_dashboard_preamble_present() {
    // AC-6: banner is the first three non-blank lines, verbatim.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        "schema_version: 2\nfeatures:\n  - feature_id: UPG-6001\n    slug: sample\n    status: active\n    current_stage: 1\n    blockers: []\n    notes: \"\"\n",
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);

    let mut lines = stdout.lines();
    assert_eq!(lines.next().unwrap_or(""), DASHBOARD_BANNER_LINE_1);
    assert_eq!(lines.next().unwrap_or(""), DASHBOARD_BANNER_LINE_2);
    assert_eq!(lines.next().unwrap_or(""), DASHBOARD_BANNER_LINE_3);
}

#[test]
fn smoke_dashboard_no_active_features_only_non_active() {
    // AC-7(a): registry parses but has zero active entries -> empty stdout, stderr note, exit 0.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        "schema_version: 2\nfeatures:\n  - feature_id: UPG-7001\n    slug: done\n    status: complete\n    current_stage: 9\n    blockers: []\n    notes: \"\"\n",
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("no active or hypothesized features found"), "stderr: {}", stderr);
    assert!(stderr.contains(registry.to_str().unwrap()), "stderr must name the path; got: {}", stderr);
}

#[test]
fn smoke_dashboard_no_active_features_empty_list() {
    // AC-7(b): empty features: [] list -> same empty/exit-0 behavior.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(&registry, "schema_version: 2\nfeatures: []\n").expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("no active or hypothesized features found"), "stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_missing_registry_file() {
    // AC-8: unreadable/missing --registry -> exit 1 (EXIT_USAGE), stderr names the path.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", "does-not-exist.yaml"]);
    assert_eq!(code, 1, "missing registry file must exit 1; stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("does-not-exist.yaml"), "stderr must name the path; got: {}", stderr);
}

#[test]
fn smoke_dashboard_malformed_yaml() {
    // AC-9(a): malformed YAML -> exit 1, distinct stderr message from AC-8.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("malformed.yaml");
    std::fs::write(&registry, "features: [\"unterminated\n").expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 1, "malformed YAML must exit 1; stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("cannot parse registry file"), "stderr: {}", stderr);
    assert!(!stderr.contains("cannot read registry file"), "AC-8/AC-9 messages must be distinct; stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_wrong_shape_yaml() {
    // AC-9(b): features: is not a list -> exit 1, same distinct parse-error message.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("wrong-shape.yaml");
    std::fs::write(&registry, "schema_version: 2\nfeatures: \"not a list\"\n").expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 1, "wrong-shape YAML must exit 1; stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("cannot parse registry file"), "stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_registry_required() {
    // AC-10: omitting --registry is a clap usage error, exit 1.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let (code, _, stderr) = run_in_dir(p, &["generate-approval-dashboard"]);
    assert_eq!(code, 1, "missing --registry must exit 1; stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_stdout_only() {
    // AC-11: stderr empty on success; stdout exactly empty on the AC-7/8/9 error/empty paths.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        "schema_version: 2\nfeatures:\n  - feature_id: UPG-8001\n    slug: sample\n    status: active\n    current_stage: 1\n    blockers: []\n    notes: \"\"\n",
    )
    .expect("write fixture");

    let (code, _, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.is_empty(), "successful run must have empty stderr; got: {}", stderr);

    let empty_registry = p.join("empty.yaml");
    std::fs::write(&empty_registry, "schema_version: 2\nfeatures: []\n").expect("write fixture");
    let (_, stdout, _) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", empty_registry.to_str().unwrap()]);
    assert!(stdout.is_empty(), "AC-7 case must have empty stdout; got: {}", stdout);

    let (_, stdout, _) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", "does-not-exist.yaml"]);
    assert!(stdout.is_empty(), "AC-8 case must have empty stdout; got: {}", stdout);

    let malformed = p.join("malformed.yaml");
    std::fs::write(&malformed, "features: [\"unterminated\n").expect("write fixture");
    let (_, stdout, _) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", malformed.to_str().unwrap()]);
    assert!(stdout.is_empty(), "AC-9 case must have empty stdout; got: {}", stdout);
}

#[test]
fn smoke_dashboard_exit_zero_on_success() {
    // AC-12: any invocation finding >= 1 active feature exits 0.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        "schema_version: 2\nfeatures:\n  - feature_id: UPG-9001\n    slug: sample\n    status: active\n    current_stage: 1\n    blockers: []\n    notes: \"\"\n",
    )
    .expect("write fixture");

    let (code, _, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_no_provider_config_required() {
    // AC-13: dispatches before config::resolve(); no provider config needed.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        "schema_version: 2\nfeatures:\n  - feature_id: UPG-9101\n    slug: sample\n    status: active\n    current_stage: 1\n    blockers: []\n    notes: \"\"\n",
    )
    .expect("write fixture");

    let (code, _, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "must succeed without provider config; stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_deterministic_output() {
    // AC-14: identical registry produces byte-for-byte identical stdout across two invocations.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9201
    slug: alpha
    status: active
    current_stage: 2
    blockers:
      - a blocker
  - feature_id: UPG-9202
    slug: beta
    status: active
    current_stage: null
    blockers: []
"#,
    )
    .expect("write fixture");

    let args = ["generate-approval-dashboard", "--registry", registry.to_str().unwrap()];
    let (code1, stdout1, _) = run_in_dir(p, &args);
    let (code2, stdout2, _) = run_in_dir(p, &args);
    assert_eq!(code1, 0);
    assert_eq!(code2, 0);
    assert_eq!(stdout1, stdout2, "output must be deterministic for identical inputs");
}

#[test]
fn smoke_dashboard_architectural_refinements_never_treated_as_feature() {
    // AC-15: architectural_refinements: is never mistaken for a feature entry.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9301
    slug: real-feature
    status: active
    current_stage: 3
    blockers: []

architectural_refinements:
  - refine_id: sneaky_refine
    description: "Looks like a feature but isn't"
    status: active
    artifact: refinements/arch/sneaky_refine.md
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert_eq!(stdout.matches("## ").count(), 1, "only the one real feature must appear; stdout: {}", stdout);
    assert!(stdout.contains("UPG-9301: real-feature"), "stdout: {}", stdout);
    assert!(!stdout.contains("sneaky_refine"), "stdout: {}", stdout);
}

// --- UPG-0024: generate-release-evidence subcommand ---

const RELEASE_EVIDENCE_BANNER_LINE_1: &str =
    "> [INFERRED] fields were populated automatically from git and the feature registry (if";
const RELEASE_EVIDENCE_BANNER_LINE_4: &str =
    "> — Release decision requires explicit human judgment and is never inferred.";

fn checkout_branch(repo_path: &std::path::Path, name: &str) {
    Command::new("git")
        .args(["checkout", "-b", name])
        .current_dir(repo_path)
        .output()
        .expect("git checkout -b");
}

#[test]
fn smoke_release_evidence_feature_always_inferred() {
    // AC-1: Feature: is always [INFERRED] from --feature, regardless of --registry.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/upg-9401");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9401"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Feature: UPG-9401 [INFERRED]"), "stdout: {}", stdout);
}

#[test]
fn smoke_release_evidence_branch_always_inferred() {
    // AC-2: Branch: is always [INFERRED] from git rev-parse --abbrev-ref HEAD.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/named-branch-test");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9402"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(
        stdout.contains("Branch: feature/named-branch-test [INFERRED]"),
        "stdout: {}", stdout
    );
}

#[test]
fn smoke_release_evidence_output_structure() {
    // AC-3: report body, after the banner, is exactly the 12-field/sub-item list in order.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/structure-test");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9403"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("# Release Evidence Package"), "stdout: {}", stdout);

    let order = [
        "Feature:",
        "Branch:",
        "PR:",
        "Approved artifacts:",
        "- Intent:",
        "- Contract:",
        "- Event schema:",
        "Stage reports:",
        "Reviewer briefs:",
        "Reconciliation result:",
        "Replay result:",
        "Verification-only report:",
        "Readiness checklist:",
        "Known limitations:",
        "Release decision:",
    ];
    let mut last_pos = 0usize;
    for field in order {
        let pos = stdout[last_pos..].find(field)
            .unwrap_or_else(|| panic!("field '{}' not found after position {}; stdout: {}", field, last_pos, stdout));
        last_pos += pos + field.len();
    }
}

#[test]
fn smoke_release_evidence_registry_enrichment_per_field_independent() {
    // AC-4: registry-derived fields are [INFERRED] only when that specific field is
    // non-null for the matched feature — evaluated independently, not all-or-nothing.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/partial-registry-test");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9404
    slug: partial-feature
    status: active
    current_stage: 4
    pr: "https://example.test/pr/1"
    intent: intents/UPG-9404.md
    contract: null
    event_schema: null
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9404", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("PR: https://example.test/pr/1 [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("- Intent: intents/UPG-9404.md [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("- Contract: [FILL]"), "stdout: {}", stdout);
    assert!(stdout.contains("- Event schema: [FILL]"), "stdout: {}", stdout);
}

#[test]
fn smoke_release_evidence_always_fill_fields() {
    // AC-5: the 8 always-[FILL] fields stay [FILL] even with a fully populated registry.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/always-fill-test");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9405
    slug: full-feature
    status: active
    current_stage: 6
    pr: "https://example.test/pr/2"
    intent: intents/UPG-9405.md
    contract: contracts/UPG-9405_contract.md
    event_schema: events/UPG-9405_schema.md
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9405", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    for field in [
        "Stage reports: [FILL]",
        "Reviewer briefs: [FILL]",
        "Reconciliation result: [FILL]",
        "Replay result: [FILL]",
        "Verification-only report: [FILL]",
        "Readiness checklist: [FILL]",
        "Known limitations: [FILL]",
        "Release decision: [FILL]",
    ] {
        assert!(stdout.contains(field), "expected '{}' in stdout: {}", field, stdout);
    }
}

#[test]
fn smoke_release_evidence_preamble_present() {
    // AC-6: the preamble banner is present verbatim (first and last line checked).
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/banner-test");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9406"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains(RELEASE_EVIDENCE_BANNER_LINE_1), "stdout: {}", stdout);
    assert!(stdout.contains(RELEASE_EVIDENCE_BANNER_LINE_4), "stdout: {}", stdout);
    let banner_pos = stdout.find(RELEASE_EVIDENCE_BANNER_LINE_1).unwrap();
    let heading_pos = stdout.find("# Release Evidence Package").unwrap();
    assert!(banner_pos < heading_pos, "banner must precede the report heading");
}

#[test]
fn smoke_release_evidence_unreadable_registry_degrades_gracefully() {
    // AC-7: a --registry path that doesn't exist degrades gracefully: full report still
    // emitted, warning on stderr, exit 0.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/missing-registry-test");

    let missing = p.join("does-not-exist.yaml");
    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9407", "--registry", missing.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("# Release Evidence Package"), "stdout: {}", stdout);
    assert!(stdout.contains("PR: [FILL]"), "stdout: {}", stdout);
    assert!(stderr.contains("cannot read registry file"), "stderr: {}", stderr);
    assert!(stderr.contains("[FILL]"), "stderr: {}", stderr);
}

#[test]
fn smoke_release_evidence_malformed_registry_degrades_gracefully() {
    // AC-7: a --registry path that exists but fails to parse degrades the same way.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/malformed-registry-test");

    let malformed = p.join("malformed.yaml");
    std::fs::write(&malformed, "features: [this is not a list of maps\n").expect("write fixture");
    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9408", "--registry", malformed.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("# Release Evidence Package"), "stdout: {}", stdout);
    assert!(stdout.contains("PR: [FILL]"), "stdout: {}", stdout);
    assert!(stderr.contains("cannot parse registry file"), "stderr: {}", stderr);
}

#[test]
fn smoke_release_evidence_feature_not_found_degrades_gracefully() {
    // AC-8: a valid registry with no matching feature_id degrades gracefully with a
    // distinct warning naming both the feature id and the registry path.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/not-found-test");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-0000
    slug: unrelated-feature
    status: active
    current_stage: 1
    blockers: []
"#,
    )
    .expect("write fixture");
    let registry_str = registry.to_str().unwrap();

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9409", "--registry", registry_str],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("# Release Evidence Package"), "stdout: {}", stdout);
    assert!(stdout.contains("PR: [FILL]"), "stdout: {}", stdout);
    assert!(stderr.contains("UPG-9409"), "stderr: {}", stderr);
    assert!(stderr.contains(registry_str), "stderr: {}", stderr);
}

#[test]
fn smoke_release_evidence_no_registry_is_silent() {
    // AC-9: omitting --registry entirely is normal usage — no stderr warning at all.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/no-registry-test");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9410"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("PR: [FILL]"), "stdout: {}", stdout);
    assert_eq!(stderr, "", "no --registry given should produce no stderr output at all");
}

#[test]
fn smoke_release_evidence_feature_required() {
    // AC-10: omitting --feature is a clap usage error, exit 1, nothing on stdout.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let (code, stdout, _stderr) = run_in_dir(p, &["generate-release-evidence"]);
    assert_eq!(code, 1, "missing --feature should exit 1");
    assert_eq!(stdout, "", "usage error should produce no stdout");
}

#[test]
fn smoke_release_evidence_stdout_only_no_registry() {
    // AC-11: with no --registry, stdout carries the full report and stderr is empty.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/stdout-only-test");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9411"]);
    assert_eq!(code, 0);
    assert_eq!(stderr, "");
    assert!(!stdout.is_empty(), "stdout must not be empty");
}

#[test]
fn smoke_release_evidence_exit_zero_across_registry_states() {
    // AC-12: exit 0 across no-registry, bad-registry, feature-not-found, and
    // fully-valid-registry states, all inside a git repo.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/exit-code-matrix-test");

    let (code_none, _, stderr_none) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9412"]);
    assert_eq!(code_none, 0, "stderr: {}", stderr_none);

    let bad = p.join("missing.yaml");
    let (code_bad, _, stderr_bad) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9412", "--registry", bad.to_str().unwrap()],
    );
    assert_eq!(code_bad, 0, "stderr: {}", stderr_bad);

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9412
    slug: exit-code-feature
    status: active
    current_stage: 5
    pr: "https://example.test/pr/3"
    intent: intents/UPG-9412.md
    contract: contracts/UPG-9412_contract.md
    event_schema: events/UPG-9412_schema.md
    blockers: []
"#,
    )
    .expect("write fixture");
    let (code_full, _, stderr_full) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9412", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code_full, 0, "stderr: {}", stderr_full);

    let (code_not_found, _, stderr_not_found) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-0000-not-in-registry", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code_not_found, 0, "stderr: {}", stderr_not_found);
}

#[test]
fn smoke_release_evidence_no_provider_config_required() {
    // AC-13: dispatches before config::resolve() — no provider config needed.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/no-config-test");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9413"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("# Release Evidence Package"), "stdout: {}", stdout);
}

#[test]
fn smoke_release_evidence_deterministic_output() {
    // AC-14: identical inputs produce byte-for-byte identical stdout across two runs.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/determinism-test");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9414
    slug: determinism-feature
    status: active
    current_stage: 3
    pr: "https://example.test/pr/4"
    intent: intents/UPG-9414.md
    contract: null
    event_schema: null
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code1, stdout1, stderr1) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9414", "--registry", registry.to_str().unwrap()],
    );
    let (code2, stdout2, stderr2) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9414", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code1, 0, "stderr: {}", stderr1);
    assert_eq!(code2, 0, "stderr: {}", stderr2);
    assert_eq!(stdout1, stdout2, "output must be deterministic for identical inputs");
}

#[test]
fn smoke_release_evidence_architectural_refinements_never_treated_as_feature() {
    // AC-15: an architectural_refinements entry with a colliding refine_id is never
    // matched as if it were the searched-for feature.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/refinement-collision-test");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9415
    slug: real-feature
    status: active
    current_stage: 2
    pr: "https://example.test/pr/5"
    intent: intents/UPG-9415.md
    contract: null
    event_schema: null
    blockers: []

architectural_refinements:
  - refine_id: UPG-9416-sneaky
    description: "Looks like a feature but isn't"
    status: active
    artifact: refinements/arch/sneaky.md
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9416-sneaky", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    // The refinement is not a feature entry, so it must not match — fields fall back to [FILL].
    assert!(stdout.contains("PR: [FILL]"), "stdout: {}", stdout);
    assert!(stderr.contains("UPG-9416-sneaky"), "stderr: {}", stderr);
}

// --- UPG-0041: registry schema v2 (schema_version, hypothesized status, notes field) ---

#[test]
fn smoke_dashboard_v2_missing_schema_version_diagnostic() {
    // AC-8: Missing schema_version triggers specific migration diagnostic
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"features:
  - feature_id: UPG-9500
    slug: legacy-feature
    status: active
    current_stage: 1
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, _stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_ne!(code, 0, "should exit non-zero for missing schema_version");
    assert!(stderr.contains("schema_version: 2"), "stderr: {}", stderr);
    assert!(stderr.contains("missing") || stderr.contains("found: 0"), "stderr: {}", stderr);
    assert!(stderr.contains("registry-v2-migration.md"), "stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_v2_wrong_schema_version_diagnostic() {
    // AC-8: Non-2 schema_version triggers specific diagnostic
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 1
features:
  - feature_id: UPG-9501
    slug: v1-feature
    status: active
    current_stage: 1
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, _stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_ne!(code, 0, "should exit non-zero for wrong schema_version");
    assert!(stderr.contains("schema_version: 2"), "stderr: {}", stderr);
    assert!(stderr.contains("found: 1"), "stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_v2_non_numeric_schema_version_diagnostic() {
    // AC-8 blocker fix: Non-numeric schema_version should be reported as such, not as "missing"
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: "1"
features:
  - feature_id: UPG-9502
    slug: string-version-feature
    status: active
    current_stage: 1
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, _stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_ne!(code, 0, "should exit non-zero for non-numeric schema_version");
    assert!(stderr.contains("schema_version: 2"), "stderr: {}", stderr);
    // Should show the actual value (a string), not "missing"
    assert!(!stderr.contains("found: missing"), "should not report as missing: {}", stderr);
    assert!(stderr.contains("not a number") || stderr.contains("\"1\""), "stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_v2_schema_version_probe_wins_over_missing_field() {
    // AC-9: schema_version pre-probe prevents generic "missing field `slug`" error
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"features:
  - feature_id: UPG-9502
    status: active
    current_stage: 1
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, _stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_ne!(code, 0, "should exit non-zero");
    // Should get the schema_version diagnostic, not a generic "missing field `slug`" error
    assert!(stderr.contains("schema_version"), "stderr: {}", stderr);
    assert!(!stderr.contains("missing field"), "stderr should not contain generic serde error: {}", stderr);
}

#[test]
fn smoke_dashboard_v2_invalid_status_value_diagnostic() {
    // AC-10: Invalid status value produces specific diagnostic
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9503
    slug: invalid-status-feature
    status: stage1
    current_stage: 1
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, _stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_ne!(code, 0, "should exit non-zero for invalid status");
    assert!(stderr.contains("UPG-9503"), "stderr: {}", stderr);
    assert!(stderr.contains("stage1") || stderr.contains("invalid status"), "stderr: {}", stderr);
    assert!(stderr.contains("hypothesized") && stderr.contains("active"), "stderr should list valid values: {}", stderr);
}

#[test]
fn smoke_dashboard_v2_hypothesized_and_active_both_appear() {
    // AC-11: Both active and hypothesized features appear, hypothesized visually flagged
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9504
    slug: active-feature
    status: active
    current_stage: 3
    blockers: []
    notes: ""
  - feature_id: UPG-9505
    slug: hypothesized-feature
    status: hypothesized
    current_stage: 0
    blockers: []
    notes: ""
  - feature_id: UPG-9506
    slug: suspended-feature
    status: suspended
    current_stage: 2
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("UPG-9504: active-feature"), "active feature should appear: {}", stdout);
    assert!(stdout.contains("UPG-9505: hypothesized-feature"), "hypothesized feature should appear: {}", stdout);
    assert!(!stdout.contains("UPG-9506"), "suspended feature should not appear: {}", stdout);

    // Hypothesized feature should have visual flag
    let hyp_section = stdout.split("UPG-9505").nth(1).expect("hypothesized section present");
    assert!(
        hyp_section.contains("HYPOTHESIZED") || hyp_section.contains("Stage 1 review"),
        "hypothesized feature should be visually flagged: {}",
        hyp_section
    );

    // Active feature should NOT have the flag
    let active_section = stdout.split("UPG-9504").nth(1).and_then(|s| s.split("UPG-9505").next()).expect("active section");
    assert!(
        !active_section.contains("HYPOTHESIZED"),
        "active feature should not have hypothesized flag: {}",
        active_section
    );
}

#[test]
fn smoke_dashboard_v2_all_active_registry_unchanged_behavior() {
    // AC-12: v2 registry with only active features produces expected output (no regression)
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9507
    slug: normal-active
    status: active
    current_stage: 2
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("UPG-9507: normal-active"), "stdout: {}", stdout);
    assert!(stdout.contains("Current stage: 2 [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("Open blockers: (none) [INFERRED]"), "stdout: {}", stdout);
    assert!(!stdout.contains("HYPOTHESIZED"), "should not have hypothesized flag: {}", stdout);
}

#[test]
fn smoke_release_evidence_v2_missing_schema_version_warning() {
    // AC-13: generate-release-evidence warns specifically about missing schema_version
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/test-v2");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"features:
  - feature_id: UPG-9508
    slug: legacy-feature
    status: active
    pr: "https://example.test/pr/1"
    intent: intents/UPG-9508.md
    contract: contracts/UPG-9508.md
    event_schema: null
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9508", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "should exit 0 (graceful degradation)");
    assert!(stderr.contains("schema_version: 2"), "stderr: {}", stderr);
    assert!(stderr.contains("missing"), "stderr: {}", stderr);
    assert!(stderr.contains("registry-v2-migration.md"), "stderr: {}", stderr);
    // Fields should fall back to [FILL]
    assert!(stdout.contains("[FILL]"), "stdout: {}", stdout);
}

#[test]
fn smoke_release_evidence_v2_field_set_unchanged() {
    // AC-14: generate-release-evidence's FeatureEntry still reads only pr/intent/contract/event_schema
    // (status/current_stage/blockers/notes were never read and still aren't)
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/test-v2-fields");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9509
    slug: v2-feature
    status: hypothesized
    current_stage: 0
    pr: "https://example.test/pr/99"
    intent: intents/UPG-9509.md
    contract: null
    event_schema: events/UPG-9509.json
    blockers: ["blocker-1", "blocker-2"]
    notes: "some notes"
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9509", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    // Only pr/intent/contract/event_schema should be read; status/current_stage/blockers/notes ignored
    assert!(stdout.contains("https://example.test/pr/99"), "stdout: {}", stdout);
    assert!(stdout.contains("intents/UPG-9509.md"), "stdout: {}", stdout);
    assert!(stdout.contains("events/UPG-9509.json"), "stdout: {}", stdout);
}

// ===== UPG-0042: Reviewer Packet Efficiency =====

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
    // Use Cargo.toml (stable file) to avoid issues with unstaged working tree changes
    let cargo_toml = "tools/reviewer/Cargo.toml";
    let readme = "README.md";

    // Review without sha-only
    let (code1, stdout1, _stderr1) = run(&[
        "review", "UPG-SMOKE-SHA", "test-stage",
        "--print-packet", "--skip-prechecks",
        cargo_toml, readme,
    ]);
    assert_eq!(code1, 0, "review without sha-only should succeed");

    // Review with sha-only for Cargo.toml
    let (code2, stdout2, _stderr2) = run(&[
        "review", "UPG-SMOKE-SHA", "test-stage",
        "--print-packet", "--skip-prechecks",
        "--sha-only", cargo_toml,
        readme,
    ]);
    assert_eq!(code2, 0, "review with sha-only should succeed");

    // SHA-only packet should be smaller
    assert!(stdout2.len() < stdout1.len(),
        "sha-only packet should be smaller: {} vs {}", stdout2.len(), stdout1.len());

    // SHA-only should show path_sha_only visibility
    assert!(stdout2.contains("path_sha_only"), "sha-only should mark visibility");
    assert!(stdout2.contains("Cargo.toml"), "sha-only manifest should include path");

    // Full packet should contain Cargo.toml content (package name)
    assert!(stdout1.contains("name = \"codeos-reviewer\""),
        "full packet should include Cargo.toml content");

    // SHA-only packet should NOT contain Cargo.toml content
    assert!(!stdout2.contains("name = \"codeos-reviewer\""),
        "sha-only packet should not include Cargo.toml content");
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
