//! Immutable feature candidates shared by F7, F8, F9, and post-acceptance validity checks.

use crate::checker::{self, State, Subject};
use crate::hashing;
use crate::project::Project;
use crate::receipts::ReceiptStore;
use crate::verification::{self, RecordState, VerificationStore};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CandidateInput {
    pub path: String,
    pub sha256: String,
    pub role: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CandidateBundle {
    pub schema_version: u8,
    pub feature: String,
    pub semantic_revision: u64,
    pub kind: String,
    pub configuration_sha256: String,
    pub governed_inputs: BTreeMap<String, String>,
    pub implementation_inputs: Vec<CandidateInput>,
    pub dependency_additions: Vec<String>,
    pub verification_records: BTreeMap<String, String>,
    pub f6_receipt_sha256: String,
    pub production_evidence: Option<String>,
    pub runtime_evidence: Option<String>,
    pub journey_previous: String,
    pub journey_target: String,
    pub predecessor_bundle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_acceptance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_commit: Option<String>,
    /// Feature Workflow v11: inherited claim -> the upstream evidence at candidate time.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub inherited_evidence: BTreeMap<String, InheritedEvidence>,
}

/// The upstream production truth an inherited claim restated when the candidate was prepared.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InheritedEvidence {
    pub feature: String,
    pub claim: String,
    pub bundle: String,
    pub evidence: String,
}

pub struct CandidateStore {
    root: PathBuf,
}

impl CandidateStore {
    pub fn at(codeos_dir: &Path) -> Self {
        Self {
            root: codeos_dir.join("06-workflow/candidates"),
        }
    }

    pub fn path(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.json"))
    }

    pub fn read(&self, id: &str) -> Result<CandidateBundle> {
        let bytes =
            std::fs::read(self.path(id)).with_context(|| format!("read candidate bundle {id}"))?;
        let bundle: CandidateBundle = serde_json::from_slice(&bytes)?;
        anyhow::ensure!(
            identity(&bundle)? == id,
            "candidate bundle identity does not match its canonical payload"
        );
        Ok(bundle)
    }

    pub fn write(&self, bundle: &CandidateBundle) -> Result<String> {
        let id = identity(bundle)?;
        std::fs::create_dir_all(&self.root)?;
        let path = self.path(&id);
        let bytes = serde_json::to_vec_pretty(bundle)?;
        if path.exists() {
            anyhow::ensure!(
                std::fs::read(&path)? == bytes,
                "candidate bundle identity collision"
            );
        } else {
            std::fs::write(path, bytes)?;
        }
        Ok(id)
    }

    pub fn all(&self) -> Result<Vec<(String, CandidateBundle)>> {
        if !self.root.is_dir() {
            return Ok(vec![]);
        }
        let mut paths = std::fs::read_dir(&self.root)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("json"))
            .collect::<Vec<_>>();
        paths.sort();
        let mut out = Vec::new();
        for path in paths {
            let Some(id) = path.file_stem().and_then(|v| v.to_str()) else {
                continue;
            };
            out.push((id.to_string(), self.read(id)?));
        }
        Ok(out)
    }
}

pub fn identity(bundle: &CandidateBundle) -> Result<String> {
    hashing::record_identity(bundle)
}

pub fn prepare(
    project: &Project,
    subject: &Subject,
    additions: &[String],
    kind: &str,
    trigger: Option<&str>,
) -> Result<(String, CandidateBundle)> {
    let bundle = assemble(project, subject, additions, kind, trigger)?;
    let id = CandidateStore::at(&project.codeos_dir()).write(&bundle)?;
    Ok((id, bundle))
}

/// Every check `prepare` performs, without writing. `status` uses it so that it advertises a
/// `prepare-candidate` command only when that command would succeed.
pub fn assemble(
    project: &Project,
    subject: &Subject,
    additions: &[String],
    kind: &str,
    trigger: Option<&str>,
) -> Result<CandidateBundle> {
    anyhow::ensure!(
        project.workflow_governance_version()? >= 3,
        "Candidate Bundles require Workflow Governance v3"
    );
    let revalidation_case = if kind == "post_acceptance_revalidation" {
        let trigger = trigger.context("revalidation candidate requires --trigger")?;
        Some(crate::revalidation::build_case(
            project,
            &subject.id,
            trigger,
        )?)
    } else {
        anyhow::ensure!(
            trigger.is_none(),
            "--trigger applies only to revalidation candidates"
        );
        let reports = checker::evaluate(project, subject)?;
        let unresolved: Vec<_> = reports
            .iter()
            .take_while(|r| r.id != "F7")
            .filter(|r| !matches!(r.state, State::Pass | State::NotApplicable))
            .map(|r| r.id.clone())
            .collect();
        anyhow::ensure!(
            unresolved.is_empty(),
            "candidate is not ready; unresolved checkpoints: {}",
            unresolved.join(", ")
        );
        None
    };

    let governed_inputs = governed(project, &subject.id, 2)?;
    let configuration_sha256 =
        hashing::file_sha256(&project.codeos_yaml())?.context("missing codeos.yaml")?;
    let receipt = ReceiptStore::at(&project.codeos_dir())
        .latest("feature", &subject.id, "reconciliation")?
        .context("candidate requires the F6 reconciliation receipt")?;
    let f6_receipt_sha256 = hashing::record_identity(&receipt)?;
    // Revalidation creates no F6 receipt (Feature Workflow v8), so it selects the production
    // evidence verified for the revalidation implementation instead of the historical F6 selection.
    let production_evidence = match &revalidation_case {
        Some(case) => case.production_evidence.clone(),
        None => receipt.bindings.get("production_evidence").cloned(),
    };

    let mut verification_records = BTreeMap::new();
    let store = VerificationStore::at(&project.codeos_dir());
    let qualification_bindings = checker::verification_bindings_for_mechanic(
        project,
        subject,
        "candidate",
        "project_qualification",
    )?;
    let qualification =
        store.latest("feature", &subject.id, "candidate", "project_qualification")?;
    let qualification =
        require_pre_candidate_qualification(qualification, &qualification_bindings)?;
    verification_records.insert(
        "candidate/project_qualification".into(),
        hashing::record_identity(&qualification)?,
    );
    for (checkpoint, mechanic) in checker::candidate_verification_records(project, subject)? {
        if checkpoint == "candidate" {
            continue;
        }
        let record = store
            .latest("feature", &subject.id, checkpoint, mechanic)?
            .with_context(|| {
                format!("candidate requires current {checkpoint}/{mechanic} verification")
            })?;
        let want =
            checker::verification_bindings_for_mechanic(project, subject, checkpoint, mechanic)?;
        anyhow::ensure!(
            matches!(
                crate::verification::evaluate(Some(record.clone()), &want),
                crate::verification::RecordState::Current(_)
            ),
            "candidate verification {checkpoint}/{mechanic} is stale"
        );
        verification_records.insert(
            format!("{checkpoint}/{mechanic}"),
            hashing::record_identity(&record)?,
        );
    }
    let runtime_evidence = if observation_mode(project, &subject.id)? == "events" {
        let runtime = store
            .latest("feature", &subject.id, "F7", "runtime_event_schema")?
            .context("candidate requires current F7/runtime_event_schema verification")?;
        let runtime_want = checker::verification_bindings_for_mechanic(
            project,
            subject,
            "F7",
            "runtime_event_schema",
        )?;
        let validator_path = runtime
            .details
            .as_ref()
            .and_then(|details| details.get("event_evidence_validator"))
            .and_then(|value| value.as_str())
            .context("candidate runtime verification has no validator path binding")?;
        let validator = Path::new(validator_path);
        anyhow::ensure!(
            !validator.is_absolute()
                && !validator
                    .components()
                    .any(|component| matches!(component, std::path::Component::ParentDir)),
            "candidate runtime validator must be a project-relative path"
        );
        let mut runtime_want = runtime_want;
        runtime_want.insert(
            "event_evidence_validator".into(),
            hashing::file_sha256(&project.root.join(validator))?
                .context("candidate runtime validator disappeared")?,
        );
        anyhow::ensure!(
            matches!(
                crate::verification::evaluate(Some(runtime.clone()), &runtime_want),
                crate::verification::RecordState::Current(_)
            ),
            "candidate F7/runtime_event_schema verification is stale"
        );
        let evidence = runtime
            .details
            .as_ref()
            .and_then(|d| d.get("event_evidence"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        anyhow::ensure!(
            evidence.is_some(),
            "candidate runtime verification has no inspectable event evidence binding"
        );
        verification_records.insert(
            "F7/runtime_event_schema".into(),
            hashing::record_identity(&runtime)?,
        );
        evidence
    } else {
        None
    };

    let dependency_additions = validate_additions(project, additions)?;
    let implementation_inputs = derive_inputs(project, &subject.id, &dependency_additions)?;
    anyhow::ensure!(
        !implementation_inputs.is_empty(),
        "candidate implementation/dependency surface is empty"
    );
    let mut inherited_evidence = BTreeMap::new();
    for (claim, state) in crate::revalidation::inherited_evidence(project, subject)? {
        let evidence = state
            .map_err(|why| anyhow::anyhow!("inherited claim '{claim}' is not current: {why}"))?;
        inherited_evidence.insert(claim, evidence);
    }
    let predecessor_bundle = latest_accepted_bundle(project, &subject.id)?;
    let journey_previous = checker::current_accepted_frontier(project)?;
    let journey_target = checker::product_journey_target(project, subject)?;
    let bundle = CandidateBundle {
        schema_version: 2,
        feature: subject.id.clone(),
        semantic_revision: semantic_revision(&std::fs::read_to_string(
            project.intent(&subject.id),
        )?)?,
        kind: kind.into(),
        configuration_sha256,
        governed_inputs,
        implementation_inputs,
        dependency_additions,
        verification_records,
        f6_receipt_sha256,
        production_evidence,
        runtime_evidence,
        journey_previous,
        journey_target,
        predecessor_bundle,
        baseline_acceptance: revalidation_case
            .as_ref()
            .map(|case| case.original_acceptance_sha256.clone()),
        trigger_commit: revalidation_case.map(|case| case.trigger_commit),
        inherited_evidence,
    };
    Ok(bundle)
}

fn require_pre_candidate_qualification(
    record: Option<crate::verification::VerificationRecord>,
    bindings: &BTreeMap<String, String>,
) -> Result<crate::verification::VerificationRecord> {
    match verification::evaluate(record, bindings) {
        RecordState::Current(record) => Ok(record),
        RecordState::Absent => {
            anyhow::bail!("candidate requires a passed pre-candidate project qualification")
        }
        RecordState::Failed(_) => {
            anyhow::bail!("candidate pre-candidate project qualification failed")
        }
        RecordState::Stale { binding, .. } => anyhow::bail!(
            "candidate pre-candidate project qualification is stale: {binding} changed"
        ),
    }
}

/// Product/governed currency used after acceptance. Test and evidence changes do not change the
/// accepted product; they matter only when preparing another candidate.
pub fn acceptance_current(project: &Project, id: &str) -> Result<bool> {
    acceptance_current_inner(project, id, false)
}

/// DBA-24 may project a legacy accepted bundle with feature-scoped architecture comparison.
/// The caller must independently prove that the applicable architecture identity is unchanged.
pub fn acceptance_current_with_scoped_legacy_architecture(
    project: &Project,
    id: &str,
) -> Result<bool> {
    acceptance_current_inner(project, id, true)
}

fn acceptance_current_inner(project: &Project, id: &str, scoped_legacy: bool) -> Result<bool> {
    Ok(!acceptance_comparisons(project, id, scoped_legacy)?
        .iter()
        .any(hashing::Comparison::changed))
}

/// The comparisons that decide whether an accepted bundle is still current: every non-test
/// implementation input, then, only if all are unchanged, every governed input present on either
/// side.
pub fn acceptance_comparisons(
    project: &Project,
    id: &str,
    scoped_legacy: bool,
) -> Result<Vec<hashing::Comparison>> {
    let bundle = CandidateStore::at(&project.codeos_dir()).read(id)?;
    let mut comparisons = Vec::new();
    for input in bundle
        .implementation_inputs
        .iter()
        .filter(|input| input.role != "test")
    {
        comparisons.push(hashing::Comparison {
            surface: input.path.clone(),
            was: Some(input.sha256.clone()),
            now: hashing::file_sha256(&project.root.join(&input.path))?,
        });
    }
    if comparisons.iter().any(hashing::Comparison::changed) {
        return Ok(comparisons);
    }
    let mut current = governed(project, &bundle.feature, bundle.schema_version)?;
    let mut accepted = bundle.governed_inputs;
    if scoped_legacy && bundle.schema_version == 1 {
        current.retain(|key, _| !key.starts_with("architecture:"));
        accepted.retain(|key, _| !key.starts_with("architecture:"));
    }
    let keys: std::collections::BTreeSet<_> =
        accepted.keys().chain(current.keys()).cloned().collect();
    comparisons.extend(keys.into_iter().map(|key| hashing::Comparison {
        was: accepted.get(&key).cloned(),
        now: current.get(&key).cloned(),
        surface: key,
    }));
    Ok(comparisons)
}

pub fn current_for_review(project: &Project, id: &str) -> Result<bool> {
    let bundle = CandidateStore::at(&project.codeos_dir()).read(id)?;
    if !acceptance_current(project, id)? {
        return Ok(false);
    }
    let receipt = ReceiptStore::at(&project.codeos_dir())
        .latest("feature", &bundle.feature, "reconciliation")?
        .context("candidate reconciliation receipt disappeared")?;
    if hashing::record_identity(&receipt)? != bundle.f6_receipt_sha256 {
        return Ok(false);
    }
    let store = VerificationStore::at(&project.codeos_dir());
    for (key, identity) in &bundle.verification_records {
        let Some((checkpoint, mechanic)) = key.split_once('/') else {
            return Ok(false);
        };
        let Some(record) = store.latest("feature", &bundle.feature, checkpoint, mechanic)? else {
            return Ok(false);
        };
        if hashing::record_identity(&record)? != *identity {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn unique_current(project: &Project, feature: &str) -> Result<Option<String>> {
    let mut current = Vec::new();
    for (id, bundle) in CandidateStore::at(&project.codeos_dir()).all()? {
        if bundle.feature == feature && current_for_review(project, &id)? {
            current.push(id);
        }
    }
    anyhow::ensure!(current.len() <= 1, "multiple current candidate bundles exist for {feature}; change a bound input before preparing another candidate");
    Ok(current.pop())
}

/// A PASSED post-acceptance candidate restores current validity without creating another F9.
pub fn passed_revalidation(
    project: &Project,
    feature: &str,
    baseline_acceptance: &str,
) -> Result<bool> {
    Ok(passed_revalidation_bundle(project, feature, baseline_acceptance)?.is_some())
}

/// The current revalidation candidate that a PASSED assessment restored to validity, if any.
pub fn passed_revalidation_bundle(
    project: &Project,
    feature: &str,
    baseline_acceptance: &str,
) -> Result<Option<String>> {
    let Some(review) = crate::evidence::latest_review_record(&project.root, feature, "8")? else {
        return Ok(None);
    };
    if review.candidate_outcome.as_deref() != Some("PASSED") {
        return Ok(None);
    }
    let Some(reviewed_id) = review.candidate_bundle.as_deref() else {
        return Ok(None);
    };
    let bundle = match CandidateStore::at(&project.codeos_dir()).read(reviewed_id) {
        Ok(bundle) => bundle,
        Err(_) => return Ok(None),
    };
    Ok((bundle.feature == feature
        && bundle.kind == "post_acceptance_revalidation"
        && bundle.baseline_acceptance.as_deref() == Some(baseline_acceptance)
        && current_for_review(project, reviewed_id)?)
    .then(|| reviewed_id.to_string()))
}

pub(crate) fn governed(
    project: &Project,
    feature: &str,
    schema_version: u8,
) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for (name, path) in [
        ("intent", project.intent(feature)),
        ("contract", project.contract(feature)),
        ("event_schema", project.event_schema(feature)),
    ] {
        out.insert(
            name.into(),
            hashing::file_sha256(&path)?.with_context(|| format!("missing {name}"))?,
        );
    }
    if schema_version >= 2 {
        if let Some(path) = crate::architecture::scope_path(project, feature)? {
            out.insert(
                format!("architecture:{path}"),
                crate::architecture::feature_identity(project, feature, "WORKTREE")?,
            );
        }
        return Ok(out);
    }
    let scopes = project.codeos_dir().join("02-architecture/scopes");
    if scopes.is_dir() {
        let mut paths = std::fs::read_dir(scopes)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            let text = std::fs::read_to_string(&path)?;
            let Some(frontmatter) = text
                .strip_prefix("---\n")
                .and_then(|rest| rest.split_once("\n---").map(|pair| pair.0))
            else {
                continue;
            };
            let metadata: serde_yaml::Value = serde_yaml::from_str(frontmatter)?;
            let owns_feature = metadata
                .get("features")
                .and_then(|value| value.as_sequence())
                .is_some_and(|features| {
                    features.iter().any(|value| value.as_str() == Some(feature))
                });
            if owns_feature {
                let relative = path
                    .strip_prefix(&project.root)?
                    .to_string_lossy()
                    .to_string();
                out.insert(
                    format!("architecture:{relative}"),
                    hashing::file_sha256(&path)?
                        .context("owning architecture scope disappeared")?,
                );
            }
        }
    }
    Ok(out)
}

fn latest_accepted_bundle(project: &Project, feature: &str) -> Result<Option<String>> {
    Ok(ReceiptStore::at(&project.codeos_dir())
        .latest("feature", feature, "acceptance")?
        .and_then(|r| r.bindings.get("candidate_bundle").cloned()))
}

fn validate_additions(project: &Project, additions: &[String]) -> Result<Vec<String>> {
    let mut out = BTreeSet::new();
    for supplied in additions {
        let rel = Path::new(supplied);
        anyhow::ensure!(
            !rel.is_absolute()
                && !rel
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir)),
            "dependency addition must be a non-traversing project-relative path"
        );
        let full = project.root.join(rel);
        anyhow::ensure!(
            full.is_file(),
            "dependency addition does not name a file: {supplied}"
        );
        out.insert(supplied.clone());
    }
    Ok(out.into_iter().collect())
}

fn derive_inputs(
    project: &Project,
    feature: &str,
    additions: &[String],
) -> Result<Vec<CandidateInput>> {
    let intent = std::fs::read_to_string(project.intent(feature))?;
    let approved_at =
        yaml_string(&intent, "approved_at").unwrap_or_else(|| "1970-01-01T00:00:00Z".into());
    let paths = project.implementation_paths();
    let mut args = vec![
        "log".to_string(),
        format!("--since={approved_at}"),
        "--name-only".into(),
        "--format=".into(),
        "--".into(),
    ];
    args.extend(paths.clone());
    let mut files: BTreeSet<String> = git(project, &args)?
        .lines()
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .collect();
    let mut dirty_args = vec![
        "ls-files".to_string(),
        "--modified".into(),
        "--others".into(),
        "--exclude-standard".into(),
        "--".into(),
    ];
    dirty_args.extend(paths);
    files.extend(
        git(project, &dirty_args)?
            .lines()
            .filter(|v| !v.is_empty())
            .map(str::to_string),
    );
    for conventional in [
        "Cargo.toml",
        "Cargo.lock",
        "backend/Cargo.toml",
        "backend/Cargo.lock",
        "web/package.json",
        "web/package-lock.json",
        "docker-compose.yml",
    ] {
        if project.root.join(conventional).is_file() {
            files.insert(conventional.into());
        }
    }
    files.extend(additions.iter().cloned());
    let mut out = Vec::new();
    for path in files {
        let Some(sha256) = hashing::file_sha256(&project.root.join(&path))? else {
            continue;
        };
        let role = if hashing::is_test_path(&path) {
            "test"
        } else if path.ends_with("Cargo.toml")
            || path.ends_with("Cargo.lock")
            || path.ends_with("package.json")
            || path.ends_with("package-lock.json")
        {
            "dependency"
        } else if path == "docker-compose.yml" {
            "runtime_topology"
        } else {
            "product"
        };
        let source = if additions.contains(&path) {
            "explicit_addition"
        } else {
            "derived"
        };
        out.push(CandidateInput {
            path,
            sha256,
            role: role.into(),
            source: source.into(),
        });
    }
    Ok(out)
}

fn semantic_revision(text: &str) -> Result<u64> {
    yaml_string(text, "semantic_revision")
        .context("missing semantic_revision")?
        .parse()
        .context("invalid semantic_revision")
}

pub(crate) fn observation_mode(project: &Project, feature: &str) -> Result<&'static str> {
    let schema = std::fs::read_to_string(project.event_schema(feature))?;
    if schema
        .lines()
        .any(|line| line.trim() == "**Mode:** `events`")
    {
        Ok("events")
    } else if schema
        .lines()
        .any(|line| line.trim() == "**Mode:** `external-observation`")
    {
        Ok("external-observation")
    } else {
        anyhow::bail!("Event Schema must declare exactly one supported Observation Mode")
    }
}
fn yaml_string(text: &str, key: &str) -> Option<String> {
    let fm = text.strip_prefix("---\n")?.split_once("\n---")?.0;
    let value: serde_yaml::Value = serde_yaml::from_str(fm).ok()?;
    value.get(key).and_then(|v| {
        v.as_str()
            .map(str::to_string)
            .or_else(|| v.as_u64().map(|n| n.to_string()))
    })
}
fn git(project: &Project, args: &[String]) -> Result<String> {
    let out = crate::instrument::command("git")
        .arg("-C")
        .arg(&project.root)
        .args(args)
        .output()?;
    anyhow::ensure!(out.status.success(), "git candidate-surface query failed");
    Ok(String::from_utf8_lossy(&out.stdout).trim().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture() -> (tempfile::TempDir, Project, CandidateBundle) {
        let dir = tempfile::tempdir().unwrap();
        for path in [
            ".codeos/00-project/codeos.yaml",
            ".codeos/01-specification/intents/F-1.md",
            ".codeos/01-specification/contracts/F-1_contract.md",
            ".codeos/01-specification/event-schemas/F-1_schema.md",
            "src/product.rs",
            "tests/product.spec.ts",
            ".codeos/02-architecture/scopes/shared.md",
        ] {
            let full = dir.path().join(path);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(&full, format!("{path}\n")).unwrap();
        }
        fs::write(
            dir.path().join(".codeos/02-architecture/scopes/shared.md"),
            "---\nartifact_type: architecture\nfeatures: [F-1]\napproval:\n  by: Human\n  at: 2026-01-01T00:00:00Z\nreader_model: whole-before-parts\n---\n## Shared\nRule.\n",
        )
        .unwrap();
        let project = Project::discover(dir.path()).unwrap();
        let governed_inputs = governed(&project, "F-1", 1).unwrap();
        let input = |path: &str, role: &str| CandidateInput {
            path: path.into(),
            sha256: hashing::file_sha256(&project.root.join(path))
                .unwrap()
                .unwrap(),
            role: role.into(),
            source: "derived".into(),
        };
        let bundle = CandidateBundle {
            schema_version: 1,
            feature: "F-1".into(),
            semantic_revision: 1,
            kind: "ordinary".into(),
            configuration_sha256: "config".into(),
            governed_inputs,
            implementation_inputs: vec![
                input("src/product.rs", "product"),
                input("tests/product.spec.ts", "test"),
            ],
            dependency_additions: vec![],
            verification_records: BTreeMap::new(),
            f6_receipt_sha256: "f6".into(),
            production_evidence: None,
            runtime_evidence: None,
            journey_previous: "entry".into(),
            journey_target: "outcome".into(),
            predecessor_bundle: None,
            baseline_acceptance: None,
            trigger_commit: None,
            inherited_evidence: Default::default(),
        };
        (dir, project, bundle)
    }

    #[test]
    fn identity_is_deterministic_and_store_verifies_payload() {
        let (_dir, project, bundle) = fixture();
        let store = CandidateStore::at(&project.codeos_dir());
        let first = store.write(&bundle).unwrap();
        let second = store.write(&bundle).unwrap();
        assert_eq!(first, second);
        assert_eq!(store.read(&first).unwrap(), bundle);
    }

    #[test]
    fn accepted_product_ignores_test_only_drift_but_not_product_drift() {
        let (_dir, project, bundle) = fixture();
        let id = CandidateStore::at(&project.codeos_dir())
            .write(&bundle)
            .unwrap();
        fs::write(project.root.join("tests/product.spec.ts"), "changed test\n").unwrap();
        assert!(acceptance_current(&project, &id).unwrap());
        fs::write(project.root.join("src/product.rs"), "changed product\n").unwrap();
        assert!(!acceptance_current(&project, &id).unwrap());
    }

    #[test]
    fn acceptance_explanation_is_the_comparison_that_decides_currency() {
        let (_dir, project, bundle) = fixture();
        let id = CandidateStore::at(&project.codeos_dir())
            .write(&bundle)
            .unwrap();
        let changed = |project: &Project| {
            acceptance_comparisons(project, &id, false)
                .unwrap()
                .into_iter()
                .filter(hashing::Comparison::changed)
                .map(|c| (c.surface, c.now.is_some()))
                .collect::<Vec<_>>()
        };
        // A relevant file added after acceptance is outside the recorded inputs: still current.
        fs::write(project.root.join("src/added.rs"), "new\n").unwrap();
        // Test-only drift is excluded from accepted-product currency.
        fs::write(project.root.join("tests/product.spec.ts"), "changed test\n").unwrap();
        assert!(acceptance_current(&project, &id).unwrap());
        assert!(changed(&project).is_empty());

        fs::write(project.root.join("src/product.rs"), "changed product\n").unwrap();
        assert!(!acceptance_current(&project, &id).unwrap());
        assert_eq!(changed(&project), [("src/product.rs".to_string(), true)]);

        fs::remove_file(project.root.join("src/product.rs")).unwrap();
        assert!(!acceptance_current(&project, &id).unwrap());
        assert_eq!(changed(&project), [("src/product.rs".to_string(), false)]);
    }

    #[test]
    fn scoped_projection_does_not_rewrite_legacy_bundle_or_invalidate_sibling_membership() {
        let (_dir, project, bundle) = fixture();
        let id = CandidateStore::at(&project.codeos_dir())
            .write(&bundle)
            .unwrap();
        let scope = project
            .root
            .join(".codeos/02-architecture/scopes/shared.md");
        let original = fs::read_to_string(&scope).unwrap();
        fs::write(
            &scope,
            original.replace("features: [F-1]", "features: [F-1, F-2]"),
        )
        .unwrap();

        assert!(!acceptance_current(&project, &id).unwrap());
        assert!(acceptance_current_with_scoped_legacy_architecture(&project, &id).unwrap());
        assert_eq!(
            CandidateStore::at(&project.codeos_dir()).read(&id).unwrap(),
            bundle
        );
    }

    #[test]
    fn explicit_dependency_additions_cannot_escape_the_project() {
        let (_dir, project, _) = fixture();
        assert!(validate_additions(&project, &["../outside".into()]).is_err());
    }

    #[test]
    fn runtime_evidence_applicability_follows_the_governed_observation_mode() {
        let (_dir, project, _) = fixture();
        let schema = project.event_schema("F-1");
        fs::write(
            &schema,
            "# Event Schema\n\n**Mode:** `external-observation`\n",
        )
        .unwrap();
        assert_eq!(
            observation_mode(&project, "F-1").unwrap(),
            "external-observation"
        );
        fs::write(&schema, "# Event Schema\n\n**Mode:** `events`\n").unwrap();
        assert_eq!(observation_mode(&project, "F-1").unwrap(), "events");
        fs::write(&schema, "# Event Schema\n").unwrap();
        assert!(observation_mode(&project, "F-1").is_err());
    }

    #[test]
    fn revalidation_requires_runtime_evidence_only_in_events_mode() {
        let (_dir, project, _) = fixture();
        let subject = crate::checker::Subject {
            workflow: crate::contract::Workflow::Feature,
            id: "F-1".into(),
            gui: false,
            persistence: false,
            arch_approval_required: false,
            reality_claims: Vec::new(),
            inherited_claims: Vec::new(),
        };
        let event_evidence = || {
            crate::revalidation::required_verifications(&project, &subject)
                .unwrap()
                .contains(&("F7", "runtime_event_schema"))
        };
        let schema = project.event_schema("F-1");
        fs::write(
            &schema,
            "# Event Schema\n\n**Mode:** `external-observation`\n",
        )
        .unwrap();
        assert!(!event_evidence());
        fs::write(&schema, "# Event Schema\n\n**Mode:** `events`\n").unwrap();
        assert!(event_evidence());
    }

    #[test]
    fn runtime_candidate_preflight_uses_the_same_governed_bindings_as_event_evidence() {
        let (_dir, project, _) = fixture();
        let subject = Subject {
            workflow: crate::contract::Workflow::Feature,
            id: "F-1".into(),
            gui: true,
            persistence: false,
            arch_approval_required: false,
            reality_claims: vec![],
            inherited_claims: vec![],
        };
        let bindings = checker::verification_bindings_for_mechanic(
            &project,
            &subject,
            "F7",
            "runtime_event_schema",
        )
        .unwrap();
        assert_eq!(
            bindings.get("intent"),
            hashing::file_sha256(&project.intent("F-1"))
                .unwrap()
                .as_ref()
        );
        assert_eq!(
            bindings.get("contract"),
            hashing::file_sha256(&project.contract("F-1"))
                .unwrap()
                .as_ref()
        );
        assert_eq!(
            bindings.get("event_schema"),
            hashing::file_sha256(&project.event_schema("F-1"))
                .unwrap()
                .as_ref()
        );
        assert!(bindings.contains_key("implementation_state"));

        let qualification = checker::verification_bindings_for_mechanic(
            &project,
            &subject,
            "candidate",
            "project_qualification",
        )
        .unwrap();
        assert!(qualification.contains_key("intent"));
        assert!(qualification.contains_key("contract"));
        assert!(qualification.contains_key("event_schema"));
        assert!(qualification.contains_key("implementation_state"));
    }

    #[test]
    fn pre_candidate_qualification_fails_closed_when_missing_failed_or_stale() {
        let bindings = BTreeMap::from([("implementation_state".into(), "current".into())]);
        let record = crate::verification::VerificationRecord {
            workflow: "feature".into(),
            subject: "F-1".into(),
            checkpoint: "candidate".into(),
            verification: "project_qualification".into(),
            result: "passed".into(),
            bindings: bindings.clone(),
            command: "scripts/codeos-qualify".into(),
            timestamp: "2026-10-10T00:00:00Z".into(),
            details: None,
        };

        assert!(require_pre_candidate_qualification(Some(record.clone()), &bindings).is_ok());
        assert!(require_pre_candidate_qualification(None, &bindings).is_err());

        let mut failed = record.clone();
        failed.result = "failed".into();
        assert!(require_pre_candidate_qualification(Some(failed), &bindings).is_err());

        let stale = BTreeMap::from([("implementation_state".into(), "changed".into())]);
        assert!(require_pre_candidate_qualification(Some(record), &stale).is_err());
    }

    #[test]
    fn pre_candidate_qualification_does_not_replace_f7_f8_or_f9() {
        use crate::contract::{contract_for, Applies, Requirement, Workflow};

        let contract = contract_for(Workflow::Feature, 9, 2);
        let f7 = contract
            .iter()
            .find(|checkpoint| checkpoint.id == "F7")
            .unwrap();
        let f8 = contract
            .iter()
            .find(|checkpoint| checkpoint.id == "F8")
            .unwrap();
        let f9 = contract
            .iter()
            .find(|checkpoint| checkpoint.id == "F9")
            .unwrap();

        assert!(f7
            .requirements
            .iter()
            .any(|requirement| matches!(requirement, Requirement::ReviewRecord { stage: "8" })));
        assert_eq!(f8.applies, Applies::WhenGui);
        assert!(f8.requirements.iter().any(|requirement| matches!(
            requirement,
            Requirement::Receipt {
                checkpoint: "final_ux_validation",
                ..
            }
        )));
        assert!(f9.requirements.iter().any(|requirement| matches!(
            requirement,
            Requirement::Receipt {
                checkpoint: "acceptance",
                ..
            }
        )));
    }
}
