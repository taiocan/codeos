//! Feature Workflow v13 Human Validation Sessions (DQA WO-06): one sealed human observation that
//! records only the outcomes the human stated, one per feature and checkpoint. Codeos enforces
//! explicitness; it never infers or propagates a verdict.

use crate::checker::{self, State, Subject};
use crate::claims::Evidence;
use crate::contract::Workflow;
use crate::hashing;
use crate::project::Project;
use crate::receipts::{Receipt, ReceiptStore};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The human checkpoints a session can answer, as (checkpoint, receipt checkpoint).
const QUESTIONS: [(&str, &str); 2] = [("F4", "early_preview"), ("F8", "final_ux_validation")];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub feature: String,
    /// The receipt checkpoint: `early_preview` or `final_ux_validation`.
    pub checkpoint: String,
    pub result: String,
    /// `observation:human` claim -> `met` or `not-met`, each stated explicitly.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub claims: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub schema: u8,
    pub implementation_state: String,
    pub product_state: String,
    pub environment: String,
    /// Feature -> the Observed Surface the human saw for it.
    pub surfaces: BTreeMap<String, String>,
    pub outcomes: Vec<Outcome>,
    pub rationale: String,
    pub recorded_at: String,
}

/// A governed human question currently waiting.
#[derive(Debug, Clone, Serialize)]
pub struct Question {
    pub feature: String,
    pub checkpoint: String,
    pub observed_surface: Option<String>,
    /// The `observation:human` claims the outcome must state.
    pub claims: Vec<String>,
}

fn dir(project: &Project) -> std::path::PathBuf {
    project.codeos_dir().join("06-workflow/sessions")
}

/// The question a feature waits on now, if it is F4 or F8.
pub fn question(project: &Project, subject: &Subject) -> Result<Option<Question>> {
    let reports = checker::evaluate(project, subject)?;
    let Some(waiting) = reports
        .iter()
        .find(|r| matches!(r.state, State::Blocked | State::Waiting))
    else {
        return Ok(None);
    };
    let Some((id, checkpoint)) = QUESTIONS.iter().find(|(id, _)| *id == waiting.id) else {
        return Ok(None);
    };
    if waiting
        .unmet
        .iter()
        .any(|u| u.contains("earlier checkpoint"))
    {
        return Ok(None);
    }
    let observed_surface = checker::bindings_for_decision(project, subject, checkpoint)
        .ok()
        .and_then(|bindings| bindings.get("observed_surface").cloned());
    let claims = checker::subject_claims(project, subject)?
        .unwrap_or_default()
        .into_iter()
        .filter(|claim| claim.checkpoint == *id && claim.evidence == Evidence::Human)
        .map(|claim| claim.id)
        .collect();
    Ok(Some(Question {
        feature: subject.id.clone(),
        checkpoint: checkpoint.to_string(),
        observed_surface,
        claims,
    }))
}

/// Seal one session over the stated outcomes and append each as its feature's receipt.
pub fn record(project: &Project, outcomes: Vec<Outcome>, rationale: &str) -> Result<String> {
    anyhow::ensure!(
        !outcomes.is_empty(),
        "a session records at least one stated outcome"
    );
    anyhow::ensure!(
        !rationale.trim().is_empty(),
        "a session requires a rationale"
    );
    let mut named = BTreeSet::new();
    let mut surfaces = BTreeMap::new();
    let mut receipts = Vec::new();
    for outcome in &outcomes {
        anyhow::ensure!(
            named.insert(outcome.feature.clone()),
            "{} has more than one outcome; a session answers one question per feature",
            outcome.feature
        );
        let subject = Subject::resolve(project, Workflow::Feature, &outcome.feature)?;
        anyhow::ensure!(
            project.feature_workflow_version()? >= 13,
            "a Human Validation Session requires Feature Workflow v13"
        );
        let question = question(project, &subject)?;
        let Some(question) = question.filter(|q| q.checkpoint == outcome.checkpoint) else {
            anyhow::bail!(
                "{} is not waiting on {}; a session answers only a current question",
                outcome.feature,
                outcome.checkpoint
            );
        };
        let stated: BTreeSet<_> = outcome.claims.keys().cloned().collect();
        let asked: BTreeSet<_> = question.claims.iter().cloned().collect();
        anyhow::ensure!(
            stated == asked,
            "{} must state an outcome for exactly its human claims at {}: {}",
            outcome.feature,
            outcome.checkpoint,
            question.claims.join(", ")
        );
        anyhow::ensure!(
            outcome
                .claims
                .values()
                .all(|v| v == "met" || v == "not-met"),
            "a claim outcome is `met` or `not-met`"
        );
        let mut bindings = checker::bindings_for_decision(project, &subject, &outcome.checkpoint)?;
        let surface = bindings
            .get("observed_surface")
            .cloned()
            .with_context(|| format!("{} has no Observed Surface to judge", outcome.feature))?;
        surfaces.insert(outcome.feature.clone(), surface);
        bindings.remove("session");
        receipts.push((outcome.clone(), bindings));
    }
    let session = Session {
        schema: 1,
        implementation_state: project.implementation_state()?,
        product_state: project.product_state()?,
        environment: crate::execution::Environment::Deterministic.as_str().into(),
        surfaces,
        outcomes,
        rationale: rationale.trim().to_string(),
        recorded_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    };
    let text = serde_json::to_string(&session)?;
    let id = hashing::text_sha256(&text);
    std::fs::create_dir_all(dir(project))?;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir(project).join(format!("{id}.json")))
        .and_then(|mut file| std::io::Write::write_all(&mut file, text.as_bytes()))
        .context("seal Human Validation Session")?;
    let store = ReceiptStore::at(&project.codeos_dir());
    for (outcome, mut bindings) in receipts {
        bindings.insert("session".into(), id.clone());
        store.append(&Receipt {
            workflow: "feature".into(),
            subject: outcome.feature,
            checkpoint: outcome.checkpoint,
            result: outcome.result,
            bindings,
            timestamp: session.recorded_at.clone(),
            rationale: Some(session.rationale.clone()),
            observation: None,
        })?;
    }
    Ok(id)
}

/// The sealed session a receipt binds, if it is intact and states exactly this receipt's outcome
/// on the surface the receipt binds.
pub fn attesting(project: &Project, receipt: &Receipt) -> Result<Option<Session>> {
    let Some(id) = receipt.bindings.get("session") else {
        return Ok(None);
    };
    let Ok(text) = std::fs::read_to_string(dir(project).join(format!("{id}.json"))) else {
        return Ok(None);
    };
    if hashing::text_sha256(&text) != *id {
        return Ok(None);
    }
    let session: Session = serde_json::from_str(&text).context("parse Human Validation Session")?;
    let stated = session.outcomes.iter().any(|outcome| {
        outcome.feature == receipt.subject
            && outcome.checkpoint == receipt.checkpoint
            && outcome.result == receipt.result
    });
    let surface =
        session.surfaces.get(&receipt.subject) == receipt.bindings.get("observed_surface");
    Ok((stated && surface).then_some(session))
}

/// The human outcome a receipt's session states for a claim.
pub fn claim_outcome(project: &Project, receipt: &Receipt, claim: &str) -> Result<Option<String>> {
    Ok(attesting(project, receipt)?.and_then(|session| {
        session
            .outcomes
            .into_iter()
            .find(|o| o.feature == receipt.subject && o.checkpoint == receipt.checkpoint)
            .and_then(|o| o.claims.get(claim).cloned())
    }))
}
