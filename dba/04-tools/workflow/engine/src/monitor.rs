//! Read-only project status projection and terminal rendering.

use crate::checker::{self, Actor, CheckpointReport, State, Subject};
use crate::contract::{ActionType, ReasonCode, Workflow};
use crate::evidence;
use crate::project::Project;
use crate::receipts::ReceiptStore;
use crate::revalidation;
use anyhow::{Context, Result};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<Actor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<ReasonCode>,
    /// Exposed through attention only.
    #[serde(skip)]
    pub action: Option<ActionType>,
    /// Exposed through attention only.
    #[serde(skip)]
    pub surfaces: Option<checker::Surfaces>,
    /// The derived revalidation trigger an action's command binds to.
    #[serde(skip)]
    pub trigger: Option<String>,
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

/// One subject that is not complete, in workflow order: bootstrap, the current feature, accepted
/// features whose acceptance is no longer current, then other incomplete features by ID.
#[derive(Debug, Serialize)]
pub struct AttentionItem {
    pub subject: String,
    pub class: AttentionClass,
    pub checkpoint: Option<String>,
    pub issue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<ReasonCode>,
    /// The governing policy path, or `NOT_CONFIGURED` when the configuration selects none.
    pub authority: String,
    /// Surfaces the deciding comparison found changed; always present, possibly empty.
    pub affected_surfaces: Vec<crate::hashing::Comparison>,
    /// Surfaces the same comparison positively found unchanged; never inferred.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unchanged_surfaces: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<crate::hashing::EvidenceRef>,
    /// The checkpoint whose comparison produced the surfaces, when it is not `checkpoint`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surfaces_origin: Option<String>,
    pub next_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<Actor>,
}

/// Attention class: a presentation classification that never replaces a workflow state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttentionClass {
    /// An unmet condition prevents progression, including an actionable prerequisite.
    Blocker,
    /// A previously valid accepted state is no longer current.
    Stale,
    /// A required human disposition does not yet exist.
    Unresolved,
    /// Important information that does not block. No source emits it yet.
    Warning,
}

impl AttentionClass {
    pub fn label(self) -> &'static str {
        match self {
            Self::Blocker => "BLOCKER",
            Self::Stale => "STALE",
            Self::Unresolved => "UNRESOLVED",
            Self::Warning => "WARNING",
        }
    }
}

/// An Observation v2 identity without validated closure evidence.
#[derive(Debug, Serialize)]
pub struct OpenObservation {
    pub id: String,
    pub route: Option<String>,
    pub target: Option<String>,
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
    pub attention: Vec<AttentionItem>,
    pub open_observations: Vec<OpenObservation>,
    pub health: Vec<crate::health::Health>,
    /// Present only when requested with `status --history`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<crate::history::History>,
}

impl ProjectStatus {
    pub fn derive(project: &Project) -> Result<Self> {
        let bootstrap_subject = Subject::resolve(project, Workflow::Bootstrap, "solution")?;
        let bootstrap = summarize(&checker::evaluate(project, &bootstrap_subject)?);

        let mut features = Vec::new();
        for id in specification_feature_ids(project)? {
            let subject = Subject::resolve(project, Workflow::Feature, &id)?;
            let historically_accepted = crate::revalidation::historically_accepted(project, &id)?;
            // One assessment decides current validity and explains it.
            let assessment = if historically_accepted {
                Some(crate::revalidation::acceptance_assessment(project, &id)?)
            } else {
                None
            };
            let current_acceptance_valid = assessment.as_ref().is_some_and(|a| a.current);
            let governance_v3 = project.workflow_governance_version()? >= 3;
            let mut workflow = summarize(&checker::evaluate(project, &subject)?);
            let stale_reason = match &assessment {
                Some(assessment) if !assessment.current => Some(
                    assessment
                        .reason
                        .context("stale accepted feature has no reason")?,
                ),
                _ => None,
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
                (workflow.actor, workflow.reason, workflow.action) = stale_cause(reason);
                Some(reason.into())
            } else if current_acceptance_valid {
                workflow.status = DisplayStatus::Complete;
                workflow.current_checkpoint = None;
                workflow.primary_issue = None;
                workflow.additional_issue_count = 0;
                workflow.next_action = None;
                workflow.actor = None;
                workflow.reason = None;
                workflow.surfaces = None;
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
                    (workflow.actor, workflow.reason, workflow.action) = stale_cause(reason);
                    if reason == "REVALIDATION_REQUIRED" {
                        revalidation_step(
                            &mut workflow,
                            revalidation::next_step(project, &subject),
                        );
                    }
                }
            }
            if let (Some(assessment), Some(_)) = (assessment, stale_reason) {
                workflow.surfaces = Some(checker::Surfaces::new(
                    workflow.current_checkpoint.as_deref().unwrap_or_default(),
                    assessment.comparisons,
                    assessment.evidence,
                ));
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
        let attention = attention(project, &bootstrap, &features, current_feature.as_deref())?;
        let open_observations = open_observations(project)?;
        let health = crate::health::derive(project, &features, &open_observations)?;

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
            attention,
            open_observations,
            health,
            history: None,
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

/// Post-acceptance revalidation is mechanical; semantic or architecture change needs bounded work.
fn stale_cause(reason: &str) -> (Option<Actor>, Option<ReasonCode>, Option<ActionType>) {
    let (actor, code, action) = match reason {
        "SEMANTIC_CHANGE_REQUIRED" => (
            Actor::Assisted,
            ReasonCode::SemanticChangeRequired,
            ActionType::ReviseSpecification,
        ),
        "ARCHITECTURE_CHANGE_REQUIRED" => (
            Actor::Assisted,
            ReasonCode::ArchitectureChangeRequired,
            ActionType::ReassessArchitecture,
        ),
        // The atomic revalidation step is derived separately (`revalidation_step`).
        _ => {
            return (
                Some(Actor::Auto),
                Some(ReasonCode::RevalidationRequired),
                None,
            )
        }
    };
    (Some(actor), Some(code), Some(action))
}

/// Report the next atomic Governance v4 revalidation operation.
fn revalidation_step(workflow: &mut WorkflowStatus, step: revalidation::RevalidationStep) {
    use revalidation::RevalidationStep as S;
    let (actor, action, next, trigger) = match step {
        S::CommitChange => (
            Actor::Assisted,
            Some(ActionType::CommitChange),
            "commit the changed accepted surface so a revalidation trigger can be identified"
                .to_string(),
            None,
        ),
        S::RunVerification { trigger } => (
            Actor::Auto,
            Some(ActionType::RunVerification),
            format!("run the stale revalidation verifications for trigger {trigger}"),
            Some(trigger),
        ),
        S::PrepareCandidate { trigger } => (
            Actor::Auto,
            Some(ActionType::PrepareCandidate),
            format!(
                "prepare the post-acceptance revalidation Candidate Bundle for trigger {trigger}"
            ),
            Some(trigger),
        ),
        S::RunReview { bundle } => (
            Actor::Auto,
            Some(ActionType::RunReview),
            format!("assess revalidation Candidate Bundle {bundle} at Stage 8"),
            None,
        ),
        S::ResolveReviewFinding { bundle } => (
            Actor::Assisted,
            Some(ActionType::ResolveReviewFinding),
            format!("resolve the assessment of revalidation Candidate Bundle {bundle}"),
            None,
        ),
        S::Ineligible(why) => (
            Actor::Assisted,
            None,
            format!("revalidation cannot proceed: {why}"),
            None,
        ),
    };
    workflow.actor = Some(actor);
    workflow.action = action;
    workflow.next_action = Some(next);
    workflow.trigger = trigger;
}

/// The exact context an action arises in; a command depends on all of it, not the type alone.
pub struct ActionContext<'a> {
    pub action: ActionType,
    pub workflow: &'a str,
    pub subject: &'a str,
    /// The derived Governance v4 revalidation trigger, when the action is a revalidation step.
    pub trigger: Option<&'a str>,
}

/// A command only when executing it fully performs the action in this context. AUTO does not
/// imply a command.
pub fn action_command(context: &ActionContext) -> Option<String> {
    match (context.action, context.trigger) {
        (ActionType::RunVerification, None) => Some(format!(
            "codeos-workflow check --workflow {} --subject {}",
            context.workflow, context.subject
        )),
        (ActionType::RunVerification, Some(trigger)) => Some(format!(
            "codeos-workflow check --workflow {} --subject {} --revalidation --trigger {trigger}",
            context.workflow, context.subject
        )),
        // An ordinary candidate may need dynamic `--include` inputs; a revalidation candidate's
        // command is emitted only after `candidate::assemble` succeeded without them.
        (ActionType::PrepareCandidate, Some(trigger)) => Some(format!(
            "codeos-workflow prepare-candidate --subject {} --revalidation --trigger {trigger}",
            context.subject
        )),
        _ => None,
    }
}

/// The atomic next operation of an attention item; its subject, reason, actor, and authority
/// are the item's sibling fields.
#[derive(Debug, Serialize)]
pub struct Action {
    #[serde(rename = "type")]
    pub kind: ActionType,
    pub command: Option<String>,
}

/// The policy that governs a subject: its path, `NOT_CONFIGURED` when none is selected, or an
/// error when the selected policy is missing from the toolkit.
fn authority(project: &Project, key: &str) -> Result<String> {
    let Some(path) = project.selected_policy(key)? else {
        return Ok("NOT_CONFIGURED".into());
    };
    anyhow::ensure!(
        project.toolkit_root().join(&path).is_file(),
        "configuration selects {key} {path}, which is missing from the toolkit"
    );
    Ok(path)
}

fn attention_class(stale: bool, status: DisplayStatus) -> AttentionClass {
    match (stale, status) {
        (true, _) => AttentionClass::Stale,
        (false, DisplayStatus::WaitingHuman) => AttentionClass::Unresolved,
        (false, DisplayStatus::Blocked) => AttentionClass::Blocker,
        (false, DisplayStatus::Complete | DisplayStatus::NotApplicable) => {
            unreachable!("attention holds only subjects that are not complete")
        }
    }
}

fn attention(
    project: &Project,
    bootstrap: &WorkflowStatus,
    features: &[FeatureStatus],
    current_feature: Option<&str>,
) -> Result<Vec<AttentionItem>> {
    let item = |subject: &str, workflow: &WorkflowStatus, stale: bool, key: &str| {
        let surfaces = workflow.surfaces.clone();
        let flow = if subject == "bootstrap" {
            "bootstrap"
        } else {
            "feature"
        };
        let action = workflow.action.map(|kind| Action {
            kind,
            command: action_command(&ActionContext {
                action: kind,
                workflow: flow,
                subject: if subject == "bootstrap" {
                    "solution"
                } else {
                    subject
                },
                trigger: workflow.trigger.as_deref(),
            }),
        });
        Ok::<_, anyhow::Error>(AttentionItem {
            subject: subject.into(),
            class: attention_class(stale, workflow.status),
            checkpoint: workflow.current_checkpoint.clone(),
            issue: workflow.primary_issue.clone(),
            reason: workflow.reason,
            authority: authority(project, key)?,
            surfaces_origin: surfaces
                .as_ref()
                .map(|s| s.origin.clone())
                .filter(|origin| Some(origin) != workflow.current_checkpoint.as_ref()),
            affected_surfaces: surfaces
                .as_ref()
                .map(|s| s.changed.clone())
                .unwrap_or_default(),
            unchanged_surfaces: surfaces
                .as_ref()
                .map(|s| s.unchanged.clone())
                .unwrap_or_default(),
            evidence: surfaces.map(|s| s.evidence).unwrap_or_default(),
            next_action: workflow.next_action.clone(),
            action,
            actor: workflow.actor,
        })
    };
    let mut items = Vec::new();
    if bootstrap.status != DisplayStatus::Complete {
        items.push(item(
            "bootstrap",
            bootstrap,
            false,
            "workflow_bootstrap_policy",
        )?);
    }
    let mut incomplete: Vec<_> = features
        .iter()
        .filter(|feature| feature.workflow.status != DisplayStatus::Complete)
        .collect();
    // Features are already in ID order; the stable sort keeps it within each rank.
    incomplete.sort_by_key(|feature| {
        if current_feature == Some(feature.id.as_str()) {
            0
        } else if feature.historically_accepted && !feature.current_acceptance_valid {
            1
        } else {
            2
        }
    });
    for feature in incomplete {
        let stale = feature.historically_accepted && !feature.current_acceptance_valid;
        // Workflow governance owns an accepted snapshot's current validity.
        let key = if stale {
            "workflow_governance_policy"
        } else {
            "workflow_feature_policy"
        };
        items.push(item(&feature.id, &feature.workflow, stale, key)?);
    }
    Ok(items)
}

fn open_observations(project: &Project) -> Result<Vec<OpenObservation>> {
    let store = crate::observations::ObservationStore::at(&project.codeos_dir());
    let mut open = Vec::new();
    for observation in store.observations()? {
        if store.closure(&observation.id)?.is_some() {
            continue;
        }
        let route = store.route(&observation.id)?;
        open.push(OpenObservation {
            id: observation.id,
            target: route.as_ref().and_then(|route| {
                route.target_id.as_ref().map(|id| match &route.target_type {
                    Some(kind) => format!("{kind} {id}"),
                    None => id.clone(),
                })
            }),
            route: route.map(|route| route.route),
        });
    }
    Ok(open)
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
    let (stale, not_accepted) = status
        .features
        .iter()
        .filter(|feature| feature.workflow.status != DisplayStatus::Complete)
        .fold((0, 0), |(stale, not_accepted), feature| {
            if feature.historically_accepted {
                (stale + 1, not_accepted)
            } else {
                (stale, not_accepted + 1)
            }
        });
    match status.current_feature.as_deref() {
        Some(id) => {
            let feature = status
                .features
                .iter()
                .find(|feature| feature.id == id)
                .unwrap();
            out.push_str(&format!("\n  Active feature     {} — {}", id, feature.name));
        }
        None if stale + not_accepted > 1 => {
            let parts = [
                (not_accepted, "not yet accepted"),
                (stale, "accepted, currently STALE"),
            ]
            .into_iter()
            .filter(|(count, _)| *count > 0)
            .map(|(count, label)| format!("{count} {label}"))
            .collect::<Vec<_>>();
            out.push_str(&format!(
                "\n  Active feature     none derived — {}",
                parts.join("; ")
            ));
        }
        None => out.push_str("\n  Active feature     —"),
    }
    out.push_str("\n\nENGINEERING HEALTH\n");
    for item in &status.health {
        let shown = match item.status {
            crate::health::HealthStatus::Open => format!("{} OPEN", item.basis.len()),
            status => status.label().into(),
        };
        // The basis is shown verbatim and in order: inline when single, indented otherwise.
        match item.basis.as_slice() {
            [] => out.push_str(&format!("  {:<22}{shown}\n", item.domain.label())),
            [basis] => out.push_str(&format!("  {:<22}{shown} — {basis}\n", item.domain.label())),
            bases => {
                out.push_str(&format!("  {:<22}{shown}\n", item.domain.label()));
                for basis in bases {
                    out.push_str(&format!("    {basis}\n"));
                }
            }
        }
    }
    out.push_str("\nFEATURES\n");
    if status.features.is_empty() {
        out.push_str("  No Specification Packages found.\n");
    } else {
        for feature in &status.features {
            let current = if status.current_feature.as_deref() == Some(&feature.id) {
                "  ◀ CURRENT"
            } else {
                ""
            };
            // Historical acceptance and current validity, shown separately (C-3). The date is
            // the one recorded in `accepted_at`, without timezone conversion.
            let acceptance = match (&feature.accepted_snapshot, &feature.current_validity) {
                (Some(snapshot), Some(validity)) if feature.historically_accepted => {
                    let reasons = if validity.reasons.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", validity.reasons.join(", "))
                    };
                    format!(
                        "  accepted snapshot {} · current validity {}{reasons}",
                        snapshot
                            .accepted_at
                            .get(..10)
                            .unwrap_or(&snapshot.accepted_at),
                        validity.status
                    )
                }
                _ => String::new(),
            };
            out.push_str(&format!(
                "  {}  {}  {}  {}{}{}\n",
                feature.id,
                feature.name,
                feature
                    .workflow
                    .current_checkpoint
                    .as_deref()
                    .unwrap_or("—"),
                feature.workflow.status.label(),
                acceptance,
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
        render_next_action(&mut out, &feature.workflow);
    } else if status.attention.len() > 1 {
        out.push_str("\nATTENTION\n");
        for item in &status.attention {
            if let Some(issue) = item.issue.as_deref() {
                let reason = item
                    .reason
                    .and_then(|reason| serde_json::to_value(reason).ok())
                    .and_then(|value| value.as_str().map(str::to_string))
                    .filter(|code| !issue.contains(code.as_str()))
                    .map(|code| format!(" ({code})"))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "  [{}] {} — {issue}{reason}\n",
                    item.class.label(),
                    item.subject
                ));
            }
            if !item.affected_surfaces.is_empty() {
                let names: Vec<_> = item
                    .affected_surfaces
                    .iter()
                    .map(|c| c.surface.as_str())
                    .collect();
                let mut line = format!("    changed: {}", abbreviated(&names));
                if !item.unchanged_surfaces.is_empty() {
                    let names: Vec<_> =
                        item.unchanged_surfaces.iter().map(String::as_str).collect();
                    line.push_str(&format!("; unchanged: {}", abbreviated(&names)));
                }
                out.push_str(&line);
                out.push('\n');
            }
            if let Some(action) = item.next_action.as_deref() {
                out.push_str(&format!("    next{}: {action}\n", actor_tag(item.actor)));
            }
        }
    } else if status.bootstrap.status != DisplayStatus::Complete {
        render_issue(&mut out, &status.bootstrap);
    }
    if !status.open_observations.is_empty() {
        out.push_str("\nOPEN OBSERVATIONS\n");
        for observation in &status.open_observations {
            out.push_str(&format!(
                "  {}  {}\n",
                observation.id,
                match (&observation.route, &observation.target) {
                    (Some(route), Some(target)) => format!("routed: {route} → {target}"),
                    (Some(route), None) => format!("routed: {route}"),
                    _ => "not routed".into(),
                }
            ));
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
        Some(action) => out.push_str(&format!(
            "\nNEXT{}\n  {action}\n",
            actor_tag(feature.workflow.actor)
        )),
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
    render_next_action(out, status);
}

fn render_next_action(out: &mut String, status: &WorkflowStatus) {
    if let Some(action) = status.next_action.as_deref() {
        out.push_str(&format!(
            "\nNEXT ACTION{}\n  {action}\n",
            actor_tag(status.actor)
        ));
    }
}

/// Up to three names, then a count of the rest; JSON keeps the full list.
fn abbreviated(names: &[&str]) -> String {
    let shown = names.iter().take(3).copied().collect::<Vec<_>>().join(", ");
    match names.len() {
        n if n > 3 => format!("{shown} (+{})", n - 3),
        _ => shown,
    }
}

fn actor_tag(actor: Option<Actor>) -> String {
    actor
        .map(|actor| format!(" [{}]", actor.label()))
        .unwrap_or_default()
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
        actor: current.and_then(|report| report.actor),
        reason: current.and_then(|report| report.reason),
        action: current.and_then(|report| report.action),
        surfaces: current.and_then(|report| report.surfaces.clone()),
        trigger: None,
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
            actor: None,
            reason: None,
            action: None,
            surfaces: None,
        }
    }

    #[test]
    fn waiting_maps_to_unresolved_without_changing_workflow_status() {
        let summary = summarize(&[report("F4", State::Waiting, &["no decision"])]);
        assert_eq!(
            attention_class(false, summary.status),
            AttentionClass::Unresolved
        );
        assert_eq!(summary.status, DisplayStatus::WaitingHuman);
        assert_eq!(
            attention_class(false, DisplayStatus::Blocked),
            AttentionClass::Blocker
        );
        assert_eq!(
            attention_class(true, DisplayStatus::WaitingHuman),
            AttentionClass::Stale
        );
    }

    #[test]
    fn attention_vocabularies_are_pinned() {
        let names = |values: Vec<serde_json::Value>| {
            values
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        };
        use AttentionClass::*;
        assert_eq!(
            names(
                [Blocker, Stale, Unresolved, Warning]
                    .iter()
                    .map(|v| serde_json::to_value(v).unwrap())
                    .collect()
            ),
            ["BLOCKER", "STALE", "UNRESOLVED", "WARNING"]
        );
        use ReasonCode as R;
        assert_eq!(
            names(
                [
                    R::CharterApproval,
                    R::SpecificationApproval,
                    R::SemanticRevision,
                    R::Configuration,
                    R::ContractSection,
                    R::JourneyContribution,
                    R::Decision,
                    R::IndependentReview,
                    R::Verification,
                    R::ProductionEvidence,
                    R::ObservationStatement,
                    R::ObservationRoute,
                    R::ObservationResolution,
                    R::BootstrapIncomplete,
                    R::SemanticChangeRequired,
                    R::ArchitectureChangeRequired,
                    R::RevalidationRequired,
                ]
                .iter()
                .map(|v| serde_json::to_value(v).unwrap())
                .collect()
            ),
            [
                "CHARTER_APPROVAL",
                "SPECIFICATION_APPROVAL",
                "SEMANTIC_REVISION",
                "CONFIGURATION",
                "CONTRACT_SECTION",
                "JOURNEY_CONTRIBUTION",
                "DECISION",
                "INDEPENDENT_REVIEW",
                "VERIFICATION",
                "PRODUCTION_EVIDENCE",
                "OBSERVATION_STATEMENT",
                "OBSERVATION_ROUTE",
                "OBSERVATION_RESOLUTION",
                "BOOTSTRAP_INCOMPLETE",
                "SEMANTIC_CHANGE_REQUIRED",
                "ARCHITECTURE_CHANGE_REQUIRED",
                "REVALIDATION_REQUIRED",
            ]
        );
    }

    #[test]
    fn action_vocabulary_is_pinned_and_only_run_verification_has_a_command() {
        use ActionType as A;
        let all = [
            A::AuthorArtifact,
            A::RepairConfiguration,
            A::ReviseSpecification,
            A::ApproveArtifact,
            A::RecordDecision,
            A::FollowDecisionOutcome,
            A::RunVerification,
            A::CollectProductionEvidence,
            A::RepairFailure,
            A::PrepareCandidate,
            A::RunReview,
            A::ResolveReviewFinding,
            A::RestoreReviewHistory,
            A::ReassessArchitecture,
            A::RevalidateAcceptance,
            A::RecordObservation,
            A::RouteObservation,
            A::CloseObservation,
            A::ResolveRoutedTarget,
            A::CommitChange,
        ];
        assert_eq!(
            all.iter()
                .map(|a| serde_json::to_value(a)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string())
                .collect::<Vec<_>>(),
            [
                "author_artifact",
                "repair_configuration",
                "revise_specification",
                "approve_artifact",
                "record_decision",
                "follow_decision_outcome",
                "run_verification",
                "collect_production_evidence",
                "repair_failure",
                "prepare_candidate",
                "run_review",
                "resolve_review_finding",
                "restore_review_history",
                "reassess_architecture",
                "revalidate_acceptance",
                "record_observation",
                "route_observation",
                "close_observation",
                "resolve_routed_target",
                "commit_change",
            ]
        );
        for action in all {
            let command = action_command(&ActionContext {
                action,
                workflow: "feature",
                subject: "F-0001",
                trigger: None,
            });
            if action == A::RunVerification {
                assert_eq!(
                    command.as_deref(),
                    Some("codeos-workflow check --workflow feature --subject F-0001")
                );
            } else {
                assert_eq!(command, None, "{action:?}");
            }
        }
    }

    #[test]
    fn auto_does_not_imply_a_command() {
        let mut workflow = summarize(&[]);
        revalidation_step(
            &mut workflow,
            revalidation::RevalidationStep::RunReview { bundle: "b".into() },
        );
        assert_eq!(workflow.actor, Some(Actor::Auto));
        assert_eq!(workflow.action, Some(ActionType::RunReview));
        assert_eq!(
            action_command(&ActionContext {
                action: ActionType::RunReview,
                workflow: "feature",
                subject: "F-0002",
                trigger: workflow.trigger.as_deref(),
            }),
            None
        );
    }

    #[test]
    fn revalidation_is_reported_as_atomic_steps_with_exact_commands() {
        use revalidation::RevalidationStep as S;
        // The non-atomic outcome type is reserved, never emitted.
        assert_eq!(stale_cause("REVALIDATION_REQUIRED").2, None);
        let sha = "c87c46d46b85a7b5ad3b4c5417e9c0f82adf16c7";
        for (step, actor, action, command) in [
            (S::CommitChange, Actor::Assisted, Some(ActionType::CommitChange), None),
            (
                S::RunVerification { trigger: sha.into() },
                Actor::Auto,
                Some(ActionType::RunVerification),
                Some(format!("codeos-workflow check --workflow feature --subject F-0002 --revalidation --trigger {sha}")),
            ),
            (
                S::PrepareCandidate { trigger: sha.into() },
                Actor::Auto,
                Some(ActionType::PrepareCandidate),
                Some(format!("codeos-workflow prepare-candidate --subject F-0002 --revalidation --trigger {sha}")),
            ),
            (
                S::RunReview { bundle: "b".into() },
                Actor::Auto,
                Some(ActionType::RunReview),
                None,
            ),
            (
                S::ResolveReviewFinding { bundle: "b".into() },
                Actor::Assisted,
                Some(ActionType::ResolveReviewFinding),
                None,
            ),
            (S::Ineligible("refused".into()), Actor::Assisted, None, None),
        ] {
            let mut workflow = summarize(&[]);
            revalidation_step(&mut workflow, step);
            assert_eq!(workflow.actor, Some(actor));
            assert_eq!(workflow.action, action);
            assert_ne!(workflow.action, Some(ActionType::RevalidateAcceptance));
            let emitted = workflow.action.and_then(|action| {
                action_command(&ActionContext {
                    action,
                    workflow: "feature",
                    subject: "F-0002",
                    trigger: workflow.trigger.as_deref(),
                })
            });
            assert_eq!(emitted, command, "{action:?}");
        }
    }

    #[test]
    fn a_selected_policy_missing_from_the_toolkit_is_an_error_not_an_absence() {
        let dir = tempfile::tempdir().unwrap();
        let resolver = dir
            .path()
            .join(".codeos/toolkit/dba/04-tools/configuration/resolve-project-configuration.sh");
        fs::create_dir_all(resolver.parent().unwrap()).unwrap();
        fs::write(
            &resolver,
            "printf 'dba_configuration=DBA-X\\nworkflow_feature_policy=dba/02-policies/missing.md\\n'\n",
        )
        .unwrap();
        let project = Project {
            root: dir.path().to_path_buf(),
            self_dev: false,
        };
        let error = authority(&project, "workflow_feature_policy")
            .unwrap_err()
            .to_string();
        assert!(error.contains("dba/02-policies/missing.md"), "{error}");
        assert_eq!(
            authority(&project, "workflow_bootstrap_policy").unwrap(),
            "NOT_CONFIGURED"
        );
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
