//! generate-adr-candidates command tests.
//!
//! Tests for the generate-adr-candidates subcommand (ADR candidate extraction from source docs).

mod common;
use common::{setup_temp_git_repo, run_in_dir, binary};
use std::process::Command;

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

