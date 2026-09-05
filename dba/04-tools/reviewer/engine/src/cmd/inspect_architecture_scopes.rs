use crate::{EXIT_CONFIG, EXIT_SUCCESS};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopeMetadata {
    features: Vec<String>,
    approval: serde_yaml::Value,
    // Present on a DBA-5 (or later) Architecture Scope; absent on an already-adopted DBA-4 scope,
    // which remains valid unchanged. Neither field affects scope facts — governance state is read
    // from `features`/`approval` only, per the Downstream Artifact Frontmatter Contract.
    #[serde(default)]
    artifact_type: Option<String>,
    #[serde(default)]
    reader_model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Approval {
    by: String,
    at: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum ApprovalState {
    Draft,
    Approved,
}

#[derive(Debug)]
struct Scope {
    id: String,
    path: String,
    state: ApprovalState,
    features: Vec<String>,
}

#[derive(Serialize)]
struct ScopeFact<'a> {
    scope: &'a str,
    path: &'a str,
    state: ApprovalState,
    features: &'a [String],
}

#[derive(Serialize)]
struct FeatureFact<'a> {
    id: &'a str,
    resolution: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<&'a str>,
}

#[derive(Serialize)]
struct Inspection<'a> {
    architecture_scopes: Vec<ScopeFact<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feature: Option<FeatureFact<'a>>,
}

pub fn run(feature: Option<&str>, repo_root: &Path) -> i32 {
    match inspect(repo_root) {
        Ok(scopes) => {
            let feature_fact = feature.map(|id| {
                let matched = scopes
                    .iter()
                    .find(|scope| scope.features.iter().any(|item| item == id));
                match matched {
                    Some(scope) => FeatureFact {
                        id,
                        resolution: match scope.state {
                            ApprovalState::Draft => "draft",
                            ApprovalState::Approved => "approved",
                        },
                        scope: Some(&scope.id),
                        path: Some(&scope.path),
                    },
                    None => FeatureFact {
                        id,
                        resolution: "none",
                        scope: None,
                        path: None,
                    },
                }
            });
            let output = Inspection {
                architecture_scopes: scopes
                    .iter()
                    .map(|scope| ScopeFact {
                        scope: &scope.id,
                        path: &scope.path,
                        state: scope.state,
                        features: &scope.features,
                    })
                    .collect(),
                feature: feature_fact,
            };
            match serde_yaml::to_string(&output) {
                Ok(text) => {
                    print!("{text}");
                    EXIT_SUCCESS
                }
                Err(error) => {
                    eprintln!("error: cannot format architecture inspection: {error}");
                    EXIT_CONFIG
                }
            }
        }
        Err(error) => {
            eprintln!("error: {error}");
            EXIT_CONFIG
        }
    }
}

fn inspect(repo_root: &Path) -> Result<Vec<Scope>, String> {
    let directory = repo_root.join(".codeos/02-architecture/scopes");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    if !directory.is_dir() {
        return Err(".codeos/02-architecture/scopes exists but is not a directory".to_string());
    }

    let mut paths = fs::read_dir(&directory)
        .map_err(|error| format!("cannot read .codeos/02-architecture/scopes: {error}"))?
        .filter_map(|entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                (path.extension().and_then(|value| value.to_str()) == Some("md"))
                    .then_some(Ok(path))
            }
            Err(error) => Some(Err(format!(
                "cannot read .codeos/02-architecture/scopes entry: {error}"
            ))),
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();

    let mut scopes = Vec::new();
    let mut owners: BTreeMap<String, String> = BTreeMap::new();
    for path in paths {
        let scope = parse_scope(repo_root, &path)?;
        for feature in &scope.features {
            if let Some(existing) = owners.insert(feature.clone(), scope.id.clone()) {
                return Err(format!(
                    "feature {feature:?} belongs to both architecture scopes {existing:?} and {:?}",
                    scope.id
                ));
            }
        }
        scopes.push(scope);
    }
    Ok(scopes)
}

fn parse_scope(repo_root: &Path, path: &Path) -> Result<Scope, String> {
    if !path.is_file() {
        return Err(format!(
            "architecture scope is not a regular file: {}",
            path.display()
        ));
    }
    let id = path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!(
                "architecture scope has an invalid filename: {}",
                path.display()
            )
        })?
        .to_string();
    let relative = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let content =
        fs::read_to_string(path).map_err(|error| format!("cannot read {relative}: {error}"))?;
    let front_matter =
        extract_front_matter(&content).map_err(|error| format!("invalid {relative}: {error}"))?;
    let metadata: ScopeMetadata = serde_yaml::from_str(&front_matter)
        .map_err(|error| format!("invalid {relative} metadata: {error}"))?;

    if metadata.features.is_empty() {
        return Err(format!("invalid {relative}: features must not be empty"));
    }
    let mut unique = BTreeSet::new();
    for feature in &metadata.features {
        if feature.trim().is_empty() || feature != feature.trim() {
            return Err(format!(
                "invalid {relative}: feature ids must be non-empty and trimmed"
            ));
        }
        if !unique.insert(feature.clone()) {
            return Err(format!("invalid {relative}: duplicate feature {feature:?}"));
        }
    }

    let state = match metadata.approval {
        serde_yaml::Value::Null => ApprovalState::Draft,
        value => {
            let approval: Approval = serde_yaml::from_value(value)
                .map_err(|error| format!("invalid {relative} approval: {error}"))?;
            if approval.by.trim().is_empty() || approval.at.trim().is_empty() {
                return Err(format!(
                    "invalid {relative}: approval.by and approval.at must be non-empty"
                ));
            }
            ApprovalState::Approved
        }
    };

    Ok(Scope {
        id,
        path: relative,
        state,
        features: metadata.features,
    })
}

fn extract_front_matter(content: &str) -> Result<String, String> {
    let mut lines = content.lines();
    if lines.next() != Some("---") {
        return Err("missing opening YAML front-matter delimiter".to_string());
    }
    let mut front_matter = Vec::new();
    for line in lines {
        if line == "---" {
            return Ok(front_matter.join("\n"));
        }
        front_matter.push(line);
    }
    Err("missing closing YAML front-matter delimiter".to_string())
}
