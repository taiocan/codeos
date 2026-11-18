//! VALIDITY WO-03: read-only deterministic preflight, run before spec approval or F4. It reports
//! cheap, provable defects early and records nothing. No prose interpretation and no AI review.

use crate::checker::{self, Subject};
use crate::contract::Workflow;
use crate::project::Project;
use anyhow::Result;

pub struct Finding {
    pub check: &'static str,
    pub ok: bool,
    pub detail: String,
}

fn finding(check: &'static str, result: Result<String>) -> Finding {
    match result {
        Ok(detail) => Finding {
            check,
            ok: true,
            detail,
        },
        Err(error) => Finding {
            check,
            ok: false,
            detail: format!("{error:#}"),
        },
    }
}

pub fn run(project: &Project, feature: &str) -> Vec<Finding> {
    let mut findings = vec![finding("specification", specification(project, feature))];
    match Subject::resolve(project, Workflow::Feature, feature) {
        Ok(subject) => {
            findings.push(finding(
                "validation-claims",
                checker::subject_claims(project, &subject).map(|claims| match claims {
                    Some(claims) => format!("{} claims attributable", claims.len()),
                    None => "legacy coarse claims".into(),
                }),
            ));
            findings.push(finding(
                "journey-contribution",
                checker::product_journey_target(project, &subject)
                    .map(|target| format!("journey target '{target}'")),
            ));
        }
        Err(error) => findings.push(Finding {
            check: "subject",
            ok: false,
            detail: format!("{error:#}"),
        }),
    }
    findings.push(finding("qualification", qualification(project)));
    findings
}

/// Every Specification Package artifact exists, its frontmatter parses, and the three share one
/// `semantic_revision` and `semantic_change` (Feature Workflow v3).
fn specification(project: &Project, feature: &str) -> Result<String> {
    let mut shared: Option<(serde_yaml::Value, serde_yaml::Value)> = None;
    for (artifact, _) in crate::validity::ARTIFACTS {
        let path = crate::validity::artifact_path(project, feature, artifact).expect("known");
        let text = std::fs::read_to_string(&path)
            .map_err(|_| anyhow::anyhow!("{artifact} is missing at {}", path.display()))?;
        let frontmatter = text
            .strip_prefix("---\n")
            .and_then(|rest| rest.split_once("\n---\n"))
            .map(|(frontmatter, _)| frontmatter)
            .ok_or_else(|| anyhow::anyhow!("{artifact} has no frontmatter"))?;
        let parsed: serde_yaml::Value = serde_yaml::from_str(frontmatter)
            .map_err(|error| anyhow::anyhow!("{artifact} frontmatter does not parse: {error}"))?;
        let pair = (
            parsed["semantic_revision"].clone(),
            parsed["semantic_change"].clone(),
        );
        match &shared {
            None => shared = Some(pair),
            Some(first) if *first != pair => anyhow::bail!(
                "{artifact} semantic_revision or semantic_change differs from the intent's"
            ),
            Some(_) => {}
        }
    }
    Ok("intent, contract and event schema parse and agree on their semantic revision".into())
}

/// The project's declared qualification in acceptance mode: Charter prerequisites such as C-1
/// and the dependency, licence, secret and static-analysis baseline.
fn qualification(project: &Project) -> Result<String> {
    if !project.root.join("scripts/codeos-qualify").is_file() {
        return Ok("no project qualification declared".into());
    }
    let outcome = crate::qualification::run(project, "acceptance");
    anyhow::ensure!(outcome.ok, "{}", outcome.summary);
    Ok(outcome.summary)
}
