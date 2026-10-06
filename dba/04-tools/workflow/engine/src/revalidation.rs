//! Post-acceptance dependency revalidation for an accepted feature whose governed meaning is
//! unchanged but whose implementation evidence was invalidated by a later shared change.

use crate::checker::Subject;
use crate::hashing;
use crate::project::Project;
use crate::receipts::{Receipt, ReceiptStore};
use crate::verification::{self, VerificationRecord, VerificationStore};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const REQUIRED_BASE: &[(&str, &str)] = &[
    ("F3", "smoke"),
    ("F3", "product_journey"),
    ("F5", "behavior"),
    ("F5", "repeatability"),
    ("F7", "runtime_event_schema"),
];

/// The (checkpoint, verification) records a feature's acceptance evidence requires.
pub fn required_verifications(subject: &Subject) -> Vec<(&'static str, &'static str)> {
    let mut required = REQUIRED_BASE.to_vec();
    if subject.gui {
        required.push(("F5g", "playwright"));
    }
    if !subject.reality_claims.is_empty() {
        required.push(("F5", "production_source_evidence"));
    }
    required
}

#[derive(Debug, Clone, Serialize)]
pub struct RevalidationCase {
    pub subject: String,
    pub semantic_revision: u64,
    pub original_acceptance_sha256: String,
    pub original_acceptance_timestamp: String,
    pub original_implementation_state: String,
    pub trigger_commit: String,
    pub trigger_parent: String,
    pub trigger_diff_sha256: String,
    pub affected_surface: Vec<String>,
    pub current_implementation_state: String,
    pub governed_inputs: BTreeMap<String, String>,
    pub current_verifications: BTreeMap<String, String>,
    pub production_evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IndependentAssessment {
    pub decision: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RevalidationRecord {
    pub operation: String,
    pub subject: String,
    pub semantic_revision: u64,
    pub original_acceptance_sha256: String,
    pub original_acceptance_timestamp: String,
    pub original_implementation_state: String,
    pub trigger_commit: String,
    pub trigger_parent: String,
    pub trigger_diff_sha256: String,
    pub affected_surface: Vec<String>,
    pub current_implementation_state: String,
    pub governed_inputs: BTreeMap<String, String>,
    pub current_verifications: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub production_evidence: Option<String>,
    pub assessment_sha256: String,
    pub decision: String,
    pub timestamp: String,
}

pub struct RevalidationStore {
    path: PathBuf,
}

impl RevalidationStore {
    pub fn at(codeos_dir: &Path) -> Self {
        Self {
            path: codeos_dir.join("06-workflow/revalidations.jsonl"),
        }
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn all(&self) -> Result<Vec<RevalidationRecord>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        std::fs::read_to_string(&self.path)?
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
            .map(|(i, l)| {
                serde_json::from_str(l).with_context(|| {
                    format!(
                        "{}:{}: malformed revalidation record",
                        self.path.display(),
                        i + 1
                    )
                })
            })
            .collect()
    }
    pub fn append(&self, record: &RevalidationRecord) -> Result<()> {
        if self
            .all()?
            .iter()
            .any(|r| r.subject == record.subject && r.trigger_commit == record.trigger_commit)
        {
            anyhow::bail!("this subject and trigger already have a bounded revalidation lineage");
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{}", serde_json::to_string(record)?)?;
        Ok(())
    }
}

pub fn build_case(project: &Project, subject_id: &str, trigger: &str) -> Result<RevalidationCase> {
    if project.feature_workflow_version()? < 5 {
        anyhow::bail!("POST_ACCEPTANCE_REVALIDATION requires Feature Workflow v5");
    }
    let subject = Subject::resolve(project, crate::contract::Workflow::Feature, subject_id)?;
    let acceptance = latest_acceptance(project, subject_id)?;
    let acceptance_bytes = serde_json::to_vec(&acceptance)?;
    let original_acceptance_sha256 = hex::encode(Sha256::digest(&acceptance_bytes));
    let governed_inputs = governed(project, subject_id)?;
    for key in ["intent", "contract", "event_schema"] {
        if acceptance.bindings.get(key) != governed_inputs.get(key) {
            anyhow::bail!("governed meaning changed at {key}; use a normal semantic revision");
        }
    }
    let trigger_commit = git(
        project,
        &["rev-parse", "--verify", &format!("{trigger}^{{commit}}")],
    )?;
    let head = git(project, &["rev-parse", "HEAD"])?;
    if trigger_commit != head {
        anyhow::bail!("trigger must be current HEAD so evidence can bind exactly to it");
    }
    let trigger_parent = git(project, &["rev-parse", &format!("{trigger_commit}^")])?;
    let commit_time = git(project, &["show", "-s", "--format=%cI", &trigger_commit])?;
    if commit_time.as_str() <= acceptance.timestamp.as_str() {
        anyhow::bail!("trigger is not a mechanically later change after F9 acceptance");
    }
    let architecture_diff = git(
        project,
        &[
            "diff",
            "--name-only",
            &trigger_parent,
            &trigger_commit,
            "--",
            ".codeos/02-architecture",
        ],
    )?;
    if !architecture_diff.trim().is_empty() {
        anyhow::bail!("architecture meaning changed; normal governed recovery is required");
    }
    let changed = git(
        project,
        &[
            "diff",
            "--name-only",
            &trigger_parent,
            &trigger_commit,
            "--",
        ],
    )?;
    let implementation_paths = project.implementation_paths();
    let affected_surface: Vec<_> = changed
        .lines()
        .filter(|path| {
            implementation_paths
                .iter()
                .any(|root| path == root || path.starts_with(&format!("{root}/")))
        })
        .map(str::to_string)
        .collect();
    if affected_surface.is_empty() {
        anyhow::bail!("trigger does not intersect the accepted implementation/dependency surface");
    }
    let original = latest_impl_before(project, subject_id, &acceptance.timestamp)?;
    let current = hashing::working_tree_state(&project.root, &implementation_paths)?;
    if original == current {
        anyhow::bail!("acceptance implementation evidence is not stale");
    }
    let vstore = VerificationStore::at(&project.codeos_dir());
    let required = required_verifications(&subject);
    let mut current_verifications = BTreeMap::new();
    let mut production_evidence = None;
    for (checkpoint, name) in required {
        let record = vstore
            .latest("feature", subject_id, checkpoint, name)?
            .with_context(|| format!("missing fresh {name} verification"))?;
        let current_record = if name == "runtime_event_schema" {
            record.bindings.get("implementation_state") == Some(&current)
                && ["intent", "contract", "event_schema"]
                    .iter()
                    .all(|key| record.bindings.get(*key) == governed_inputs.get(*key))
                && record.timestamp.as_str() > acceptance.timestamp.as_str()
        } else {
            let want = crate::checker::verification_bindings_for_mechanic(
                project, &subject, checkpoint, name,
            )?;
            matches!(
                verification::evaluate(Some(record.clone()), &want),
                verification::RecordState::Current(_)
            )
        };
        if !current_record {
            anyhow::bail!("{name} verification is not current for the revalidation implementation");
        }
        current_verifications.insert(name.into(), record_identity(&record)?);
        if name == "production_source_evidence" {
            production_evidence = record
                .details
                .as_ref()
                .and_then(|d| d.get("evidence_sha256"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
        }
    }
    let semantic_revision =
        semantic_revision(&std::fs::read_to_string(project.intent(subject_id))?)?;
    let diff = git_bytes(
        project,
        &["diff", "--binary", &trigger_parent, &trigger_commit, "--"],
    )?;
    Ok(RevalidationCase {
        subject: subject_id.into(),
        semantic_revision,
        original_acceptance_sha256,
        original_acceptance_timestamp: acceptance.timestamp,
        original_implementation_state: original,
        trigger_commit,
        trigger_parent,
        trigger_diff_sha256: hex::encode(Sha256::digest(&diff)),
        affected_surface,
        current_implementation_state: current,
        governed_inputs,
        current_verifications,
        production_evidence,
    })
}

pub fn packet(project: &Project, case: &RevalidationCase) -> Result<String> {
    let context = |path: PathBuf| -> Result<String> { Ok(std::fs::read_to_string(path)?) };
    Ok(format!("# POST_ACCEPTANCE_REVALIDATION\n\nDecide exactly `REVALIDATED` or `REVALIDATION FAILED`. Answer only whether the previously accepted feature still satisfies its unchanged approved meaning after the identified later change. Do not reopen historical findings or propose a new semantic revision.\n\n## Mechanical case\n```json\n{}\n```\n\n## Unchanged Intent\n{}\n\n## Unchanged Contract\n{}\n\n## Unchanged Event Schema\n{}\n\n## Trigger diff\n```diff\n{}\n```\n", serde_json::to_string_pretty(case)?, context(project.intent(&case.subject))?, context(project.contract(&case.subject))?, context(project.event_schema(&case.subject))?, git(project, &["diff", &case.trigger_parent, &case.trigger_commit, "--"])?))
}

pub fn record(
    project: &Project,
    case: RevalidationCase,
    assessment_path: &Path,
) -> Result<RevalidationRecord> {
    let bytes = std::fs::read(assessment_path)?;
    let assessment: IndependentAssessment = serde_json::from_slice(&bytes)
        .context("assessment must be JSON with decision and summary only")?;
    if !matches!(
        assessment.decision.as_str(),
        "REVALIDATED" | "REVALIDATION FAILED"
    ) {
        anyhow::bail!("assessment decision must be REVALIDATED or REVALIDATION FAILED");
    }
    if assessment.summary.trim().is_empty() {
        anyhow::bail!("assessment summary must be non-empty");
    }
    let record = RevalidationRecord {
        operation: "POST_ACCEPTANCE_REVALIDATION".into(),
        subject: case.subject,
        semantic_revision: case.semantic_revision,
        original_acceptance_sha256: case.original_acceptance_sha256,
        original_acceptance_timestamp: case.original_acceptance_timestamp,
        original_implementation_state: case.original_implementation_state,
        trigger_commit: case.trigger_commit,
        trigger_parent: case.trigger_parent,
        trigger_diff_sha256: case.trigger_diff_sha256,
        affected_surface: case.affected_surface,
        current_implementation_state: case.current_implementation_state,
        governed_inputs: case.governed_inputs,
        current_verifications: case.current_verifications,
        production_evidence: case.production_evidence,
        assessment_sha256: hex::encode(Sha256::digest(&bytes)),
        decision: assessment.decision,
        timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    };
    RevalidationStore::at(&project.codeos_dir()).append(&record)?;
    Ok(record)
}

pub fn current_acceptance_valid(project: &Project, subject: &str) -> Result<bool> {
    if project.feature_workflow_version()? < 5 {
        return Ok(true);
    }
    let acceptance = match latest_acceptance(project, subject) {
        Ok(value) => value,
        Err(_) => return Ok(false),
    };
    let original = latest_impl_before(project, subject, &acceptance.timestamp)?;
    let current = hashing::working_tree_state(&project.root, &project.implementation_paths())?;
    if original == current {
        return Ok(true);
    }
    let governed_now = governed(project, subject)?;
    if ["intent", "contract", "event_schema"]
        .iter()
        .any(|key| acceptance.bindings.get(*key) != governed_now.get(*key))
    {
        return Ok(false);
    }
    Ok(RevalidationStore::at(&project.codeos_dir())
        .all()?
        .into_iter()
        .rev()
        .find(|r| {
            r.subject == subject
                && r.current_implementation_state == current
                && r.governed_inputs == governed_now
        })
        .is_some_and(|r| r.decision == "REVALIDATED"))
}

pub fn historically_accepted(project: &Project, subject: &str) -> Result<bool> {
    Ok(latest_acceptance(project, subject).is_ok())
}

fn latest_acceptance(project: &Project, subject: &str) -> Result<Receipt> {
    ReceiptStore::at(&project.codeos_dir())
        .all()?
        .into_iter()
        .rfind(|r| {
            r.workflow == "feature"
                && r.subject == subject
                && r.checkpoint == "acceptance"
                && r.result == "accepted"
        })
        .context("feature never reached F9 acceptance")
}
fn governed(project: &Project, subject: &str) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for (key, path) in [
        ("intent", project.intent(subject)),
        ("contract", project.contract(subject)),
        ("event_schema", project.event_schema(subject)),
    ] {
        out.insert(
            key.into(),
            hashing::file_sha256(&path)?.with_context(|| format!("missing {key}"))?,
        );
    }
    Ok(out)
}
fn latest_impl_before(project: &Project, subject: &str, before: &str) -> Result<String> {
    VerificationStore::at(&project.codeos_dir())
        .all()?
        .into_iter()
        .filter(|r| r.subject == subject && r.result == "passed" && r.timestamp.as_str() <= before)
        .rev()
        .find_map(|r| r.bindings.get("implementation_state").cloned())
        .context("accepted feature has no implementation-bound verification")
}
fn record_identity(record: &VerificationRecord) -> Result<String> {
    Ok(hex::encode(Sha256::digest(serde_json::to_vec(record)?)))
}
fn semantic_revision(text: &str) -> Result<u64> {
    text.lines()
        .find_map(|l| l.trim().strip_prefix("semantic_revision:"))
        .map(str::trim)
        .context("Intent has no semantic_revision")?
        .parse()
        .context("semantic_revision is not a positive integer")
}
fn git(project: &Project, args: &[&str]) -> Result<String> {
    Ok(String::from_utf8(git_bytes(project, args)?)?
        .trim()
        .to_string())
}
fn git_bytes(project: &Project, args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(&project.root)
        .args(args)
        .output()?;
    if !out.status.success() {
        anyhow::bail!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(out.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(trigger: &str) -> RevalidationRecord {
        RevalidationRecord {
            operation: "POST_ACCEPTANCE_REVALIDATION".into(),
            subject: "F-0001".into(),
            semantic_revision: 3,
            original_acceptance_sha256: "a".repeat(64),
            original_acceptance_timestamp: "2026-01-01T00:00:00Z".into(),
            original_implementation_state: "b".repeat(64),
            trigger_commit: trigger.into(),
            trigger_parent: "c".repeat(40),
            trigger_diff_sha256: "d".repeat(64),
            affected_surface: vec!["backend/src/main.rs".into()],
            current_implementation_state: "e".repeat(64),
            governed_inputs: BTreeMap::new(),
            current_verifications: BTreeMap::new(),
            production_evidence: None,
            assessment_sha256: "f".repeat(64),
            decision: "REVALIDATED".into(),
            timestamp: "2026-01-02T00:00:00Z".into(),
        }
    }

    #[test]
    fn one_trigger_creates_only_one_lineage() {
        let dir = tempfile::tempdir().unwrap();
        let store = RevalidationStore::at(dir.path());
        store.append(&record("trigger-a")).unwrap();
        assert!(store.append(&record("trigger-a")).is_err());
        store.append(&record("trigger-b")).unwrap();
        assert_eq!(store.all().unwrap().len(), 2);
    }

    #[test]
    fn assessment_has_a_closed_decision_set() {
        let good: IndependentAssessment = serde_json::from_str(
            r#"{"decision":"REVALIDATED","summary":"unchanged meaning remains satisfied"}"#,
        )
        .unwrap();
        assert_eq!(good.decision, "REVALIDATED");
        assert!(serde_json::from_str::<IndependentAssessment>(
            r#"{"decision":"R4","summary":"try again","extra":true}"#
        )
        .is_err());
    }
}
