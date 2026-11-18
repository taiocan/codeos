//! Feature Workflow v14 (VALIDITY WO-02): dependency-accurate validity. A decision stays current
//! exactly while its own dependency set is unchanged; recorded values that are not dependencies
//! never make it stale.

use crate::checker::Subject;
use crate::hashing;
use crate::project::Project;
use crate::receipts::{Receipt, ReceiptStore};
use crate::verification::VerificationRecord;
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The Specification Package artifacts whose meaning decisions bind, with their binding keys.
pub const ARTIFACTS: [(&str, &str); 3] = [
    ("intent", "intent_meaning"),
    ("contract", "contract_meaning"),
    ("event_schema", "event_schema_meaning"),
];

pub const EQUIVALENCE: &str = "semantic_equivalence";

/// The file-hash bindings a v14 record or receipt keeps as history next to the meaning keys.
pub const SPEC_FILE_KEYS: [&str; 3] = ["intent", "contract", "event_schema"];

/// Whether this project selects Feature Workflow v14 or later.
pub fn active(project: &Project) -> Result<bool> {
    Ok(project.feature_workflow_version()? >= 14)
}

/// The v14 dependency set of a receipt-bearing feature checkpoint, besides the narrowest product
/// binding F4, F6 and F8 judge (`observed_surface`, else `delivered_state` or `product_state`).
pub fn dependencies(checkpoint: &str) -> Option<&'static [&'static str]> {
    Some(match checkpoint {
        "early_preview" | "final_ux_validation" => &["intent_meaning", "judged_claims", "session"],
        "reconciliation" => &[
            "intent_meaning",
            "contract_meaning",
            "event_schema_meaning",
            "codeos_yaml",
            "production_evidence",
        ],
        "acceptance" => &[
            "intent_meaning",
            "contract_meaning",
            "event_schema_meaning",
            "architecture",
        ],
        _ => return None,
    })
}

/// Whether `checkpoint` judges the product through its narrowest product binding.
pub fn judges_product(checkpoint: &str) -> bool {
    matches!(
        checkpoint,
        "early_preview" | "reconciliation" | "final_ux_validation"
    )
}

/// The meaning identity of one artifact: the canonical parsed frontmatter followed by the exact
/// body bytes. A file without frontmatter is all body.
pub fn spec_meaning(path: &Path) -> Result<Option<String>> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(None);
    };
    let (frontmatter, body) = match text
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
    {
        Some((frontmatter, body)) => (frontmatter, body),
        None => ("", text.as_str()),
    };
    let parsed: serde_yaml::Value = if frontmatter.is_empty() {
        serde_yaml::Value::Null
    } else {
        serde_yaml::from_str(frontmatter)
            .with_context(|| format!("parse the frontmatter of {}", path.display()))?
    };
    // serde_json maps are ordered by key, so the rendering is canonical.
    let canonical = serde_json::to_string(&serde_json::to_value(parsed)?)?;
    Ok(Some(hashing::text_sha256(&format!(
        "{canonical}\n---\n{body}"
    ))))
}

/// The artifact's meaning identity, resolved through the feature's Semantic Equivalence Receipts:
/// a receipt maps the exact identity it covers back to the one it was equivalent to.
pub fn resolved_meaning(
    project: &Project,
    feature: &str,
    artifact: &str,
    path: &Path,
) -> Result<Option<String>> {
    let Some(mut identity) = spec_meaning(path)? else {
        return Ok(None);
    };
    let receipts = equivalences(project, feature, artifact)?;
    let mut seen = BTreeSet::new();
    while seen.insert(identity.clone()) {
        match receipts
            .iter()
            .rev()
            .find(|r| r.bindings.get("to") == Some(&identity))
        {
            Some(receipt) => identity = receipt.bindings["from"].clone(),
            None => break,
        }
    }
    Ok(Some(identity))
}

fn equivalences(project: &Project, feature: &str, artifact: &str) -> Result<Vec<Receipt>> {
    Ok(ReceiptStore::at(&project.codeos_dir())
        .all()?
        .into_iter()
        .filter(|r| {
            r.workflow == "feature"
                && r.subject == feature
                && r.checkpoint == EQUIVALENCE
                && r.result == "equivalent"
                && r.bindings.get("artifact").map(String::as_str) == Some(artifact)
                && r.bindings.contains_key("from")
                && r.bindings.contains_key("to")
        })
        .collect())
}

pub fn artifact_path(
    project: &Project,
    feature: &str,
    artifact: &str,
) -> Option<std::path::PathBuf> {
    Some(match artifact {
        "intent" => project.intent(feature),
        "contract" => project.contract(feature),
        "event_schema" => project.event_schema(feature),
        _ => return None,
    })
}

/// The bindings of a Semantic Equivalence Receipt (Workflow Governance v6).
pub fn equivalence_bindings(
    project: &Project,
    feature: &str,
    artifact: &str,
    from: &str,
) -> Result<BTreeMap<String, String>> {
    anyhow::ensure!(
        project.workflow_governance_version()? >= 6,
        "semantic_equivalence requires Workflow Governance v6"
    );
    let key = ARTIFACTS
        .iter()
        .find(|(name, _)| *name == artifact)
        .map(|(_, key)| *key)
        .with_context(|| {
            format!("unknown artifact '{artifact}'; use intent, contract or event_schema")
        })?;
    let path = artifact_path(project, feature, artifact).expect("known artifact");
    let to = spec_meaning(&path)?.with_context(|| format!("missing {artifact} of {feature}"))?;
    anyhow::ensure!(
        to != from,
        "the {artifact} of {feature} is already at identity {from}"
    );
    let bound = ReceiptStore::at(&project.codeos_dir())
        .all()?
        .iter()
        .any(|r| r.subject == feature && r.bindings.get(key).map(String::as_str) == Some(from));
    anyhow::ensure!(
        bound,
        "no receipt of {feature} binds {key} {from}; name an identity a decision depended on"
    );
    Ok(BTreeMap::from([
        ("artifact".into(), artifact.into()),
        ("from".into(), from.into()),
        ("to".into(), to),
    ]))
}

/// The meaning digests of the subject's claims at `checkpoint_id` (F4, F8), or `legacy` when the
/// Contract has no Validation Claims table.
pub fn judged_claims(project: &Project, subject: &Subject, checkpoint_id: &str) -> Result<String> {
    let Some(claims) = crate::checker::subject_claims(project, subject).unwrap_or(None) else {
        return Ok(hashing::text_sha256("legacy"));
    };
    let mut lines: Vec<String> = claims
        .iter()
        .filter(|claim| claim.checkpoint == checkpoint_id)
        .map(|claim| format!("{}={}", claim.id, claim.digest))
        .collect();
    lines.sort();
    Ok(hashing::text_sha256(&lines.join("\n")))
}

/// What a verification established, without provenance noise (Feature Workflow v14): its
/// checkpoint, name, result and bindings without `execution_identity`, and its evidence content.
pub fn content_identity(project: &Project, record: &VerificationRecord) -> Result<String> {
    let mut bindings = record.bindings.clone();
    let execution_identity = bindings.remove("execution_identity");
    if bindings.contains_key("intent_meaning") {
        for key in SPEC_FILE_KEYS {
            bindings.remove(key);
        }
    }
    let evidence = match record
        .details
        .as_ref()
        .and_then(|d| d.get("evidence_set"))
        .and_then(|v| v.as_str())
    {
        Some(id) => {
            let set = execution_identity
                .as_deref()
                .map(|identity| crate::evidence_set::read(project, identity, id))
                .transpose()?
                .flatten()
                .with_context(|| {
                    format!("Validation Evidence Set {id} cited by a record is missing")
                })?;
            serde_json::json!({
                "environment": set.environment,
                "execution_inputs": set.execution_inputs,
                "result": set.result,
                "observations": set.observations,
            })
        }
        None => record.details.clone().unwrap_or(serde_json::Value::Null),
    };
    let content = serde_json::json!({
        "checkpoint": record.checkpoint,
        "verification": record.verification,
        "result": record.result,
        "bindings": bindings,
        "evidence": evidence,
    });
    Ok(hashing::text_sha256(&serde_json::to_string(&content)?))
}

/// The configuration a v14 Candidate Bundle depends on: the selected reviewer tool contract and
/// review policy (Review Policy v18), not the bytes of `codeos.yaml`.
pub fn review_configuration_identity(project: &Project) -> Result<String> {
    let contract = project
        .selected_policy("reviewer_tool_contract")?
        .unwrap_or_default();
    let policy = project
        .selected_policy("review_policy")?
        .unwrap_or_default();
    Ok(hashing::text_sha256(&format!(
        "reviewer_tool_contract={contract}\nreview_policy={policy}"
    )))
}

/// A receipt's content identity: what was decided, against which bindings, without its timestamp.
pub fn receipt_content_identity(receipt: &Receipt) -> Result<String> {
    let content = serde_json::json!({
        "workflow": receipt.workflow,
        "subject": receipt.subject,
        "checkpoint": receipt.checkpoint,
        "result": receipt.result,
        "bindings": receipt.bindings,
    });
    Ok(hashing::text_sha256(&serde_json::to_string(&content)?))
}
