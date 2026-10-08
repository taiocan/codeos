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
