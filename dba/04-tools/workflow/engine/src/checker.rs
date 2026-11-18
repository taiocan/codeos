//! Derive each checkpoint's state from three direct evidence classes, with fail-closed ordering:
//!
//! 1. **Canonical state** — artifacts, approval metadata, configuration, binding currency. Cheap
//!    deterministic predicates, re-evaluated live on every query.
//! 2. **Verification records** — mechanical verifications `check` actually executed and that passed;
//!    persisted, hash-bound, and stale the moment a bound input drifts.
//! 3. **Decision receipts** — the closed set of irreducible human / authorized-agent decisions.
//!
//! `status` and `next` derive state read-only from all three. `check` executes pending mechanical
//! verifications and persists a record per pass. `decide` records a receipt. Nothing reconstructs an
//! earlier PASS indirectly: a later receipt never stands in for an earlier verification.

use crate::contract::{
    contract_for, ActionType, Applies, Checkpoint, ReasonCode, Requirement, Workflow,
};
use crate::evidence;
use crate::execution::Environment;
use crate::hashing;
use crate::project::Project;
use crate::receipts::{self, Receipt, ReceiptState, ReceiptStore};
use crate::test_evidence;
use crate::verification::{self, RecordState, VerificationRecord, VerificationStore};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Pass,
    Blocked,
    Waiting,
    NotApplicable,
}

impl State {
    pub fn label(&self) -> &'static str {
        match self {
            State::Pass => "PASS",
            State::Blocked => "BLOCKED",
            State::Waiting => "WAITING",
            State::NotApplicable => "n/a",
        }
    }
}

/// Actor class: who may execute a reported next action. It grants no approval, acceptance, or
/// decision authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Actor {
    /// A deterministic CodeOS command may execute the step without human initiation.
    Auto,
    /// An agent or tool may perform bounded work.
    Assisted,
    /// Human judgment or authority is required.
    Human,
}

impl Actor {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "AUTO",
            Self::Assisted => "ASSISTED",
            Self::Human => "HUMAN",
        }
    }

    /// The actor for a requirement's next action when it needs no repair work first.
    fn default_for(requirement: &Requirement) -> Self {
        match requirement {
            Requirement::CharterApproved
            | Requirement::SpecPackageApproved
            | Requirement::Receipt { .. }
            | Requirement::ObservationStated
            | Requirement::ObservationRouted
            | Requirement::RouteResolution => Self::Human,
            Requirement::LiveVerification { .. }
            | Requirement::ProductionSourceEvidence
            | Requirement::ReviewRecord { .. } => Self::Auto,
            Requirement::CodeosYamlValid
            | Requirement::SemanticPackageRevision
            | Requirement::ContractSection(_)
            | Requirement::ProductJourneyContribution => Self::Assisted,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckpointReport {
    pub id: String,
    pub title: String,
    pub state: State,
    pub met: Vec<String>,
    pub unmet: Vec<String>,
    pub next_action: Option<String>,
    pub actor: Option<Actor>,
    /// The cause of `next_action`, from the same requirement as `actor`.
    pub reason: Option<ReasonCode>,
    /// The one atomic operation `next_action` describes, from the same requirement as `reason`.
    pub action: Option<ActionType>,
    /// The comparison behind a stale cause, from the same requirement as `reason`.
    pub surfaces: Option<Surfaces>,
}

/// The surfaces one comparison found changed or positively unchanged, with the evidence it
/// compared. `origin` is the checkpoint whose comparison this is.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Surfaces {
    pub origin: String,
    pub changed: Vec<hashing::Comparison>,
    pub unchanged: Vec<String>,
    pub evidence: Vec<hashing::EvidenceRef>,
}

impl Surfaces {
    pub fn new(
        origin: &str,
        comparisons: Vec<hashing::Comparison>,
        evidence: Vec<hashing::EvidenceRef>,
    ) -> Self {
        let (changed, unchanged): (Vec<_>, Vec<_>) = comparisons
            .into_iter()
            .partition(hashing::Comparison::changed);
        Self {
            origin: origin.into(),
            changed,
            unchanged: unchanged.into_iter().map(|c| c.surface).collect(),
            evidence,
        }
    }
}

/// A feature subject with its applicability resolved from the Contract.
pub struct Subject {
    pub workflow: Workflow,
    pub id: String,
    pub gui: bool,
    pub backend: bool,
    pub persistence: bool,
    /// All three Platform Baseline rows have an explicit changed/unchanged value.
    pub impact_accounting_complete: bool,
    pub arch_approval_required: bool,
    pub reality_claims: Vec<String>,
    /// Feature Workflow v11 inherited claims, which need no production run of their own.
    pub inherited_claims: Vec<InheritedClaim>,
}

/// A claim whose production truth is one upstream reality-dependent claim (Feature Workflow v11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InheritedClaim {
    pub claim: String,
    pub upstream_feature: String,
    pub upstream_claim: String,
}

impl Subject {
    pub fn resolve(project: &Project, workflow: Workflow, id: &str) -> Result<Self> {
        let (mut gui, mut backend, mut persistence) = (false, false, false);
        let mut impact_accounting_complete = workflow != Workflow::Feature;
        let mut reality_claims = Vec::new();
        let mut inherited_claims = Vec::new();
        if workflow == Workflow::Feature {
            let contract_path = project.contract(id);
            if let Ok(text) = std::fs::read_to_string(&contract_path) {
                // Read the Feature Impact Accounting table rows only, plus an explicit marker.
                let lower = text.to_lowercase();
                let gui_impact = impact_row(&lower, "gui");
                let backend_impact = impact_row(&lower, "backend");
                let persistence_impact = impact_row(&lower, "persistence");
                impact_accounting_complete = gui_impact.is_some()
                    && backend_impact.is_some()
                    && persistence_impact.is_some();
                gui = lower.contains("gui-visible outcome") || gui_impact == Some(true);
                backend = backend_impact == Some(true);
                persistence = persistence_impact == Some(true);
                let inherits = project.feature_workflow_version()? >= 11;
                (reality_claims, inherited_claims) = reality_basis(&text, inherits)?;
            }
        }
        Ok(Self {
            workflow,
            id: id.to_string(),
            gui,
            backend,
            persistence,
            impact_accounting_complete,
            arch_approval_required: false,
            reality_claims,
            inherited_claims,
        })
    }
}

/// Read-only: derive every checkpoint's state for a subject from canonical evidence, verification
/// records, and decision receipts. Never executes a command.
pub fn evaluate(project: &Project, subject: &Subject) -> Result<Vec<CheckpointReport>> {
    let store = ReceiptStore::at(&project.codeos_dir());
    let vstore = VerificationStore::at(&project.codeos_dir());
    let contract = contract_for(
        subject.workflow,
        project.feature_workflow_version()?,
        project.operation_workflow_version()?,
    );
    let mut reports = Vec::new();
    let mut all_prior_pass = true;

    // bootstrap-entry: a solution's first Feature Development workflow cannot enter its first
    // checkpoint while Bootstrap is not complete. Once any feature has an acceptance receipt,
    // Bootstrap has by definition already happened and the gate is satisfied.
    if subject.workflow == Workflow::Feature {
        let a_feature_was_accepted = store
            .all()?
            .iter()
            .any(|r| r.checkpoint == "acceptance" && r.result == "accepted");
        if !a_feature_was_accepted {
            let boot = Subject {
                workflow: Workflow::Bootstrap,
                id: "solution".into(),
                gui: false,
                backend: false,
                persistence: false,
                impact_accounting_complete: true,
                arch_approval_required: false,
                reality_claims: vec![],
                inherited_claims: vec![],
            };
            let boot_reports = evaluate(project, &boot)?;
            let boot_complete = boot_reports
                .iter()
                .filter(|r| matches!(r.id.as_str(), "B1" | "B2" | "B3" | "B4"))
                .all(|r| r.state == State::Pass);
            if !boot_complete {
                let blocking = boot_reports
                    .iter()
                    .find(|r| matches!(r.state, State::Blocked | State::Waiting));
                let blocker = blocking
                    .map(|r| r.id.clone())
                    .unwrap_or_else(|| "B?".into());
                reports.push(CheckpointReport {
                    id: "bootstrap-entry".into(),
                    title: "Solution Bootstrap complete".into(),
                    state: State::Blocked,
                    met: vec![],
                    unmet: vec![format!(
                        "Solution Bootstrap is not complete — {blocker} is not PASS"
                    )],
                    next_action: Some(
                        "run `codeos-workflow check --workflow bootstrap --subject solution`"
                            .into(),
                    ),
                    actor: blocking.and_then(|r| r.actor),
                    reason: Some(ReasonCode::BootstrapIncomplete),
                    // Inherit the blocking bootstrap action rather than an outcome.
                    action: blocking.and_then(|r| r.action),
                    surfaces: None,
                });
                all_prior_pass = false;
            } else {
                reports.push(CheckpointReport {
                    id: "bootstrap-entry".into(),
                    title: "Solution Bootstrap complete".into(),
                    state: State::Pass,
                    met: vec!["Bootstrap B1–B4 are PASS".into()],
                    unmet: vec![],
                    next_action: None,
                    actor: None,
                    reason: None,
                    action: None,
                    surfaces: None,
                });
            }
        }
    }

    // Feature Workflow v13: a Validation Claims table Codeos cannot attribute blocks the feature.
    if let Err(error) = subject_claims(project, subject) {
        reports.push(CheckpointReport {
            id: "validation-claims".into(),
            title: "Validation Claims are attributable".into(),
            state: State::Blocked,
            met: vec![],
            unmet: vec![error.to_string()],
            next_action: Some(format!(
                "repair the ## Validation Claims table of {}",
                subject.id
            )),
            actor: None,
            reason: None,
            action: None,
            surfaces: None,
        });
        all_prior_pass = false;
    }

    for c in &contract {
        let applicable = applies(c.applies, subject);
        if !applicable {
            reports.push(CheckpointReport {
                id: c.id.into(),
                title: c.title.into(),
                state: State::NotApplicable,
                met: vec![],
                unmet: vec![],
                next_action: None,
                actor: None,
                reason: None,
                action: None,
                surfaces: None,
            });
            continue;
        }

        if !all_prior_pass {
            let blocking = reports
                .iter()
                .find(|r| r.state == State::Blocked || r.state == State::Waiting);
            let blocker = blocking.map(|r| r.id.clone()).unwrap_or_default();
            reports.push(CheckpointReport {
                id: c.id.into(),
                title: c.title.into(),
                state: State::Blocked,
                met: vec![],
                unmet: vec![format!("earlier checkpoint {blocker} is not PASS")],
                next_action: Some(format!("resolve {blocker} first")),
                actor: blocking.and_then(|r| r.actor),
                reason: blocking.and_then(|r| r.reason),
                action: blocking.and_then(|r| r.action),
                surfaces: blocking.and_then(|r| r.surfaces.clone()),
            });
            continue;
        }

        let report = evaluate_checkpoint(project, subject, c, &store, &vstore)?;
        if report.state != State::Pass {
            all_prior_pass = false;
        }
        reports.push(report);
    }
    Ok(reports)
}

/// One explicit Feature Impact Accounting row: changed, unchanged, or absent/invalid.
fn impact_row(lower_text: &str, tier_keyword: &str) -> Option<bool> {
    lower_text.lines().find_map(|line| {
        let l = line.trim();
        if !l.starts_with('|') {
            return None;
        }
        let values: Vec<_> = l.split('|').map(str::trim).collect();
        if !values
            .get(1)
            .is_some_and(|tier| tier.contains(tier_keyword))
        {
            return None;
        }
        match values.get(2).copied() {
            Some("changed") => Some(true),
            Some("unchanged") => Some(false),
            _ => None,
        }
    })
}

/// The Reality Basis: reality-dependent claim IDs, and (v11) inherited claims.
fn reality_basis(contract: &str, inherits: bool) -> Result<(Vec<String>, Vec<InheritedClaim>)> {
    let Some(section) = contract.split("## Reality Basis").nth(1) else {
        return Ok((vec![], vec![]));
    };
    let section = section.split("\n## ").next().unwrap_or(section);
    let rows: Vec<Vec<&str>> = section
        .lines()
        .filter_map(|line| {
            let cells: Vec<_> = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect();
            (cells.len() >= 3
                && !matches!(cells[0], "" | "Claim ID")
                && !cells[0].chars().all(|c| c == '-'))
            .then_some(cells)
        })
        .collect();
    let mut reality = Vec::new();
    let mut inherited = Vec::new();
    for cells in rows {
        let inherited_row = inherits && cells[2] == "inherited";
        if !inherited_row
            && !matches!(
                cells[2],
                "reality-dependent" | "simulated" | "not-reality-dependent"
            )
        {
            anyhow::bail!("claim '{}' has an invalid reality classification", cells[0]);
        }
        if inherited_row {
            if cells.len() < 8 || cells[3..8].iter().any(|value| value.trim().is_empty()) {
                anyhow::bail!(
                    "inherited claim '{}' has incomplete source requirements",
                    cells[0]
                );
            }
            let source = cells[3].trim_matches('`');
            let Some((feature, claim)) = source
                .split_once('/')
                .filter(|(f, c)| f.starts_with("F-") && !c.is_empty() && !c.contains(char::is_whitespace))
            else {
                anyhow::bail!(
                    "inherited claim '{}' must name exactly one upstream F-NNNN/claim-id",
                    cells[0]
                );
            };
            inherited.push(InheritedClaim {
                claim: cells[0].to_string(),
                upstream_feature: feature.to_string(),
                upstream_claim: claim.to_string(),
            });
            continue;
        }
        if cells[2] == "reality-dependent" {
            if cells.len() < 8 || cells[3..8].iter().any(|value| value.trim().is_empty()) {
                anyhow::bail!(
                    "reality-dependent claim '{}' has incomplete source requirements",
                    cells[0]
                );
            }
            reality.push(cells[0].to_string());
        }
    }
    Ok((reality, inherited))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JourneyContribution {
    Preserve {
        reason: String,
    },
    Advance {
        previous: String,
        proposed: String,
        transition: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct JourneySetup {
    category: String,
    resource: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct JourneyObservation {
    transition_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    handoff_identity: Option<String>,
    state_origin: String,
    result: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct JourneyEvidence {
    schema: u8,
    journey_id: String,
    target_frontier: String,
    #[serde(default)]
    setup: Vec<JourneySetup>,
    observations: Vec<JourneyObservation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ExpectedTransition {
    transition_id: String,
    from_frontier: String,
    to_frontier: String,
    producer_role: String,
    handoff_type: String,
    consumer_role: String,
    authority_kind: String,
    authority_id: String,
    owner_feature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    authority_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct JourneyDefinition {
    schema: u8,
    journey_id: String,
    target_frontier: String,
    transitions: Vec<ExpectedTransition>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArchitectureTransitions {
    transitions: Vec<ArchitectureTransition>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArchitectureTransition {
    transition_id: String,
    from_frontier: String,
    to_frontier: String,
    producer_feature: String,
    producer_role: String,
    handoff_type: String,
    consumer_feature: String,
    consumer_role: String,
}

#[derive(Debug, Deserialize)]
struct ArchitectureFrontmatter {
    #[serde(default)]
    features: Vec<String>,
}

fn section_fields(text: &str, heading: &str) -> BTreeMap<String, String> {
    let marker = format!("## {heading}");
    let mut inside = false;
    let mut fields = BTreeMap::new();
    for line in text.lines() {
        if line.trim() == marker {
            inside = true;
            continue;
        }
        if inside && line.starts_with("## ") {
            break;
        }
        if inside {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                if !key.is_empty() && !value.is_empty() {
                    fields.insert(key.to_string(), value.to_string());
                }
            }
        }
    }
    fields
}

pub(crate) fn journey_contribution(path: &Path) -> Result<JourneyContribution> {
    let text = std::fs::read_to_string(path)?;
    let fields = section_fields(&text, "Product Journey Contribution");
    match fields.get("mode").map(String::as_str) {
        Some("preserve") => fields
            .get("reason")
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .map(|reason| JourneyContribution::Preserve { reason })
            .ok_or_else(|| anyhow::anyhow!("mode preserve requires a non-empty reason"))
            .and_then(|contribution| {
                if ["previous_frontier", "proposed_frontier", "transition"]
                    .iter()
                    .any(|name| fields.contains_key(*name))
                {
                    anyhow::bail!("mode preserve must not declare advance fields");
                }
                Ok(contribution)
            }),
        Some("advance") => {
            let required = |name: &str| {
                fields
                    .get(name)
                    .filter(|value| !value.trim().is_empty())
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("mode advance requires {name}"))
            };
            let previous = required("previous_frontier")?;
            let proposed = required("proposed_frontier")?;
            let transition = required("transition")?;
            if fields.contains_key("reason") {
                anyhow::bail!("mode advance must not declare preserve reason");
            }
            if !valid_frontier_id(&previous) || !valid_frontier_id(&proposed) {
                anyhow::bail!(
                    "frontier identifiers must use only letters, digits, dot, underscore, or hyphen"
                );
            }
            if previous == proposed {
                anyhow::bail!("proposed_frontier must differ from previous_frontier");
            }
            Ok(JourneyContribution::Advance {
                previous,
                proposed,
                transition,
            })
        }
        Some(other) => anyhow::bail!("unsupported product journey mode '{other}'"),
        None => anyhow::bail!("Product Journey Contribution must declare mode"),
    }
}

fn valid_frontier_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn charter_initial_frontier(project: &Project) -> Result<String> {
    let text = std::fs::read_to_string(project.charter())?;
    let fields = section_fields(&text, "Primary Product Journey");
    for name in ["journey_id", "legitimate_entry", "target_outcome"] {
        if fields
            .get(name)
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
        {
            anyhow::bail!("Charter Primary Product Journey must declare {name}");
        }
    }
    if fields.get("verification_route").map(String::as_str) != Some("product_journey") {
        anyhow::bail!("Charter Primary Product Journey verification_route must be product_journey");
    }
    let frontier = fields
        .get("initial_frontier")
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("Charter Primary Product Journey must declare initial_frontier")
        })?;
    if !valid_frontier_id(&frontier) {
        anyhow::bail!(
            "Charter initial_frontier must use only letters, digits, dot, underscore, or hyphen"
        );
    }
    Ok(frontier)
}

fn charter_journey(project: &Project) -> Result<(String, String, String, String)> {
    let text = std::fs::read_to_string(project.charter())?;
    let fields = section_fields(&text, "Primary Product Journey");
    let required = |name: &str| {
        fields
            .get(name)
            .filter(|v| !v.trim().is_empty())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Charter Primary Product Journey must declare {name}"))
    };
    Ok((
        required("journey_id")?,
        required("legitimate_entry")?,
        required("initial_frontier")?,
        required("target_outcome")?,
    ))
}

fn accepted_advances_excluding(
    project: &Project,
    excluded_subject: Option<&str>,
) -> Result<Vec<(String, String, String)>> {
    let receipts = ReceiptStore::at(&project.codeos_dir()).all()?;
    let mut latest = BTreeMap::new();
    for (index, receipt) in receipts.iter().enumerate() {
        if receipt.workflow == "feature"
            && receipt.checkpoint == "acceptance"
            && receipt.result == "accepted"
        {
            latest.insert(receipt.subject.clone(), index);
        }
    }
    let latest_indices: BTreeSet<_> = latest.into_values().collect();
    let mut advances = Vec::new();
    for (index, receipt) in receipts.into_iter().enumerate() {
        if !latest_indices.contains(&index) || excluded_subject == Some(receipt.subject.as_str()) {
            continue;
        }
        let contract = project.contract(&receipt.subject);
        let Some(now) = hashing::file_sha256(&contract)? else {
            continue;
        };
        if receipt.bindings.get("contract") != Some(&now) {
            continue;
        }
        if let JourneyContribution::Advance {
            previous, proposed, ..
        } = journey_contribution(&contract)?
        {
            advances.push((receipt.subject, previous, proposed));
        }
    }
    Ok(advances)
}

fn markdown_frontmatter_and_body(text: &str) -> Result<(&str, &str)> {
    let rest = text
        .strip_prefix("---\n")
        .ok_or_else(|| anyhow::anyhow!("missing YAML frontmatter"))?;
    let (frontmatter, body) = rest
        .split_once("\n---\n")
        .ok_or_else(|| anyhow::anyhow!("unterminated YAML frontmatter"))?;
    Ok((frontmatter, body))
}

fn fenced_yaml_section<'a>(body: &'a str, heading: &str) -> Result<Option<&'a str>> {
    let marker = format!("## {heading}");
    let Some(after) = body.split_once(&marker).map(|(_, after)| after) else {
        return Ok(None);
    };
    let section = after.split("\n## ").next().unwrap_or(after);
    let Some(after_fence) = section.split_once("```yaml\n").map(|(_, after)| after) else {
        anyhow::bail!("{heading} must contain one fenced YAML block");
    };
    let Some((yaml, _)) = after_fence.split_once("\n```") else {
        anyhow::bail!("{heading} has an unterminated YAML block");
    };
    Ok(Some(yaml))
}

fn architecture_transition(
    project: &Project,
    producer_feature: &str,
    consumer_feature: &str,
    from: &str,
    to: &str,
) -> Result<ExpectedTransition> {
    let dir = project.codeos_dir().join("02-architecture/scopes");
    let mut matches = Vec::new();
    let mut seen_ids = BTreeSet::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            let text = std::fs::read_to_string(&path)?;
            let (frontmatter, body) = markdown_frontmatter_and_body(&text)?;
            let metadata: ArchitectureFrontmatter = serde_yaml::from_str(frontmatter)?;
            let Some(yaml) = fenced_yaml_section(body, "Primary Journey Transitions")? else {
                continue;
            };
            let definitions: ArchitectureTransitions = serde_yaml::from_str(yaml)?;
            for definition in definitions.transitions {
                let valid_id = definition
                    .transition_id
                    .strip_prefix("T-")
                    .map(|digits| digits.len() == 4 && digits.chars().all(|c| c.is_ascii_digit()))
                    .unwrap_or(false);
                if !valid_id {
                    anyhow::bail!("architecture transition_id must use T-####");
                }
                if !seen_ids.insert(definition.transition_id.clone()) {
                    anyhow::bail!(
                        "duplicate architecture transition_id {}",
                        definition.transition_id
                    );
                }
                if [
                    definition.producer_role.as_str(),
                    definition.handoff_type.as_str(),
                    definition.consumer_role.as_str(),
                ]
                .iter()
                .any(|value| value.trim().is_empty())
                {
                    anyhow::bail!(
                        "architecture transition {} has an empty topology role",
                        definition.transition_id
                    );
                }
                if definition.from_frontier == from && definition.to_frontier == to {
                    if !evidence::approval_recorded(&path)? {
                        anyhow::bail!(
                            "cross-feature journey transition {from} -> {to} is defined by unapproved architecture scope {}",
                            path.display()
                        );
                    }
                    for feature in [&definition.producer_feature, &definition.consumer_feature] {
                        if !metadata.features.contains(feature) {
                            anyhow::bail!(
                                "architecture transition {} names feature {} outside its scope membership",
                                definition.transition_id,
                                feature
                            );
                        }
                    }
                    if definition.producer_feature != producer_feature
                        || definition.consumer_feature != consumer_feature
                    {
                        anyhow::bail!(
                            "architecture transition {} assigns the wrong producer or consumer feature",
                            definition.transition_id
                        );
                    }
                    let scope_id = path.file_stem().unwrap().to_string_lossy().into_owned();
                    let relative = path
                        .strip_prefix(&project.root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .into_owned();
                    matches.push(ExpectedTransition {
                        transition_id: definition.transition_id,
                        from_frontier: definition.from_frontier,
                        to_frontier: definition.to_frontier,
                        producer_role: definition.producer_role,
                        handoff_type: definition.handoff_type,
                        consumer_role: definition.consumer_role,
                        authority_kind: "architecture".into(),
                        authority_id: scope_id,
                        owner_feature: consumer_feature.into(),
                        authority_path: Some(relative),
                    });
                }
            }
        }
    }
    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => anyhow::bail!(
            "cross-feature journey transition {from} -> {to} requires one approved Architecture Scope definition"
        ),
        _ => anyhow::bail!(
            "cross-feature journey transition {from} -> {to} has duplicate Architecture Scope definitions"
        ),
    }
}

fn accepted_journey_definition(
    project: &Project,
    excluded_subject: Option<&str>,
) -> Result<(JourneyDefinition, Option<String>)> {
    let (journey_id, _, initial, _) = charter_journey(project)?;
    // From v13 the definition follows the same stable accepted chain as the journey target, so a
    // stale acceptance stays in the topology here too.
    let advances = if project.feature_workflow_version()? >= 13 {
        accepted_journey_chain(project, excluded_subject)?.1
    } else {
        accepted_chain(
            &initial,
            accepted_advances_excluding(project, excluded_subject)?,
        )?
    };
    let mut frontier = initial;
    let mut prior_owner: Option<String> = None;
    let mut transitions = Vec::new();
    for (owner, previous, proposed) in advances {
        let transition = match prior_owner.as_deref() {
            Some(producer) if producer != owner => {
                architecture_transition(project, producer, &owner, &previous, &proposed)?
            }
            _ => ExpectedTransition {
                transition_id: format!("feature-{}-{}-{}", owner, previous, proposed),
                from_frontier: previous.clone(),
                to_frontier: proposed.clone(),
                producer_role: "primary-entry".into(),
                handoff_type: "feature-output".into(),
                consumer_role: owner.clone(),
                authority_kind: "feature".into(),
                authority_id: owner.clone(),
                owner_feature: owner.clone(),
                authority_path: None,
            },
        };
        transitions.push(transition);
        frontier = proposed;
        prior_owner = Some(owner);
    }
    Ok((
        JourneyDefinition {
            schema: 1,
            journey_id,
            target_frontier: frontier,
            transitions,
        },
        prior_owner,
    ))
}

fn journey_definition(project: &Project, subject: &Subject) -> Result<JourneyDefinition> {
    let (mut definition, prior_owner) = accepted_journey_definition(project, Some(&subject.id))?;
    let mut frontier = definition.target_frontier.clone();
    if let JourneyContribution::Advance {
        previous, proposed, ..
    } = journey_contribution(&project.contract(&subject.id))?
    {
        if previous != frontier {
            anyhow::bail!("previous_frontier '{previous}' does not match current accepted frontier '{frontier}'");
        }
        let transition = match prior_owner.as_deref() {
            Some(producer) if producer != subject.id => {
                architecture_transition(project, producer, &subject.id, &previous, &proposed)?
            }
            _ => ExpectedTransition {
                transition_id: format!("feature-{}-{}-{}", subject.id, previous, proposed),
                from_frontier: previous.clone(),
                to_frontier: proposed.clone(),
                producer_role: "primary-entry".into(),
                handoff_type: "feature-output".into(),
                consumer_role: subject.id.clone(),
                authority_kind: "feature".into(),
                authority_id: subject.id.clone(),
                owner_feature: subject.id.clone(),
                authority_path: None,
            },
        };
        definition.transitions.push(transition);
        frontier = proposed;
    }
    definition.target_frontier = frontier;
    Ok(definition)
}

/// Derive the sole accepted frontier from the Charter and the latest current acceptance receipt
/// for each journey-advancing feature. No parallel frontier state is stored.
/// The accepted frontier without the subject's own acceptance.
pub fn accepted_frontier_before(project: &Project, subject: &Subject) -> Result<String> {
    accepted_frontier_excluding(project, Some(&subject.id))
}

pub fn current_accepted_frontier(project: &Project) -> Result<String> {
    accepted_frontier_excluding(project, None)
}

fn accepted_frontier_excluding(
    project: &Project,
    excluded_subject: Option<&str>,
) -> Result<String> {
    let (initial, chain) = accepted_journey_chain(project, excluded_subject)?;
    Ok(chain
        .last()
        .map_or(initial, |(_, _, proposed)| proposed.clone()))
}

/// Accepted journey advances as (owner, previous, proposed).
type Advances = Vec<(String, String, String)>;

/// The Charter's initial frontier and the accepted advances ordered from it. Before Feature
/// Workflow v13 only currently valid acceptances count; from v13 every latest acceptance does.
fn accepted_journey_chain(
    project: &Project,
    excluded_subject: Option<&str>,
) -> Result<(String, Advances)> {
    let initial = charter_initial_frontier(project)?;
    let stable_topology = project.feature_workflow_version()? >= 13;
    let receipts = ReceiptStore::at(&project.codeos_dir()).all()?;
    let mut latest = BTreeMap::new();
    for (index, receipt) in receipts.iter().enumerate() {
        if receipt.workflow == "feature"
            && receipt.checkpoint == "acceptance"
            && receipt.result == "accepted"
        {
            latest.insert(receipt.subject.clone(), index);
        }
    }
    let latest_indices: BTreeSet<_> = latest.into_values().collect();
    let mut advances = Vec::new();
    // Frontiers at which a bundle-bound non-advancing acceptance was prepared.
    let mut preserved = Vec::new();
    for (index, receipt) in receipts.into_iter().enumerate() {
        if !latest_indices.contains(&index) {
            continue;
        }
        // From v13 the subject's own step is cut from the full chain below, not removed from it,
        // so the steps after it stay reachable.
        if excluded_subject == Some(receipt.subject.as_str()) && !stable_topology {
            continue;
        }
        if !stable_topology
            && !crate::revalidation::current_acceptance_valid(project, &receipt.subject)?
        {
            continue;
        }
        if project.workflow_governance_version()? >= 3 {
            if let Some(bundle_id) = receipt.bindings.get("candidate_bundle") {
                let bundle =
                    crate::candidate::CandidateStore::at(&project.codeos_dir()).read(bundle_id)?;
                if bundle.journey_previous == bundle.journey_target {
                    preserved.push((receipt.subject, bundle.journey_previous));
                } else {
                    advances.push((receipt.subject, bundle.journey_previous, bundle.journey_target));
                }
                continue;
            }
        }
        let contract = project.contract(&receipt.subject);
        let Some(now) = hashing::file_sha256(&contract)? else {
            continue;
        };
        if receipt.bindings.get("contract") != Some(&now) {
            continue;
        }
        if let JourneyContribution::Advance {
            previous, proposed, ..
        } = journey_contribution(&contract)?
        {
            advances.push((receipt.subject, previous, proposed));
        }
    }
    let chain = accepted_chain(&initial, advances)?;
    for (subject, frontier) in preserved {
        if frontier != initial && !chain.iter().any(|(_, _, proposed)| *proposed == frontier) {
            anyhow::bail!(
                "accepted candidate journey conflict: {subject} was prepared at frontier '{frontier}', which the accepted journey does not reach"
            );
        }
    }
    let chain = match excluded_subject {
        Some(subject) if stable_topology => before(chain, subject),
        _ => chain,
    };
    Ok((initial, chain))
}

/// Feature Workflow v13: the accepted journey a feature builds on — the full accepted chain, cut
/// just before that feature's own step when it has one.
fn before(chain: Advances, subject: &str) -> Advances {
    chain
        .into_iter()
        .take_while(|(owner, ..)| owner != subject)
        .collect()
}

/// Order accepted advances `(owner, previous, proposed)` into the one chain that starts at the
/// Charter's initial frontier. Journey order is a property of the transitions, never of the order
/// in which acceptance receipts were appended: re-accepting an upstream feature must not reorder
/// the journey. Two advances from one frontier, a cycle, or an advance the chain never reaches
/// fail closed.
fn accepted_chain(
    initial: &str,
    advances: Vec<(String, String, String)>,
) -> Result<Vec<(String, String, String)>> {
    let mut by_previous: BTreeMap<String, (String, String, String)> = BTreeMap::new();
    for advance in advances {
        if let Some((other, ..)) = by_previous.get(&advance.1) {
            anyhow::bail!(
                "accepted product-journey frontier conflict: {} and {} both advance from '{}'",
                other,
                advance.0,
                advance.1
            );
        }
        by_previous.insert(advance.1.clone(), advance);
    }
    let mut reached = BTreeSet::from([initial.to_string()]);
    let mut frontier = initial.to_string();
    let mut chain = Vec::new();
    while let Some(advance) = by_previous.remove(&frontier) {
        if !reached.insert(advance.2.clone()) {
            anyhow::bail!(
                "accepted product-journey cycle: {} returns to '{}'",
                advance.0,
                advance.2
            );
        }
        frontier = advance.2.clone();
        chain.push(advance);
    }
    if let Some((owner, previous, proposed)) = by_previous.into_values().next() {
        anyhow::bail!(
            "accepted product-journey frontier conflict: {owner} advances '{previous}' -> '{proposed}', which the accepted journey from '{initial}' does not reach"
        );
    }
    Ok(chain)
}

pub fn product_journey_target(project: &Project, subject: &Subject) -> Result<String> {
    let accepted = accepted_frontier_excluding(project, Some(&subject.id))?;
    match journey_contribution(&project.contract(&subject.id))? {
        JourneyContribution::Preserve { .. } => Ok(accepted),
        JourneyContribution::Advance {
            previous, proposed, ..
        } => {
            if previous != accepted {
                anyhow::bail!(
                    "previous_frontier '{}' does not match current accepted frontier '{}'",
                    previous,
                    accepted
                );
            }
            Ok(proposed)
        }
    }
}

/// Read-only project-level journey reachability derived from canonical artifacts and attempts.
pub fn render_journey(project: &Project, selected_subject: Option<&str>) -> Result<String> {
    if project.codeos_mechanics_version()? < 5 {
        anyhow::bail!("structured journey status requires Codeos Mechanics v5 (DBA-10)");
    }
    let (journey_id, legitimate_entry, _, target_outcome) = charter_journey(project)?;
    let (accepted, _) = accepted_journey_definition(project, None)?;
    let records = VerificationStore::at(&project.codeos_dir()).all()?;
    let mut current: BTreeMap<String, (JourneyObservation, ExpectedTransition, String)> =
        BTreeMap::new();
    let mut stale: BTreeSet<String> = BTreeSet::new();
    let mut latest_error: Option<(String, String)> = None;

    for record in records {
        if record.workflow != "feature" || record.verification != "product_journey" {
            continue;
        }
        let Some(details) = record.details.clone() else {
            continue;
        };
        let definition = details
            .get("definition")
            .cloned()
            .and_then(|v| serde_json::from_value::<JourneyDefinition>(v).ok());
        let evidence_value = details.get("evidence").cloned();
        let evidence =
            evidence_value.and_then(|v| serde_json::from_value::<JourneyEvidence>(v).ok());
        let state = Subject::resolve(project, Workflow::Feature, &record.subject)
            .and_then(|subject| {
                verification_bindings(
                    project,
                    &subject,
                    &record.checkpoint,
                    Some("product_journey"),
                )
            })
            .map(|want| verification::evaluate(Some(record.clone()), &want));
        let is_current = matches!(
            state,
            Ok(RecordState::Current(_)) | Ok(RecordState::Failed(_))
        );
        if let (Some(definition), Some(evidence)) = (definition, evidence) {
            for observation in evidence.observations {
                if let Some(expected) = definition
                    .transitions
                    .iter()
                    .find(|item| item.transition_id == observation.transition_id)
                {
                    if is_current {
                        current.insert(
                            observation.transition_id.clone(),
                            (observation, expected.clone(), record.timestamp.clone()),
                        );
                        stale.remove(&expected.transition_id);
                    } else if !current.contains_key(&observation.transition_id) {
                        stale.insert(observation.transition_id);
                    }
                }
            }
        }
        if is_current {
            if let Some(error) = details.get("validation_error").and_then(|v| v.as_str()) {
                latest_error = Some((record.timestamp, error.to_string()));
            }
        }
    }

    let mut out = format!(
        "Primary journey: {journey_id}\nLegitimate entry: {legitimate_entry}\nTarget outcome: {target_outcome}\nAccepted frontier: {}\n",
        accepted.target_frontier
    );
    render_definition_status(
        &mut out,
        "Accepted reachability",
        &accepted,
        &current,
        &stale,
        project.workflow_governance_version()? >= 3,
    );

    let subjects = if let Some(id) = selected_subject {
        vec![id.to_string()]
    } else {
        current_proposals(project)?
    };
    if !subjects.is_empty() {
        out.push_str("\nProposals:\n");
    }
    for id in subjects {
        let subject = Subject::resolve(project, Workflow::Feature, &id)?;
        let definition = journey_definition(project, &subject)?;
        out.push_str(&format!("\n{id} -> {}\n", definition.target_frontier));
        render_definition_status(
            &mut out,
            "Candidate reachability",
            &definition,
            &current,
            &stale,
            project.workflow_governance_version()? >= 3,
        );
    }
    if let Some((timestamp, error)) = latest_error {
        out.push_str(&format!(
            "\nLatest attempt error: {error}\nEvidence: current — {timestamp}\n"
        ));
    }
    Ok(out.trim_end().to_string())
}

fn render_definition_status(
    out: &mut String,
    label: &str,
    definition: &JourneyDefinition,
    current: &BTreeMap<String, (JourneyObservation, ExpectedTransition, String)>,
    stale: &BTreeSet<String>,
    unobserved_vocabulary: bool,
) {
    let first = definition.transitions.iter().find_map(|expected| {
        if let Some((observed, _, timestamp)) = current.get(&expected.transition_id) {
            if observed.result == "passed" {
                None
            } else {
                Some((
                    "BLOCKED",
                    expected,
                    Some(observed),
                    Some(timestamp.as_str()),
                ))
            }
        } else if stale.contains(&expected.transition_id) {
            Some(("STALE", expected, None, None))
        } else {
            Some((
                if unobserved_vocabulary {
                    "UNOBSERVED"
                } else {
                    "MISSING"
                },
                expected,
                None,
                None,
            ))
        }
    });
    match first {
        None => out.push_str(&format!("{label}: PASS\n")),
        Some((state, transition, observed, timestamp)) => {
            out.push_str(&format!(
                "{label}: {state}\nFirst failing transition: {} — {} -> {}\nExpected: {} -> {} -> {}\n",
                transition.transition_id,
                transition.from_frontier,
                transition.to_frontier,
                transition.producer_role,
                transition.handoff_type,
                transition.consumer_role
            ));
            if let Some(observed) = observed {
                if let Some(identity) = observed.handoff_identity.as_deref() {
                    out.push_str(&format!("Observed handoff: {identity}\n"));
                }
                out.push_str(&format!(
                    "Reason: {}\n",
                    observed
                        .reason
                        .as_deref()
                        .unwrap_or("transition did not pass")
                ));
            }
            out.push_str(&format!(
                "Evidence: {}{}\n",
                state.to_ascii_lowercase(),
                timestamp.map(|v| format!(" — {v}")).unwrap_or_default()
            ));
        }
    }
}

fn current_proposals(project: &Project) -> Result<Vec<String>> {
    let accepted = current_accepted_frontier(project)?;
    let receipts = ReceiptStore::at(&project.codeos_dir()).all()?;
    let mut latest = BTreeMap::new();
    for receipt in receipts {
        if receipt.workflow == "feature"
            && receipt.checkpoint == "acceptance"
            && receipt.result == "accepted"
        {
            latest.insert(receipt.subject.clone(), receipt);
        }
    }
    let mut accepted_subjects = BTreeSet::new();
    for (id, receipt) in latest {
        if let Some(hash) = hashing::file_sha256(&project.contract(&id))? {
            if receipt.bindings.get("contract") == Some(&hash) {
                accepted_subjects.insert(id);
            }
        }
    }
    let mut proposals = Vec::new();
    for id in project.feature_ids()? {
        if accepted_subjects.contains(&id)
            || !evidence::spec_artifact_approved(&project.intent(&id))?
            || !evidence::spec_artifact_approved(&project.contract(&id))?
            || !evidence::spec_artifact_approved(&project.event_schema(&id))?
        {
            continue;
        }
        if let JourneyContribution::Advance { previous, .. } =
            journey_contribution(&project.contract(&id))?
        {
            if previous == accepted {
                proposals.push(id);
            }
        }
    }
    Ok(proposals)
}

/// The (checkpoint, verification) records the feature workflow requires before review, exactly as
/// `evaluate` applies them to this subject.
pub fn required_verification_records(
    project: &Project,
    subject: &Subject,
) -> Result<Vec<(&'static str, &'static str)>> {
    let mut required = Vec::new();
    for checkpoint in contract_for(
        subject.workflow,
        project.feature_workflow_version()?,
        project.operation_workflow_version()?,
    ) {
        if !applies(checkpoint.applies, subject) {
            continue;
        }
        for requirement in &checkpoint.requirements {
            match requirement {
                Requirement::LiveVerification { mechanic, .. } => {
                    required.push((checkpoint.id, *mechanic))
                }
                Requirement::ProductionSourceEvidence if !subject.reality_claims.is_empty() => {
                    required.push((checkpoint.id, "production_source_evidence"))
                }
                _ => {}
            }
        }
    }
    Ok(required)
}

/// The records a Candidate Bundle binds: those review requires, with project qualification taken
/// for the candidate itself.
pub fn candidate_verification_records(
    project: &Project,
    subject: &Subject,
) -> Result<Vec<(&'static str, &'static str)>> {
    Ok(required_verification_records(project, subject)?
        .into_iter()
        .map(|(checkpoint, verification)| match verification {
            "project_qualification" => ("candidate", verification),
            _ => (checkpoint, verification),
        })
        .collect())
}

fn applies(a: Applies, s: &Subject) -> bool {
    match a {
        Applies::Always => true,
        Applies::WhenGui => s.gui,
        Applies::WhenPersistence => s.persistence,
        Applies::WhenArchApproval => s.arch_approval_required,
    }
}

fn evaluate_checkpoint(
    project: &Project,
    subject: &Subject,
    c: &Checkpoint,
    store: &ReceiptStore,
    vstore: &VerificationStore,
) -> Result<CheckpointReport> {
    let mut met = Vec::new();
    let mut unmet = Vec::new();
    let mut soft = 0usize; // unmet conditions that are only waiting on a human decision
    let mut next_action = None;
    let mut cause = None;
    // The requirement evaluated last, and whether its next action repairs a failure rather than
    // taking that requirement's default step.
    let mut last = None;
    let mut repair = false;
    // The comparison that made the last requirement stale, if any.
    let mut surfaces = None;
    // The action of an evaluation site that differs from its requirement's default.
    let mut action_override = None;

    for req in &c.requirements {
        settle_actor(
            &mut cause,
            &next_action,
            last,
            repair,
            surfaces.take(),
            action_override.take(),
        );
        last = Some(req);
        repair = false;
        match req {
            Requirement::CharterApproved => {
                if evidence::approval_recorded(&project.charter())? {
                    met.push("Charter records approval".into());
                } else if !project.charter().exists() {
                    unmet.push(".codeos/00-project/charter.md does not exist".into());
                    repair = true;
                    next_action.get_or_insert(
                        "establish the Solution Charter (support-solution-charter.md)".into(),
                    );
                } else {
                    unmet.push(
                        "Charter has no recorded approval (approval.by / approval.at)".into(),
                    );
                    next_action.get_or_insert("approve the Solution Charter".into());
                }
            }
            Requirement::SpecPackageApproved => {
                let parts = [
                    ("Intent", project.intent(&subject.id)),
                    ("Contract", project.contract(&subject.id)),
                    ("Event Schema", project.event_schema(&subject.id)),
                ];
                let mut missing = Vec::new();
                for (name, path) in &parts {
                    if !evidence::spec_artifact_approved(path)? {
                        missing.push(*name);
                    }
                }
                if missing.is_empty() {
                    met.push("Specification Package approved (all three artifacts)".into());
                } else {
                    unmet.push(format!(
                        "not recording joint package approval: {}",
                        missing.join(", ")
                    ));
                    next_action.get_or_insert(
                        "record status: APPROVED with approved_by / approved_at on all three"
                            .into(),
                    );
                }
            }
            Requirement::CodeosYamlValid => {
                let y = project.codeos_yaml();
                if !y.exists() {
                    unmet.push(".codeos/00-project/codeos.yaml is missing".into());
                    next_action.get_or_insert("create codeos.yaml from the template".into());
                } else {
                    match validate_codeos_yaml(project, &y) {
                        Some(true) => met.push("codeos.yaml validates".into()),
                        Some(false) => {
                            unmet.push(
                                "codeos.yaml does not validate against the active configuration"
                                    .into(),
                            );
                            next_action.get_or_insert(
                                "run project-config-contract.sh and fix the reported issue".into(),
                            );
                        }
                        None => {
                            unmet.push("codeos.yaml could not be read".into());
                        }
                    }
                }
            }
            Requirement::SemanticPackageRevision => {
                match semantic_package_revision(project, &subject.id) {
                    Ok(revision) if revision > 0 => {
                        met.push(format!(
                            "Specification Package semantic revision {revision}"
                        ));
                    }
                    Ok(_) => unmet.push(
                        "new Specification Packages must use a positive semantic_revision".into(),
                    ),
                    Err(error) => {
                        unmet.push(format!("invalid semantic package revision: {error}"));
                        next_action.get_or_insert(
                            "set the same semantic_revision on Intent, Contract, and Event Schema"
                                .into(),
                        );
                    }
                }
            }
            Requirement::ContractSection(heading) => {
                if evidence::section_filled(&project.contract(&subject.id), heading)? {
                    met.push(format!("Contract '{heading}' is filled in"));
                } else {
                    unmet.push(format!(
                        "Contract '{heading}' is missing or has unfilled placeholders"
                    ));
                    next_action
                        .get_or_insert(format!("complete the Contract's '{heading}' section"));
                }
            }
            Requirement::ProductJourneyContribution => {
                match journey_contribution(&project.contract(&subject.id))
                    .and_then(|contribution| {
                        let current = accepted_frontier_excluding(project, Some(&subject.id))?;
                        if let JourneyContribution::Advance { previous, .. } = &contribution {
                            if previous != &current {
                                anyhow::bail!(
                                    "previous_frontier '{}' does not match current accepted frontier '{}'",
                                    previous,
                                    current
                                );
                            }
                        }
                        Ok(contribution)
                    }) {
                    Ok(JourneyContribution::Preserve { reason }) => {
                        met.push(format!("product journey preserved: {reason}"));
                    }
                    Ok(JourneyContribution::Advance {
                        previous,
                        proposed,
                        transition,
                    }) => {
                        met.push(format!(
                            "product journey advances {previous} -> {proposed}: {transition}"
                        ));
                    }
                    Err(error) => {
                        unmet.push(format!("invalid Product Journey Contribution: {error}"));
                        next_action.get_or_insert(
                            "declare preserve with reason, or advance from the accepted frontier"
                                .into(),
                        );
                    }
                }
            }
            Requirement::Receipt {
                checkpoint,
                results,
            } => {
                let state = decision_state(project, subject, checkpoint)?;
                match state {
                    ReceiptState::Absent => {
                        soft += 1;
                        unmet.push(format!("no {checkpoint} decision recorded"));
                        next_action.get_or_insert(format!(
                            "record it: codeos-workflow decide --workflow {} --subject {} --checkpoint {checkpoint} --result <{}>",
                            subject.workflow.slug(), subject.id, results.join("|")
                        ));
                    }
                    ReceiptState::Stale {
                        binding,
                        was,
                        now,
                        receipt,
                        comparisons,
                    } => {
                        surfaces = Some(Surfaces::new(
                            c.id,
                            comparisons,
                            vec![hashing::EvidenceRef {
                                kind: hashing::EvidenceKind::DecisionReceipt,
                                id: hashing::record_identity(&receipt)?,
                            }],
                        ));
                        soft += 1;
                        unmet.push(if receipt.bindings.contains_key(&binding) {
                            format!(
                                "{checkpoint} decision is stale — '{binding}' changed since it was recorded (was {}, now {})",
                                short(&was), short(&now)
                            )
                        } else {
                            format!(
                                "{checkpoint} decision is stale — it does not bind required '{binding}', so it cannot have judged the current one (now {})",
                                short(&now)
                            )
                        });
                        next_action.get_or_insert(format!(
                            "re-run the {checkpoint} decision against current evidence"
                        ));
                    }
                    ReceiptState::Current(r) => {
                        let unstated = human_claims_not_met(project, subject, c.id, &r)?;
                        if !unstated.is_empty() {
                            soft += 1;
                            unmet.push(format!(
                                "{checkpoint} has no explicit `met` session outcome for {}",
                                unstated.join(", ")
                            ));
                            next_action.get_or_insert(format!(
                                "record a Human Validation Session for {}: codeos-workflow session",
                                subject.id
                            ));
                        } else if results.contains(&r.result.as_str()) {
                            met.push(format!("{checkpoint}: {}", r.result));
                        } else {
                            unmet.push(format!(
                                "{checkpoint} recorded '{}', which does not permit progression here",
                                r.result
                            ));
                            repair = true;
                            next_action
                                .get_or_insert(format!("route per the '{}' outcome", r.result));
                        }
                    }
                }
            }
            Requirement::ReviewRecord { stage } => {
                let rec = evidence::latest_review_record(
                    &project.review_records_dir(),
                    &subject.id,
                    stage,
                )?;
                match rec {
                    None => {
                        unmet.push(format!(
                            "no reviewer record for {} at stage {stage}",
                            subject.id
                        ));
                        next_action
                            .get_or_insert("run the reviewer at the final review boundary".into());
                    }
                    Some(r) => {
                        if project.workflow_governance_version()? >= 3 {
                            let current = crate::candidate::unique_current(project, &subject.id)?;
                            if current.as_deref() != r.candidate_bundle.as_deref() {
                                unmet.push("candidate assessment is not bound to the unique current Candidate Bundle".into());
                                action_override = Some(if current.is_some() {
                                    ActionType::RunReview
                                } else {
                                    ActionType::PrepareCandidate
                                });
                                next_action.get_or_insert(
                                    "prepare and assess the current candidate".into(),
                                );
                                continue;
                            }
                            if r.candidate_outcome.as_deref() != Some("PASSED") {
                                unmet
                                    .push("current candidate assessment outcome is BLOCKED".into());
                                repair = true;
                                next_action.get_or_insert(
                                    "repair the finding and prepare a different candidate".into(),
                                );
                                continue;
                            }
                            met.push("current Candidate Bundle assessment PASSED".into());
                            continue;
                        }
                        if let Some(selected) = selected_production_evidence(project, subject)? {
                            if !evidence::review_binds_production_evidence(
                                &project.root,
                                &r,
                                &selected,
                            )? {
                                unmet.push("reviewer packet is not bound to the production evidence selected at F6".into());
                                next_action.get_or_insert(
                                    "review the exact selected production-evidence artifact".into(),
                                );
                                continue;
                            }
                        }
                        // A v17 review binds to the identities it recorded; older records bind to
                        // the reviewed commit.
                        let drift = evidence::review_identity_drift(project, &r)?;
                        let head = current_head(&project.root);
                        let bound = match &drift {
                            Some(drift) => drift.is_empty(),
                            None => {
                                r.review_commit.is_empty()
                                    || head.is_empty()
                                    || r.review_commit == head
                            }
                        };
                        if !bound {
                            unmet.push(match &drift {
                                Some(drift) => format!(
                                    "reviewer record no longer matches current reviewed identities ({}) — re-review current evidence",
                                    drift.join(", ")
                                ),
                                None => format!(
                                    "reviewer record is bound to {} but HEAD is {} — re-review current evidence",
                                    short(&r.review_commit),
                                    short(&head)
                                ),
                            });
                            next_action.get_or_insert(
                                "re-run the reviewer against the current commit".into(),
                            );
                        } else if r.effective_concern == "EVIDENCE CLOSED — PROGRESSION PERMITTED"
                            && !evidence::evidence_closure_history_permits_progression(
                                &project.codeos_dir().join("05-review/reviews/review-log.md"),
                                &subject.id,
                                stage,
                            )?
                        {
                            unmet.push("evidence-closure assessment lacks the required composite review history".into());
                            repair = true;
                            action_override = Some(ActionType::RestoreReviewHistory);
                            next_action.get_or_insert(
                                "restore the exact authorized evidence-closure chain".into(),
                            );
                        } else if r.effective_concern
                            == "INFRASTRUCTURE EVIDENCE REVALIDATED — PROGRESSION PERMITTED"
                            && !evidence::infrastructure_revalidation_history_permits_progression(
                                &project.codeos_dir().join("05-review/reviews/review-log.md"),
                                &subject.id,
                                stage,
                            )?
                        {
                            unmet.push("infrastructure revalidation lacks the required immutable composite history".into());
                            repair = true;
                            action_override = Some(ActionType::RestoreReviewHistory);
                            next_action.get_or_insert(
                                "restore the exact authorized infrastructure-revalidation chain"
                                    .into(),
                            );
                        } else if r.review_id.contains("__EVIDENCE-REASSESSMENT-")
                            && !evidence::evidence_reassessment_history_permits_progression(
                                &project.codeos_dir().join("05-review/reviews/review-log.md"),
                                &subject.id,
                                stage,
                                &r.review_id,
                            )?
                        {
                            unmet.push(
                                "evidence reassessment lacks a valid immutable composite history"
                                    .into(),
                            );
                            repair = true;
                            action_override = Some(ActionType::RestoreReviewHistory);
                            next_action.get_or_insert("restore the exact evidence-repair chain or supply materially new evidence".into());
                        } else if r.review_id.ends_with("__LINEAGE-REPLACEMENT-CORRECTION-1")
                            && !evidence::lineage_replacement_correction_history_permits_progression(
                                &project.codeos_dir().join("05-review/reviews/review-log.md"),
                                &subject.id,
                                stage,
                                &r.review_id,
                            )?
                        {
                            unmet.push("lineage replacement correction lacks the required immutable correction history".into());
                            repair = true;
                            action_override = Some(ActionType::RestoreReviewHistory);
                            next_action.get_or_insert(
                                "restore the exact authorized assessment-input correction chain".into(),
                            );
                        } else if r.review_id.ends_with("__LINEAGE-REPLACEMENT-1")
                            && !evidence::lineage_replacement_history_permits_progression(
                                &project.codeos_dir().join("05-review/reviews/review-log.md"),
                                &subject.id,
                                stage,
                                &r.review_id,
                            )?
                        {
                            unmet.push("lineage replacement lacks the required immutable invalid-applicability history".into());
                            repair = true;
                            action_override = Some(ActionType::RestoreReviewHistory);
                            next_action.get_or_insert(
                                "restore the exact authorized lineage-replacement chain".into(),
                            );
                        } else if (r.review_id.ends_with("__TERMINAL-CLOSURE-1")
                            || r.review_id.ends_with("__TERMINAL-CLOSURE-RETRY-1"))
                            && !evidence::terminal_review_closure_history_permits_progression(
                                &project.codeos_dir().join("05-review/reviews/review-log.md"),
                                &subject.id,
                                stage,
                                &r.review_id,
                            )?
                        {
                            unmet.push(
                                "terminal review closure lacks the required immutable repair chain"
                                    .into(),
                            );
                            repair = true;
                            action_override = Some(ActionType::RestoreReviewHistory);
                            next_action.get_or_insert(
                                "restore the exact authorized terminal repair closure chain".into(),
                            );
                        } else if !evidence::review_permits_progression(&r.effective_concern) {
                            unmet.push(format!(
                                "reviewer concern '{}' does not permit progression",
                                r.effective_concern
                            ));
                            repair = true;
                            next_action.get_or_insert("resolve the reviewer's finding".into());
                        } else {
                            met.push(format!("reviewer record present ({})", r.effective_concern));
                        }
                    }
                }
            }
            Requirement::ProductionSourceEvidence => {
                for (claim, state) in crate::revalidation::inherited_evidence(project, subject)? {
                    match state {
                        Ok(upstream) => met.push(format!(
                            "inherited claim '{claim}' follows {}/{} ({})",
                            upstream.feature,
                            upstream.claim,
                            short(&upstream.evidence)
                        )),
                        Err(why) => unmet.push(format!("inherited claim '{claim}': {why}")),
                    }
                }
                if subject.reality_claims.is_empty() {
                    if subject.inherited_claims.is_empty() {
                        met.push("no reality-dependent Contract claim".into());
                    }
                } else {
                    let want = verification_bindings(
                        project,
                        subject,
                        c.id,
                        Some("production_source_evidence"),
                    )?;
                    let have = vstore.latest(
                        subject.workflow.slug(),
                        &subject.id,
                        c.id,
                        "production_source_evidence",
                    )?;
                    match verification::evaluate(have, &want) {
                        RecordState::Current(record) => met.push(format!(
                            "production-source evidence current ({})",
                            record
                                .details
                                .as_ref()
                                .and_then(|v| v.get("evidence_sha256"))
                                .and_then(|v| v.as_str())
                                .map(short)
                                .unwrap_or_else(|| "unknown".into())
                        )),
                        RecordState::Absent => {
                            unmet.push(
                                "reality-dependent claims have no production-source evidence"
                                    .into(),
                            );
                            next_action.get_or_insert(
                                "run `codeos-workflow check` with production access".into(),
                            );
                        }
                        RecordState::Failed(_) => {
                            unmet.push("production-source verification failed".into());
                        }
                        RecordState::Stale {
                            binding,
                            record,
                            comparisons,
                            ..
                        } => {
                            surfaces = Some(verification_surfaces(c.id, &record, comparisons)?);
                            unmet.push(format!(
                                "production-source evidence is stale — '{binding}' changed"
                            ));
                            next_action
                                .get_or_insert("produce new production-source evidence".into());
                        }
                    }
                }
            }
            Requirement::LiveVerification { name, mechanic } => {
                let want = verification_bindings(project, subject, c.id, Some(mechanic))?;
                let have = vstore.latest(subject.workflow.slug(), &subject.id, c.id, mechanic)?;
                match verification::evaluate(have, &want) {
                    RecordState::Absent => {
                        unmet.push(format!(
                            "{name} has no verification record — run `codeos-workflow check`"
                        ));
                        next_action
                            .get_or_insert(format!("run `codeos-workflow check` to verify {name}"));
                    }
                    RecordState::Stale {
                        binding,
                        was,
                        now,
                        record,
                        comparisons,
                    } => {
                        surfaces = Some(verification_surfaces(c.id, &record, comparisons)?);
                        unmet.push(format!(
                            "{name} verification is stale — '{binding}' changed since it passed (was {}, now {})",
                            short(&was), short(&now)
                        ));
                        next_action.get_or_insert(format!(
                            "re-run `codeos-workflow check` to re-verify {name}"
                        ));
                    }
                    RecordState::Current(r) => {
                        met.push(format!(
                            "{name}: verification recorded {} ({})",
                            r.timestamp, r.verification
                        ));
                    }
                    RecordState::Failed(r) => {
                        let reason = r
                            .details
                            .as_ref()
                            .and_then(|v| v.get("validation_error"))
                            .and_then(|v| v.as_str())
                            .or_else(|| {
                                r.details
                                    .as_ref()
                                    .and_then(|v| v.get("evidence"))
                                    .and_then(|v| v.get("observations"))
                                    .and_then(|v| v.as_array())
                                    .and_then(|items| {
                                        items.iter().find(|item| {
                                            item.get("result").and_then(|v| v.as_str())
                                                == Some("failed")
                                        })
                                    })
                                    .and_then(|item| item.get("reason"))
                                    .and_then(|v| v.as_str())
                            })
                            .unwrap_or("latest current verification attempt failed");
                        unmet.push(format!("{name} failed: {reason}"));
                        repair = true;
                        next_action.get_or_insert(format!(
                            "repair the failure and re-run `codeos-workflow check` for {name}"
                        ));
                    }
                }
            }
            Requirement::ObservationStated => {
                if project.operation_workflow_version()? >= 2 {
                    let observation =
                        crate::observations::ObservationStore::at(&project.codeos_dir())
                            .observation(&subject.id)?;
                    if let Some(observation) = observation {
                        met.push(format!(
                            "observation stated: {}",
                            first_line(&observation.statement)
                        ));
                    } else {
                        soft += 1;
                        unmet.push("no Observation v2 record exists".into());
                        next_action.get_or_insert(
                            "record the observation with `codeos-workflow observe`".into(),
                        );
                    }
                    continue;
                }
                let latest = store.latest("operation", &subject.id, "operation_route")?;
                match latest
                    .and_then(|r| r.observation)
                    .map(|o| o.trim().to_string())
                {
                    Some(o) if !o.is_empty() => {
                        met.push(format!("observation stated: {}", first_line(&o)))
                    }
                    _ => {
                        soft += 1;
                        unmet.push("no observation statement recorded".into());
                        next_action.get_or_insert(format!(
                            "codeos-workflow decide --workflow operation --subject {} --checkpoint operation_route --result <route> --observation <text>",
                            subject.id
                        ));
                    }
                }
            }
            Requirement::ObservationRouted => {
                let route = crate::observations::ObservationStore::at(&project.codeos_dir())
                    .route(&subject.id)?;
                if let Some(route) = route {
                    met.push(format!(
                        "routed to {}{}",
                        route.route,
                        route
                            .target_id
                            .as_deref()
                            .map(|v| format!(" ({v})"))
                            .unwrap_or_default()
                    ));
                } else {
                    soft += 1;
                    unmet.push("observation has no route".into());
                    next_action.get_or_insert(
                        "record the route with `codeos-workflow route-observation`".into(),
                    );
                }
            }
            Requirement::RouteResolution => {
                if project.operation_workflow_version()? >= 2 {
                    let closure = crate::observations::ObservationStore::at(&project.codeos_dir())
                        .closure(&subject.id)?;
                    if let Some(closure) = closure {
                        met.push(format!("{} closure evidence validated", closure.route));
                    } else {
                        soft += 1;
                        unmet.push("routed observation has no valid closure evidence".into());
                        next_action.get_or_insert(
                            "close it with `codeos-workflow close-observation` after resolution evidence exists".into(),
                        );
                    }
                    continue;
                }
                let route_receipt = store.latest("operation", &subject.id, "operation_route")?;
                let Some(route_receipt) = route_receipt else {
                    soft += 1;
                    unmet.push("no route recorded yet".into());
                    action_override = Some(ActionType::RouteObservation);
                    next_action.get_or_insert("record the O2 route decision".into());
                    continue;
                };
                let route = route_receipt.result.clone();
                let decided_at = route_receipt.timestamp.clone();
                match route.as_str() {
                    "no_action" => {
                        let closure =
                            store.latest("operation", &subject.id, "no_action_closure")?;
                        let obs_hash = route_receipt.bindings.get("observation").cloned();
                        let ok = closure
                            .map(|c| c.bindings.get("observation").cloned() == obs_hash)
                            .unwrap_or(false);
                        if ok {
                            met.push("no_action closure recorded with rationale".into());
                        } else {
                            soft += 1;
                            unmet.push("no_action route has no closure receipt".into());
                            action_override = Some(ActionType::RecordDecision);
                            next_action.get_or_insert(format!(
                                "codeos-workflow decide --workflow operation --subject {} --checkpoint no_action_closure --result no_action --observation <same text> --rationale <why>",
                                subject.id
                            ));
                        }
                    }
                    "new_feature" | "implementation_defect" | "specification_change" => {
                        let accepted_after = store.all()?.into_iter().any(|r| {
                            r.checkpoint == "acceptance"
                                && r.result == "accepted"
                                && r.subject == subject.id
                                && r.timestamp >= decided_at
                        });
                        if accepted_after {
                            met.push(format!(
                                "{route}: feature {} has an acceptance recorded after the route decision",
                                subject.id
                            ));
                        } else {
                            unmet.push(format!(
                                "{route}: no accepted resolution for {} after the route decision",
                                subject.id
                            ));
                            action_override = Some(ActionType::ResolveRoutedTarget);
                            next_action.get_or_insert(format!(
                                "carry {} through Feature Development to acceptance",
                                subject.id
                            ));
                        }
                    }
                    "charter_change" | "architecture_reassessment" => {
                        soft += 1;
                        unmet.push(format!(
                            "{route}: closure derivation is not automatic for this route"
                        ));
                        action_override = Some(ActionType::ResolveRoutedTarget);
                        next_action.get_or_insert(
                            "link and verify the resolving change through its owning workflow, then re-check".into(),
                        );
                    }
                    other => {
                        unmet.push(format!("recorded route '{other}' is not valid"));
                        action_override = Some(ActionType::RouteObservation);
                        next_action.get_or_insert("re-record O2 with a valid route".into());
                    }
                }
            }
        }
    }
    settle_actor(
        &mut cause,
        &next_action,
        last,
        repair,
        surfaces.take(),
        action_override.take(),
    );

    // Blocked if any unmet condition is a real evidence gap; Waiting if the only thing missing is a
    // human decision the tool must not make itself.
    let hard = unmet.len().saturating_sub(soft);
    let state = if unmet.is_empty() {
        State::Pass
    } else if hard == 0 {
        State::Waiting
    } else {
        State::Blocked
    };

    Ok(CheckpointReport {
        id: c.id.into(),
        title: c.title.into(),
        state,
        met,
        unmet,
        next_action,
        actor: if state == State::Waiting {
            Some(Actor::Human)
        } else {
            cause.as_ref().map(|cause| cause.actor)
        },
        reason: cause.as_ref().map(|cause| cause.reason),
        action: cause.as_ref().map(|cause| cause.action),
        surfaces: cause.and_then(|cause| cause.surfaces),
    })
}

fn verification_surfaces(
    origin: &str,
    record: &VerificationRecord,
    comparisons: Vec<hashing::Comparison>,
) -> Result<Surfaces> {
    Ok(Surfaces::new(
        origin,
        comparisons,
        vec![hashing::EvidenceRef {
            kind: hashing::EvidenceKind::VerificationRecord,
            id: hashing::record_identity(record)?,
        }],
    ))
}

/// Assign the actor and reason of the first next action once the requirement that produced it is
/// known. Both always come from that one requirement.
/// The cause of a checkpoint's next action, all taken from one requirement evaluation.
struct Cause {
    actor: Actor,
    reason: ReasonCode,
    action: ActionType,
    surfaces: Option<Surfaces>,
}

fn settle_actor(
    cause: &mut Option<Cause>,
    next_action: &Option<String>,
    last: Option<&Requirement>,
    repair: bool,
    surfaces: Option<Surfaces>,
    action_override: Option<ActionType>,
) {
    if let (true, Some(_), Some(requirement)) = (cause.is_none(), next_action, last) {
        *cause = Some(Cause {
            actor: if repair {
                Actor::Assisted
            } else {
                Actor::default_for(requirement)
            },
            reason: requirement.reason(),
            action: action_override.unwrap_or_else(|| requirement.action(repair)),
            surfaces,
        });
    }
}

/// `check`: execute every mechanical verification this subject needs whose record is absent or
/// stale, in checkpoint order. On each pass, append a verification record bound to the current
/// governed inputs. Stops at the first failing verification, or at the first checkpoint `check`
/// cannot advance (one blocked on an approval, a receipt, or review — not on a mechanical
/// verification). Returns one human-readable line per verification attempted; writes nothing else.
pub fn run_checks(
    project: &Project,
    subject: &Subject,
    environment: Option<Environment>,
) -> Result<Vec<String>> {
    let environment = stated_environment(project, subject, environment)?;
    let contract = contract_for(
        subject.workflow,
        project.feature_workflow_version()?,
        project.operation_workflow_version()?,
    );
    let mut log = Vec::new();
    if environment == Some(Environment::Production) {
        return run_production_check(project, std::slice::from_ref(subject), &contract, log);
    }

    loop {
        let reports = evaluate(project, subject)?;
        let Some(stuck) = reports
            .iter()
            .find(|r| matches!(r.state, State::Blocked | State::Waiting))
        else {
            break; // every applicable checkpoint is PASS
        };
        if stuck.unmet.iter().any(|u| u.contains("earlier checkpoint")) {
            break; // blocked behind an earlier checkpoint check cannot resolve
        }
        let Some(c) = contract.iter().find(|c| c.id == stuck.id) else {
            break;
        };
        let mechanics = checkpoint_mechanics(c, subject);
        if mechanics.is_empty() {
            break; // this checkpoint waits on a human decision or a non-mechanical condition
        }

        let mut progressed = false;
        let mut production_pending = false;
        for mechanic in mechanics {
            if environment.is_some_and(|env| Environment::of(mechanic) != env) {
                // Feature Workflow v13: never started outside its environment.
                production_pending |= !is_current(project, subject, c.id, mechanic)?;
                continue;
            }
            match execute(project, subject, c.id, mechanic, environment, &mut log)? {
                Step::Current => {}
                Step::Passed => progressed = true,
                Step::Failed => return Ok(log), // fail closed at the first failing verification
            }
        }
        if production_pending {
            // The checkpoint now waits only on production-source evidence: complete the
            // deterministic verifications that follow it, then name the production run.
            for (checkpoint, mechanic) in production_window(&contract, subject, c.id) {
                if checkpoint == c.id {
                    continue;
                }
                if execute(
                    project,
                    subject,
                    checkpoint,
                    mechanic,
                    environment,
                    &mut log,
                )? == Step::Failed
                {
                    return Ok(log);
                }
            }
            log.push(production_next(project, subject, None)?);
            return Ok(log);
        }
        if !progressed {
            break; // nothing left check can do for this checkpoint
        }
    }
    Ok(log)
}

/// Feature Workflow v13: one production `check` for several named features, whose claims one
/// verifier run observes together.
pub fn run_shared_production_check(project: &Project, subjects: &[Subject]) -> Result<Vec<String>> {
    anyhow::ensure!(
        !subjects.is_empty(),
        "a production check names at least one feature"
    );
    let mut contract = None;
    for subject in subjects {
        stated_environment(project, subject, Some(Environment::Production))?;
        contract = Some(contract_for(
            subject.workflow,
            project.feature_workflow_version()?,
            project.operation_workflow_version()?,
        ));
    }
    let ids: BTreeSet<_> = subjects.iter().map(|s| s.id.as_str()).collect();
    anyhow::ensure!(ids.len() == subjects.len(), "a feature is named twice");
    run_production_check(project, subjects, &contract.unwrap_or_default(), Vec::new())
}

/// The environment a feature `check` runs in: stated only under Feature Workflow v13, where it
/// defaults to deterministic; earlier versions keep their single undivided run.
fn stated_environment(
    project: &Project,
    subject: &Subject,
    environment: Option<Environment>,
) -> Result<Option<Environment>> {
    let v13 = subject.workflow == Workflow::Feature && project.feature_workflow_version()? >= 13;
    anyhow::ensure!(
        v13 || environment.is_none(),
        "--environment requires a feature under Feature Workflow v13"
    );
    Ok(v13.then(|| environment.unwrap_or(Environment::Deterministic)))
}

/// The mechanics a checkpoint's verification requirements name, in contract order.
fn checkpoint_mechanics(c: &Checkpoint, subject: &Subject) -> Vec<&'static str> {
    c.requirements
        .iter()
        .filter_map(|r| match r {
            Requirement::LiveVerification { mechanic, .. } => Some(*mechanic),
            Requirement::ProductionSourceEvidence if !subject.reality_claims.is_empty() => {
                Some("production_source_evidence")
            }
            _ => None,
        })
        .collect()
}

/// Feature Workflow v13: run a verification in its stated environment. A shared verification
/// first reuses a passed Validation Evidence Set with its current execution identity and executes
/// nothing; otherwise it runs and stores a new set. Either way the outcome cites the set.
pub fn run_shared(
    project: &Project,
    subject: &Subject,
    mechanic: &str,
    environment: Option<Environment>,
    additions: &[String],
) -> Result<evidence::CommandOutcome> {
    let Some(env) = environment else {
        return Ok(run_mechanic(project, subject, mechanic));
    };
    let Some((identity, inputs)) = shared_execution(project, subject, mechanic, additions)? else {
        return Ok(crate::execution::within(env, || {
            run_mechanic(project, subject, mechanic)
        }));
    };
    if let Some((id, set)) = crate::evidence_set::find_passed(project, &identity)? {
        return Ok(evidence::CommandOutcome {
            ok: true,
            command: set.command,
            summary: format!(
                "reused Validation Evidence Set {} ({})",
                short(&id),
                set.summary
            ),
            details: Some(crate::evidence_set::cite(set.observations, &id)),
        });
    }
    let mut outcome = crate::execution::within(env, || run_mechanic(project, subject, mechanic));
    let id = crate::evidence_set::store(project, mechanic, env, inputs, &outcome)?;
    outcome.details = Some(crate::evidence_set::cite(outcome.details, &id));
    Ok(outcome)
}

/// The execution identity and inputs of a shared verification under Feature Workflow v13, or
/// `None` for one that is not shared.
fn shared_execution(
    project: &Project,
    subject: &Subject,
    mechanic: &str,
    additions: &[String],
) -> Result<Option<(String, BTreeMap<String, String>)>> {
    if subject.workflow != Workflow::Feature
        || project.feature_workflow_version()? < 13
        || !crate::evidence_set::SHARED.contains(&mechanic)
    {
        return Ok(None);
    }
    let mut inputs = crate::evidence_set::common_inputs(project)?;
    if mechanic == "product_journey" {
        // The journey definition `check` passes to the journey, or why it cannot be derived. An
        // underivable definition names its feature: the same error text in two features must
        // never let one feature's journey run serve the other.
        let definition = match journey_definition(project, subject) {
            Ok(definition) => serde_json::to_string(&definition)?,
            Err(error) => format!("underivable for {}: {error}", subject.id),
        };
        inputs.insert(
            "journey_definition".into(),
            hashing::text_sha256(&definition),
        );
        inputs.insert(
            "journey_runner".into(),
            if subject.gui { "browser" } else { "api" }.into(),
        );
    }
    // A qualification's declared dependency additions are execution inputs: changing one must run
    // the qualification again rather than reuse a set made before the change. None adds nothing,
    // so identities without additions are unchanged.
    if mechanic == "project_qualification" {
        bind_dependency_additions(project, &mut inputs, additions)?;
    }
    let identity = crate::evidence_set::identity(mechanic, Environment::Deterministic, &inputs);
    Ok(Some((identity, inputs)))
}

/// Feature Workflow v13: the deterministic verifications a production-source run waits on — its
/// checkpoint's own, then those of each following checkpoint that requires only verifications, up
/// to the first that requires anything else.
fn production_window(
    contract: &[Checkpoint],
    subject: &Subject,
    from: &str,
) -> Vec<(&'static str, &'static str)> {
    let mut window = Vec::new();
    let mut started = false;
    for c in contract {
        if !started {
            if c.id != from {
                continue;
            }
            started = true;
        } else if !applies(c.applies, subject) {
            continue;
        } else if !c
            .requirements
            .iter()
            .all(|r| matches!(r, Requirement::LiveVerification { .. }))
        {
            break;
        }
        for r in &c.requirements {
            if let Requirement::LiveVerification { mechanic, .. } = r {
                window.push((c.id, *mechanic));
            }
        }
    }
    window
}

fn is_current(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    mechanic: &str,
) -> Result<bool> {
    let want = verification_bindings(project, subject, checkpoint, Some(mechanic))?;
    let have = VerificationStore::at(&project.codeos_dir()).latest(
        subject.workflow.slug(),
        &subject.id,
        checkpoint,
        mechanic,
    )?;
    Ok(matches!(
        verification::evaluate(have, &want),
        RecordState::Current(_)
    ))
}

#[derive(Debug, PartialEq, Eq)]
enum Step {
    Current,
    Passed,
    Failed,
}

/// Run one verification unless it is current, and record it. Under Feature Workflow v13 its
/// processes are told the stated environment and its record states it.
fn execute(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    mechanic: &'static str,
    environment: Option<Environment>,
    log: &mut Vec<String>,
) -> Result<Step> {
    let want = verification_bindings(project, subject, checkpoint, Some(mechanic))?;
    let slug = subject.workflow.slug();
    let vstore = VerificationStore::at(&project.codeos_dir());
    let have = vstore.latest(slug, &subject.id, checkpoint, mechanic)?;
    if matches!(verification::evaluate(have, &want), RecordState::Current(_)) {
        return Ok(Step::Current); // already verified against the current inputs
    }
    let additions = declared_additions(project, subject, checkpoint, mechanic)?;
    let outcome = run_shared(project, subject, mechanic, environment, &additions)?;
    project.refresh_snapshot(); // the mechanic ran project code
    record_outcome(
        project,
        subject,
        checkpoint,
        mechanic,
        want,
        outcome,
        environment,
        log,
    )
}

/// Record one verification outcome: a failed product journey as a failed record, a pass as a
/// passed record. Under Feature Workflow v13 the record states its environment.
#[allow(clippy::too_many_arguments)]
fn record_outcome(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    mechanic: &str,
    want: BTreeMap<String, String>,
    outcome: evidence::CommandOutcome,
    environment: Option<Environment>,
    log: &mut Vec<String>,
) -> Result<Step> {
    let slug = subject.workflow.slug();
    let vstore = VerificationStore::at(&project.codeos_dir());
    log.push(format!("{checkpoint} / {mechanic}: {}", outcome.summary));
    let stamped = |details: Option<serde_json::Value>| match environment {
        Some(env) => Some(crate::execution::stamp(details, env)),
        None => details,
    };
    if !outcome.ok {
        if mechanic == "product_journey" && project.workflow_governance_version()? >= 2 {
            vstore.append(&VerificationRecord {
                workflow: slug.to_string(),
                subject: subject.id.clone(),
                checkpoint: checkpoint.to_string(),
                verification: mechanic.to_string(),
                result: "failed".to_string(),
                bindings: want,
                command: outcome.command.clone(),
                timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                details: stamped(outcome.details),
            })?;
        }
        return Ok(Step::Failed);
    }
    let command = if outcome.command.is_empty() {
        mechanic.to_string()
    } else {
        outcome.command.clone()
    };
    let (bindings, details) = recorded_bindings(want, outcome.details);
    vstore.append(&VerificationRecord {
        workflow: slug.to_string(),
        subject: subject.id.clone(),
        checkpoint: checkpoint.to_string(),
        verification: mechanic.to_string(),
        result: "passed".to_string(),
        bindings,
        command,
        timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        details: stamped(details),
    })?;
    Ok(Step::Passed)
}

/// The next command once only production-source evidence is pending: the pre-candidate project
/// qualification it waits on, else the production run itself.
fn production_next(project: &Project, subject: &Subject, trigger: Option<&str>) -> Result<String> {
    let revalidation = trigger
        .map(|trigger| format!(" --revalidation --trigger {trigger}"))
        .unwrap_or_default();
    Ok(if pre_candidate_qualification_current(project, subject)? {
        format!(
            "next: production-source evidence is pending; run `codeos-workflow check --workflow feature --subject {}{revalidation} --environment production`",
            subject.id
        )
    } else {
        format!(
            "next: production-source evidence waits on the pre-candidate project qualification; run `codeos-workflow qualify-project --mode acceptance --subject {}`",
            subject.id
        )
    })
}

fn pre_candidate_qualification_current(project: &Project, subject: &Subject) -> Result<bool> {
    let want =
        verification_bindings_for_mechanic(project, subject, "candidate", "project_qualification")?;
    let have = VerificationStore::at(&project.codeos_dir()).latest(
        "feature",
        &subject.id,
        "candidate",
        "project_qualification",
    )?;
    Ok(matches!(
        verification::evaluate(have, &want),
        RecordState::Current(_)
    ))
}

/// Report every production run that started but never recorded its totals.
fn account_aborted_runs(project: &Project, log: &mut Vec<String>) -> Result<()> {
    for (run, calls) in crate::execution::account_unfinished(project)? {
        log.push(match calls {
            Ok(calls) => format!("provider run {run} ended without accounting: {calls}"),
            Err(error) => format!("provider run {run} ended without accounting and its ledger cannot be counted: {error}"),
        });
    }
    Ok(())
}

/// Feature Workflow v13 production `check`: production-source evidence alone, started only when
/// F5 is the first unresolved checkpoint and everything deterministic it waits on is current.
fn run_production_check(
    project: &Project,
    subjects: &[Subject],
    contract: &[Checkpoint],
    mut log: Vec<String>,
) -> Result<Vec<String>> {
    account_aborted_runs(project, &mut log)?;
    let mut ready = Vec::new();
    for subject in subjects {
        match production_readiness(project, subject, contract)? {
            Ok(checkpoint) => ready.push(checkpoint),
            Err(reason) => log.push(format!(
                "production: started nothing; {}: {reason}",
                subject.id
            )),
        }
    }
    if ready.len() != subjects.len() {
        return Ok(log);
    }
    if let [subject] = subjects {
        execute(
            project,
            subject,
            ready[0],
            "production_source_evidence",
            Some(Environment::Production),
            &mut log,
        )?;
        return Ok(log);
    }
    let wants = subjects
        .iter()
        .map(|subject| {
            verification_bindings(project, subject, "F5", Some("production_source_evidence"))
        })
        .collect::<Result<Vec<_>>>()?;
    let outcomes = run_shared_production(project, subjects, &wants);
    project.refresh_snapshot(); // the verifier ran project code
    for ((subject, want), outcome) in subjects.iter().zip(wants).zip(outcomes) {
        record_outcome(
            project,
            subject,
            "F5",
            "production_source_evidence",
            want,
            outcome,
            Some(Environment::Production),
            &mut log,
        )?;
    }
    Ok(log)
}

/// Feature Workflow v13: the checkpoint at which a feature's production-source evidence may start
/// now, or why it may not and the next command.
fn production_readiness(
    project: &Project,
    subject: &Subject,
    contract: &[Checkpoint],
) -> Result<std::result::Result<&'static str, String>> {
    let deterministic = format!(
        "run `codeos-workflow check --workflow feature --subject {}`",
        subject.id
    );
    let reports = evaluate(project, subject)?;
    let stuck = reports
        .iter()
        .find(|r| matches!(r.state, State::Blocked | State::Waiting));
    let Some(c) = stuck.and_then(|stuck| contract.iter().find(|c| c.id == stuck.id)) else {
        return Ok(Err(
            "no checkpoint waits on production-source evidence".into()
        ));
    };
    if !checkpoint_mechanics(c, subject).contains(&"production_source_evidence")
        || stuck.is_some_and(|s| s.unmet.iter().any(|u| u.contains("earlier checkpoint")))
    {
        return Ok(Err(format!(
            "{} does not wait on production-source evidence. next: {deterministic}",
            c.id
        )));
    }
    let stale: Vec<String> = production_window(contract, subject, c.id)
        .into_iter()
        .filter_map(|(checkpoint, mechanic)| {
            match is_current(project, subject, checkpoint, mechanic) {
                Ok(true) => None,
                Ok(false) => Some(Ok(format!("{checkpoint}/{mechanic}"))),
                Err(error) => Some(Err(error)),
            }
        })
        .collect::<Result<_>>()?;
    if !stale.is_empty() {
        return Ok(Err(format!(
            "deterministic verification is not current: {}. next: {deterministic}",
            stale.join(", ")
        )));
    }
    if !pre_candidate_qualification_current(project, subject)? {
        return Ok(Err(production_next(project, subject, None)?));
    }
    Ok(Ok(c.id))
}

/// The bindings a passed verification records: those it reports itself (v11 production-source
/// evidence binds the Evidence Scopes it observed), else the inputs wanted before it ran.
fn recorded_bindings(
    want: BTreeMap<String, String>,
    details: Option<serde_json::Value>,
) -> (BTreeMap<String, String>, Option<serde_json::Value>) {
    let Some(mut details) = details else {
        return (want, None);
    };
    let reported = details
        .as_object_mut()
        .and_then(|object| object.remove("bindings"))
        .and_then(|value| serde_json::from_value(value).ok());
    (reported.unwrap_or(want), Some(details))
}

/// A (checkpoint, verification, wanted bindings) the revalidation check would execute.
pub type StaleVerification = (&'static str, &'static str, BTreeMap<String, String>);

/// What `check --revalidation` does with one item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanAction {
    Reuse,
    Rerun,
    Blocked,
    DecisionRequired,
}

/// One item of the revalidation plan. `reason` is a stable code: `current`, `absent`, `failed`,
/// `changed:<bound inputs>`, or the reason a claim or decision is not satisfied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanItem {
    pub checkpoint: String,
    pub item: String,
    pub action: PlanAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record: Option<String>,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
}

/// The plan `check --revalidation` executes: every required verification it reuses or reruns, in
/// execution order, then each inherited claim and each human decision the feature still needs.
/// `--plan` serializes it; `check` runs exactly its `rerun` verifications.
#[derive(Debug, Clone, Serialize)]
pub struct RevalidationCheckPlan {
    pub subject: String,
    pub items: Vec<PlanItem>,
    #[serde(skip)]
    pub reruns: Vec<StaleVerification>,
}

/// `check --revalidation --plan`: the same preflight as `check --revalidation`, then its plan.
/// Executes no verification and writes nothing.
pub fn revalidation_check_plan(
    project: &Project,
    subject: &Subject,
    trigger: &str,
) -> Result<RevalidationCheckPlan> {
    anyhow::ensure!(
        subject.workflow == Workflow::Feature,
        "revalidation applies only to features"
    );
    crate::revalidation::preflight(project, &subject.id, trigger)?;
    let mut plan = planned_verifications(project, subject)?;
    let items = &mut plan.items;
    for (claim, state) in crate::revalidation::inherited_evidence(project, subject)? {
        items.push(match state {
            Ok(evidence) => PlanItem {
                checkpoint: "F5".into(),
                item: format!("inherited:{claim}"),
                action: PlanAction::Reuse,
                record: Some(evidence.evidence),
                reason: "current".into(),
                upstream: Some(evidence.feature),
            },
            Err(reason) => PlanItem {
                checkpoint: "F5".into(),
                item: format!("inherited:{claim}"),
                action: PlanAction::Blocked,
                record: None,
                reason,
                upstream: subject
                    .inherited_claims
                    .iter()
                    .find(|c| c.claim == claim)
                    .map(|c| c.upstream_feature.clone()),
            },
        });
    }
    for report in evaluate(project, subject)? {
        if report.state == State::Waiting && report.actor == Some(Actor::Human) {
            items.push(PlanItem {
                checkpoint: report.id.clone(),
                item: "decision".into(),
                action: PlanAction::DecisionRequired,
                record: None,
                reason: match report.reason {
                    Some(code) => serde_json::to_value(code)?
                        .as_str()
                        .unwrap_or_default()
                        .to_owned(),
                    None => String::new(),
                },
                upstream: None,
            });
        }
    }
    Ok(plan)
}

/// The required verifications alone, which `check --revalidation` executes and `status` reads.
fn planned_verifications(project: &Project, subject: &Subject) -> Result<RevalidationCheckPlan> {
    let store = VerificationStore::at(&project.codeos_dir());
    let mut items = Vec::new();
    let mut reruns = Vec::new();
    for (checkpoint, mechanic) in crate::revalidation::required_verifications(project, subject)? {
        if mechanic == "runtime_event_schema" {
            continue; // verified by `verify-events`, never by `check`
        }
        let want = verification_bindings(project, subject, checkpoint, Some(mechanic))?;
        let have = store.latest("feature", &subject.id, checkpoint, mechanic)?;
        let (action, record, reason) = match verification::evaluate(have, &want) {
            RecordState::Current(record) => (
                PlanAction::Reuse,
                Some(hashing::record_identity(&record)?),
                "current".to_string(),
            ),
            RecordState::Absent => (PlanAction::Rerun, None, "absent".to_string()),
            RecordState::Failed(_) => (PlanAction::Rerun, None, "failed".to_string()),
            RecordState::Stale { comparisons, .. } => {
                let changed: Vec<_> = comparisons
                    .iter()
                    .filter(|c| c.changed())
                    .map(|c| c.surface.as_str())
                    .collect();
                (
                    PlanAction::Rerun,
                    None,
                    format!("changed:{}", changed.join(",")),
                )
            }
        };
        if action == PlanAction::Rerun {
            reruns.push((checkpoint, mechanic, want));
        }
        items.push(PlanItem {
            checkpoint: checkpoint.into(),
            item: mechanic.into(),
            action,
            record,
            reason,
            upstream: None,
        });
    }
    Ok(RevalidationCheckPlan {
        subject: subject.id.clone(),
        items,
        reruns,
    })
}

/// The revalidation verifications `check --revalidation` would execute: the plan's reruns.
/// `status` derives its next revalidation step from this same plan.
pub fn stale_revalidation_verifications(
    project: &Project,
    subject: &Subject,
) -> Result<Vec<StaleVerification>> {
    Ok(planned_verifications(project, subject)?.reruns)
}

/// Execute only evidence required by post-acceptance revalidation. This deliberately does not
/// evaluate ordinary F1-F9 gates: Feature Workflow v5 says revalidation does not reopen them.
/// Feature Workflow v13: one production revalidation run for several named features, each of which
/// must wait only on its production-source evidence with qualification current.
pub fn run_shared_revalidation_production(
    project: &Project,
    subjects: &[Subject],
    trigger: &str,
) -> Result<Vec<String>> {
    let ids: BTreeSet<_> = subjects.iter().map(|s| s.id.as_str()).collect();
    anyhow::ensure!(
        subjects.len() > 1 && ids.len() == subjects.len(),
        "a shared revalidation run names at least two distinct features"
    );
    let mut log = Vec::new();
    account_aborted_runs(project, &mut log)?;
    let mut wants = Vec::new();
    for subject in subjects {
        anyhow::ensure!(
            subject.workflow == Workflow::Feature,
            "revalidation applies only to features"
        );
        stated_environment(project, subject, Some(Environment::Production))?;
        crate::revalidation::preflight(project, &subject.id, trigger)?;
        let reruns = planned_verifications(project, subject)?.reruns;
        let (production, pending): (Vec<_>, Vec<_>) = reruns
            .into_iter()
            .partition(|(_, mechanic, _)| Environment::of(mechanic) == Environment::Production);
        let reason = if !pending.is_empty() {
            let stale: Vec<_> = pending.iter().map(|(c, m, _)| format!("{c}/{m}")).collect();
            Some(format!(
                "deterministic verification is not current: {}. next: run `codeos-workflow check --workflow feature --subject {} --revalidation --trigger {trigger}`",
                stale.join(", "),
                subject.id
            ))
        } else if production.is_empty() {
            Some("no production-source evidence is pending".into())
        } else if !pre_candidate_qualification_current(project, subject)? {
            Some(production_next(project, subject, Some(trigger))?)
        } else {
            None
        };
        match reason {
            Some(reason) => log.push(format!(
                "production: started nothing; {}: {reason}",
                subject.id
            )),
            None => wants.extend(production.into_iter().map(|(_, _, want)| want)),
        }
    }
    if wants.len() != subjects.len() {
        return Ok(log);
    }
    let outcomes = run_shared_production(project, subjects, &wants);
    project.refresh_snapshot(); // the verifier ran project code
    for ((subject, want), outcome) in subjects.iter().zip(wants).zip(outcomes) {
        record_outcome(
            project,
            subject,
            "F5",
            "production_source_evidence",
            want,
            outcome,
            Some(Environment::Production),
            &mut log,
        )?;
    }
    Ok(log)
}

pub fn run_revalidation_checks(
    project: &Project,
    subject: &Subject,
    trigger: &str,
    environment: Option<Environment>,
) -> Result<Vec<String>> {
    anyhow::ensure!(
        subject.workflow == Workflow::Feature,
        "revalidation applies only to features"
    );
    let environment = stated_environment(project, subject, environment)?;
    crate::revalidation::preflight(project, &subject.id, trigger)?;
    let mut log = Vec::new();
    let reruns = planned_verifications(project, subject)?.reruns;
    // Feature Workflow v13: each environment runs only its own reruns; production waits until
    // every deterministic rerun and the pre-candidate project qualification are current.
    let (runnable, pending): (Vec<_>, Vec<_>) = reruns.into_iter().partition(|(_, mechanic, _)| {
        environment.is_none_or(|env| Environment::of(mechanic) == env)
    });
    if environment == Some(Environment::Production) {
        account_aborted_runs(project, &mut log)?;
        if !pending.is_empty() {
            let stale: Vec<_> = pending.iter().map(|(c, m, _)| format!("{c}/{m}")).collect();
            log.push(format!(
                "production: started nothing; deterministic verification is not current: {}. next: run `codeos-workflow check --workflow feature --subject {} --revalidation --trigger {trigger}`",
                stale.join(", "),
                subject.id
            ));
            return Ok(log);
        }
        if !runnable.is_empty() && !pre_candidate_qualification_current(project, subject)? {
            log.push(format!(
                "production: started nothing; {}",
                production_next(project, subject, Some(trigger))?
            ));
            return Ok(log);
        }
    }
    let store = VerificationStore::at(&project.codeos_dir());
    for (checkpoint, mechanic, want) in runnable {
        let additions = declared_additions(project, subject, checkpoint, mechanic)?;
        let outcome = run_shared(project, subject, mechanic, environment, &additions)?;
        project.refresh_snapshot(); // the mechanic ran project code
        log.push(format!("{checkpoint} / {mechanic}: {}", outcome.summary));
        if !outcome.ok {
            return Ok(log);
        }
        let (bindings, details) = recorded_bindings(want, outcome.details);
        store.append(&VerificationRecord {
            workflow: "feature".into(),
            subject: subject.id.clone(),
            checkpoint: checkpoint.into(),
            verification: mechanic.into(),
            result: "passed".into(),
            bindings,
            command: if outcome.command.is_empty() {
                mechanic.into()
            } else {
                outcome.command
            },
            timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            details: match environment {
                Some(env) => Some(crate::execution::stamp(details, env)),
                None => details,
            },
        })?;
    }
    if environment == Some(Environment::Deterministic) && !pending.is_empty() {
        log.push(production_next(project, subject, Some(trigger))?);
    }
    Ok(log)
}

/// The bindings a receipt for `checkpoint` should currently be evaluated against. Public so
/// `decide` records exactly what `status`/`check` will later re-check.
pub fn bindings_for_decision(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
) -> Result<BTreeMap<String, String>> {
    let bindings = decision_scope(current_bindings(project, subject, checkpoint)?);
    anyhow::ensure!(
        bindings.get("observed_surface").map(String::as_str) != Some(UNOBSERVED),
        "no passing F3 product_journey record carries an Observed Surface; run check first"
    );
    anyhow::ensure!(
        !(project.workflow_governance_version()? >= 3
            && matches!(checkpoint, "final_ux_validation" | "acceptance")
            && !bindings.contains_key("candidate_bundle")),
        "no unique current Candidate Bundle; run prepare-candidate after F6"
    );
    Ok(bindings)
}

/// A new human UX judgment binds the narrowest product binding available: the Observed Surface
/// (Feature Workflow v10), else the product files, never the test files that verify it.
fn decision_scope(mut bindings: BTreeMap<String, String>) -> BTreeMap<String, String> {
    if let Some(narrowest) = PRODUCT_BINDINGS
        .iter()
        .position(|key| bindings.contains_key(*key))
    {
        for broader in &PRODUCT_BINDINGS[narrowest + 1..] {
            bindings.remove(*broader);
        }
    }
    bindings
}

/// The product bindings, narrowest first. Each later one covers what the earlier ones judged.
const PRODUCT_BINDINGS: [&str; 4] = [
    "observed_surface",
    "delivered_state",
    "product_state",
    "implementation_state",
];

/// The inputs a recorded decision must bind to be current: what `decide` records now. A receipt
/// recorded under an earlier version keeps the broader product binding it recorded, which is
/// compared as recorded, so it needs no narrower one.
fn required_decision_bindings(
    current: &BTreeMap<String, String>,
    receipt: Option<&Receipt>,
) -> BTreeSet<String> {
    let mut required: BTreeSet<String> = decision_scope(current.clone()).into_keys().collect();
    if let Some(receipt) = receipt {
        for (index, narrower) in PRODUCT_BINDINGS.iter().enumerate() {
            if !receipt.bindings.contains_key(*narrower)
                && PRODUCT_BINDINGS[index + 1..]
                    .iter()
                    .any(|broader| receipt.bindings.contains_key(*broader))
            {
                required.remove(*narrower);
            }
        }
    }
    required
}

/// Feature Workflow v10: a GUI feature's journey records its Observed Surface, which F4 and F8 bind.
fn observes_surface(project: &Project, subject: &Subject) -> Result<bool> {
    Ok(subject.workflow == Workflow::Feature
        && subject.gui
        && project.feature_workflow_version()? >= 10)
}

/// The Observed Surface of the latest F3 journey record. A record without one yields a value no
/// receipt binds, so a receipt cannot be current against it and `decide` refuses it.
fn journey_observed_surface(project: &Project, subject: &Subject) -> Result<String> {
    let record = VerificationStore::at(&project.codeos_dir()).latest(
        "feature",
        &subject.id,
        "F3",
        "product_journey",
    )?;
    Ok(record
        .filter(|record| record.result == "passed")
        .and_then(|record| record.details)
        .and_then(|details| details["observed_surface"].as_str().map(str::to_string))
        .unwrap_or_else(|| UNOBSERVED.into()))
}

const UNOBSERVED: &str = "unobserved";

/// The bindings `status`/`check` evaluate a recorded receipt against. They include every binding
/// any recorded receipt for `checkpoint` may carry, so receipts recorded before product scoping keep
/// their exact behavior.
pub fn bindings_for_evaluation(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
) -> Result<BTreeMap<String, String>> {
    current_bindings(project, subject, checkpoint)
}

/// DBA-22 and later scope F4/F8 receipts to product files (Workflow Governance v1: bindings name
/// the inputs the decision depended on).
fn product_scoped_ux_receipts(project: &Project) -> bool {
    project
        .dba_configuration()
        .ok()
        .and_then(|id| id.strip_prefix("DBA-").and_then(|n| n.parse::<u32>().ok()))
        .is_some_and(|n| n >= 22)
}

/// The bindings a receipt for `checkpoint` should currently be evaluated against.
fn current_bindings(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
) -> Result<BTreeMap<String, String>> {
    let mut b = BTreeMap::new();
    let put = |b: &mut BTreeMap<String, String>, k: &str, path: &Path| -> Result<()> {
        if let Some(h) = hashing::file_sha256(path)? {
            b.insert(k.to_string(), h);
        }
        Ok(())
    };
    match checkpoint {
        "initial_product_preview" => {
            // Bound to the two governed inputs the platform-direction call actually judges: the
            // Charter (product direction) and codeos.yaml (Platform Baseline selection). It is
            // deliberately NOT bound to working-tree implementation_state — every feature increment
            // has its own Early Development Preview, and implementing a feature must not invalidate
            // the one-time confirmation that the baseline direction is right.
            put(&mut b, "charter", &project.charter())?;
            put(&mut b, "codeos_yaml", &project.codeos_yaml())?;
        }
        "early_preview" => {
            put(&mut b, "intent", &project.intent(&subject.id))?;
            put(&mut b, "contract", &project.contract(&subject.id))?;
            put(&mut b, "event_schema", &project.event_schema(&subject.id))?;
            b.insert(
                "implementation_state".into(),
                project.implementation_state()?,
            );
            if product_scoped_ux_receipts(project) {
                b.insert("product_state".into(), project.product_state()?);
            }
            if observes_surface(project, subject)? {
                b.insert(
                    "observed_surface".into(),
                    journey_observed_surface(project, subject)?,
                );
            }
        }
        "reconciliation" | "final_ux_validation" | "acceptance" => {
            put(&mut b, "intent", &project.intent(&subject.id))?;
            put(&mut b, "contract", &project.contract(&subject.id))?;
            put(&mut b, "event_schema", &project.event_schema(&subject.id))?;
            if project.feature_workflow_version()? >= 4 {
                put(&mut b, "codeos_yaml", &project.codeos_yaml())?;
                b.insert(
                    "implementation_state".into(),
                    project.implementation_state()?,
                );
                if checkpoint == "final_ux_validation" && product_scoped_ux_receipts(project) {
                    b.insert("product_state".into(), project.product_state()?);
                }
                // Feature Workflow v11: F6 binds the product, never the tests that verify it; v12
                // binds what users receive, and still evaluates a v11 receipt as recorded.
                if checkpoint == "reconciliation" && project.feature_workflow_version()? >= 11 {
                    b.insert("product_state".into(), project.product_state()?);
                }
                if checkpoint == "reconciliation" && project.feature_workflow_version()? >= 12 {
                    b.insert(
                        "delivered_state".into(),
                        project.delivered_state(&project.implementation_paths())?,
                    );
                }
                if checkpoint == "final_ux_validation" && observes_surface(project, subject)? {
                    b.insert(
                        "observed_surface".into(),
                        journey_observed_surface(project, subject)?,
                    );
                }
            }
            if project.workflow_governance_version()? >= 3
                && matches!(checkpoint, "final_ux_validation" | "acceptance")
            {
                // Feature Workflow v14: the bundle is recorded, not a dependency, so currency does
                // not need one; `decide` still does (bindings_for_decision).
                match crate::candidate::unique_current(project, &subject.id)? {
                    Some(bundle) => {
                        b.insert("candidate_bundle".into(), bundle);
                    }
                    None if crate::validity::active(project)? => {}
                    None => anyhow::bail!(
                        "no unique current Candidate Bundle; run prepare-candidate after F6"
                    ),
                }
            }
        }
        "operation_route" | "no_action_closure" => {
            // Bound to the observation statement's hash, supplied at decide time; nothing to
            // recompute from the tree here.
        }
        _ => {}
    }
    if subject.workflow == Workflow::Feature
        && crate::validity::active(project)?
        && crate::validity::dependencies(checkpoint).is_some()
    {
        v14_bindings(project, subject, checkpoint, &mut b)?;
    }
    Ok(b)
}

/// Feature Workflow v14: the meaning identities, judged claims and architecture identity a
/// checkpoint's dependency set names, added to what v13 records.
fn v14_bindings(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    b: &mut BTreeMap<String, String>,
) -> Result<()> {
    let deps = crate::validity::dependencies(checkpoint).unwrap_or(&[]);
    for (artifact, key) in crate::validity::ARTIFACTS {
        if !deps.contains(&key) {
            continue;
        }
        let path = crate::validity::artifact_path(project, &subject.id, artifact)
            .expect("known artifact");
        if let Some(identity) =
            crate::validity::resolved_meaning(project, &subject.id, artifact, &path)?
        {
            b.insert(key.into(), identity);
        }
    }
    match checkpoint {
        "early_preview" => {
            b.insert(
                "judged_claims".into(),
                crate::validity::judged_claims(project, subject, "F4")?,
            );
        }
        "final_ux_validation" => {
            b.insert(
                "judged_claims".into(),
                crate::validity::judged_claims(project, subject, "F8")?,
            );
        }
        "acceptance" => {
            b.insert(
                "architecture".into(),
                crate::architecture::feature_identity(project, &subject.id, "WORKTREE")?,
            );
        }
        _ => {}
    }
    Ok(())
}

/// The state of a subject's latest decision at `checkpoint` against the current inputs it must
/// bind: the one rule `check` applies and the reviewer's readiness gate reports.
/// Feature Workflow v13: a feature's prospective validation claims, checked against the claim
/// meanings its latest acceptance's Candidate Bundle recorded; `None` for legacy coarse claims.
pub fn subject_claims(
    project: &Project,
    subject: &Subject,
) -> Result<Option<Vec<crate::claims::ValidationClaim>>> {
    if subject.workflow != Workflow::Feature {
        return Ok(None);
    }
    let contract = contract_for(
        subject.workflow,
        project.feature_workflow_version()?,
        project.operation_workflow_version()?,
    );
    let verifications = contract
        .iter()
        .filter(|c| applies(c.applies, subject))
        .map(|c| {
            let required = c
                .requirements
                .iter()
                .filter_map(|r| match r {
                    Requirement::LiveVerification { mechanic, .. } => Some(mechanic.to_string()),
                    _ => None,
                })
                .collect();
            (c.id.to_string(), required)
        })
        .collect();
    let Some(claims) = crate::claims::validation_claims(
        project,
        &subject.id,
        &verifications,
        &subject.reality_claims,
    )?
    else {
        return Ok(None);
    };
    let accepted = ReceiptStore::at(&project.codeos_dir())
        .latest("feature", &subject.id, "acceptance")?
        .and_then(|receipt| receipt.bindings.get("candidate_bundle").cloned())
        .map(|bundle| crate::candidate::CandidateStore::at(&project.codeos_dir()).read(&bundle))
        .transpose()?
        .map(|bundle| bundle.claim_digests)
        .unwrap_or_default();
    let changed = crate::claims::changed_meanings(&claims, &accepted);
    anyhow::ensure!(
        changed.is_empty(),
        "validation claim {} changed meaning since its acceptance; declare a successor ID",
        changed.join(", ")
    );
    Ok(Some(claims))
}

/// The `observation:human` claims at a checkpoint that the receipt's session does not state as
/// `met`.
fn human_claims_not_met(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    receipt: &Receipt,
) -> Result<Vec<String>> {
    let mut missing = Vec::new();
    for claim in subject_claims(project, subject)?.unwrap_or_default() {
        if claim.checkpoint == checkpoint && claim.evidence == crate::claims::Evidence::Human {
            let outcome = crate::session::claim_outcome(project, receipt, &claim.id)?;
            if outcome.as_deref() != Some("met") {
                missing.push(claim.id);
            }
        }
    }
    Ok(missing)
}

pub fn decision_state(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
) -> Result<ReceiptState> {
    let mut bindings = current_bindings(project, subject, checkpoint)?;
    let latest = ReceiptStore::at(&project.codeos_dir()).latest(
        subject.workflow.slug(),
        &subject.id,
        checkpoint,
    )?;
    let mut required = required_decision_bindings(&bindings, latest.as_ref());
    // The observation binding cannot be recomputed from the tree — the observation statement is a
    // historical, immutable fact. Carry the recorded hash forward so it reads as current; it exists
    // to link a closure to the same observation, not to detect drift.
    if let Some(h) = latest.as_ref().and_then(|r| r.bindings.get("observation")) {
        bindings.insert("observation".into(), h.clone());
    }
    // Feature Workflow v13: a session is a sealed fact like an observation. It reads as current
    // only while it is intact and states exactly this receipt's outcome on its surface.
    if let Some(receipt) = latest.as_ref() {
        if let Some(id) = receipt.bindings.get("session") {
            if crate::session::attesting(project, receipt)?.is_some() {
                bindings.insert("session".into(), id.clone());
            }
        }
    }
    if let Some(h) = selected_production_evidence(project, subject)? {
        if matches!(
            checkpoint,
            "reconciliation" | "final_ux_validation" | "acceptance"
        ) {
            bindings.insert("production_evidence".into(), h);
            // `decide` requires F6 to select it; F8 and F9 do not record it.
            if checkpoint == "reconciliation" {
                required.insert("production_evidence".into());
            }
        }
    }
    // Feature Workflow v14: a receipt carrying v14 dependency keys is compared on its dependency
    // set only; every other recorded value is history and carried forward as recorded.
    if let (Some(deps), Some(receipt)) = (
        crate::validity::dependencies(checkpoint),
        latest.as_ref(),
    ) {
        if subject.workflow == Workflow::Feature
            && crate::validity::active(project)?
            && receipt.bindings.contains_key("intent_meaning")
        {
            let mut dependency_set: BTreeSet<String> =
                deps.iter().map(|key| key.to_string()).collect();
            if crate::validity::judges_product(checkpoint) {
                if let Some(narrowest) = PRODUCT_BINDINGS
                    .iter()
                    .find(|key| receipt.bindings.contains_key(**key))
                {
                    dependency_set.insert(narrowest.to_string());
                }
            }
            for (key, value) in &receipt.bindings {
                if !dependency_set.contains(key) {
                    bindings.insert(key.clone(), value.clone());
                }
            }
            required.retain(|key| dependency_set.contains(key));
        }
    }
    let state = receipts::evaluate_required(latest, &bindings, &required);
    if checkpoint == "reconciliation" && subject.workflow == Workflow::Feature {
        if let ReceiptState::Stale {
            receipt,
            comparisons,
            ..
        } = &state
        {
            if revalidation_satisfies_reconciliation(project, &subject.id, comparisons)? {
                return Ok(ReceiptState::Current(receipt.clone()));
            }
        }
    }
    Ok(state)
}

/// Feature Workflow v14: an accepted feature's F6 receipt that differs only in its product binding
/// is satisfied by a PASSED Post-Acceptance Dependency Revalidation of its latest acceptance.
fn revalidation_satisfies_reconciliation(
    project: &Project,
    feature: &str,
    comparisons: &[hashing::Comparison],
) -> Result<bool> {
    if !crate::validity::active(project)? {
        return Ok(false);
    }
    let only_product = comparisons
        .iter()
        .filter(|c| c.changed())
        .all(|c| PRODUCT_BINDINGS.contains(&c.surface.as_str()));
    if !only_product {
        return Ok(false);
    }
    let Some(acceptance) = ReceiptStore::at(&project.codeos_dir())
        .latest("feature", feature, "acceptance")?
        .filter(|r| r.result == "accepted")
    else {
        return Ok(false);
    };
    crate::candidate::passed_revalidation(project, feature, &hashing::record_identity(&acceptance)?)
}

fn selected_production_evidence(project: &Project, subject: &Subject) -> Result<Option<String>> {
    if subject.reality_claims.is_empty() {
        return Ok(None);
    }
    let store = ReceiptStore::at(&project.codeos_dir());
    Ok(store
        .latest(subject.workflow.slug(), &subject.id, "reconciliation")?
        .and_then(|receipt| receipt.bindings.get("production_evidence").cloned()))
}

pub fn validate_production_evidence_selection(
    project: &Project,
    subject: &Subject,
    identity: &str,
) -> Result<()> {
    if identity.len() != 64 || !identity.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("production evidence identity must be a sha256 hex digest");
    }
    let want = verification_bindings(project, subject, "F5", Some("production_source_evidence"))?;
    let store = VerificationStore::at(&project.codeos_dir());
    let matching = store.all()?.into_iter().rev().find(|record| {
        record.workflow == subject.workflow.slug()
            && record.subject == subject.id
            && record.checkpoint == "F5"
            && record.verification == "production_source_evidence"
            && record.result == "passed"
            && record
                .details
                .as_ref()
                .and_then(|details| details.get("evidence_sha256"))
                .and_then(|value| value.as_str())
                == Some(identity)
    });
    let mut want = want;
    if project.feature_workflow_version()? >= 11 {
        scope_production_bindings(project, &mut want, Some(identity))?;
    }
    match verification::evaluate(matching, &want) {
        RecordState::Current(_) => Ok(()),
        RecordState::Absent => anyhow::bail!("selected production evidence does not exist"),
        RecordState::Failed(_) => anyhow::bail!("selected production evidence did not pass"),
        RecordState::Stale { binding, .. } => {
            anyhow::bail!("selected production evidence is stale because '{binding}' changed")
        }
    }
}

/// The governed inputs a mechanical-verification record for `checkpoint` must be bound to. Public
/// for `check` and for tests that seed verification records directly.
pub fn verification_bindings_for(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
) -> Result<BTreeMap<String, String>> {
    verification_bindings(project, subject, checkpoint, None)
}

/// The bindings a recorded verification must currently match: a runtime-event record's own
/// derivation (its F7 inputs and the validator it names), every other record's mechanic bindings.
pub fn current_record_bindings(
    project: &Project,
    subject: &Subject,
    record: &VerificationRecord,
) -> Result<BTreeMap<String, String>> {
    if record.verification == "runtime_event_schema" {
        let validator = crate::event_evidence::recorded_validator(record)?;
        return crate::event_evidence::runtime_event_bindings(project, subject, validator);
    }
    verification_bindings_for_mechanic(project, subject, &record.checkpoint, &record.verification)
}

pub fn verification_bindings_for_mechanic(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    mechanic: &str,
) -> Result<BTreeMap<String, String>> {
    verification_bindings(project, subject, checkpoint, Some(mechanic))
}

/// Binding-key prefix for a dependency addition declared to a project qualification.
const DEPENDENCY_ADDITION: &str = "dependency_addition:";

/// The dependency additions a project-qualification record declared.
pub fn declared_dependency_additions(record: &VerificationRecord) -> Vec<String> {
    record
        .bindings
        .keys()
        .filter_map(|key| key.strip_prefix(DEPENDENCY_ADDITION))
        .map(String::from)
        .collect()
}

/// Project-qualification bindings for an explicit set of dependency additions, used where the set
/// is declared on the command line rather than taken from the latest record.
pub fn qualification_bindings(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    additions: &[String],
) -> Result<BTreeMap<String, String>> {
    bindings_with(
        project,
        subject,
        checkpoint,
        Some("project_qualification"),
        Some(additions),
    )
}

/// The dependency additions the latest record of a project qualification declared; none for any
/// other mechanic.
fn declared_additions(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    mechanic: &str,
) -> Result<Vec<String>> {
    if mechanic != "project_qualification" {
        return Ok(Vec::new());
    }
    Ok(VerificationStore::at(&project.codeos_dir())
        .latest(subject.workflow.slug(), &subject.id, checkpoint, mechanic)?
        .map(|record| declared_dependency_additions(&record))
        .unwrap_or_default())
}

/// Binds each addition's current content; a removed file binds nothing, so its record is stale.
fn bind_dependency_additions(
    project: &Project,
    b: &mut BTreeMap<String, String>,
    additions: &[String],
) -> Result<()> {
    for path in additions {
        if let Some(h) = hashing::file_sha256(&project.root.join(path))? {
            b.insert(format!("{DEPENDENCY_ADDITION}{path}"), h);
        }
    }
    Ok(())
}

fn verification_bindings(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    mechanic: Option<&str>,
) -> Result<BTreeMap<String, String>> {
    bindings_with(project, subject, checkpoint, mechanic, None)
}

/// `additions` is an explicitly declared qualification set; `None` takes the latest record's.
fn bindings_with(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    mechanic: Option<&str>,
    additions: Option<&[String]>,
) -> Result<BTreeMap<String, String>> {
    let additions = match additions {
        Some(additions) => additions.to_vec(),
        None => match mechanic {
            Some(mechanic) => declared_additions(project, subject, checkpoint, mechanic)?,
            None => Vec::new(),
        },
    };
    let mut b = BTreeMap::new();
    let put = |b: &mut BTreeMap<String, String>, k: &str, path: &Path| -> Result<()> {
        if let Some(h) = hashing::file_sha256(path)? {
            b.insert(k.to_string(), h);
        }
        Ok(())
    };
    match checkpoint {
        // Bootstrap's integrated baseline. Bound to the resolved Platform Baseline only: a platform
        // change forces a re-baseline. Deliberately NOT bound to implementation_state — Bootstrap
        // is a completed phase, and from the first feature onward the live integration guarantee is
        // carried by each feature's own verification (F3 / F5 / F5d / F5g).
        "B3" => {
            put(&mut b, "codeos_yaml", &project.codeos_yaml())?;
        }
        // Every feature mechanical verification: the approved Specification Package plus the current
        // implementation. Revising the spec, or changing the implementation, re-blocks it.
        "F3" | "F5" | "F5d" | "F5g" | "F7" | "F9" | "candidate" => {
            put(&mut b, "intent", &project.intent(&subject.id))?;
            put(&mut b, "contract", &project.contract(&subject.id))?;
            put(&mut b, "event_schema", &project.event_schema(&subject.id))?;
            if subject.workflow == Workflow::Feature && crate::validity::active(project)? {
                // Feature Workflow v14: a verification also binds the specification's meaning
                // identity (R3, R4) and is compared on it, so a non-semantic edit re-runs nothing.
                for (artifact, key) in crate::validity::ARTIFACTS {
                    let path = crate::validity::artifact_path(project, &subject.id, artifact)
                        .expect("known artifact");
                    if let Some(identity) =
                        crate::validity::resolved_meaning(project, &subject.id, artifact, &path)?
                    {
                        b.insert(key.into(), identity);
                    }
                }
            }
            b.insert(
                "implementation_state".into(),
                project.implementation_state()?,
            );
        }
        _ => {}
    }
    if mechanic == Some("product_journey") {
        put(&mut b, "charter", &project.charter())?;
        b.insert(
            "accepted_frontier".into(),
            hashing::text_sha256(&accepted_frontier_excluding(project, Some(&subject.id))?),
        );
        // Under v10 the GUI journey also records its Observed Surface; binding that into the target
        // makes a journey record made before v10 stale, so it is re-run.
        let mut target = product_journey_target(project, subject)?;
        if observes_surface(project, subject)? {
            target.push_str(" observed-surface/1");
        }
        b.insert("journey_target".into(), hashing::text_sha256(&target));
        if project.codeos_mechanics_version()? == 5 {
            let definition = journey_definition(project, subject)?;
            for transition in definition.transitions {
                if let Some(relative) = transition.authority_path {
                    put(
                        &mut b,
                        &format!("architecture:{}", transition.authority_id),
                        &project.root.join(relative),
                    )?;
                }
            }
        }
    }
    if mechanic == Some("project_qualification") {
        put(&mut b, "charter", &project.charter())?;
        put(
            &mut b,
            "qualification_script",
            &project.root.join("scripts/codeos-qualify"),
        )?;
        put(&mut b, "codeos_yaml", &project.codeos_yaml())?;
        // Declared dependency additions are re-hashed, so changing one stales the record.
        bind_dependency_additions(project, &mut b, &additions)?;
    }
    if mechanic == Some("production_source_evidence") && project.feature_workflow_version()? >= 11 {
        let latest = VerificationStore::at(&project.codeos_dir())
            .latest(
                subject.workflow.slug(),
                &subject.id,
                checkpoint,
                "production_source_evidence",
            )?
            .and_then(|record| evidence_identity(&record));
        scope_production_bindings(project, &mut b, latest.as_deref())?;
    }
    // Feature Workflow v13: a shared verification's record binds its execution identity.
    if let Some(mechanic) = mechanic {
        if let Some((identity, _)) = shared_execution(project, subject, mechanic, &additions)? {
            b.insert("execution_identity".into(), identity);
        }
    }
    Ok(b)
}

/// Feature Workflow v11: production-source evidence binds the Evidence Scopes it observed and the
/// verifier that produced it, in place of the whole implementation state.
fn scope_production_bindings(
    project: &Project,
    b: &mut BTreeMap<String, String>,
    evidence: Option<&str>,
) -> Result<()> {
    b.remove("implementation_state");
    b.insert(
        "reality_verifier".into(),
        reality_verifier_identity(project)?,
    );
    let paths = project.evidence_scope_paths(&observed_scope_paths(project, evidence)?);
    let state = if project.feature_workflow_version()? >= 12 {
        project.delivered_state(&paths)?
    } else {
        project.scoped_product_state(&paths)?
    };
    b.insert("evidence_scope_state".into(), state);
    Ok(())
}

/// The content identity of the configured verifier; an unresolvable one binds nothing a record can
/// match, so its evidence is stale.
fn reality_verifier_identity(project: &Project) -> Result<String> {
    Ok(project
        .reality_verification()
        .ok()
        .and_then(|config| {
            hashing::file_sha256(&project.root.join(config.command))
                .ok()
                .flatten()
        })
        .unwrap_or_else(|| "unresolved".into()))
}

fn evidence_identity(record: &VerificationRecord) -> Option<String> {
    record.details.as_ref()?["evidence_sha256"]
        .as_str()
        .map(str::to_string)
}

/// The scope paths a piece of production evidence observed: the union of its claims' declared
/// scopes, or every implementation path when any claim names none or anything is unknown.
fn observed_scope_paths(project: &Project, evidence: Option<&str>) -> Result<Vec<String>> {
    let broad = project.implementation_paths();
    let Some(id) = evidence else {
        return Ok(broad);
    };
    let path = project
        .codeos_dir()
        .join(format!("06-workflow/reality-evidence/{id}.json"));
    let Some(record) = std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<ProductionEvidence>(&text).ok())
    else {
        return Ok(broad);
    };
    let Ok(config) = project.reality_verification() else {
        return Ok(broad);
    };
    let mut paths = BTreeSet::new();
    for claim in &record.claims {
        let Some(names) = record
            .claim_scopes
            .get(claim)
            .filter(|names| !names.is_empty())
        else {
            return Ok(broad);
        };
        for name in names {
            let Some(scope) = config.scopes.get(name) else {
                return Ok(broad);
            };
            paths.extend(scope.iter().cloned());
        }
    }
    Ok(paths.into_iter().collect())
}

pub fn semantic_package_revision(project: &Project, feature: &str) -> Result<u64> {
    let paths = [
        project.intent(feature),
        project.contract(feature),
        project.event_schema(feature),
    ];
    let mut revisions = Vec::new();
    let mut predecessors = Vec::new();
    let mut changes = Vec::new();
    for path in paths {
        let text =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let frontmatter = text
            .strip_prefix("---\n")
            .and_then(|rest| rest.split_once("\n---"))
            .map(|(frontmatter, _)| frontmatter)
            .context("missing YAML frontmatter")?;
        let value: serde_yaml::Value = serde_yaml::from_str(frontmatter)?;
        let revision = value
            .get("semantic_revision")
            .and_then(serde_yaml::Value::as_u64)
            .context("semantic_revision must be a positive integer")?;
        revisions.push(revision);
        predecessors.push(
            value
                .get("semantic_revision_from")
                .and_then(serde_yaml::Value::as_u64),
        );
        changes.push(
            value
                .get("semantic_change")
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string(),
        );
    }
    if revisions.iter().any(|revision| *revision != revisions[0]) {
        anyhow::bail!("Intent, Contract, and Event Schema revisions differ");
    }
    if predecessors.iter().any(|value| *value != predecessors[0])
        || changes.iter().any(|value| value != &changes[0])
        || changes[0].len() < 12
    {
        anyhow::bail!("semantic revision lineage or semantic_change declarations differ");
    }
    if let Some(from) = predecessors[0] {
        if from + 1 != revisions[0] {
            anyhow::bail!("semantic_revision must increment exactly once from its predecessor");
        }
    }
    Ok(revisions[0])
}

/// Validate `codeos.yaml` against the active configuration's mechanics policy. Runs the authoritative
/// shell check when the toolkit is reachable, and falls back to a structural check otherwise. This
/// is a live predicate — evaluated the same way by every command.
fn validate_codeos_yaml(project: &Project, yaml: &Path) -> Option<bool> {
    let self_dev = project
        .root
        .join("dba/04-tools/configuration/project-config-contract.sh");
    let downstream = project
        .codeos_dir()
        .join("toolkit/dba/04-tools/configuration/project-config-contract.sh");
    if let Some(script) = [self_dev, downstream].into_iter().find(|p| p.exists()) {
        if let Ok(o) = crate::instrument::command("bash")
            .arg(&script)
            .arg(yaml)
            .output()
        {
            return Some(o.status.success());
        }
    }
    // Toolkit not reachable from here: structural check rather than a hard fail. The authoritative
    // validation still runs wherever the toolkit is present.
    let text = std::fs::read_to_string(yaml).ok()?;
    Some(text.contains("platform:") && text.contains("artifacts:"))
}

fn run_mechanic(project: &Project, subject: &Subject, mechanic: &str) -> evidence::CommandOutcome {
    // Test seam: a `.codeos/06-workflow/.verify-stub` file of `mechanic=pass|fail` lines forces
    // outcomes without executing anything; `pass;key=value;...` also reports those details (a
    // replay supplies the Observed Surface a real run recorded). A real project never has this
    // file; it is per-project, so parallel tests do not interfere.
    let stub = project
        .codeos_dir()
        .join("06-workflow")
        .join(".verify-stub");
    if let Ok(spec) = std::fs::read_to_string(&stub) {
        for entry in spec.split([',', '\n']) {
            if let Some((m, v)) = entry.split_once('=') {
                if m.trim() == mechanic {
                    let mut parts = v.trim().split(';');
                    let result = parts.next().unwrap_or_default();
                    let details: serde_json::Map<_, _> = parts
                        .filter_map(|pair| pair.split_once('='))
                        .map(|(k, v)| (k.to_string(), serde_json::Value::from(v)))
                        .collect();
                    return evidence::CommandOutcome {
                        ok: result == "pass",
                        command: format!("<stub {mechanic}>"),
                        summary: format!("{mechanic} (stubbed {result})"),
                        details: (!details.is_empty())
                            .then_some(serde_json::Value::Object(details)),
                    };
                }
            }
        }
    }
    if matches!(mechanic, "baseline" | "smoke" | "product_journey" | "behavior" | "repeatability" | "data_integrity") {
        match project.runtime_verification() {
            Ok(runtime) => {
                if let Some(command) = runtime.verification_command.as_deref() {
                    return evidence::run(&project.root, &project.root.join(command).to_string_lossy(), &[mechanic]);
                }
            }
            Err(error) => return failure(error.to_string()),
        }
    }
    let web = project.root.join("web");
    match mechanic {
        "baseline" => run_integrated_baseline(project),
        "smoke" => run_smoke(project),
        "product_journey" => run_product_journey(project, subject),
        "production_source_evidence" => run_production_source_evidence(project, subject),
        "project_qualification" => crate::qualification::run(project, "acceptance"),
        "behavior" | "repeatability" => run_behavior_tests(project, mechanic),
        "data_integrity" => run_cargo_tests(project),
        "playwright" => {
            let runtime = match project.runtime_verification() {
                Ok(runtime) => runtime,
                Err(error) => return failure(error.to_string()),
            };
            if !web.is_dir() {
                return failure("no web/ to verify");
            }
            run_playwright(project, &web, runtime.web_url.as_str())
        }
        other => evidence::CommandOutcome {
            ok: false,
            command: String::new(),
            summary: format!("unknown mechanic {other}"),
            details: None,
        },
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductionEvidence {
    schema: u8,
    subject: String,
    claims: Vec<String>,
    source_kind: String,
    provider: String,
    observed_at: String,
    /// Schema 1 only: the implementation state the evidence was bound to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    implementation_sha256: Option<String>,
    governed_inputs: BTreeMap<String, String>,
    /// Schema 2 (Feature Workflow v11): claim -> the declared Evidence Scopes it observed.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    claim_scopes: BTreeMap<String, Vec<String>>,
    observations: Vec<ProductionObservation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductionObservation {
    claim_ids: Vec<String>,
    visible_result: serde_json::Value,
    visible_result_sha256: String,
    provider_receipts_sha256: String,
    provider_request_response_receipts: Vec<ProviderReceipt>,
    details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderReceipt {
    provider_host: String,
    provider_path: String,
    request_sha256: String,
    response_sha256: String,
    #[serde(default)]
    provider_request_id: Option<String>,
    requested_at: String,
    received_at: String,
    http_status: u16,
}

/// The evidence a shared production run returns (verifier definition schema 3): every named
/// feature's claims, observed together. Observation claim IDs are owner-qualified.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SharedProductionEvidence {
    schema: u8,
    subjects: Vec<SharedSubjectEvidence>,
    source_kind: String,
    provider: String,
    observed_at: String,
    observations: Vec<ProductionObservation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SharedSubjectEvidence {
    subject: String,
    claims: Vec<String>,
    governed_inputs: BTreeMap<String, String>,
    #[serde(default)]
    claim_scopes: BTreeMap<String, Vec<String>>,
}

/// Feature Workflow v13 shared production run: one verifier run over the union of the named
/// features' owner-qualified claims, stored as one Validation Evidence Set. Each feature's outcome
/// is its own evidence slice: its claims, its governed inputs, and the observations of its claims.
fn run_shared_production(
    project: &Project,
    subjects: &[Subject],
    wants: &[BTreeMap<String, String>],
) -> Vec<evidence::CommandOutcome> {
    let fail_all = |summary: String| {
        subjects
            .iter()
            .map(|_| failure(summary.clone()))
            .collect::<Vec<_>>()
    };
    let config = match project.reality_verification() {
        Ok(config) => config,
        Err(error) => return fail_all(error.to_string()),
    };
    let governed: Vec<BTreeMap<String, String>> = wants
        .iter()
        .map(|want| {
            let mut governed = want.clone();
            governed.remove("evidence_scope_state");
            governed
        })
        .collect();
    let definition = serde_json::json!({
        "schema": 3,
        "subjects": subjects.iter().zip(&governed).map(|(subject, governed)| serde_json::json!({
            "subject": subject.id,
            "claims": subject.reality_claims,
            "governed_inputs": governed,
        })).collect::<Vec<_>>(),
        "scopes": config.scopes,
    });
    let nonce = format!(
        "{}-shared-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let definition_path = std::env::temp_dir().join(format!("codeos-reality-{nonce}.json"));
    let output_path = std::env::temp_dir().join(format!("codeos-reality-evidence-{nonce}.json"));
    let definition_text = serde_json::to_string_pretty(&definition).unwrap_or_default();
    if let Err(error) = std::fs::write(&definition_path, &definition_text) {
        return fail_all(format!("write reality definition: {error}"));
    }
    let (run, ledger) = match crate::execution::start_run(project, "shared") {
        Ok(started) => started,
        Err(error) => return fail_all(error.to_string()),
    };
    let definition_env = definition_path.to_string_lossy().into_owned();
    let output_env = output_path.to_string_lossy().into_owned();
    let ledger_env = ledger.to_string_lossy().into_owned();
    let outcome = crate::execution::within(Environment::Production, || {
        evidence::run_env(
            &project.root,
            &config.command,
            &[],
            &[
                ("CODEOS_REALITY_DEFINITION_PATH", definition_env.as_str()),
                ("CODEOS_REALITY_EVIDENCE_PATH", output_env.as_str()),
                ("CODEOS_REALITY_MODE", "production-source"),
                ("CODEOS_PROVIDER_RUN_ID", run.as_str()),
                ("CODEOS_PROVIDER_LEDGER", ledger_env.as_str()),
            ],
        )
    });
    let calls = crate::execution::count(&ledger).and_then(|calls| {
        crate::execution::finish_run(&ledger, &calls)?;
        Ok(calls)
    });
    let raw = std::fs::read_to_string(&output_path);
    let _ = std::fs::remove_file(&definition_path);
    let _ = std::fs::remove_file(&output_path);
    let accounted = |summary: String| match &calls {
        Ok(calls) => format!("{summary} [provider calls: {calls}]"),
        Err(_) => summary,
    };
    let checked = raw
        .map_err(|error| anyhow::anyhow!("verifier produced no readable evidence: {error}"))
        .and_then(|text| {
            let evidence: SharedProductionEvidence = serde_json::from_str(&text)
                .map_err(|error| anyhow::anyhow!("invalid production evidence: {error}"))?;
            anyhow::ensure!(outcome.ok, "production verifier exited unsuccessfully");
            let gaps = check_shared_production(&evidence, subjects, &governed, &config.scopes)?;
            let calls = calls
                .as_ref()
                .map_err(|error| anyhow::anyhow!("provider calls cannot be accounted: {error}"))?;
            let receipts: BTreeSet<_> = evidence
                .observations
                .iter()
                .flat_map(|observation| &observation.provider_request_response_receipts)
                .map(|receipt| serde_json::to_string(receipt).unwrap_or_default())
                .collect();
            anyhow::ensure!(
                calls.successful == receipts.len() as u64,
                "{} successful provider calls do not reconcile with {} provider receipts",
                calls.successful,
                receipts.len()
            );
            Ok((evidence, text, *calls, gaps))
        });
    let (evidence, text, calls, gaps) = match checked {
        Ok(checked) => checked,
        Err(error) => {
            return fail_all(accounted(format!(
                "{} -> FAILED ({error})",
                outcome.command
            )))
        }
    };
    let set_outcome = evidence::CommandOutcome {
        ok: true,
        command: outcome.command.clone(),
        summary: accounted(outcome.summary.clone()),
        details: Some(serde_json::json!({
            "evidence": serde_json::from_str::<serde_json::Value>(&text).unwrap_or_default(),
            "provider_calls": calls,
        })),
    };
    let inputs = BTreeMap::from([(
        "verifier_definition".to_string(),
        hashing::text_sha256(&definition_text),
    )]);
    let set = match crate::evidence_set::store(
        project,
        "production_source_evidence",
        Environment::Production,
        inputs,
        &set_outcome,
    ) {
        Ok(set) => set,
        Err(error) => return fail_all(format!("store Validation Evidence Set: {error}")),
    };
    subjects
        .iter()
        .zip(governed)
        .zip(gaps)
        .map(|((subject, governed), gap)| match gap {
            // The run is shared; each feature's validity is not.
            Some(gap) => failure(format!(
                "{} -> FAILED for {} ({gap}; Validation Evidence Set {})",
                set_outcome.summary,
                subject.id,
                short(&set)
            )),
            None => {
                slice_production_evidence(project, subject, governed, &evidence, &set, &set_outcome)
                    .unwrap_or_else(|error| {
                        failure(format!("persist production evidence: {error}"))
                    })
            }
        })
        .collect()
}

/// A shared production run's evidence: run-level defects (form, source, provenance, a claim no
/// named feature owns, an unnamed feature) fail the whole run; each named feature whose own entry
/// or claim coverage is wrong fails alone, with the reason returned in its position.
fn check_shared_production(
    evidence: &SharedProductionEvidence,
    subjects: &[Subject],
    governed: &[BTreeMap<String, String>],
    scopes: &BTreeMap<String, Vec<String>>,
) -> Result<Vec<Option<String>>> {
    anyhow::ensure!(evidence.schema == 3, "evidence schema does not match");
    anyhow::ensure!(
        evidence.source_kind == "production" && !evidence.provider.trim().is_empty(),
        "evidence source must be an identified production source"
    );
    anyhow::ensure!(
        !evidence.observed_at.trim().is_empty() && !evidence.observations.is_empty(),
        "evidence requires observation time and user-visible observations"
    );
    let named: BTreeSet<_> = subjects.iter().map(|s| s.id.as_str()).collect();
    let mut stated = BTreeSet::new();
    for entry in &evidence.subjects {
        anyhow::ensure!(
            named.contains(entry.subject.as_str()) && stated.insert(entry.subject.as_str()),
            "evidence states feature {} that the run did not name, or states it twice",
            entry.subject
        );
    }
    let owned: BTreeSet<String> = subjects
        .iter()
        .flat_map(|s| {
            s.reality_claims
                .iter()
                .map(move |c| format!("{}/{c}", s.id))
        })
        .collect();
    let covered: BTreeSet<String> = evidence
        .observations
        .iter()
        .flat_map(|observation| observation.claim_ids.iter().cloned())
        .collect();
    if let Some(foreign) = covered.difference(&owned).next() {
        anyhow::bail!("production observation names '{foreign}', which no named feature claims");
    }
    check_observations(&evidence.observations)?;
    Ok(subjects
        .iter()
        .zip(governed)
        .map(|(subject, governed)| {
            subject_evidence_gap(evidence, subject, governed, scopes, &covered)
        })
        .collect())
}

/// Why a shared run's evidence does not establish one named feature, if it does not.
fn subject_evidence_gap(
    evidence: &SharedProductionEvidence,
    subject: &Subject,
    governed: &BTreeMap<String, String>,
    scopes: &BTreeMap<String, Vec<String>>,
    covered: &BTreeSet<String>,
) -> Option<String> {
    let Some(entry) = evidence.subjects.iter().find(|e| e.subject == subject.id) else {
        return Some("evidence does not state this feature".into());
    };
    let claims: BTreeSet<_> = subject.reality_claims.iter().cloned().collect();
    let actual: BTreeSet<_> = entry.claims.iter().cloned().collect();
    if actual != claims || entry.claims.len() != actual.len() {
        return Some("evidence does not cover the exact reality-dependent claim set".into());
    }
    if entry.governed_inputs != *governed {
        return Some("evidence bindings do not match the governed inputs".into());
    }
    for (claim, names) in &entry.claim_scopes {
        let unique: BTreeSet<_> = names.iter().collect();
        if !claims.contains(claim)
            || unique.len() != names.len()
            || !names.iter().all(|name| scopes.contains_key(name))
        {
            return Some(format!(
                "claim '{claim}' states an undeclared or duplicated Evidence Scope"
            ));
        }
    }
    let missing: Vec<_> = claims
        .iter()
        .filter(|claim| !covered.contains(&format!("{}/{claim}", subject.id)))
        .cloned()
        .collect();
    (!missing.is_empty()).then(|| {
        format!(
            "production observations do not cover claim {}",
            missing.join(", ")
        )
    })
}

/// One feature's slice of a shared production run, persisted as its own production-source
/// evidence: its claims, governed inputs and Evidence Scopes, and the observations of its claims
/// with their claim IDs in the feature's own form.
fn slice_production_evidence(
    project: &Project,
    subject: &Subject,
    governed: BTreeMap<String, String>,
    evidence: &SharedProductionEvidence,
    set: &str,
    set_outcome: &evidence::CommandOutcome,
) -> Result<evidence::CommandOutcome> {
    let entry = evidence
        .subjects
        .iter()
        .find(|entry| entry.subject == subject.id)
        .context("a named feature is missing from the shared evidence")?;
    let prefix = format!("{}/", subject.id);
    let observations = evidence
        .observations
        .iter()
        .filter_map(|observation| {
            let own: Vec<String> = observation
                .claim_ids
                .iter()
                .filter_map(|id| id.strip_prefix(&prefix).map(str::to_string))
                .collect();
            (!own.is_empty()).then(|| ProductionObservation {
                claim_ids: own,
                ..observation.clone()
            })
        })
        .collect();
    let slice = ProductionEvidence {
        schema: 2,
        subject: subject.id.clone(),
        claims: entry.claims.clone(),
        source_kind: evidence.source_kind.clone(),
        provider: evidence.provider.clone(),
        observed_at: evidence.observed_at.clone(),
        implementation_sha256: None,
        governed_inputs: governed.clone(),
        claim_scopes: entry.claim_scopes.clone(),
        observations,
    };
    let canonical = serde_json::to_string(&slice)?;
    let identity = hashing::text_sha256(&canonical);
    let dir = project.codeos_dir().join("06-workflow/reality-evidence");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(format!("{identity}.json")), &canonical)?;
    let mut recorded = governed;
    scope_production_bindings(project, &mut recorded, Some(&identity))?;
    Ok(evidence::CommandOutcome {
        ok: true,
        command: set_outcome.command.clone(),
        summary: format!(
            "{} (shared run, Validation Evidence Set {})",
            set_outcome.summary,
            short(set)
        ),
        details: Some(serde_json::json!({
            "evidence_sha256": identity,
            "bindings": recorded,
            "evidence_set": set,
        })),
    })
}

fn run_production_source_evidence(
    project: &Project,
    subject: &Subject,
) -> evidence::CommandOutcome {
    let config = match project.reality_verification() {
        Ok(config) => config,
        Err(error) => return failure(error.to_string()),
    };
    let bindings =
        match verification_bindings(project, subject, "F5", Some("production_source_evidence")) {
            Ok(bindings) => bindings,
            Err(error) => return failure(error.to_string()),
        };
    let version = match project.feature_workflow_version() {
        Ok(version) => version,
        Err(error) => return failure(error.to_string()),
    };
    let v11 = version >= 11;
    // The verifier receives the inputs known before it runs; the Evidence Scope state follows
    // from the scopes the new evidence states it observed.
    let mut governed = bindings.clone();
    governed.remove("evidence_scope_state");
    let nonce = format!(
        "{}-{}-{}",
        std::process::id(),
        subject.id,
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let definition_path = std::env::temp_dir().join(format!("codeos-reality-{nonce}.json"));
    let output_path = std::env::temp_dir().join(format!("codeos-reality-evidence-{nonce}.json"));
    let definition = if v11 {
        serde_json::json!({
            "schema": 2,
            "subject": subject.id,
            "claims": subject.reality_claims,
            "governed_inputs": governed,
            "scopes": config.scopes,
        })
    } else {
        serde_json::json!({
            "schema": 1,
            "subject": subject.id,
            "claims": subject.reality_claims,
            "governed_inputs": governed,
        })
    };
    if let Err(error) = std::fs::write(
        &definition_path,
        serde_json::to_vec_pretty(&definition).unwrap_or_default(),
    ) {
        return failure(format!("write reality definition: {error}"));
    }
    let definition_text = definition_path.to_string_lossy().into_owned();
    let output_text = output_path.to_string_lossy().into_owned();
    // Feature Workflow v13: the run is accounted from the adapter's per-attempt ledger, which
    // exists before the verifier starts so an abort leaves it discoverable.
    let ledger = if version >= 13 {
        match crate::execution::start_run(project, &subject.id) {
            Ok((run, path)) => Some((run, path.to_string_lossy().into_owned(), path)),
            Err(error) => return failure(error.to_string()),
        }
    } else {
        None
    };
    let mut env = vec![
        ("CODEOS_REALITY_DEFINITION_PATH", definition_text.as_str()),
        ("CODEOS_REALITY_EVIDENCE_PATH", output_text.as_str()),
        ("CODEOS_REALITY_MODE", "production-source"),
    ];
    if let Some((run, path, _)) = &ledger {
        env.push(("CODEOS_PROVIDER_RUN_ID", run.as_str()));
        env.push(("CODEOS_PROVIDER_LEDGER", path.as_str()));
    }
    let mut outcome = evidence::run_env(&project.root, &config.command, &[], &env);
    let calls = ledger.as_ref().map(|(_, _, path)| {
        crate::execution::count(path).and_then(|calls| {
            crate::execution::finish_run(path, &calls)?;
            Ok(calls)
        })
    });
    let parsed = std::fs::read_to_string(&output_path)
        .map_err(|error| anyhow::anyhow!("verifier produced no readable evidence: {error}"))
        .and_then(|text| {
            serde_json::from_str::<ProductionEvidence>(&text)
                .map_err(|error| anyhow::anyhow!("invalid production evidence: {error}"))
        });
    let _ = std::fs::remove_file(&definition_path);
    let _ = std::fs::remove_file(&output_path);
    let expected_implementation = bindings
        .get("implementation_state")
        .cloned()
        .unwrap_or_default();
    let mut outcome = match parsed.and_then(|record| {
        if !outcome.ok {
            anyhow::bail!("production verifier exited unsuccessfully");
        }
        if record.schema != if v11 { 2 } else { 1 } || record.subject != subject.id {
            anyhow::bail!("evidence schema or subject does not match");
        }
        let expected: BTreeSet<_> = subject.reality_claims.iter().cloned().collect();
        for (claim, names) in &record.claim_scopes {
            if !v11 || !expected.contains(claim) {
                anyhow::bail!("Evidence Scopes are stated for an uncovered claim '{claim}'");
            }
            let unique: BTreeSet<_> = names.iter().collect();
            if unique.len() != names.len()
                || names.iter().any(|name| !config.scopes.contains_key(name))
            {
                anyhow::bail!("claim '{claim}' states an undeclared or duplicated Evidence Scope");
            }
        }
        let actual: BTreeSet<_> = record.claims.iter().cloned().collect();
        if actual != expected || record.claims.len() != actual.len() {
            anyhow::bail!("evidence does not cover the exact reality-dependent claim set");
        }
        if record.source_kind != "production" || record.provider.trim().is_empty() {
            anyhow::bail!("evidence source must be an identified production source");
        }
        if record.observed_at.trim().is_empty() || record.observations.is_empty() {
            anyhow::bail!("evidence requires observation time and user-visible observations");
        }
        let covered: BTreeSet<_> = record
            .observations
            .iter()
            .flat_map(|observation| observation.claim_ids.iter().cloned())
            .collect();
        if covered != expected {
            anyhow::bail!("production observations do not cover the exact claim set");
        }
        check_observations(&record.observations)?;
        let implementation_bound = if v11 {
            record.implementation_sha256.is_none()
        } else {
            record.implementation_sha256.as_deref() == Some(expected_implementation.as_str())
        };
        if !implementation_bound || record.governed_inputs != governed {
            anyhow::bail!(
                "evidence bindings do not match the current implementation and governed inputs"
            );
        }
        if let Some(calls) = &calls {
            let calls = calls
                .as_ref()
                .map_err(|error| anyhow::anyhow!("provider calls cannot be accounted: {error}"))?;
            let receipts: BTreeSet<_> = record
                .observations
                .iter()
                .flat_map(|observation| &observation.provider_request_response_receipts)
                .map(|receipt| serde_json::to_string(receipt).unwrap_or_default())
                .collect();
            if calls.successful != receipts.len() as u64 {
                anyhow::bail!(
                    "{} successful provider calls do not reconcile with {} provider receipts",
                    calls.successful,
                    receipts.len()
                );
            }
        }
        Ok(record)
    }) {
        Ok(record) => {
            let canonical = serde_json::to_string(&record).unwrap_or_default();
            let identity = hashing::text_sha256(&canonical);
            let dir = project.codeos_dir().join("06-workflow/reality-evidence");
            if let Err(error) = std::fs::create_dir_all(&dir)
                .and_then(|_| std::fs::write(dir.join(format!("{identity}.json")), &canonical))
            {
                return failure(format!("persist production evidence: {error}"));
            }
            // The evidence file is the canonical copy; the record references it by identity.
            outcome.details = Some(serde_json::json!({ "evidence_sha256": identity }));
            if v11 {
                // The record binds the Evidence Scopes this evidence observed.
                let mut recorded = governed;
                if let Err(error) =
                    scope_production_bindings(project, &mut recorded, Some(&identity))
                {
                    return failure(format!("bind Evidence Scopes: {error}"));
                }
                outcome.details = Some(serde_json::json!({
                    "evidence_sha256": identity,
                    "bindings": recorded,
                }));
            }
            outcome
        }
        Err(error) => {
            outcome.ok = false;
            outcome.summary = format!("{} -> FAILED ({error})", outcome.command);
            outcome.details = Some(serde_json::json!({ "validation_error": error.to_string() }));
            outcome
        }
    };
    if let Some(Ok(calls)) = &calls {
        outcome.summary = format!("{} [provider calls: {calls}]", outcome.summary);
        if let Some(serde_json::Value::Object(details)) = outcome.details.as_mut() {
            details.insert("provider_calls".into(), serde_json::json!(calls));
        }
    }
    outcome
}

/// The content binding and provider provenance every production observation must carry.
fn check_observations(observations: &[ProductionObservation]) -> Result<()> {
    for observation in observations {
        if !is_sha256(&observation.visible_result_sha256)
            || !is_sha256(&observation.provider_receipts_sha256)
            || observation.provider_request_response_receipts.is_empty()
        {
            anyhow::bail!("production observation lacks content-bound visible/provider evidence");
        }
        if hashing::text_sha256(&serde_json::to_string(&observation.visible_result)?)
            != observation.visible_result_sha256
            || hashing::text_sha256(&serde_json::to_string(
                &observation.provider_request_response_receipts,
            )?) != observation.provider_receipts_sha256
        {
            anyhow::bail!("production observation content hashes do not match their evidence");
        }
        for receipt in &observation.provider_request_response_receipts {
            if receipt.provider_host.trim().is_empty()
                || !receipt.provider_path.starts_with('/')
                || !is_sha256(&receipt.request_sha256)
                || !is_sha256(&receipt.response_sha256)
                || receipt.requested_at.trim().is_empty()
                || receipt.received_at.trim().is_empty()
                || !(200..300).contains(&receipt.http_status)
            {
                anyhow::bail!("provider request/response provenance is incomplete or invalid");
            }
        }
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn run_product_journey(project: &Project, subject: &Subject) -> evidence::CommandOutcome {
    if project.codeos_mechanics_version().unwrap_or(4) != 5 {
        return run_legacy_product_journey(project, subject);
    }
    let definition = match journey_definition(project, subject) {
        Ok(definition) => definition,
        Err(error) => return journey_failure(error.to_string()),
    };
    let runtime = match project.runtime_verification() {
        Ok(runtime) => runtime,
        Err(error) => return journey_failure(error.to_string()),
    };
    let nonce = format!(
        "{}-{}-{}",
        std::process::id(),
        subject.id,
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let temp = std::env::temp_dir();
    let definition_path = temp.join(format!("codeos-journey-definition-{nonce}.json"));
    let evidence_path = temp.join(format!("codeos-journey-evidence-{nonce}.json"));
    if let Err(error) = std::fs::write(
        &definition_path,
        serde_json::to_vec_pretty(&definition).unwrap(),
    ) {
        return journey_failure(format!("write derived journey definition: {error}"));
    }
    let definition_text = definition_path.to_string_lossy().into_owned();
    let evidence_text = evidence_path.to_string_lossy().into_owned();
    let target = definition.target_frontier.clone();
    let env = [
        ("PLAYWRIGHT_BASE_URL", runtime.web_url.as_str()),
        ("CODEOS_EXPECTED_FRONTIER", target.as_str()),
        ("CODEOS_JOURNEY_DEFINITION_PATH", definition_text.as_str()),
        ("CODEOS_JOURNEY_EVIDENCE_PATH", evidence_text.as_str()),
    ];
    let (cwd, program, args): (&Path, String, Vec<String>) = if subject.gui {
        if !project.web_has_script("test:journey") {
            let _ = std::fs::remove_file(&definition_path);
            return journey_failure("web/package.json has no test:journey script");
        }
        let tag = format!("@frontier:{target}");
        (
            &project.root.join("web"),
            "npm".into(),
            vec![
                "run".into(),
                "test:journey".into(),
                "--".into(),
                "--grep".into(),
                tag,
            ],
        )
    } else {
        let script = project.root.join("scripts/test-product-journey.sh");
        if !script.is_file() {
            let _ = std::fs::remove_file(&definition_path);
            return journey_failure("GUI-exempt project has no scripts/test-product-journey.sh");
        }
        (
            &project.root,
            "bash".into(),
            vec!["scripts/test-product-journey.sh".into(), target.clone()],
        )
    };
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut outcome = if let Some(guard) = runtime.journey_guard_script.as_deref() {
        let guard_path = project.root.join(guard).to_string_lossy().into_owned();
        let mut guarded = vec!["--", program.as_str()];
        guarded.extend(arg_refs.iter().copied());
        evidence::run_env(cwd, &guard_path, &guarded, &env)
    } else {
        evidence::run_env(cwd, &program, &arg_refs, &env)
    };
    let parsed = std::fs::read_to_string(&evidence_path)
        .map_err(|error| anyhow::anyhow!("journey runner produced no readable evidence: {error}"))
        .and_then(|text| {
            serde_json::from_str::<JourneyEvidence>(&text)
                .map_err(|error| anyhow::anyhow!("invalid journey evidence: {error}"))
        });
    let _ = std::fs::remove_file(&definition_path);
    let _ = std::fs::remove_file(&evidence_path);
    match parsed.and_then(|observed| {
        validate_journey_evidence(&definition, &observed, outcome.ok)?;
        Ok(observed)
    }) {
        Ok(observed) => {
            let passed = observed.observations.len() == definition.transitions.len()
                && observed
                    .observations
                    .iter()
                    .all(|item| item.result == "passed");
            outcome.ok = passed;
            outcome.details = Some(serde_json::json!({
                "definition": definition,
                "evidence": observed
            }));
            if let Some(failed) = observed
                .observations
                .iter()
                .find(|item| item.result == "failed")
            {
                outcome.summary = format!(
                    "{} -> FAILED at {} ({})",
                    outcome.command,
                    failed.transition_id,
                    failed.reason.as_deref().unwrap_or("no reason")
                );
            }
            outcome
        }
        Err(error) => {
            outcome.ok = false;
            outcome.summary = format!("{} -> FAILED ({error})", outcome.command);
            outcome.details = Some(serde_json::json!({
                "definition": definition,
                "validation_error": error.to_string()
            }));
            outcome
        }
    }
}

fn journey_failure(summary: impl Into<String>) -> evidence::CommandOutcome {
    let summary = summary.into();
    evidence::CommandOutcome {
        ok: false,
        command: String::new(),
        details: Some(serde_json::json!({ "validation_error": summary })),
        summary,
    }
}

fn validate_journey_evidence(
    definition: &JourneyDefinition,
    evidence: &JourneyEvidence,
    command_ok: bool,
) -> Result<()> {
    if evidence.schema != 1 || evidence.journey_id != definition.journey_id {
        anyhow::bail!(
            "journey evidence schema or journey_id does not match the derived definition"
        );
    }
    if evidence.target_frontier != definition.target_frontier {
        anyhow::bail!("journey evidence target does not match the expected frontier");
    }
    for setup in &evidence.setup {
        if !matches!(
            setup.category.as_str(),
            "infrastructure"
                | "actor_identity"
                | "static_reference_data"
                | "external_boundary_simulation"
        ) || setup.resource.trim().is_empty()
        {
            anyhow::bail!("journey setup contains an unsupported category or empty resource");
        }
    }
    if evidence.observations.len() > definition.transitions.len() {
        anyhow::bail!("journey evidence contains more transitions than the derived definition");
    }
    let mut saw_failure = false;
    for (index, observed) in evidence.observations.iter().enumerate() {
        let expected = &definition.transitions[index];
        if observed.transition_id != expected.transition_id {
            anyhow::bail!("journey observation order or transition identity differs from the derived definition");
        }
        if observed.state_origin != "production" {
            anyhow::bail!(
                "{} was not created through its production capability",
                observed.transition_id
            );
        }
        match observed.result.as_str() {
            "passed" if !saw_failure => {
                if observed.handoff_identity.as_deref().map(str::trim).unwrap_or("").is_empty() {
                    anyhow::bail!("passed transition {} has no observed handoff identity", observed.transition_id);
                }
                if observed.reason.is_some() {
                    anyhow::bail!("passed transition {} must not report a failure reason", observed.transition_id);
                }
            }
            "failed" if !saw_failure && index + 1 == evidence.observations.len() => {
                if observed.reason.as_deref().map(str::trim).unwrap_or("").is_empty() {
                    anyhow::bail!("failed transition {} has no reason", observed.transition_id);
                }
                saw_failure = true;
            }
            _ => anyhow::bail!("journey observations must be an ordered passed prefix followed by at most one failure"),
        }
    }
    let complete_pass = evidence.observations.len() == definition.transitions.len() && !saw_failure;
    if command_ok != complete_pass {
        anyhow::bail!("journey command exit status contradicts its structured observations");
    }
    if !complete_pass && !saw_failure {
        anyhow::bail!("incomplete journey evidence does not identify the first failed transition");
    }
    Ok(())
}

fn run_legacy_product_journey(project: &Project, subject: &Subject) -> evidence::CommandOutcome {
    let target = match product_journey_target(project, subject) {
        Ok(target) => target,
        Err(error) => return failure(error.to_string()),
    };
    if subject.gui {
        if !project.web_has_script("test:journey") {
            return failure("web/package.json has no test:journey script");
        }
        let runtime = match project.runtime_verification() {
            Ok(runtime) => runtime,
            Err(error) => return failure(error.to_string()),
        };
        if observes_surface(project, subject).unwrap_or(false) {
            return run_observed_journey(project, runtime.web_url.as_str(), &target);
        }
        let tag = format!("@frontier:{target}");
        return evidence::run_env(
            &project.root.join("web"),
            "npm",
            &["run", "test:journey", "--", "--grep", &tag],
            &[
                ("PLAYWRIGHT_BASE_URL", runtime.web_url.as_str()),
                ("CODEOS_EXPECTED_FRONTIER", target.as_str()),
            ],
        );
    }
    let script = project.root.join("scripts/test-product-journey.sh");
    if !script.is_file() {
        return failure("GUI-exempt project has no scripts/test-product-journey.sh");
    }
    evidence::run(
        &project.root,
        "bash",
        &["scripts/test-product-journey.sh", &target],
    )
}

/// Feature Workflow v10: run the GUI journey twice, each recording its Observed Surface. Both runs
/// must pass and observe the same identity. Only this governed journey, which the project runs in
/// its deterministic provider mode, is repeated; production-source evidence never is.
fn run_observed_journey(
    project: &Project,
    base_url: &str,
    target: &str,
) -> evidence::CommandOutcome {
    let tag = format!("@frontier:{target}");
    let mut identities = Vec::new();
    let mut last = None;
    for _ in 0..2 {
        let path = report_path("observed-surface");
        let path_text = path.to_string_lossy().into_owned();
        let _ = std::fs::remove_file(&path);
        let outcome = evidence::run_env(
            &project.root.join("web"),
            "npm",
            &["run", "test:journey", "--", "--grep", &tag],
            &[
                ("PLAYWRIGHT_BASE_URL", base_url),
                ("CODEOS_EXPECTED_FRONTIER", target),
                ("CODEOS_OBSERVED_SURFACE_PATH", path_text.as_str()),
            ],
        );
        let observed = read_observations(&path).and_then(|lines| observed_surface_identity(&lines));
        let _ = std::fs::remove_file(&path);
        if !outcome.ok {
            return outcome;
        }
        match observed {
            Ok(identity) => identities.push(identity),
            Err(error) => {
                return failure(format!(
                    "{} -> no valid Observed Surface: {error:#}",
                    outcome.command
                ))
            }
        }
        last = Some(outcome);
    }
    let mut outcome = last.expect("two journey runs");
    if identities[0] != identities[1] {
        outcome.ok = false;
        outcome.summary = format!(
            "{} -> the Observed Surface differs between two runs; the journey is not deterministic",
            outcome.command
        );
        return outcome;
    }
    outcome.details = Some(serde_json::json!({ "observed_surface": identities[0] }));
    outcome
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SurfaceObservation {
    schema: u8,
    state: String,
    viewport: SurfaceViewport,
    elements: Vec<SurfaceElement>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SurfaceViewport {
    width: u32,
    height: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SurfaceElement {
    name: String,
    aria: String,
    #[serde(rename = "box")]
    bounds: SurfaceBox,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SurfaceBox {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// The journey's observation lines, only from the regular file Codeos named.
fn read_observations(path: &Path) -> Result<String> {
    let metadata = std::fs::symlink_metadata(path)
        .context("the journey wrote no observation file at the named path")?;
    anyhow::ensure!(
        metadata.file_type().is_file(),
        "the observation path is not a regular file"
    );
    Ok(std::fs::read_to_string(path)?)
}

/// The Observed Surface identity of a journey's observation lines: accessibility text trimmed,
/// geometry rounded to whole CSS pixels (a fixed tolerance), observations ordered by state and
/// viewport so parallel runs agree. A state observed twice at one viewport must observe the same
/// thing; a contradiction fails rather than being normalized away.
fn observed_surface_identity(lines: &str) -> Result<String> {
    let mut normalized: BTreeMap<(String, u32, u32), String> = BTreeMap::new();
    for line in lines.lines().filter(|line| !line.trim().is_empty()) {
        let observation: SurfaceObservation =
            serde_json::from_str(line).context("invalid Observed Surface line")?;
        anyhow::ensure!(
            observation.schema == 1,
            "unsupported Observed Surface schema"
        );
        let elements: Vec<serde_json::Value> = observation
            .elements
            .iter()
            .map(|element| {
                let b = &element.bounds;
                serde_json::json!({
                    "name": element.name.trim(),
                    "aria": element.aria.trim(),
                    "box": [b.x.round() as i64, b.y.round() as i64, b.width.round() as i64, b.height.round() as i64],
                })
            })
            .collect();
        let state = observation.state.trim().to_string();
        anyhow::ensure!(!state.is_empty(), "an observation has no state label");
        let (width, height) = (observation.viewport.width, observation.viewport.height);
        let line = serde_json::to_string(&serde_json::json!({
            "state": state,
            "viewport": [width, height],
            "elements": elements,
        }))?;
        if let Some(earlier) = normalized.insert((state.clone(), width, height), line.clone()) {
            anyhow::ensure!(
                earlier == line,
                "state '{state}' at {width}x{height} was observed twice with different content"
            );
        }
    }
    anyhow::ensure!(
        !normalized.is_empty(),
        "the journey recorded no observation"
    );
    let lines: Vec<String> = normalized.into_values().collect();
    Ok(hashing::bytes_sha256(lines.join("\n").as_bytes()))
}

fn failure(summary: impl Into<String>) -> evidence::CommandOutcome {
    evidence::CommandOutcome {
        ok: false,
        command: String::new(),
        summary: summary.into(),
        details: None,
    }
}

fn resolved_runtime(
    project: &Project,
) -> Result<(crate::project::RuntimeVerification, String), evidence::CommandOutcome> {
    let runtime = project
        .runtime_verification()
        .map_err(|e| failure(e.to_string()))?;
    match project.compose_has_service(&runtime.database_service) {
        Ok(true) => {}
        Ok(false) => {
            return Err(failure(format!(
                "runtime_verification.database_service '{}' is not declared by docker compose",
                runtime.database_service
            )))
        }
        Err(e) => return Err(failure(e.to_string())),
    }
    let database_url = project
        .database_test_url(&runtime)
        .map_err(|e| failure(e.to_string()))?;
    Ok((runtime, database_url))
}

fn compose(project: &Project, args: &[&str]) -> evidence::CommandOutcome {
    evidence::run(&project.root, "docker", &[&["compose"], args].concat())
}

fn endpoint_ready(project: &Project, url: &str) -> bool {
    for _ in 0..40 {
        if evidence::run(&project.root, "curl", &["-fsS", "-o", "/dev/null", url]).ok {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    false
}

fn run_smoke(project: &Project) -> evidence::CommandOutcome {
    let runtime = match resolved_runtime(project) {
        Ok((runtime, _)) => runtime,
        Err(error) => return error,
    };
    let up = compose(project, &["up", "-d", "--build", "--wait"]);
    if !up.ok {
        return up;
    }
    let backend = endpoint_ready(project, &runtime.backend_health_url);
    let web = endpoint_ready(project, &runtime.web_url);
    evidence::CommandOutcome {
        ok: backend && web,
        command: "docker compose up -d --build --wait; configured endpoint probes".into(),
        summary: format!(
            "backend {} reachable: {backend}; web {} reachable: {web}",
            runtime.backend_health_url, runtime.web_url
        ),
        details: None,
    }
}

fn run_cargo_tests(project: &Project) -> evidence::CommandOutcome {
    cargo_tests(project, &["test", "--workspace", "--quiet"]).0
}

/// `cargo test` against the resolved database, with its stdout.
fn cargo_tests(project: &Project, args: &[&str]) -> (evidence::CommandOutcome, String) {
    let database_url = match resolved_runtime(project) {
        Ok((_, database_url)) => database_url,
        Err(error) => return (error, String::new()),
    };
    let Some(workspace) = project.cargo_workspace_root() else {
        return (
            failure("cargo metadata did not resolve a Rust workspace"),
            String::new(),
        );
    };
    evidence::run_env_output(
        &workspace,
        "cargo",
        args,
        &[("DATABASE_URL", database_url.as_str())],
    )
}

/// A temporary path for one runner's JSON report.
fn report_path(runner: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "codeos-{runner}-report-{}-{}.json",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ))
}

/// Read and remove a runner's JSON report.
fn take_report(path: &Path) -> Result<serde_json::Value> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("test runner wrote no JSON report at {}", path.display()));
    let _ = std::fs::remove_file(path);
    Ok(serde_json::from_str(&text?)?)
}

/// One run of the deterministic suite: the Rust workspace and, when present, the web unit suite,
/// with the per-test outcomes each reported. Any failure, or a suite reporting no cases, is `Err`.
fn behavior_suite(
    project: &Project,
    attempt: u8,
) -> std::result::Result<
    (evidence::CommandOutcome, Vec<test_evidence::TestRun>),
    evidence::CommandOutcome,
> {
    let (backend, stdout) = cargo_tests(project, &["test", "--workspace"]);
    if !backend.ok {
        return Err(backend);
    }
    let mut runs = vec![test_evidence::TestRun::new(
        "libtest",
        attempt,
        test_evidence::libtest_cases(&stdout),
    )
    .map_err(|error| failure(error.to_string()))?];
    let web = project.root.join("web");
    if !web.is_dir() || !project.web_has_script("test:unit") {
        return Ok((backend, runs));
    }
    let report = report_path("vitest");
    let output = format!("--outputFile.json={}", report.display());
    let frontend = evidence::run(
        &web,
        "npm",
        &[
            "run",
            "test:unit",
            "--",
            "--run",
            "--reporter=default",
            "--reporter=json",
            &output,
        ],
    );
    if !frontend.ok {
        let _ = std::fs::remove_file(&report);
        return Err(frontend);
    }
    let cases = take_report(&report)
        .map(|value| test_evidence::vitest_cases(&value, &web))
        .map_err(|error| failure(error.to_string()))?;
    runs.push(
        test_evidence::TestRun::new("vitest", attempt, cases)
            .map_err(|error| failure(error.to_string()))?,
    );
    Ok((
        evidence::CommandOutcome {
            ok: true,
            command: format!("{} && (cd web && {})", backend.command, frontend.command),
            summary: format!(
                "backend: {} | web unit/component: {}",
                backend.summary, frontend.summary
            ),
            details: None,
        },
        runs,
    ))
}

/// Store the per-test outcomes and bind their identity to the record this outcome writes.
fn with_test_evidence(
    project: &Project,
    verification: &str,
    runs: Vec<test_evidence::TestRun>,
    mut outcome: evidence::CommandOutcome,
) -> evidence::CommandOutcome {
    let evidence = test_evidence::TestEvidence {
        schema: 1,
        verification: verification.into(),
        runs,
    };
    match test_evidence::store(project, &evidence) {
        Ok(identity) => {
            outcome.details = Some(serde_json::json!({ test_evidence::DETAILS_KEY: identity }));
            outcome
        }
        Err(error) => failure(format!("store test evidence: {error:#}")),
    }
}

/// Playwright through the project's `test:e2e` script, with its per-test outcomes.
fn run_playwright(project: &Project, web: &Path, base_url: &str) -> evidence::CommandOutcome {
    let report = report_path("playwright");
    let report_text = report.to_string_lossy().into_owned();
    let outcome = evidence::run_env(
        web,
        "npm",
        &["run", "test:e2e", "--", "--reporter=list,json"],
        &[
            ("PLAYWRIGHT_BASE_URL", base_url),
            ("PLAYWRIGHT_JSON_OUTPUT_FILE", report_text.as_str()),
        ],
    );
    if !outcome.ok {
        let _ = std::fs::remove_file(&report);
        return outcome;
    }
    let run = take_report(&report).and_then(|value| {
        test_evidence::TestRun::new("playwright", 1, test_evidence::playwright_cases(&value))
    });
    match run {
        Ok(run) => with_test_evidence(project, "playwright", vec![run], outcome),
        Err(error) => failure(error.to_string()),
    }
}

/// Approved Contract behavior spans every tier the project ships tests for: the Rust workspace and,
/// when present, the web unit/component suite (Codeos Mechanics: `behavior`, `repeatability`).
/// `repeatability` runs the suite twice and fails on any difference in per-test outcomes.
fn run_behavior_tests(project: &Project, verification: &str) -> evidence::CommandOutcome {
    let (mut outcome, mut runs) = match behavior_suite(project, 1) {
        Ok(first) => first,
        Err(failed) => return failed,
    };
    if verification == "repeatability" {
        let second = match behavior_suite(project, 2) {
            Ok((_, second)) => second,
            Err(failed) => return failed,
        };
        let mut differences = Vec::new();
        for runner in ["libtest", "vitest"] {
            let cases = |runs: &[test_evidence::TestRun]| {
                runs.iter()
                    .find(|run| run.runner == runner)
                    .map(|run| run.cases.clone())
                    .unwrap_or_default()
            };
            differences.extend(
                test_evidence::differences(&cases(&runs), &cases(&second))
                    .into_iter()
                    .map(|difference| format!("{runner} {difference}")),
            );
        }
        if !differences.is_empty() {
            return failure(format!(
                "nondeterministic test outcomes across two runs: {}",
                differences.join("; ")
            ));
        }
        outcome.command = format!("{} (twice; per-test outcomes compared)", outcome.command);
        runs.extend(second);
    }
    with_test_evidence(project, verification, runs, outcome)
}

/// B3: bring the declared integrated stack up, prove DB<->backend<->GUI reachability and a clean
/// migration, run the project's shipped tests and Playwright journey, then tear the stack down.
fn run_integrated_baseline(project: &Project) -> evidence::CommandOutcome {
    let root = &project.root;
    if !root.join("docker-compose.yml").is_file() {
        return evidence::CommandOutcome {
            ok: false,
            command: String::new(),
            summary: "no docker-compose.yml".into(),
            details: None,
        };
    }
    let (runtime, database_url) = match resolved_runtime(project) {
        Ok(resolved) => resolved,
        Err(error) => return error,
    };
    let mut steps: Vec<String> = Vec::new();

    // Always start from a clean volume so the migration runs from an empty database.
    let _ = compose(project, &["down", "-v"]);
    let up = compose(project, &["up", "-d", "--build", "--wait"]);
    if !up.ok {
        let _ = compose(project, &["down", "-v"]);
        return evidence::CommandOutcome {
            ok: false,
            command: "docker compose up -d --build --wait".into(),
            summary: format!("docker compose up failed: {}", up.summary),
            details: None,
        };
    }

    let mut ok = true;
    // The project declares a DB-backed backend health endpoint; 2xx proves the migrated database
    // is reachable through the real backend boundary.
    let db_connected = endpoint_ready(project, &runtime.backend_health_url);
    ok &= db_connected;
    steps.push(format!(
        "health: {}",
        if db_connected {
            "db connected"
        } else {
            "NOT connected"
        }
    ));

    let web_reachable = endpoint_ready(project, &runtime.web_url);
    ok &= web_reachable;
    steps.push(format!("web reachable: {web_reachable}"));

    if let Some(workspace) = project.cargo_workspace_root() {
        let t = evidence::run_env(
            &workspace,
            "cargo",
            &["test", "--workspace", "--quiet"],
            &[("DATABASE_URL", database_url.as_str())],
        );
        ok &= t.ok;
        steps.push(format!("backend tests: {}", t.ok));
    }
    if root.join("web").is_dir() {
        if project.web_has_script("test:unit") {
            let u = evidence::run(
                &root.join("web"),
                "npm",
                &["run", "test:unit", "--", "--run"],
            );
            ok &= u.ok;
            steps.push(format!("web unit/component tests: {}", u.ok));
        }
        let e = evidence::run_env(
            &root.join("web"),
            "npm",
            &["run", "test:e2e"],
            &[("PLAYWRIGHT_BASE_URL", runtime.web_url.as_str())],
        );
        ok &= e.ok;
        steps.push(format!("Playwright journey: {}", e.ok));
    }

    let _ = compose(project, &["down", "-v"]);
    evidence::CommandOutcome {
        ok,
        command: "docker compose integrated baseline (migration + curl + cargo test + npm test)"
            .into(),
        summary: steps.join("; "),
        details: None,
    }
}

fn current_head(root: &Path) -> String {
    crate::instrument::command("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").chars().take(80).collect()
}

fn short(s: &str) -> String {
    if s.len() > 10 {
        format!("{}…", &s[..10])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod journey_tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn integrated_baseline_uses_declared_project_native_command_without_docker() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".codeos/00-project")).unwrap();
        std::fs::create_dir_all(dir.path().join("scripts")).unwrap();
        std::fs::write(
            dir.path().join(".codeos/00-project/codeos.yaml"),
            "runtime_verification:\n  verification_command: scripts/verify-bootstrap\n",
        )
        .unwrap();
        let command = dir.path().join("scripts/verify-bootstrap");
        std::fs::write(&command, "#!/usr/bin/env bash\necho native-baseline\n").unwrap();
        let mut permissions = command.metadata().unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&command, permissions).unwrap();
        let project = Project::discover(dir.path()).unwrap();
        let subject = Subject {
            workflow: Workflow::Bootstrap,
            id: "solution".into(),
            gui: false,
            backend: true,
            persistence: true,
            impact_accounting_complete: true,
            arch_approval_required: false,
            reality_claims: vec![],
            inherited_claims: vec![],
        };
        let result = run_mechanic(&project, &subject, "baseline");
        assert!(result.ok, "{}", result.summary);
        assert!(result.summary.contains("native-baseline"));
        assert!(!dir.path().join("docker-compose.yml").exists());
    }

    #[test]
    fn impact_accounting_reads_only_the_tier_and_state_cells() {
        let table = "| Persistence (PostgreSQL) | unchanged | backend stores nothing |\n\
                     | Backend (Rust) | changed | endpoint |\n\
                     | GUI (Svelte) | invalid | undecided |"
            .to_lowercase();
        assert_eq!(impact_row(&table, "persistence"), Some(false));
        assert_eq!(impact_row(&table, "backend"), Some(true));
        assert_eq!(impact_row(&table, "gui"), None);
    }

    #[test]
    fn required_decision_bindings_follow_decide_and_accept_pre_product_scope_receipts() {
        let map = |names: &[&str]| -> BTreeMap<String, String> {
            names
                .iter()
                .map(|n| (n.to_string(), "h".to_string()))
                .collect()
        };
        let receipt = |names: &[&str]| Receipt {
            workflow: "feature".into(),
            subject: "F-0001".into(),
            checkpoint: "final_ux_validation".into(),
            result: "accepted".into(),
            bindings: map(names),
            timestamp: "2026-09-30T00:00:00Z".into(),
            rationale: None,
            observation: None,
        };
        let names = |set: BTreeSet<String>| set.into_iter().collect::<Vec<_>>();
        // DBA-25 F8: decide records product_state instead of implementation_state.
        let f8 = map(&[
            "intent",
            "codeos_yaml",
            "implementation_state",
            "product_state",
            "candidate_bundle",
        ]);
        assert_eq!(
            names(required_decision_bindings(&f8, Some(&receipt(&["intent"])))),
            ["candidate_bundle", "codeos_yaml", "intent", "product_state"]
        );
        // A receipt bound to the whole implementation covers the product files.
        assert_eq!(
            names(required_decision_bindings(
                &f8,
                Some(&receipt(&["intent", "implementation_state"]))
            )),
            ["candidate_bundle", "codeos_yaml", "intent"]
        );
        // F9 has no product scope: implementation_state stays required.
        let f9 = map(&[
            "intent",
            "codeos_yaml",
            "implementation_state",
            "candidate_bundle",
        ]);
        assert_eq!(
            names(required_decision_bindings(&f9, None)),
            [
                "candidate_bundle",
                "codeos_yaml",
                "implementation_state",
                "intent"
            ]
        );
        // Feature Workflow v10 F8: the Observed Surface replaces both broader product bindings;
        // the Candidate Bundle stays required.
        let v10 = map(&[
            "intent",
            "codeos_yaml",
            "implementation_state",
            "product_state",
            "observed_surface",
            "candidate_bundle",
        ]);
        assert_eq!(
            names(required_decision_bindings(&v10, None)),
            [
                "candidate_bundle",
                "codeos_yaml",
                "intent",
                "observed_surface"
            ]
        );
        // Receipts recorded before v10 keep the broader binding they recorded.
        for broader in ["product_state", "implementation_state"] {
            assert_eq!(
                names(required_decision_bindings(
                    &v10,
                    Some(&receipt(&["intent", broader]))
                )),
                ["candidate_bundle", "codeos_yaml", "intent"]
            );
        }
    }

    #[test]
    fn observed_surface_identity_normalizes_order_and_subpixel_geometry() {
        let line = |state: &str, aria: &str, x: f64| {
            format!(
                r#"{{"schema":1,"state":"{state}","viewport":{{"width":390,"height":844}},"elements":[{{"name":"best","aria":"{aria}","box":{{"x":{x},"y":10,"width":100,"height":20}}}}]}}"#
            )
        };
        let base = observed_surface_identity(&format!(
            "{}\n{}\n",
            line("result", "heading 08:00", 4.0),
            line("form", "button Compare", 0.0)
        ))
        .unwrap();
        // Parallel workers may append in any order; sub-pixel geometry and padding do not count.
        let reordered = observed_surface_identity(&format!(
            "{}\n\n{}",
            line("form", " button Compare ", 0.2),
            line("result", "heading 08:00", 3.6)
        ))
        .unwrap();
        assert_eq!(base, reordered);
        // Visible text or a whole-pixel layout change is a different surface.
        for changed in [
            line("result", "heading 08:15", 4.0),
            line("result", "heading 08:00", 6.0),
        ] {
            let other = observed_surface_identity(&format!(
                "{changed}\n{}",
                line("form", "button Compare", 0.0)
            ))
            .unwrap();
            assert_ne!(base, other);
        }
        assert!(observed_surface_identity("").is_err());
        assert!(observed_surface_identity("not json").is_err());
        // The same state twice: identical is one observation, contradictory fails closed.
        let form = line("form", "button Compare", 0.0);
        let once = observed_surface_identity(&form).unwrap();
        assert_eq!(
            observed_surface_identity(&format!("{form}\n{form}")).unwrap(),
            once
        );
        assert!(observed_surface_identity(&format!(
            "{form}\n{}",
            line("form", "button Search", 0.0)
        ))
        .is_err());
        assert!(observed_surface_identity(
            r#"{"schema":2,"state":"s","viewport":{"width":1,"height":1},"elements":[]}"#
        )
        .is_err());
        assert!(observed_surface_identity(
            r#"{"schema":1,"state":"s","viewport":{"width":1,"height":1},"elements":[],"extra":1}"#
        )
        .is_err());
    }
    #[test]
    fn observations_are_read_only_from_the_named_regular_file() {
        let dir = tempfile::tempdir().unwrap();
        let named = dir.path().join("observations.jsonl");
        assert!(read_observations(&named).is_err());
        let elsewhere = dir.path().join("elsewhere.jsonl");
        fs::write(&elsewhere, "{}").unwrap();
        std::os::unix::fs::symlink(&elsewhere, &named).unwrap();
        assert!(read_observations(&named).is_err());
        fs::remove_file(&named).unwrap();
        fs::write(&named, "line").unwrap();
        assert_eq!(read_observations(&named).unwrap(), "line");
    }

    use std::fs;

    fn fixture(gui: bool) -> (tempfile::TempDir, Project, Subject) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        fs::create_dir_all(root.join(".codeos/00-project")).unwrap();
        fs::create_dir_all(root.join(".codeos/01-specification/contracts")).unwrap();
        fs::write(
            root.join(".codeos/00-project/charter.md"),
            "## Primary Product Journey\njourney_id: primary\nlegitimate_entry: product home\ninitial_frontier: entry\ntarget_outcome: useful result\nverification_route: product_journey\n",
        )
        .unwrap();
        fs::write(
            root.join(".codeos/00-project/codeos.yaml"),
            "runtime_verification:\n  database_service: db\n  database_test_url_env: TEST_DB_URL\n  backend_health_url: http://backend/health\n  web_url: http://product/\n",
        )
        .unwrap();
        fs::write(
            root.join(".codeos/01-specification/contracts/F-1_contract.md"),
            "## Product Journey Contribution\nmode: advance\nprevious_frontier: entry\nproposed_frontier: usable\ntransition: entry reaches useful output\n",
        )
        .unwrap();
        let project = Project::new(root, false);
        let subject = Subject {
            workflow: Workflow::Feature,
            id: "F-1".into(),
            gui,
            backend: false,
            persistence: false,
            impact_accounting_complete: true,
            arch_approval_required: false,
            reality_claims: vec![],
            inherited_claims: vec![],
        };
        (dir, project, subject)
    }

    #[test]
    fn gui_journey_selects_the_proposed_frontier_tag() {
        let (_dir, project, subject) = fixture(true);
        fs::create_dir_all(project.root.join("web")).unwrap();
        fs::write(
            project.root.join("web/package.json"),
            r#"{"scripts":{"test:journey":"node journey.js"}}"#,
        )
        .unwrap();
        fs::write(
            project.root.join("web/journey.js"),
            "if (!process.argv.includes('@frontier:usable')) process.exit(7);\n",
        )
        .unwrap();
        let outcome = run_product_journey(&project, &subject);
        assert!(outcome.ok, "{}", outcome.summary);
    }

    #[test]
    fn gui_journey_fails_closed_without_the_dedicated_script() {
        let (_dir, project, subject) = fixture(true);
        fs::create_dir_all(project.root.join("web")).unwrap();
        fs::write(project.root.join("web/package.json"), r#"{"scripts":{}}"#).unwrap();
        let outcome = run_product_journey(&project, &subject);
        assert!(!outcome.ok);
        assert!(outcome.summary.contains("test:journey"));
    }

    #[test]
    fn gui_exempt_journey_receives_the_proposed_frontier() {
        let (_dir, project, subject) = fixture(false);
        fs::create_dir_all(project.root.join("scripts")).unwrap();
        fs::write(
            project.root.join("scripts/test-product-journey.sh"),
            "#!/usr/bin/env bash\n[[ \"$1\" == usable ]]\n",
        )
        .unwrap();
        let outcome = run_product_journey(&project, &subject);
        assert!(outcome.ok, "{}", outcome.summary);
    }

    fn transition(id: &str, from: &str, to: &str) -> ExpectedTransition {
        ExpectedTransition {
            transition_id: id.into(),
            from_frontier: from.into(),
            to_frontier: to.into(),
            producer_role: format!("producer-{from}"),
            handoff_type: format!("{to}-handoff"),
            consumer_role: format!("consumer-{to}"),
            authority_kind: "architecture".into(),
            authority_id: "core".into(),
            owner_feature: "F-2".into(),
            authority_path: None,
        }
    }

    #[test]
    fn failed_extension_preserves_a_reproven_accepted_prefix() {
        let first = transition("T-0001", "entry", "accepted");
        let second = transition("T-0002", "accepted", "candidate");
        let current = BTreeMap::from([
            (
                first.transition_id.clone(),
                (
                    JourneyObservation {
                        transition_id: first.transition_id.clone(),
                        handoff_identity: Some("accepted-1".into()),
                        state_origin: "production".into(),
                        result: "passed".into(),
                        reason: None,
                    },
                    first.clone(),
                    "now".into(),
                ),
            ),
            (
                second.transition_id.clone(),
                (
                    JourneyObservation {
                        transition_id: second.transition_id.clone(),
                        handoff_identity: None,
                        state_origin: "production".into(),
                        result: "failed".into(),
                        reason: Some("no production consumer".into()),
                    },
                    second.clone(),
                    "now".into(),
                ),
            ),
        ]);
        let accepted = JourneyDefinition {
            schema: 1,
            journey_id: "primary".into(),
            target_frontier: "accepted".into(),
            transitions: vec![first],
        };
        let candidate = JourneyDefinition {
            schema: 1,
            journey_id: "primary".into(),
            target_frontier: "candidate".into(),
            transitions: vec![accepted.transitions[0].clone(), second],
        };
        let mut out = String::new();
        render_definition_status(
            &mut out,
            "Accepted",
            &accepted,
            &current,
            &BTreeSet::new(),
            false,
        );
        render_definition_status(
            &mut out,
            "Candidate",
            &candidate,
            &current,
            &BTreeSet::new(),
            false,
        );
        assert!(out.contains("Accepted: PASS"));
        assert!(out.contains("Candidate: BLOCKED"));
        assert!(out.contains("T-0002 — accepted -> candidate"));
    }

    #[test]
    fn structured_evidence_rejects_self_reported_topology_and_domain_seeding() {
        let raw = r#"{
          "schema":1,"journey_id":"primary","target_frontier":"done",
          "producer":"invented",
          "setup":[],"observations":[]
        }"#;
        assert!(serde_json::from_str::<JourneyEvidence>(raw).is_err());

        let definition = JourneyDefinition {
            schema: 1,
            journey_id: "primary".into(),
            target_frontier: "done".into(),
            transitions: vec![transition("T-0001", "entry", "done")],
        };
        let evidence = JourneyEvidence {
            schema: 1,
            journey_id: "primary".into(),
            target_frontier: "done".into(),
            setup: vec![JourneySetup {
                category: "retrieved_material".into(),
                resource: "fixture row".into(),
            }],
            observations: vec![JourneyObservation {
                transition_id: "T-0001".into(),
                handoff_identity: Some("run-1".into()),
                state_origin: "production".into(),
                result: "passed".into(),
                reason: None,
            }],
        };
        assert!(validate_journey_evidence(&definition, &evidence, true).is_err());
    }

    #[test]
    fn architecture_owns_a_cross_feature_transition_definition() {
        let (_dir, project, _subject) = fixture(false);
        let scopes = project.root.join(".codeos/02-architecture/scopes");
        fs::create_dir_all(&scopes).unwrap();
        fs::write(
            scopes.join("core.md"),
            "---\nartifact_type: architecture\nfeatures: [F-1, F-2]\napproval:\n  by: Human\n  at: 2026-09-15\n---\n\n## Primary Journey Transitions\n\n```yaml\ntransitions:\n  - transition_id: T-0001\n    from_frontier: one\n    to_frontier: two\n    producer_feature: F-1\n    producer_role: producer\n    handoff_type: Handoff\n    consumer_feature: F-2\n    consumer_role: consumer\n```\n",
        )
        .unwrap();
        let derived = architecture_transition(&project, "F-1", "F-2", "one", "two").unwrap();
        assert_eq!(derived.transition_id, "T-0001");
        assert_eq!(derived.producer_role, "producer");
        assert_eq!(derived.authority_kind, "architecture");
    }

    #[test]
    fn reality_classification_is_claim_scoped_and_closed() {
        let contract = "## Reality Basis\n\n| Claim ID | Claim | Classification | Source | Conditions | Currency | Unavailable | Fixture |\n|---|---|---|---|---|---|---|---|\n| route | Road route | reality-dependent | provider | roads | now | unavailable | test-only |\n| label | Heading | not-reality-dependent | n/a | n/a | n/a | n/a | n/a |\n| demo | Example | simulated | n/a | n/a | n/a | n/a | fixture |\n\n## Scenarios\n";
        assert_eq!(reality_basis(contract, false).unwrap().0, vec!["route"]);
    }

    #[test]
    fn production_evidence_schema_rejects_fixture_substitution_and_extra_fields() {
        let fixture = r#"{"schema":1,"subject":"F-1","claims":["route"],"source_kind":"fixture","provider":"test","observed_at":"now","implementation_sha256":"abc","governed_inputs":{},"observations":[{"claim_ids":["route"],"visible_result":{},"visible_result_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","provider_receipts_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","provider_request_response_receipts":[],"details":{}}]}"#;
        let parsed: ProductionEvidence = serde_json::from_str(fixture).unwrap();
        assert_ne!(parsed.source_kind, "production");
        let extra = fixture.replace("\"schema\":1", "\"schema\":1,\"made_up\":true");
        assert!(serde_json::from_str::<ProductionEvidence>(&extra).is_err());
    }

    #[test]
    fn human_validation_and_acceptance_are_never_executed_by_codeos() {
        for version in 1..=9 {
            for checkpoint in contract_for(Workflow::Feature, version, 2) {
                for requirement in &checkpoint.requirements {
                    if let Requirement::Receipt { checkpoint, .. } = requirement {
                        if matches!(*checkpoint, "final_ux_validation" | "acceptance") {
                            assert_eq!(Actor::default_for(requirement), Actor::Human);
                        }
                    }
                }
            }
        }
    }
}
