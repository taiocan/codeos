//! Read-only Development Evidence projection across Platform Baseline areas.

use crate::candidate::{self, CandidateBundle, CandidateStore};
use crate::checker::{self, Subject};
use crate::contract::Workflow;
use crate::hashing;
use crate::monitor::FeatureStatus;
use crate::project::Project;
use crate::verification::{self, RecordState, VerificationRecord, VerificationStore};
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Area {
    Persistence,
    Backend,
    Gui,
    IntegratedProduct,
}

impl Area {
    pub const ALL: [Self; 4] = [
        Self::Persistence,
        Self::Backend,
        Self::Gui,
        Self::IntegratedProduct,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Persistence => "Persistence",
            Self::Backend => "Backend",
            Self::Gui => "GUI",
            Self::IntegratedProduct => "Integrated product",
        }
    }

    pub fn platform_label(self) -> &'static str {
        match self {
            Self::Persistence => "Persistence (PostgreSQL)",
            Self::Backend => "Backend (Rust)",
            Self::Gui => "GUI (Svelte)",
            Self::IntegratedProduct => "Integrated product",
        }
    }

    fn mechanics(self) -> &'static [&'static str] {
        match self {
            Self::Persistence => &["data_integrity"],
            Self::Backend => &["smoke", "behavior"],
            Self::Gui => &["playwright"],
            Self::IntegratedProduct => {
                &["smoke", "product_journey", "data_integrity", "playwright"]
            }
        }
    }

    fn applies(self, subject: &Subject) -> bool {
        match self {
            Self::Persistence => subject.persistence,
            Self::Backend => subject.backend,
            Self::Gui => subject.gui,
            Self::IntegratedProduct => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssuranceStatus {
    Pass,
    Blocked,
    Stale,
    Unobserved,
    NotApplicable,
}

impl AssuranceStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Blocked => "BLOCKED",
            Self::Stale => "STALE",
            Self::Unobserved => "UNOBSERVED",
            Self::NotApplicable => "NOT_APPLICABLE",
        }
    }

    fn severity(self) -> u8 {
        match self {
            Self::Blocked => 4,
            Self::Stale => 3,
            Self::Unobserved => 2,
            Self::Pass => 1,
            Self::NotApplicable => 0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PlatformAssurance {
    pub area: Area,
    pub status: AssuranceStatus,
    pub subjects: Vec<String>,
    pub basis: Vec<String>,
}

impl PlatformAssurance {
    pub fn subject_summary(&self) -> String {
        abbreviated(&self.subjects)
    }

    pub fn evidence_summary(&self) -> String {
        let issues = self.basis.iter().filter(|item| is_issue(item)).count();
        let current = self.basis.len().saturating_sub(issues);
        match self.status {
            AssuranceStatus::NotApplicable => "no applicable subjects".into(),
            AssuranceStatus::Pass => format!("{current} current records"),
            _ if current == 0 => format!("{issues} issues"),
            _ => format!("{issues} issues · {current} current"),
        }
    }

    pub fn compact_summary(&self) -> String {
        match self.subjects.len() {
            0 => self.evidence_summary(),
            1 | 2 => format!("{} · {}", self.subjects.join(", "), self.evidence_summary()),
            count => format!("{count} subjects · {}", self.evidence_summary()),
        }
    }
}

struct ExactBundle {
    id: String,
    bundle: CandidateBundle,
}

pub fn derive(project: &Project, features: &[FeatureStatus]) -> Result<Vec<PlatformAssurance>> {
    let store = VerificationStore::at(&project.codeos_dir());
    let mut by_identity = BTreeMap::new();
    for record in store.all()? {
        // A schema 3 Candidate Bundle cites content identities (Feature Workflow v14).
        if crate::validity::active(project)? {
            if let Ok(identity) = crate::validity::content_identity(project, &record) {
                by_identity.insert(identity, record.clone());
            }
        }
        by_identity.insert(hashing::record_identity(&record)?, record);
    }

    let mut rows = Vec::new();
    for area in Area::ALL {
        let mut status = AssuranceStatus::NotApplicable;
        let mut subjects = Vec::new();
        let mut basis = Vec::new();
        for feature in features {
            let subject = Subject::resolve(project, Workflow::Feature, &feature.id)?;
            if !subject.impact_accounting_complete {
                subjects.push(feature.id.clone());
                promote(&mut status, AssuranceStatus::Unobserved);
                basis.push(format!(
                    "{}: Feature Impact Accounting is missing or incomplete",
                    feature.id
                ));
                continue;
            }
            if !area.applies(&subject) {
                continue;
            }
            subjects.push(feature.id.clone());
            promote(&mut status, AssuranceStatus::Pass);
            let required: Vec<_> = checker::required_verification_records(project, &subject)?
                .into_iter()
                .filter(|(_, mechanic)| area.mechanics().contains(mechanic))
                .collect();
            if required.is_empty() {
                promote(&mut status, AssuranceStatus::Unobserved);
                basis.push(format!(
                    "{}: no applicable verification mechanic",
                    feature.id
                ));
                continue;
            }

            let bundle = exact_bundle(project, feature)?;
            if feature.historically_accepted && bundle.is_none() {
                promote(&mut status, AssuranceStatus::Unobserved);
                basis.push(format!(
                    "{}: accepted evidence has no exact current Candidate Bundle identity",
                    feature.id
                ));
                continue;
            }
            for (checkpoint, mechanic) in required {
                let exact = bundle.as_ref().and_then(|bundle| {
                    bundle
                        .bundle
                        .verification_records
                        .get(&format!("{checkpoint}/{mechanic}"))
                        .and_then(|identity| {
                            by_identity
                                .get(identity)
                                .cloned()
                                .map(|record| (record, identity.clone(), Some(bundle.id.clone())))
                        })
                });
                let evidence = match exact {
                    Some(value) => Some(value),
                    None if !feature.historically_accepted => {
                        match store.latest("feature", &feature.id, checkpoint, mechanic)? {
                            Some(record) => {
                                let identity = hashing::record_identity(&record)?;
                                Some((record, identity, None))
                            }
                            None => None,
                        }
                    }
                    None => None,
                };
                assess_record(
                    project,
                    &subject,
                    checkpoint,
                    mechanic,
                    evidence,
                    &mut status,
                    &mut basis,
                )?;
            }
        }
        rows.push(PlatformAssurance {
            area,
            status,
            subjects,
            basis,
        });
    }
    Ok(rows)
}

fn exact_bundle(project: &Project, feature: &FeatureStatus) -> Result<Option<ExactBundle>> {
    let Some(snapshot) = &feature.accepted_snapshot else {
        return Ok(None);
    };
    let accepted = snapshot.candidate_bundle_id.as_deref();
    let selected = if feature.current_acceptance_valid {
        match accepted {
            Some(id) if candidate::acceptance_current(project, id)? => Some(id.to_string()),
            _ => candidate::passed_revalidation_bundle(
                project,
                &feature.id,
                &snapshot.receipt_sha256,
            )?,
        }
    } else {
        accepted.map(str::to_string)
    };
    let Some(id) = selected else {
        return Ok(None);
    };
    let bundle = CandidateStore::at(&project.codeos_dir()).read(&id)?;
    Ok(Some(ExactBundle { id, bundle }))
}

#[allow(clippy::too_many_arguments)]
fn assess_record(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
    mechanic: &str,
    evidence: Option<(VerificationRecord, String, Option<String>)>,
    aggregate: &mut AssuranceStatus,
    basis: &mut Vec<String>,
) -> Result<()> {
    let Some((record, identity, bundle)) = evidence else {
        promote(aggregate, AssuranceStatus::Unobserved);
        basis.push(format!(
            "{} {checkpoint}/{mechanic}: no exact evidence",
            subject.id
        ));
        return Ok(());
    };
    let wanted =
        checker::verification_bindings_for_mechanic(project, subject, checkpoint, mechanic)?;
    let prefix = format!("{} {checkpoint}/{mechanic} {identity}", subject.id);
    let bundle = bundle
        .map(|id| format!(" via Candidate Bundle {id}"))
        .unwrap_or_default();
    match verification::evaluate(Some(record), &wanted) {
        RecordState::Current(_) => basis.push(format!("{prefix}{bundle}")),
        RecordState::Failed(_) => {
            promote(aggregate, AssuranceStatus::Blocked);
            basis.push(format!("{prefix}: failed{bundle}"));
        }
        RecordState::Stale { binding, .. } => {
            promote(aggregate, AssuranceStatus::Stale);
            basis.push(format!("{prefix}: stale ({binding} changed){bundle}"));
        }
        RecordState::Absent => unreachable!("a record was supplied"),
    }
    Ok(())
}

fn promote(status: &mut AssuranceStatus, candidate: AssuranceStatus) {
    if candidate.severity() > status.severity() {
        *status = candidate;
    }
}

pub fn render(rows: &[PlatformAssurance]) -> String {
    let mut out = String::from("PLATFORM ASSURANCE — DEVELOPMENT EVIDENCE\n\n");
    out.push_str("AREA                 STATUS          SUBJECTS                 BASIS\n");
    for row in rows {
        out.push_str(&format!(
            "{:<20} {:<15} {:<24} {}\n",
            row.area.label(),
            row.status.label(),
            row.subject_summary(),
            row.evidence_summary()
        ));
    }
    out.push_str("\nISSUES\n");
    for row in rows {
        let issues: Vec<_> = row.basis.iter().filter(|item| is_issue(item)).collect();
        for item in issues.iter().take(3) {
            out.push_str(&format!(
                "  {} — {}\n",
                row.area.label(),
                abbreviate_identities(item)
            ));
        }
        if issues.len() > 3 {
            out.push_str(&format!(
                "  {} — (+{} more issues; use status --json for full basis)\n",
                row.area.label(),
                issues.len() - 3
            ));
        }
    }
    if rows
        .iter()
        .all(|row| row.basis.iter().all(|item| !is_issue(item)))
    {
        out.push_str("  none\n");
    }
    out.trim_end().to_string()
}

fn abbreviated(subjects: &[String]) -> String {
    if subjects.is_empty() {
        return "—".into();
    }
    if subjects.len() > 2 {
        format!("{} subjects", subjects.len())
    } else {
        subjects.join(", ")
    }
}

fn is_issue(basis: &str) -> bool {
    [
        "missing or incomplete",
        "no exact",
        "no applicable verification",
        ": failed",
        ": stale",
    ]
    .iter()
    .any(|marker| basis.contains(marker))
}

fn abbreviate_identities(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let token = word.trim_matches(|c: char| !c.is_ascii_hexdigit());
            if token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit()) {
                word.replacen(token, &format!("{}…", &token[..8]), 1)
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_precedence_is_fixed() {
        let mut status = AssuranceStatus::NotApplicable;
        for candidate in [
            AssuranceStatus::Pass,
            AssuranceStatus::Unobserved,
            AssuranceStatus::Stale,
            AssuranceStatus::Blocked,
        ] {
            promote(&mut status, candidate);
            assert_eq!(status, candidate);
        }
        promote(&mut status, AssuranceStatus::Pass);
        assert_eq!(status, AssuranceStatus::Blocked);
    }

    #[test]
    fn terminal_output_abbreviates_identity_and_issue_volume() {
        let identity = "a".repeat(64);
        let rows = vec![PlatformAssurance {
            area: Area::Backend,
            status: AssuranceStatus::Stale,
            subjects: vec!["F-1".into(), "F-2".into(), "F-3".into()],
            basis: vec![
                format!("F-1 F3/smoke {identity}: stale (contract changed)"),
                "F-2 F3/smoke: no exact evidence".into(),
                "F-3: Feature Impact Accounting is missing or incomplete".into(),
                "F-4 F3/smoke: no exact evidence".into(),
            ],
        }];
        let output = render(&rows);
        assert!(output.contains("3 subjects"));
        assert!(output.contains("aaaaaaaa…"));
        assert!(output.contains("(+1 more issues"));
        assert!(!output.contains(&identity));
    }
}
