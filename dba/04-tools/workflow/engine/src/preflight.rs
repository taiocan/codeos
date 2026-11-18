//! VALIDITY WO-03: read-only deterministic preflight, run before spec approval or F4. It reports
//! cheap, provable defects early and records nothing. It inspects authoritative state and never
//! runs a verification, a qualification, an audit or any network work. No prose interpretation
//! and no AI review.

use crate::checker::{self, Subject};
use crate::contract::Workflow;
use crate::project::Project;
use crate::verification::{self, RecordState, VerificationStore};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Pass,
    Blocked(String),
    NotApplicable(String),
}

pub struct Finding {
    pub check: &'static str,
    pub status: Status,
    pub detail: String,
}

impl Finding {
    pub fn blocked(&self) -> bool {
        matches!(self.status, Status::Blocked(_))
    }
}

fn computed(check: &'static str, result: Result<String>) -> Finding {
    match result {
        Ok(detail) => Finding {
            check,
            status: Status::Pass,
            detail,
        },
        Err(error) => Finding {
            check,
            status: Status::Blocked("DEFECT".into()),
            detail: format!("{error:#}"),
        },
    }
}

pub fn run(project: &Project, feature: &str) -> Vec<Finding> {
    let mut findings = vec![computed("specification", specification(project, feature))];
    match Subject::resolve(project, Workflow::Feature, feature) {
        Ok(subject) => {
            findings.push(computed(
                "validation-claims",
                checker::subject_claims(project, &subject).map(|claims| match claims {
                    Some(claims) => format!("{} claims attributable", claims.len()),
                    None => "legacy coarse claims".into(),
                }),
            ));
            findings.push(computed(
                "journey-contribution",
                checker::product_journey_target(project, &subject)
                    .map(|target| format!("journey target '{target}'")),
            ));
            findings.push(qualification(project, &subject));
        }
        Err(error) => findings.push(Finding {
            check: "subject",
            status: Status::Blocked("DEFECT".into()),
            detail: format!("{error:#}"),
        }),
    }
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

/// The feature's latest acceptance qualification record, the authoritative state of its Charter
/// prerequisites (such as C-1) and dependency baseline. Inspected, never run.
fn qualification(project: &Project, subject: &Subject) -> Finding {
    let finding = |status, detail: String| Finding {
        check: "qualification",
        status,
        detail,
    };
    if !project.root.join("scripts/codeos-qualify").is_file()
        || project.feature_workflow_version().unwrap_or(0) < 9
    {
        return finding(
            Status::NotApplicable("NO_PROJECT_QUALIFICATION".into()),
            "the project declares no qualification".into(),
        );
    }
    let required =
        |detail: String| finding(Status::Blocked("QUALIFICATION_REQUIRED".into()), detail);
    let record = match VerificationStore::at(&project.codeos_dir()).latest(
        "feature",
        &subject.id,
        "candidate",
        "project_qualification",
    ) {
        Ok(Some(record)) => record,
        Ok(None) => {
            return required(format!(
                "no acceptance qualification of {} exists",
                subject.id
            ))
        }
        Err(error) => return required(format!("{error:#}")),
    };
    let additions = checker::declared_dependency_additions(&record);
    let want = match checker::qualification_bindings(project, subject, "candidate", &additions) {
        Ok(want) => want,
        Err(error) => return required(format!("{error:#}")),
    };
    match verification::evaluate(Some(record), &want) {
        RecordState::Current(record) => finding(
            Status::Pass,
            format!("current acceptance qualification of {}", record.timestamp),
        ),
        RecordState::Failed(record) => finding(
            Status::Blocked("QUALIFICATION_BLOCKERS".into()),
            format!(
                "current qualification blocked by {}",
                blockers(&record).join(", ")
            ),
        ),
        RecordState::Stale { binding, .. } => required(format!(
            "the latest qualification is stale: '{binding}' changed"
        )),
        RecordState::Absent => required("no acceptance qualification exists".into()),
    }
}

/// The acceptance-phase constraints and baseline items a recorded qualification did not pass.
fn blockers(record: &verification::VerificationRecord) -> Vec<String> {
    let Some(details) = record.details.as_ref() else {
        return vec!["unrecorded".into()];
    };
    let mut out = Vec::new();
    for item in details["constraints"].as_array().into_iter().flatten() {
        if item["phase"] == "acceptance"
            && matches!(item["status"].as_str(), Some("fail" | "unresolved"))
        {
            out.push(item["id"].as_str().unwrap_or("?").to_string());
        }
    }
    for item in details["baseline"].as_array().into_iter().flatten() {
        if item["status"] == "fail" {
            out.push(item["id"].as_str().unwrap_or("?").to_string());
        }
    }
    out
}
