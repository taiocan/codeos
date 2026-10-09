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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArchitectureMetadata {
    artifact_type: String,
    features: Vec<String>,
    approval: ArchitectureApproval,
    reader_model: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArchitectureApproval {
    by: String,
    at: String,
}

#[derive(Serialize)]
struct FeatureArchitectureProjection<'a> {
    scope_path: &'a str,
    subject_membership: &'a str,
    body: &'a str,
}

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
    /// Base of the identified change: the commit before the first surface change since acceptance
    /// was last current. For a single-commit change it is the trigger's parent.
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

/// The later change a revalidation assesses.
#[derive(Debug)]
pub(crate) struct IdentifiedChange {
    pub base: String,
    pub affected_surface: Vec<String>,
}

fn normalized_architecture(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn architecture_parts(text: &str) -> Result<(ArchitectureMetadata, &str)> {
    let rest = text
        .strip_prefix("---\n")
        .context("Architecture Scope has no opening frontmatter delimiter")?;
    let (frontmatter, body) = rest
        .split_once("\n---\n")
        .context("Architecture Scope has no closing frontmatter delimiter")?;
    let metadata: ArchitectureMetadata =
        serde_yaml::from_str(frontmatter).context("parse Architecture Scope frontmatter")?;
    if metadata.artifact_type != "architecture" {
        anyhow::bail!("Architecture Scope artifact_type must be architecture");
    }
    if metadata.reader_model.trim().is_empty() {
        anyhow::bail!("Architecture Scope reader_model must be non-empty");
    }
    if metadata.features.is_empty()
        || metadata
            .features
            .iter()
            .any(|feature| feature.trim().is_empty())
    {
        anyhow::bail!("Architecture Scope features must be non-empty");
    }
    let unique = metadata
        .features
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
    if unique.len() != metadata.features.len() {
        anyhow::bail!("Architecture Scope features must be duplicate-free");
    }
    if metadata.approval.by.trim().is_empty() || metadata.approval.at.trim().is_empty() {
        anyhow::bail!("Architecture Scope approval must be complete");
    }
    Ok((metadata, body))
}

/// Resolve membership only from canonical scope frontmatter at `treeish`, then hash the complete
/// normalized body for the one scope that owns `subject`. Parsing every scope before resolving zero
/// matches prevents a malformed file from being mistaken for unrelated architecture.
fn feature_architecture_identity(
    project: &Project,
    subject: &str,
    treeish: &str,
) -> Result<String> {
    let listing = git(
        project,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            treeish,
            "--",
            ".codeos/02-architecture/scopes",
        ],
    )?;
    let paths = listing
        .lines()
        .filter(|path| path.ends_with(".md"))
        .collect::<Vec<_>>();
    let mut matches = Vec::new();
    for path in paths {
        let object = format!("{treeish}:{path}");
        let text = String::from_utf8(git_bytes(project, &["show", &object])?)
            .with_context(|| format!("Architecture Scope {path} is not UTF-8"))?;
        let normalized = normalized_architecture(&text);
        let (metadata, body) = architecture_parts(&normalized)
            .with_context(|| format!("invalid Architecture Scope {path} at {treeish}"))?;
        if metadata.features.iter().any(|feature| feature == subject) {
            matches.push((path, body.to_string()));
        }
    }
    match matches.as_slice() {
        [] => Ok("NO_APPLICABLE_SCOPE".into()),
        [(path, body)] => {
            let projection = FeatureArchitectureProjection {
                scope_path: path,
                subject_membership: subject,
                body,
            };
            Ok(hex::encode(Sha256::digest(serde_json::to_vec(
                &projection,
            )?)))
        }
        _ => anyhow::bail!("feature has ambiguous Architecture Scope membership at {treeish}"),
    }
}

/// Validate `trigger_commit` and derive its identified change: the trigger is the latest commit
/// changing the accepted surface, and the change runs from the point acceptance was last current.
pub(crate) fn identified_change(
    project: &Project,
    subject_id: &str,
    trigger_commit: &str,
    acceptance_timestamp: &str,
) -> Result<IdentifiedChange> {
    // Evidence binds to the implementation state, so the trigger is the latest commit changing the
    // accepted surface; later commits and the working tree must leave that surface unchanged.
    let implementation_paths = project.implementation_paths();
    let surface_args = |mut args: Vec<String>| {
        args.push("--".into());
        args.extend(implementation_paths.iter().cloned());
        args
    };
    let run = |args: Vec<String>| {
        git(
            project,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )
    };
    if !run(surface_args(vec![
        "log".into(),
        "--format=%H".into(),
        format!("{trigger_commit}..HEAD"),
    ]))?
    .is_empty()
        || !run(surface_args(vec!["status".into(), "--porcelain".into()]))?.is_empty()
    {
        anyhow::bail!("trigger must be the latest change to the accepted surface so evidence binds exactly to it");
    }
    if run(surface_args(vec![
        "diff".into(),
        "--name-only".into(),
        format!("{trigger_commit}^"),
        trigger_commit.to_string(),
    ]))?
    .is_empty()
    {
        anyhow::bail!("trigger does not change the accepted implementation/dependency surface");
    }
    // Commit times carry their local offset and receipts are UTC, so compare instants.
    let instant = |value: &str| {
        chrono::DateTime::parse_from_rfc3339(value.trim())
            .with_context(|| format!("unreadable timestamp {value}"))
    };
    let commit_time = git(project, &["show", "-s", "--format=%cI", trigger_commit])?;
    if instant(&commit_time)? <= instant(acceptance_timestamp)? {
        anyhow::bail!("trigger is not a mechanically later change after F9 acceptance");
    }
    // The identified change runs from the point acceptance was last current (the latest
    // REVALIDATED record, otherwise F9) through the trigger; `trigger_parent` is its base.
    let latest_revalidation = RevalidationStore::at(&project.codeos_dir())
        .all()?
        .into_iter()
        .filter(|r| {
            r.subject == subject_id
                && r.decision == "REVALIDATED"
                && r.timestamp.as_str() > acceptance_timestamp
        })
        .max_by(|left, right| left.timestamp.cmp(&right.timestamp));
    let (last_current, architecture_baseline) = if let Some(record) = latest_revalidation {
        (record.timestamp, record.trigger_commit)
    } else {
        let baseline = git(
            project,
            &[
                "rev-list",
                "-1",
                &format!("--before={acceptance_timestamp}"),
                "HEAD",
            ],
        )?;
        if baseline.is_empty() {
            anyhow::bail!("cannot resolve architecture baseline at F9 acceptance");
        }
        (acceptance_timestamp.to_string(), baseline)
    };
    let surface_commits = run(surface_args(vec![
        "log".into(),
        "--format=%H %cI".into(),
        "--reverse".into(),
        trigger_commit.to_string(),
    ]))?;
    let last_current = instant(&last_current)?;
    let mut first_change = None;
    for (sha, time) in surface_commits
        .lines()
        .filter_map(|line| line.split_once(' '))
    {
        if instant(time)? > last_current {
            first_change = Some(sha.to_string());
            break;
        }
    }
    let first_change = first_change
        .context("no change to the accepted surface since acceptance was last current")?;
    let trigger_parent = git(project, &["rev-parse", &format!("{first_change}^")])?;
    // Workflow v5 introduced post-acceptance revalidation in terms of unchanged applicable
    // architecture meaning. Resolve that meaning per feature instead of treating any edit to the
    // shared architecture directory as a semantic change.
    if project.feature_workflow_version()? >= 5 {
        let accepted_architecture =
            feature_architecture_identity(project, subject_id, &architecture_baseline)?;
        let current_architecture = feature_architecture_identity(project, subject_id, "HEAD")?;
        if accepted_architecture != current_architecture {
            anyhow::bail!(
                "feature-scoped architecture identity changed; normal governed recovery is required"
            );
        }
    } else {
        let architecture_diff = git(
            project,
            &[
                "diff",
                "--name-only",
                &trigger_parent,
                trigger_commit,
                "--",
                ".codeos/02-architecture",
            ],
        )?;
        if !architecture_diff.trim().is_empty() {
            anyhow::bail!("architecture meaning changed; normal governed recovery is required");
        }
    }
    let changed = run(surface_args(vec![
        "diff".into(),
        "--name-only".into(),
        trigger_parent.clone(),
        trigger_commit.to_string(),
    ]))?;
    Ok(IdentifiedChange {
        base: trigger_parent,
        affected_surface: changed.lines().map(str::to_string).collect(),
    })
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
    let IdentifiedChange {
        base: trigger_parent,
        affected_surface,
    } = identified_change(project, subject_id, &trigger_commit, &acceptance.timestamp)?;
    let implementation_paths = project.implementation_paths();
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
    let diff = surface_diff(project, &trigger_parent, &trigger_commit, true)?;
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
    Ok(format!("# POST_ACCEPTANCE_REVALIDATION\n\nDecide exactly `REVALIDATED` or `REVALIDATION FAILED`. Answer only whether the previously accepted feature still satisfies its unchanged approved meaning after the identified later change. Do not reopen historical findings or propose a new semantic revision.\n\n## Mechanical case\n```json\n{}\n```\n\n## Unchanged Intent\n{}\n\n## Unchanged Contract\n{}\n\n## Unchanged Event Schema\n{}\n\n## Trigger diff\n```diff\n{}\n```\n", serde_json::to_string_pretty(case)?, context(project.intent(&case.subject))?, context(project.contract(&case.subject))?, context(project.event_schema(&case.subject))?, String::from_utf8(surface_diff(project, &case.trigger_parent, &case.trigger_commit, false)?)?))
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
    if project.workflow_governance_version()? >= 3 {
        if let Some(bundle) = acceptance.bindings.get("candidate_bundle") {
            return crate::candidate::acceptance_current(project, bundle);
        }
    }
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
/// The identified change: the diff from its base through the trigger, limited to the accepted
/// implementation surface.
fn surface_diff(project: &Project, base: &str, trigger: &str, binary: bool) -> Result<Vec<u8>> {
    let mut args = vec!["diff"];
    if binary {
        args.push("--binary");
    }
    args.extend([base, trigger, "--"]);
    let paths = project.implementation_paths();
    args.extend(paths.iter().map(String::as_str));
    git_bytes(project, &args)
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

    fn approved_scope(features: &str, approval: &str, body: &str) -> String {
        format!(
            "---\nartifact_type: architecture\nfeatures: [{features}]\napproval:\n  by: Human\n  at: {approval}\nreader_model: whole-before-parts\n---\n{body}"
        )
    }

    #[test]
    fn architecture_identity_ignores_only_sibling_membership_and_approval() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let git = |args: &[&str]| {
            let out = Command::new("git")
                .args(args)
                .current_dir(root)
                .env("GIT_AUTHOR_DATE", "2026-01-01T12:00:00+01:00")
                .env("GIT_COMMITTER_DATE", "2026-01-01T12:00:00+01:00")
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        git(&["init", "-q"]);
        git(&["config", "user.email", "t@t"]);
        git(&["config", "user.name", "t"]);
        let scope = root.join(".codeos/02-architecture/scopes/shared.md");
        std::fs::create_dir_all(scope.parent().unwrap()).unwrap();
        let body = "\n# Shared scope\n\nThe adapter owns the boundary.\n";
        std::fs::write(&scope, approved_scope("F-0001, F-0002", "2026-01-01", body)).unwrap();
        git(&["add", "."]);
        git(&["commit", "-qm", "base"]);
        let base = git(&["rev-parse", "HEAD"]);
        let project = Project::discover(root).unwrap();
        let f1_base = feature_architecture_identity(&project, "F-0001", &base).unwrap();
        let f2_base = feature_architecture_identity(&project, "F-0002", &base).unwrap();
        let f3_base = feature_architecture_identity(&project, "F-0003", &base).unwrap();

        std::fs::write(
            &scope,
            approved_scope("F-0001, F-0002, F-0004", "2026-01-02", body),
        )
        .unwrap();
        git(&["add", "."]);
        git(&["commit", "-qm", "add sibling"]);
        let sibling = git(&["rev-parse", "HEAD"]);
        assert_eq!(
            feature_architecture_identity(&project, "F-0001", &sibling).unwrap(),
            f1_base
        );
        assert_eq!(
            feature_architecture_identity(&project, "F-0002", &sibling).unwrap(),
            f2_base
        );
        assert_eq!(
            feature_architecture_identity(&project, "F-0003", &sibling).unwrap(),
            f3_base
        );

        let changed_body = "\n# Shared scope\n\nThe adapter owns the boundary and every member must persist its handoff.\n";
        std::fs::write(
            &scope,
            approved_scope("F-0001, F-0002, F-0004", "2026-01-03", changed_body),
        )
        .unwrap();
        git(&["add", "."]);
        git(&["commit", "-qm", "change obligation"]);
        let obligation = git(&["rev-parse", "HEAD"]);
        assert_ne!(
            feature_architecture_identity(&project, "F-0001", &obligation).unwrap(),
            f1_base
        );
        assert_eq!(
            feature_architecture_identity(&project, "F-0003", &obligation).unwrap(),
            f3_base
        );
    }

    #[test]
    fn architecture_identity_fails_closed_on_unresolved_membership() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let git = |args: &[&str]| {
            let out = Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        git(&["init", "-q"]);
        git(&["config", "user.email", "t@t"]);
        git(&["config", "user.name", "t"]);
        let scopes = root.join(".codeos/02-architecture/scopes");
        std::fs::create_dir_all(&scopes).unwrap();
        std::fs::write(
            scopes.join("one.md"),
            approved_scope("F-0001", "2026-01-01", "\n# One\n"),
        )
        .unwrap();
        std::fs::write(
            scopes.join("two.md"),
            approved_scope("F-0001", "2026-01-01", "\n# Two\n"),
        )
        .unwrap();
        git(&["add", "."]);
        git(&["commit", "-qm", "ambiguous"]);
        let ambiguous = git(&["rev-parse", "HEAD"]);
        let project = Project::discover(root).unwrap();
        assert!(
            feature_architecture_identity(&project, "F-0001", &ambiguous)
                .unwrap_err()
                .to_string()
                .contains("ambiguous")
        );

        std::fs::write(scopes.join("two.md"), "---\nfeatures: [F-0002]\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-qm", "malformed"]);
        let malformed = git(&["rev-parse", "HEAD"]);
        assert!(feature_architecture_identity(&project, "F-0003", &malformed).is_err());
    }

    /// A repository with commits at fixed instants: base (before F9 at 2026-01-01T00:00Z), a product
    /// change, a test-only change, and a governance-only commit.
    fn change_history() -> (tempfile::TempDir, Project, Vec<String>) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let git = |args: &[&str], date: &str| {
            let out = Command::new("git")
                .args(args)
                .current_dir(root)
                .env("GIT_AUTHOR_DATE", date)
                .env("GIT_COMMITTER_DATE", date)
                .output()
                .unwrap();
            assert!(out.status.success(), "git {args:?}");
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        git(&["init", "-q"], "2025-12-31T00:00:00+01:00");
        git(
            &["config", "user.email", "t@t"],
            "2025-12-31T00:00:00+01:00",
        );
        git(&["config", "user.name", "t"], "2025-12-31T00:00:00+01:00");
        let mut commits = Vec::new();
        for (path, content, date) in [
            ("src/lib.rs", "base", "2025-12-31T12:00:00+01:00"),
            ("src/lib.rs", "product change", "2026-01-02T12:00:00+01:00"),
            (
                "src/lib.spec.ts",
                "test change",
                "2026-01-03T12:00:00+01:00",
            ),
            (
                ".codeos/06-workflow/decisions.jsonl",
                "receipt",
                "2026-01-04T12:00:00+01:00",
            ),
        ] {
            let full = root.join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, content).unwrap();
            git(&["add", "."], date);
            git(&["commit", "-qm", path], date);
            commits.push(git(&["rev-parse", "HEAD"], date));
        }
        let project = Project::discover(root).unwrap();
        (dir, project, commits)
    }

    #[test]
    fn identified_change_spans_every_surface_commit_since_acceptance_was_current() {
        let (_dir, project, c) = change_history();
        let change = identified_change(&project, "F-0001", &c[2], "2026-01-01T00:00:00Z").unwrap();
        assert_eq!(change.base, c[0]);
        assert_eq!(
            change.affected_surface,
            vec!["src/lib.rs", "src/lib.spec.ts"]
        );

        let earlier = identified_change(&project, "F-0001", &c[1], "2026-01-01T00:00:00Z");
        assert!(earlier.unwrap_err().to_string().contains("latest change"));
        let governance = identified_change(&project, "F-0001", &c[3], "2026-01-01T00:00:00Z");
        assert!(governance
            .unwrap_err()
            .to_string()
            .contains("does not change"));
    }

    #[test]
    fn identified_change_starts_after_the_latest_revalidation() {
        let (_dir, project, c) = change_history();
        let mut revalidated = record(&c[1]);
        revalidated.timestamp = "2026-01-02T18:00:00Z".into();
        RevalidationStore::at(&project.codeos_dir())
            .append(&revalidated)
            .unwrap();
        let change = identified_change(&project, "F-0001", &c[2], "2026-01-01T00:00:00Z").unwrap();
        assert_eq!(change.base, c[1]);
        assert_eq!(change.affected_surface, vec!["src/lib.spec.ts"]);
    }

    #[test]
    fn identified_change_refuses_uncommitted_surface_drift() {
        let (dir, project, c) = change_history();
        std::fs::write(dir.path().join("src/lib.rs"), "uncommitted").unwrap();
        let drift = identified_change(&project, "F-0001", &c[2], "2026-01-01T00:00:00Z");
        assert!(drift.unwrap_err().to_string().contains("latest change"));
    }

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
