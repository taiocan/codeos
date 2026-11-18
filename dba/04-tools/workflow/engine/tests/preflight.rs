//! VALIDITY WO-03: deterministic preflight reports the defect classes JamFree met late in
//! 2026-11-13..17 (a YAML colon in semantic_change, an unsettled privacy prerequisite, an open
//! advisory) before any approval, and records nothing.

mod fai_fixture;

use fai_fixture::*;
use std::fs;

/// A feature with a project qualification whose C-1 and dependency-vulnerability results are given.
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
printf '{{"schema":1,"project":"fixture","implementation_sha256":"%s","charter_sha256":"%s","configuration_sha256":"%s","tools":{{"fixture":"1"}},"constraints":[{{"id":"C-1","phase":"acceptance","status":"{c1}","evidence":[],"measurements":{{}}}}],"baseline":[{{"id":"static_analysis","status":"pass","tool":"fixture","version":"1","summary":"ok"}},{{"id":"dependency_vulnerabilities","status":"{vulnerabilities}","tool":"fixture","version":"1","summary":"advisories"}},{{"id":"dependency_licences","status":"pass","tool":"fixture","version":"1","summary":"ok"}},{{"id":"committed_secrets","status":"pass","tool":"fixture","version":"1","summary":"ok"}}]}}' "$CODEOS_IMPLEMENTATION_SHA256" "$CODEOS_CHARTER_SHA256" "$CODEOS_CONFIGURATION_SHA256" > "$CODEOS_QUALIFICATION_OUTPUT"
"#
        ),
    );
    std::process::Command::new("chmod")
        .args(["+x", "scripts/codeos-qualify"])
        .current_dir(&fx.root)
        .status()
        .unwrap();
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "fixture"]);
    fx
}

fn failed(fx: &Fixture) -> Vec<(String, String)> {
    codeos_workflow::preflight::run(&fx.project(), "F-0001")
        .into_iter()
        .filter(|f| !f.ok)
        .map(|f| (f.check.to_string(), f.detail))
        .collect()
}

fn workflow_files(fx: &Fixture) -> Vec<(String, usize)> {
    let dir = fx.root.join(".codeos/06-workflow");
    let mut files: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| {
            let n = fs::read(e.path()).unwrap().len();
            (e.file_name().to_string_lossy().into_owned(), n)
        })
        .collect();
    files.sort();
    files
}

#[test]
fn a_clean_feature_passes_and_preflight_records_nothing() {
    let fx = project("pass", "pass");
    let before = workflow_files(&fx);
    assert_eq!(failed(&fx), vec![]);
    assert_eq!(workflow_files(&fx), before, "preflight records nothing");
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
fn a_yaml_colon_in_semantic_change_fails_the_specification() {
    let fx = project("pass", "pass");
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
    let failed = failed(&fx);
    assert!(
        failed
            .iter()
            .any(|(check, detail)| check == "specification" && detail.contains("does not parse")),
        "{failed:?}"
    );
}

#[test]
fn an_unsettled_privacy_prerequisite_fails_the_qualification() {
    let fx = project("unresolved", "pass");
    let failed = failed(&fx);
    assert!(
        failed
            .iter()
            .any(|(check, detail)| check == "qualification" && detail.contains("C-1")),
        "{failed:?}"
    );
}

#[test]
fn an_open_advisory_fails_the_qualification() {
    let fx = project("pass", "fail");
    let failed = failed(&fx);
    assert!(
        failed.iter().any(|(check, detail)| check == "qualification"
            && detail.contains("dependency_vulnerabilities")),
        "{failed:?}"
    );
}
