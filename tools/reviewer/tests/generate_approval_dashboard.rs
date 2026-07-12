//! generate-approval-dashboard command tests.
//!
//! Tests for the generate-approval-dashboard subcommand (registry-based approval dashboard generation).

mod common;
use common::{setup_temp_git_repo, run_in_dir, binary};
use std::process::Command;

const DASHBOARD_BANNER_LINE_1: &str =
    "> [INFERRED] fields were populated automatically from the feature registry — verify before";
const DASHBOARD_BANNER_LINE_2: &str =
    "> submitting. [FILL] fields require human or model authorship. This dashboard is a navigation";
const DASHBOARD_BANNER_LINE_3: &str =
    "> aid, not a decision record — the registry and change records remain authoritative.";

#[test]
fn smoke_dashboard_full_vs_minimal_schema_identical_output() {
    // AC-1: an entry with every registry field populated and an entry with only the fields
    // this tool reads must produce byte-identical dashboard blocks for the same values.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let full = p.join("full.yaml");
    std::fs::write(
        &full,
        r#"schema_version: 2
features:
  - feature_id: UPG-1001
    slug: sample-feature
    description: "A feature with every field populated"
    type: F
    status: active
    branch: feature/sample
    current_stage: 4
    intent: intents/UPG-1001.md
    contract: contracts/UPG-1001_contract.md
    event_schema: events/UPG-1001_schema.md
    pr: null
    last_commit: abc123
    reconciliation_status: pending
    replay_status: na
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let minimal = p.join("minimal.yaml");
    std::fs::write(
        &minimal,
        r#"schema_version: 2
features:
  - feature_id: UPG-1001
    slug: sample-feature
    status: active
    current_stage: 4
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code1, stdout1, stderr1) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", full.to_str().unwrap()]);
    let (code2, stdout2, stderr2) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", minimal.to_str().unwrap()]);
    assert_eq!(code1, 0, "stderr: {}", stderr1);
    assert_eq!(code2, 0, "stderr: {}", stderr2);
    assert_eq!(stdout1, stdout2, "full-schema and minimal-schema entries must produce identical output");
}

#[test]
fn smoke_dashboard_only_active_features_in_registry_order() {
    // AC-2: only status: active entries appear, in original registry order.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-2001
    slug: first-active
    status: active
    current_stage: 1
    blockers: []
    notes: ""
  - feature_id: UPG-2002
    slug: suspended-one
    status: suspended
    current_stage: 2
    blockers: []
    notes: ""
  - feature_id: UPG-2003
    slug: second-active
    status: active
    current_stage: 3
    blockers: []
    notes: ""
  - feature_id: UPG-2004
    slug: complete-one
    status: complete
    current_stage: 9
    blockers: []
    notes: ""
  - feature_id: UPG-2005
    slug: blocked-one
    status: blocked
    current_stage: 4
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("UPG-2001: first-active"), "stdout: {}", stdout);
    assert!(stdout.contains("UPG-2003: second-active"), "stdout: {}", stdout);
    assert!(!stdout.contains("suspended-one"), "stdout: {}", stdout);
    assert!(!stdout.contains("complete-one"), "stdout: {}", stdout);
    assert!(!stdout.contains("blocked-one"), "stdout: {}", stdout);

    let pos_first = stdout.find("UPG-2001: first-active").expect("first-active present");
    let pos_second = stdout.find("UPG-2003: second-active").expect("second-active present");
    assert!(pos_first < pos_second, "active features must appear in registry order");
}

#[test]
fn smoke_dashboard_output_structure() {
    // AC-3: one heading, one subsection per active feature, six fields each, in order.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-3001
    slug: alpha
    status: active
    current_stage: 1
    blockers: []
  - feature_id: UPG-3002
    slug: beta
    status: active
    current_stage: 2
    blockers: []
  - feature_id: UPG-3003
    slug: gamma
    status: active
    current_stage: 3
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert_eq!(stdout.matches("# Approval Dashboard").count(), 1, "stdout: {}", stdout);

    let c1 = stdout.find("## UPG-3001: alpha").expect("alpha present");
    let c2 = stdout.find("## UPG-3002: beta").expect("beta present");
    let c3 = stdout.find("## UPG-3003: gamma").expect("gamma present");
    assert!(c1 < c2 && c2 < c3, "subsections must appear in order; stdout: {}", stdout);

    for field in [
        "Active features:",
        "Current stage:",
        "Reviewer recommendation: [FILL]",
        "Open blockers:",
        "Next human decision: [FILL]",
        "Risk: [FILL]",
    ] {
        assert_eq!(
            stdout.matches(field).count(),
            3,
            "field '{}' must appear once per active feature; stdout: {}",
            field,
            stdout
        );
    }
}

#[test]
fn smoke_dashboard_inferred_edge_cases() {
    // AC-4: (a) populated stage + blockers; (b) null stage + empty blockers.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-4001
    slug: with-stage-and-blockers
    status: active
    current_stage: 4
    blockers:
      - waiting on review
      - blocked by dependency
  - feature_id: UPG-4002
    slug: not-started
    status: active
    current_stage: null
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Current stage: 4 [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("Open blockers: [INFERRED]\nwaiting on review\nblocked by dependency"), "stdout: {}", stdout);
    assert!(stdout.contains("Current stage: not started [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("Open blockers: (none) [INFERRED]"), "stdout: {}", stdout);
}

#[test]
fn smoke_dashboard_fill_fields_always_present() {
    // AC-5: Reviewer recommendation / Next human decision / Risk are always [FILL].
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-5001
    slug: sample
    status: active
    current_stage: 5
    blockers:
      - some blocker
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("Reviewer recommendation: [FILL]"), "stdout: {}", stdout);
    assert!(stdout.contains("Next human decision: [FILL]"), "stdout: {}", stdout);
    assert!(stdout.contains("Risk: [FILL]"), "stdout: {}", stdout);
}

#[test]
fn smoke_dashboard_preamble_present() {
    // AC-6: banner is the first three non-blank lines, verbatim.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        "schema_version: 2\nfeatures:\n  - feature_id: UPG-6001\n    slug: sample\n    status: active\n    current_stage: 1\n    blockers: []\n    notes: \"\"\n",
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);

    let mut lines = stdout.lines();
    assert_eq!(lines.next().unwrap_or(""), DASHBOARD_BANNER_LINE_1);
    assert_eq!(lines.next().unwrap_or(""), DASHBOARD_BANNER_LINE_2);
    assert_eq!(lines.next().unwrap_or(""), DASHBOARD_BANNER_LINE_3);
}

#[test]
fn smoke_dashboard_no_active_features_only_non_active() {
    // AC-7(a): registry parses but has zero active entries -> empty stdout, stderr note, exit 0.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        "schema_version: 2\nfeatures:\n  - feature_id: UPG-7001\n    slug: done\n    status: complete\n    current_stage: 9\n    blockers: []\n    notes: \"\"\n",
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("no active or hypothesized features found"), "stderr: {}", stderr);
    assert!(stderr.contains(registry.to_str().unwrap()), "stderr must name the path; got: {}", stderr);
}

#[test]
fn smoke_dashboard_no_active_features_empty_list() {
    // AC-7(b): empty features: [] list -> same empty/exit-0 behavior.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(&registry, "schema_version: 2\nfeatures: []\n").expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("no active or hypothesized features found"), "stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_missing_registry_file() {
    // AC-8: unreadable/missing --registry -> exit 1 (EXIT_USAGE), stderr names the path.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", "does-not-exist.yaml"]);
    assert_eq!(code, 1, "missing registry file must exit 1; stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("does-not-exist.yaml"), "stderr must name the path; got: {}", stderr);
}

#[test]
fn smoke_dashboard_malformed_yaml() {
    // AC-9(a): malformed YAML -> exit 1, distinct stderr message from AC-8.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("malformed.yaml");
    std::fs::write(&registry, "features: [\"unterminated\n").expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 1, "malformed YAML must exit 1; stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("cannot parse registry file"), "stderr: {}", stderr);
    assert!(!stderr.contains("cannot read registry file"), "AC-8/AC-9 messages must be distinct; stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_wrong_shape_yaml() {
    // AC-9(b): features: is not a list -> exit 1, same distinct parse-error message.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("wrong-shape.yaml");
    std::fs::write(&registry, "schema_version: 2\nfeatures: \"not a list\"\n").expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 1, "wrong-shape YAML must exit 1; stderr: {}", stderr);
    assert!(stdout.is_empty(), "stdout must be empty; got: {}", stdout);
    assert!(stderr.contains("cannot parse registry file"), "stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_registry_required() {
    // AC-10: omitting --registry is a clap usage error, exit 1.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();

    let (code, _, stderr) = run_in_dir(p, &["generate-approval-dashboard"]);
    assert_eq!(code, 1, "missing --registry must exit 1; stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_stdout_only() {
    // AC-11: stderr empty on success; stdout exactly empty on the AC-7/8/9 error/empty paths.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        "schema_version: 2\nfeatures:\n  - feature_id: UPG-8001\n    slug: sample\n    status: active\n    current_stage: 1\n    blockers: []\n    notes: \"\"\n",
    )
    .expect("write fixture");

    let (code, _, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.is_empty(), "successful run must have empty stderr; got: {}", stderr);

    let empty_registry = p.join("empty.yaml");
    std::fs::write(&empty_registry, "schema_version: 2\nfeatures: []\n").expect("write fixture");
    let (_, stdout, _) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", empty_registry.to_str().unwrap()]);
    assert!(stdout.is_empty(), "AC-7 case must have empty stdout; got: {}", stdout);

    let (_, stdout, _) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", "does-not-exist.yaml"]);
    assert!(stdout.is_empty(), "AC-8 case must have empty stdout; got: {}", stdout);

    let malformed = p.join("malformed.yaml");
    std::fs::write(&malformed, "features: [\"unterminated\n").expect("write fixture");
    let (_, stdout, _) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", malformed.to_str().unwrap()]);
    assert!(stdout.is_empty(), "AC-9 case must have empty stdout; got: {}", stdout);
}

#[test]
fn smoke_dashboard_exit_zero_on_success() {
    // AC-12: any invocation finding >= 1 active feature exits 0.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        "schema_version: 2\nfeatures:\n  - feature_id: UPG-9001\n    slug: sample\n    status: active\n    current_stage: 1\n    blockers: []\n    notes: \"\"\n",
    )
    .expect("write fixture");

    let (code, _, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_no_provider_config_required() {
    // AC-13: dispatches before config::resolve(); no provider config needed.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        "schema_version: 2\nfeatures:\n  - feature_id: UPG-9101\n    slug: sample\n    status: active\n    current_stage: 1\n    blockers: []\n    notes: \"\"\n",
    )
    .expect("write fixture");

    let (code, _, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "must succeed without provider config; stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_deterministic_output() {
    // AC-14: identical registry produces byte-for-byte identical stdout across two invocations.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9201
    slug: alpha
    status: active
    current_stage: 2
    blockers:
      - a blocker
  - feature_id: UPG-9202
    slug: beta
    status: active
    current_stage: null
    blockers: []
"#,
    )
    .expect("write fixture");

    let args = ["generate-approval-dashboard", "--registry", registry.to_str().unwrap()];
    let (code1, stdout1, _) = run_in_dir(p, &args);
    let (code2, stdout2, _) = run_in_dir(p, &args);
    assert_eq!(code1, 0);
    assert_eq!(code2, 0);
    assert_eq!(stdout1, stdout2, "output must be deterministic for identical inputs");
}

#[test]
fn smoke_dashboard_architectural_refinements_never_treated_as_feature() {
    // AC-15: architectural_refinements: is never mistaken for a feature entry.
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9301
    slug: real-feature
    status: active
    current_stage: 3
    blockers: []

architectural_refinements:
  - refine_id: sneaky_refine
    description: "Looks like a feature but isn't"
    status: active
    artifact: refinements/arch/sneaky_refine.md
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) =
        run_in_dir(p, &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert_eq!(stdout.matches("## ").count(), 1, "only the one real feature must appear; stdout: {}", stdout);
    assert!(stdout.contains("UPG-9301: real-feature"), "stdout: {}", stdout);
    assert!(!stdout.contains("sneaky_refine"), "stdout: {}", stdout);
}

#[test]
fn smoke_dashboard_v2_missing_schema_version_diagnostic() {
    // AC-8: Missing schema_version triggers specific migration diagnostic
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"features:
  - feature_id: UPG-9500
    slug: legacy-feature
    status: active
    current_stage: 1
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, _stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_ne!(code, 0, "should exit non-zero for missing schema_version");
    assert!(stderr.contains("schema_version: 2"), "stderr: {}", stderr);
    assert!(stderr.contains("missing") || stderr.contains("found: 0"), "stderr: {}", stderr);
    assert!(stderr.contains("registry-v2-migration.md"), "stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_v2_wrong_schema_version_diagnostic() {
    // AC-8: Non-2 schema_version triggers specific diagnostic
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 1
features:
  - feature_id: UPG-9501
    slug: v1-feature
    status: active
    current_stage: 1
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, _stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_ne!(code, 0, "should exit non-zero for wrong schema_version");
    assert!(stderr.contains("schema_version: 2"), "stderr: {}", stderr);
    assert!(stderr.contains("found: 1"), "stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_v2_non_numeric_schema_version_diagnostic() {
    // AC-8 blocker fix: Non-numeric schema_version should be reported as such, not as "missing"
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: "1"
features:
  - feature_id: UPG-9502
    slug: string-version-feature
    status: active
    current_stage: 1
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, _stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_ne!(code, 0, "should exit non-zero for non-numeric schema_version");
    assert!(stderr.contains("schema_version: 2"), "stderr: {}", stderr);
    // Should show the actual value (a string), not "missing"
    assert!(!stderr.contains("found: missing"), "should not report as missing: {}", stderr);
    assert!(stderr.contains("not a number") || stderr.contains("\"1\""), "stderr: {}", stderr);
}

#[test]
fn smoke_dashboard_v2_schema_version_probe_wins_over_missing_field() {
    // AC-9: schema_version pre-probe prevents generic "missing field `slug`" error
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"features:
  - feature_id: UPG-9502
    status: active
    current_stage: 1
    blockers: []
"#,
    )
    .expect("write fixture");

    let (code, _stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_ne!(code, 0, "should exit non-zero");
    // Should get the schema_version diagnostic, not a generic "missing field `slug`" error
    assert!(stderr.contains("schema_version"), "stderr: {}", stderr);
    assert!(!stderr.contains("missing field"), "stderr should not contain generic serde error: {}", stderr);
}

#[test]
fn smoke_dashboard_v2_invalid_status_value_diagnostic() {
    // AC-10: Invalid status value produces specific diagnostic
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9503
    slug: invalid-status-feature
    status: stage1
    current_stage: 1
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, _stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_ne!(code, 0, "should exit non-zero for invalid status");
    assert!(stderr.contains("UPG-9503"), "stderr: {}", stderr);
    assert!(stderr.contains("stage1") || stderr.contains("invalid status"), "stderr: {}", stderr);
    assert!(stderr.contains("hypothesized") && stderr.contains("active"), "stderr should list valid values: {}", stderr);
}

#[test]
fn smoke_dashboard_v2_hypothesized_and_active_both_appear() {
    // AC-11: Both active and hypothesized features appear, hypothesized visually flagged
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9504
    slug: active-feature
    status: active
    current_stage: 3
    blockers: []
    notes: ""
  - feature_id: UPG-9505
    slug: hypothesized-feature
    status: hypothesized
    current_stage: 0
    blockers: []
    notes: ""
  - feature_id: UPG-9506
    slug: suspended-feature
    status: suspended
    current_stage: 2
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("UPG-9504: active-feature"), "active feature should appear: {}", stdout);
    assert!(stdout.contains("UPG-9505: hypothesized-feature"), "hypothesized feature should appear: {}", stdout);
    assert!(!stdout.contains("UPG-9506"), "suspended feature should not appear: {}", stdout);

    // Hypothesized feature should have visual flag
    let hyp_section = stdout.split("UPG-9505").nth(1).expect("hypothesized section present");
    assert!(
        hyp_section.contains("HYPOTHESIZED") || hyp_section.contains("Stage 1 review"),
        "hypothesized feature should be visually flagged: {}",
        hyp_section
    );

    // Active feature should NOT have the flag
    let active_section = stdout.split("UPG-9504").nth(1).and_then(|s| s.split("UPG-9505").next()).expect("active section");
    assert!(
        !active_section.contains("HYPOTHESIZED"),
        "active feature should not have hypothesized flag: {}",
        active_section
    );
}

#[test]
fn smoke_dashboard_v2_all_active_registry_unchanged_behavior() {
    // AC-12: v2 registry with only active features produces expected output (no regression)
    let (dir, _sha) = setup_temp_git_repo();
    let p = dir.path();
    let registry = p.join("registry.yaml");
    std::fs::write(
        &registry,
        r#"schema_version: 2
features:
  - feature_id: UPG-9507
    slug: normal-active
    status: active
    current_stage: 2
    blockers: []
    notes: ""
"#,
    )
    .expect("write fixture");

    let (code, stdout, stderr) = run_in_dir(
        p,
        &["generate-approval-dashboard", "--registry", registry.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
    assert!(stdout.contains("UPG-9507: normal-active"), "stdout: {}", stdout);
    assert!(stdout.contains("Current stage: 2 [INFERRED]"), "stdout: {}", stdout);
    assert!(stdout.contains("Open blockers: (none) [INFERRED]"), "stdout: {}", stdout);
    assert!(!stdout.contains("HYPOTHESIZED"), "should not have hypothesized flag: {}", stdout);
}

