//! Engineering Health: one read-only result per engineering domain, each derived from existing
//! CodeOS authority or reported with an explicit absence status.

use crate::checker::{self, Subject};
use crate::contract::Workflow;
use crate::monitor::{DisplayStatus, FeatureStatus, OpenObservation};
use crate::project::Project;
use crate::verification::{self, RecordState, VerificationStore};
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Product,
    Specification,
    Architecture,
    Verification,
    Evidence,
    IndependentReview,
    ProjectQuality,
    Security,
    SupplyChain,
    Deployment,
    Release,
    Compatibility,
    OperationalLearning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HealthStatus {
    Current,
    Stale,
    Pass,
    Passed,
    Blocked,
    Unresolved,
    Unobserved,
    NotApplicable,
    NotConfigured,
    Qualified,
    Unqualified,
    Supported,
    Unsupported,
    Open,
    None,
}

impl HealthStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Current => "CURRENT",
            Self::Stale => "STALE",
            Self::Pass => "PASS",
            Self::Passed => "PASSED",
            Self::Blocked => "BLOCKED",
            Self::Unresolved => "UNRESOLVED",
            Self::Unobserved => "UNOBSERVED",
            Self::NotApplicable => "NOT_APPLICABLE",
            Self::NotConfigured => "NOT_CONFIGURED",
            Self::Qualified => "QUALIFIED",
            Self::Unqualified => "UNQUALIFIED",
            Self::Supported => "SUPPORTED",
            Self::Unsupported => "UNSUPPORTED",
            Self::Open => "OPEN",
            Self::None => "NONE",
        }
    }
}

use HealthStatus as S;

impl Domain {
    pub const ALL: [Domain; 13] = [
        Domain::Product,
        Domain::Specification,
        Domain::Architecture,
        Domain::Verification,
        Domain::Evidence,
        Domain::IndependentReview,
        Domain::ProjectQuality,
        Domain::Security,
        Domain::SupplyChain,
        Domain::Deployment,
        Domain::Release,
        Domain::Compatibility,
        Domain::OperationalLearning,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Product => "Product",
            Self::Specification => "Specification",
            Self::Architecture => "Architecture",
            Self::Verification => "Verification",
            Self::Evidence => "Evidence",
            Self::IndependentReview => "Independent review",
            Self::ProjectQuality => "Project quality",
            Self::Security => "Security",
            Self::SupplyChain => "Supply chain",
            Self::Deployment => "Deployment",
            Self::Release => "Release",
            Self::Compatibility => "Compatibility",
            Self::OperationalLearning => "Operational learning",
        }
    }

    /// The closed set of statuses this domain may report.
    pub fn allowed(self) -> &'static [HealthStatus] {
        match self {
            Self::Product => &[S::Current, S::Unresolved],
            Self::Specification => &[S::Current, S::Stale, S::Unresolved, S::Unobserved],
            Self::Architecture => &[S::Current, S::Stale, S::Unresolved, S::NotApplicable],
            Self::Verification => &[S::Pass, S::Blocked, S::Stale, S::Unobserved],
            Self::Evidence => &[S::Current, S::Stale, S::Unresolved, S::Unobserved],
            Self::IndependentReview => &[S::Passed, S::Blocked, S::Stale, S::Unobserved],
            Self::ProjectQuality | Self::Security | Self::SupplyChain | Self::Deployment => &[
                S::Pass,
                S::Blocked,
                S::Stale,
                S::NotApplicable,
                S::Unobserved,
                S::NotConfigured,
            ],
            Self::Release => &[S::Qualified, S::Unqualified],
            Self::Compatibility => &[S::Supported, S::Unsupported, S::NotConfigured],
            Self::OperationalLearning => &[S::Open, S::None, S::NotConfigured],
        }
    }
}

/// One domain's result. `basis` names the authoritative sources, or explains the absence.
#[derive(Debug, Serialize)]
pub struct Health {
    pub domain: Domain,
    pub status: HealthStatus,
    pub basis: Vec<String>,
}

fn health(domain: Domain, status: HealthStatus, basis: Vec<String>) -> Health {
    debug_assert!(
        domain.allowed().contains(&status),
        "{domain:?} may not report {status:?}"
    );
    Health {
        domain,
        status,
        basis,
    }
}

pub fn derive(
    project: &Project,
    features: &[FeatureStatus],
    open_observations: &[OpenObservation],
) -> Result<Vec<Health>> {
    let mut out = vec![product(project)?];
    out.push(checkpoint_domain(
        Domain::Specification,
        features,
        &["F1", "F2"],
        true,
        (S::Current, S::Unresolved, S::Unresolved),
    ));
    out.push(architecture(project, features)?);
    out.push(checkpoint_domain(
        Domain::Verification,
        features,
        &["F3", "F5", "F5d", "F5g"],
        false,
        (S::Pass, S::Blocked, S::Blocked),
    ));
    out.push(checkpoint_domain(
        Domain::Evidence,
        features,
        &["F6"],
        false,
        (S::Current, S::Unresolved, S::Unresolved),
    ));
    out.push(checkpoint_domain(
        Domain::IndependentReview,
        features,
        &["F7"],
        false,
        (S::Passed, S::Blocked, S::Blocked),
    ));
    out.extend(qualification(project, features)?);
    out.push(release());
    out.push(compatibility(project)?);
    out.push(operational_learning(project, open_observations)?);
    Ok(out)
}

fn product(project: &Project) -> Result<Health> {
    let charter = ".codeos/00-project/charter.md";
    Ok(if crate::evidence::approval_recorded(&project.charter())? {
        health(
            Domain::Product,
            S::Current,
            vec![format!("{charter} records approval")],
        )
    } else if project.charter().exists() {
        health(
            Domain::Product,
            S::Unresolved,
            vec![format!("{charter} has no recorded approval")],
        )
    } else {
        health(
            Domain::Product,
            S::Unresolved,
            vec![format!("{charter} does not exist")],
        )
    })
}

fn stale_reason(feature: &FeatureStatus) -> Option<&str> {
    (feature.historically_accepted && !feature.current_acceptance_valid)
        .then_some(feature.acceptance_reason.as_deref())
        .flatten()
}

/// Derive a domain from the feature checkpoints it covers. An accepted feature with a current
/// snapshot counts as positive evidence; an in-progress feature contributes its completed
/// checkpoints and, when its first unresolved checkpoint is in the domain, an issue. An accepted
/// feature that is no longer current makes the domain STALE, except that with `semantic_only` it is
/// STALE only for a semantic change and otherwise proves the Specification Package current.
fn checkpoint_domain(
    domain: Domain,
    features: &[FeatureStatus],
    checkpoints: &[&str],
    semantic_only: bool,
    (pass, blocked, waiting): (HealthStatus, HealthStatus, HealthStatus),
) -> Health {
    let mut stale = Vec::new();
    let mut issues = Vec::new();
    let mut issue_status = None;
    let mut passed = Vec::new();
    for feature in features {
        if let Some(reason) = stale_reason(feature) {
            if semantic_only && reason != "SEMANTIC_CHANGE_REQUIRED" {
                // Revalidation compares the Specification Package first, so any other reason
                // proves it still equals the accepted snapshot.
                passed.push(format!(
                    "{}: Specification Package equals the accepted snapshot ({reason})",
                    feature.id
                ));
            } else {
                stale.push(format!(
                    "{}: accepted snapshot is no longer current ({reason})",
                    feature.id
                ));
            }
            continue;
        }
        let applicable = |id: &str| {
            feature
                .workflow
                .checkpoints
                .iter()
                .any(|cp| cp.id == id && cp.status != DisplayStatus::NotApplicable)
        };
        if feature.historically_accepted {
            if checkpoints.iter().any(|id| applicable(id)) {
                passed.push(format!("{}: accepted snapshot is current", feature.id));
            }
            continue;
        }
        if let Some(current) = feature.workflow.current_checkpoint.as_deref() {
            if checkpoints.contains(&current) {
                let status = if feature.workflow.status == DisplayStatus::WaitingHuman {
                    waiting
                } else {
                    blocked
                };
                if issue_status.is_none() || status == blocked {
                    issue_status = Some(status);
                }
                issues.push(format!(
                    "{} {current}: {}",
                    feature.id,
                    feature
                        .workflow
                        .primary_issue
                        .as_deref()
                        .unwrap_or("not complete")
                ));
            }
        }
        for checkpoint in &feature.workflow.checkpoints {
            if checkpoints.contains(&checkpoint.id.as_str())
                && checkpoint.status == DisplayStatus::Complete
            {
                passed.push(format!("{} {}: complete", feature.id, checkpoint.id));
            }
        }
    }
    if !stale.is_empty() {
        health(domain, S::Stale, stale)
    } else if let Some(status) = issue_status {
        health(domain, status, issues)
    } else if !passed.is_empty() {
        health(domain, pass, passed)
    } else {
        health(
            domain,
            S::Unobserved,
            vec![format!(
                "no feature has reached {}",
                checkpoints.join(" / ")
            )],
        )
    }
}

fn architecture(project: &Project, features: &[FeatureStatus]) -> Result<Health> {
    let version = project.feature_workflow_version()?;
    if version < 8 {
        return Ok(health(
            Domain::Architecture,
            S::NotApplicable,
            vec![format!(
                "Feature Workflow v{version} does not govern feature architecture identity"
            )],
        ));
    }
    let stale: Vec<_> = features
        .iter()
        .filter(|feature| stale_reason(feature) == Some("ARCHITECTURE_CHANGE_REQUIRED"))
        .map(|feature| {
            format!(
                "{}: applicable architecture changed after acceptance",
                feature.id
            )
        })
        .collect();
    if !stale.is_empty() {
        return Ok(health(Domain::Architecture, S::Stale, stale));
    }
    let scopes = match crate::architecture::scope_approvals(project) {
        Ok(scopes) => scopes,
        Err(error) => {
            return Ok(health(
                Domain::Architecture,
                S::Unresolved,
                vec![format!("Architecture Scope is invalid: {error:#}")],
            ))
        }
    };
    if scopes.is_empty() {
        return Ok(health(
            Domain::Architecture,
            S::NotApplicable,
            vec!["no Architecture Scope exists".into()],
        ));
    }
    let pending: Vec<_> = scopes
        .iter()
        .filter(|(_, approved)| !approved)
        .map(|(path, _)| format!("{path}: approval pending"))
        .collect();
    Ok(if pending.is_empty() {
        health(
            Domain::Architecture,
            S::Current,
            scopes
                .into_iter()
                .map(|(path, _)| format!("{path}: approved"))
                .collect(),
        )
    } else {
        health(Domain::Architecture, S::Unresolved, pending)
    })
}

const QUALIFICATION_DOMAINS: [Domain; 4] = [
    Domain::ProjectQuality,
    Domain::Security,
    Domain::SupplyChain,
    Domain::Deployment,
];

fn all_qualification(status: HealthStatus, basis: Vec<String>) -> Vec<Health> {
    QUALIFICATION_DOMAINS
        .iter()
        .map(|domain| health(*domain, status, basis.clone()))
        .collect()
}

/// Project qualification domains come only from an identity-current `project_qualification`
/// record: one whose bindings still match the current Charter, configuration, qualification
/// script, implementation, and its feature's Specification Package.
fn qualification(project: &Project, features: &[FeatureStatus]) -> Result<Vec<Health>> {
    let version = project.feature_workflow_version()?;
    if version < 9 {
        return Ok(all_qualification(
            S::NotConfigured,
            vec![format!(
                "Feature Workflow v{version} has no project qualification"
            )],
        ));
    }
    if !project.root.join("scripts/codeos-qualify").is_file() {
        return Ok(all_qualification(
            S::NotConfigured,
            vec!["scripts/codeos-qualify does not exist".into()],
        ));
    }
    let store = VerificationStore::at(&project.codeos_dir());
    let mut current = None;
    let mut stale = Vec::new();
    for feature in features {
        let record = store.latest("feature", &feature.id, "F9", "project_qualification")?;
        if record.is_none() {
            continue;
        }
        let subject = Subject::resolve(project, Workflow::Feature, &feature.id)?;
        let want = checker::verification_bindings_for_mechanic(
            project,
            &subject,
            "F9",
            "project_qualification",
        )?;
        match verification::evaluate(record, &want) {
            RecordState::Current(record) | RecordState::Failed(record) => {
                if current.as_ref().is_none_or(
                    |(_, latest): &(String, verification::VerificationRecord)| {
                        record.timestamp > latest.timestamp
                    },
                ) {
                    current = Some((feature.id.clone(), record));
                }
            }
            RecordState::Stale { binding, .. } => stale.push(format!(
                "{} F9 project_qualification record is stale: '{binding}' changed",
                feature.id
            )),
            RecordState::Absent => {}
        }
    }
    let Some((feature, record)) = current else {
        return Ok(if stale.is_empty() {
            all_qualification(
                S::Unobserved,
                vec!["no project_qualification verification record exists".into()],
            )
        } else {
            all_qualification(S::Stale, stale)
        });
    };
    let source = format!(
        "{feature} F9 project_qualification record {}",
        record.timestamp
    );
    let details = record.details.unwrap_or_default();
    if let Some(error) = details.get("validation_error").and_then(|v| v.as_str()) {
        return Ok(all_qualification(
            S::Blocked,
            vec![format!("{source}: qualification invalid: {error}")],
        ));
    }
    let items = |key: &str| {
        details
            .get(key)
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
    };
    let field = |item: &serde_json::Value, key: &str| {
        item.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };
    let baseline = items("baseline");
    let constraints = items("constraints");
    let select = |domain: Domain| -> Vec<(String, String)> {
        let baseline_ids: &[&str] = match domain {
            Domain::ProjectQuality => &["static_analysis"],
            Domain::Security => &["committed_secrets"],
            Domain::SupplyChain => &["dependency_vulnerabilities", "dependency_licences"],
            _ => &[],
        };
        let phase = match domain {
            Domain::ProjectQuality => Some("acceptance"),
            Domain::Deployment => Some("deployment"),
            _ => None,
        };
        baseline
            .iter()
            .filter(|item| baseline_ids.contains(&field(item, "id").as_str()))
            .chain(
                constraints
                    .iter()
                    .filter(|item| phase.is_some_and(|phase| field(item, "phase") == phase)),
            )
            .map(|item| (field(item, "id"), field(item, "status")))
            .collect()
    };
    Ok(QUALIFICATION_DOMAINS
        .iter()
        .map(|domain| {
            let results = select(*domain);
            let basis = std::iter::once(source.clone())
                .chain(results.iter().map(|(id, status)| format!("{id}: {status}")))
                .collect();
            let status = if results.is_empty() {
                S::NotConfigured
            } else if results
                .iter()
                .any(|(_, status)| matches!(status.as_str(), "fail" | "unresolved"))
            {
                S::Blocked
            } else if results.iter().all(|(_, status)| status == "not_applicable") {
                S::NotApplicable
            } else {
                S::Pass
            };
            health(*domain, status, basis)
        })
        .collect())
}

/// A release binary verifies its embedded identity against the release manifest at startup and
/// refuses to run otherwise; a development build carries no release identity.
fn release() -> Health {
    let release = env!("CODEOS_RELEASE_ID");
    if cfg!(debug_assertions) || release == "UNQUALIFIED" {
        health(
            Domain::Release,
            S::Unqualified,
            vec!["development build without a qualified release identity".into()],
        )
    } else {
        health(
            Domain::Release,
            S::Qualified,
            vec![format!(
                "release {release}, binary verified against its manifest at startup"
            )],
        )
    }
}

fn compatibility(project: &Project) -> Result<Health> {
    let manifest = project
        .toolkit_root()
        .join("dba/04-tools/compatibility/support.json");
    let configuration = project.dba_configuration()?;
    let support = std::fs::read(&manifest)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| {
            value
                .get("configurations")?
                .get(&configuration)?
                .as_str()
                .map(str::to_string)
        });
    Ok(match support {
        Some(value) if value == "supported" => health(
            Domain::Compatibility,
            S::Supported,
            vec![format!(
                "compatibility/support.json: {configuration} supported"
            )],
        ),
        Some(value) => health(
            Domain::Compatibility,
            S::Unsupported,
            vec![format!(
                "compatibility/support.json: {configuration} {value}"
            )],
        ),
        None => health(
            Domain::Compatibility,
            S::NotConfigured,
            vec![format!(
                "no support entry for {configuration} in the toolkit compatibility manifest"
            )],
        ),
    })
}

fn operational_learning(project: &Project, open: &[OpenObservation]) -> Result<Health> {
    if project.operation_workflow_version()? < 2 {
        return Ok(health(
            Domain::OperationalLearning,
            S::NotConfigured,
            vec!["Observation v2 is not active in this configuration".into()],
        ));
    }
    Ok(if open.is_empty() {
        health(
            Domain::OperationalLearning,
            S::None,
            vec!["no Observation v2 identity lacks closure evidence".into()],
        )
    } else {
        health(
            Domain::OperationalLearning,
            S::Open,
            open.iter()
                .map(|observation| format!("{} has no closure evidence", observation.id))
                .collect(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::WorkflowStatus;

    fn stale_feature(reason: &str) -> FeatureStatus {
        FeatureStatus {
            id: "F-0002".into(),
            name: "fixture".into(),
            historically_accepted: true,
            current_acceptance_valid: false,
            accepted_snapshot: None,
            current_validity: None,
            acceptance_reason: Some(reason.into()),
            workflow: WorkflowStatus {
                status: DisplayStatus::Blocked,
                current_checkpoint: None,
                completed: 0,
                total: 0,
                primary_issue: None,
                additional_issue_count: 0,
                next_action: None,
                actor: None,
                reason: None,
                action: None,
                surfaces: None,
                checkpoints: vec![],
            },
        }
    }

    #[test]
    fn stale_accepted_features_mark_evidence_domains_stale_and_prove_unchanged_specification() {
        let derive = |domain, reason, semantic_only| {
            checkpoint_domain(
                domain,
                &[stale_feature(reason)],
                &[],
                semantic_only,
                (S::Current, S::Unresolved, S::Unresolved),
            )
        };
        let specification = derive(Domain::Specification, "REVALIDATION_REQUIRED", true);
        assert_eq!(specification.status, S::Current);
        assert_eq!(
            specification.basis,
            ["F-0002: Specification Package equals the accepted snapshot (REVALIDATION_REQUIRED)"]
        );
        assert_eq!(
            derive(Domain::Specification, "SEMANTIC_CHANGE_REQUIRED", true).status,
            S::Stale
        );
        for reason in [
            "SEMANTIC_CHANGE_REQUIRED",
            "ARCHITECTURE_CHANGE_REQUIRED",
            "REVALIDATION_REQUIRED",
        ] {
            assert_eq!(derive(Domain::Evidence, reason, false).status, S::Stale);
        }
    }

    #[test]
    fn every_domain_has_a_closed_status_set_with_an_absence_or_issue_state() {
        for domain in Domain::ALL {
            let allowed = domain.allowed();
            assert!(allowed.len() >= 2, "{domain:?}");
            // CURRENT and PASS never share a domain, so they cannot become synonyms.
            assert!(
                !(allowed.contains(&S::Current) && allowed.contains(&S::Pass)),
                "{domain:?}"
            );
        }
    }
}
