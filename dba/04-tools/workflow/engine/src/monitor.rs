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
    /// Workflow Governance v5 retired features; absent before v5 or when none exist.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub retired_features: Vec<crate::retirement::RetiredFeature>,
    pub health: Vec<crate::health::Health>,
    pub platform_assurance: Vec<crate::platform_assurance::PlatformAssurance>,
    /// Present only when requested with `status --history`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<crate::history::History>,
}

impl ProjectStatus {
    pub fn derive(project: &Project) -> Result<Self> {
        let bootstrap_subject = Subject::resolve(project, Workflow::Bootstrap, "solution")?;
        let bootstrap = summarize(&checker::evaluate(project, &bootstrap_subject)?);

        let mut features = Vec::new();
        let specified = specification_feature_ids(project)?;
        let (retired_features, retirement_violations) =
            crate::retirement::derive(project, &specified.iter().cloned().collect())?;
        for id in specified {
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
            // The checker's own step for an ordinary candidate already under way.
            let ordinary_step = (workflow.action, workflow.next_action.clone());
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
                    "UPSTREAM_REVALIDATION_REQUIRED" => "historical F9 acceptance is preserved, but an inherited claim's upstream feature is not valid".into(),
                    _ => "historical F9 acceptance is preserved, but current acceptance is stale after a later implementation/dependency change".into(),
                });
                workflow.additional_issue_count = 0;
                workflow.next_action = Some(match reason {
                    "SEMANTIC_CHANGE_REQUIRED" => "start ordinary governed work at F1".into(),
                    "ARCHITECTURE_CHANGE_REQUIRED" => {
                        "complete architecture impact and use an ordinary Candidate Bundle".into()
                    }
                    "UPSTREAM_REVALIDATION_REQUIRED" => {
                        "restore the upstream feature's validity; this feature then follows it"
                            .into()
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
                        "UPSTREAM_REVALIDATION_REQUIRED" => {
                            "restore the upstream feature's validity; this feature then follows it"
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
                    if reason == "ARCHITECTURE_CHANGE_REQUIRED" {
                        ordinary_candidate_step(project, &id, &mut workflow, ordinary_step)?;
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
        let mut attention = attention(project, &bootstrap, &features, current_feature.as_deref())?;
        for violation in retirement_violations {
            attention.push(AttentionItem {
                subject: violation.feature,
                class: AttentionClass::Blocker,
                checkpoint: Some("RETIREMENT".into()),
                issue: Some(violation.issue),
                reason: None,
                authority: authority(project, "workflow_governance_policy")?,
                affected_surfaces: Vec::new(),
                unchanged_surfaces: Vec::new(),
                evidence: Vec::new(),
                surfaces_origin: None,
                next_action: Some(violation.next_action),
                action: None,
                actor: Some(Actor::Assisted),
            });
        }
        let open_observations = open_observations(project)?;
        let health = crate::health::derive(project, &features, &open_observations)?;
        let platform_assurance = crate::platform_assurance::derive(project, &features)?;

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
            retired_features,
            health,
            platform_assurance,
            history: None,
        })
    }

    pub fn is_blocked(&self) -> bool {
        self.bootstrap.status != DisplayStatus::Complete
            || self
                .attention
                .iter()
                .any(|item| item.checkpoint.as_deref() == Some("RETIREMENT"))
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

/// After an architecture change, report the checker's F7 step once a current ordinary Candidate
/// Bundle exists; until then, architecture reassessment stays the next step.
fn ordinary_candidate_step(
    project: &Project,
    feature: &str,
    workflow: &mut WorkflowStatus,
    (step, checker_next): (Option<ActionType>, Option<String>),
) -> Result<()> {
    let Some(bundle) = crate::candidate::unique_current(project, feature)? else {
        return Ok(());
    };
    if crate::candidate::CandidateStore::at(&project.codeos_dir())
        .read(&bundle)?
        .kind
        != "ordinary"
    {
        return Ok(());
    }
    let (actor, next) = match step {
        Some(ActionType::RunReview) => (
            Actor::Auto,
            format!("assess ordinary Candidate Bundle {bundle} at Stage 8"),
        ),
        Some(ActionType::ResolveReviewFinding) => (
            Actor::Assisted,
            format!("resolve the assessment of ordinary Candidate Bundle {bundle}"),
        ),
        // F8 and F9 after a PASSED assessment: the checker names the decision.
        Some(ActionType::RecordDecision) => match checker_next {
            Some(next) => (
                Actor::Human,
                format!("{next} for ordinary Candidate Bundle {bundle}"),
            ),
            None => return Ok(()),
        },
        _ => return Ok(()),
    };
    workflow.actor = Some(actor);
    workflow.action = step;
    workflow.next_action = Some(next);
    Ok(())
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

/// The terminal width for `status`: `COLUMNS` when it holds a usable width, otherwise 100.
fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|width| *width >= 40)
        .unwrap_or(100)
}

pub fn render(status: &ProjectStatus) -> String {
    render_width(status, terminal_width())
}

/// The compact project report: condition first, then exceptions, then supporting detail. It
/// presents the same derivation as `status --json`; only layout and grouping differ.
pub fn render_width(status: &ProjectStatus, width: usize) -> String {
    let health = |domain| status.health.iter().find(|item| item.domain == domain);
    let mut header = format!(
        "CODEOS — {} | {}",
        status.project.name, status.project.configuration
    );
    if let Some(release) = health(crate::health::Domain::Release) {
        header.push_str(&format!(" | {}", release.status.label()));
    }
    let mut out = format!("{header}\n\nSUMMARY\n");

    let (valid, stale, unaccepted) =
        status
            .features
            .iter()
            .fold((0, 0, 0), |(valid, stale, unaccepted), feature| {
                match (
                    feature.historically_accepted,
                    feature.current_acceptance_valid,
                ) {
                    (true, true) => (valid + 1, stale, unaccepted),
                    (true, false) => (valid, stale + 1, unaccepted),
                    (false, _) => (valid, stale, unaccepted + 1),
                }
            });
    let features = [
        (valid, "VALID"),
        (stale, "STALE"),
        (unaccepted, "not yet accepted"),
        (status.planned_feature_candidates.len(), "CANDIDATE"),
    ]
    .into_iter()
    .filter(|(count, _)| *count > 0)
    .map(|(count, label)| format!("{count} {label}"))
    .collect::<Vec<_>>();
    let mut classes: Vec<(&str, usize)> = Vec::new();
    for item in &status.attention {
        match classes
            .iter_mut()
            .find(|(label, _)| *label == item.class.label())
        {
            Some((_, count)) => *count += 1,
            None => classes.push((item.class.label(), 1)),
        }
    }
    let attention = match status.attention.len() {
        0 => "none".to_string(),
        n => format!(
            "{n} ({})",
            classes
                .iter()
                .map(|(label, count)| format!("{count} {label}"))
                .collect::<Vec<_>>()
                .join(" · ")
        ),
    };
    let active = match status.current_feature.as_deref() {
        Some(id) => {
            let feature = status
                .features
                .iter()
                .find(|feature| feature.id == id)
                .unwrap();
            format!("{id} — {}", feature.name)
        }
        None if stale + incomplete_unaccepted(status) > 1 => {
            let parts = [
                (incomplete_unaccepted(status), "not yet accepted"),
                (stale, "accepted, currently STALE"),
            ]
            .into_iter()
            .filter(|(count, _)| *count > 0)
            .map(|(count, label)| format!("{count} {label}"))
            .collect::<Vec<_>>();
            format!("none derived — {}", parts.join("; "))
        }
        None => "—".into(),
    };
    table(
        &mut out,
        &[("", false), ("", true)],
        &[
            vec!["Bootstrap".into(), bootstrap_summary(&status.bootstrap)],
            vec![
                "Features".into(),
                if features.is_empty() {
                    "none".into()
                } else {
                    features.join(" · ")
                },
            ],
            vec!["Attention".into(), attention],
            vec!["Active feature".into(), active],
        ],
        width,
        false,
    );

    out.push_str("\nHEALTH\n");
    let mut shown: Vec<(&str, &Vec<String>)> = Vec::new();
    let rows = status
        .health
        .iter()
        .map(|item| {
            let state = match item.status {
                crate::health::HealthStatus::Open => format!("{} OPEN", item.basis.len()),
                status => status.label().into(),
            };
            // A basis identical to an earlier domain's is referenced, not repeated.
            let basis = match shown
                .iter()
                .find(|(_, bases)| !item.basis.is_empty() && **bases == item.basis)
            {
                Some((domain, _)) => format!("same as {domain}"),
                None => grouped(&item.basis).join("\n"),
            };
            shown.push((item.domain.label(), &item.basis));
            vec![item.domain.label().to_string(), state, basis]
        })
        .collect::<Vec<_>>();
    table(
        &mut out,
        &[("DOMAIN", false), ("STATE", false), ("BASIS", true)],
        &rows,
        width,
        true,
    );

    out.push_str("\nPLATFORM ASSURANCE — DEVELOPMENT EVIDENCE\n");
    let rows = status
        .platform_assurance
        .iter()
        .map(|item| {
            vec![
                item.area.platform_label().to_string(),
                item.status.label().to_string(),
                item.compact_summary(),
            ]
        })
        .collect::<Vec<_>>();
    table(
        &mut out,
        &[("AREA", false), ("STATE", false), ("BASIS", true)],
        &rows,
        width,
        true,
    );

    out.push_str("\nFEATURES\n");
    if status.features.is_empty() && status.planned_feature_candidates.is_empty() {
        out.push_str("  No Specification Packages found.\n");
    } else {
        let mut rows = status
            .features
            .iter()
            .map(|feature| {
                let mut name = feature.name.clone();
                if status.current_feature.as_deref() == Some(&feature.id) {
                    name.push_str(" (current)");
                }
                // Historical acceptance and current validity, shown separately (C-3). The date
                // is the one recorded in `accepted_at`, without timezone conversion.
                let (accepted, validity) =
                    match (&feature.accepted_snapshot, &feature.current_validity) {
                        (Some(snapshot), Some(validity)) if feature.historically_accepted => (
                            snapshot
                                .accepted_at
                                .get(..10)
                                .unwrap_or(&snapshot.accepted_at)
                                .to_string(),
                            if validity.reasons.is_empty() {
                                validity.status.clone()
                            } else {
                                format!("{} ({})", validity.status, validity.reasons.join(", "))
                            },
                        ),
                        _ => ("—".into(), "—".into()),
                    };
                // An incomplete feature's checkpoint is shown once, in ATTENTION.
                vec![
                    feature.id.clone(),
                    name,
                    feature.workflow.status.label().into(),
                    accepted,
                    validity,
                ]
            })
            .collect::<Vec<_>>();
        rows.extend(status.planned_feature_candidates.iter().map(|candidate| {
            vec![
                "—".into(),
                candidate.name.clone(),
                "CANDIDATE (NOT STARTED)".into(),
                "—".into(),
                "—".into(),
            ]
        }));
        table(
            &mut out,
            &[
                ("ID", false),
                ("NAME", true),
                ("STATE", true),
                ("ACCEPTED", false),
                ("VALIDITY", true),
            ],
            &rows,
            width,
            true,
        );
    }
    if !status.feature_inventory_warnings.is_empty() {
        out.push_str("\nFEATURE INVENTORY WARNINGS\n");
        for warning in &status.feature_inventory_warnings {
            out.push_str(&format!("  {warning}\n"));
        }
    }

    if !status.attention.is_empty() {
        out.push_str("\nATTENTION\n");
        let rows = status
            .attention
            .iter()
            .map(|item| {
                let changed: Vec<_> = item
                    .affected_surfaces
                    .iter()
                    .map(|c| c.surface.as_str())
                    .collect();
                vec![
                    item.subject.clone(),
                    item.class.label().into(),
                    item.checkpoint.clone().unwrap_or_else(|| "—".into()),
                    item.actor.map_or("—", |actor| actor.label()).into(),
                    if changed.is_empty() {
                        "—".into()
                    } else {
                        abbreviated(&changed)
                    },
                ]
            })
            .collect::<Vec<_>>();
        table(
            &mut out,
            &[
                ("SUBJECT", false),
                ("CLASS", false),
                ("POINT", false),
                ("ACTOR", false),
                ("CHANGED", true),
            ],
            &rows,
            width,
            true,
        );

        // Each fact once, with every subject it applies to.
        let mut facts: Vec<(&str, String, Vec<&str>)> = Vec::new();
        for item in &status.attention {
            let mut push = |field: &'static str, value: String| match facts
                .iter_mut()
                .find(|(f, v, _)| *f == field && *v == value)
            {
                Some((_, _, subjects)) => subjects.push(item.subject.as_str()),
                None => facts.push((field, value, vec![item.subject.as_str()])),
            };
            if let Some(issue) = item.issue.as_deref() {
                let reason = item
                    .reason
                    .and_then(|reason| serde_json::to_value(reason).ok())
                    .and_then(|value| value.as_str().map(str::to_string))
                    .filter(|code| !issue.contains(code.as_str()))
                    .map(|code| format!(" ({code})"))
                    .unwrap_or_default();
                push("issue", format!("{issue}{reason}"));
            }
            if !item.unchanged_surfaces.is_empty() {
                let names: Vec<_> = item.unchanged_surfaces.iter().map(String::as_str).collect();
                push("unchanged", abbreviated(&names));
            }
            if let Some(action) = item.next_action.as_deref() {
                push("next", action.to_string());
            }
        }
        if let Some(feature) = status
            .current_feature
            .as_deref()
            .and_then(|id| status.features.iter().find(|feature| feature.id == id))
        {
            if feature.workflow.additional_issue_count > 0 {
                facts.push((
                    "also",
                    format!(
                        "{} additional issue(s) at this checkpoint",
                        feature.workflow.additional_issue_count
                    ),
                    vec![feature.id.as_str()],
                ));
            }
        }
        const FIELDS: [&str; 4] = ["issue", "also", "unchanged", "next"];
        facts.sort_by_key(|(field, _, _)| FIELDS.iter().position(|f| f == field));
        out.push_str("\nDETAILS\n");
        let rows = facts
            .into_iter()
            .map(|(field, value, subjects)| vec![subjects.join(", "), field.into(), value])
            .collect::<Vec<_>>();
        table(
            &mut out,
            &[("SUBJECT", true), ("FIELD", false), ("VALUE", true)],
            &rows,
            width,
            true,
        );
        // Commands are printed whole so they can be copied; the terminal may soft-wrap them.
        let commands = status
            .attention
            .iter()
            .filter_map(|item| {
                let command = item.action.as_ref()?.command.as_deref()?;
                Some(format!("  {}  {command}\n", item.subject))
            })
            .collect::<String>();
        if !commands.is_empty() {
            out.push_str("\nCOMMANDS\n");
            out.push_str(&commands);
        }
    }

    if let Some(feature) = status
        .current_feature
        .as_deref()
        .and_then(|id| status.features.iter().find(|feature| feature.id == id))
    {
        out.push_str(&format!("\n{} CHECKPOINTS\n", feature.id));
        let rows = feature
            .workflow
            .checkpoints
            .iter()
            .map(|checkpoint| {
                vec![
                    checkpoint.id.clone(),
                    checkpoint.title.clone(),
                    checkpoint.status.label().into(),
                ]
            })
            .collect::<Vec<_>>();
        table(
            &mut out,
            &[("ID", false), ("TITLE", true), ("STATE", false)],
            &rows,
            width,
            true,
        );
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
    if !status.retired_features.is_empty() {
        out.push_str("\nRETIRED\n");
        for feature in &status.retired_features {
            out.push_str(&format!(
                "  {}  retired {}\n",
                feature.id, feature.retired_at
            ));
        }
    }
    out.trim_end().to_string()
}

fn incomplete_unaccepted(status: &ProjectStatus) -> usize {
    status
        .features
        .iter()
        .filter(|feature| {
            !feature.historically_accepted && feature.workflow.status != DisplayStatus::Complete
        })
        .count()
}

/// Completed of total, then every checkpoint that is not complete.
fn bootstrap_summary(status: &WorkflowStatus) -> String {
    let open = status
        .checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.status != DisplayStatus::Complete)
        .map(|checkpoint| format!("{} {}", checkpoint.id, checkpoint.status.label()))
        .collect::<Vec<_>>();
    match open.is_empty() {
        true => format!("{}/{}", status.completed, status.total),
        false => format!(
            "{}/{} · {}",
            status.completed,
            status.total,
            open.join(" · ")
        ),
    }
}

/// Bases that differ only by a leading feature ID are listed once, after their IDs, in order of
/// first occurrence. Every other basis is kept verbatim.
fn grouped(bases: &[String]) -> Vec<String> {
    let mut groups: Vec<(Vec<&str>, &str)> = Vec::new();
    for basis in bases {
        let split = basis
            .find([':', ' '])
            .map(|at| basis.split_at(at))
            .filter(|(id, _)| {
                id.strip_prefix("F-")
                    .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
            });
        match split {
            Some((id, rest)) => match groups
                .iter_mut()
                .find(|(ids, other)| !ids.is_empty() && *other == rest)
            {
                Some((ids, _)) => ids.push(id),
                None => groups.push((vec![id], rest)),
            },
            None => groups.push((vec![], basis.as_str())),
        }
    }
    groups
        .into_iter()
        .map(|(ids, rest)| format!("{}{rest}", ids.join(", ")))
        .collect()
}

/// An aligned table indented by two spaces. Only `wrap` columns shrink to fit `width`: first by
/// word-wrapping down to their longest word, then, only if still needed, by splitting words.
/// Other columns keep their full width, so a table that cannot shrink further exceeds `width`
/// rather than truncate. A cell may hold several lines, separated by `\n`.
fn table(
    out: &mut String,
    columns: &[(&str, bool)],
    rows: &[Vec<String>],
    width: usize,
    headed: bool,
) {
    const MIN_SPLIT: usize = 12;
    let chars = |text: &str| text.chars().count();
    let mut widths: Vec<usize> = columns
        .iter()
        .enumerate()
        .map(|(i, (heading, _))| {
            rows.iter()
                .flat_map(|row| row[i].split('\n'))
                .map(chars)
                .chain([if headed { chars(heading) } else { 0 }])
                .max()
                .unwrap_or(0)
        })
        .collect();
    let total = |widths: &[usize]| 2 + widths.iter().sum::<usize>() + 2 * (widths.len() - 1);
    let longest_word: Vec<usize> = (0..columns.len())
        .map(|i| {
            rows.iter()
                .flat_map(|row| row[i].split(['\n', ' ']))
                .map(chars)
                .max()
                .unwrap_or(0)
        })
        .collect();
    for floor in [&longest_word, &vec![MIN_SPLIT; columns.len()]] {
        while total(&widths) > width {
            let Some(widest) = (0..widths.len())
                .filter(|&i| columns[i].1 && widths[i] > floor[i].max(MIN_SPLIT))
                .max_by_key(|&i| (widths[i], std::cmp::Reverse(i)))
            else {
                break;
            };
            widths[widest] -= 1;
        }
    }
    let heading: Vec<String> = columns.iter().map(|(h, _)| h.to_string()).collect();
    for row in headed.then_some(&heading).into_iter().chain(rows) {
        let cells: Vec<Vec<String>> = row
            .iter()
            .zip(&widths)
            .map(|(cell, width)| {
                cell.split('\n')
                    .flat_map(|part| wrap(part, *width))
                    .collect()
            })
            .collect();
        let height = cells.iter().map(Vec::len).max().unwrap_or(1);
        for line in 0..height {
            let mut text = String::from("  ");
            for (i, cell) in cells.iter().enumerate() {
                let part = cell.get(line).map(String::as_str).unwrap_or("");
                text.push_str(part);
                if i + 1 < cells.len() {
                    text.push_str(&" ".repeat(widths[i] - chars(part) + 2));
                }
            }
            out.push_str(text.trim_end());
            out.push('\n');
        }
    }
}

/// Word-wraps `text` to `width` characters. A word longer than `width` is split, preferably after
/// `/`, `-`, or `_`, and never dropped.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        let mut word: Vec<char> = word.chars().collect();
        while word.len() > width {
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            let at = (1..width)
                .rev()
                .find(|&at| matches!(word[at - 1], '/' | '-' | '_'))
                .unwrap_or(width);
            lines.push(word.drain(..at).collect());
        }
        let word: String = word.into_iter().collect();
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(&word);
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
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
        let project = Project::new(dir.path().to_path_buf(), false);
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
    fn identical_feature_bases_are_listed_once_with_their_ids() {
        let bases = [
            "F-0002: accepted snapshot is no longer current (REVALIDATION_REQUIRED)",
            "F-0003: accepted snapshot is no longer current (REVALIDATION_REQUIRED)",
            "F-0006 F9 project_qualification record is stale: 'implementation_state' changed",
            "compatibility/support.json: DBA-28 supported",
            "F-0001 F9 project_qualification record is stale: 'implementation_state' changed",
        ]
        .map(String::from);
        assert_eq!(
            grouped(&bases),
            [
                "F-0002, F-0003: accepted snapshot is no longer current (REVALIDATION_REQUIRED)",
                "F-0006, F-0001 F9 project_qualification record is stale: 'implementation_state' changed",
                "compatibility/support.json: DBA-28 supported",
            ]
        );
    }

    #[test]
    fn wrapping_never_drops_text_and_splits_words_only_when_needed() {
        let text = "changed backend/Cargo.lock and scripts/verify-production-claims.mjs (+3)";
        for width in [12, 20, 40] {
            let lines = wrap(text, width);
            assert!(
                lines.iter().all(|line| line.chars().count() <= width),
                "{lines:?}"
            );
            assert_eq!(lines.concat().replace(' ', ""), text.replace(' ', ""));
        }
        assert_eq!(
            wrap("scripts/verify-production-claims.mjs", 20),
            ["scripts/verify-", "production-", "claims.mjs"]
        );
        assert_eq!(wrap("REVALIDATION_REQUIRED", 30), ["REVALIDATION_REQUIRED"]);
    }

    #[test]
    fn tables_shrink_only_text_columns_and_keep_identifiers_whole() {
        let rows = vec![vec![
            "POST_ACCEPTANCE_REVALIDATION".to_string(),
            "historical F9 acceptance is preserved; current acceptance validity is stale".into(),
        ]];
        let mut out = String::new();
        table(
            &mut out,
            &[("POINT", false), ("ISSUE", true)],
            &rows,
            60,
            true,
        );
        assert!(out.lines().all(|line| line.chars().count() <= 60), "{out}");
        assert!(
            out.contains("  POST_ACCEPTANCE_REVALIDATION  historical"),
            "{out}"
        );
        // An identifier column wider than the terminal is kept whole, not truncated.
        let mut narrow = String::new();
        table(
            &mut narrow,
            &[("POINT", false), ("ISSUE", true)],
            &rows,
            30,
            true,
        );
        assert!(narrow.contains("POST_ACCEPTANCE_REVALIDATION"), "{narrow}");
        let mut again = String::new();
        table(
            &mut again,
            &[("POINT", false), ("ISSUE", true)],
            &rows,
            30,
            true,
        );
        assert_eq!(narrow, again);
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
        let rendered = render_width(&status, 100);
        assert!(
            rendered.contains("  Active feature  F-0001 — Only feature\n"),
            "{rendered}"
        );
        assert!(rendered.contains("Only feature (current)"), "{rendered}");
        // The current feature's issue and checkpoints remain in the report.
        let issue = status
            .attention
            .iter()
            .find(|item| item.subject == "F-0001")
            .and_then(|item| item.issue.clone())
            .unwrap();
        assert!(
            rendered.lines().any(|line| line.starts_with("  F-0001 ")
                && line.contains(" issue ")
                && line.contains(&issue)),
            "{rendered}"
        );
        assert!(rendered.contains("\nF-0001 CHECKPOINTS\n"), "{rendered}");
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

    #[test]
    fn architecture_change_reports_the_current_ordinary_candidate_step() {
        use crate::candidate::{CandidateBundle, CandidateStore};
        use crate::receipts::{Receipt, ReceiptStore};
        let dir = tempfile::tempdir().unwrap();
        for path in [
            ".codeos/01-specification/intents/F-1.md",
            ".codeos/01-specification/contracts/F-1_contract.md",
            ".codeos/01-specification/event-schemas/F-1_schema.md",
        ] {
            let full = dir.path().join(path);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(&full, format!("{path}\n")).unwrap();
        }
        fs::create_dir_all(dir.path().join(".codeos/00-project")).unwrap();
        fs::write(
            dir.path().join(".codeos/00-project/codeos.yaml"),
            "codeos_schema: 2\n",
        )
        .unwrap();
        let project = Project::discover(dir.path()).unwrap();
        let reassess = || {
            let mut workflow = summarize(&[]);
            workflow.action = Some(ActionType::ReassessArchitecture);
            workflow
        };
        // No candidate yet: architecture reassessment stays the next step.
        let mut workflow = reassess();
        ordinary_candidate_step(
            &project,
            "F-1",
            &mut workflow,
            (Some(ActionType::ResolveReviewFinding), None),
        )
        .unwrap();
        assert_eq!(workflow.action, Some(ActionType::ReassessArchitecture));

        let receipt = Receipt {
            workflow: "feature".into(),
            subject: "F-1".into(),
            checkpoint: "reconciliation".into(),
            result: "reconciled".into(),
            bindings: Default::default(),
            timestamp: "2026-01-01T00:00:00Z".into(),
            rationale: Some("reconciled".into()),
            observation: None,
        };
        ReceiptStore::at(&project.codeos_dir())
            .append(&receipt)
            .unwrap();
        let bundle = CandidateBundle {
            schema_version: 2,
            feature: "F-1".into(),
            semantic_revision: 1,
            kind: "ordinary".into(),
            configuration_sha256: "config".into(),
            governed_inputs: crate::candidate::governed(&project, "F-1", 2).unwrap(),
            implementation_inputs: vec![],
            dependency_additions: vec![],
            verification_records: Default::default(),
            f6_receipt_sha256: crate::hashing::record_identity(&receipt).unwrap(),
            production_evidence: None,
            runtime_evidence: None,
            journey_previous: "entry".into(),
            journey_target: "outcome".into(),
            predecessor_bundle: None,
            baseline_acceptance: None,
            trigger_commit: None,
            inherited_evidence: Default::default(),
            claim_digests: Default::default(),
        };
        let id = CandidateStore::at(&project.codeos_dir())
            .write(&bundle)
            .unwrap();
        for (step, actor, next) in [
            (
                ActionType::RunReview,
                Actor::Auto,
                format!("assess ordinary Candidate Bundle {id} at Stage 8"),
            ),
            (
                ActionType::ResolveReviewFinding,
                Actor::Assisted,
                format!("resolve the assessment of ordinary Candidate Bundle {id}"),
            ),
            (
                ActionType::RecordDecision,
                Actor::Human,
                format!("re-run the final_ux_validation decision against current evidence for ordinary Candidate Bundle {id}"),
            ),
        ] {
            let mut workflow = reassess();
            let checker_next = "re-run the final_ux_validation decision against current evidence";
            ordinary_candidate_step(
                &project,
                "F-1",
                &mut workflow,
                (Some(step), Some(checker_next.into())),
            )
            .unwrap();
            assert_eq!((workflow.action, workflow.actor), (Some(step), Some(actor)));
            assert_eq!(workflow.next_action, Some(next));
        }
        // Any other checker step leaves reassessment in place.
        let mut workflow = reassess();
        ordinary_candidate_step(
            &project,
            "F-1",
            &mut workflow,
            (Some(ActionType::PrepareCandidate), None),
        )
        .unwrap();
        assert_eq!(workflow.action, Some(ActionType::ReassessArchitecture));
    }
}
