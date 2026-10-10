use serde_json::Value;
use std::fs;
use std::process::Command;

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".codeos/00-project")).unwrap();
    fs::create_dir_all(dir.path().join(".codeos/01-specification/intents")).unwrap();
    fs::write(
        dir.path()
            .join(".codeos/01-specification/intents/F-0001.md"),
        "# Intent: F-0001 — Monitor feature\n",
    )
    .unwrap();
    dir
}

fn run(project: &tempfile::TempDir, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
        .arg("--project")
        .arg(project.path())
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn project_status_json_is_structured_and_read_only() {
    let project = project();
    let before = fs::read_to_string(
        project
            .path()
            .join(".codeos/01-specification/intents/F-0001.md"),
    )
    .unwrap();

    let output = run(&project, &["status", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["project"]["configuration"], "DBA-7");
    assert_eq!(json["current_feature"], "F-0001");
    assert_eq!(json["features"][0]["name"], "Monitor feature");
    assert_eq!(json["planned_feature_candidates"], serde_json::json!([]));
    assert_eq!(json["feature_inventory_warnings"], serde_json::json!([]));
    assert!(!project.path().join(".codeos/06-workflow").exists());
    assert_eq!(
        before,
        fs::read_to_string(
            project
                .path()
                .join(".codeos/01-specification/intents/F-0001.md")
        )
        .unwrap()
    );
}

#[test]
fn project_status_lists_unstarted_decomposition_candidates_without_governing_state() {
    let project = project();
    let discovery = project.path().join(".codeos/00-discovery");
    fs::create_dir_all(&discovery).unwrap();
    fs::write(
        discovery.join("a-first.md"),
        r#"---
artifact_type: feature_decomposition
---

## Candidate Feature Boundaries

| Candidate | Human outcome | Boundary or dependency |
|---|---|---|
| First candidate by source path | A person gains the first later outcome. | Independent candidate. |
"#,
    )
    .unwrap();
    fs::write(
        discovery.join("features.md"),
        r#"---
artifact_type: feature_decomposition
---

# Feature Decomposition

## Candidate Feature Boundaries

| Candidate | Human outcome | Boundary or dependency |
|---|---|---|
| Monitor feature | Already represented by the Intent. | Existing feature. |
| Later feature | A person gains a later outcome. | Begins after the monitor feature. |
| Conditional feature — boundary unresolved | A person may gain another outcome. | Requires a later decision. |
"#,
    )
    .unwrap();

    let output = run(&project, &["status", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["current_feature"], "F-0001");
    assert_eq!(json["features"].as_array().unwrap().len(), 1);
    let candidates = json["planned_feature_candidates"].as_array().unwrap();
    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[0]["name"], "First candidate by source path");
    assert_eq!(candidates[1]["name"], "Later feature");
    assert_eq!(
        candidates[1]["human_outcome"],
        "A person gains a later outcome."
    );
    assert_eq!(
        candidates[1]["boundary_or_dependency"],
        "Begins after the monitor feature."
    );
    assert_eq!(candidates[1]["source"], ".codeos/00-discovery/features.md");
    assert_eq!(
        candidates[2]["name"],
        "Conditional feature — boundary unresolved"
    );

    let rendered = run(&project, &["status"]);
    let stdout = String::from_utf8_lossy(&rendered.stdout);
    assert!(stdout.contains("Later feature  CANDIDATE (NOT STARTED)"));
    assert!(stdout.contains("Conditional feature — boundary unresolved  CANDIDATE (NOT STARTED)"));
    assert!(!stdout.contains("Monitor feature  CANDIDATE"));
}

#[test]
fn malformed_decomposition_warns_without_changing_workflow_exit() {
    let project = project();
    let discovery = project.path().join(".codeos/00-discovery");
    fs::create_dir_all(&discovery).unwrap();
    fs::write(
        discovery.join("malformed.md"),
        "---\nartifact_type: feature_decomposition\n---\n\n# Feature Decomposition\n",
    )
    .unwrap();

    let output = run(&project, &["status"]);
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FEATURE INVENTORY WARNINGS"));
    assert!(stdout.contains(
        ".codeos/00-discovery/malformed.md: missing 'Candidate Feature Boundaries' section"
    ));
    assert!(stdout.contains("Active feature     F-0001 — Monitor feature"));
}

#[test]
fn targeted_status_remains_available_and_json_rejects_a_target() {
    let project = project();
    let targeted = run(
        &project,
        &["status", "--workflow", "feature", "--subject", "F-0001"],
    );
    assert_eq!(targeted.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&targeted.stdout).contains("workflow: feature   subject: F-0001")
    );

    let invalid = run(
        &project,
        &[
            "status",
            "--workflow",
            "feature",
            "--subject",
            "F-0001",
            "--json",
        ],
    );
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("cannot be used with"));
}

fn dba25_project() -> tempfile::TempDir {
    let dir = project();
    let root = dir.path();
    std::os::unix::fs::symlink(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../.."),
        root.join(".codeos/toolkit"),
    )
    .unwrap();
    fs::write(
        root.join(".codeos/00-project/codeos.yaml"),
        "codeos_schema: 2\ndba_configuration: DBA-25\n",
    )
    .unwrap();
    for (dir, suffix) in [
        ("intents", ".md"),
        ("contracts", "_contract.md"),
        ("event-schemas", "_schema.md"),
    ] {
        let dir = root.join(".codeos/01-specification").join(dir);
        fs::create_dir_all(&dir).unwrap();
        for feature in ["F-0002", "F-0003"] {
            fs::write(dir.join(format!("{feature}{suffix}")), "draft\n").unwrap();
        }
    }
    // F-0002 was accepted against an Intent that has since changed.
    fs::create_dir_all(root.join(".codeos/06-workflow")).unwrap();
    fs::write(
        root.join(".codeos/06-workflow/decisions.jsonl"),
        r#"{"workflow":"feature","subject":"F-0002","checkpoint":"acceptance","result":"accepted","bindings":{"intent":"old"},"timestamp":"2026-10-01T00:00:00Z"}
"#,
    )
    .unwrap();
    dir
}

fn snapshot(root: &std::path::Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    let mut pending = vec![root.join(".codeos")];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_symlink() {
                continue;
            }
            if path.is_dir() {
                pending.push(path);
            } else {
                files.push((path.clone(), fs::read(&path).unwrap()));
            }
        }
    }
    files.sort();
    files
}

#[test]
fn attention_follows_workflow_order_with_actor_classes() {
    let project = dba25_project();
    let receipts = project.path().join(".codeos/06-workflow/decisions.jsonl");
    assert!(fs::read_to_string(&receipts).unwrap().contains("F-0002"));
    let before = snapshot(project.path());

    let output = run(&project, &["status", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema_version"], 3);
    let attention = json["attention"].as_array().unwrap();
    let subjects: Vec<_> = attention
        .iter()
        .map(|item| item["subject"].as_str().unwrap())
        .filter(|subject| subject.starts_with("F-"))
        .collect();
    assert_eq!(subjects, ["F-0002", "F-0001", "F-0003"]);
    let stale = attention
        .iter()
        .find(|item| item["subject"] == "F-0002")
        .unwrap();
    assert_eq!(stale["checkpoint"], "F1");
    assert_eq!(stale["actor"], "ASSISTED");
    assert_eq!(stale["class"], "STALE");
    assert_eq!(stale["reason"], "SEMANTIC_CHANGE_REQUIRED");
    assert_eq!(
        stale["authority"],
        "dba/02-policies/workflow-governance/v4.md"
    );
    let item = |subject: &str| {
        attention
            .iter()
            .find(|item| item["subject"] == subject)
            .unwrap()
    };
    assert_eq!(item("F-0001")["class"], "BLOCKER");
    assert_eq!(
        item("F-0001")["authority"],
        "dba/02-policies/workflow-feature/v9.md"
    );
    assert_eq!(item("bootstrap")["class"], "BLOCKER");
    assert_eq!(
        item("bootstrap")["authority"],
        "dba/02-policies/workflow-bootstrap/v1.md"
    );
    for item in attention {
        let authority = item["authority"].as_str().unwrap();
        assert!(project
            .path()
            .join(".codeos/toolkit")
            .join(authority)
            .is_file());
        assert!(
            item["subject"] == "bootstrap" || item["subject"].as_str().unwrap().starts_with("F-")
        );
    }
    assert!(attention.iter().all(|item| item["actor"].is_string()));

    let health = json["health"].as_array().unwrap();
    assert_eq!(health.len(), 13);
    for item in health {
        assert!(!item["basis"].as_array().unwrap().is_empty(), "{item}");
    }
    let specification = health
        .iter()
        .find(|item| item["domain"] == "specification")
        .unwrap();
    assert_eq!(specification["status"], "STALE");
    assert_eq!(json["features"][1]["historically_accepted"], true);

    let rendered = String::from_utf8_lossy(&run(&project, &["status"]).stdout).to_string();
    assert!(rendered.contains("ENGINEERING HEALTH"), "{rendered}");
    assert!(
        rendered.contains("  Specification         STALE"),
        "{rendered}"
    );
    assert!(rendered.contains("ATTENTION"), "{rendered}");
    assert!(rendered.contains("next [ASSISTED]: start ordinary governed work at F1"));
    assert!(rendered.contains("[STALE] F-0002 — "), "{rendered}");
    assert!(
        rendered.contains("(SEMANTIC_CHANGE_REQUIRED)"),
        "{rendered}"
    );
    assert_eq!(before, snapshot(project.path()));
}

#[test]
fn open_observations_are_listed_until_closed() {
    let project = dba25_project();
    let observed = run(&project, &["observe", "--statement", "slow route lookup"]);
    assert!(
        observed.status.success(),
        "{}",
        String::from_utf8_lossy(&observed.stderr)
    );

    let json: Value = serde_json::from_slice(&run(&project, &["status", "--json"]).stdout).unwrap();
    assert_eq!(
        json["open_observations"],
        serde_json::json!([{"id": "OBS-0001", "route": null, "target": null}])
    );
    assert!(String::from_utf8_lossy(&run(&project, &["status"]).stdout)
        .contains("OPEN OBSERVATIONS\n  OBS-0001  not routed"));

    for args in [
        &[
            "route-observation",
            "--observation",
            "OBS-0001",
            "--route",
            "no_action",
        ][..],
        &[
            "close-observation",
            "--observation",
            "OBS-0001",
            "--rationale",
            "accepted as is",
        ],
    ] {
        let output = run(&project, args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let json: Value = serde_json::from_slice(&run(&project, &["status", "--json"]).stdout).unwrap();
    assert_eq!(json["open_observations"], serde_json::json!([]));
}
