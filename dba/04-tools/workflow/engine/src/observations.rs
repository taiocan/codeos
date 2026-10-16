//! Durable Operation v2 observations, routes, and human closures.

use crate::contract::OPERATION_ROUTES;
use crate::hashing;
use crate::project::Project;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub schema: u8,
    pub id: String,
    pub statement: String,
    pub statement_sha256: String,
    #[serde(default)]
    pub sources: Vec<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub schema: u8,
    pub observation: String,
    pub observation_sha256: String,
    pub route: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_revision: Option<u64>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClosureEvidence {
    pub kind: String,
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Closure {
    pub schema: u8,
    pub observation: String,
    pub observation_sha256: String,
    pub route: String,
    #[serde(default)]
    pub evidence: Vec<ClosureEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    pub timestamp: String,
}

pub struct ObservationStore {
    dir: PathBuf,
}

impl ObservationStore {
    pub fn at(codeos_dir: &Path) -> Self {
        Self {
            dir: codeos_dir.join("06-workflow"),
        }
    }

    fn observations_path(&self) -> PathBuf {
        self.dir.join("observations.jsonl")
    }
    fn routes_path(&self) -> PathBuf {
        self.dir.join("operation-routes.jsonl")
    }
    fn closures_path(&self) -> PathBuf {
        self.dir.join("operation-closures.jsonl")
    }

    pub fn observations(&self) -> Result<Vec<Observation>> {
        read_jsonl(&self.observations_path())
    }
    pub fn routes(&self) -> Result<Vec<Route>> {
        read_jsonl(&self.routes_path())
    }
    pub fn closures(&self) -> Result<Vec<Closure>> {
        read_jsonl(&self.closures_path())
    }
    pub fn observation(&self, id: &str) -> Result<Option<Observation>> {
        Ok(self.observations()?.into_iter().find(|v| v.id == id))
    }
    pub fn route(&self, id: &str) -> Result<Option<Route>> {
        Ok(self.routes()?.into_iter().find(|v| v.observation == id))
    }
    pub fn closure(&self, id: &str) -> Result<Option<Closure>> {
        Ok(self.closures()?.into_iter().find(|v| v.observation == id))
    }

    pub fn observe(&self, statement: String, sources: Vec<String>) -> Result<Observation> {
        anyhow::ensure!(
            !statement.trim().is_empty(),
            "observation statement must not be empty"
        );
        let all = self.observations()?;
        let next = all
            .iter()
            .filter_map(|v| v.id.strip_prefix("OBS-")?.parse::<u64>().ok())
            .max()
            .unwrap_or(0)
            + 1;
        let item = Observation {
            schema: 2,
            id: format!("OBS-{next:04}"),
            statement_sha256: hashing::text_sha256(&statement),
            statement,
            sources,
            timestamp: now(),
        };
        append_json(&self.observations_path(), &item)?;
        Ok(item)
    }

    pub fn route_observation(
        &self,
        project: &Project,
        id: &str,
        route: &str,
        target_type: Option<String>,
        target_id: Option<String>,
    ) -> Result<Route> {
        anyhow::ensure!(
            OPERATION_ROUTES.contains(&route),
            "invalid operation route '{route}'"
        );
        anyhow::ensure!(
            self.route(id)?.is_none(),
            "observation {id} already has a route"
        );
        let observation = self
            .observation(id)?
            .with_context(|| format!("unknown observation {id}"))?;
        validate_target(route, target_type.as_deref(), target_id.as_deref())?;
        let target_path = target_path(project, route, target_id.as_deref());
        let target_sha256 = target_path
            .as_deref()
            .map(hashing::file_sha256)
            .transpose()?
            .flatten();
        let target_revision = if route == "specification_change" {
            target_path
                .as_deref()
                .map(semantic_revision)
                .transpose()?
                .flatten()
        } else {
            None
        };
        let item = Route {
            schema: 2,
            observation: id.into(),
            observation_sha256: observation.statement_sha256,
            route: route.into(),
            target_type,
            target_id,
            target_sha256,
            target_revision,
            timestamp: now(),
        };
        append_json(&self.routes_path(), &item)?;
        Ok(item)
    }

    pub fn close(
        &self,
        project: &Project,
        id: &str,
        evidence: Vec<ClosureEvidence>,
        rationale: Option<String>,
    ) -> Result<Closure> {
        anyhow::ensure!(
            self.closure(id)?.is_none(),
            "observation {id} is already closed"
        );
        let observation = self
            .observation(id)?
            .with_context(|| format!("unknown observation {id}"))?;
        let route = self
            .route(id)?
            .with_context(|| format!("observation {id} has no route"))?;
        anyhow::ensure!(
            route.observation_sha256 == observation.statement_sha256,
            "route is bound to a different observation statement"
        );
        validate_closure(project, &route, &evidence, rationale.as_deref())?;
        let item = Closure {
            schema: 2,
            observation: id.into(),
            observation_sha256: observation.statement_sha256,
            route: route.route,
            evidence,
            rationale,
            timestamp: now(),
        };
        append_json(&self.closures_path(), &item)?;
        Ok(item)
    }
}

fn validate_target(route: &str, target_type: Option<&str>, target_id: Option<&str>) -> Result<()> {
    let expected = match route {
        "implementation_defect" | "specification_change" | "new_feature" => Some("feature"),
        "charter_change" => Some("charter"),
        "architecture_reassessment" => Some("architecture_scope"),
        "no_action" => None,
        _ => unreachable!(),
    };
    match expected {
        Some(kind) => {
            anyhow::ensure!(
                target_type == Some(kind),
                "route {route} requires target type {kind}"
            );
            anyhow::ensure!(
                target_id.is_some_and(|v| !v.trim().is_empty()),
                "route {route} requires a target id"
            );
        }
        None => anyhow::ensure!(
            target_type.is_none() && target_id.is_none(),
            "no_action has no target"
        ),
    }
    Ok(())
}

fn target_path(project: &Project, route: &str, target: Option<&str>) -> Option<PathBuf> {
    match route {
        "implementation_defect" | "new_feature" | "specification_change" => {
            target.map(|v| project.intent(v))
        }
        "charter_change" => Some(project.charter()),
        "architecture_reassessment" => target.map(|v| {
            project
                .codeos_dir()
                .join(format!("02-architecture/scopes/{v}.md"))
        }),
        _ => None,
    }
}

fn validate_closure(
    project: &Project,
    route: &Route,
    evidence: &[ClosureEvidence],
    rationale: Option<&str>,
) -> Result<()> {
    let has = |kind: &str| evidence.iter().any(|v| v.kind == kind);
    match route.route.as_str() {
        "implementation_defect" => {
            anyhow::ensure!(
                has("commit") && has("verification_record"),
                "implementation_defect closure requires commit and verification_record evidence"
            );
        }
        "new_feature" => anyhow::ensure!(
            has("acceptance_receipt"),
            "new_feature closure requires acceptance_receipt evidence"
        ),
        "specification_change" => anyhow::ensure!(
            has("acceptance_receipt"),
            "specification_change closure requires acceptance_receipt evidence"
        ),
        "charter_change" | "architecture_reassessment" => anyhow::ensure!(
            has("approved_artifact"),
            "{} closure requires approved_artifact evidence",
            route.route
        ),
        "no_action" => anyhow::ensure!(
            rationale.is_some_and(|v| !v.trim().is_empty()),
            "no_action closure requires rationale"
        ),
        _ => anyhow::bail!("invalid route {}", route.route),
    }
    for item in evidence {
        validate_evidence(project, route, item)?;
    }
    if route.route == "specification_change" {
        let path = project.intent(route.target_id.as_deref().unwrap_or_default());
        let current =
            semantic_revision(&path)?.context("target feature has no semantic_revision")?;
        anyhow::ensure!(
            current > route.target_revision.unwrap_or(0),
            "specification_change closure requires a later semantic revision"
        );
    }
    Ok(())
}

fn validate_evidence(project: &Project, route: &Route, evidence: &ClosureEvidence) -> Result<()> {
    match evidence.kind.as_str() {
        "commit" => {
            let status = crate::instrument::command("git")
                .args([
                    "cat-file",
                    "-e",
                    &format!("{}^{{commit}}", evidence.reference),
                ])
                .current_dir(&project.root)
                .status()?;
            anyhow::ensure!(status.success(), "commit evidence does not exist");
        }
        "verification_record" => validate_jsonl_identity(
            &project.codeos_dir().join("06-workflow/verifications.jsonl"),
            &evidence.reference,
            &route.timestamp,
            None,
            None,
            Some("passed"),
        )?,
        "acceptance_receipt" => validate_jsonl_identity(
            &project.codeos_dir().join("06-workflow/decisions.jsonl"),
            &evidence.reference,
            &route.timestamp,
            route.target_id.as_deref(),
            Some("acceptance"),
            Some("accepted"),
        )?,
        "approved_artifact" => {
            let path = project.root.join(&evidence.reference);
            anyhow::ensure!(path.is_file(), "approved artifact evidence does not exist");
            let expected = target_path(project, &route.route, route.target_id.as_deref())
                .context("route has no governed artifact target")?;
            anyhow::ensure!(
                path == expected,
                "approved artifact evidence does not match the routed target"
            );
            let text = std::fs::read_to_string(&path)?;
            anyhow::ensure!(
                text.contains("status: APPROVED")
                    || (text.contains("approval:") && text.contains("by:") && text.contains("at:")),
                "artifact is not approved"
            );
            let current =
                hashing::file_sha256(&path)?.context("approved artifact has no identity")?;
            anyhow::ensure!(
                Some(current) != route.target_sha256,
                "approved artifact identity did not change after routing"
            );
        }
        other => anyhow::bail!("unsupported closure evidence kind '{other}'"),
    }
    Ok(())
}

fn validate_jsonl_identity(
    path: &Path,
    wanted: &str,
    after: &str,
    subject: Option<&str>,
    checkpoint: Option<&str>,
    result: Option<&str>,
) -> Result<()> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let found = text.lines().filter(|v| !v.trim().is_empty()).any(|line| {
        if hashing::text_sha256(line) != wanted {
            return false;
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            return false;
        };
        value
            .get("timestamp")
            .and_then(|v| v.as_str())
            .is_some_and(|v| v > after)
            && subject.is_none_or(|s| value.get("subject").and_then(|v| v.as_str()) == Some(s))
            && checkpoint
                .is_none_or(|s| value.get("checkpoint").and_then(|v| v.as_str()) == Some(s))
            && result.is_none_or(|s| value.get("result").and_then(|v| v.as_str()) == Some(s))
    });
    anyhow::ensure!(
        found,
        "closure evidence identity is absent, too early, or targets another subject"
    );
    Ok(())
}

fn semantic_revision(path: &Path) -> Result<Option<u64>> {
    let text = match std::fs::read_to_string(path) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    Ok(text.lines().find_map(|line| {
        line.trim()
            .strip_prefix("semantic_revision:")?
            .trim()
            .parse()
            .ok()
    }))
}

fn read_jsonl<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    std::fs::read_to_string(path)?
        .lines()
        .enumerate()
        .filter(|(_, v)| !v.trim().is_empty())
        .map(|(i, line)| {
            serde_json::from_str(line)
                .with_context(|| format!("{}:{}: malformed record", path.display(), i + 1))
        })
        .collect()
}

fn append_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

fn now() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}
