//! Read-only project status projection and terminal rendering.

use crate::checker::{self, CheckpointReport, State, Subject};
use crate::contract::Workflow;
use crate::project::Project;
use anyhow::Result;
use serde::Serialize;
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
    pub acceptance_reason: Option<String>,
    #[serde(flatten)]
    pub workflow: WorkflowStatus,
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
            let mut workflow = summarize(&checker::evaluate(project, &subject)?);
            let acceptance_reason = if historically_accepted && !current_acceptance_valid {
                workflow.status = DisplayStatus::Blocked;
                workflow.current_checkpoint = Some("POST_ACCEPTANCE_REVALIDATION".into());
                workflow.primary_issue = Some("historical F9 acceptance is preserved, but current acceptance is stale after a later implementation/dependency change".into());
                workflow.additional_issue_count = 0;
                workflow.next_action = Some(
                    "run one bounded POST_ACCEPTANCE_REVALIDATION for the identified trigger"
                        .into(),
                );
                Some("REVALIDATION_REQUIRED".into())
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
            features.push(FeatureStatus {
                name: feature_name(project, &id),
                id,
                historically_accepted,
                current_acceptance_valid,
                acceptance_reason,
                workflow,
            });
        }

        let incomplete: Vec<_> = features
            .iter()
            .filter(|feature| feature.workflow.status != DisplayStatus::Complete)
            .map(|feature| feature.id.clone())
            .collect();
        let current_feature = (incomplete.len() == 1).then(|| incomplete[0].clone());

        Ok(Self {
            schema_version: 1,
            project: ProjectIdentity {
                name: project_name(project),
                configuration: project.dba_configuration()?,
            },
            bootstrap,
            current_feature,
            features,
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
