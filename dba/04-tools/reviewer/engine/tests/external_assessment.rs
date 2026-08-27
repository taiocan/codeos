//! Behavioral tests for the external-assessment path: exporting the canonical packet and importing
//! an assessment produced outside Codeos. No real Codex process is invoked, and the import path
//! must not invoke one at all.

mod common;
use common::{
    repo_root, run_in_dir, run_in_dir_with_env, run_with_fake_codex, setup_fake_codex,
    setup_temp_git_repo,
};

fn setup_codeos_symlink(repo_path: &std::path::Path) {
    std::fs::create_dir_all(repo_path.join(".codeos")).unwrap();
    std::os::unix::fs::symlink(repo_root(), repo_path.join(".codeos/toolkit")).unwrap();
    std::fs::write(repo_path.join(".git/info/exclude"), "/.codeos/toolkit\n").unwrap();
}

/// Export a packet + sidecar outside the reviewed tree and return both paths.
fn export_packet(
    repo: &std::path::Path,
    export_dir: &std::path::Path,
    feature: &str,
    artifacts: &[&str],
) -> std::path::PathBuf {
    let packet = export_dir.join(format!("{feature}.packet.txt"));
    let mut args = vec![
        "plan",
        feature,
        "selfdev-step-1",
        "--emit-packet",
        packet.to_str().unwrap(),
    ];
    args.extend_from_slice(artifacts);
    let (code, _, stderr) = run_in_dir(repo, &args);
    assert_eq!(code, 0, "{stderr}");
    packet
}

const FIXTURE_REPLY: &str = "\
Finding: hard-coded path breaks downstream use / Severity: High / Classification: IN-SCOPE BLOCKER
Evidence: tracked.md:1 / Why: the path only exists in this repo / Required action: fix now
Scope reason: the change introduces it

PR decision: REQUEST CHANGES
Scope drift warning: no — nothing beyond the stated scope

LOG SUMMARY: CHANGES ADVISED — one in-scope blocker
EVIDENCE: B
HIGHEST-IMPACT UNCERTAINTY: whether downstream callers already depend on the path
";

/// The exported packet must be the packet `review` sends. `plan` and `review` share
/// `review::prepare`, so any divergence here means a second construction path appeared — the one
/// thing that would make an external assessment evidence about something other than the review.
///
/// The manifest's `generated:` timestamp is the single permitted difference: the two runs happen at
/// different wall-clock seconds. Every other byte must match, and this test fails if any additional
/// line diverges.
#[test]
fn emitted_packet_matches_the_packet_review_sends() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let fake = setup_fake_codex();
    // Exported outside the reviewed tree: an in-repo export would itself become evidence.
    let export_dir = tempfile::tempdir().unwrap();
    let emitted = export_dir.path().join("emitted-packet.txt");

    let (plan_code, _, plan_stderr) = run_in_dir(
        repo.path(),
        &[
            "plan",
            "UPG-EXPORT",
            "selfdev-step-1",
            "--emit-packet",
            emitted.to_str().unwrap(),
            "tracked.md",
        ],
    );
    assert_eq!(plan_code, 0, "{plan_stderr}");

    let (review_code, _, review_stderr) = run_with_fake_codex(
        repo.path(),
        &fake,
        &["review", "UPG-EXPORT", "selfdev-step-1", "tracked.md"],
        "success",
    );
    assert_eq!(review_code, 0, "{review_stderr}");

    let exported = std::fs::read_to_string(&emitted).unwrap();
    let sent = std::fs::read_to_string(&fake.packet_log).unwrap();
    let exported_lines: Vec<&str> = exported.lines().collect();
    let sent_lines: Vec<&str> = sent.lines().collect();
    assert_eq!(
        exported_lines.len(),
        sent_lines.len(),
        "exported packet has a different line count than the packet sent to Codex"
    );
    for (index, (left, right)) in exported_lines.iter().zip(sent_lines.iter()).enumerate() {
        if left != right {
            assert!(
                left.starts_with("  generated: ") && right.starts_with("  generated: "),
                "packet line {} differs outside the manifest timestamp:\n  exported: {left}\n  sent:     {right}",
                index + 1
            );
        }
    }
}

/// An empty packet fails before invocation, so there is nothing legitimate to export either.
#[test]
fn emit_packet_writes_nothing_for_an_empty_packet() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let export_dir = tempfile::tempdir().unwrap();
    let emitted = export_dir.path().join("should-not-exist.txt");

    let (code, _, _) = run_in_dir(
        repo.path(),
        &[
            "plan",
            "UPG-EMPTY",
            "selfdev-step-1",
            "--emit-packet",
            emitted.to_str().unwrap(),
            "missing.md",
        ],
    );
    assert_ne!(code, 0);
    assert!(!emitted.exists(), "an empty packet must not be exported");
}

/// The import path must not start a Codex process. `codex` is absent from PATH here, so a spawn
/// attempt fails the test rather than silently succeeding against a stand-in.
#[test]
fn external_assessment_records_provenance_without_invoking_codex() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let export_dir = tempfile::tempdir().unwrap();
    let packet = export_packet(repo.path(), export_dir.path(), "UPG-EXTERNAL", &["tracked.md"]);
    let reply = export_dir.path().join("assessment.txt");
    std::fs::write(&reply, FIXTURE_REPLY).unwrap();

    let (code, stdout, stderr) = run_in_dir(
        repo.path(),
        &[
            "review",
            "UPG-EXTERNAL",
            "selfdev-step-1",
            "--assessment",
            reply.to_str().unwrap(),
            "--packet",
            packet.to_str().unwrap(),
            "--reviewer-label",
            "deepseek-v4-flash",
            "tracked.md",
        ],
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("external assessment logged:"),
        "{stdout}"
    );
    assert!(
        stdout.contains("does NOT satisfy a required review round"),
        "{stdout}"
    );
    assert!(
        !repo.path().join(".codeos-state/codex-sessions").exists(),
        "the import path must not create Codex session state"
    );

    let reviews = repo.path().join(".codeos/05-review/reviews");
    let assessment_dir = reviews.join("codex");
    let file = std::fs::read_dir(&assessment_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "md"))
        .expect("assessment file");
    let recorded = std::fs::read_to_string(file).unwrap();
    assert!(recorded.contains("  source: external\n"), "{recorded}");
    assert!(
        recorded.contains("not invoked or verified by Codeos"),
        "{recorded}"
    );
    assert!(
        recorded.contains("  counts_as_review_round: false\n"),
        "{recorded}"
    );
    assert!(
        !recorded.contains("reasoning_effort:") && !recorded.contains("reconnect_count:"),
        "process measurements must not be invented for an assessment Codeos did not run:\n{recorded}"
    );
    // The unchanged parser handled the external text: the fixture's single finding is structured,
    // and nothing was dropped.
    assert!(recorded.contains("  reported_concern: CHANGES ADVISED\n"), "{recorded}");
    assert!(!recorded.contains("codex_concern:"), "{recorded}");
    assert!(recorded.contains("classification: IN-SCOPE BLOCKER"), "{recorded}");
    assert!(recorded.contains("unparsed_findings_count: 0"), "{recorded}");
    assert!(recorded.contains("EXT__UPG-EXTERNAL__selfdev-step-1__A1"), "{recorded}");

    let log = std::fs::read_to_string(reviews.join("review-log.md")).unwrap();
    assert!(log.contains("EXTERNAL ASSESSMENT — UPG-EXTERNAL — Stage selfdev-step-1"), "{log}");
    assert!(log.contains("Reported concern: CHANGES ADVISED"), "{log}");
    assert!(!log.contains("Codex concern:"), "{log}");
}

/// Recording an external assessment must not consume a review round the human still owes the
/// boundary: the next Codex-backed review is still R1.
#[test]
fn external_assessment_does_not_advance_the_review_round() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let fake = setup_fake_codex();
    let export_dir = tempfile::tempdir().unwrap();
    let packet = export_packet(repo.path(), export_dir.path(), "UPG-ROUND", &["tracked.md"]);
    let reply = export_dir.path().join("assessment.txt");
    std::fs::write(&reply, FIXTURE_REPLY).unwrap();

    for _ in 0..2 {
        let (code, _, stderr) = run_in_dir(
            repo.path(),
            &[
                "review",
                "UPG-ROUND",
                "selfdev-step-1",
                "--assessment",
                reply.to_str().unwrap(),
                "--packet",
                packet.to_str().unwrap(),
                "tracked.md",
            ],
        );
        assert_eq!(code, 0, "{stderr}");
    }

    let (code, stdout, stderr) = run_with_fake_codex(
        repo.path(),
        &fake,
        &["review", "UPG-ROUND", "selfdev-step-1", "tracked.md"],
        "success",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("REV__UPG-ROUND__selfdev-step-1__R1"),
        "two external assessments must leave the first review round unclaimed:\n{stdout}"
    );

    let log =
        std::fs::read_to_string(repo.path().join(".codeos/05-review/reviews/review-log.md")).unwrap();
    assert!(log.contains("EXT__UPG-ROUND__selfdev-step-1__A1"), "{log}");
    assert!(log.contains("EXT__UPG-ROUND__selfdev-step-1__A2"), "{log}");
}

/// Exporting into the reviewed tree is allowed but must say what it costs.
#[test]
fn in_repo_export_warns_that_it_pollutes_the_next_packet() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let (code, _, stderr) = run_in_dir(
        repo.path(),
        &[
            "plan",
            "UPG-WARN",
            "selfdev-step-1",
            "--emit-packet",
            "packet.txt",
            "tracked.md",
        ],
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stderr.contains("adds an untracked file to the reviewed tree"),
        "{stderr}"
    );
}

/// An unreadable or empty assessment file fails before any durable record is written.
#[test]
fn missing_or_empty_assessment_writes_no_records() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let export_dir = tempfile::tempdir().unwrap();
    let packet = export_packet(repo.path(), export_dir.path(), "UPG-BAD", &["tracked.md"]);
    let empty = export_dir.path().join("empty.txt");
    std::fs::write(&empty, "   \n").unwrap();

    for path in ["does-not-exist.txt", empty.to_str().unwrap()] {
        let (code, _, stderr) = run_in_dir(
            repo.path(),
            &[
                "review",
                "UPG-BAD",
                "selfdev-step-1",
                "--assessment",
                path,
                "--packet",
                packet.to_str().unwrap(),
                "tracked.md",
            ],
        );
        assert_eq!(code, 3, "{stderr}");
    }
    assert!(
        !repo.path().join(".codeos/05-review/reviews").exists(),
        "no reviewer records may be written when the assessment cannot be read"
    );
}

/// `--reviewer-label` is metadata attached to an imported assessment, never a provider selector.
#[test]
fn reviewer_label_requires_an_assessment() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let (code, _, stderr) = run_in_dir(
        repo.path(),
        &[
            "review",
            "UPG-LABEL",
            "selfdev-step-1",
            "--reviewer-label",
            "deepseek-v4-flash",
            "tracked.md",
        ],
    );
    assert_eq!(code, 1, "{stderr}");
}

// --- the four integrity properties the pilot exposed -------------------------------------------

/// Property 1: the recorded packet is the exported packet, byte for byte. The import path adopts
/// the exported bytes instead of rebuilding, so a tree that moved on between export and import
/// cannot silently swap the evidence the assessment is bound to.
#[test]
fn recorded_packet_is_the_exported_packet_even_after_the_tree_moves_on() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let export_dir = tempfile::tempdir().unwrap();
    let packet = export_packet(repo.path(), export_dir.path(), "UPG-BIND", &["tracked.md"]);
    let exported = std::fs::read_to_string(&packet).unwrap();

    // The reviewed artifact changes after the model read it. A rebuild here would record different
    // evidence than the one assessed.
    std::fs::write(repo.path().join("tracked.md"), "# tracked\nchanged after export\n").unwrap();

    let reply = export_dir.path().join("assessment.txt");
    std::fs::write(&reply, FIXTURE_REPLY).unwrap();
    let (code, _, stderr) = run_in_dir(
        repo.path(),
        &[
            "review",
            "UPG-BIND",
            "selfdev-step-1",
            "--assessment",
            reply.to_str().unwrap(),
            "--packet",
            packet.to_str().unwrap(),
            "tracked.md",
        ],
    );
    assert_eq!(code, 0, "{stderr}");

    let packets = repo.path().join(".codeos/05-review/reviews/codex/packets");
    let saved = std::fs::read_dir(packets)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(
        std::fs::read_to_string(saved).unwrap(),
        exported,
        "the recorded packet must be the bytes the model read, not a rebuild"
    );
}

/// A sidecar that describes different evidence than the operator named is refused outright.
#[test]
fn packet_bound_to_other_evidence_is_refused() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let export_dir = tempfile::tempdir().unwrap();
    let packet = export_packet(repo.path(), export_dir.path(), "UPG-OTHER", &["tracked.md"]);
    let reply = export_dir.path().join("assessment.txt");
    std::fs::write(&reply, FIXTURE_REPLY).unwrap();

    let (code, _, stderr) = run_in_dir(
        repo.path(),
        &[
            "review",
            "UPG-MISMATCH",
            "selfdev-step-1",
            "--assessment",
            reply.to_str().unwrap(),
            "--packet",
            packet.to_str().unwrap(),
            "tracked.md",
        ],
    );
    assert_eq!(code, 4, "{stderr}");
    assert!(stderr.contains("is for feature 'UPG-OTHER'"), "{stderr}");
    assert!(!repo.path().join(".codeos/05-review/reviews").exists());
}

/// A missing sidecar fails closed rather than falling back to a rebuild.
#[test]
fn missing_sidecar_fails_closed() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let export_dir = tempfile::tempdir().unwrap();
    let packet = export_packet(repo.path(), export_dir.path(), "UPG-NOMETA", &["tracked.md"]);
    std::fs::remove_file(format!("{}.meta.json", packet.display())).unwrap();
    let reply = export_dir.path().join("assessment.txt");
    std::fs::write(&reply, FIXTURE_REPLY).unwrap();

    let (code, _, stderr) = run_in_dir(
        repo.path(),
        &[
            "review",
            "UPG-NOMETA",
            "selfdev-step-1",
            "--assessment",
            reply.to_str().unwrap(),
            "--packet",
            packet.to_str().unwrap(),
            "tracked.md",
        ],
    );
    assert_eq!(code, 4, "{stderr}");
    assert!(stderr.contains("packet sidecar"), "{stderr}");
}

/// Property 2: a new file that no diff can show is either in the packet or downgrades coverage.
/// Reviewing an implementation that adds a module must not report FULL_COVERAGE while the module
/// is invisible.
#[test]
fn untracked_files_are_shown_and_do_not_leave_coverage_overstated() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    std::fs::write(
        repo.path().join("new_module.rs"),
        "pub fn added_but_never_committed() {}\n",
    )
    .unwrap();

    let export_dir = tempfile::tempdir().unwrap();
    let packet = export_packet(repo.path(), export_dir.path(), "UPG-UNTRACKED", &["tracked.md"]);
    let text = std::fs::read_to_string(&packet).unwrap();
    assert!(text.contains("UNTRACKED FILES"), "{text}");
    assert!(text.contains("new_module.rs (untracked"), "{text}");
    assert!(
        text.contains("pub fn added_but_never_committed"),
        "the untracked file's content must actually reach the reviewer"
    );

    // An untracked file that cannot be shown downgrades coverage instead of being dropped silently.
    std::fs::write(repo.path().join("secrets.pem"), "-----BEGIN PRIVATE KEY-----\n").unwrap();
    let (code, stdout, stderr) = run_in_dir(
        repo.path(),
        &["plan", "UPG-UNTRACKED2", "selfdev-step-1", "tracked.md"],
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("coverage: PARTIAL_COVERAGE"),
        "an excluded untracked file must downgrade coverage:\n{stdout}"
    );
}

/// Property 3: findings the reviewer declared in an unexpected shape are not silently lost. This is
/// the exact failure the DeepSeek pilot produced — bulleted findings, `findings: []`, and a record
/// that claimed nothing was dropped.
#[test]
fn findings_in_an_unrecognised_shape_fail_closed() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let export_dir = tempfile::tempdir().unwrap();
    let packet = export_packet(repo.path(), export_dir.path(), "UPG-SHAPE", &["tracked.md"]);
    let reply = export_dir.path().join("assessment.txt");
    std::fs::write(
        &reply,
        "\
### Finding 1
- **Finding:** something is wrong
- **Severity:** High
- **Classification:** IN-SCOPE BLOCKER
- **Required action:** fix now

LOG SUMMARY: NO OBJECTION — looks fine to me
EVIDENCE: A
HIGHEST-IMPACT UNCERTAINTY: none
",
    )
    .unwrap();

    let (code, stdout, stderr) = run_in_dir(
        repo.path(),
        &[
            "review",
            "UPG-SHAPE",
            "selfdev-step-1",
            "--assessment",
            reply.to_str().unwrap(),
            "--packet",
            packet.to_str().unwrap(),
            "tracked.md",
        ],
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("INCOMPLETE"), "{stdout}");

    let file = std::fs::read_dir(repo.path().join(".codeos/05-review/reviews/codex"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "md"))
        .expect("assessment file");
    let recorded = std::fs::read_to_string(file).unwrap();
    assert!(recorded.contains("parse_status: FAILED"), "{recorded}");
    assert!(recorded.contains("assessment_status: INCOMPLETE"), "{recorded}");
    assert!(recorded.contains("unparsed_findings_count: 1"), "{recorded}");
    assert!(
        recorded.contains("effective_concern: DO NOT ADVANCE"),
        "a reviewer's NO OBJECTION must not survive findings that could not be recorded:\n{recorded}"
    );
}

/// Property 4: a reply that never reached a verdict cannot produce a clean assessment.
#[test]
fn truncated_reply_cannot_produce_a_clean_assessment() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let export_dir = tempfile::tempdir().unwrap();
    let packet = export_packet(repo.path(), export_dir.path(), "UPG-TRUNC", &["tracked.md"]);
    let reply = export_dir.path().join("assessment.txt");
    std::fs::write(
        &reply,
        "The artifact looks broadly consistent with its stated scope and I was still checking",
    )
    .unwrap();

    let (code, stdout, stderr) = run_in_dir(
        repo.path(),
        &[
            "review",
            "UPG-TRUNC",
            "selfdev-step-1",
            "--assessment",
            reply.to_str().unwrap(),
            "--packet",
            packet.to_str().unwrap(),
            "tracked.md",
        ],
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("INCOMPLETE"), "{stdout}");
    let file = std::fs::read_dir(repo.path().join(".codeos/05-review/reviews/codex"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "md"))
        .expect("assessment file");
    let recorded = std::fs::read_to_string(file).unwrap();
    assert!(recorded.contains("assessment_status: INCOMPLETE"), "{recorded}");
    assert!(recorded.contains("no LOG SUMMARY verdict line"), "{recorded}");
    assert!(recorded.contains("effective_concern: DO NOT ADVANCE"), "{recorded}");
}

// ── UPG-0070: packet-integrity gaps ─────────────────────────────────────────────────────────────

/// Gap 1. The sidecar's coverage claims are only about the packet they were written for. Before the
/// content binding, altered packet bytes could be imported under an untouched sidecar and recorded
/// as reviewed evidence carrying that sidecar's coverage_state.
#[test]
fn altered_packet_bytes_are_rejected_against_their_sidecar() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());
    let export_dir = tempfile::tempdir().unwrap();
    let emitted = export_dir.path().join("emitted-packet.txt");

    let (plan_code, _, plan_stderr) = run_in_dir(
        repo.path(),
        &[
            "plan",
            "UPG-0070",
            "selfdev-step-1",
            "--emit-packet",
            emitted.to_str().unwrap(),
            "tracked.md",
        ],
    );
    assert_eq!(plan_code, 0, "{plan_stderr}");

    // The sidecar is left exactly as written; only the reviewed bytes change.
    let original = std::fs::read_to_string(&emitted).unwrap();
    std::fs::write(&emitted, format!("{original}\nINJECTED CONTENT THAT WAS NEVER REVIEWED\n"))
        .unwrap();

    let assessment = export_dir.path().join("assessment.md");
    std::fs::write(
        &assessment,
        "LOG SUMMARY: NO OBJECTION — fine\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n",
    )
    .unwrap();

    let (code, _, stderr) = run_in_dir(
        repo.path(),
        &[
            "review",
            "UPG-0070",
            "selfdev-step-1",
            "--assessment",
            assessment.to_str().unwrap(),
            "--packet",
            emitted.to_str().unwrap(),
            "--reviewer-label",
            "test",
            "tracked.md",
        ],
    );
    assert_ne!(code, 0, "altered packet bytes must not import: {stderr}");
    assert!(
        stderr.contains("does not match its sidecar"),
        "the failure must name the binding that was violated, got: {stderr}"
    );
}

/// Gap 2. `git ls-files --others` failing is not the same answer as "no untracked files". When git
/// cannot be consulted the packet must not claim full coverage, because the files it would have
/// listed are exactly the ones no diff can show.
#[test]
fn untracked_discovery_failure_downgrades_coverage_instead_of_reading_as_none() {
    let (repo, _) = setup_temp_git_repo();
    setup_codeos_symlink(repo.path());

    // A `git` shim that fails ONLY `ls-files --others` and passes everything else through to the
    // real binary. A blanket-failing git would break branch and diff resolution too, and the packet
    // would then fail for an unrelated reason instead of exercising this defect.
    let real_git = String::from_utf8(
        std::process::Command::new("sh")
            .args(["-c", "command -v git"])
            .output()
            .expect("locate git")
            .stdout,
    )
    .expect("git path is utf8")
    .trim()
    .to_string();
    assert!(!real_git.is_empty(), "a real git is required for this test");

    let bin = tempfile::tempdir().unwrap();
    let shim = bin.path().join("git");
    std::fs::write(
        &shim,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"ls-files\" ] && [ \"$2\" = \"--others\" ]; then exit 1; fi\nexec {real_git} \"$@\"\n"
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    // An untracked file that discovery would have found, so the packet is genuinely incomplete.
    std::fs::write(repo.path().join("undiscovered.rs"), "fn hidden() {}\n").unwrap();

    let export_dir = tempfile::tempdir().unwrap();
    let emitted = export_dir.path().join("emitted-packet.txt");
    let path_with_shim = format!(
        "{}:{}",
        bin.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let (code, _, stderr) = run_in_dir_with_env(
        repo.path(),
        &[
            "plan",
            "UPG-0070",
            "selfdev-step-1",
            "--emit-packet",
            emitted.to_str().unwrap(),
            "tracked.md",
        ],
        &[("PATH", path_with_shim.as_str())],
    );
    assert_eq!(code, 0, "the packet should still build: {stderr}");

    let sidecar = std::fs::read_to_string(format!("{}.meta.json", emitted.display())).unwrap();
    assert!(
        !sidecar.contains("FULL_COVERAGE"),
        "coverage must be downgraded when untracked discovery is unavailable: {sidecar}"
    );
    assert!(
        sidecar.contains("untracked-file discovery"),
        "the reason must be visible in the packet's own exclusions: {sidecar}"
    );
}
