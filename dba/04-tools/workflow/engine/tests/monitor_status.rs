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
        // The terminal report wraps to COLUMNS; pin it so assertions do not depend on the caller.
        .env("COLUMNS", "120")
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
    let row = |name: &str| {
        stdout
            .lines()
            .find(|line| line.starts_with("  —") && line.contains(name))
            .map(str::to_string)
    };
    for name in ["Later feature", "Conditional feature — boundary unresolved"] {
        assert!(
            row(name).is_some_and(|row| row.contains("CANDIDATE (NOT STARTED)")),
            "{stdout}"
        );
    }
    assert!(row("Monitor feature").is_none(), "{stdout}");
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
    assert!(stdout.contains("  Active feature  F-0001 — Monitor feature\n"));
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
    assert_eq!(
        stale["action"],
        serde_json::json!({"type": "revise_specification", "command": null})
    );
    for item in attention {
        assert!(item["action"]["type"].is_string(), "{item}");
        if !item["action"]["command"].is_null() {
            assert_eq!(item["actor"], "AUTO", "{item}");
        }
    }
    let affected = stale["affected_surfaces"].as_array().unwrap();
    let intent = affected.iter().find(|c| c["surface"] == "intent").unwrap();
    assert_eq!(intent["was"], "old");
    assert!(intent["now"].is_string());
    // The fixture receipt never recorded the contract; validity compared it as unrecorded.
    let contract = affected
        .iter()
        .find(|c| c["surface"] == "contract")
        .unwrap();
    assert!(contract["was"].is_null());
    assert!(stale.get("unchanged_surfaces").is_none());
    let accepted = json["features"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == "F-0002")
        .unwrap();
    assert_eq!(
        stale["evidence"][0],
        serde_json::json!({
            "kind": "acceptance_receipt",
            "id": accepted["accepted_snapshot"]["receipt_sha256"]
        })
    );
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
    for domain in ["verification", "evidence", "independent_review"] {
        let item = health.iter().find(|item| item["domain"] == domain).unwrap();
        assert_eq!(item["status"], "STALE", "{domain}");
    }
    assert_eq!(json["features"][1]["historically_accepted"], true);

    let rendered = String::from_utf8_lossy(&run(&project, &["status"]).stdout).to_string();
    let section = |name: &str| {
        rendered
            .split(&format!("\n{name}\n"))
            .nth(1)
            .unwrap_or_else(|| panic!("no {name} section: {rendered}"))
            .split("\n\n")
            .next()
            .unwrap()
            .to_string()
    };
    let row = |name: &str, first: &str| {
        section(name)
            .lines()
            .find(|line| line.starts_with(&format!("  {first} ")))
            .unwrap_or_else(|| panic!("no {first} row in {name}: {rendered}"))
            .to_string()
    };
    assert!(
        row("HEALTH", "Specification").contains(" STALE "),
        "{rendered}"
    );
    // The stale accepted feature is an attention item with its class, actor, and changed surfaces.
    let stale_row = row("ATTENTION", "F-0002");
    assert!(stale_row.contains(" STALE "), "{rendered}");
    assert!(stale_row.contains(" ASSISTED "), "{rendered}");
    assert!(
        stale_row.contains("intent, contract, event_schema"),
        "{rendered}"
    );
    assert!(
        section("DETAILS")
            .lines()
            .any(|line| line.contains("F-0002")
                && line.contains(" next ")
                && line.contains("start ordinary governed work at F1")),
        "{rendered}"
    );
    assert!(
        rendered.contains("(SEMANTIC_CHANGE_REQUIRED)"),
        "{rendered}"
    );

    // At 80 columns every line fits except the whole commands, and output is deterministic.
    let narrow = |_: ()| {
        let output = Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
            .arg("--project")
            .arg(project.path())
            .arg("status")
            .env("COLUMNS", "80")
            .output()
            .unwrap();
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    let at80 = narrow(());
    assert_eq!(at80, narrow(()));
    let body = at80.split("\nCOMMANDS\n").next().unwrap();
    assert!(
        body.lines().all(|line| line.chars().count() <= 80),
        "{at80}"
    );

    // Accepted Snapshot and Current Validity are shown apart from incomplete work (WO-08).
    assert!(
        rendered.contains(
            "  Active feature  none derived — 2 not yet accepted; 1 accepted, currently STALE\n"
        ),
        "{rendered}"
    );
    assert!(!rendered.contains("incomplete"), "{rendered}");
    let accepted = row("FEATURES", "F-0002");
    assert!(accepted.contains(" 2026-10-01 "), "{rendered}");
    assert!(
        accepted.ends_with(" STALE (SEMANTIC_CHANGE_REQUIRED)"),
        "{rendered}"
    );
    assert!(!row("FEATURES", "F-0001").contains("2026-"), "{rendered}");
    // Every domain is shown with its state, and every basis remains: verbatim, grouped after its
    // feature IDs, or referenced as the same basis as an earlier domain.
    let health_text = section("HEALTH")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut earlier: Vec<Vec<&str>> = Vec::new();
    for item in health {
        let domain = item["domain"].as_str().unwrap().replace('_', " ");
        let label = format!("{}{}", domain[..1].to_uppercase(), &domain[1..]);
        let head = row("HEALTH", &label);
        let bases: Vec<&str> = item["basis"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b.as_str().unwrap())
            .collect();
        if !bases.is_empty() && earlier.contains(&bases) {
            assert!(head.contains("same as "), "{head}");
        } else {
            for basis in &bases {
                let rest = match basis.split_once([':', ' ']) {
                    Some((id, _)) if id.starts_with("F-") => &basis[id.len()..],
                    _ => basis,
                };
                let rest = rest.split_whitespace().collect::<Vec<_>>().join(" ");
                assert!(health_text.contains(&rest), "{basis} missing: {rendered}");
            }
        }
        earlier.push(bases);
    }
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
