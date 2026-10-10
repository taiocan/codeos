use codeos_workflow::checker::{evaluate, State, Subject};
use codeos_workflow::contract::Workflow;
use codeos_workflow::observations::ObservationStore;
use codeos_workflow::project::Project;
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};

struct Fx {
    _dir: tempfile::TempDir,
    root: std::path::PathBuf,
}

impl Fx {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        fs::create_dir_all(root.join(".codeos/00-project")).unwrap();
        fs::create_dir_all(root.join(".codeos/01-specification/intents")).unwrap();
        fs::create_dir_all(root.join(".codeos/06-workflow")).unwrap();
        symlink(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../.."),
            root.join(".codeos/toolkit"),
        )
        .unwrap();
        fs::write(
            root.join(".codeos/00-project/codeos.yaml"),
            "codeos_schema: 2\ndba_configuration: DBA-25\nplatform:\n  persistence: postgresql\n  backend: rust\n  webapp: svelte\n  runtime: docker\nartifacts:\n  charter: governed\n  intent: governed\n  contract: governed\n  event_schema: governed\n",
        )
        .unwrap();
        fs::write(
            root.join(".codeos/00-project/charter.md"),
            "---\nartifact_type: charter\napproval:\n  by: Human\n  at: 2026-10-09\n---\n\n## System Constraints\n\n| ID | Constraint | Type | Verification route |\n|---|---|---|---|\n| C-1 | usable | quality | scripts/codeos-qualify |\n",
        )
        .unwrap();
        let _ = std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status();
        Self { _dir: dir, root }
    }

    fn project(&self) -> Project {
        Project::discover(&self.root).unwrap()
    }
}

#[test]
fn qualification_is_project_owned_but_identity_bound() {
    let fx = Fx::new();
    fs::create_dir_all(fx.root.join("scripts")).unwrap();
    let script = fx.root.join("scripts/codeos-qualify");
    fs::write(
        &script,
        r#"#!/usr/bin/env bash
set -eu
printf '{"schema":1,"project":"fixture","implementation_sha256":"%s","charter_sha256":"%s","configuration_sha256":"%s","tools":{"fixture":"1"},"constraints":[{"id":"C-1","phase":"deployment","status":"unresolved","evidence":["fixture"],"measurements":{}}],"baseline":[{"id":"static_analysis","status":"pass","tool":"fixture","version":"1","summary":"ok"},{"id":"dependency_vulnerabilities","status":"pass","tool":"fixture","version":"1","summary":"ok"},{"id":"dependency_licences","status":"pass","tool":"fixture","version":"1","summary":"ok"},{"id":"committed_secrets","status":"pass","tool":"fixture","version":"1","summary":"ok"}]}' "$CODEOS_IMPLEMENTATION_SHA256" "$CODEOS_CHARTER_SHA256" "$CODEOS_CONFIGURATION_SHA256" > "$CODEOS_QUALIFICATION_OUTPUT"
"#,
    )
    .unwrap();
    let mut permissions = script.metadata().unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let outcome = codeos_workflow::qualification::run(&fx.project(), "acceptance");
    assert!(outcome.ok, "{}", outcome.summary);
    let deployment = codeos_workflow::qualification::run(&fx.project(), "deployment");
    assert!(!deployment.ok);
    assert!(deployment.summary.contains("C-1"));

    fs::write(
        &script,
        "#!/usr/bin/env bash\nprintf '{\"schema\":1}' > \"$CODEOS_QUALIFICATION_OUTPUT\"\n",
    )
    .unwrap();
    let outcome = codeos_workflow::qualification::run(&fx.project(), "acceptance");
    assert!(!outcome.ok);
}

#[test]
fn acceptance_qualification_cli_persists_subject_bound_candidate_evidence() {
    let fx = Fx::new();
    fs::create_dir_all(fx.root.join("scripts")).unwrap();
    fs::write(
        fx.root.join(".codeos/01-specification/intents/F-0001.md"),
        "---\nsemantic_revision: 1\n---\n",
    )
    .unwrap();
    let script = fx.root.join("scripts/codeos-qualify");
    fs::write(
        &script,
        r#"#!/usr/bin/env bash
set -eu
printf '{"schema":1,"project":"fixture","implementation_sha256":"%s","charter_sha256":"%s","configuration_sha256":"%s","tools":{"fixture":"1"},"constraints":[{"id":"C-1","phase":"acceptance","status":"pass","evidence":["fixture"],"measurements":{}}],"baseline":[{"id":"static_analysis","status":"pass","tool":"fixture","version":"1","summary":"ok"},{"id":"dependency_vulnerabilities","status":"pass","tool":"fixture","version":"1","summary":"ok"},{"id":"dependency_licences","status":"pass","tool":"fixture","version":"1","summary":"ok"},{"id":"committed_secrets","status":"pass","tool":"fixture","version":"1","summary":"ok"}]}' "$CODEOS_IMPLEMENTATION_SHA256" "$CODEOS_CHARTER_SHA256" "$CODEOS_CONFIGURATION_SHA256" > "$CODEOS_QUALIFICATION_OUTPUT"
"#,
    )
    .unwrap();
    let mut permissions = script.metadata().unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let status = std::process::Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
        .args([
            "--project",
            fx.root.to_str().unwrap(),
            "qualify-project",
            "--mode",
            "acceptance",
            "--subject",
            "F-0001",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let record = codeos_workflow::verification::VerificationStore::at(&fx.project().codeos_dir())
        .latest("feature", "F-0001", "candidate", "project_qualification")
        .unwrap()
        .unwrap();
    assert_eq!(record.result, "passed");
    assert!(record.bindings.contains_key("charter"));
    assert!(record.bindings.contains_key("qualification_script"));
    assert!(record.bindings.contains_key("codeos_yaml"));
}

#[test]
fn observations_have_independent_identity_route_and_closure() {
    let fx = Fx::new();
    fs::write(
        fx.root.join(".codeos/01-specification/intents/F-0001.md"),
        "---\nsemantic_revision: 1\n---\n",
    )
    .unwrap();
    let project = fx.project();
    let store = ObservationStore::at(&project.codeos_dir());
    let first = store
        .observe("first defect".into(), vec!["traveler-1".into()])
        .unwrap();
    let second = store
        .observe("second defect".into(), vec!["traveler-2".into()])
        .unwrap();
    assert_eq!(first.id, "OBS-0001");
    assert_eq!(second.id, "OBS-0002");
    store
        .route_observation(
            &project,
            &first.id,
            "implementation_defect",
            Some("feature".into()),
            Some("F-0001".into()),
        )
        .unwrap();
    store
        .route_observation(
            &project,
            &second.id,
            "implementation_defect",
            Some("feature".into()),
            Some("F-0001".into()),
        )
        .unwrap();
    assert_eq!(store.routes().unwrap().len(), 2);

    let no_action = store
        .observe("considered cosmetic issue".into(), vec![])
        .unwrap();
    store
        .route_observation(&project, &no_action.id, "no_action", None, None)
        .unwrap();
    store
        .close(
            &project,
            &no_action.id,
            vec![],
            Some("deliberately accepted".into()),
        )
        .unwrap();
    let reports = evaluate(
        &project,
        &Subject::resolve(&project, Workflow::Operation, &no_action.id).unwrap(),
    )
    .unwrap();
    assert!(reports.iter().all(|report| report.state == State::Pass));
}

fn health_of(project: &Project, domain: &str) -> (String, Vec<String>) {
    let status = codeos_workflow::monitor::ProjectStatus::derive(project).unwrap();
    let json = serde_json::to_value(&status.health).unwrap();
    let item = json
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["domain"] == domain)
        .unwrap_or_else(|| panic!("no {domain} health"))
        .clone();
    (
        item["status"].as_str().unwrap().to_string(),
        serde_json::from_value(item["basis"].clone()).unwrap(),
    )
}

fn qualification_record(fx: &Fx, details: serde_json::Value, result: &str) {
    let project = fx.project();
    let subject = Subject::resolve(&project, Workflow::Feature, "F-0001").unwrap();
    let bindings = codeos_workflow::checker::verification_bindings_for_mechanic(
        &project,
        &subject,
        "F9",
        "project_qualification",
    )
    .unwrap();
    codeos_workflow::verification::VerificationStore::at(&project.codeos_dir())
        .append(&codeos_workflow::verification::VerificationRecord {
            workflow: "feature".into(),
            subject: "F-0001".into(),
            checkpoint: "F9".into(),
            verification: "project_qualification".into(),
            result: result.into(),
            bindings,
            command: "scripts/codeos-qualify".into(),
            timestamp: "2026-10-10T00:00:00Z".into(),
            details: Some(details),
        })
        .unwrap();
}

#[test]
fn qualification_health_uses_only_an_identity_current_record() {
    let fx = Fx::new();
    fs::write(
        fx.root.join(".codeos/01-specification/intents/F-0001.md"),
        "---\nsemantic_revision: 1\n---\n",
    )
    .unwrap();
    let (status, basis) = health_of(&fx.project(), "security");
    assert_eq!(status, "NOT_CONFIGURED");
    assert_eq!(basis, ["scripts/codeos-qualify does not exist"]);

    fs::create_dir_all(fx.root.join("scripts")).unwrap();
    fs::write(fx.root.join("scripts/codeos-qualify"), "#!/bin/sh\n").unwrap();
    assert_eq!(health_of(&fx.project(), "security").0, "UNOBSERVED");

    qualification_record(
        &fx,
        serde_json::json!({
            "constraints": [{"id": "C-1", "phase": "deployment", "status": "unresolved"}],
            "baseline": [
                {"id": "static_analysis", "status": "pass"},
                {"id": "dependency_vulnerabilities", "status": "pass"},
                {"id": "dependency_licences", "status": "not_applicable"},
                {"id": "committed_secrets", "status": "pass"}
            ]
        }),
        "passed",
    );
    let project = fx.project();
    assert_eq!(health_of(&project, "security").0, "PASS");
    assert_eq!(health_of(&project, "supply_chain").0, "PASS");
    let (deployment, basis) = health_of(&project, "deployment");
    assert_eq!(deployment, "BLOCKED");
    assert!(basis.contains(&"C-1: unresolved".to_string()), "{basis:?}");
    assert!(basis[0].contains("F-0001 F9 project_qualification record"));

    // The newest record no longer matches the Charter it was bound to: never PASS.
    let charter = fx.root.join(".codeos/00-project/charter.md");
    let text = fs::read_to_string(&charter).unwrap();
    fs::write(&charter, format!("{text}\n")).unwrap();
    let (status, basis) = health_of(&fx.project(), "security");
    assert_eq!(status, "STALE");
    assert!(basis[0].contains("'charter' changed"), "{basis:?}");
}
