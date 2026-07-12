//! generate-release-evidence command tests.
//!
//! Tests for the generate-release-evidence command.

mod common;
use common::{setup_temp_git_repo, run_in_dir, binary};
use std::process::Command;

const RELEASE_EVIDENCE_BANNER_LINE_1: &str =
    "> [INFERRED] fields were populated automatically from git and the feature registry (if";
const RELEASE_EVIDENCE_BANNER_LINE_4: &str =
    "> — Release decision requires explicit human judgment and is never inferred.";

fn checkout_branch(repo_path: &std::path::Path, name: &str) {
    Command::new("git")
        .args(["checkout", "-b", name])
        .current_dir(repo_path)
        .output()
        .expect("git checkout -b");
}

#[test]
fn smoke_release_evidence_feature_always_inferred() {
    // AC-1: Feature: is always [INFERRED] from --feature, regardless of --registry.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/upg-9401");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9401"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Feature: UPG-9401 [INFERRED]"), "stdout: {}", stdout);
}

#[test]
fn smoke_release_evidence_branch_always_inferred() {
    // AC-2: Branch: is always [INFERRED] from git rev-parse --abbrev-ref HEAD.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/named-branch-test");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9402"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(
        stdout.contains("Branch: feature/named-branch-test [INFERRED]"),
        "stdout: {}", stdout
    );
}

#[test]
fn smoke_release_evidence_output_structure() {
    // AC-3: report body, after the banner, is exactly the 12-field/sub-item list in order.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/structure-test");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9403"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("# Release Evidence Package"), "stdout: {}", stdout);

    let order = [
        "Feature:",
        "Branch:",
        "PR:",
        "Approved artifacts:",
        "- Intent:",
        "- Contract:",
        "- Event schema:",
        "Stage reports:",
        "Reviewer briefs:",
        "Reconciliation result:",
        "Replay result:",
        "Verification-only report:",
        "Readiness checklist:",
        "Known limitations:",
        "Release decision:",
    ];
    let mut last_pos = 0usize;
    for field in order {
        let pos = stdout[last_pos..].find(field)
            .unwrap_or_else(|| panic!("field '{}' not found after position {}; stdout: {}", field, last_pos, stdout));
        last_pos += pos + field.len();
    }
}

#[test]
fn smoke_release_evidence_registry_enrichment_per_field_independent() {
    // AC-4: registry-derived fields are [INFERRED] only when that specific field is
    // non-null for the matched feature — evaluated independently, not all-or-nothing.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/partial-registry-test");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9404
    slug: partial-feature
    status: active
    current_stage: 4
    pr: "https://example.test/pr/1"
    intent: intents/UPG-9404.md
    contract: null
    event_schema: null
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9404", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("PR: https://example.test/pr/1 [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("- Intent: intents/UPG-9404.md [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("- Contract: [FILL]"), "stdout: {}", stdout);
    assert!(stdout.contains("- Event schema: [FILL]"), "stdout: {}", stdout);
}

#[test]
fn smoke_release_evidence_always_fill_fields() {
    // AC-5: the 8 always-[FILL] fields stay [FILL] even with a fully populated registry.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/always-fill-test");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9405
    slug: full-feature
    status: active
    current_stage: 6
    pr: "https://example.test/pr/2"
    intent: intents/UPG-9405.md
    contract: contracts/UPG-9405_contract.md
    event_schema: events/UPG-9405_schema.md
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9405", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    for field in [
        "Stage reports: [FILL]",
        "Reviewer briefs: [FILL]",
        "Reconciliation result: [FILL]",
        "Replay result: [FILL]",
        "Verification-only report: [FILL]",
        "Readiness checklist: [FILL]",
        "Known limitations: [FILL]",
        "Release decision: [FILL]",
    ] {
        assert!(stdout.contains(field), "expected '{}' in stdout: {}", field, stdout);
    }
}

#[test]
fn smoke_release_evidence_preamble_present() {
    // AC-6: the preamble banner is present verbatim (first and last line checked).
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/banner-test");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9406"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains(RELEASE_EVIDENCE_BANNER_LINE_1), "stdout: {}", stdout);
    assert!(stdout.contains(RELEASE_EVIDENCE_BANNER_LINE_4), "stdout: {}", stdout);
    let banner_pos = stdout.find(RELEASE_EVIDENCE_BANNER_LINE_1).unwrap();
    let heading_pos = stdout.find("# Release Evidence Package").unwrap();
    assert!(banner_pos < heading_pos, "banner must precede the report heading");
}

#[test]
fn smoke_release_evidence_unreadable_registry_degrades_gracefully() {
    // AC-7: a --registry path that doesn't exist degrades gracefully: full report still
    // emitted, warning on stderr, exit 0.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/missing-registry-test");

    let missing = p.join("does-not-exist.yaml");
    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9407", "--registry", missing.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("# Release Evidence Package"), "stdout: {}", stdout);
    assert!(stdout.contains("PR: [FILL]"), "stdout: {}", stdout);
    assert!(stderr.contains("cannot read registry file"), "stderr: {}", stderr);
    assert!(stderr.contains("[FILL]"), "stderr: {}", stderr);
}

#[test]
fn smoke_release_evidence_malformed_registry_degrades_gracefully() {
    // AC-7: a --registry path that exists but fails to parse degrades the same way.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/malformed-registry-test");

    let malformed = p.join("malformed.yaml");
    std::fs::write(&malformed, "features: [this is not a list of maps\n").expect("write fixture");
    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9408", "--registry", malformed.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("# Release Evidence Package"), "stdout: {}", stdout);
    assert!(stdout.contains("PR: [FILL]"), "stdout: {}", stdout);
    assert!(stderr.contains("cannot parse registry file"), "stderr: {}", stderr);
}

#[test]
fn smoke_release_evidence_feature_not_found_degrades_gracefully() {
    // AC-8: a valid registry with no matching feature_id degrades gracefully with a
    // distinct warning naming both the feature id and the registry path.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/not-found-test");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-0000
    slug: unrelated-feature
    status: active
    current_stage: 1
    blockers: []
"#,
    )
    .expect("write fixture");
    let registry_str = registry.to_str().unwrap();

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9409", "--registry", registry_str],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("# Release Evidence Package"), "stdout: {}", stdout);
    assert!(stdout.contains("PR: [FILL]"), "stdout: {}", stdout);
    assert!(stderr.contains("UPG-9409"), "stderr: {}", stderr);
    assert!(stderr.contains(registry_str), "stderr: {}", stderr);
}

#[test]
fn smoke_release_evidence_no_registry_is_silent() {
    // AC-9: omitting --registry entirely is normal usage — no stderr warning at all.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/no-registry-test");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9410"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("PR: [FILL]"), "stdout: {}", stdout);
    assert_eq!(stderr, "", "no --registry given should produce no stderr output at all");
}

#[test]
fn smoke_release_evidence_feature_required() {
    // AC-10: omitting --feature is a clap usage error, exit 1, nothing on stdout.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let (code, stdout, _stderr) = run_in_dir(p, &["generate-release-evidence"]);
    assert_eq!(code, 1, "missing --feature should exit 1");
    assert_eq!(stdout, "", "usage error should produce no stdout");
}

#[test]
fn smoke_release_evidence_stdout_only_no_registry() {
    // AC-11: with no --registry, stdout carries the full report and stderr is empty.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/stdout-only-test");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9411"]);
    assert_eq!(code, 0);
    assert_eq!(stderr, "");
    assert!(!stdout.is_empty(), "stdout must not be empty");
}

#[test]
fn smoke_release_evidence_exit_zero_across_registry_states() {
    // AC-12: exit 0 across no-registry, bad-registry, feature-not-found, and
    // fully-valid-registry states, all inside a git repo.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/exit-code-matrix-test");

    let (code_none, _, stderr_none) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9412"]);
    assert_eq!(code_none, 0, "stderr: {}", stderr_none);

    let bad = p.join("missing.yaml");
    let (code_bad, _, stderr_bad) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9412", "--registry", bad.to_str().unwrap()],
    );
    assert_eq!(code_bad, 0, "stderr: {}", stderr_bad);

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9412
    slug: exit-code-feature
    status: active
    current_stage: 5
    pr: "https://example.test/pr/3"
    intent: intents/UPG-9412.md
    contract: contracts/UPG-9412_contract.md
    event_schema: events/UPG-9412_schema.md
    blockers: []
"#,
    )
    .expect("write fixture");
    let (code_full, _, stderr_full) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9412", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code_full, 0, "stderr: {}", stderr_full);

    let (code_not_found, _, stderr_not_found) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-0000-not-in-registry", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code_not_found, 0, "stderr: {}", stderr_not_found);
}

#[test]
fn smoke_release_evidence_no_provider_config_required() {
    // AC-13: dispatches before config::resolve() — no provider config needed.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/no-config-test");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-release-evidence", "--feature", "UPG-9413"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("# Release Evidence Package"), "stdout: {}", stdout);
}

#[test]
fn smoke_release_evidence_deterministic_output() {
    // AC-14: identical inputs produce byte-for-byte identical stdout across two runs.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/determinism-test");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9414
    slug: determinism-feature
    status: active
    current_stage: 3
    pr: "https://example.test/pr/4"
    intent: intents/UPG-9414.md
    contract: null
    event_schema: null
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code1, stdout1, stderr1) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9414", "--registry", registry.to_str().unwrap()],
    );
    let (code2, stdout2, stderr2) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9414", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code1, 0, "stderr: {}", stderr1);
    assert_eq!(code2, 0, "stderr: {}", stderr2);
    assert_eq!(stdout1, stdout2, "output must be deterministic for identical inputs");
}

#[test]
fn smoke_release_evidence_architectural_refinements_never_treated_as_feature() {
    // AC-15: an architectural_refinements entry with a colliding refine_id is never
    // matched as if it were the searched-for feature.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/refinement-collision-test");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9415
    slug: real-feature
    status: active
    current_stage: 2
    pr: "https://example.test/pr/5"
    intent: intents/UPG-9415.md
    contract: null
    event_schema: null
    blockers: []

architectural_refinements:
  - refine_id: UPG-9416-sneaky
    description: "Looks like a feature but isn't"
    status: active
    artifact: refinements/arch/sneaky.md
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9416-sneaky", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    // The refinement is not a feature entry, so it must not match — fields fall back to [FILL].
    assert!(stdout.contains("PR: [FILL]"), "stdout: {}", stdout);
    assert!(stderr.contains("UPG-9416-sneaky"), "stderr: {}", stderr);
}

#[test]
fn smoke_release_evidence_v2_missing_schema_version_warning() {
    // AC-13: generate-release-evidence warns specifically about missing schema_version
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/test-v2");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"features:
  - feature_id: UPG-9508
    slug: legacy-feature
    status: active
    pr: "https://example.test/pr/1"
    intent: intents/UPG-9508.md
    contract: contracts/UPG-9508.md
    event_schema: null
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9508", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "should exit 0 (graceful degradation)");
    assert!(stderr.contains("schema_version: 2"), "stderr: {}", stderr);
    assert!(stderr.contains("missing"), "stderr: {}", stderr);
    assert!(stderr.contains("registry-v2-migration.md"), "stderr: {}", stderr);
    // Fields should fall back to [FILL]
    assert!(stdout.contains("[FILL]"), "stdout: {}", stdout);
}

#[test]
fn smoke_release_evidence_v2_field_set_unchanged() {
    // AC-14: generate-release-evidence's FeatureEntry still reads only pr/intent/contract/event_schema
    // (status/current_stage/blockers/notes were never read and still aren't)
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    checkout_branch(p, "feature/test-v2-fields");

    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9509
    slug: v2-feature
    status: hypothesized
    current_stage: 0
    pr: "https://example.test/pr/99"
    intent: intents/UPG-9509.md
    contract: null
    event_schema: events/UPG-9509.json
    blockers: ["blocker-1", "blocker-2"]
    notes: "some notes"
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-release-evidence", "--feature", "UPG-9509", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    // Only pr/intent/contract/event_schema should be read; status/current_stage/blockers/notes ignored
    assert!(stdout.contains("https://example.test/pr/99"), "stdout: {}", stdout);
    assert!(stdout.contains("intents/UPG-9509.md"), "stdout: {}", stdout);
    assert!(stdout.contains("events/UPG-9509.json"), "stdout: {}", stdout);
}

