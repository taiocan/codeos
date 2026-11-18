//! VALIDITY WO-03: deterministic preflight reports the defect classes JamFree met late in
//! 2026-11-13..17 (a YAML colon in semantic_change, an unsettled privacy prerequisite, an open
//! advisory) before any approval. It inspects the recorded qualification state, never runs the
//! qualification, and records nothing.

mod fai_fixture;

use codeos_workflow::preflight::Status;
use fai_fixture::*;
use std::fs;

/// A feature with a project qualification stub whose C-1 and dependency-vulnerability results are
/// given; each run of the stub is counted.
fn project(c1: &str, vulnerabilities: &str) -> Fixture {
    let fx = Fixture::under("DBA-35", "    provider: [backend]\n");
    fx.spec("F-0001", ADVANCE, ROUTE);
    let charter = fs::read_to_string(fx.root.join(".codeos/00-project/charter.md")).unwrap();
    fx.write(
        ".codeos/00-project/charter.md",
        &format!("{charter}\n## System Constraints\n\n| ID | Constraint | Type | Verification route |\n|---|---|---|---|\n| C-1 | private | quality | scripts/codeos-qualify |\n"),
    );
    fx.write(
        "scripts/codeos-qualify",
        &format!(
            r#"#!/usr/bin/env bash
set -eu
echo run >> "$CODEOS_PROJECT_ROOT/qualify-runs"
printf '{{"schema":1,"project":"fixture","implementation_sha256":"%s","charter_sha256":"%s","configuration_sha256":"%s","tools":{{"fixture":"1"}},"constraints":[{{"id":"C-1","phase":"acceptance","status":"{c1}","evidence":[],"measurements":{{}}}}],"baseline":[{{"id":"static_analysis","status":"pass","tool":"fixture","version":"1","summary":"ok"}},{{"id":"dependency_vulnerabilities","status":"{vulnerabilities}","tool":"fixture","version":"1","summary":"advisories"}},{{"id":"dependency_licences","status":"pass","tool":"fixture","version":"1","summary":"ok"}},{{"id":"committed_secrets","status":"pass","tool":"fixture","version":"1","summary":"ok"}}]}}' "$CODEOS_IMPLEMENTATION_SHA256" "$CODEOS_CHARTER_SHA256" "$CODEOS_CONFIGURATION_SHA256" > "$CODEOS_QUALIFICATION_OUTPUT"
"#
        ),
    );
    std::process::Command::new("chmod")
        .args(["+x", "scripts/codeos-qualify"])
        .current_dir(&fx.root)
        .status()
        .unwrap();
    fx.write(
        ".gitignore",
        ".codeos/toolkit\nbackend/target\nqualify-runs\n",
    );
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "fixture"]);
    fx
}

/// The project's own acceptance qualification, recorded as `qualify-project` records it.
fn qualify(fx: &Fixture) {
    std::process::Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
        .args(["--project", fx.root.to_str().unwrap()])
        .args([
            "qualify-project",
            "--mode",
            "acceptance",
            "--subject",
            "F-0001",
        ])
        .output()
        .unwrap();
}

fn runs(fx: &Fixture) -> usize {
    fs::read_to_string(fx.root.join("qualify-runs"))
        .map(|t| t.lines().count())
        .unwrap_or(0)
}

/// The non-PASS findings, checking that preflight ran no qualification.
fn findings(fx: &Fixture) -> Vec<(String, Status, String)> {
    let before = runs(fx);
    let out = codeos_workflow::preflight::run(&fx.project(), "F-0001")
        .into_iter()
        .filter(|f| f.status != Status::Pass)
        .map(|f| (f.check.to_string(), f.status, f.detail))
        .collect();
    assert_eq!(runs(fx), before, "preflight never runs the qualification");
    out
}

fn blocked(reason: &str) -> Status {
    Status::Blocked(reason.into())
}

#[test]
fn a_clean_qualified_feature_passes_and_preflight_records_nothing() {
    let fx = project("pass", "pass");
    qualify(&fx);
    let store = fs::read(fx.root.join(".codeos/06-workflow/verifications.jsonl")).unwrap();
    assert_eq!(findings(&fx), vec![]);
    assert_eq!(
        fs::read(fx.root.join(".codeos/06-workflow/verifications.jsonl")).unwrap(),
        store,
        "preflight records nothing"
    );
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
        .args([
            "--project",
            fx.root.to_str().unwrap(),
            "preflight",
            "--subject",
            "F-0001",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn a_yaml_colon_in_semantic_change_blocks_the_specification() {
    let fx = project("pass", "pass");
    qualify(&fx);
    let path = fx.root.join(".codeos/01-specification/intents/F-0001.md");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        text.replace(
            "semantic_change: initial feature specification",
            "semantic_change: Boundary Path 3: the one-form page",
        ),
    )
    .unwrap();
    let found = findings(&fx);
    assert!(
        found
            .iter()
            .any(|(check, status, detail)| check == "specification"
                && *status == blocked("DEFECT")
                && detail.contains("does not parse")),
        "{found:?}"
    );
}

#[test]
fn an_unsettled_privacy_prerequisite_blocks_on_the_recorded_qualification() {
    let fx = project("unresolved", "pass");
    qualify(&fx);
    let found = findings(&fx);
    assert!(
        found
            .iter()
            .any(|(check, status, detail)| check == "qualification"
                && *status == blocked("QUALIFICATION_BLOCKERS")
                && detail.contains("C-1")),
        "{found:?}"
    );
}

#[test]
fn an_open_advisory_blocks_on_the_recorded_qualification() {
    let fx = project("pass", "fail");
    qualify(&fx);
    let found = findings(&fx);
    assert!(
        found
            .iter()
            .any(|(check, status, detail)| check == "qualification"
                && *status == blocked("QUALIFICATION_BLOCKERS")
                && detail.contains("dependency_vulnerabilities")),
        "{found:?}"
    );
}

#[test]
fn a_missing_or_stale_qualification_is_required_not_run() {
    let fx = project("pass", "pass");
    assert_eq!(
        findings(&fx),
        vec![(
            "qualification".to_string(),
            blocked("QUALIFICATION_REQUIRED"),
            "no acceptance qualification of F-0001 exists".to_string()
        )]
    );
    qualify(&fx);
    fx.write("backend/src/lib.rs", "pub fn route() { let _ = 1; }\n");
    let found = findings(&fx);
    assert!(
        found
            .iter()
            .any(|(check, status, detail)| check == "qualification"
                && *status == blocked("QUALIFICATION_REQUIRED")
                && detail.contains("stale")),
        "{found:?}"
    );
}

#[test]
fn a_project_without_qualification_is_not_applicable() {
    let fx = project("pass", "pass");
    fs::remove_file(fx.root.join("scripts/codeos-qualify")).unwrap();
    let found = findings(&fx);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(matches!(found[0].1, Status::NotApplicable(_)), "{found:?}");
}
