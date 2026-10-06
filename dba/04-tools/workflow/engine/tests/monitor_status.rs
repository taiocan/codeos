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
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["project"]["configuration"], "DBA-7");
    assert_eq!(json["current_feature"], "F-0001");
    assert_eq!(json["features"][0]["name"], "Monitor feature");
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
