use chrono::NaiveDate;
use codeos_workflow::hashing::file_sha256;
use codeos_workflow::history::{self, State, Window};
use codeos_workflow::project::Project;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

fn write(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// Records dated on both sides of 2026-10-01.
fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for (rel, kind) in [
        (".codeos/01-specification/intents/F-0001.md", "intent"),
        (
            ".codeos/01-specification/contracts/F-0001_contract.md",
            "contract",
        ),
        (
            ".codeos/01-specification/event-schemas/F-0001_schema.md",
            "event_schema",
        ),
    ] {
        write(
            root,
            rel,
            &format!(
                "---\nstatus: APPROVED\napproved_by: Human\napproved_at: 2026-09-30\n---\n{kind}\n"
            ),
        );
    }
    let hash = |rel: &str| file_sha256(&root.join(rel)).unwrap().unwrap();
    let bound = serde_json::json!({
        "intent": hash(".codeos/01-specification/intents/F-0001.md"),
        "contract": hash(".codeos/01-specification/contracts/F-0001_contract.md"),
        "event_schema": hash(".codeos/01-specification/event-schemas/F-0001_schema.md"),
    });
    let receipt = |subject: &str, bindings: &Value, at: &str| {
        serde_json::json!({"workflow": "feature", "subject": subject, "checkpoint": "acceptance",
            "result": "accepted", "bindings": bindings, "timestamp": at})
        .to_string()
    };
    write(
        root,
        ".codeos/06-workflow/decisions.jsonl",
        &[
            receipt("F-0001", &bound, "2026-09-20T00:00:00Z"),
            receipt("F-0001", &bound, "2026-10-02T00:00:00Z"),
            receipt("F-0001", &bound, "2026-10-04T00:00:00Z"),
            receipt(
                "F-0002",
                &serde_json::json!({"intent": "old"}),
                "2026-10-01T00:00:00Z",
            ),
        ]
        .join("\n"),
    );
    let verification = |result: &str, at: &str| {
        serde_json::json!({"workflow": "feature", "subject": "F-0001", "checkpoint": "F3",
            "verification": "smoke", "result": result, "bindings": {}, "command": "x",
            "timestamp": at})
        .to_string()
    };
    write(
        root,
        ".codeos/06-workflow/verifications.jsonl",
        &[
            verification("failed", "2026-09-01T00:00:00Z"),
            verification("failed", "2026-10-05T00:00:00Z"),
            verification("passed", "2026-10-06T00:00:00Z"),
        ]
        .join("\n"),
    );
    write(
        root,
        ".codeos/05-review/reviews/review-log.md",
        "# Review log\n\n## 2026-09-01T00:00:00Z REVIEW — F-0001 — Stage 8\nEffective concern: DO NOT ADVANCE\n\n## 2026-10-03T00:00:00Z REVIEW — F-0001 — Stage 8\nEffective concern: DO NOT ADVANCE\n\n## 2026-10-03T06:00:00Z REVIEW — F-0001 — Stage 8\nEffective concern: ADVANCE\n\n## 2026-10-04T00:00:00Z REVIEW EXECUTION INVALID — F-0001 — Stage 8\nResult: REVIEW_EXECUTION_INVALID\n\n## 2026-10-05T00:00:00Z REVIEW\nEffective concern: DO NOT ADVANCE\n",
    );
    write(
        root,
        ".codeos/06-workflow/observations.jsonl",
        r#"{"schema":2,"id":"OBS-0001","statement":"slow","statement_sha256":"s","sources":[],"timestamp":"2026-10-02T00:00:00.000Z"}"#,
    );
    write(
        root,
        ".codeos/06-workflow/operation-routes.jsonl",
        r#"{"schema":2,"observation":"OBS-0001","observation_sha256":"s","route":"implementation_defect","target_type":"feature","target_id":"F-0001","timestamp":"2026-10-02T01:00:00.000Z"}"#,
    );
    write(
        root,
        ".codeos/06-workflow/operation-closures.jsonl",
        r#"{"schema":2,"observation":"OBS-0001","observation_sha256":"s","route":"implementation_defect","evidence":[],"timestamp":"2026-10-03T00:00:00.000Z"}"#,
    );
    dir
}

fn metric<'a>(history: &'a history::History, name: &str) -> &'a history::Metric {
    history
        .metrics
        .iter()
        .find(|m| m.metric == name)
        .unwrap_or_else(|| panic!("no metric {name}"))
}

#[test]
fn history_counts_only_what_records_establish_within_an_inclusive_utc_window() {
    let dir = fixture();
    let project = Project::discover(dir.path()).unwrap();
    let since = NaiveDate::from_ymd_opt(2026, 10, 1);
    let history = history::derive(&project, since).unwrap();
    assert_eq!(history.since, "2026-10-01");

    for (name, value) in [
        // F-0001 (accepted twice in the window) and F-0002 exactly at the inclusive boundary;
        // distinct subjects.
        ("accepted_features", 2),
        // Both in-window F-0001 repeats count, although the first receipt precedes the window.
        ("additional_decision_receipts", 2),
        ("failed_verifications", 1),
        // One attributed entry and one malformed heading, which still counts.
        ("non_progressing_assessments", 2),
        ("execution_invalid", 1),
        ("candidate_bundles", 0),
        ("observations_opened", 1),
        ("observations_closed", 1),
        ("implementation_defect_routes", 1),
    ] {
        let m = metric(&history, name);
        assert_eq!((m.state, m.value), (State::Observed, Some(value)), "{name}");
        assert!(!m.source.is_empty(), "{name}");
    }
    assert_eq!(
        metric(&history, "candidate_bundles").window,
        Window::AllTime
    );
    for name in [
        "implementation_escapes",
        "not_ready",
        "evidence_invalidations",
        "attributed_waiting",
    ] {
        let m = metric(&history, name);
        assert_eq!((m.state, m.value), (State::Unobserved, None), "{name}");
    }

    let all = history::derive(&project, None).unwrap();
    assert_eq!(all.since, "all");
    assert_eq!(metric(&all, "failed_verifications").value, Some(2));
    assert_eq!(metric(&all, "accepted_features").value, Some(2));
    assert_eq!(metric(&all, "additional_decision_receipts").value, Some(2));
}

/// (subject, checkpoint, value) parts, and the unattributed count.
fn located(history: &history::History, name: &str) -> (Vec<(String, Option<String>, u64)>, u64) {
    let m = metric(history, name);
    let parts = m.breakdown.as_ref().unwrap();
    (
        parts
            .iter()
            .map(|p| (p.subject.clone(), p.checkpoint.clone(), p.value))
            .collect(),
        m.unattributed.unwrap(),
    )
}

#[test]
fn breakdowns_locate_exactly_the_counted_events() {
    let dir = fixture();
    let project = Project::discover(dir.path()).unwrap();
    let history = history::derive(&project, NaiveDate::from_ymd_opt(2026, 10, 1)).unwrap();
    let at = |subject: &str, checkpoint: Option<&str>, value| {
        (subject.to_string(), checkpoint.map(str::to_string), value)
    };
    assert_eq!(
        located(&history, "accepted_features"),
        (vec![at("F-0001", None, 1), at("F-0002", None, 1)], 0)
    );
    assert_eq!(
        located(&history, "additional_decision_receipts"),
        (vec![at("F-0001", Some("acceptance"), 2)], 0)
    );
    assert_eq!(
        located(&history, "failed_verifications"),
        (vec![at("F-0001", Some("F3"), 1)], 0)
    );
    assert_eq!(
        located(&history, "non_progressing_assessments"),
        (vec![at("F-0001", Some("Stage 8"), 1)], 1)
    );
    assert_eq!(
        located(&history, "execution_invalid"),
        (vec![at("F-0001", Some("Stage 8"), 1)], 0)
    );
    assert_eq!(
        located(&history, "implementation_defect_routes"),
        (vec![at("F-0001", None, 1)], 0)
    );
    // Observations name no subject; a later route target is not applied to them.
    assert_eq!(located(&history, "observations_opened"), (vec![], 1));

    for m in history
        .metrics
        .iter()
        .filter(|m| m.state == State::Observed)
    {
        let parts: u64 = m.breakdown.as_ref().unwrap().iter().map(|p| p.value).sum();
        assert_eq!(
            Some(parts + m.unattributed.unwrap()),
            m.value,
            "{}",
            m.metric
        );
    }
    for m in history
        .metrics
        .iter()
        .filter(|m| m.state == State::Unobserved)
    {
        assert!(
            m.breakdown.is_none() && m.unattributed.is_none(),
            "{}",
            m.metric
        );
    }

    let rendered = history::render(&history);
    // Child values align with the metric's value column.
    assert!(rendered.contains(&format!(
        "  {:<30}2\n    {:<28}1\n    {:<28}1\n",
        "non_progressing_assessments", "F-0001 Stage 8", "unattributed"
    )));
    assert!(rendered.contains("  execution_invalid             1  (F-0001 Stage 8)\n"));
    assert!(rendered.contains("  observations_opened           1\n"));
}

#[test]
fn durations_are_elapsed_between_named_records_and_never_substitute_another_package() {
    let dir = fixture();
    let project = Project::discover(dir.path()).unwrap();
    let history = history::derive(&project, NaiveDate::from_ymd_opt(2026, 10, 1)).unwrap();
    let find = |metric: &str, subject: &str| {
        history
            .durations
            .iter()
            .find(|d| d.metric == metric && d.subject == subject)
            .unwrap_or_else(|| panic!("no {metric} for {subject}"))
    };
    assert_eq!(
        find("observation_closure", "OBS-0001").elapsed_seconds,
        Some(86_400)
    );
    // The accepted package still matches its binding: approval 2026-09-30 → acceptance 10-02.
    let accepted = find("specification_to_acceptance", "F-0001");
    assert_eq!(accepted.elapsed_seconds, Some(2 * 86_400));
    assert!(accepted
        .from
        .as_ref()
        .unwrap()
        .record
        .contains("approved_at"));
    // F-0002's acceptance bound a different package; no substitute approval is used.
    let unmatched = find("specification_to_acceptance", "F-0002");
    assert_eq!(unmatched.state, State::Unobserved);
    assert!(unmatched.from.is_none() && unmatched.elapsed_seconds.is_none());

    let json = serde_json::to_string(&history).unwrap();
    assert!(!json.contains("waiting_seconds"));
    let rendered = history::render(&history);
    assert!(rendered.contains("DURATIONS (elapsed time, not attributed waiting)"));
    assert!(rendered.starts_with("HISTORY — since 2026-10-01 00:00 UTC (inclusive)"));
}

#[test]
fn window_start_parses_dates_all_and_the_default() {
    let today = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();
    assert_eq!(history::window_start(Some("all"), today).unwrap(), None);
    assert_eq!(
        history::window_start(None, today).unwrap(),
        NaiveDate::from_ymd_opt(2026, 10, 1)
    );
    assert!(history::window_start(Some("10/01/2026"), today).is_err());
}

fn status_project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join(".codeos/00-project")).unwrap();
    std::os::unix::fs::symlink(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.."),
        root.join(".codeos/toolkit"),
    )
    .unwrap();
    write(
        root,
        ".codeos/00-project/codeos.yaml",
        "codeos_schema: 2\ndba_configuration: DBA-25\n",
    );
    write(
        root,
        ".codeos/01-specification/intents/F-0001.md",
        "draft\n",
    );
    dir
}

fn run(dir: &tempfile::TempDir, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
        .arg("--project")
        .arg(dir.path())
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn history_is_opt_in_and_leaves_plain_status_unchanged() {
    let dir = status_project();
    let plain = run(&dir, &["status", "--json"]);
    let json: Value = serde_json::from_slice(&plain.stdout).unwrap();
    assert!(json.get("history").is_none());
    assert_eq!(run(&dir, &["status", "--json"]).stdout, plain.stdout);

    let with = run(&dir, &["status", "--json", "--history", "--since", "all"]);
    let json: Value = serde_json::from_slice(&with.stdout).unwrap();
    assert_eq!(json["history"]["since"], "all");
    assert_eq!(with.status.code(), plain.status.code());

    let terminal = run(&dir, &["status", "--history"]);
    assert!(String::from_utf8_lossy(&terminal.stdout).starts_with("HISTORY — since "));
    assert!(!run(&dir, &["status", "--since", "all"]).status.success());
    assert!(!run(&dir, &["status", "--history", "--since", "bad"])
        .status
        .success());
    assert!(!dir.path().join(".codeos/06-workflow").exists());
}
