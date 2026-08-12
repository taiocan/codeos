mod common;

use common::{repo_root, run_in_dir, setup_temp_git_repo};
use std::fs;
use std::path::Path;

fn write_scope(root: &Path, name: &str, metadata: &str) {
    let directory = root.join("architecture/scopes");
    fs::create_dir_all(&directory).expect("create scope directory");
    fs::write(
        directory.join(format!("{name}.md")),
        format!("---\n{metadata}\n---\n\n# Architecture Scope: {name}\n"),
    )
    .expect("write scope");
}

#[test]
fn absent_directory_and_unmatched_feature_are_valid() {
    let (repo, _) = setup_temp_git_repo();
    let (code, stdout, stderr) = run_in_dir(
        repo.path(),
        &["inspect-architecture-scopes", "--feature", "F-0001"],
    );

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("architecture_scopes: []"));
    assert!(stdout.contains("resolution: none"));
}

#[test]
fn shipped_scope_template_matches_the_inspector_contract() {
    let (repo, _) = setup_temp_git_repo();
    let directory = repo.path().join("architecture/scopes");
    fs::create_dir_all(&directory).expect("create scope directory");
    fs::copy(
        repo_root().join("templates/architecture-scope.md"),
        directory.join("example.md"),
    )
    .expect("copy template");

    let (code, stdout, stderr) = run_in_dir(repo.path(), &["inspect-architecture-scopes"]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("scope: example"));
    assert!(stdout.contains("state: draft"));
}

#[test]
fn reports_draft_and_approved_scopes_deterministically() {
    let (repo, _) = setup_temp_git_repo();
    write_scope(repo.path(), "zeta", "features: [F-0002]\napproval: null");
    write_scope(
        repo.path(),
        "alpha",
        "features: [F-0001]\napproval:\n  by: Ada\n  at: '2026-08-12'",
    );

    let (code, stdout, stderr) = run_in_dir(
        repo.path(),
        &["inspect-architecture-scopes", "--feature", "F-0001"],
    );

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.find("scope: alpha") < stdout.find("scope: zeta"));
    assert!(stdout.contains("state: approved"));
    assert!(stdout.contains("state: draft"));
    assert!(stdout.contains("resolution: approved"));
    assert!(stdout.contains("path: architecture/scopes/alpha.md"));
}

#[test]
fn rejects_malformed_or_extended_governance_metadata() {
    for (name, metadata) in [
        ("missing-approval", "features: [F-0001]"),
        (
            "partial-approval",
            "features: [F-0001]\napproval:\n  by: Ada",
        ),
        (
            "empty-approval",
            "features: [F-0001]\napproval:\n  by: ''\n  at: ''",
        ),
        (
            "unknown-field",
            "features: [F-0001]\napproval: null\nstatus: approved",
        ),
        ("empty-features", "features: []\napproval: null"),
        (
            "duplicate-feature",
            "features: [F-0001, F-0001]\napproval: null",
        ),
    ] {
        let (repo, _) = setup_temp_git_repo();
        write_scope(repo.path(), name, metadata);
        let (code, _, stderr) = run_in_dir(repo.path(), &["inspect-architecture-scopes"]);
        assert_eq!(code, 2, "case {name} unexpectedly passed");
        assert!(stderr.contains("error:"), "case {name}: {stderr}");
    }
}

#[test]
fn rejects_duplicate_membership_across_scopes() {
    let (repo, _) = setup_temp_git_repo();
    write_scope(repo.path(), "alpha", "features: [F-0001]\napproval: null");
    write_scope(repo.path(), "beta", "features: [F-0001]\napproval: null");

    let (code, _, stderr) = run_in_dir(repo.path(), &["inspect-architecture-scopes"]);

    assert_eq!(code, 2);
    assert!(stderr.contains("belongs to both architecture scopes"));
}

#[test]
fn rejects_missing_front_matter() {
    let (repo, _) = setup_temp_git_repo();
    let directory = repo.path().join("architecture/scopes");
    fs::create_dir_all(&directory).expect("create scope directory");
    fs::write(directory.join("broken.md"), "# no front matter\n").expect("write scope");

    let (code, _, stderr) = run_in_dir(repo.path(), &["inspect-architecture-scopes"]);

    assert_eq!(code, 2);
    assert!(stderr.contains("missing opening YAML front-matter delimiter"));
}
