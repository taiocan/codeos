use std::path::Path;

const PREAMBLE: &str = "\
> [INFERRED] fields were populated automatically from CLI arguments, git, test output, or\n\
> events — verify before submitting. [FILL] fields require human or model authorship.";

pub struct GenerateReportArgs<'a> {
    pub stage: &'a str,
    pub feature: Option<&'a str>,
    pub base: Option<&'a str>,
    pub test_output: Option<&'a str>,
    pub events: Option<&'a str>,
}

pub fn run(args: GenerateReportArgs, repo_root: &Path) -> i32 {
    let feature_val = match args.feature {
        Some(f) => format!("{} [INFERRED]", f),
        None => "[FILL]".to_string(),
    };

    let report = match args.stage {
        "4" => build_stage4(&feature_val, args.base, repo_root),
        "5" => build_stage5(&feature_val, args.test_output),
        "6" => build_stage6(&feature_val, args.events),
        other => {
            eprintln!("error: --stage must be 4, 5, or 6; got '{}'", other);
            return crate::EXIT_USAGE;
        }
    };

    println!("{}\n", PREAMBLE);
    print!("{}", report);
    crate::EXIT_SUCCESS
}

fn build_stage4(feature_val: &str, base: Option<&str>, repo_root: &Path) -> String {
    let files_changed = match base {
        None => "[FILL]".to_string(),
        Some(b) => git_diff_files(b, repo_root),
    };

    format!(
        "## Stage 4 Implementation Report\n\n\
Feature: {feature}\n\n\
Approved artifacts used: [FILL]\n\
- Intent: [FILL]\n\
- Contract: [FILL]\n\
- Event schema: [FILL]\n\n\
Files changed: {files_changed}\n\n\
Files inspected but not changed: [FILL]\n\n\
Contract clauses implemented: [FILL]\n\n\
Schema events emitted: [FILL]\n\n\
Correlation ID propagation: [FILL]\n\n\
Runtime artifacts touched: [FILL]\n\n\
Unimplemented clauses: [FILL]\n\n\
Assumptions: [FILL]\n\n\
Blocked items: [FILL]\n\n\
Requires earlier-stage change: [FILL]\n\n\
Unexpected complexity: [FILL]\n",
        feature = feature_val,
        files_changed = files_changed,
    )
}

fn git_diff_files(base: &str, repo_root: &Path) -> String {
    let spec = format!("{}..HEAD", base);
    match std::process::Command::new("git")
        .args(["diff", "--name-only", &spec])
        .current_dir(repo_root)
        .output()
    {
        Err(e) => {
            eprintln!("error: git diff failed: {}", e);
            "[FILL]".to_string()
        }
        Ok(out) if !out.status.success() => {
            let msg = String::from_utf8_lossy(&out.stderr);
            eprintln!("error: git diff failed: {}", msg.trim());
            "[FILL]".to_string()
        }
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
            if lines.is_empty() {
                "(none) [INFERRED]".to_string()
            } else {
                format!("[INFERRED]\n{}", lines.join("\n"))
            }
        }
    }
}

fn build_stage5(feature_val: &str, test_output: Option<&str>) -> String {
    let (tests_run, tests_passed, tests_failed, tests_skipped) = match test_output {
        None => fill4(),
        Some(path) => parse_test_counts(path),
    };

    format!(
        "## Stage 5 Test Report\n\n\
Feature: {feature}\n\n\
Approved artifacts used: [FILL]\n\n\
Behavioral tests added: [FILL]\n\n\
Failure-mode tests added: [FILL]\n\n\
Invariant tests added: [FILL]\n\n\
Telemetry/event tests added: [FILL]\n\n\
Replay tests added: [FILL]\n\n\
Tests run: {tests_run}\n\n\
Tests passed: {tests_passed}\n\n\
Tests failed: {tests_failed}\n\n\
Tests skipped: {tests_skipped}\n\n\
Tests not run: [FILL]\n\n\
Known test gaps: [FILL]\n\n\
Why gaps are acceptable or not acceptable: [FILL]\n",
        feature = feature_val,
        tests_run = tests_run,
        tests_passed = tests_passed,
        tests_failed = tests_failed,
        tests_skipped = tests_skipped,
    )
}

fn fill4() -> (String, String, String, String) {
    ("[FILL]".into(), "[FILL]".into(), "[FILL]".into(), "[FILL]".into())
}

fn parse_test_counts(path: &str) -> (String, String, String, String) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: cannot read test output file '{}': {}", path, e);
            return fill4();
        }
    };

    for line in content.lines() {
        if let Some(counts) = parse_cargo_summary_line(line) {
            return counts;
        }
    }

    fill4()
}

fn parse_cargo_summary_line(line: &str) -> Option<(String, String, String, String)> {
    // Matches: "test result: ok. N passed; M failed; P ignored[; ...]"
    //      or: "test result: FAILED. N passed; M failed; P ignored[; ...]"
    // Status is case-sensitive and must be exactly "ok" or "FAILED" (AC-9's stated regex);
    // any other status is treated as a non-matching line, not a parse of arbitrary text.
    let line = line.trim();
    if !line.starts_with("test result: ") {
        return None;
    }
    let after_prefix = &line["test result: ".len()..];
    let mut parts = after_prefix.splitn(2, ". ");
    let status = parts.next()?;
    if status != "ok" && status != "FAILED" {
        return None;
    }
    let after_status = parts.next()?;

    let passed = extract_count(after_status, "passed")?;
    let failed = extract_count(after_status, "failed")?;
    let ignored = extract_count(after_status, "ignored")?;

    let run = passed + failed;
    Some((
        format!("{} [INFERRED]", run),
        format!("{} [INFERRED]", passed),
        format!("{} [INFERRED]", failed),
        format!("{} [INFERRED]", ignored),
    ))
}

fn extract_count(s: &str, label: &str) -> Option<u64> {
    for part in s.split(';') {
        let part = part.trim();
        let mut words = part.splitn(2, ' ');
        let num = words.next()?.trim();
        let lbl = words.next()?.trim();
        if lbl == label {
            return num.parse().ok();
        }
    }
    None
}

fn build_stage6(feature_val: &str, events: Option<&str>) -> String {
    let events_captured = match events {
        None => "[FILL]".to_string(),
        Some(path) => count_event_lines(path),
    };

    format!(
        "## Stage 6 Runtime Evidence Report\n\n\
Feature: {feature}\n\n\
How the system was run: [FILL]\n\n\
Input fixture/scenario: [FILL]\n\n\
Runtime command: [FILL]\n\n\
Runtime log path: [FILL]\n\n\
Events captured: {events_captured}\n\n\
Unexpected events: [FILL]\n\n\
Missing expected events: [FILL]\n\n\
Correlation chains observed: [FILL]\n\n\
Sanitization status: [FILL]\n\n\
Raw logs committed: [FILL]\n\
- yes/no: [FILL]\n\
- if yes, why safe: [FILL]\n\n\
Derived replay fixtures produced: [FILL]\n\n\
Ready for reconciliation: [FILL]\n\n\
Known runtime gaps: [FILL]\n",
        feature = feature_val,
        events_captured = events_captured,
    )
}

fn count_event_lines(path: &str) -> String {
    match std::fs::read_to_string(path) {
        Ok(content) => format!("{} [INFERRED]", content.lines().count()),
        Err(e) => {
            eprintln!("error: cannot read events file '{}': {}", path, e);
            "[FILL]".to_string()
        }
    }
}
