//! Minimal project-owned qualification protocol. Codeos validates results and bindings; the
//! project chooses and runs the underlying tools.

use crate::evidence::CommandOutcome;
use crate::hashing;
use crate::project::Project;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

const BASELINE_IDS: &[&str] = &[
    "static_analysis",
    "dependency_vulnerabilities",
    "dependency_licences",
    "committed_secrets",
];

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct QualificationResult {
    schema: u8,
    project: String,
    implementation_sha256: String,
    charter_sha256: String,
    configuration_sha256: String,
    #[serde(default)]
    tools: BTreeMap<String, String>,
    constraints: Vec<ConstraintResult>,
    baseline: Vec<BaselineResult>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConstraintResult {
    id: String,
    phase: String,
    status: String,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    measurements: BTreeMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    justification: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BaselineResult {
    id: String,
    status: String,
    tool: String,
    version: String,
    summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    justification: Option<String>,
}

pub fn run(project: &Project, mode: &str) -> CommandOutcome {
    match run_inner(project, mode) {
        Ok((ok, summary, details)) => CommandOutcome {
            ok,
            command: format!("CODEOS_QUALIFICATION_MODE={mode} scripts/codeos-qualify"),
            summary,
            details: Some(details),
        },
        Err(error) => CommandOutcome {
            ok: false,
            command: format!("CODEOS_QUALIFICATION_MODE={mode} scripts/codeos-qualify"),
            summary: error.to_string(),
            details: None,
        },
    }
}

fn run_inner(project: &Project, mode: &str) -> anyhow::Result<(bool, String, serde_json::Value)> {
    anyhow::ensure!(
        matches!(mode, "acceptance" | "deployment"),
        "qualification mode must be acceptance or deployment"
    );
    let command = project.root.join("scripts/codeos-qualify");
    anyhow::ensure!(
        command.is_file(),
        "missing project qualification executable scripts/codeos-qualify"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        anyhow::ensure!(
            command.metadata()?.permissions().mode() & 0o111 != 0,
            "scripts/codeos-qualify is not executable"
        );
    }
    let charter_sha256 = hashing::file_sha256(&project.charter())?
        .ok_or_else(|| anyhow::anyhow!("Charter is missing"))?;
    let configuration_sha256 = hashing::file_sha256(&project.codeos_yaml())?
        .ok_or_else(|| anyhow::anyhow!("codeos.yaml is missing"))?;
    let implementation_sha256 =
        hashing::working_tree_state(&project.root, &project.implementation_paths())?;
    let output = tempfile::NamedTempFile::new()?;
    let status = Command::new(&command)
        .current_dir(&project.root)
        .env("CODEOS_QUALIFICATION_MODE", mode)
        .env("CODEOS_QUALIFICATION_OUTPUT", output.path())
        .env("CODEOS_CHARTER_PATH", project.charter())
        .env("CODEOS_PROJECT_ROOT", &project.root)
        .env("CODEOS_IMPLEMENTATION_SHA256", &implementation_sha256)
        .env("CODEOS_CHARTER_SHA256", &charter_sha256)
        .env("CODEOS_CONFIGURATION_SHA256", &configuration_sha256)
        .status()?;
    anyhow::ensure!(
        status.success(),
        "project qualification command exited with {status}"
    );
    let bytes = std::fs::read(output.path())?;
    let result: QualificationResult = serde_json::from_slice(&bytes)?;
    anyhow::ensure!(
        result.schema == 1,
        "unsupported qualification result schema"
    );
    anyhow::ensure!(
        result.implementation_sha256 == implementation_sha256,
        "qualification implementation identity mismatch"
    );
    anyhow::ensure!(
        result.charter_sha256 == charter_sha256,
        "qualification Charter identity mismatch"
    );
    anyhow::ensure!(
        result.configuration_sha256 == configuration_sha256,
        "qualification configuration identity mismatch"
    );

    let expected_constraints = charter_constraint_ids(&std::fs::read_to_string(project.charter())?);
    let actual_constraints: BTreeSet<_> = result.constraints.iter().map(|v| v.id.clone()).collect();
    anyhow::ensure!(
        actual_constraints.len() == result.constraints.len(),
        "qualification contains duplicate constraint IDs"
    );
    anyhow::ensure!(
        actual_constraints == expected_constraints,
        "qualification constraint IDs do not exactly match the Charter"
    );
    let actual_baseline: BTreeSet<_> = result.baseline.iter().map(|v| v.id.as_str()).collect();
    anyhow::ensure!(
        actual_baseline.len() == result.baseline.len(),
        "qualification contains duplicate baseline IDs"
    );
    anyhow::ensure!(
        actual_baseline == BASELINE_IDS.iter().copied().collect(),
        "qualification baseline must contain exactly the four required IDs"
    );

    let mut blockers = Vec::new();
    for item in &result.constraints {
        anyhow::ensure!(
            matches!(item.phase.as_str(), "acceptance" | "deployment"),
            "constraint {} has invalid phase",
            item.id
        );
        anyhow::ensure!(
            matches!(
                item.status.as_str(),
                "pass" | "fail" | "unresolved" | "not_applicable"
            ),
            "constraint {} has invalid status",
            item.id
        );
        if item.status == "not_applicable" {
            anyhow::ensure!(
                item.justification
                    .as_deref()
                    .is_some_and(|v| !v.trim().is_empty()),
                "constraint {} has unjustified not_applicable status",
                item.id
            );
        }
        let applicable_now = mode == "deployment" || item.phase == "acceptance";
        if applicable_now && matches!(item.status.as_str(), "fail" | "unresolved") {
            blockers.push(item.id.clone());
        }
    }
    for item in &result.baseline {
        anyhow::ensure!(
            matches!(item.status.as_str(), "pass" | "fail" | "not_applicable"),
            "baseline {} has invalid status",
            item.id
        );
        anyhow::ensure!(
            !item.tool.trim().is_empty() && !item.version.trim().is_empty(),
            "baseline {} omits tool identity",
            item.id
        );
        if item.status == "not_applicable" {
            anyhow::ensure!(
                item.justification
                    .as_deref()
                    .is_some_and(|v| !v.trim().is_empty()),
                "baseline {} has unjustified not_applicable status",
                item.id
            );
        }
        if item.status == "fail" {
            blockers.push(item.id.clone());
        }
    }
    let details = serde_json::to_value(&result)?;
    let summary = if blockers.is_empty() {
        format!("{mode} qualification passed")
    } else {
        format!("{mode} qualification blocked by {}", blockers.join(", "))
    };
    Ok((blockers.is_empty(), summary, details))
}

fn charter_constraint_ids(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| {
            let mut cells = line.trim().trim_matches('|').split('|').map(str::trim);
            let id = cells.next()?;
            (id.starts_with("C-") && id[2..].chars().all(|v| v.is_ascii_digit()))
                .then(|| id.to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_only_constraint_table_ids() {
        let ids = charter_constraint_ids(
            "| ID | Constraint |\n|---|---|\n| C-1 | mobile |\ntext C-2\n| C-x | no |",
        );
        assert_eq!(ids, ["C-1".to_string()].into_iter().collect());
    }
}
