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

use crate::contract::{contract_for, Applies, Checkpoint, Requirement, Workflow};
use crate::evidence;
use crate::hashing;
use crate::project::Project;
use crate::receipts::{self, ReceiptState, ReceiptStore};
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

#[derive(Debug, Clone)]
pub struct CheckpointReport {
    pub id: String,
    pub title: String,
    pub state: State,
    pub met: Vec<String>,
    pub unmet: Vec<String>,
    pub next_action: Option<String>,
}

/// A feature subject with its applicability resolved from the Contract.
pub struct Subject {
    pub workflow: Workflow,
    pub id: String,
    pub gui: bool,
    pub persistence: bool,
    pub arch_approval_required: bool,
    pub reality_claims: Vec<String>,
}

impl Subject {
    pub fn resolve(project: &Project, workflow: Workflow, id: &str) -> Result<Self> {
        let (mut gui, mut persistence) = (false, false);
        let mut reality_claims = Vec::new();
        if workflow == Workflow::Feature {
            let contract_path = project.contract(id);
            if let Ok(text) = std::fs::read_to_string(&contract_path) {
                // Read the Feature Impact Accounting table rows only, plus an explicit marker.
                let lower = text.to_lowercase();
                gui = lower.contains("gui-visible outcome") || impact_row_changed(&lower, "gui");
                persistence = impact_row_changed(&lower, "persistence");
                reality_claims = reality_dependent_claims(&text)?;
            }
        }
        Ok(Self {
            workflow,
            id: id.to_string(),
            gui,
            persistence,
            arch_approval_required: false,
            reality_claims,
        })
    }
}

/// Read-only: derive every checkpoint's state for a subject from canonical evidence, verification
/// records, and decision receipts. Never executes a command.
pub fn evaluate(project: &Project, subject: &Subject) -> Result<Vec<CheckpointReport>> {
    let store = ReceiptStore::at(&project.codeos_dir());
    let vstore = VerificationStore::at(&project.codeos_dir());
    let contract = contract_for(subject.workflow, project.feature_workflow_version()?);
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
                persistence: false,
                arch_approval_required: false,
                reality_claims: vec![],
            };
            let boot_reports = evaluate(project, &boot)?;
            let boot_complete = boot_reports
                .iter()
                .filter(|r| matches!(r.id.as_str(), "B1" | "B2" | "B3" | "B4"))
                .all(|r| r.state == State::Pass);
            if !boot_complete {
                let blocker = boot_reports
                    .iter()
                    .find(|r| matches!(r.state, State::Blocked | State::Waiting))
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
                });
            }
        }
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
            });
            continue;
        }

        if !all_prior_pass {
            let blocker = reports
                .iter()
                .find(|r| r.state == State::Blocked || r.state == State::Waiting)
                .map(|r| r.id.clone())
                .unwrap_or_default();
            reports.push(CheckpointReport {
                id: c.id.into(),
                title: c.title.into(),
                state: State::Blocked,
                met: vec![],
                unmet: vec![format!("earlier checkpoint {blocker} is not PASS")],
                next_action: Some(format!("resolve {blocker} first")),
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

/// A Feature Impact Accounting row `| <tier…> | changed | <reason> |` for the given tier keyword.
fn impact_row_changed(lower_text: &str, tier_keyword: &str) -> bool {
    lower_text.lines().any(|line| {
        let l = line.trim();
        l.starts_with('|')
            && l.contains(tier_keyword)
            && l.split('|').map(str::trim).any(|cell| cell == "changed")
    })
}

fn reality_dependent_claims(contract: &str) -> Result<Vec<String>> {
    let Some(section) = contract.split("## Reality Basis").nth(1) else {
        return Ok(vec![]);
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
    for cells in rows {
        if !matches!(
            cells[2],
            "reality-dependent" | "simulated" | "not-reality-dependent"
        ) {
            anyhow::bail!("claim '{}' has an invalid reality classification", cells[0]);
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
    Ok(reality)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum JourneyContribution {
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

fn journey_contribution(path: &Path) -> Result<JourneyContribution> {
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
    let advances = accepted_advances_excluding(project, excluded_subject)?;
    let mut frontier = initial;
    let mut prior_owner: Option<String> = None;
    let mut transitions = Vec::new();
    for (owner, previous, proposed) in advances {
        if previous != frontier {
            anyhow::bail!("accepted journey chain expected previous frontier '{frontier}', found '{previous}'");
        }
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
/// for each journey-advancing feature, in append order. No parallel frontier state is stored.
pub fn current_accepted_frontier(project: &Project) -> Result<String> {
    accepted_frontier_excluding(project, None)
}

fn accepted_frontier_excluding(
    project: &Project,
    excluded_subject: Option<&str>,
) -> Result<String> {
    let mut frontier = charter_initial_frontier(project)?;
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
    for (index, receipt) in receipts.into_iter().enumerate() {
        if !latest_indices.contains(&index) {
            continue;
        }
        if excluded_subject == Some(receipt.subject.as_str()) {
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
            if previous != frontier {
                anyhow::bail!(
                    "accepted product-journey frontier conflict: {} expected previous '{}' but accepted frontier is '{}'",
                    receipt.subject,
                    previous,
                    frontier
                );
            }
            frontier = proposed;
        }
    }
    Ok(frontier)
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
            Some(("MISSING", expected, None, None))
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

    for req in &c.requirements {
        match req {
            Requirement::CharterApproved => {
                if evidence::approval_recorded(&project.charter())? {
                    met.push("Charter records approval".into());
                } else if !project.charter().exists() {
                    unmet.push(".codeos/00-project/charter.md does not exist".into());
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
                let mut bindings = current_bindings(project, subject, checkpoint)?;
                let latest = store.latest(subject.workflow.slug(), &subject.id, checkpoint)?;
                // The observation binding cannot be recomputed from the tree — the observation
                // statement is a historical, immutable fact. Carry the recorded hash forward so it
                // reads as current; it exists to link a closure to the same observation, not to
                // detect drift.
                if let Some(h) = latest.as_ref().and_then(|r| r.bindings.get("observation")) {
                    bindings.insert("observation".into(), h.clone());
                }
                if let Some(h) = selected_production_evidence(project, subject)? {
                    if matches!(
                        *checkpoint,
                        "reconciliation" | "final_ux_validation" | "acceptance"
                    ) {
                        bindings.insert("production_evidence".into(), h);
                    }
                }
                match receipts::evaluate(latest, &bindings) {
                    ReceiptState::Absent => {
                        soft += 1;
                        unmet.push(format!("no {checkpoint} decision recorded"));
                        next_action.get_or_insert(format!(
                            "record it: codeos-workflow decide --workflow {} --subject {} --checkpoint {checkpoint} --result <{}>",
                            subject.workflow.slug(), subject.id, results.join("|")
                        ));
                    }
                    ReceiptState::Stale {
                        binding, was, now, ..
                    } => {
                        soft += 1;
                        unmet.push(format!(
                            "{checkpoint} decision is stale — '{binding}' changed since it was recorded (was {}, now {})",
                            short(&was), short(&now)
                        ));
                        next_action.get_or_insert(format!(
                            "re-run the {checkpoint} decision against current evidence"
                        ));
                    }
                    ReceiptState::Current(r) => {
                        if results.contains(&r.result.as_str()) {
                            met.push(format!("{checkpoint}: {}", r.result));
                        } else {
                            unmet.push(format!(
                                "{checkpoint} recorded '{}', which does not permit progression here",
                                r.result
                            ));
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
                        if let Some(selected) = selected_production_evidence(project, subject)? {
                            if r.production_evidence.as_deref() != Some(selected.as_str()) {
                                unmet.push("reviewer packet is not bound to the production evidence selected at F6".into());
                                next_action.get_or_insert(
                                    "review the exact selected production-evidence artifact".into(),
                                );
                                continue;
                            }
                        }
                        let head = current_head(&project.root);
                        let bound = r.review_commit.is_empty()
                            || head.is_empty()
                            || r.review_commit == head;
                        if !bound {
                            unmet.push(format!(
                                "reviewer record is bound to {} but HEAD is {} — re-review current evidence",
                                short(&r.review_commit), short(&head)
                            ));
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
                            next_action.get_or_insert("restore the exact evidence-repair chain or supply materially new evidence".into());
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
                            next_action.get_or_insert(
                                "restore the exact authorized terminal repair closure chain".into(),
                            );
                        } else if !evidence::review_permits_progression(&r.effective_concern) {
                            unmet.push(format!(
                                "reviewer concern '{}' does not permit progression",
                                r.effective_concern
                            ));
                            next_action.get_or_insert("resolve the reviewer's finding".into());
                        } else {
                            met.push(format!("reviewer record present ({})", r.effective_concern));
                        }
                    }
                }
            }
            Requirement::ProductionSourceEvidence => {
                if subject.reality_claims.is_empty() {
                    met.push("no reality-dependent Contract claim".into());
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
                        RecordState::Stale { binding, .. } => {
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
                        binding, was, now, ..
                    } => {
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
                        next_action.get_or_insert(format!(
                            "repair the failure and re-run `codeos-workflow check` for {name}"
                        ));
                    }
                }
            }
            Requirement::ObservationStated => {
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
            Requirement::RouteResolution => {
                let route_receipt = store.latest("operation", &subject.id, "operation_route")?;
                let Some(route_receipt) = route_receipt else {
                    soft += 1;
                    unmet.push("no route recorded yet".into());
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
                        next_action.get_or_insert(
                            "link and verify the resolving change through its owning workflow, then re-check".into(),
                        );
                    }
                    other => {
                        unmet.push(format!("recorded route '{other}' is not valid"));
                        next_action.get_or_insert("re-record O2 with a valid route".into());
                    }
                }
            }
        }
    }

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
    })
}

/// `check`: execute every mechanical verification this subject needs whose record is absent or
/// stale, in checkpoint order. On each pass, append a verification record bound to the current
/// governed inputs. Stops at the first failing verification, or at the first checkpoint `check`
/// cannot advance (one blocked on an approval, a receipt, or review — not on a mechanical
/// verification). Returns one human-readable line per verification attempted; writes nothing else.
pub fn run_checks(project: &Project, subject: &Subject) -> Result<Vec<String>> {
    let contract = contract_for(subject.workflow, project.feature_workflow_version()?);
    let vstore = VerificationStore::at(&project.codeos_dir());
    let slug = subject.workflow.slug();
    let mut log = Vec::new();

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
        let mechanics: Vec<&'static str> = c
            .requirements
            .iter()
            .filter_map(|r| match r {
                Requirement::LiveVerification { mechanic, .. } => Some(*mechanic),
                Requirement::ProductionSourceEvidence if !subject.reality_claims.is_empty() => {
                    Some("production_source_evidence")
                }
                _ => None,
            })
            .collect();
        if mechanics.is_empty() {
            break; // this checkpoint waits on a human decision or a non-mechanical condition
        }

        let mut progressed = false;
        for mechanic in mechanics {
            let want = verification_bindings(project, subject, &stuck.id, Some(mechanic))?;
            let have = vstore.latest(slug, &subject.id, &stuck.id, mechanic)?;
            if matches!(verification::evaluate(have, &want), RecordState::Current(_)) {
                continue; // already verified against the current inputs
            }
            let outcome = run_mechanic(project, subject, mechanic);
            log.push(format!("{} / {mechanic}: {}", stuck.id, outcome.summary));
            if !outcome.ok {
                if mechanic == "product_journey" && project.workflow_governance_version()? >= 2 {
                    vstore.append(&VerificationRecord {
                        workflow: slug.to_string(),
                        subject: subject.id.clone(),
                        checkpoint: stuck.id.clone(),
                        verification: mechanic.to_string(),
                        result: "failed".to_string(),
                        bindings: want.clone(),
                        command: outcome.command.clone(),
                        timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        details: outcome.details.clone(),
                    })?;
                }
                return Ok(log); // fail closed at the first failing verification
            }
            let command = if outcome.command.is_empty() {
                mechanic.to_string()
            } else {
                outcome.command.clone()
            };
            vstore.append(&VerificationRecord {
                workflow: slug.to_string(),
                subject: subject.id.clone(),
                checkpoint: stuck.id.clone(),
                verification: mechanic.to_string(),
                result: "passed".to_string(),
                bindings: want.clone(),
                command,
                timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                details: outcome.details.clone(),
            })?;
            progressed = true;
        }
        if !progressed {
            break; // nothing left check can do for this checkpoint
        }
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
    current_bindings(project, subject, checkpoint)
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
                hashing::working_tree_state(&project.root, &project.implementation_paths())?,
            );
        }
        "reconciliation" | "final_ux_validation" | "acceptance" => {
            put(&mut b, "intent", &project.intent(&subject.id))?;
            put(&mut b, "contract", &project.contract(&subject.id))?;
            put(&mut b, "event_schema", &project.event_schema(&subject.id))?;
        }
        "operation_route" | "no_action_closure" => {
            // Bound to the observation statement's hash, supplied at decide time; nothing to
            // recompute from the tree here.
        }
        _ => {}
    }
    Ok(b)
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

pub fn verification_bindings_for_mechanic(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    mechanic: &str,
) -> Result<BTreeMap<String, String>> {
    verification_bindings(project, subject, checkpoint, Some(mechanic))
}

fn verification_bindings(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    mechanic: Option<&str>,
) -> Result<BTreeMap<String, String>> {
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
        "F3" | "F5" | "F5d" | "F5g" => {
            put(&mut b, "intent", &project.intent(&subject.id))?;
            put(&mut b, "contract", &project.contract(&subject.id))?;
            put(&mut b, "event_schema", &project.event_schema(&subject.id))?;
            b.insert(
                "implementation_state".into(),
                hashing::working_tree_state(&project.root, &project.implementation_paths())?,
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
        b.insert(
            "journey_target".into(),
            hashing::text_sha256(&product_journey_target(project, subject)?),
        );
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
    Ok(b)
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
        if let Ok(o) = std::process::Command::new("bash")
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
    // outcomes without executing anything. A real project never has this file; it is per-project,
    // so parallel tests do not interfere.
    let stub = project
        .codeos_dir()
        .join("06-workflow")
        .join(".verify-stub");
    if let Ok(spec) = std::fs::read_to_string(&stub) {
        for entry in spec.split([',', '\n']) {
            if let Some((m, v)) = entry.split_once('=') {
                if m.trim() == mechanic {
                    return evidence::CommandOutcome {
                        ok: v.trim() == "pass",
                        command: format!("<stub {mechanic}>"),
                        summary: format!("{mechanic} (stubbed {})", v.trim()),
                        details: None,
                    };
                }
            }
        }
    }
    let web = project.root.join("web");
    match mechanic {
        "baseline" => run_integrated_baseline(project),
        "smoke" => run_smoke(project),
        "product_journey" => run_product_journey(project, subject),
        "production_source_evidence" => run_production_source_evidence(project, subject),
        "behavior" | "repeatability" | "data_integrity" => run_cargo_tests(project),
        "playwright" => {
            let runtime = match project.runtime_verification() {
                Ok(runtime) => runtime,
                Err(error) => return failure(error.to_string()),
            };
            if !web.is_dir() {
                return failure("no web/ to verify");
            }
            evidence::run_env(
                &web,
                "npm",
                &["run", "test:e2e"],
                &[("PLAYWRIGHT_BASE_URL", runtime.web_url.as_str())],
            )
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
    implementation_sha256: String,
    governed_inputs: BTreeMap<String, String>,
    observations: Vec<ProductionObservation>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductionObservation {
    claim_ids: Vec<String>,
    visible_result: serde_json::Value,
    visible_result_sha256: String,
    provider_receipts_sha256: String,
    provider_request_response_receipts: Vec<ProviderReceipt>,
    details: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
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
    let nonce = format!(
        "{}-{}-{}",
        std::process::id(),
        subject.id,
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let definition_path = std::env::temp_dir().join(format!("codeos-reality-{nonce}.json"));
    let output_path = std::env::temp_dir().join(format!("codeos-reality-evidence-{nonce}.json"));
    let definition = serde_json::json!({
        "schema": 1,
        "subject": subject.id,
        "claims": subject.reality_claims,
        "governed_inputs": bindings,
    });
    if let Err(error) = std::fs::write(
        &definition_path,
        serde_json::to_vec_pretty(&definition).unwrap_or_default(),
    ) {
        return failure(format!("write reality definition: {error}"));
    }
    let definition_text = definition_path.to_string_lossy().into_owned();
    let output_text = output_path.to_string_lossy().into_owned();
    let mut outcome = evidence::run_env(
        &project.root,
        &config.command,
        &[],
        &[
            ("CODEOS_REALITY_DEFINITION_PATH", definition_text.as_str()),
            ("CODEOS_REALITY_EVIDENCE_PATH", output_text.as_str()),
            ("CODEOS_REALITY_MODE", "production-source"),
        ],
    );
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
    match parsed.and_then(|record| {
        if !outcome.ok {
            anyhow::bail!("production verifier exited unsuccessfully");
        }
        if record.schema != 1 || record.subject != subject.id {
            anyhow::bail!("evidence schema or subject does not match");
        }
        let expected: BTreeSet<_> = subject.reality_claims.iter().cloned().collect();
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
        for observation in &record.observations {
            if !is_sha256(&observation.visible_result_sha256)
                || !is_sha256(&observation.provider_receipts_sha256)
                || observation.provider_request_response_receipts.is_empty()
            {
                anyhow::bail!(
                    "production observation lacks content-bound visible/provider evidence"
                );
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
        if record.implementation_sha256 != expected_implementation
            || record.governed_inputs != bindings
        {
            anyhow::bail!(
                "evidence bindings do not match the current implementation and governed inputs"
            );
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
            outcome.details = Some(serde_json::json!({
                "evidence_sha256": identity,
                "evidence": record,
            }));
            outcome
        }
        Err(error) => {
            outcome.ok = false;
            outcome.summary = format!("{} -> FAILED ({error})", outcome.command);
            outcome.details = Some(serde_json::json!({ "validation_error": error.to_string() }));
            outcome
        }
    }
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
    let database_url = match resolved_runtime(project) {
        Ok((_, database_url)) => database_url,
        Err(error) => return error,
    };
    let Some(workspace) = project.cargo_workspace_root() else {
        return failure("cargo metadata did not resolve a Rust workspace");
    };
    evidence::run_env(
        &workspace,
        "cargo",
        &["test", "--workspace", "--quiet"],
        &[("DATABASE_URL", database_url.as_str())],
    )
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
    std::process::Command::new("git")
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
        let project = Project {
            root,
            self_dev: false,
        };
        let subject = Subject {
            workflow: Workflow::Feature,
            id: "F-1".into(),
            gui,
            persistence: false,
            arch_approval_required: false,
            reality_claims: vec![],
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
        render_definition_status(&mut out, "Accepted", &accepted, &current, &BTreeSet::new());
        render_definition_status(
            &mut out,
            "Candidate",
            &candidate,
            &current,
            &BTreeSet::new(),
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
        assert_eq!(reality_dependent_claims(contract).unwrap(), vec!["route"]);
    }

    #[test]
    fn production_evidence_schema_rejects_fixture_substitution_and_extra_fields() {
        let fixture = r#"{"schema":1,"subject":"F-1","claims":["route"],"source_kind":"fixture","provider":"test","observed_at":"now","implementation_sha256":"abc","governed_inputs":{},"observations":[{"claim_ids":["route"],"visible_result":{},"visible_result_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","provider_receipts_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","provider_request_response_receipts":[],"details":{}}]}"#;
        let parsed: ProductionEvidence = serde_json::from_str(fixture).unwrap();
        assert_ne!(parsed.source_kind, "production");
        let extra = fixture.replace("\"schema\":1", "\"schema\":1,\"made_up\":true");
        assert!(serde_json::from_str::<ProductionEvidence>(&extra).is_err());
    }
}
