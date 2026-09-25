//! decision command tests.
//!
//! Tests for the decision command (APPROVE_STAGE, REQUEST_CHANGES verdicts).

mod common;
use common::{binary, repo_root, run, setup_temp_git_repo};
use sha2::{Digest, Sha256};
use std::process::Command;

#[test]
fn smoke_decision_bad_verdict_exits_usage() {
    let (code, _, stderr) = run(&[
        "decision",
        "UPG-SMOKE",
        "selfdev-step-0",
        "WRONG_VERDICT",
        "reason",
    ]);
    assert_ne!(code, 0, "bad verdict should fail");
    assert!(
        stderr.contains("APPROVE_STAGE") || stderr.contains("REQUEST_CHANGES"),
        "should mention valid verdicts: {}",
        stderr
    );
}

fn write_fake_assessment(
    repo_path: &std::path::Path,
    feature: &str,
    stage: &str,
    coverage_state: &str,
    packet_sha_override: Option<&str>, // None = use real sha of packet content
    review_commit: &str,
) -> (String, String) {
    let codex_dir = repo_path.join(".codeos/05-review/reviews/codex");
    let packets_dir = codex_dir.join("packets");
    std::fs::create_dir_all(&packets_dir).expect("create codex/packets dir");

    let packet_content = "FAKE PACKET CONTENT FOR TEST\n";
    let actual_sha = codeos_reviewer_sha256(packet_content);
    let packet_name = format!(
        "20260101T000000Z-{}-stage-{}-abcdef1.packet.txt",
        feature, stage
    );
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
    format!(
        "{:016x}{:016x}{:016x}{:016x}",
        h.finish(),
        h.finish() ^ 0xdead,
        h.finish() ^ 0xbeef,
        h.finish() ^ 0xcafe
    )
}

/// Write a malformed assessment file (exists but has no parseable frontmatter).
fn write_malformed_assessment(repo_path: &std::path::Path, feature: &str, stage: &str) {
    let codex_dir = repo_path.join(".codeos/05-review/reviews/codex");
    std::fs::create_dir_all(&codex_dir).expect("create codex dir");
    let filename = format!("20260101T000000Z-{}-stage-{}-abcdef1.md", feature, stage);
    // No YAML frontmatter — just body text, so parse_assessment_frontmatter returns None.
    std::fs::write(
        codex_dir.join(&filename),
        "This assessment has no frontmatter.\n",
    )
    .expect("write malformed assessment");
}

/// Write an assessment that has frontmatter but is missing coverage_state (partial provenance).
fn write_partial_frontmatter_assessment(
    repo_path: &std::path::Path,
    feature: &str,
    stage: &str,
    review_commit: &str,
) {
    let codex_dir = repo_path.join(".codeos/05-review/reviews/codex");
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
    let reviews_dir = repo_path.join(".codeos/05-review/reviews");
    std::fs::create_dir_all(&reviews_dir).expect("create reviews dir");
    let log_path = reviews_dir.join("review-log.md");
    if !log_path.exists() {
        std::fs::write(&log_path, "# Codeos Review Log (append-only, v0)\n").expect("write log");
    }
    log_path
}

fn setup_dba(repo_path: &std::path::Path, configuration: &str) {
    std::fs::create_dir_all(repo_path.join(".codeos/00-project")).expect("create project config");
    std::os::unix::fs::symlink(repo_root(), repo_path.join(".codeos/toolkit"))
        .expect("create toolkit symlink");
    std::fs::write(
        repo_path.join(".codeos/00-project/codeos.yaml"),
        format!("codeos_schema: 2\ndba_configuration: {configuration}\n"),
    )
    .expect("write project config");
}

fn write_approved_package(repo_path: &std::path::Path, revision: Option<u64>) {
    let specs = repo_path.join(".codeos/01-specification");
    for (directory, file) in [
        ("intents", "FEAT.md"),
        ("contracts", "FEAT_contract.md"),
        ("event-schemas", "FEAT_schema.md"),
    ] {
        let dir = specs.join(directory);
        std::fs::create_dir_all(&dir).expect("create specification directory");
        let revision = revision
            .map(|value| format!("semantic_revision: {value}\n"))
            .unwrap_or_default();
        std::fs::write(
            dir.join(file),
            format!(
                "---\nstatus: APPROVED\napproved_by: Human\napproved_at: 2026-09-24\n{revision}---\n# Package\n"
            ),
        )
        .expect("write approved package artifact");
    }
}

fn r1_r3_history() -> &'static str {
    "# Codeos Review Log (append-only, v0)\n\
## 2026-01-01T00:00:00Z REVIEW — FEAT — Stage 8\nReview ID: REV__FEAT__8__R1\n\
## 2026-01-01T00:01:00Z REVIEW — FEAT — Stage 8\nReview ID: REV__FEAT__8__R2\n\
## 2026-01-01T00:02:00Z REVIEW — FEAT — Stage 8\nReview ID: REV__FEAT__8__R3\n"
}

fn authorize_reentry(repo: &std::path::Path) -> (i32, String, String) {
    authorize_reentry_after(repo, "REV__FEAT__8__R3")
}

fn authorize_reentry_after(repo: &std::path::Path, after_review: &str) -> (i32, String, String) {
    run_decision_in(
        repo,
        &[
            "decision",
            "FEAT",
            "8",
            "AUTHORIZE_REENTRY",
            "One reviewer execution for repaired findings.",
            "--after-review",
            after_review,
        ],
    )
}

fn run_decision_in(repo_path: &std::path::Path, args: &[&str]) -> (i32, String, String) {
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
fn authorize_reentry_is_bound_review_control_not_stage_approval() {
    let (dir, _) = setup_temp_git_repo();
    let repo = dir.path();
    setup_dba(repo, "DBA-11");
    let log_path = setup_review_log(repo);
    let original_history = r1_r3_history();
    std::fs::write(&log_path, original_history).expect("write R1-R3 history");

    let (code, _, stderr) = authorize_reentry(repo);
    assert_eq!(code, 0, "authorization failed: {stderr}");

    let log = std::fs::read_to_string(log_path).expect("read log");
    assert!(
        log.starts_with(original_history),
        "authorization must preserve R1-R3 byte-for-byte: {log}"
    );
    assert!(log.contains("Decision: AUTHORIZE_REENTRY"), "{log}");
    assert!(log.contains("Re-entry after: REV__FEAT__8__R3"), "{log}");
    assert!(!log.contains("Decision: APPROVE_STAGE"), "{log}");
    assert!(!log.contains("Effective concern: NO OBJECTION"), "{log}");
    assert!(
        !log.contains("REENTRY-1"),
        "authorization must not execute review: {log}"
    );
    assert!(
        !log.contains("Provenance:"),
        "review control must not claim assessment provenance: {log}"
    );
}

#[test]
fn contract_v9_accepts_valid_reentry_authorization() {
    let (dir, _) = setup_temp_git_repo();
    let repo = dir.path();
    setup_dba(repo, "DBA-14");
    let log_path = setup_review_log(repo);
    std::fs::write(&log_path, r1_r3_history()).expect("write R1-R3 history");

    let (code, _, stderr) = authorize_reentry(repo);
    assert_eq!(code, 0, "v9 authorization failed: {stderr}");
    let log = std::fs::read_to_string(log_path).expect("read log");
    assert!(log.contains("Decision: AUTHORIZE_REENTRY"), "{log}");
}

#[test]
fn contract_v10_accepts_valid_reentry_authorization() {
    let (dir, _) = setup_temp_git_repo();
    let repo = dir.path();
    setup_dba(repo, "DBA-15");
    let log_path = setup_review_log(repo);
    std::fs::write(&log_path, r1_r3_history()).expect("write R1-R3 history");

    let (code, _, stderr) = authorize_reentry(repo);
    assert_eq!(code, 0, "v10 authorization failed: {stderr}");
    let log = std::fs::read_to_string(log_path).expect("read log");
    assert!(log.contains("Decision: AUTHORIZE_REENTRY"), "{log}");
}

#[test]
fn contract_v10_accepts_semantic_revision_r3_authorization() {
    let (dir, _) = setup_temp_git_repo();
    let repo = dir.path();
    setup_dba(repo, "DBA-15");
    let log_path = setup_review_log(repo);
    let history = "# Codeos Review Log (append-only, v0)\n\
## 2026-01-01T00:00:00Z REVIEW — FEAT — Stage 8\nReview ID: REV__FEAT__8__SR2__R1\n\
## 2026-01-01T00:01:00Z REVIEW — FEAT — Stage 8\nReview ID: REV__FEAT__8__SR2__R2\n\
## 2026-01-01T00:02:00Z REVIEW — FEAT — Stage 8\nReview ID: REV__FEAT__8__SR2__R3\n";
    std::fs::write(&log_path, history).expect("write semantic R1-R3 history");

    let (code, _, stderr) = authorize_reentry_after(repo, "REV__FEAT__8__SR2__R3");
    assert_eq!(code, 0, "v10 semantic R3 authorization failed: {stderr}");
    let log = std::fs::read_to_string(log_path).expect("read log");
    assert!(log.starts_with(history), "{log}");
    assert!(
        log.contains("Re-entry after: REV__FEAT__8__SR2__R3"),
        "{log}"
    );
    assert!(!log.contains("REENTRY-1"), "{log}");
}

#[test]
fn contract_v11_binds_authorization_to_current_semantic_revision() {
    let (dir, _) = setup_temp_git_repo();
    let repo = dir.path();
    setup_dba(repo, "DBA-16");
    write_approved_package(repo, Some(2));
    let log_path = setup_review_log(repo);
    let history = "# Codeos Review Log (append-only, v0)\n\
## 2026-01-01T00:00:00Z REVIEW — FEAT — Stage 8\nReview ID: REV__FEAT__8__SR1__R3\n\
## 2026-01-01T00:01:00Z RE-ENTRY REVIEW — FEAT — Stage 8\nReview ID: REV__FEAT__8__SR1__REENTRY-1\n\
## 2026-01-01T00:02:00Z REVIEW — FEAT — Stage 8\nReview ID: REV__FEAT__8__SR2__R3\n";
    std::fs::write(&log_path, history).expect("write package-bound history");

    let (code, _, stderr) = authorize_reentry_after(repo, "REV__FEAT__8__SR2__R3");
    assert_eq!(code, 0, "v11 authorization failed: {stderr}");
    let log = std::fs::read_to_string(&log_path).expect("read log");
    assert!(log.starts_with(history), "history changed: {log}");
    assert!(log.contains("Re-entry after: REV__FEAT__8__SR2__R3"));

    let before = log.clone();
    let (code, _, stderr) = authorize_reentry_after(repo, "REV__FEAT__8__SR1__R3");
    assert_eq!(code, 1);
    assert!(stderr.contains("current approved package"), "{stderr}");
    assert_eq!(std::fs::read_to_string(log_path).unwrap(), before);
}

#[test]
fn contract_v11_resolves_legacy_stage8_cycle_as_revision_zero() {
    let (dir, _) = setup_temp_git_repo();
    let repo = dir.path();
    setup_dba(repo, "DBA-16");
    write_approved_package(repo, None);
    let log_path = setup_review_log(repo);
    std::fs::write(&log_path, r1_r3_history()).expect("write legacy history");

    let (code, _, stderr) = authorize_reentry(repo);
    assert_eq!(code, 0, "legacy v11 authorization failed: {stderr}");
    assert!(std::fs::read_to_string(log_path)
        .unwrap()
        .contains("Re-entry after: REV__FEAT__8__R3"));
}

#[test]
fn contract_v12_authorizes_only_immutable_source_terminal_conditions() {
    let (dir, head) = setup_temp_git_repo();
    let repo = dir.path();
    setup_dba(repo, "DBA-17");
    write_approved_package(repo, Some(2));
    let reviews = repo.join(".codeos/05-review/reviews");
    std::fs::create_dir_all(&reviews).unwrap();
    let assessment = reviews.join("terminal-assessment.md");
    let assessment_text = "---\nfindings:\n  - finding_id: TERM__F1\n    classification: IN-SCOPE BLOCKER\nassessment_status: COMPLETE\n---\n\nFinding: implementation defect\n";
    std::fs::write(&assessment, assessment_text).unwrap();
    let assessment_sha = hex::encode(Sha256::digest(assessment_text.as_bytes()));
    let packet = reviews.join("terminal.packet.txt");
    std::fs::write(&packet, "packet").unwrap();
    let packet_sha = hex::encode(Sha256::digest(b"packet"));
    let log = setup_review_log(repo);
    std::fs::write(
        &log,
        format!(
            "# log\n## t RE-ENTRY REVIEW — FEAT — Stage 8\nReview ID: REV__FEAT__8__SR2__REENTRY-1\nBase: {head}  Review: {head}  Branch: main\nDiff-hash: original-diff\nEffective concern: DO NOT ADVANCE\nParse status: OK   Assessment status: COMPLETE\nFull assessment: {} (sha256:{assessment_sha})\nReviewed packet: {} (sha256:{packet_sha})\n",
            assessment.display(), packet.display()
        ),
    )
    .unwrap();
    let scope = repo.join("scope.json");
    std::fs::write(
        &scope,
        serde_json::json!({
            "terminal_review_id":"REV__FEAT__8__SR2__REENTRY-1",
            "assessment_sha256":assessment_sha,
            "conditions":[{"id":"C1","description":"Repair implementation defect","source":{"kind":"parsed_finding","finding_id":"TERM__F1"}}]
        }).to_string(),
    ).unwrap();
    let scope_arg = scope.to_str().unwrap();
    let (code, _, stderr) = run_decision_in(
        repo,
        &[
            "decision",
            "FEAT",
            "8",
            "AUTHORIZE_TERMINAL_REVIEW_CLOSURE",
            "Close only the repaired terminal condition.",
            "--after-review",
            "REV__FEAT__8__SR2__REENTRY-1",
            "--closure-scope",
            scope_arg,
        ],
    );
    assert_eq!(code, 0, "authorization failed: {stderr}");
    let history = std::fs::read_to_string(log).unwrap();
    assert!(history.contains("Decision: AUTHORIZE_TERMINAL_REVIEW_CLOSURE"));
    assert!(history.contains("Terminal closure scope JSON:"));
    assert!(!history.contains("TERMINAL-CLOSURE-1"));
}

#[test]
fn contracts_v7_and_v8_retain_reentry_authorization() {
    for configuration in ["DBA-12", "DBA-13"] {
        let (dir, _) = setup_temp_git_repo();
        let repo = dir.path();
        setup_dba(repo, configuration);
        let log_path = setup_review_log(repo);
        std::fs::write(&log_path, r1_r3_history()).expect("write R1-R3 history");

        let (code, _, stderr) = authorize_reentry(repo);
        assert_eq!(
            code, 0,
            "{configuration} re-entry authorization failed: {stderr}"
        );
        let log = std::fs::read_to_string(log_path).expect("read log");
        assert!(log.contains("Decision: AUTHORIZE_REENTRY"), "{log}");
    }
}

#[test]
fn contract_without_reentry_authority_fails_closed() {
    let (dir, _) = setup_temp_git_repo();
    let repo = dir.path();
    setup_dba(repo, "DBA-10");
    let log_path = setup_review_log(repo);
    let original_history = r1_r3_history();
    std::fs::write(&log_path, original_history).expect("write R1-R3 history");

    let (code, _, stderr) = authorize_reentry(repo);
    assert_eq!(code, 1);
    assert!(stderr.contains("not supported"), "{stderr}");
    assert_eq!(
        std::fs::read_to_string(log_path).expect("read log"),
        original_history
    );
}

#[test]
fn reentry_authorization_requires_reason_and_r3_reference() {
    let (dir, _) = setup_temp_git_repo();
    let repo = dir.path();
    setup_dba(repo, "DBA-15");
    let log_path = setup_review_log(repo);
    let original_history = r1_r3_history();
    std::fs::write(&log_path, original_history).expect("write R1-R3 history");

    let (missing_reason, _, reason_error) = run_decision_in(
        repo,
        &[
            "decision",
            "FEAT",
            "8",
            "AUTHORIZE_REENTRY",
            "",
            "--after-review",
            "REV__FEAT__8__R3",
        ],
    );
    assert_eq!(missing_reason, 1);
    assert!(reason_error.contains("non-empty reason and scope"));

    let (missing_reference, _, reference_error) = run_decision_in(
        repo,
        &[
            "decision",
            "FEAT",
            "8",
            "AUTHORIZE_REENTRY",
            "Bounded scope.",
        ],
    );
    assert_eq!(missing_reference, 1);
    assert!(reference_error.contains("requires --after-review"));
    assert_eq!(
        std::fs::read_to_string(log_path).expect("read log"),
        original_history
    );
}

#[test]
fn consumed_reentry_cannot_be_authorized_again_under_v10() {
    let (dir, _) = setup_temp_git_repo();
    let repo = dir.path();
    setup_dba(repo, "DBA-15");
    let log_path = setup_review_log(repo);
    let consumed_history = format!(
        "{}\n## 2026-01-01T00:03:00Z HUMAN DECISION — FEAT — Stage 8\nDecision: AUTHORIZE_REENTRY\nRe-entry after: REV__FEAT__8__R3\nReason/next: First bounded review.\n\n## 2026-01-01T00:04:00Z RE-ENTRY REVIEW — FEAT — Stage 8\nReview ID: REV__FEAT__8__REENTRY-1\n",
        r1_r3_history()
    );
    std::fs::write(&log_path, &consumed_history).expect("write consumed history");

    let (code, _, stderr) = authorize_reentry(repo);
    assert_eq!(code, 1);
    assert!(stderr.contains("already been consumed"), "{stderr}");
    assert_eq!(
        std::fs::read_to_string(log_path).expect("read log"),
        consumed_history
    );
}

#[test]
fn stage_disposition_rejects_reentry_binding() {
    let (dir, _) = setup_temp_git_repo();
    setup_review_log(dir.path());
    let (code, _, stderr) = run_decision_in(
        dir.path(),
        &[
            "decision",
            "FEAT",
            "8",
            "APPROVE_STAGE",
            "reason",
            "--after-review",
            "REV__FEAT__8__R3",
        ],
    );
    assert_eq!(code, 1);
    assert!(
        stderr.contains("valid only with AUTHORIZE_REENTRY"),
        "{stderr}"
    );
}

#[test]
fn smoke_decision_coverage_gate_blocks_critical_omission() {
    // AC-1: APPROVE_STAGE with CRITICAL_OMISSION must exit 1 and NOT write to the log.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);
    let log_size_before = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    write_fake_assessment(p, "FEAT", "my-stage", "CRITICAL_OMISSION", None, &head_sha);

    let (code, _, stderr) = run_decision_in(
        p,
        &["decision", "FEAT", "my-stage", "APPROVE_STAGE", "reason"],
    );
    assert_eq!(code, 1, "coverage gate must exit 1; stderr: {}", stderr);
    assert!(
        stderr.contains("APPROVE_STAGE refused"),
        "must mention refusal: {}",
        stderr
    );
    assert!(
        stderr.contains("CRITICAL_OMISSION"),
        "must name coverage_state: {}",
        stderr
    );

    let log_size_after = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);
    assert_eq!(
        log_size_before, log_size_after,
        "log must not grow when gate blocks"
    );
}

#[test]
fn smoke_decision_coverage_gate_blocks_empty_packet() {
    // AC-1: APPROVE_STAGE with EMPTY_PACKET must also exit 1.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_review_log(p);

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    write_fake_assessment(p, "FEAT", "my-stage", "EMPTY_PACKET", None, &head_sha);

    let (code, _, stderr) = run_decision_in(
        p,
        &["decision", "FEAT", "my-stage", "APPROVE_STAGE", "reason"],
    );
    assert_eq!(code, 1, "EMPTY_PACKET must also exit 1; stderr: {}", stderr);
    assert!(
        stderr.contains("EMPTY_PACKET"),
        "must name coverage_state: {}",
        stderr
    );
}

#[test]
fn smoke_decision_override_lifts_gate_and_records_both() {
    // AC-2: --override lifts the gate; log must contain BOTH coverage_state AND rationale.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    write_fake_assessment(p, "FEAT", "my-stage", "CRITICAL_OMISSION", None, &head_sha);

    let (code, stdout, stderr) = run_decision_in(
        p,
        &[
            "decision",
            "FEAT",
            "my-stage",
            "APPROVE_STAGE",
            "reason",
            "--override",
            "I accept the risk: packet was empty due to scratch run",
        ],
    );
    assert_eq!(
        code, 0,
        "override must lift gate; stderr: {}; stdout: {}",
        stderr, stdout
    );
    assert!(
        stdout.contains("decision appended"),
        "must confirm log write: {}",
        stdout
    );

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(
        log_content.contains("CRITICAL_OMISSION"),
        "log must record coverage_state: {}",
        log_content
    );
    assert!(
        log_content.contains("COVERAGE_GATE_OVERRIDDEN"),
        "log must record gate overridden: {}",
        log_content
    );
    assert!(
        log_content.contains("I accept the risk"),
        "log must record override rationale: {}",
        log_content
    );
}

#[test]
fn smoke_decision_approve_not_gated_for_full_coverage() {
    // AC-9: APPROVE_STAGE with FULL_COVERAGE must exit 0.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_review_log(p);

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    write_fake_assessment(p, "FEAT", "my-stage", "FULL_COVERAGE", None, &head_sha);

    let (code, _, stderr) = run_decision_in(
        p,
        &["decision", "FEAT", "my-stage", "APPROVE_STAGE", "all good"],
    );
    assert_eq!(
        code, 0,
        "FULL_COVERAGE must not be gated; stderr: {}",
        stderr
    );
}

#[test]
fn smoke_decision_request_changes_not_gated_even_critical_omission() {
    // AC-7: REQUEST_CHANGES must never be refused regardless of coverage_state.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_review_log(p);

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    write_fake_assessment(p, "FEAT", "my-stage", "CRITICAL_OMISSION", None, &head_sha);

    let (code, _, stderr) = run_decision_in(
        p,
        &[
            "decision",
            "FEAT",
            "my-stage",
            "REQUEST_CHANGES",
            "needs rework",
        ],
    );
    assert_eq!(
        code, 0,
        "REQUEST_CHANGES must never be gated; stderr: {}",
        stderr
    );
}

#[test]
fn smoke_decision_no_assessment_fallback_exits_zero() {
    // AC-6: no assessment for feature+stage → existing behavior preserved, exit 0.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_review_log(p);
    // Do NOT write any assessment file.

    let (code, _, stderr) = run_decision_in(
        p,
        &[
            "decision",
            "FEAT-UNKNOWN",
            "no-stage",
            "APPROVE_STAGE",
            "proceeding without review",
        ],
    );
    assert_eq!(
        code, 0,
        "no assessment must fall through to exit 0; stderr: {}",
        stderr
    );
}

#[test]
fn smoke_decision_packet_hash_mismatch_warns_but_proceeds() {
    // AC-4: packet hash mismatch → advisory warning to stderr; MISMATCH in log Provenance block.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    // Write assessment with a deliberately wrong stored SHA.
    write_fake_assessment(
        p,
        "FEAT",
        "my-stage",
        "FULL_COVERAGE",
        Some("0000000000000000000000000000000000000000000000000000000000000000"),
        &head_sha,
    );

    let (code, _, stderr) = run_decision_in(
        p,
        &[
            "decision",
            "FEAT",
            "my-stage",
            "APPROVE_STAGE",
            "proceeding",
        ],
    );
    assert_eq!(
        code, 0,
        "packet mismatch must not block; stderr: {}",
        stderr
    );
    assert!(
        stderr.contains("mismatch") || stderr.contains("MISMATCH") || stderr.contains("warning"),
        "mismatch must produce a warning: {}",
        stderr
    );

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(
        log_content.contains("MISMATCH"),
        "log Provenance block must contain MISMATCH: {}",
        log_content
    );
}

#[test]
fn smoke_decision_provenance_block_written_to_log() {
    // AC-3: every decision log entry must contain a Provenance block when assessment exists.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    write_fake_assessment(p, "FEAT", "my-stage", "FULL_COVERAGE", None, &head_sha);

    let (code, _, stderr) = run_decision_in(
        p,
        &[
            "decision",
            "FEAT",
            "my-stage",
            "APPROVE_STAGE",
            "all verified",
        ],
    );
    assert_eq!(code, 0, "should succeed; stderr: {}", stderr);

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(
        log_content.contains("Provenance:"),
        "log must contain Provenance block: {}",
        log_content
    );
    assert!(
        log_content.contains("assessment:"),
        "provenance must show assessment path: {}",
        log_content
    );
    assert!(
        log_content.contains("coverage_state:"),
        "provenance must show coverage_state: {}",
        log_content
    );
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

    let (code, _, stderr) = run_decision_in(
        p,
        &[
            "decision",
            "FEAT",
            "my-stage",
            "APPROVE_STAGE",
            "proceeding with drift",
        ],
    );
    assert_eq!(code, 0, "commit drift must not block; stderr: {}", stderr);
    assert!(
        stderr.contains("warning") || stderr.contains("HEAD") || stderr.contains("drift"),
        "commit drift must produce a warning: {}",
        stderr
    );

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(
        log_content.contains("HEAD_DRIFT"),
        "log Provenance block must contain HEAD_DRIFT: {}",
        log_content
    );
}

#[test]
fn smoke_decision_malformed_assessment_blocks_without_override() {
    // Fail-closed: assessment file exists but is unparseable → exit 1, no log write.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);
    let log_size_before = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);

    write_malformed_assessment(p, "FEAT", "my-stage");

    let (code, _, stderr) = run_decision_in(
        p,
        &[
            "decision",
            "FEAT",
            "my-stage",
            "APPROVE_STAGE",
            "proceeding",
        ],
    );
    assert_eq!(
        code, 1,
        "malformed assessment must block without --override; stderr: {}",
        stderr
    );
    assert!(
        stderr.contains("provenance") || stderr.contains("assessment") || stderr.contains("parsed"),
        "must explain provenance issue: {}",
        stderr
    );

    let log_size_after = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);
    assert_eq!(
        log_size_before, log_size_after,
        "log must not grow when fail-closed"
    );
}

#[test]
fn smoke_decision_partial_frontmatter_also_blocks() {
    // Fail-closed: assessment with frontmatter but missing coverage_state → exit 1.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);
    let log_size_before = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();
    write_partial_frontmatter_assessment(p, "FEAT", "my-stage", &head_sha);

    let (code, _, stderr) = run_decision_in(
        p,
        &[
            "decision",
            "FEAT",
            "my-stage",
            "APPROVE_STAGE",
            "proceeding",
        ],
    );
    assert_eq!(
        code, 1,
        "partial frontmatter (missing coverage_state) must also block; stderr: {}",
        stderr
    );
    let log_size_after = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);
    assert_eq!(
        log_size_before, log_size_after,
        "log must not grow when fail-closed"
    );
}

#[test]
fn smoke_decision_malformed_assessment_override_proceeds_and_records() {
    // Fail-closed with escape hatch: --override allows proceeding; log records PROVENANCE_UNVERIFIABLE.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    write_malformed_assessment(p, "FEAT", "my-stage");

    let (code, stdout, stderr) = run_decision_in(
        p,
        &[
            "decision",
            "FEAT",
            "my-stage",
            "APPROVE_STAGE",
            "proceeding despite broken assessment",
            "--override",
            "assessment was regenerated and lost; evidence reviewed in session",
        ],
    );
    assert_eq!(
        code, 0,
        "--override must allow proceeding past broken provenance; stderr: {}; stdout: {}",
        stderr, stdout
    );

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(
        log_content.contains("PROVENANCE_UNVERIFIABLE"),
        "log must record PROVENANCE_UNVERIFIABLE: {}",
        log_content
    );
    assert!(
        log_content.contains("assessment was regenerated"),
        "log must record override rationale: {}",
        log_content
    );
}

#[test]
fn smoke_decision_override_with_request_changes_is_graceful() {
    // AC-10: --override alongside REQUEST_CHANGES must not error or change exit code.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    setup_review_log(p);

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();
    write_fake_assessment(p, "FEAT", "my-stage", "FULL_COVERAGE", None, &head_sha);

    let (code, _, stderr) = run_decision_in(
        p,
        &[
            "decision",
            "FEAT",
            "my-stage",
            "REQUEST_CHANGES",
            "needs rework",
            "--override",
            "unused rationale",
        ],
    );
    assert_eq!(
        code, 0,
        "--override with REQUEST_CHANGES must exit 0; stderr: {}",
        stderr
    );
}

#[test]
fn smoke_decision_packet_missing_records_provenance_unverifiable() {
    // AC-6c: when the packet file is missing, log must show PROVENANCE_UNVERIFIABLE.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    // Write assessment that references a packet file that does NOT exist.
    let codex_dir = p.join(".codeos/05-review/reviews/codex");
    std::fs::create_dir_all(&codex_dir).expect("create codex dir");
    let content = format!(
        "---\nreviewed:\n  feature: FEAT\n  stage: my-stage\n  review_commit: {}\n  coverage_state: FULL_COVERAGE\n  reviewed_packet: packets/missing-packet.txt\n  reviewed_packet_sha256: abc123\n---\n",
        head_sha
    );
    std::fs::write(
        codex_dir.join("20260101T000000Z-FEAT-stage-my-stage-abcdef1.md"),
        &content,
    )
    .expect("write assessment");
    // Do NOT write the packet file.

    let (code, _, stderr) = run_decision_in(
        p,
        &["decision", "FEAT", "my-stage", "APPROVE_STAGE", "all clear"],
    );
    assert_eq!(code, 0, "missing packet must not block; stderr: {}", stderr);
    assert!(
        stderr.contains("warning")
            || stderr.contains("unverifiable")
            || stderr.contains("not found"),
        "must warn about missing packet: {}",
        stderr
    );

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(
        log_content.contains("PROVENANCE_UNVERIFIABLE"),
        "log must show PROVENANCE_UNVERIFIABLE for missing packet: {}",
        log_content
    );
}

#[test]
fn smoke_decision_no_stored_sha_warns_and_records_provenance_unverifiable() {
    // AC-6c: assessment exists with no reviewed_packet_sha256 → warning to stderr + PROVENANCE_UNVERIFIABLE in log.
    let (dir, _) = setup_temp_git_repo();
    let p = dir.path();
    let log_path = setup_review_log(p);

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();

    let codex_dir = p.join(".codeos/05-review/reviews/codex");
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
    )
    .expect("write assessment");

    let (code, _, stderr) = run_decision_in(
        p,
        &[
            "decision",
            "FEAT",
            "my-stage",
            "APPROVE_STAGE",
            "looks good",
        ],
    );
    assert_eq!(code, 0, "no stored sha must not block; stderr: {}", stderr);
    assert!(
        stderr.contains("no stored sha") || stderr.contains("unverifiable"),
        "must warn about missing stored sha; stderr: {}",
        stderr
    );

    let log_content = std::fs::read_to_string(&log_path).expect("read log");
    assert!(
        log_content.contains("PROVENANCE_UNVERIFIABLE"),
        "log must record PROVENANCE_UNVERIFIABLE for no-stored-sha case: {}",
        log_content
    );
}
