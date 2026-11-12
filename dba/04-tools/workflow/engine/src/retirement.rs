//! Feature retirement (Workflow Governance v5, Operation & Learning v3): an accepted feature
//! leaves the governed feature set only through one human `retirement` receipt, and its accepted
//! history stays immutable.

use crate::checker::{self, JourneyContribution, Subject};
use crate::contract::{Workflow, RETIREMENT_ROUTE};
use crate::hashing;
use crate::observations::ObservationStore;
use crate::project::Project;
use crate::receipts::{Receipt, ReceiptStore};
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// A historically accepted feature withdrawn by a retirement receipt, with no remaining artifact.
#[derive(Debug, Serialize)]
pub struct RetiredFeature {
    pub id: String,
    pub retired_at: String,
    pub rationale: String,
}

/// A retirement-state violation that blocks progression.
pub struct Violation {
    pub feature: String,
    pub issue: String,
    pub next_action: String,
}

/// The feature's Intent, Contract, and Event Schema files that still exist.
pub fn specification_artifacts(project: &Project, feature: &str) -> Vec<PathBuf> {
    [
        project.intent(feature),
        project.contract(feature),
        project.event_schema(feature),
    ]
    .into_iter()
    .filter(|path| path.exists())
    .collect()
}

fn latest_receipt(project: &Project, feature: &str, checkpoint: &str) -> Result<Option<Receipt>> {
    ReceiptStore::at(&project.codeos_dir()).latest("feature", feature, checkpoint)
}

/// The content identity of the feature's latest acceptance receipt line, as closure evidence names
/// receipts.
fn acceptance_identity(project: &Project, feature: &str) -> Result<Option<String>> {
    let path = ReceiptStore::at(&project.codeos_dir()).path().to_path_buf();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(None);
    };
    Ok(text
        .lines()
        .rfind(|line| {
            serde_json::from_str::<Receipt>(line).is_ok_and(|r| {
                r.workflow == "feature"
                    && r.subject == feature
                    && r.checkpoint == "acceptance"
                    && r.result == "accepted"
            })
        })
        .map(hashing::text_sha256))
}

fn advances_journey(project: &Project, feature: &str, acceptance: &Receipt) -> Result<bool> {
    if let Some(bundle_id) = acceptance.bindings.get("candidate_bundle") {
        let bundle = crate::candidate::CandidateStore::at(&project.codeos_dir()).read(bundle_id)?;
        return Ok(bundle.journey_previous != bundle.journey_target);
    }
    Ok(matches!(
        checker::journey_contribution(&project.contract(feature))?,
        JourneyContribution::Advance { .. }
    ))
}

/// Refuse the retirement receipt unless every Operation v3 precondition holds; return the bindings
/// it records.
pub fn decision_bindings(project: &Project, feature: &str) -> Result<BTreeMap<String, String>> {
    anyhow::ensure!(
        project.workflow_governance_version()? >= 5,
        "feature retirement requires Workflow Governance v5"
    );
    let store = ObservationStore::at(&project.codeos_dir());
    let mut routed = false;
    for route in store.routes()? {
        if route.route == RETIREMENT_ROUTE
            && route.target_id.as_deref() == Some(feature)
            && store.closure(&route.observation)?.is_none()
        {
            routed = true;
        }
    }
    anyhow::ensure!(
        routed,
        "no open feature_retirement observation targets {feature}"
    );
    anyhow::ensure!(
        latest_receipt(project, feature, "retirement")?.is_none(),
        "{feature} is already retired"
    );
    let acceptance = latest_receipt(project, feature, "acceptance")?
        .filter(|r| r.result == "accepted")
        .with_context(|| format!("{feature} has no acceptance receipt"))?;
    anyhow::ensure!(
        !advances_journey(project, feature, &acceptance)?,
        "{feature} advanced the Product Journey Frontier; retiring it changes the Charter's primary journey"
    );
    let scopes = crate::architecture::scopes_naming(project, feature)?;
    anyhow::ensure!(
        scopes.is_empty(),
        "Architecture Scope still names {feature}: {}",
        scopes.join(", ")
    );
    for other in project.feature_ids()? {
        if other == feature {
            continue;
        }
        let subject = Subject::resolve(project, Workflow::Feature, &other)?;
        anyhow::ensure!(
            !subject
                .inherited_claims
                .iter()
                .any(|claim| claim.upstream_feature == feature),
            "{other} inherits a claim from {feature}; revise {other} first"
        );
    }

    let mut bindings = BTreeMap::new();
    bindings.insert(
        "acceptance".into(),
        acceptance_identity(project, feature)?.context("acceptance receipt line not found")?,
    );
    for (name, path) in [
        ("intent", project.intent(feature)),
        ("contract", project.contract(feature)),
        ("event_schema", project.event_schema(feature)),
        ("charter", project.charter()),
    ] {
        let hash = hashing::file_sha256(&path)?.with_context(|| {
            format!(
                "{} is missing; record retirement before removal",
                path.display()
            )
        })?;
        bindings.insert(name.into(), hash);
    }
    Ok(bindings)
}

/// Retired features and retirement-state violations, from Workflow Governance v5.
pub fn derive(
    project: &Project,
    specified: &BTreeSet<String>,
) -> Result<(Vec<RetiredFeature>, Vec<Violation>)> {
    if project.workflow_governance_version()? < 5 {
        return Ok((Vec::new(), Vec::new()));
    }
    let mut accepted = BTreeSet::new();
    let mut retired = BTreeMap::new();
    for receipt in ReceiptStore::at(&project.codeos_dir()).all()? {
        if receipt.workflow != "feature" {
            continue;
        }
        match (receipt.checkpoint.as_str(), receipt.result.as_str()) {
            ("acceptance", "accepted") => {
                accepted.insert(receipt.subject);
            }
            ("retirement", "retired") => {
                retired.insert(receipt.subject.clone(), receipt);
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    let mut violations = Vec::new();
    for (feature, receipt) in &retired {
        if specified.contains(feature) {
            violations.push(Violation {
                feature: feature.clone(),
                issue: format!(
                    "retired feature {feature} still has Specification Package artifacts; its Feature ID is never reused"
                ),
                next_action: format!(
                    "remove {feature}'s Intent, Contract, Event Schema, and implementation in one commit"
                ),
            });
        } else {
            out.push(RetiredFeature {
                id: feature.clone(),
                retired_at: receipt.timestamp.clone(),
                rationale: receipt.rationale.clone().unwrap_or_default(),
            });
        }
    }
    for feature in accepted {
        if !retired.contains_key(&feature) && !specified.contains(&feature) {
            violations.push(Violation {
                issue: format!(
                    "accepted feature {feature} was removed without a retirement decision"
                ),
                next_action: format!(
                    "restore {feature}'s Specification Package; to retire it, route a feature_retirement observation and record retirement before removal"
                ),
                feature,
            });
        }
    }
    Ok((out, violations))
}
