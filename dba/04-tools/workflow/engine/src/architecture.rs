//! Feature-scoped architecture identity and deterministic proposal impact.

use crate::project::Project;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    artifact_type: String,
    features: Vec<String>,
    approval: Approval,
    reader_model: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Approval {
    Pending(()),
    Approved { by: String, at: String },
}

#[derive(Debug, Serialize)]
struct Projection<'a> {
    scope_path: &'a str,
    subject_membership: &'a str,
    applicable_sections: &'a [String],
}

#[derive(Debug, Serialize)]
pub struct ArchitectureImpact {
    pub base: String,
    pub proposed: String,
    pub affected_features: Vec<String>,
    pub unchanged_features: Vec<String>,
    pub feature_identities: BTreeMap<String, IdentityChange>,
}

#[derive(Debug, Serialize)]
pub struct IdentityChange {
    pub before: String,
    pub after: String,
}

fn normalized(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn parts(text: &str) -> Result<(Metadata, &str)> {
    let rest = text
        .strip_prefix("---\n")
        .context("Architecture Scope has no opening frontmatter delimiter")?;
    let (frontmatter, body) = rest
        .split_once("\n---\n")
        .context("Architecture Scope has no closing frontmatter delimiter")?;
    let metadata: Metadata =
        serde_yaml::from_str(frontmatter).context("parse Architecture Scope frontmatter")?;
    anyhow::ensure!(
        metadata.artifact_type == "architecture",
        "Architecture Scope artifact_type must be architecture"
    );
    anyhow::ensure!(
        !metadata.reader_model.trim().is_empty(),
        "Architecture Scope reader_model must be non-empty"
    );
    anyhow::ensure!(
        !metadata.features.is_empty() && metadata.features.iter().all(|v| !v.trim().is_empty()),
        "Architecture Scope features must be non-empty"
    );
    let unique = metadata.features.iter().collect::<BTreeSet<_>>();
    anyhow::ensure!(
        unique.len() == metadata.features.len(),
        "Architecture Scope features must be duplicate-free"
    );
    if let Approval::Approved { by, at } = &metadata.approval {
        anyhow::ensure!(
            !by.trim().is_empty() && !at.trim().is_empty(),
            "Architecture Scope approval must be complete"
        );
    }
    Ok((metadata, body))
}

fn sections(body: &str, subject: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for line in body.split_inclusive('\n') {
        if line.starts_with("## ") && !current.is_empty() {
            if section_applies(&current, subject) {
                out.push(format!("{}\n", current.trim_end_matches('\n')));
            }
            current = String::new();
        }
        current.push_str(line);
    }
    if !current.is_empty() && section_applies(&current, subject) {
        out.push(format!("{}\n", current.trim_end_matches('\n')));
    }
    out
}

/// Unscoped prose is shared and therefore fail-closed for every member. A section can narrow its
/// applicability with an immediate `Applies to:` line. Structured journey transitions are scoped
/// mechanically to their producer and consumer features.
fn section_applies(section: &str, subject: &str) -> bool {
    let applies = section
        .lines()
        .skip(1)
        .find(|line| !line.trim().is_empty())
        .and_then(|line| line.trim().strip_prefix("Applies to:"));
    if let Some(list) = applies {
        return list
            .split(',')
            .map(str::trim)
            .any(|feature| feature == subject || feature == "all");
    }
    if section.contains("producer_feature:") || section.contains("consumer_feature:") {
        return section.lines().any(|line| {
            line.trim()
                .strip_prefix("producer_feature:")
                .or_else(|| line.trim().strip_prefix("consumer_feature:"))
                .is_some_and(|feature| feature.trim() == subject)
        });
    }
    true
}

fn identity_from_files(
    files: &[(String, String)],
    subject: &str,
    require_approval: bool,
) -> Result<String> {
    let mut matches = Vec::new();
    for (path, text) in files {
        let normalized = normalized(text);
        let (metadata, body) =
            parts(&normalized).with_context(|| format!("invalid Architecture Scope {path}"))?;
        if metadata.features.iter().any(|feature| feature == subject) {
            if require_approval {
                anyhow::ensure!(
                    matches!(metadata.approval, Approval::Approved { .. }),
                    "applicable Architecture Scope is not approved"
                );
            }
            matches.push((path.as_str(), sections(body, subject)));
        }
    }
    match matches.as_slice() {
        [] => Ok("NO_APPLICABLE_SCOPE".into()),
        [(path, applicable_sections)] => Ok(hex::encode(Sha256::digest(serde_json::to_vec(
            &Projection {
                scope_path: path,
                subject_membership: subject,
                applicable_sections,
            },
        )?))),
        _ => anyhow::bail!("feature has ambiguous Architecture Scope membership"),
    }
}

fn tree_files(project: &Project, treeish: &str) -> Result<Vec<(String, String)>> {
    if treeish == "WORKTREE" {
        let root = project.codeos_dir().join("02-architecture/scopes");
        if !root.is_dir() {
            return Ok(Vec::new());
        }
        let mut paths = std::fs::read_dir(root)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("md"))
            .collect::<Vec<_>>();
        paths.sort();
        return paths
            .into_iter()
            .map(|path| {
                let relative = path
                    .strip_prefix(&project.root)?
                    .to_string_lossy()
                    .to_string();
                Ok((relative, std::fs::read_to_string(path)?))
            })
            .collect();
    }
    let listing = git(
        project,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            treeish,
            "--",
            ".codeos/02-architecture/scopes",
        ],
    )?;
    listing
        .lines()
        .filter(|path| path.ends_with(".md"))
        .map(|path| {
            let object = format!("{treeish}:{path}");
            let bytes = git_bytes(project, &["show", &object])?;
            Ok((
                path.to_string(),
                String::from_utf8(bytes)
                    .with_context(|| format!("Architecture Scope {path} is not UTF-8"))?,
            ))
        })
        .collect()
}

pub fn feature_identity(project: &Project, subject: &str, treeish: &str) -> Result<String> {
    identity_from_files(&tree_files(project, treeish)?, subject, true)
}

pub fn impact(project: &Project, base: &str, proposed: &str) -> Result<ArchitectureImpact> {
    let before_files = tree_files(project, base)?;
    let after_files = tree_files(project, proposed)?;
    let mut features = BTreeSet::new();
    for (_, text) in before_files.iter().chain(after_files.iter()) {
        let normalized = normalized(text);
        let (metadata, _) = parts(&normalized)?;
        features.extend(metadata.features);
    }
    let mut affected = Vec::new();
    let mut unchanged = Vec::new();
    let mut identities = BTreeMap::new();
    for feature in features {
        let before = identity_from_files(&before_files, &feature, false)?;
        let after = identity_from_files(&after_files, &feature, false)?;
        if before == after {
            unchanged.push(feature.clone());
        } else {
            affected.push(feature.clone());
        }
        identities.insert(feature, IdentityChange { before, after });
    }
    Ok(ArchitectureImpact {
        base: base.into(),
        proposed: proposed.into(),
        affected_features: affected,
        unchanged_features: unchanged,
        feature_identities: identities,
    })
}

fn git(project: &Project, args: &[&str]) -> Result<String> {
    Ok(String::from_utf8(git_bytes(project, args)?)?
        .trim()
        .to_string())
}

fn git_bytes(project: &Project, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(&project.root)
        .output()?;
    anyhow::ensure!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(output.stdout)
}

/// Every current Architecture Scope with whether it records human approval.
pub fn scope_approvals(project: &Project) -> Result<Vec<(String, bool)>> {
    tree_files(project, "WORKTREE")?
        .into_iter()
        .map(|(path, text)| {
            let normalized = normalized(&text);
            let (metadata, _) = parts(&normalized)?;
            Ok((path, matches!(metadata.approval, Approval::Approved { .. })))
        })
        .collect()
}

pub fn scope_path(project: &Project, subject: &str) -> Result<Option<String>> {
    let files = tree_files(project, "WORKTREE")?;
    let mut matches = Vec::new();
    for (path, text) in files {
        let normalized = normalized(&text);
        let (metadata, _) = parts(&normalized)?;
        if metadata.features.iter().any(|feature| feature == subject) {
            matches.push(path);
        }
    }
    anyhow::ensure!(
        matches.len() <= 1,
        "feature has ambiguous Architecture Scope membership"
    );
    Ok(matches.pop())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(features: &str, approval: &str, body: &str) -> String {
        format!(
            "---\nartifact_type: architecture\nfeatures: [{features}]\napproval: {approval}\nreader_model: whole-before-parts\n---\n{body}"
        )
    }

    fn repo() -> (tempfile::TempDir, Project) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let run = |args: &[&str]| {
            let output = Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        std::fs::create_dir_all(root.join(".codeos")).unwrap();
        (dir, Project::discover(&root).unwrap())
    }

    #[test]
    fn scoped_decision_changes_only_named_features() {
        let (dir, project) = repo();
        let root = dir.path();
        let path = root.join(".codeos/02-architecture/scopes/shared.md");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            scope(
                "F-0001, F-0002",
                "{by: Human, at: 2026-01-01}",
                "\n# Scope\n\n## Shared\n\nShared rule.\n",
            ),
        )
        .unwrap();
        git(&project, &["add", "."]).unwrap();
        git(&project, &["commit", "-qm", "base"]).unwrap();
        let base = git(&project, &["rev-parse", "HEAD"]).unwrap();

        std::fs::write(
            &path,
            scope(
                "F-0001, F-0002, F-0004",
                "null",
                "\n# Scope\n\n## Shared\n\nShared rule.\n\n## F4 adapter\n\nApplies to: F-0004\n\nF4-only rule.\n",
            ),
        )
        .unwrap();
        let report = impact(&project, &base, "WORKTREE").unwrap();
        assert_eq!(report.affected_features, vec!["F-0004"]);
        assert_eq!(report.unchanged_features, vec!["F-0001", "F-0002"]);
        assert!(feature_identity(&project, "F-0004", "WORKTREE").is_err());
    }

    #[test]
    fn unscoped_change_is_fail_closed_for_every_member() {
        let (dir, project) = repo();
        let root = dir.path();
        let path = root.join(".codeos/02-architecture/scopes/shared.md");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            scope(
                "F-0001, F-0002",
                "{by: Human, at: 2026-01-01}",
                "\n# Scope\n\n## Shared\n\nShared rule.\n",
            ),
        )
        .unwrap();
        git(&project, &["add", "."]).unwrap();
        git(&project, &["commit", "-qm", "base"]).unwrap();
        let base = git(&project, &["rev-parse", "HEAD"]).unwrap();
        std::fs::write(
            &path,
            scope(
                "F-0001, F-0002",
                "null",
                "\n# Scope\n\n## Shared\n\nChanged shared rule.\n",
            ),
        )
        .unwrap();
        let report = impact(&project, &base, "WORKTREE").unwrap();
        assert_eq!(report.affected_features, vec!["F-0001", "F-0002"]);
    }
}
