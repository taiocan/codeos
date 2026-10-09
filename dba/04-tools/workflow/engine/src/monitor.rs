//! Read-only project status projection and terminal rendering.

use crate::checker::{self, CheckpointReport, State, Subject};
use crate::contract::Workflow;
use crate::evidence;
use crate::project::Project;
use crate::receipts::ReceiptStore;
use anyhow::Result;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisplayStatus {
    Complete,
    WaitingHuman,
    Blocked,
    NotApplicable,
}

impl DisplayStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Complete => "COMPLETE",
            Self::WaitingHuman => "WAITING HUMAN",
            Self::Blocked => "BLOCKED",
            Self::NotApplicable => "n/a",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CheckpointStatus {
    pub id: String,
    pub title: String,
    pub status: DisplayStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basis: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkflowStatus {
    pub status: DisplayStatus,
    pub current_checkpoint: Option<String>,
    pub completed: usize,
    pub total: usize,
    pub primary_issue: Option<String>,
    pub additional_issue_count: usize,
    pub next_action: Option<String>,
    pub checkpoints: Vec<CheckpointStatus>,
}

#[derive(Debug, Serialize)]
pub struct FeatureStatus {
    pub id: String,
    pub name: String,
    pub historically_accepted: bool,
    pub current_acceptance_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_snapshot: Option<AcceptedSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_validity: Option<CurrentValidity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acceptance_reason: Option<String>,
    #[serde(flatten)]
    pub workflow: WorkflowStatus,
}

#[derive(Debug, Serialize)]
pub struct AcceptedSnapshot {
    pub receipt_sha256: String,
    pub accepted_at: String,
    pub semantic_revision: Option<u64>,
    pub bindings: std::collections::BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_bundle_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CurrentValidity {
    pub status: String,
    pub reasons: Vec<String>,
}

/// An informational feature candidate from a non-governed Feature Decomposition artifact.
/// Candidates have no Feature ID or lifecycle state and cannot affect workflow progression.
#[derive(Debug, Serialize)]
pub struct PlannedFeatureCandidate {
    pub name: String,
    pub human_outcome: String,
    pub boundary_or_dependency: String,
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectIdentity {
    pub name: String,
    pub configuration: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectStatus {
    pub schema_version: u8,
    pub project: ProjectIdentity,
    pub bootstrap: WorkflowStatus,
    pub current_feature: Option<String>,
    pub features: Vec<FeatureStatus>,
    pub planned_feature_candidates: Vec<PlannedFeatureCandidate>,
    pub feature_inventory_warnings: Vec<String>,
}

impl ProjectStatus {
    pub fn derive(project: &Project) -> Result<Self> {
        let bootstrap_subject = Subject::resolve(project, Workflow::Bootstrap, "solution")?;
        let bootstrap = summarize(&checker::evaluate(project, &bootstrap_subject)?);

        let mut features = Vec::new();
        for id in specification_feature_ids(project)? {
            let subject = Subject::resolve(project, Workflow::Feature, &id)?;
            let historically_accepted = crate::revalidation::historically_accepted(project, &id)?;
            let current_acceptance_valid = historically_accepted
                && crate::revalidation::current_acceptance_valid(project, &id)?;
            let governance_v3 = project.workflow_governance_version()? >= 3;
            let mut workflow = summarize(&checker::evaluate(project, &subject)?);
            let stale_reason = if historically_accepted && !current_acceptance_valid {
                Some(crate::revalidation::invalidity_reason(project, &id)?)
            } else {
                None
            };
            let acceptance_reason = if historically_accepted && !current_acceptance_valid {
                let reason = stale_reason.expect("stale accepted feature has a reason");
                workflow.status = DisplayStatus::Blocked;
                workflow.current_checkpoint = Some(
                    match reason {
                        "SEMANTIC_CHANGE_REQUIRED" => "F1",
                        "ARCHITECTURE_CHANGE_REQUIRED" => "ARCHITECTURE_REASSESSMENT",
                        _ => "POST_ACCEPTANCE_REVALIDATION",
                    }
                    .into(),
                );
                workflow.primary_issue = Some(match reason {
                    "SEMANTIC_CHANGE_REQUIRED" => "historical F9 acceptance is preserved, but governed feature meaning changed".into(),
                    "ARCHITECTURE_CHANGE_REQUIRED" => "historical F9 acceptance is preserved, but applicable architecture changed".into(),
                    _ => "historical F9 acceptance is preserved, but current acceptance is stale after a later implementation/dependency change".into(),
                });
                workflow.additional_issue_count = 0;
                workflow.next_action = Some(match reason {
                    "SEMANTIC_CHANGE_REQUIRED" => "start ordinary governed work at F1".into(),
                    "ARCHITECTURE_CHANGE_REQUIRED" => {
                        "complete architecture impact and use an ordinary Candidate Bundle".into()
                    }
                    _ => "run targeted checks and prepare one post-acceptance Candidate Bundle"
                        .into(),
                });
                Some(reason.into())
            } else if current_acceptance_valid {
                workflow.status = DisplayStatus::Complete;
                workflow.current_checkpoint = None;
                workflow.primary_issue = None;
                workflow.additional_issue_count = 0;
                workflow.next_action = None;
                None
            } else {
                None
            };
            let accepted_snapshot = if governance_v3 {
                acceptance_snapshot(project, &id)?
            } else {
                None
            };
            let current_validity = governance_v3.then(|| CurrentValidity {
                status: if !historically_accepted {
                    "NOT_ACCEPTED"
                } else if current_acceptance_valid {
                    "VALID"
                } else {
                    "STALE"
                }
                .into(),
                reasons: if historically_accepted && !current_acceptance_valid {
                    vec![stale_reason
                        .expect("stale accepted feature has a reason")
                        .into()]
                } else {
                    vec![]
                },
            });
            if governance_v3 && historically_accepted {
                snapshot_completed_checkpoints(&mut workflow);
                if !current_acceptance_valid {
                    workflow.status = DisplayStatus::Blocked;
                    let reason = stale_reason.expect("stale accepted feature has a reason");
                    workflow.current_checkpoint = Some(
                        match reason {
                            "SEMANTIC_CHANGE_REQUIRED" => "F1",
                            "ARCHITECTURE_CHANGE_REQUIRED" => "ARCHITECTURE_REASSESSMENT",
                            _ => "POST_ACCEPTANCE_REVALIDATION",
                        }
                        .into(),
                    );
                    workflow.primary_issue = Some(format!("historical F9 acceptance is preserved; current acceptance validity is stale ({reason})"));
                    workflow.additional_issue_count = 0;
                    workflow.next_action = Some(match reason {
                        "SEMANTIC_CHANGE_REQUIRED" => "start ordinary governed work at F1".into(),
                        "ARCHITECTURE_CHANGE_REQUIRED" => {
                            "complete architecture impact and use an ordinary Candidate Bundle"
                                .into()
                        }
                        _ => "revalidate only the affected accepted surface".into(),
                    });
                }
            }
            features.push(FeatureStatus {
                name: feature_name(project, &id),
                id,
                historically_accepted,
                current_acceptance_valid,
                accepted_snapshot,
                current_validity,
                acceptance_reason,
                workflow,
            });
        }

        let started_names = features
            .iter()
            .map(|feature| normalized_name(&feature.name))
            .collect::<BTreeSet<_>>();
        let (planned_feature_candidates, feature_inventory_warnings) =
            planned_feature_candidates(project, &started_names)?;

        let incomplete: Vec<_> = features
            .iter()
            .filter(|feature| feature.workflow.status != DisplayStatus::Complete)
            .map(|feature| feature.id.clone())
            .collect();
        let current_feature = (incomplete.len() == 1).then(|| incomplete[0].clone());

        Ok(Self {
            schema_version: if project.workflow_governance_version()? >= 3 {
                3
            } else {
                2
            },
            project: ProjectIdentity {
                name: project_name(project),
                configuration: project.dba_configuration()?,
            },
            bootstrap,
            current_feature,
            features,
            planned_feature_candidates,
            feature_inventory_warnings,
        })
    }

    pub fn is_blocked(&self) -> bool {
        self.bootstrap.status != DisplayStatus::Complete
            || self
                .features
                .iter()
                .any(|feature| feature.workflow.status != DisplayStatus::Complete)
    }
}

fn acceptance_snapshot(project: &Project, subject: &str) -> Result<Option<AcceptedSnapshot>> {
    let Some(receipt) = ReceiptStore::at(&project.codeos_dir())
        .latest("feature", subject, "acceptance")?
        .filter(|receipt| receipt.result == "accepted")
    else {
        return Ok(None);
    };
    let bytes = serde_json::to_vec(&receipt)?;
    let semantic_revision = std::fs::read_to_string(project.intent(subject))
        .ok()
        .and_then(|text| yaml_u64(&text, "semantic_revision"));
    Ok(Some(AcceptedSnapshot {
        receipt_sha256: hex::encode(Sha256::digest(bytes)),
        accepted_at: receipt.timestamp.clone(),
        semantic_revision,
        candidate_bundle_id: receipt.bindings.get("candidate_bundle").cloned(),
        bindings: receipt.bindings,
    }))
}

fn yaml_u64(text: &str, key: &str) -> Option<u64> {
    let frontmatter = text.strip_prefix("---\n")?.split_once("\n---")?.0;
    let value: serde_yaml::Value = serde_yaml::from_str(frontmatter).ok()?;
    value.get(key)?.as_u64()
}

fn snapshot_completed_checkpoints(workflow: &mut WorkflowStatus) {
    for checkpoint in &mut workflow.checkpoints {
        if checkpoint.status != DisplayStatus::NotApplicable {
            checkpoint.status = DisplayStatus::Complete;
            checkpoint.basis = Some("ACCEPTED_SNAPSHOT".into());
        }
    }
    workflow.completed = workflow
        .checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.status == DisplayStatus::Complete)
        .count();
}

pub fn render(status: &ProjectStatus) -> String {
    let mut out = format!(
        "CODEOS — {}    {}\n\nPROJECT\n  Bootstrap          {}",
        status.project.name,
        status.project.configuration,
        checkpoint_line(&status.bootstrap)
    );
    let incomplete = status
        .features
        .iter()
        .filter(|feature| feature.workflow.status != DisplayStatus::Complete)
        .count();
    match status.current_feature.as_deref() {
        Some(id) => {
            let feature = status
                .features
                .iter()
                .find(|feature| feature.id == id)
                .unwrap();
            out.push_str(&format!("\n  Active feature     {} — {}", id, feature.name));
        }
        None if incomplete > 1 => out.push_str(&format!(
            "\n  Active feature     none derived — {incomplete} incomplete features"
        )),
        None => out.push_str("\n  Active feature     —"),
    }
    out.push_str("\n\nFEATURES\n");
    if status.features.is_empty() {
        out.push_str("  No Specification Packages found.\n");
    } else {
        for feature in &status.features {
            let current = if status.current_feature.as_deref() == Some(&feature.id) {
                "  ◀ CURRENT"
            } else {
                ""
            };
            out.push_str(&format!(
                "  {}  {}  {}  {}{}\n",
                feature.id,
                feature.name,
                feature
                    .workflow
                    .current_checkpoint
                    .as_deref()
                    .unwrap_or("—"),
                feature.workflow.status.label(),
                current
            ));
        }
    }
    for candidate in &status.planned_feature_candidates {
        out.push_str(&format!(
            "  —  {}  CANDIDATE (NOT STARTED)\n",
            candidate.name
        ));
    }
    if !status.feature_inventory_warnings.is_empty() {
        out.push_str("\nFEATURE INVENTORY WARNINGS\n");
        for warning in &status.feature_inventory_warnings {
            out.push_str(&format!("  {warning}\n"));
        }
    }

    if let Some(id) = status.current_feature.as_deref() {
        let feature = status
            .features
            .iter()
            .find(|feature| feature.id == id)
            .unwrap();
        out.push_str(&format!("\n{}\n", feature.id));
        for checkpoint in &feature.workflow.checkpoints {
            out.push_str(&format!(
                "  {:<4}{:<32} {}\n",
                checkpoint.id,
                checkpoint.title,
                checkpoint.status.label()
            ));
        }
        if let Some(issue) = feature.workflow.primary_issue.as_deref() {
            out.push_str(&format!(
                "\nCURRENT ISSUE\n  {}\n  {}\n",
                feature.workflow.status.label(),
                issue
            ));
            if feature.workflow.additional_issue_count > 0 {
                out.push_str(&format!(
                    "  Also: {} additional issue(s) at this checkpoint\n",
                    feature.workflow.additional_issue_count
                ));
            }
        }
        if let Some(action) = feature.workflow.next_action.as_deref() {
            out.push_str(&format!("\nNEXT ACTION\n  {action}\n"));
        }
    } else if status.bootstrap.status != DisplayStatus::Complete {
        render_issue(&mut out, &status.bootstrap);
    } else if incomplete > 1 {
        out.push_str("\nCURRENT ISSUES\n");
        for feature in status
            .features
            .iter()
            .filter(|feature| feature.workflow.status != DisplayStatus::Complete)
        {
            if let Some(issue) = feature.workflow.primary_issue.as_deref() {
                out.push_str(&format!("  {} — {}\n", feature.id, issue));
            }
        }
    }
    out.trim_end().to_string()
}

pub fn render_targeted_feature(feature: &FeatureStatus) -> String {
    let mut out = format!("workflow: feature   subject: {}\n", feature.id);
    if let Some(snapshot) = &feature.accepted_snapshot {
        out.push_str(&format!(
            "\nACCEPTED SNAPSHOT\n  receipt: {}\n  accepted_at: {}\n",
            snapshot.receipt_sha256, snapshot.accepted_at
        ));
    }
    if let Some(validity) = &feature.current_validity {
        out.push_str(&format!("CURRENT VALIDITY\n  {}\n", validity.status));
        for reason in &validity.reasons {
            out.push_str(&format!("  - {reason}\n"));
        }
    }
    for checkpoint in &feature.workflow.checkpoints {
        out.push_str(&format!(
            "\n{} — {}: {}{}\n",
            checkpoint.id,
            checkpoint.title,
            checkpoint.status.label(),
            checkpoint
                .basis
                .as_deref()
                .map(|b| format!(" ({b})"))
                .unwrap_or_default()
        ));
    }
    match feature.workflow.next_action.as_deref() {
        Some(action) => out.push_str(&format!("\nNEXT\n  {action}\n")),
        None => out.push_str("\nNEXT\n  nothing blocked — accepted snapshot is current\n"),
    }
    out.trim_end().to_string()
}

fn render_issue(out: &mut String, status: &WorkflowStatus) {
    if let Some(issue) = status.primary_issue.as_deref() {
        out.push_str(&format!(
            "\nCURRENT ISSUE\n  {}\n  {}\n",
            status.status.label(),
            issue
        ));
    }
    if let Some(action) = status.next_action.as_deref() {
        out.push_str(&format!("\nNEXT ACTION\n  {action}\n"));
    }
}

fn summarize(reports: &[CheckpointReport]) -> WorkflowStatus {
    let current = reports
        .iter()
        .find(|report| matches!(report.state, State::Blocked | State::Waiting));
    let status = current.map_or(DisplayStatus::Complete, |report| match report.state {
        State::Waiting => DisplayStatus::WaitingHuman,
        _ => DisplayStatus::Blocked,
    });
    let counted = reports
        .iter()
        .filter(|report| report.id != "bootstrap-entry");
    let completed = counted
        .clone()
        .filter(|report| report.state == State::Pass)
        .count();
    let total = counted
        .filter(|report| report.state != State::NotApplicable)
        .count();
    let primary_issue = current.and_then(|report| report.unmet.first().cloned());
    let additional_issue_count = current
        .map(|report| report.unmet.len().saturating_sub(1))
        .unwrap_or(0);

    WorkflowStatus {
        status,
        current_checkpoint: current.map(|report| report.id.clone()),
        completed,
        total,
        primary_issue,
        additional_issue_count,
        next_action: current.and_then(|report| report.next_action.clone()),
        checkpoints: reports
            .iter()
            .map(|report| CheckpointStatus {
                id: report.id.clone(),
                title: report.title.clone(),
                status: match report.state {
                    State::Pass => DisplayStatus::Complete,
                    State::Waiting => DisplayStatus::WaitingHuman,
                    State::Blocked => DisplayStatus::Blocked,
                    State::NotApplicable => DisplayStatus::NotApplicable,
                },
                basis: None,
            })
            .collect(),
    }
}

fn specification_feature_ids(project: &Project) -> Result<Vec<String>> {
    let specification = project.codeos_dir().join("01-specification");
    let mut ids = BTreeSet::new();
    collect_ids(&specification.join("intents"), ".md", &mut ids)?;
    collect_ids(&specification.join("contracts"), "_contract.md", &mut ids)?;
    collect_ids(&specification.join("event-schemas"), "_schema.md", &mut ids)?;
    Ok(ids.into_iter().collect())
}

fn planned_feature_candidates(
    project: &Project,
    started_names: &BTreeSet<String>,
) -> Result<(Vec<PlannedFeatureCandidate>, Vec<String>)> {
    let discovery = project.codeos_dir().join("00-discovery");
    if !discovery.is_dir() {
        return Ok((vec![], vec![]));
    }

    let mut paths = std::fs::read_dir(&discovery)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    paths.sort();

    let mut candidates = Vec::new();
    let mut warnings = Vec::new();
    for path in paths {
        let source = path
            .strip_prefix(&project.root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                warnings.push(format!(
                    "{source}: cannot read feature decomposition: {error}"
                ));
                continue;
            }
        };
        let (Some(frontmatter), body) = evidence::split_frontmatter(&text) else {
            continue;
        };
        let Ok(frontmatter) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) else {
            continue;
        };
        if frontmatter
            .get("artifact_type")
            .and_then(serde_yaml::Value::as_str)
            != Some("feature_decomposition")
        {
            continue;
        }

        match parse_candidate_table(body, &source) {
            Ok(rows) => {
                for candidate in rows {
                    let key = normalized_name(&candidate.name);
                    if started_names.contains(&key) {
                        continue;
                    }
                    candidates.push(candidate);
                }
            }
            Err(warning) => warnings.push(warning),
        }
    }
    Ok((candidates, warnings))
}

fn parse_candidate_table(
    body: &str,
    source: &str,
) -> std::result::Result<Vec<PlannedFeatureCandidate>, String> {
    let section = body
        .split_once("## Candidate Feature Boundaries")
        .map(|(_, rest)| rest.split("\n## ").next().unwrap_or(rest))
        .ok_or_else(|| format!("{source}: missing 'Candidate Feature Boundaries' section"))?;
    let mut lines = section
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let header = lines
        .find(|line| line.starts_with('|'))
        .ok_or_else(|| format!("{source}: missing candidate table"))?;
    if table_cells(header) != ["Candidate", "Human outcome", "Boundary or dependency"] {
        return Err(format!("{source}: candidate table has unexpected columns"));
    }
    let separator = lines
        .next()
        .ok_or_else(|| format!("{source}: candidate table has no separator"))?;
    if table_cells(separator).len() != 3
        || table_cells(separator).iter().any(|cell| {
            !cell
                .chars()
                .all(|ch| ch == '-' || ch == ':' || ch.is_whitespace())
        })
    {
        return Err(format!(
            "{source}: candidate table has an invalid separator"
        ));
    }

    let mut rows = Vec::new();
    for (index, line) in lines.filter(|line| line.starts_with('|')).enumerate() {
        let cells = table_cells(line);
        if cells.len() != 3 || cells.iter().any(|cell| cell.is_empty()) {
            return Err(format!(
                "{source}: candidate table row {} must contain three non-empty cells",
                index + 1
            ));
        }
        if cells[0].contains("[short name]") {
            continue;
        }
        rows.push(PlannedFeatureCandidate {
            name: cells[0].to_string(),
            human_outcome: cells[1].to_string(),
            boundary_or_dependency: cells[2].to_string(),
            source: source.to_string(),
        });
    }
    Ok(rows)
}

fn table_cells(line: &str) -> Vec<&str> {
    line.trim_matches('|').split('|').map(str::trim).collect()
}

fn normalized_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn collect_ids(dir: &Path, suffix: &str, ids: &mut BTreeSet<String>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if let Some(id) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(suffix))
        {
            ids.insert(id.to_string());
        }
    }
    Ok(())
}

fn project_name(project: &Project) -> String {
    heading_value(&project.charter(), "# Solution Charter:").unwrap_or_else(|| {
        project
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project")
            .to_string()
    })
}

fn feature_name(project: &Project, id: &str) -> String {
    heading_value(&project.intent(id), &format!("# Intent: {id} —"))
        .unwrap_or_else(|| id.to_string())
}

fn heading_value(path: &Path, prefix: &str) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()?
        .lines()
        .find_map(|line| line.trim().strip_prefix(prefix).map(str::trim))
        .filter(|value| !value.is_empty() && !value.contains('['))
        .map(str::to_owned)
}

fn checkpoint_line(status: &WorkflowStatus) -> String {
    let checkpoints = status
        .checkpoints
        .iter()
        .map(|checkpoint| {
            let marker = match checkpoint.status {
                DisplayStatus::Complete => "✓",
                DisplayStatus::NotApplicable => "–",
                DisplayStatus::WaitingHuman => "?",
                DisplayStatus::Blocked => "×",
            };
            format!("{} {marker}", checkpoint.id)
        })
        .collect::<Vec<_>>()
        .join("  ");
    format!("{checkpoints}  ({}/{})", status.completed, status.total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::CheckpointReport;
    use std::fs;

    fn report(id: &str, state: State, unmet: &[&str]) -> CheckpointReport {
        CheckpointReport {
            id: id.into(),
            title: format!("checkpoint {id}"),
            state,
            met: vec![],
            unmet: unmet.iter().map(|value| value.to_string()).collect(),
            next_action: (state != State::Pass).then(|| "take the next action".into()),
        }
    }

    #[test]
    fn summary_reuses_the_first_unresolved_report() {
        let summary = summarize(&[
            report("F1", State::Pass, &[]),
            report(
                "F2",
                State::Waiting,
                &["human decision absent", "approval absent"],
            ),
            report("F3", State::Blocked, &["earlier checkpoint F2 is not PASS"]),
            report("F4", State::NotApplicable, &[]),
        ]);

        assert_eq!(summary.status, DisplayStatus::WaitingHuman);
        assert_eq!(summary.current_checkpoint.as_deref(), Some("F2"));
        assert_eq!(
            summary.primary_issue.as_deref(),
            Some("human decision absent")
        );
        assert_eq!(summary.additional_issue_count, 1);
        assert_eq!((summary.completed, summary.total), (1, 3));
    }

    #[test]
    fn accepted_snapshot_keeps_historical_checkpoints_complete_when_current_state_is_stale() {
        let mut workflow = summarize(&[
            report("F1", State::Pass, &[]),
            report("F2", State::Blocked, &["current implementation drifted"]),
            report("F3", State::Blocked, &["earlier checkpoint F2 is not PASS"]),
        ]);

        snapshot_completed_checkpoints(&mut workflow);

        assert_eq!(workflow.completed, 3);
        assert!(workflow.checkpoints.iter().all(|checkpoint| {
            checkpoint.status == DisplayStatus::Complete
                && checkpoint.basis.as_deref() == Some("ACCEPTED_SNAPSHOT")
        }));
    }

    #[test]
    fn heading_parsing_falls_back_for_placeholders() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("intent.md");
        std::fs::write(&path, "# Intent: F-0001 — Useful name\n").unwrap();
        assert_eq!(
            heading_value(&path, "# Intent: F-0001 —").as_deref(),
            Some("Useful name")
        );
        std::fs::write(&path, "# Intent: F-0001 — [short name]\n").unwrap();
        assert_eq!(heading_value(&path, "# Intent: F-0001 —"), None);
    }

    #[test]
    fn project_projection_discovers_partial_packages_without_writing_state() {
        let dir = tempfile::tempdir().unwrap();
        let codeos = dir.path().join(".codeos");
        fs::create_dir_all(codeos.join("00-project")).unwrap();
        fs::create_dir_all(codeos.join("01-specification/intents")).unwrap();
        fs::write(
            codeos.join("01-specification/intents/F-0002.md"),
            "# Intent: F-0002 — Second feature\n",
        )
        .unwrap();
        fs::write(
            codeos.join("01-specification/intents/F-0001.md"),
            "# Intent: F-0001 — First feature\n",
        )
        .unwrap();
        let project = Project::discover(dir.path()).unwrap();

        let before = tree_contents(&codeos);
        let first = ProjectStatus::derive(&project).unwrap();
        let first_json = serde_json::to_string(&first).unwrap();
        let second_json = serde_json::to_string(&ProjectStatus::derive(&project).unwrap()).unwrap();

        assert_eq!(first_json, second_json);
        assert_eq!(before, tree_contents(&codeos));
        assert_eq!(first.current_feature, None);
        assert_eq!(
            first
                .features
                .iter()
                .map(|feature| feature.id.as_str())
                .collect::<Vec<_>>(),
            ["F-0001", "F-0002"]
        );
    }

    #[test]
    fn exactly_one_incomplete_feature_becomes_current() {
        let dir = tempfile::tempdir().unwrap();
        let codeos = dir.path().join(".codeos");
        fs::create_dir_all(codeos.join("00-project")).unwrap();
        fs::create_dir_all(codeos.join("01-specification/intents")).unwrap();
        fs::write(
            codeos.join("01-specification/intents/F-0001.md"),
            "# Intent: F-0001 — Only feature\n",
        )
        .unwrap();
        let status = ProjectStatus::derive(&Project::discover(dir.path()).unwrap()).unwrap();

        assert_eq!(status.current_feature.as_deref(), Some("F-0001"));
        assert!(render(&status).contains("F-0001 — Only feature"));
        assert!(render(&status).contains("CURRENT ISSUE"));
    }

    fn tree_contents(root: &Path) -> Vec<(String, Vec<u8>)> {
        fn visit(root: &Path, path: &Path, found: &mut Vec<(String, Vec<u8>)>) {
            let mut entries = fs::read_dir(path)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();
            entries.sort();
            for entry in entries {
                if entry.is_dir() {
                    visit(root, &entry, found);
                } else {
                    found.push((
                        entry.strip_prefix(root).unwrap().display().to_string(),
                        fs::read(entry).unwrap(),
                    ));
                }
            }
        }

        let mut found = Vec::new();
        visit(root, root, &mut found);
        found
    }
}
