//! Per-test outcomes a verification observed, stored content-addressed so a Stage 8 packet can show
//! what was checked rather than only `passed` (Reviewer Tool Contract v17: required verification
//! records with their underlying content). Only case identities and outcomes are kept, never logs.

use crate::project::Project;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The verifications whose records must bind test evidence.
pub const VERIFICATIONS: [&str; 3] = ["behavior", "repeatability", "playwright"];

/// Store directory relative to the project root.
pub const DIR: &str = ".codeos/06-workflow/test-evidence";

/// The `details` key a verification record binds the evidence identity under.
pub const DETAILS_KEY: &str = "test_evidence_sha256";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// `passed`, `failed`, `skipped`, or `flaky`.
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestRun {
    /// `libtest`, `vitest`, or `playwright`.
    pub runner: String,
    /// 1, or 2 for the repeated run of `repeatability`.
    pub attempt: u8,
    pub cases: Vec<TestCase>,
    /// Wall-clock duration of the runner command that produced these cases, in milliseconds
    /// (for libtest this includes compilation). Absent in evidence recorded before it was
    /// measured: unknown, not zero. Diagnostic only; no decision reads it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestEvidence {
    pub schema: u8,
    pub verification: String,
    pub runs: Vec<TestRun>,
}

impl TestRun {
    pub fn new(runner: &str, attempt: u8, mut cases: Vec<TestCase>) -> Result<Self> {
        anyhow::ensure!(
            !cases.is_empty(),
            "{runner} reported no inspectable test cases"
        );
        cases.sort();
        Ok(Self {
            runner: runner.into(),
            attempt,
            cases,
            duration_ms: None,
        })
    }

    /// Record how long the runner command took.
    pub fn timed(mut self, elapsed: std::time::Duration) -> Self {
        self.duration_ms = Some(elapsed.as_millis() as u64);
        self
    }
}

/// Write the evidence and return its identity, the SHA-256 of the stored bytes.
pub fn store(project: &Project, evidence: &TestEvidence) -> Result<String> {
    let bytes = serde_json::to_vec_pretty(evidence)?;
    let identity = crate::hashing::bytes_sha256(&bytes);
    let dir = project.root.join(DIR);
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    std::fs::write(dir.join(format!("{identity}.json")), bytes)?;
    Ok(identity)
}

/// The cases differing between two runs of the same suite, as `id: first -> second`.
pub fn differences(first: &[TestCase], second: &[TestCase]) -> Vec<String> {
    let status = |cases: &[TestCase]| {
        cases
            .iter()
            .map(|case| (case.id.clone(), case.status.clone()))
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    let (first, second) = (status(first), status(second));
    let mut ids: Vec<_> = first.keys().chain(second.keys()).cloned().collect();
    ids.sort();
    ids.dedup();
    ids.into_iter()
        .filter(|id| first.get(id) != second.get(id))
        .map(|id| {
            let show = |value: Option<&String>| value.map_or("absent", String::as_str).to_string();
            format!(
                "{id}: {} -> {}",
                show(first.get(&id)),
                show(second.get(&id))
            )
        })
        .collect()
}

/// Rust libtest's per-test lines: `test <id> ... ok|FAILED|ignored`.
pub fn libtest_cases(stdout: &str) -> Vec<TestCase> {
    stdout
        .lines()
        .filter_map(|line| {
            let (id, result) = line.strip_prefix("test ")?.rsplit_once(" ... ")?;
            let status = match result.split([' ', ',']).next()? {
                "ok" => "passed",
                "FAILED" => "failed",
                "ignored" => "skipped",
                _ => return None,
            };
            Some(TestCase {
                id: id.trim().into(),
                file: None,
                status: status.into(),
            })
        })
        .collect()
}

/// Vitest's JSON reporter (Jest-compatible `testResults[].assertionResults[]`).
pub fn vitest_cases(report: &Value, root: &std::path::Path) -> Vec<TestCase> {
    let mut cases = Vec::new();
    for file in report["testResults"].as_array().into_iter().flatten() {
        let path = relative(file["name"].as_str().unwrap_or_default(), root);
        for assertion in file["assertionResults"].as_array().into_iter().flatten() {
            let status = match assertion["status"].as_str().unwrap_or_default() {
                "passed" => "passed",
                "failed" => "failed",
                _ => "skipped",
            };
            cases.push(TestCase {
                id: assertion["fullName"].as_str().unwrap_or_default().into(),
                file: Some(path.clone()),
                status: status.into(),
            });
        }
    }
    cases
}

/// Playwright's JSON reporter: nested `suites[].specs[].tests[]`.
pub fn playwright_cases(report: &Value) -> Vec<TestCase> {
    fn visit(suite: &Value, titles: &[String], cases: &mut Vec<TestCase>) {
        let mut titles = titles.to_vec();
        if let Some(title) = suite["title"].as_str().filter(|title| !title.is_empty()) {
            if suite.get("file").and_then(Value::as_str) != Some(title) {
                titles.push(title.into());
            }
        }
        for spec in suite["specs"].as_array().into_iter().flatten() {
            let mut path = titles.clone();
            path.push(spec["title"].as_str().unwrap_or_default().into());
            for test in spec["tests"].as_array().into_iter().flatten() {
                let status = match test["status"].as_str().unwrap_or_default() {
                    "expected" => "passed",
                    "skipped" => "skipped",
                    "flaky" => "flaky",
                    _ => "failed",
                };
                let project = test["projectName"].as_str().unwrap_or_default();
                let id = if project.is_empty() {
                    path.join(" › ")
                } else {
                    format!("[{project}] {}", path.join(" › "))
                };
                cases.push(TestCase {
                    id,
                    file: spec["file"].as_str().map(str::to_string),
                    status: status.into(),
                });
            }
        }
        for child in suite["suites"].as_array().into_iter().flatten() {
            visit(child, &titles, cases);
        }
    }
    let mut cases = Vec::new();
    for suite in report["suites"].as_array().into_iter().flatten() {
        visit(suite, &[], &mut cases);
    }
    cases
}

/// The Rust test function names libtest runs recorded: each case's last path segment. Doctests
/// (`src/lib.rs - item (line N)`) name no function and are skipped.
pub fn rust_test_names(evidence: &TestEvidence) -> std::collections::BTreeSet<String> {
    evidence
        .runs
        .iter()
        .filter(|run| run.runner == "libtest")
        .flat_map(|run| &run.cases)
        .filter(|case| !case.id.contains(' '))
        .filter_map(|case| case.id.rsplit("::").next().map(str::to_string))
        .collect()
}

/// Whether Rust source defines a function with one of `names`.
pub fn defines_any(source: &str, names: &std::collections::BTreeSet<String>) -> bool {
    let ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    source.match_indices("fn ").any(|(at, _)| {
        let preceded = source[..at].chars().next_back().is_none_or(|c| !ident(c));
        let name: String = source[at + 3..]
            .trim_start()
            .chars()
            .take_while(|c| ident(*c))
            .collect();
        preceded && names.contains(&name)
    })
}

fn relative(path: &str, root: &std::path::Path) -> String {
    std::path::Path::new(path)
        .strip_prefix(root)
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn libtest_lines_become_cases() {
        let out = "running 3 tests\ntest a::ok_case ... ok\ntest a::bad ... FAILED\ntest a::later ... ignored, slow\n\ntest result: FAILED. 1 passed\n";
        let cases = libtest_cases(out);
        assert_eq!(
            cases
                .iter()
                .map(|c| (c.id.as_str(), c.status.as_str()))
                .collect::<Vec<_>>(),
            [
                ("a::ok_case", "passed"),
                ("a::bad", "failed"),
                ("a::later", "skipped")
            ]
        );
    }

    #[test]
    fn vitest_and_playwright_reports_become_cases() {
        let root = std::path::Path::new("/p/web");
        let vitest = json!({"testResults": [{"name": "/p/web/src/a.spec.ts", "assertionResults": [
            {"fullName": "summary shows minima", "status": "passed"},
            {"fullName": "summary todo", "status": "todo"}]}]});
        let cases = vitest_cases(&vitest, root);
        assert_eq!(cases[0].file.as_deref(), Some("src/a.spec.ts"));
        assert_eq!(cases[1].status, "skipped");

        let playwright = json!({"suites": [{"title": "page.e2e.ts", "file": "page.e2e.ts", "specs": [],
            "suites": [{"title": "journey", "file": "page.e2e.ts", "specs": [{"title": "reaches decision", "file": "page.e2e.ts",
                "tests": [{"projectName": "chromium", "status": "expected"}]}]}]}]});
        let cases = playwright_cases(&playwright);
        assert_eq!(cases[0].id, "[chromium] journey › reaches decision");
        assert_eq!(cases[0].status, "passed");
    }

    #[test]
    fn rust_test_definitions_are_found_by_recorded_name() {
        let evidence = TestEvidence {
            schema: 1,
            verification: "behavior".into(),
            runs: vec![TestRun::new(
                "libtest",
                1,
                libtest_cases("test tests::supports_day_sixty ... ok\ntest src/lib.rs - item (line 3) ... ok\n"),
            )
            .unwrap()],
        };
        let names = rust_test_names(&evidence);
        assert_eq!(
            names.into_iter().collect::<Vec<_>>(),
            ["supports_day_sixty"]
        );
        let names = rust_test_names(&evidence);
        assert!(defines_any(
            "#[test]\n    fn supports_day_sixty() {}",
            &names
        ));
        assert!(!defines_any("fn supports_day_sixty_extra() {}", &names));
        assert!(!defines_any(
            "fn other() { not_fn supports_day_sixty }",
            &names
        ));
    }

    #[test]
    fn duration_is_additive_and_unknown_for_older_evidence() {
        // Evidence recorded before durations existed parses unchanged, with the duration unknown.
        let older = r#"{"schema":1,"verification":"behavior","runs":[{"runner":"libtest","attempt":1,"cases":[{"id":"a","status":"passed"}]}]}"#;
        let parsed: TestEvidence = serde_json::from_str(older).unwrap();
        assert_eq!(parsed.runs[0].duration_ms, None);
        // An untimed run serializes byte-identically to the older format, so stored identities
        // of existing evidence are unaffected.
        assert_eq!(serde_json::to_string(&parsed).unwrap(), older);

        let run = TestRun::new("libtest", 1, libtest_cases("test a ... ok\n"))
            .unwrap()
            .timed(std::time::Duration::from_millis(1234));
        assert_eq!(run.duration_ms, Some(1234));
        assert!(serde_json::to_string(&run)
            .unwrap()
            .ends_with(r#""duration_ms":1234}"#));
    }

    #[test]
    fn empty_runs_fail_closed_and_differences_are_named() {
        assert!(TestRun::new("libtest", 1, vec![]).is_err());
        let case = |id: &str, status: &str| TestCase {
            id: id.into(),
            file: None,
            status: status.into(),
        };
        assert!(differences(&[case("a", "passed")], &[case("a", "passed")]).is_empty());
        assert_eq!(
            differences(
                &[case("a", "passed"), case("b", "passed")],
                &[case("a", "failed")]
            ),
            ["a: passed -> failed", "b: passed -> absent"]
        );
    }
}
