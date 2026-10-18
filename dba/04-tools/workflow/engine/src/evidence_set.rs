//! Feature Workflow v13 Validation Evidence Sets (DQA WO-06): one verification run, stored once
//! under its execution identity, which every feature's record may cite. Sharing reduces execution;
//! it never creates a record a feature did not earn by its own current bindings.

use crate::evidence::CommandOutcome;
use crate::execution::Environment;
use crate::hashing;
use crate::project::Project;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The verifications `check` shares: their command and inputs do not depend on which feature runs
/// them, beyond what the execution inputs state.
pub const SHARED: &[&str] = &[
    "smoke",
    "product_journey",
    "behavior",
    "repeatability",
    "data_integrity",
    "playwright",
    "project_qualification",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSet {
    pub schema: u8,
    pub verification: String,
    pub environment: String,
    pub execution_identity: String,
    pub execution_inputs: BTreeMap<String, String>,
    pub result: String,
    pub command: String,
    pub summary: String,
    pub recorded_at: String,
    /// What the verification reported: its Observed Surface, production observations, results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observations: Option<serde_json::Value>,
}

/// The execution identity of a set of inputs.
pub fn identity(
    verification: &str,
    environment: Environment,
    inputs: &BTreeMap<String, String>,
) -> String {
    let canonical = serde_json::json!({
        "verification": verification,
        "environment": environment.as_str(),
        "execution_inputs": inputs,
    });
    hashing::text_sha256(&canonical.to_string())
}

/// The inputs every shared verification consumes: the implementation, the build contexts of the
/// project's Compose services (a fixture provider or emulator among them), the project
/// configuration and the selected mechanics.
pub fn common_inputs(project: &Project) -> Result<BTreeMap<String, String>> {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "implementation_state".into(),
        project.implementation_state()?,
    );
    let contexts = compose_build_contexts(&project.root)?;
    inputs.insert(
        "compose_build_contexts".into(),
        hashing::working_tree_state(&project.root, &contexts)?,
    );
    inputs.insert(
        "codeos_yaml".into(),
        hashing::file_sha256(&project.codeos_yaml())?.unwrap_or_default(),
    );
    inputs.insert(
        "codeos_mechanics_policy".into(),
        project
            .selected_policy("codeos_mechanics_policy")?
            .unwrap_or_default(),
    );
    Ok(inputs)
}

/// The project-relative build context of every service the Compose file builds, sorted.
pub fn compose_build_contexts(root: &Path) -> Result<Vec<String>> {
    let Some(file) = [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ]
    .iter()
    .map(|name| root.join(name))
    .find(|path| path.is_file()) else {
        return Ok(Vec::new());
    };
    let text = std::fs::read_to_string(&file)?;
    let compose: serde_yaml::Value =
        serde_yaml::from_str(&text).with_context(|| format!("parse {}", file.display()))?;
    let mut contexts = Vec::new();
    if let Some(services) = compose.get("services").and_then(|s| s.as_mapping()) {
        for service in services.values() {
            let context = match service.get("build") {
                Some(serde_yaml::Value::String(context)) => Some(context.as_str()),
                Some(build) => build.get("context").and_then(|c| c.as_str()),
                None => None,
            };
            if let Some(context) = context {
                let relative = context.trim_start_matches("./").trim_end_matches('/');
                anyhow::ensure!(
                    !Path::new(relative).is_absolute()
                        && !relative.split('/').any(|part| part == ".."),
                    "Compose build context '{context}' is not inside the project"
                );
                contexts.push(if relative.is_empty() {
                    ".".into()
                } else {
                    relative.to_string()
                });
            }
        }
    }
    contexts.sort();
    contexts.dedup();
    Ok(contexts)
}

fn dir(project: &Project, identity: &str) -> PathBuf {
    project
        .codeos_dir()
        .join("06-workflow/evidence-sets")
        .join(identity)
}

/// A passed Evidence Set for this execution identity, with its content identity.
pub fn find_passed(project: &Project, identity: &str) -> Result<Option<(String, EvidenceSet)>> {
    let dir = dir(project, identity);
    if !dir.is_dir() {
        return Ok(None);
    }
    let mut paths: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    for path in paths {
        let text = std::fs::read_to_string(&path)?;
        let id = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        anyhow::ensure!(
            hashing::text_sha256(&text) == id,
            "Validation Evidence Set {} does not match its content",
            path.display()
        );
        let set: EvidenceSet = serde_json::from_str(&text)
            .with_context(|| format!("parse Validation Evidence Set {}", path.display()))?;
        anyhow::ensure!(
            set.execution_identity == identity,
            "Validation Evidence Set {id} is filed under another execution identity"
        );
        if set.result == "passed" {
            return Ok(Some((id, set)));
        }
    }
    Ok(None)
}

/// Store a run's Evidence Set; returns its content identity.
pub fn store(
    project: &Project,
    verification: &str,
    environment: Environment,
    inputs: BTreeMap<String, String>,
    outcome: &CommandOutcome,
) -> Result<String> {
    let execution_identity = identity(verification, environment, &inputs);
    let set = EvidenceSet {
        schema: 1,
        verification: verification.into(),
        environment: environment.as_str().into(),
        execution_identity: execution_identity.clone(),
        execution_inputs: inputs,
        result: if outcome.ok { "passed" } else { "failed" }.into(),
        command: outcome.command.clone(),
        summary: outcome.summary.clone(),
        recorded_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        observations: outcome.details.clone(),
    };
    let text = serde_json::to_string(&set)?;
    let id = hashing::text_sha256(&text);
    let dir = dir(project, &execution_identity);
    std::fs::create_dir_all(&dir).context("create Validation Evidence Set directory")?;
    std::fs::write(dir.join(format!("{id}.json")), text)
        .context("store Validation Evidence Set")?;
    Ok(id)
}

/// Record details citing a set: what the verification reported, plus the set's identity.
pub fn cite(details: Option<serde_json::Value>, id: &str) -> serde_json::Value {
    let mut details = match details {
        Some(serde_json::Value::Object(object)) => object,
        Some(other) => serde_json::Map::from_iter([("result".to_string(), other)]),
        None => serde_json::Map::new(),
    };
    details.insert("evidence_set".into(), id.into());
    serde_json::Value::Object(details)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_contexts_come_from_the_compose_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("docker-compose.yml"),
            "services:\n  db:\n    image: postgres\n  api:\n    build: ./backend\n  boundary:\n    build:\n      context: provider-boundary/\n",
        )
        .unwrap();
        assert_eq!(
            compose_build_contexts(dir.path()).unwrap(),
            vec!["backend".to_string(), "provider-boundary".to_string()]
        );
        std::fs::write(
            dir.path().join("docker-compose.yml"),
            "services:\n  x:\n    build: ../outside\n",
        )
        .unwrap();
        assert!(compose_build_contexts(dir.path()).is_err());
    }
}
