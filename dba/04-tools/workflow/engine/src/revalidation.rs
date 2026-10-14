//! Post-acceptance dependency revalidation for an accepted feature whose governed meaning is
//! unchanged but whose implementation evidence was invalidated by a later shared change.

use crate::checker::Subject;
use crate::hashing;
use crate::project::Project;
use crate::receipts::{Receipt, ReceiptStore};
use crate::verification::{self, VerificationStore};
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
];

/// The (checkpoint, verification) records a feature's acceptance evidence requires. Runtime event
/// evidence applies only in the `events` Observation Mode, as for Candidate Bundles.
pub fn required_verifications(
    project: &Project,
    subject: &Subject,
) -> Result<Vec<(&'static str, &'static str)>> {
    let mut required = REQUIRED_BASE.to_vec();
    if crate::candidate::observation_mode(project, &subject.id)? == "events" {
        required.push(("F7", "runtime_event_schema"));
    }
    if subject.gui {
        required.push(("F5g", "playwright"));
    }
    if !subject.reality_claims.is_empty() {
        required.push(("F5", "production_source_evidence"));
    }
    Ok(required)
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

/// Whether the working tree changes the accepted implementation/dependency surface.
pub(crate) fn surface_uncommitted(project: &Project) -> Result<bool> {
    let mut args = vec!["status", "--porcelain", "--"];
    let paths = project.implementation_paths();
    args.extend(paths.iter().map(String::as_str));
    Ok(!git(project, &args)?.is_empty())
}

/// The only commit `identified_change` accepts as a trigger: the latest one changing the surface.
fn latest_surface_commit(project: &Project) -> Result<Option<String>> {
    let mut args = vec!["log", "-1", "--format=%H", "--"];
    let paths = project.implementation_paths();
    args.extend(paths.iter().map(String::as_str));
    Ok(Some(git(project, &args)?).filter(|sha| !sha.is_empty()))
}

/// The next atomic operation of Governance v4 post-acceptance revalidation for an accepted
/// feature stale by `REVALIDATION_REQUIRED`. Each step is derived from the same check the
/// command performing it applies, so `status` never advertises a command that would refuse.
#[derive(Debug, PartialEq, Eq)]
pub enum RevalidationStep {
    /// The surface has uncommitted changes; no trigger can be identified.
    CommitChange,
    /// `check --revalidation --trigger` would execute stale verifications.
    RunVerification { trigger: String },
    /// `prepare-candidate --revalidation --trigger` would succeed.
    PrepareCandidate { trigger: String },
    /// A current revalidation Candidate Bundle awaits its Stage 8 assessment.
    RunReview { bundle: String },
    /// The current revalidation bundle's assessment did not pass.
    ResolveReviewFinding { bundle: String },
    /// The revalidation commands would refuse, for this reason.
    Ineligible(String),
}

pub fn next_step(project: &Project, subject: &Subject) -> RevalidationStep {
    // Any refusal on the way is the refusal the revalidation commands would report.
    derive_step(project, subject)
        .unwrap_or_else(|error| RevalidationStep::Ineligible(format!("{error:#}")))
}

fn derive_step(project: &Project, subject: &Subject) -> Result<RevalidationStep> {
    use RevalidationStep::*;
    if surface_uncommitted(project)? {
        return Ok(CommitChange);
    }
    let Some(trigger) = latest_surface_commit(project)? else {
        return Ok(Ineligible("no commit changes the accepted surface".into()));
    };
    if let Err(error) = preflight(project, &subject.id, &trigger) {
        return Ok(Ineligible(format!("{error:#}")));
    }
    let acceptance = hashing::record_identity(&latest_acceptance(project, &subject.id)?)?;
    for (id, bundle) in crate::candidate::CandidateStore::at(&project.codeos_dir()).all()? {
        if bundle.feature != subject.id
            || bundle.kind != "post_acceptance_revalidation"
            || bundle.baseline_acceptance.as_deref() != Some(acceptance.as_str())
            || bundle.trigger_commit.as_deref() != Some(trigger.as_str())
            || !crate::candidate::current_for_review(project, &id)?
        {
            continue;
        }
        let review = crate::evidence::latest_review_record(&project.root, &subject.id, "8")?;
        let assessed = review.is_some_and(|review| {
            review.candidate_bundle.as_deref() == Some(id.as_str())
                && review.candidate_outcome.is_some()
        });
        return Ok(if assessed {
            ResolveReviewFinding { bundle: id }
        } else {
            RunReview { bundle: id }
        });
    }
    if !crate::checker::stale_revalidation_verifications(project, subject)?.is_empty() {
        return Ok(RunVerification { trigger });
    }
    Ok(
        match crate::candidate::assemble(
            project,
            subject,
            &[],
            "post_acceptance_revalidation",
            Some(&trigger),
        ) {
            Ok(_) => PrepareCandidate { trigger },
            Err(error) => Ineligible(format!("{error:#}")),
        },
    )
}

/// The later change a revalidation assesses.
#[derive(Debug)]
pub(crate) struct IdentifiedChange {
    pub base: String,
    pub affected_surface: Vec<String>,
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
        || surface_uncommitted(project)?
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
            crate::architecture::feature_identity(project, subject_id, &architecture_baseline)?;
        let current_architecture =
            crate::architecture::feature_identity(project, subject_id, "WORKTREE")?;
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
    let required = required_verifications(project, &subject)?;
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
        current_verifications.insert(name.into(), hashing::record_identity(&record)?);
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

/// Validate revalidation eligibility without requiring or writing fresh evidence.
pub fn preflight(project: &Project, subject_id: &str, trigger: &str) -> Result<()> {
    anyhow::ensure!(
        project.feature_workflow_version()? >= 5,
        "POST_ACCEPTANCE_REVALIDATION requires Feature Workflow v5"
    );
    let acceptance = latest_acceptance(project, subject_id)?;
    let governed_inputs = governed(project, subject_id)?;
    for key in ["intent", "contract", "event_schema"] {
        if acceptance.bindings.get(key) != governed_inputs.get(key) {
            anyhow::bail!("governed meaning changed at {key}; use ordinary governed work");
        }
    }
    let trigger_commit = git(
        project,
        &["rev-parse", "--verify", &format!("{trigger}^{{commit}}")],
    )?;
    identified_change(project, subject_id, &trigger_commit, &acceptance.timestamp)?;
    Ok(())
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

/// Whether an accepted snapshot still applies, why not, and the exact comparisons that decided it.
/// Status explanations read this; they never reconstruct the cause independently.
#[derive(Debug, Default)]
pub struct AcceptanceAssessment {
    pub current: bool,
    pub reason: Option<&'static str>,
    pub comparisons: Vec<hashing::Comparison>,
    pub evidence: Vec<hashing::EvidenceRef>,
}

pub fn current_acceptance_valid(project: &Project, subject: &str) -> Result<bool> {
    Ok(acceptance_assessment(project, subject)?.current)
}

pub fn acceptance_assessment(project: &Project, subject: &str) -> Result<AcceptanceAssessment> {
    let mut assessment = AcceptanceAssessment::default();
    if project.feature_workflow_version()? < 5 {
        assessment.current = true;
        return Ok(assessment);
    }
    let acceptance = match latest_acceptance(project, subject) {
        Ok(value) => value,
        Err(_) => return Ok(assessment),
    };
    let acceptance_identity = hashing::record_identity(&acceptance)?;
    assessment.evidence.push(hashing::EvidenceRef {
        kind: hashing::EvidenceKind::AcceptanceReceipt,
        id: acceptance_identity.clone(),
    });
    let stale = |mut assessment: AcceptanceAssessment, reason| {
        assessment.reason = Some(reason);
        Ok(assessment)
    };
    let governed_now = governed(project, subject)?;
    for key in ["intent", "contract", "event_schema"] {
        assessment.comparisons.push(hashing::Comparison {
            surface: key.into(),
            was: acceptance.bindings.get(key).cloned(),
            now: governed_now.get(key).cloned(),
        });
    }
    if assessment
        .comparisons
        .iter()
        .any(hashing::Comparison::changed)
    {
        return stale(assessment, "SEMANTIC_CHANGE_REQUIRED");
    }
    if project.feature_workflow_version()? >= 8 {
        let comparison = hashing::Comparison {
            surface: "architecture".into(),
            was: Some(accepted_architecture_identity(
                project,
                subject,
                &acceptance.timestamp,
            )?),
            now: Some(crate::architecture::feature_identity(
                project, subject, "WORKTREE",
            )?),
        };
        let changed = comparison.changed();
        assessment.comparisons.push(comparison);
        if changed {
            return stale(assessment, "ARCHITECTURE_CHANGE_REQUIRED");
        }
    }
    if project.workflow_governance_version()? >= 3 {
        if let Some(bundle) = acceptance.bindings.get("candidate_bundle") {
            assessment.evidence.push(hashing::EvidenceRef {
                kind: hashing::EvidenceKind::CandidateBundle,
                id: bundle.clone(),
            });
            let comparisons = crate::candidate::acceptance_comparisons(
                project,
                bundle,
                project.feature_workflow_version()? >= 8,
            )?;
            let candidate_current = !comparisons.iter().any(hashing::Comparison::changed);
            assessment.comparisons.extend(comparisons);
            if candidate_current
                || crate::candidate::passed_revalidation(project, subject, &acceptance_identity)?
            {
                assessment.current = true;
                return Ok(assessment);
            }
            return stale(assessment, "REVALIDATION_REQUIRED");
        }
    }
    let original = latest_impl_before(project, subject, &acceptance.timestamp)?;
    let current = hashing::working_tree_state(&project.root, &project.implementation_paths())?;
    assessment.comparisons.push(hashing::Comparison {
        surface: "implementation_state".into(),
        was: Some(original.clone()),
        now: Some(current.clone()),
    });
    assessment.current = original == current
        || crate::candidate::passed_revalidation(project, subject, &acceptance_identity)?
        || RevalidationStore::at(&project.codeos_dir())
            .all()?
            .into_iter()
            .rev()
            .find(|r| {
                r.subject == subject
                    && r.current_implementation_state == current
                    && r.governed_inputs == governed_now
            })
            .is_some_and(|r| r.decision == "REVALIDATED");
    if assessment.current {
        Ok(assessment)
    } else {
        stale(assessment, "REVALIDATION_REQUIRED")
    }
}

fn accepted_architecture_identity(
    project: &Project,
    subject: &str,
    acceptance_timestamp: &str,
) -> Result<String> {
    let baseline = git(
        project,
        &[
            "rev-list",
            "-1",
            &format!("--before={acceptance_timestamp}"),
            "HEAD",
        ],
    )?;
    anyhow::ensure!(
        !baseline.is_empty(),
        "cannot resolve architecture baseline at F9 acceptance"
    );
    crate::architecture::feature_identity(project, subject, &baseline)
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
        let f1_base = crate::architecture::feature_identity(&project, "F-0001", &base).unwrap();
        let f2_base = crate::architecture::feature_identity(&project, "F-0002", &base).unwrap();
        let f3_base = crate::architecture::feature_identity(&project, "F-0003", &base).unwrap();

        std::fs::write(
            &scope,
            approved_scope("F-0001, F-0002, F-0004", "2026-01-02", body),
        )
        .unwrap();
        git(&["add", "."]);
        git(&["commit", "-qm", "add sibling"]);
        let sibling = git(&["rev-parse", "HEAD"]);
        assert_eq!(
            crate::architecture::feature_identity(&project, "F-0001", &sibling).unwrap(),
            f1_base
        );
        assert_eq!(
            crate::architecture::feature_identity(&project, "F-0002", &sibling).unwrap(),
            f2_base
        );
        assert_eq!(
            crate::architecture::feature_identity(&project, "F-0003", &sibling).unwrap(),
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
            crate::architecture::feature_identity(&project, "F-0001", &obligation).unwrap(),
            f1_base
        );
        assert_eq!(
            crate::architecture::feature_identity(&project, "F-0003", &obligation).unwrap(),
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
            crate::architecture::feature_identity(&project, "F-0001", &ambiguous)
                .unwrap_err()
                .to_string()
                .contains("ambiguous")
        );

        std::fs::write(scopes.join("two.md"), "---\nfeatures: [F-0002]\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-qm", "malformed"]);
        let malformed = git(&["rev-parse", "HEAD"]);
        assert!(crate::architecture::feature_identity(&project, "F-0003", &malformed).is_err());
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
    fn next_step_uses_the_commands_own_trigger_and_refusals() {
        let (dir, project, c) = change_history();
        let root = dir.path();
        // The derived trigger is the only commit `identified_change` accepts: the latest surface
        // commit, not the later governance-only commit.
        assert_eq!(
            latest_surface_commit(&project).unwrap().as_deref(),
            Some(c[2].as_str())
        );
        identified_change(&project, "F-0001", &c[2], "2026-01-01T00:00:00Z").unwrap();
        let subject =
            Subject::resolve(&project, crate::contract::Workflow::Feature, "F-0001").unwrap();
        // Status fails closed with the refusal the commands would report, never an error.
        let ineligible = |why: &str| match next_step(&project, &subject) {
            RevalidationStep::Ineligible(reason) => assert!(reason.contains(why), "{reason}"),
            other => panic!("{other:?}"),
        };
        ineligible("requires Feature Workflow v5");

        std::os::unix::fs::symlink(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.."),
            root.join(".codeos/toolkit"),
        )
        .unwrap();
        let write = |rel: &str, text: &str| {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        };
        write(
            ".codeos/00-project/codeos.yaml",
            "codeos_schema: 2\ndba_configuration: DBA-25\n",
        );
        for rel in [
            ".codeos/01-specification/intents/F-0001.md",
            ".codeos/01-specification/contracts/F-0001_contract.md",
            ".codeos/01-specification/event-schemas/F-0001_schema.md",
        ] {
            write(rel, "approved\n");
        }
        write(
            ".codeos/06-workflow/decisions.jsonl",
            r#"{"workflow":"feature","subject":"F-0001","checkpoint":"acceptance","result":"accepted","bindings":{"intent":"old"},"timestamp":"2026-01-01T00:00:00Z"}"#,
        );
        ineligible("governed meaning changed at intent");

        // An uncommitted surface change precedes every other step; the command refuses it too.
        write("src/lib.rs", "uncommitted");
        assert_eq!(
            next_step(&project, &subject),
            RevalidationStep::CommitChange
        );
        assert!(
            identified_change(&project, "F-0001", &c[2], "2026-01-01T00:00:00Z")
                .unwrap_err()
                .to_string()
                .contains("latest change")
        );
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
