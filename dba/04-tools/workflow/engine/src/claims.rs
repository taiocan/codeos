//! Feature Workflow v13 validation claims (DQA WO-06): the Contract-owned `## Validation Claims`
//! table. The Contract defines each claim; this module only reads it and fails closed on anything
//! it cannot attribute.

use crate::hashing;
use crate::project::Project;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationClaim {
    /// Owner-qualified: `F-NNNN/<kebab-case>`.
    pub id: String,
    pub text: String,
    pub checkpoint: String,
    pub evidence: Evidence,
    /// sha256 of the checkpoint and the claim text with whitespace collapsed.
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evidence {
    /// A passed record of this verification at the claim's checkpoint.
    Run(String),
    /// An observation naming the claim in the selected production-source evidence.
    Production,
    /// An explicit Human Validation Session outcome for the claim.
    Human,
}

/// A feature's prospective claims, or `None` for a Contract without the section (legacy coarse
/// claims). `verifications` maps each applicable checkpoint to the verifications it requires;
/// `reality_claims` are the feature's reality-dependent claim IDs.
pub fn validation_claims(
    project: &Project,
    feature: &str,
    verifications: &BTreeMap<String, Vec<String>>,
    reality_claims: &[String],
) -> Result<Option<Vec<ValidationClaim>>> {
    let text = std::fs::read_to_string(project.contract(feature)).unwrap_or_default();
    let Some(section) = text.split("\n## Validation Claims").nth(1) else {
        return Ok(None);
    };
    anyhow::ensure!(
        project.feature_workflow_version()? >= 13,
        "## Validation Claims requires Feature Workflow v13"
    );
    let section = section.split("\n## ").next().unwrap_or_default();
    let mut claims = Vec::new();
    let mut seen = BTreeSet::new();
    for line in section
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('|'))
    {
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cells.first() == Some(&"Claim ID")
            || cells
                .iter()
                .all(|c| c.chars().all(|ch| ch == '-' || ch == ':'))
        {
            continue;
        }
        anyhow::ensure!(
            cells.len() == 4,
            "Validation Claims row must have Claim ID, Claim, Checkpoint and Evidence: {line}"
        );
        let (id, claim, checkpoint, evidence) = (cells[0], cells[1], cells[2], cells[3]);
        let Some(slug) = id.strip_prefix(&format!("{feature}/")) else {
            anyhow::bail!("validation claim '{id}' is not owned by {feature}");
        };
        anyhow::ensure!(
            !slug.is_empty()
                && slug.split('-').all(|part| {
                    !part.is_empty()
                        && part
                            .chars()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                }),
            "validation claim '{id}' is not `{feature}/<kebab-case>`"
        );
        anyhow::ensure!(
            seen.insert(id.to_string()),
            "validation claim '{id}' is declared twice"
        );
        let text = claim.split_whitespace().collect::<Vec<_>>().join(" ");
        anyhow::ensure!(
            !text.is_empty(),
            "validation claim '{id}' has no claim text"
        );
        let Some(required) = verifications.get(checkpoint) else {
            anyhow::bail!(
                "validation claim '{id}' names checkpoint '{checkpoint}', which does not apply"
            );
        };
        let evidence = match evidence.strip_prefix("run:") {
            Some(verification) if required.iter().any(|v| v == verification) => {
                Evidence::Run(verification.to_string())
            }
            Some(verification) => anyhow::bail!(
                "validation claim '{id}' cites run:{verification}, which {checkpoint} does not require"
            ),
            None if evidence == "observation:production" => {
                anyhow::ensure!(
                    checkpoint == "F5" && reality_claims.iter().any(|claim| claim == slug),
                    "validation claim '{id}' cites production observation but is not an F5 reality-dependent claim"
                );
                Evidence::Production
            }
            None if evidence == "observation:human" => {
                anyhow::ensure!(
                    matches!(checkpoint, "F4" | "F8"),
                    "validation claim '{id}' cites human observation outside F4 or F8"
                );
                Evidence::Human
            }
            None => anyhow::bail!("validation claim '{id}' has unknown evidence '{evidence}'"),
        };
        claims.push(ValidationClaim {
            digest: hashing::text_sha256(&format!("{checkpoint}\n{text}")),
            id: id.to_string(),
            text,
            checkpoint: checkpoint.to_string(),
            evidence,
        });
    }
    Ok(Some(claims))
}

/// Claims whose meaning digest differs from the one the accepted bundle recorded for the same ID.
pub fn changed_meanings(
    claims: &[ValidationClaim],
    accepted: &BTreeMap<String, String>,
) -> Vec<String> {
    claims
        .iter()
        .filter(|claim| {
            accepted
                .get(&claim.id)
                .is_some_and(|digest| *digest != claim.digest)
        })
        .map(|claim| claim.id.clone())
        .collect()
}
