use serde::Deserialize;
use std::path::Path;

const PREAMBLE: &str = "\
> [INFERRED] fields were populated automatically from git and the feature registry (if\n\
> --registry was given) — verify before submitting. [FILL] fields require human or model\n\
> authorship. This package aggregates existing evidence; it is not itself a decision record\n\
> — Release decision requires explicit human judgment and is never inferred.";

#[derive(Debug, Deserialize)]
struct Registry {
    #[serde(default)]
    features: Vec<FeatureEntry>,
}

#[derive(Debug, Deserialize)]
struct FeatureEntry {
    feature_id: String,
    #[serde(default)]
    pr: Option<String>,
    #[serde(default)]
    intent: Option<String>,
    #[serde(default)]
    contract: Option<String>,
    #[serde(default)]
    event_schema: Option<String>,
}

pub struct GenerateReleaseEvidenceArgs<'a> {
    pub feature: &'a str,
    pub registry: Option<&'a str>,
}

pub fn run(args: GenerateReleaseEvidenceArgs, repo_root: &Path) -> i32 {
    let branch = git_branch(repo_root);
    let (pr, intent, contract, event_schema) = resolve_registry_fields(&args);

    let report = format!(
        "# Release Evidence Package\n\n\
Feature: {feature} [INFERRED]\n\n\
Branch: {branch}\n\n\
PR: {pr}\n\n\
Approved artifacts:\n\
- Intent: {intent}\n\
- Contract: {contract}\n\
- Event schema: {event_schema}\n\n\
Stage reports: [FILL]\n\n\
Reviewer briefs: [FILL]\n\n\
Reconciliation result: [FILL]\n\n\
Replay result: [FILL]\n\n\
Verification-only report: [FILL]\n\n\
Readiness checklist: [FILL]\n\n\
Known limitations: [FILL]\n\n\
Release decision: [FILL]\n",
        feature = args.feature,
        branch = branch,
        pr = pr,
        intent = intent,
        contract = contract,
        event_schema = event_schema,
    );

    println!("{}\n", PREAMBLE);
    print!("{}", report);
    crate::EXIT_SUCCESS
}

fn fill() -> String {
    "[FILL]".to_string()
}

fn tag_inferred(v: &Option<String>) -> String {
    match v {
        Some(s) => format!("{} [INFERRED]", s),
        None => fill(),
    }
}

/// Resolves PR / Intent / Contract / Event schema from an optional registry lookup.
/// Any failure (no --registry, unreadable file, parse error, feature not found) degrades
/// gracefully to [FILL] for these four fields — never fatal to the command as a whole.
fn resolve_registry_fields(args: &GenerateReleaseEvidenceArgs) -> (String, String, String, String) {
    let all_fill = (fill(), fill(), fill(), fill());

    let Some(path) = args.registry else {
        // No --registry given at all: normal usage, no stderr note.
        return all_fill;
    };

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "warning: cannot read registry file '{}': {}; registry-derived fields left as [FILL]",
                path, e
            );
            return all_fill;
        }
    };

    // Schema version pre-probe for more specific diagnostic
    let pre_probe: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "warning: cannot parse registry file '{}': {}; registry-derived fields left as [FILL]",
                path, e
            );
            return all_fill;
        }
    };

    let schema_version_value = pre_probe.get("schema_version");
    let schema_version_u64 = schema_version_value.and_then(|v| v.as_u64());

    if schema_version_u64 != Some(2) {
        let version_display = match schema_version_value {
            None => "missing".to_string(),
            Some(v) => match v.as_u64() {
                Some(n) => n.to_string(),
                None => format!("{:?} (not a number)", v),
            },
        };
        eprintln!(
            "warning: registry '{}' does not declare schema_version: 2 (found: {}); registry-derived fields left as [FILL]",
            path, version_display
        );
        eprintln!("         See dba/06-reference/registry-v2-migration.md for migration instructions.");
        return all_fill;
    }

    let registry: Registry = match serde_yaml::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "warning: cannot parse registry file '{}': {}; registry-derived fields left as [FILL]",
                path, e
            );
            return all_fill;
        }
    };

    match registry.features.iter().find(|f| f.feature_id == args.feature) {
        Some(entry) => (
            tag_inferred(&entry.pr),
            tag_inferred(&entry.intent),
            tag_inferred(&entry.contract),
            tag_inferred(&entry.event_schema),
        ),
        None => {
            eprintln!(
                "warning: feature '{}' not found in registry '{}'; registry-derived fields left as [FILL]",
                args.feature, path
            );
            all_fill
        }
    }
}

fn git_branch(repo_root: &Path) -> String {
    match std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_root)
        .output()
    {
        Ok(out) if out.status.success() => {
            format!("{} [INFERRED]", String::from_utf8_lossy(&out.stdout).trim())
        }
        _ => fill(),
    }
}
