use crate::assessment::ParsedReview;
use crate::packet::ReviewPacket;
use crate::run::{ReviewerRun, RunSource};
use anyhow::{Context, Result};
use std::path::Path;

/// Parsed provenance from the most recent assessment file, used to build the
/// structured Provenance block in the decision log entry.
pub struct DecisionProvenance {
    pub assessment_path: String,
    pub review_commit: String,
    pub head_sha: String,
    pub packet_sha_stored: String,
    pub packet_sha_actual: String,
    pub coverage_state: String,
    /// Present when the human provided --override to lift the coverage gate or broken provenance.
    pub override_rationale: Option<String>,
    /// Set when an assessment file was found but could not be read or parsed. When set, all
    /// other fields except assessment_path and override_rationale are unreliable.
    pub provenance_error: Option<String>,
}

const LOG_HEADER: &str = "# Codeos Review Log (append-only, v0)

Append-only record of automated advisory reviews and the human decisions that follow them.
Entries are NEVER edited — a human decision is a separately appended entry. The reviewer is
advisory and read-only; APPROVE belongs to the human. The reviewer tool is the authoritative
owner of automated review records; see the reviewer_tool_contract selected by the project's DBA
configuration.
";

/// Initialize the log file if it does not exist.
fn ensure_log_exists(log_path: &Path) -> Result<()> {
    if !log_path.exists() {
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent).context("create review log directory")?;
        }
        std::fs::write(log_path, LOG_HEADER).context("initialize review log")?;
    }
    Ok(())
}

/// Compute the round number for the next review of `feature`+`stage`, by counting existing
/// `## ... REVIEW — {feature} — Stage {stage}` entries already in `log_path` and adding 1.
///
/// No new storage: derived entirely from the existing append-only log. A log that does not yet
/// exist, or exists with zero matching entries, both correctly yield round 1 — that is not an
/// error case. A log that exists but cannot be **read** (a distinct failure from "not found") is
/// a hard error: the caller must abort before any Codex invocation rather than silently
/// stamping a guessed round.
///
/// Matching is done by exact, `—`/newline-bounded suffix (`content.lines()` strips the trailing
/// newline, so `line.ends_with(" REVIEW — {feature} — Stage {stage}")` cannot confuse
/// `Stage 1` with `Stage 10`, or a feature id with a longer id sharing its prefix).
pub fn compute_review_round(log_path: &Path, feature: &str, stage: &str) -> Result<u32> {
    // Read directly rather than pre-checking `exists()`: `Path::exists()` collapses genuine
    // absence and other metadata-access failures (e.g. a permission error) into the same
    // `false` result, which would silently mask a real error as "no log yet" — exactly the
    // fail-closed violation AC-10 forbids. Matching `io::ErrorKind::NotFound` specifically is
    // the only way to distinguish "round 1, no error" from "cannot determine, must abort."
    let content = match std::fs::read_to_string(log_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(1),
        Err(e) => {
            return Err(e).with_context(|| {
                format!(
                    "could not read review log to compute round: {}",
                    log_path.display()
                )
            });
        }
    };
    let next = count_entries(&content, REVIEW_HEADING, feature, stage) + 1;
    if next > ROUND_BUDGET {
        anyhow::bail!(
            "ordinary review round {next} would exceed the {ROUND_BUDGET}-round budget — use an explicitly authorized review re-entry instead"
        );
    }
    Ok(next)
}

/// Stage 8 ordinary round within one approved semantic Specification Package revision.
pub fn compute_semantic_review_round(
    log_path: &Path,
    feature: &str,
    stage: &str,
    semantic_revision: u64,
) -> Result<u32> {
    let content = match std::fs::read_to_string(log_path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(1),
        Err(error) => return Err(error).context("read review log for semantic review round"),
    };
    let count = content
        .lines()
        .filter_map(|line| line.strip_prefix("Review ID: "))
        .filter_map(|id| parse_ordinary_review_id(id, feature, stage))
        .filter(|id| id.semantic_revision == Some(semantic_revision))
        .count() as u32;
    let next = count + 1;
    if next > ROUND_BUDGET {
        anyhow::bail!(
            "ordinary semantic-revision review round {next} would exceed the {ROUND_BUDGET}-round budget"
        );
    }
    Ok(next)
}

/// Heading verbs that distinguish the two record kinds in the log. `compute_review_round` counts
/// only `REVIEW`, so recording an external assessment can never advance a required review round —
/// the exclusion falls out of the heading rather than needing a separate rule to stay in sync.
pub const REVIEW_HEADING: &str = "REVIEW";
pub const EXTERNAL_ASSESSMENT_HEADING: &str = "EXTERNAL ASSESSMENT";
pub const REENTRY_REVIEW_HEADING: &str = "RE-ENTRY REVIEW";
pub const EVIDENCE_CLOSURE_HEADING: &str = "EVIDENCE CLOSURE REVIEW";
pub const INFRASTRUCTURE_REVALIDATION_HEADING: &str = "INFRASTRUCTURE REVALIDATION";
pub const EVIDENCE_REASSESSMENT_HEADING: &str = "EVIDENCE REASSESSMENT";
pub const TERMINAL_CLOSURE_HEADING: &str = "TERMINAL REVIEW CLOSURE";

/// `REVIEW_EXECUTION_INVALID` (Reviewer Tool Contract v17). The entry carries an execution
/// target, never a `Review ID:`, and its heading matches no counted kind, so no round or bounded
/// mechanism can consume it.
pub const EXECUTION_INVALID_HEADING: &str = "REVIEW EXECUTION INVALID";

pub fn append_execution_invalid(
    log_path: &Path,
    feature: &str,
    stage: &str,
    target: &str,
    packet_sha256: &str,
    attempts: &[std::path::PathBuf],
) -> Result<()> {
    ensure_log_exists(log_path)?;
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut entry = format!(
        "\n## {ts} {EXECUTION_INVALID_HEADING} — {feature} — Stage {stage}\nExecution target: {target}\nResult: REVIEW_EXECUTION_INVALID\nReviewed packet SHA-256: {packet_sha256}\n"
    );
    for attempt in attempts {
        entry.push_str(&format!("Raw attempt: {}\n", attempt.display()));
    }
    entry.push_str("Semantic conclusion: none. Review capacity consumed: none.\n");
    append_to_log(log_path, &entry)
}

fn count_entries(content: &str, kind: &str, feature: &str, stage: &str) -> u32 {
    let record = format!("{} — {} — Stage {}", kind, feature, stage);
    content
        .lines()
        .filter(|line| {
            line.strip_prefix("## ")
                .and_then(|line| line.split_once(' '))
                .is_some_and(|(_, body)| body == record)
        })
        .count() as u32
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OrdinaryReviewId {
    pub feature: String,
    pub stage: String,
    pub semantic_revision: Option<u64>,
    pub round: u32,
}

fn canonical_positive_u64(value: &str) -> Option<u64> {
    let parsed = value.parse::<u64>().ok()?;
    (parsed > 0 && parsed.to_string() == value).then_some(parsed)
}

fn canonical_round(value: &str) -> Option<u32> {
    let parsed = value.parse::<u32>().ok()?;
    ((1..=ROUND_BUDGET).contains(&parsed) && parsed.to_string() == value).then_some(parsed)
}

/// Parse an ordinary review ID for an already-known feature and stage. Special review records use
/// different terminal components and are deliberately excluded.
pub(crate) fn parse_ordinary_review_id(
    review_id: &str,
    feature: &str,
    stage: &str,
) -> Option<OrdinaryReviewId> {
    let tail = review_id.strip_prefix(&format!("REV__{feature}__{stage}__"))?;
    let (semantic_revision, round) = if let Some(round) = tail.strip_prefix('R') {
        (None, canonical_round(round)?)
    } else {
        let semantic = tail.strip_prefix("SR")?;
        let (revision, round) = semantic.split_once("__R")?;
        (
            Some(canonical_positive_u64(revision)?),
            canonical_round(round)?,
        )
    };
    Some(OrdinaryReviewId {
        feature: feature.to_string(),
        stage: stage.to_string(),
        semantic_revision,
        round,
    })
}

fn review_ids_for_kind<'a>(
    content: &'a str,
    kind: &str,
    feature: &str,
    stage: &str,
) -> Vec<&'a str> {
    let record = format!("{kind} — {feature} — Stage {stage}");
    let lines: Vec<&str> = content.lines().collect();
    let mut ids = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let ordinary_heading = lines[index]
            .strip_prefix("## ")
            .and_then(|line| line.split_once(' '))
            .is_some_and(|(_, body)| body == record);
        if !ordinary_heading {
            index += 1;
            continue;
        }
        index += 1;
        while index < lines.len() && !lines[index].starts_with("## ") {
            if let Some(id) = lines[index].strip_prefix("Review ID: ") {
                ids.push(id.trim());
                break;
            }
            index += 1;
        }
        while index < lines.len() && !lines[index].starts_with("## ") {
            index += 1;
        }
    }
    ids
}

fn ordinary_review_ids<'a>(content: &'a str, feature: &str, stage: &str) -> Vec<&'a str> {
    review_ids_for_kind(content, REVIEW_HEADING, feature, stage)
}

fn reentry_bindings<'a>(content: &'a str, feature: &str, stage: &str) -> Vec<&'a str> {
    let record = format!("{REENTRY_REVIEW_HEADING} — {feature} — Stage {stage}");
    let lines: Vec<&str> = content.lines().collect();
    let mut bindings = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let reentry_heading = lines[index]
            .strip_prefix("## ")
            .and_then(|line| line.split_once(' '))
            .is_some_and(|(_, body)| body == record);
        index += 1;
        if !reentry_heading {
            continue;
        }
        while index < lines.len() && !lines[index].starts_with("## ") {
            if let Some(binding) = lines[index].strip_prefix("Re-entry after: ") {
                bindings.push(binding);
            }
            index += 1;
        }
    }
    bindings
}

/// The round budget stated in `dba/02-policies/review/v2.md`: rounds 1-3 run automatically; an
/// ordinary fourth round is rejected. One separately authorized Review re-entry may follow.
const ROUND_BUDGET: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReentryAuthorization {
    pub after_review_id: String,
    pub decision_timestamp: String,
    pub scope: String,
    pub semantic_revision: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceClosureAuthorization {
    pub after_review_id: String,
    pub decision_timestamp: String,
    pub scope: String,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfrastructureRevalidationAuthorization {
    pub after_review_id: String,
    pub decision_timestamp: String,
    pub scope: String,
    pub findings: Vec<String>,
    pub event_evidence_sha: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceReassessment {
    pub after_review_id: String,
    pub findings: Vec<String>,
    pub event_evidence_sha: String,
    pub semantic_sha: String,
    pub substantive: bool,
}

pub fn format_evidence_reassessment_id(
    log_path: &Path,
    feature: &str,
    stage: &str,
) -> Result<String> {
    let content = std::fs::read_to_string(log_path).unwrap_or_default();
    let sequence = count_entries(&content, EVIDENCE_REASSESSMENT_HEADING, feature, stage) + 1;
    Ok(format!(
        "REV__{}__{}__EVIDENCE-REASSESSMENT-{}",
        feature, stage, sequence
    ))
}

fn canonical_event_evidence(
    repo_root: &Path,
    feature: &str,
    event_evidence_sha: &str,
) -> Result<(String, serde_json::Map<String, serde_json::Value>)> {
    anyhow::ensure!(
        event_evidence_sha.len() == 64 && event_evidence_sha.chars().all(|c| c.is_ascii_hexdigit()),
        "event evidence identity must be a SHA-256 digest"
    );
    let store = repo_root.join(".codeos/06-workflow/verifications.jsonl");
    let text = std::fs::read_to_string(&store)
        .with_context(|| format!("read canonical verification store {}", store.display()))?;
    let record = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|record| {
            record.get("workflow").and_then(|v| v.as_str()) == Some("feature")
                && record.get("subject").and_then(|v| v.as_str()) == Some(feature)
                && record.get("verification").and_then(|v| v.as_str())
                    == Some("runtime_event_schema")
                && record
                    .pointer("/details/event_set_sha256")
                    .and_then(|v| v.as_str())
                    == Some(event_evidence_sha)
        })
        .context("no canonical runtime-event verification matches the supplied evidence")?;
    anyhow::ensure!(
        record.get("result").and_then(|v| v.as_str()) == Some("passed"),
        "runtime-event verification did not pass"
    );
    anyhow::ensure!(
        record
            .pointer("/details/event_schema_conformance")
            .and_then(|v| v.as_str())
            == Some("passed"),
        "runtime-event verification does not establish Event Schema conformance"
    );
    let semantic = record
        .pointer("/details/semantic_sha256")
        .and_then(|v| v.as_str())
        .context("runtime-event verification omits semantic_sha256")?
        .to_string();
    let bindings = record
        .get("bindings")
        .and_then(|v| v.as_object())
        .context("runtime-event verification has no bindings")?
        .clone();
    for name in ["implementation_state", "intent", "contract", "event_schema"] {
        anyhow::ensure!(
            bindings.get(name).and_then(|v| v.as_str()).is_some(),
            "runtime-event verification omits {name} binding"
        );
    }
    let artifact = repo_root.join(format!(
        ".codeos/06-workflow/event-evidence/{event_evidence_sha}.jsonl"
    ));
    anyhow::ensure!(
        artifact.is_file(),
        "canonical runtime-event evidence artifact is missing"
    );
    anyhow::ensure!(
        crate::packet::sha256_file(&artifact.to_string_lossy())? == event_evidence_sha,
        "canonical runtime-event evidence artifact hash mismatch"
    );
    Ok((semantic, bindings))
}

pub fn resolve_evidence_reassessment(
    log_path: &Path,
    repo_root: &Path,
    feature: &str,
    stage: &str,
    after_review_id: &str,
    findings: &[String],
    event_evidence_sha: &str,
) -> Result<EvidenceReassessment> {
    anyhow::ensure!(
        !findings.is_empty() && findings.iter().all(|f| !f.trim().is_empty()),
        "evidence reassessment requires named open findings"
    );
    let content = std::fs::read_to_string(log_path)
        .with_context(|| format!("the review log could not be read: {}", log_path.display()))?;
    let marker = format!("Review ID: {after_review_id}");
    let (_, predecessor_tail) = content
        .split_once(&marker)
        .context("the named reassessment predecessor does not exist")?;
    let predecessor = predecessor_tail
        .split("\n## ")
        .next()
        .unwrap_or(predecessor_tail);
    anyhow::ensure!(
        predecessor
            .lines()
            .any(|line| line == "Effective concern: DO NOT ADVANCE"),
        "the predecessor is not an open negative review boundary"
    );

    let mut requested = findings
        .iter()
        .map(|f| f.trim().to_string())
        .collect::<Vec<_>>();
    requested.sort();
    requested.dedup();
    let mut open = predecessor
        .lines()
        .find_map(|line| line.strip_prefix("Authorized findings: "))
        .or_else(|| {
            predecessor
                .lines()
                .find_map(|line| line.strip_prefix("Open evidence findings: "))
        })
        .unwrap_or_default()
        .split(',')
        .map(|f| f.trim().to_string())
        .filter(|f| !f.is_empty())
        .collect::<Vec<_>>();
    open.sort();
    open.dedup();
    anyhow::ensure!(
        requested == open,
        "requested findings do not exactly match the predecessor's open evidence findings"
    );

    // DBA-13 bridge: its terminal record is structurally evidence-only. Later reassessments are
    // eligible only while they remain negative and explicitly carry the same open finding set.
    let legacy_id = format!("REV__{feature}__{stage}__INFRA-REVALIDATION-1");
    let reassessment_predecessor =
        after_review_id.starts_with(&format!("REV__{feature}__{stage}__EVIDENCE-REASSESSMENT-"));
    anyhow::ensure!(after_review_id == legacy_id || reassessment_predecessor, "the predecessor is neither the DBA-13 terminal evidence record nor an evidence reassessment");

    let (semantic_sha, bindings) =
        canonical_event_evidence(repo_root, feature, event_evidence_sha)?;
    let reviewed_store = repo_root.join(".codeos/06-workflow/verifications.jsonl");
    let verification_text = std::fs::read_to_string(reviewed_store)?;
    let binding_match = verification_text
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .any(|record| {
            record.get("subject").and_then(|v| v.as_str()) == Some(feature)
                && record.get("result").and_then(|v| v.as_str()) == Some("passed")
                && record
                    .get("bindings")
                    .and_then(|v| v.as_object())
                    .is_some_and(|prior| {
                        ["implementation_state", "intent", "contract", "event_schema"]
                            .iter()
                            .all(|name| prior.get(*name) == bindings.get(*name))
                    })
        });
    anyhow::ensure!(
        binding_match,
        "event evidence bindings do not match the reviewed verification history"
    );

    for block in content.split("\n## ") {
        let same_findings = block
            .lines()
            .find_map(|line| line.strip_prefix("Open evidence findings: "))
            .map(|listed| {
                let mut values = listed
                    .split(',')
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
                    .collect::<Vec<_>>();
                values.sort();
                values.dedup();
                values == requested
            })
            .unwrap_or(false);
        if same_findings
            && block
                .lines()
                .any(|line| line == format!("Evidence semantic SHA-256: {semantic_sha}"))
            && !block
                .lines()
                .any(|line| line.contains("Assessment status: INCOMPLETE"))
        {
            anyhow::bail!(
                "semantically equivalent evidence was already assessed for these findings"
            );
        }
        anyhow::ensure!(
            !block
                .lines()
                .any(|line| line == "Evidence loop state: CLOSED — SUBSTANTIVE"),
            "a substantive finding closed evidence-loop eligibility"
        );
    }
    Ok(EvidenceReassessment {
        after_review_id: after_review_id.to_string(),
        findings: requested,
        event_evidence_sha: event_evidence_sha.to_string(),
        semantic_sha,
        substantive: false,
    })
}

pub fn format_infrastructure_revalidation_id(feature: &str, stage: &str) -> String {
    format!("REV__{}__{}__INFRA-REVALIDATION-1", feature, stage)
}

fn validate_canonical_event_evidence(
    repo_root: &Path,
    feature: &str,
    event_evidence_sha: &str,
) -> Result<()> {
    let _ = canonical_event_evidence(repo_root, feature, event_evidence_sha)?;
    /* Legacy v8 additionally requires equality with any prior passed verification. */
    anyhow::ensure!(
        event_evidence_sha.len() == 64 && event_evidence_sha.chars().all(|c| c.is_ascii_hexdigit()),
        "event evidence identity must be a SHA-256 digest"
    );
    let store = repo_root.join(".codeos/06-workflow/verifications.jsonl");
    let text = std::fs::read_to_string(&store)
        .with_context(|| format!("read canonical verification store {}", store.display()))?;
    let mut matching: Option<serde_json::Value> = None;
    let mut prior_bindings = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let record: serde_json::Value =
            serde_json::from_str(line).context("malformed canonical verification record")?;
        if record.get("workflow").and_then(|v| v.as_str()) == Some("feature")
            && record.get("subject").and_then(|v| v.as_str()) == Some(feature)
        {
            if record.get("verification").and_then(|v| v.as_str()) == Some("runtime_event_schema")
                && record
                    .pointer("/details/event_set_sha256")
                    .and_then(|v| v.as_str())
                    == Some(event_evidence_sha)
            {
                matching = Some(record.clone());
            } else if record.get("result").and_then(|v| v.as_str()) == Some("passed") {
                if let Some(bindings) = record.get("bindings").and_then(|v| v.as_object()) {
                    prior_bindings.push(bindings.clone());
                }
            }
        }
    }
    let record = matching
        .context("no canonical runtime-event verification matches the authorized evidence")?;
    anyhow::ensure!(
        record.get("result").and_then(|v| v.as_str()) == Some("passed"),
        "runtime-event verification did not pass"
    );
    anyhow::ensure!(
        record
            .pointer("/details/event_schema_conformance")
            .and_then(|v| v.as_str())
            == Some("passed"),
        "runtime-event verification does not establish Event Schema conformance"
    );
    let bindings = record
        .get("bindings")
        .and_then(|v| v.as_object())
        .context("runtime-event verification has no bindings")?;
    for name in ["implementation_state", "intent", "contract", "event_schema"] {
        anyhow::ensure!(
            bindings.get(name).and_then(|v| v.as_str()).is_some(),
            "runtime-event verification omits {name} binding"
        );
    }
    anyhow::ensure!(
        prior_bindings.iter().any(|prior| {
            ["implementation_state", "intent", "contract", "event_schema"]
                .iter()
                .all(|name| prior.get(*name) == bindings.get(*name))
        }),
        "runtime-event evidence bindings do not match the prior reviewed verification history"
    );
    let artifact = repo_root.join(format!(
        ".codeos/06-workflow/event-evidence/{event_evidence_sha}.jsonl"
    ));
    anyhow::ensure!(
        artifact.is_file(),
        "canonical runtime-event evidence artifact is missing"
    );
    anyhow::ensure!(
        crate::packet::sha256_file(&artifact.to_string_lossy())? == event_evidence_sha,
        "canonical runtime-event evidence artifact hash mismatch"
    );
    Ok(())
}

pub fn validate_infrastructure_revalidation_target(
    log_path: &Path,
    repo_root: &Path,
    feature: &str,
    stage: &str,
    after_review_id: &str,
    event_evidence_sha: &str,
) -> Result<()> {
    let content = std::fs::read_to_string(log_path)?;
    anyhow::ensure!(
        after_review_id == format_evidence_closure_id(feature, stage),
        "terminal revalidation must follow the matching EVIDENCE-CLOSURE-1"
    );
    let marker = format!("Review ID: {after_review_id}");
    let (_, tail) = content
        .split_once(&marker)
        .context("the named EVIDENCE-CLOSURE-1 does not exist")?;
    let block = tail.split("\n## ").next().unwrap_or(tail);
    anyhow::ensure!(
        block
            .lines()
            .any(|line| line == "Effective concern: DO NOT ADVANCE"),
        "EVIDENCE-CLOSURE-1 is not a negative exhausted boundary"
    );
    anyhow::ensure!(
        count_entries(
            &content,
            INFRASTRUCTURE_REVALIDATION_HEADING,
            feature,
            stage
        ) == 0,
        "Infrastructure Repair Revalidation has already been consumed"
    );
    validate_canonical_event_evidence(repo_root, feature, event_evidence_sha)
}

pub fn resolve_infrastructure_revalidation_authorization(
    log_path: &Path,
    repo_root: &Path,
    feature: &str,
    stage: &str,
    after_review_id: &str,
    findings: &[String],
    event_evidence_sha: &str,
) -> Result<InfrastructureRevalidationAuthorization> {
    validate_infrastructure_revalidation_target(
        log_path,
        repo_root,
        feature,
        stage,
        after_review_id,
        event_evidence_sha,
    )?;
    let content = std::fs::read_to_string(log_path)?;
    let tail = content
        .split_once(&format!("Review ID: {after_review_id}"))
        .map(|(_, tail)| tail)
        .unwrap_or_default();
    let suffix = format!(" HUMAN DECISION — {feature} — Stage {stage}");
    let mut selected = None;
    for block in tail.split("\n## ").skip(1) {
        let mut lines = block.lines();
        let heading = lines.next().unwrap_or_default();
        if heading.ends_with(&suffix) {
            let lines: Vec<_> = lines.collect();
            let decision = lines
                .iter()
                .find_map(|line| line.strip_prefix("Decision: "));
            if decision == Some("AUTHORIZE_INFRASTRUCTURE_REVALIDATION")
                && lines
                    .iter()
                    .find_map(|line| line.strip_prefix("Infrastructure revalidation after: "))
                    == Some(after_review_id)
                && lines
                    .iter()
                    .find_map(|line| line.strip_prefix("Infrastructure repair evidence: "))
                    == Some(event_evidence_sha)
            {
                let scope = lines
                    .iter()
                    .find_map(|line| line.strip_prefix("Reason/next: "))
                    .unwrap_or_default()
                    .to_string();
                let authorized = lines
                    .iter()
                    .find_map(|line| line.strip_prefix("Authorized findings: "))
                    .unwrap_or_default()
                    .split(',')
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
                    .collect::<Vec<_>>();
                selected = Some((
                    heading
                        .strip_suffix(&suffix)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                    scope,
                    authorized,
                ));
            } else if decision == Some("STOP") && selected.is_some() {
                selected = None;
            }
        }
    }
    let (timestamp, scope, authorized) = selected.context("no active Infrastructure Repair Revalidation authorization matches the boundary and evidence")?;
    let mut requested = findings
        .iter()
        .map(|v| v.trim().to_string())
        .collect::<Vec<_>>();
    let mut authorized_sorted = authorized.clone();
    requested.sort();
    requested.dedup();
    authorized_sorted.sort();
    authorized_sorted.dedup();
    anyhow::ensure!(
        requested == authorized_sorted && !requested.is_empty(),
        "requested findings do not exactly match the authorization"
    );
    Ok(InfrastructureRevalidationAuthorization {
        after_review_id: after_review_id.to_string(),
        decision_timestamp: timestamp,
        scope,
        findings: authorized,
        event_evidence_sha: event_evidence_sha.to_string(),
    })
}

pub fn validate_evidence_closure_target(
    log_path: &Path,
    feature: &str,
    stage: &str,
    after_review_id: &str,
) -> Result<()> {
    let content = std::fs::read_to_string(log_path)
        .with_context(|| format!("the review log could not be read: {}", log_path.display()))?;
    anyhow::ensure!(
        after_review_id == format_reentry_id(feature, stage),
        "{after_review_id} is not the REENTRY-1 for {feature} at stage {stage}"
    );
    anyhow::ensure!(
        content.contains(&format!("Review ID: {after_review_id}")),
        "{after_review_id} does not exist"
    );
    let marker = format!("Review ID: {after_review_id}");
    let tail = content
        .split_once(&marker)
        .map(|(_, t)| t)
        .unwrap_or_default();
    let block = tail.split("\n## ").next().unwrap_or(tail);
    anyhow::ensure!(
        block
            .lines()
            .any(|l| l == "Effective concern: DO NOT ADVANCE"),
        "{after_review_id} did not end in DO NOT ADVANCE"
    );
    anyhow::ensure!(
        count_entries(&content, EVIDENCE_CLOSURE_HEADING, feature, stage) == 0,
        "the single evidence closure for {feature} at stage {stage} has already been consumed"
    );
    Ok(())
}

pub fn resolve_evidence_closure_authorization(
    log_path: &Path,
    feature: &str,
    stage: &str,
    after_review_id: &str,
    findings: &[String],
) -> Result<EvidenceClosureAuthorization> {
    validate_evidence_closure_target(log_path, feature, stage, after_review_id)?;
    let content = std::fs::read_to_string(log_path)?;
    let marker = format!("Review ID: {after_review_id}");
    let tail = content
        .split_once(&marker)
        .map(|(_, t)| t)
        .unwrap_or_default();
    let human_suffix = format!(" HUMAN DECISION — {feature} — Stage {stage}");
    let mut selected = None;
    for block in tail.split("\n## ").skip(1) {
        let mut lines = block.lines();
        let heading = lines.next().unwrap_or_default();
        if heading.ends_with(&human_suffix) {
            let all: Vec<_> = lines.collect();
            let decision = all.iter().find_map(|l| l.strip_prefix("Decision: "));
            let binding = all
                .iter()
                .find_map(|l| l.strip_prefix("Evidence closure after: "));
            if decision == Some("AUTHORIZE_EVIDENCE_CLOSURE") && binding == Some(after_review_id) {
                let scope = all
                    .iter()
                    .find_map(|l| l.strip_prefix("Reason/next: "))
                    .unwrap_or_default();
                let listed = all
                    .iter()
                    .find_map(|l| l.strip_prefix("Authorized findings: "))
                    .unwrap_or_default();
                selected = Some((
                    heading
                        .strip_suffix(&human_suffix)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                    scope.to_string(),
                    listed
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>(),
                ));
            } else if decision == Some("STOP") && selected.is_some() {
                selected = None;
            }
        }
        if heading.ends_with(&format!(
            " {EVIDENCE_CLOSURE_HEADING} — {feature} — Stage {stage}"
        )) {
            break;
        }
    }
    let (timestamp, scope, authorized) = selected.ok_or_else(|| {
        anyhow::anyhow!("no active AUTHORIZE_EVIDENCE_CLOSURE bound to {after_review_id}")
    })?;
    let mut requested = findings
        .iter()
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
    let mut authorized_sorted = authorized.clone();
    requested.sort();
    requested.dedup();
    authorized_sorted.sort();
    authorized_sorted.dedup();
    anyhow::ensure!(
        requested == authorized_sorted,
        "requested findings do not exactly match the authorized findings"
    );
    Ok(EvidenceClosureAuthorization {
        after_review_id: after_review_id.to_string(),
        decision_timestamp: timestamp,
        scope,
        findings: authorized,
    })
}

pub fn format_evidence_closure_id(feature: &str, stage: &str) -> String {
    format!("REV__{}__{}__EVIDENCE-CLOSURE-1", feature, stage)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReentryDecisionSemantics {
    /// Contract v4 has no rule for ordering conflicting post-R3 decisions. One decision can
    /// authorize re-entry; more than one is ambiguous and therefore fails closed.
    LegacySingleDecision,
    /// Contract v6 makes physical append order authoritative. A bound AUTHORIZE_REENTRY creates
    /// review-control authority; a later STOP revokes it before consumption. Stage dispositions
    /// neither create nor consume that authority. Timestamps remain evidence only.
    CanonicalLogOrder,
    /// Contract v11 retains canonical decision ordering while scoping Stage 8 consumption to the
    /// target review cycle. `None` is the legacy revision-0 cycle.
    PackageBoundCanonicalLogOrder {
        current_semantic_revision: Option<u64>,
    },
}

#[cfg(test)]
pub fn validate_reentry_authorization_target(
    log_path: &Path,
    feature: &str,
    stage: &str,
    after_review_id: &str,
) -> Result<()> {
    validate_reentry_authorization_target_with_semantics(
        log_path,
        feature,
        stage,
        after_review_id,
        ReentryDecisionSemantics::CanonicalLogOrder,
    )
}

pub fn validate_reentry_authorization_target_with_semantics(
    log_path: &Path,
    feature: &str,
    stage: &str,
    after_review_id: &str,
    semantics: ReentryDecisionSemantics,
) -> Result<()> {
    let content = std::fs::read_to_string(log_path)
        .with_context(|| format!("the review log could not be read: {}", log_path.display()))?;
    let target = parse_ordinary_review_id(after_review_id, feature, stage).ok_or_else(|| {
        anyhow::anyhow!("{after_review_id} is not a canonical ordinary review ID for {feature} at stage {stage}")
    })?;
    anyhow::ensure!(
        ordinary_review_ids(&content, feature, stage).contains(&after_review_id),
        "{after_review_id} is not an ordinary review record for {feature} at stage {stage}"
    );
    anyhow::ensure!(
        target.round == ROUND_BUDGET,
        "{after_review_id} is round {}; authorization is available only for R{ROUND_BUDGET}",
        target.round
    );
    if let ReentryDecisionSemantics::PackageBoundCanonicalLogOrder {
        current_semantic_revision,
    } = semantics
    {
        if stage == "8" {
            anyhow::ensure!(
                target.semantic_revision == current_semantic_revision,
                "{after_review_id} belongs to semantic revision {:?}, but the current approved package is revision {:?}",
                target.semantic_revision,
                current_semantic_revision
            );
            let reentry_id = format_package_reentry_id(feature, stage, target.semantic_revision);
            anyhow::ensure!(
                !review_ids_for_kind(&content, REENTRY_REVIEW_HEADING, feature, stage)
                    .contains(&reentry_id.as_str())
                    && !reentry_bindings(&content, feature, stage)
                        .contains(&after_review_id),
                "the single re-entry for {feature} at stage {stage} in this review cycle has already been consumed"
            );
            return Ok(());
        }
    }
    anyhow::ensure!(
        count_entries(&content, REENTRY_REVIEW_HEADING, feature, stage) == 0,
        "the single re-entry for {feature} at stage {stage} has already been consumed"
    );
    Ok(())
}

/// Resolve the one exceptional post-budget review authorized by the human decision following R3.
/// The authorization remains in the append-only history; this derives permission without adding a
/// second authority store.
pub fn resolve_reentry_authorization(
    log_path: &Path,
    feature: &str,
    stage: &str,
    after_review_id: &str,
    semantics: ReentryDecisionSemantics,
) -> Result<ReentryAuthorization> {
    let content = std::fs::read_to_string(log_path).with_context(|| {
        format!(
            "--reentry-after {after_review_id} was given but the review log could not be read: {}",
            log_path.display()
        )
    })?;
    validate_reentry_authorization_target_with_semantics(
        log_path,
        feature,
        stage,
        after_review_id,
        semantics,
    )?;
    let target = parse_ordinary_review_id(after_review_id, feature, stage)
        .expect("validated ordinary review ID must remain parseable");

    let review_marker = format!("Review ID: {after_review_id}");
    let review_pos = content
        .find(&review_marker)
        .ok_or_else(|| anyhow::anyhow!("could not locate {after_review_id} in the review log"))?;
    let tail = &content[review_pos + review_marker.len()..];
    let human_suffix = format!(" HUMAN DECISION — {feature} — Stage {stage}");
    let mut decisions = Vec::new();
    for block in tail.split("\n## ").skip(1) {
        let mut lines = block.lines();
        let heading = lines.next().unwrap_or_default();
        if heading.ends_with(&human_suffix) {
            let decision = lines
                .clone()
                .find_map(|line| line.strip_prefix("Decision: "));
            let binding = lines
                .clone()
                .find_map(|line| line.strip_prefix("Re-entry after: "));
            let reason = lines.find_map(|line| line.strip_prefix("Reason/next: "));
            let timestamp = heading
                .strip_suffix(&human_suffix)
                .unwrap_or_default()
                .trim()
                .to_string();
            decisions.push((
                decision.map(str::to_string),
                reason.map(str::to_string),
                binding.map(str::to_string),
                timestamp,
            ));
        }
        if heading.ends_with(&format!(" {REVIEW_HEADING} — {feature} — Stage {stage}"))
            || heading.ends_with(&format!(
                " {REENTRY_REVIEW_HEADING} — {feature} — Stage {stage}"
            ))
        {
            break;
        }
    }

    let selected = match semantics {
        ReentryDecisionSemantics::LegacySingleDecision => match decisions.as_slice() {
            [] => anyhow::bail!("no human re-entry authorization follows {after_review_id}"),
            [decision] => decision,
            _ => anyhow::bail!(
                "multiple human decisions follow {after_review_id}; the selected reviewer contract does not define their ordering"
            ),
        },
        ReentryDecisionSemantics::CanonicalLogOrder
        | ReentryDecisionSemantics::PackageBoundCanonicalLogOrder { .. } => {
            let authorization_index = decisions.iter().rposition(|(decision, _, binding, _)| {
                decision.as_deref() == Some("AUTHORIZE_REENTRY")
                    && binding.as_deref() == Some(after_review_id)
            }).ok_or_else(|| anyhow::anyhow!(
                "no AUTHORIZE_REENTRY bound to {after_review_id} follows that review"
            ))?;
            if decisions[authorization_index + 1..]
                .iter()
                .any(|(decision, _, _, _)| decision.as_deref() == Some("STOP"))
            {
                anyhow::bail!(
                    "a STOP appended after the authorization denies re-entry after {after_review_id}"
                );
            }
            &decisions[authorization_index]
        }
    };

    let (decision, reason, _binding, timestamp) = selected;
    let explicit = reason
        .as_deref()
        .map(|value| value.to_ascii_lowercase().contains("re-entry authorized"))
        .unwrap_or(false);
    let authorized_decision = match semantics {
        ReentryDecisionSemantics::LegacySingleDecision => {
            decision.as_deref() == Some("REQUEST_CHANGES") && explicit
        }
        ReentryDecisionSemantics::CanonicalLogOrder
        | ReentryDecisionSemantics::PackageBoundCanonicalLogOrder { .. } => {
            decision.as_deref() == Some("AUTHORIZE_REENTRY")
                && reason.as_deref().is_some_and(|r| !r.trim().is_empty())
        }
    };
    if !authorized_decision {
        anyhow::bail!("the controlling human decision following {after_review_id} does not explicitly authorize review re-entry");
    }
    Ok(ReentryAuthorization {
        after_review_id: after_review_id.to_string(),
        decision_timestamp: timestamp.clone(),
        scope: reason.as_deref().unwrap().trim().to_string(),
        semantic_revision: target.semantic_revision,
    })
}

pub fn format_reentry_id(feature: &str, stage: &str) -> String {
    format!("REV__{}__{}__REENTRY-1", feature, stage)
}

pub fn format_package_reentry_id(
    feature: &str,
    stage: &str,
    semantic_revision: Option<u64>,
) -> String {
    match semantic_revision {
        Some(revision) => format!("REV__{feature}__{stage}__SR{revision}__REENTRY-1"),
        None => format_reentry_id(feature, stage),
    }
}

/// Resolve the round for a review that explicitly `--continues` an existing one, instead of
/// deriving it by counting headings in whatever log file this invocation happens to read. Exists
/// because that heading-count derivation is scoped to a single log file, so a review that runs on
/// an isolated branch or worktree — whose local log predates its true predecessors — mis-derives
/// round 1 for what is substantively a later round (the demonstrated cause of UPG-0074's
/// `REV__F-0004__8__R1` recorded where `R3` was correct, requiring a manual annotation to explain).
///
/// The referenced predecessor is validated against the log, not trusted from `continues` alone:
/// same feature and stage (one lookup, scoped by the same heading suffix `compute_review_round`
/// already matches on), the entry actually exists, its own predecessor chain (if any) terminates
/// without cycling back on itself within the round budget, and the resolved round does not exceed
/// that budget — a stricter check than the ordinary path has today, appropriate here because using
/// `--continues` is a deliberate act with a moment to check, unlike the default derivation.
pub fn resolve_continued_round(
    log_path: &Path,
    feature: &str,
    stage: &str,
    continues: &str,
) -> Result<u32> {
    let content = std::fs::read_to_string(log_path).with_context(|| {
        format!(
            "--continues {continues} was given but the review log could not be read: {}",
            log_path.display()
        )
    })?;

    // The round to use is the referenced predecessor's own round, by its position among matching
    // headings — never wherever its ancestry eventually terminates. Its own Continues link, if
    // any, is walked below purely to verify the chain, not to pick a different round.
    let (predecessor_round, mut next) = find_predecessor_entry(&content, feature, stage, continues)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "--continues {continues} does not resolve to an existing review for {feature} \
                 at stage {stage} — the referenced ID must be a prior REVIEW entry for the same \
                 feature and stage"
            )
        })?;

    // Walk the predecessor's own ancestry to confirm it terminates without cycling back on
    // itself, bounded by the round budget — a chain that does not terminate within that many hops
    // is invalid regardless of whether it is a literal cycle.
    let mut seen = std::collections::HashSet::new();
    seen.insert(continues.to_string());
    let mut hops = 0u32;
    while let Some(ancestor_id) = next {
        hops += 1;
        if hops >= ROUND_BUDGET || !seen.insert(ancestor_id.clone()) {
            anyhow::bail!(
                "--continues {continues} does not resolve within the {ROUND_BUDGET}-round budget \
                 — its predecessor chain is a cycle, or longer than the budget allows"
            );
        }
        let (_, ancestor_next) = find_predecessor_entry(&content, feature, stage, &ancestor_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "--continues {continues} has a broken predecessor chain: {ancestor_id} does \
                     not resolve to an existing review for {feature} at stage {stage}"
                )
            })?;
        next = ancestor_next;
    }

    let next_round = predecessor_round + 1;
    if next_round > ROUND_BUDGET {
        anyhow::bail!(
            "--continues {continues} resolves to round {predecessor_round}; continuing would \
             exceed the {ROUND_BUDGET}-round budget — a human decision is required instead"
        );
    }
    Ok(next_round)
}

/// Locate the REVIEW entry for `review_id` among entries matching `feature`+`stage`, returning its
/// round — its 1-based position among those entries, the same derivation `compute_review_round`
/// uses — and its own `Continues:` predecessor link, if it has one.
fn find_predecessor_entry(
    content: &str,
    feature: &str,
    stage: &str,
    review_id: &str,
) -> Option<(u32, Option<String>)> {
    let heading_suffix = format!(" {} — {} — Stage {}", REVIEW_HEADING, feature, stage);
    let lines: Vec<&str> = content.lines().collect();
    let mut round = 0u32;
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("## ") && lines[i].ends_with(&heading_suffix) {
            round += 1;
            let mut j = i + 1;
            let mut this_id: Option<&str> = None;
            let mut this_continues: Option<String> = None;
            while j < lines.len() && !lines[j].starts_with("## ") {
                if let Some(id) = lines[j].strip_prefix("Review ID: ") {
                    this_id = Some(id.trim());
                }
                if let Some(c) = lines[j].strip_prefix("Continues: ") {
                    this_continues = Some(c.trim().to_string());
                }
                j += 1;
            }
            if this_id == Some(review_id) {
                return Some((round, this_continues));
            }
            i = j;
        } else {
            i += 1;
        }
    }
    None
}

/// Sequence number for the next external assessment of `feature`+`stage`. Counted separately from
/// review rounds and never mixed with them.
pub fn compute_assessment_sequence(log_path: &Path, feature: &str, stage: &str) -> Result<u32> {
    let content = match std::fs::read_to_string(log_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(1),
        Err(e) => {
            return Err(e).with_context(|| {
                format!(
                    "could not read review log to compute assessment sequence: {}",
                    log_path.display()
                )
            });
        }
    };
    Ok(count_entries(&content, EXTERNAL_ASSESSMENT_HEADING, feature, stage) + 1)
}

/// Format the `EXT__<feature>__<stage>__A<N>` external-assessment id. Deliberately not the
/// `REV__…__R<N>` shape: an external assessment is not a round of review.
pub fn format_assessment_id(feature: &str, stage: &str, sequence: u32) -> String {
    format!("EXT__{}__{}__A{}", feature, stage, sequence)
}

/// Format the `REV__<feature>__<stage>__R<N>` review id. The stage is used verbatim — no
/// numeric-stage-to-`S<N>` conversion — because `S<N>` (from `UPG-0001`) was defined only for
/// self-dev steps 1-4 and has no mapping for downstream DBA stage ids.
pub fn format_review_id(feature: &str, stage: &str, round: u32) -> String {
    format!("REV__{}__{}__R{}", feature, stage, round)
}

pub fn format_semantic_review_id(
    feature: &str,
    stage: &str,
    semantic_revision: u64,
    round: u32,
) -> String {
    format!("REV__{feature}__{stage}__SR{semantic_revision}__R{round}")
}

/// Append a REVIEW entry to the log. Uses a temp-file + rename for atomicity.
#[allow(clippy::too_many_arguments)]
pub fn append_review(
    log_path: &Path,
    review_id: &str,
    packet: &ReviewPacket,
    raw: &ReviewerRun,
    parsed: &ParsedReview,
    assessment_file: &Path,
    assessment_hash: &str,
    packet_saved: &Path,
    packet_hash: &str,
    continues: Option<&str>,
    reentry: Option<&ReentryAuthorization>,
    closure: Option<&EvidenceClosureAuthorization>,
    infrastructure: Option<&InfrastructureRevalidationAuthorization>,
    reassessment: Option<&EvidenceReassessment>,
    terminal_closure: Option<&crate::terminal_closure::TerminalClosureExecution>,
) -> Result<()> {
    ensure_log_exists(log_path)?;

    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let dirty_text = if packet.workspace_dirty {
        "true"
    } else {
        "false"
    };
    let coverage_note_suffix = if !parsed.coverage_note.is_empty() {
        format!("; note: {}", parsed.coverage_note)
    } else {
        String::new()
    };

    let structural_closure_failure =
        terminal_closure.is_some() && review_id.contains("__TERMINAL-CLOSURE-FAILURE-");
    let mut entry = String::new();
    entry.push('\n');
    let kind = if raw.is_external() {
        EXTERNAL_ASSESSMENT_HEADING
    } else if infrastructure.is_some() {
        INFRASTRUCTURE_REVALIDATION_HEADING
    } else if terminal_closure.is_some() {
        if structural_closure_failure {
            "TERMINAL REVIEW CLOSURE EXECUTION FAILURE"
        } else {
            TERMINAL_CLOSURE_HEADING
        }
    } else if reassessment.is_some() {
        EVIDENCE_REASSESSMENT_HEADING
    } else if closure.is_some() {
        EVIDENCE_CLOSURE_HEADING
    } else if reentry.is_some() {
        REENTRY_REVIEW_HEADING
    } else {
        REVIEW_HEADING
    };
    entry.push_str(&format!(
        "## {} {} — {} — Stage {}\n",
        ts, kind, packet.feature, packet.stage
    ));
    entry.push_str(&format!(
        "{} ID: {}\n",
        if raw.is_external() {
            "Assessment"
        } else {
            "Review"
        },
        review_id
    ));
    if let Some(revision) = parse_ordinary_review_id(review_id, &packet.feature, &packet.stage)
        .and_then(|id| id.semantic_revision)
    {
        entry.push_str(&format!("Semantic revision: {revision}\n"));
    }
    if let Some(predecessor) = continues {
        entry.push_str(&format!("Continues: {}\n", predecessor));
    }
    if let Some(auth) = reentry {
        entry.push_str(&format!("Re-entry after: {}\n", auth.after_review_id));
        entry.push_str(&format!(
            "Human authorization: {} HUMAN DECISION\n",
            auth.decision_timestamp
        ));
        entry.push_str(&format!("Authorized scope: {}\n", auth.scope));
    }
    if let Some(auth) = closure {
        entry.push_str(&format!(
            "Evidence closure after: {}\n",
            auth.after_review_id
        ));
        entry.push_str(&format!(
            "Human authorization: {} HUMAN DECISION\n",
            auth.decision_timestamp
        ));
        entry.push_str(&format!("Authorized scope: {}\n", auth.scope));
        entry.push_str(&format!(
            "Authorized findings: {}\n",
            auth.findings.join(", ")
        ));
    }
    if let Some(auth) = infrastructure {
        entry.push_str(&format!(
            "Infrastructure revalidation after: {}\n",
            auth.after_review_id
        ));
        entry.push_str(&format!(
            "Human authorization: {} HUMAN DECISION\n",
            auth.decision_timestamp
        ));
        entry.push_str(&format!("Authorized scope: {}\n", auth.scope));
        entry.push_str(&format!(
            "Authorized findings: {}\n",
            auth.findings.join(", ")
        ));
        entry.push_str(&format!(
            "Infrastructure repair evidence: {}\n",
            auth.event_evidence_sha
        ));
    }
    if let Some(item) = reassessment {
        entry.push_str(&format!(
            "Evidence reassessment after: {}\n",
            item.after_review_id
        ));
        entry.push_str(&format!(
            "Open evidence findings: {}\n",
            item.findings.join(", ")
        ));
        entry.push_str(&format!(
            "Canonical event evidence: {}\n",
            item.event_evidence_sha
        ));
        entry.push_str(&format!(
            "Evidence semantic SHA-256: {}\n",
            item.semantic_sha
        ));
        let state = if parsed.assessment_status != "COMPLETE" {
            "OPEN — INCOMPLETE"
        } else if item.substantive {
            "CLOSED — SUBSTANTIVE"
        } else if parsed.effective_concern != "DO NOT ADVANCE" {
            "CLOSED — EVIDENCE SATISFIED"
        } else if parsed.effective_concern == "DO NOT ADVANCE" {
            "OPEN — EVIDENCE INSUFFICIENT"
        } else {
            "CLOSED — SUBSTANTIVE"
        };
        entry.push_str(&format!("Evidence loop state: {state}\n"));
        if let Some(materiality) = &parsed.evidence_materiality {
            entry.push_str(&format!("Evidence materiality: {materiality}\n"));
        }
    }
    if let Some(item) = terminal_closure {
        entry.push_str(&format!(
            "Terminal closure after: {}\n",
            item.scope.scope.terminal_review_id
        ));
        entry.push_str(&format!(
            "Human authorization: {} HUMAN DECISION\n",
            item.decision_timestamp
        ));
        entry.push_str(&format!("Authorized scope: {}\n", item.reason));
        entry.push_str(&format!(
            "Terminal closure scope SHA-256: {}\n",
            item.scope.sha256
        ));
        entry.push_str(&format!(
            "Repair manifest SHA-256: {}\n",
            item.repair.sha256
        ));
        entry.push_str(&format!(
            "Authorized conditions: {}\n",
            item.repair.condition_ids.join(", ")
        ));
        if let Some(sha) = &item.assessability_correction_sha256 {
            entry.push_str("Evidence change classification: ASSESSABILITY_CORRECTION\n");
            entry.push_str(&format!("Assessability correction SHA-256: {sha}\n"));
        }
        if !item.input_sha256.is_empty() {
            entry.push_str(&format!("Closure input SHA-256: {}\n", item.input_sha256));
            if structural_closure_failure {
                entry.push_str("Closure execution status: STRUCTURAL FAILURE\n");
                entry.push_str(&format!(
                    "Retry status: {}\n",
                    if item.attempt == 1 {
                        "ONE BOUNDED BRIDGE RETRY AVAILABLE"
                    } else {
                        "EXHAUSTED"
                    }
                ));
            } else {
                entry.push_str(&format!(
                    "Closure semantic disposition: {}\n",
                    if parsed.effective_concern == "TERMINAL REVIEW CLOSED — PROGRESSION PERMITTED"
                    {
                        "CLOSED"
                    } else {
                        "OPEN"
                    }
                ));
            }
        }
    }
    entry.push_str(&format!(
        "Base: {}  Review: {}  Branch: {}\n",
        packet.base_sha, packet.review_sha, packet.branch
    ));
    entry.push_str(&format!("Diff-hash: {}\n", packet.diff_hash));
    entry.push_str(&format!("Reviewer: {}\n", raw.reviewer_field()));
    match &raw.source {
        RunSource::Codex {
            reconnect_count,
            effort,
            ..
        } => {
            entry.push_str(&format!(
                "Effort: {}   Wall time: {}ms   Reconnects: {}\n",
                effort, raw.elapsed_ms, reconnect_count
            ));
            if !structural_closure_failure {
                entry.push_str(&format!("Codex concern: {}\n", parsed.codex_concern));
            }
        }
        RunSource::External { .. } => {
            entry.push_str("Source: external — advisory findings only; does NOT satisfy a required review round.\n");
            if !structural_closure_failure {
                entry.push_str(&format!("Reported concern: {}\n", parsed.codex_concern));
            }
        }
    }
    if !structural_closure_failure {
        entry.push_str(&format!(
            "Effective concern: {}\n",
            parsed.effective_concern
        ));
        entry.push_str(&format!("Evidence: {}\n", parsed.evidence));
    }
    entry.push_str(&format!(
        "Parse status: {}   Assessment status: {}\n",
        parsed.parse_status, parsed.assessment_status
    ));
    if !parsed.incomplete_reason.is_empty() {
        entry.push_str(&format!(
            "INCOMPLETE — {} — advancement prohibited on this record.\n",
            parsed.incomplete_reason
        ));
    }
    if !structural_closure_failure && !parsed.highest_impact_uncertainty.is_empty() {
        entry.push_str(&format!(
            "Highest-impact uncertainty: {}\n",
            parsed.highest_impact_uncertainty
        ));
    }
    entry.push_str(&format!(
        "Coverage: {}; redactions: {}; workspace_dirty: {}{}\n",
        packet.coverage_state.as_str(),
        packet.redaction_count,
        dirty_text,
        coverage_note_suffix
    ));
    if !structural_closure_failure {
        entry.push_str(&format!(
            "Log summary: {}\n",
            parsed.summary_line.trim_start_matches("LOG SUMMARY: ")
        ));
    }
    entry.push_str(&format!(
        "Full assessment: {} (sha256:{})\n",
        assessment_file.display(),
        assessment_hash
    ));
    entry.push_str(&format!(
        "Reviewed packet: {} (sha256:{})\n",
        packet_saved.display(),
        packet_hash
    ));
    if packet.secret_flag
        || packet.coverage_state.as_str() == "CRITICAL_OMISSION"
        || packet.coverage_state.as_str() == "EMPTY_PACKET"
    {
        let excluded: Vec<String> = packet
            .excluded_paths
            .iter()
            .map(|(p, _, _)| p.clone())
            .collect();
        entry.push_str(&format!(
            "Coverage gap: {} — excluded/redacted [{}] — MANUAL SECURITY REVIEW REQUIRED\n",
            packet.coverage_state.as_str(),
            excluded.join(", ")
        ));
    }
    if raw.is_external() {
        entry.push_str("Progression: unchanged — a required review still needs a Codex-backed review round or a recorded Review Waiver.\n");
    } else {
        entry.push_str(&format!("Human decision: (append with: codeos-reviewer decision {} {} <DECISION> \"<reason>\")\n",
            packet.feature, packet.stage));
    }

    append_to_log(log_path, &entry)
}

/// Find the most recent assessment for `feature+stage`, parse its frontmatter, re-hash the
/// packet file, compare commit + hashes, emit advisory warnings, and return provenance.
///
/// - `Ok(None)` — no assessment file exists (AC-6: legacy path allowed, no coverage gate)
/// - `Ok(Some(...))` — assessment parsed and provenance loaded (full path)
/// - `Err(...)` — assessment file **exists** but could not be read or parsed (fail-closed:
///   caller must either stop or require --override from the human)
pub fn load_decision_provenance(
    feature: &str,
    stage: &str,
    override_reason: Option<&str>,
    codex_dir: &Path,
    repo_root: &Path,
) -> anyhow::Result<Option<DecisionProvenance>> {
    let pattern = format!("{}-stage-{}-", feature, stage);
    let entries: Vec<_> = match std::fs::read_dir(codex_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.contains(&pattern) && !name.ends_with(".packet.txt")
            })
            .collect(),
        Err(_) => return Ok(None), // codex dir absent → no assessment → legacy path
    };

    let latest = match entries.into_iter().max_by_key(|e| e.file_name()) {
        Some(e) => e,
        None => return Ok(None), // no matching assessment → legacy path
    };
    let path = latest.path();

    // Past this point: an assessment file EXISTS. Failures are fail-closed.
    let content = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "assessment file exists but could not be read: {}",
            path.display()
        )
    })?;

    let fm = parse_assessment_frontmatter(&content).ok_or_else(|| {
        anyhow::anyhow!(
            "assessment file exists but provenance frontmatter could not be parsed \
             (missing coverage_state or review_commit): {}",
            path.display()
        )
    })?;

    let head_sha = git_head_sha(repo_root).unwrap_or_else(|_| {
        // HEAD unknown is advisory only.
        eprintln!(
            "warning: could not determine current HEAD (git rev-parse failed) \
                   — commit drift check skipped"
        );
        "(unknown)".to_string()
    });

    // Re-hash the saved packet file. Unverifiable cases are recorded explicitly.
    let packet_sha_actual = if fm.reviewed_packet.is_empty() {
        eprintln!(
            "warning: assessment has no reviewed_packet path — packet integrity unverifiable"
        );
        "UNVERIFIABLE:no-packet-path".to_string()
    } else {
        let packet_full = codex_dir.join(&fm.reviewed_packet);
        if !packet_full.exists() {
            eprintln!(
                "warning: packet file not found — packet integrity unverifiable: {}",
                packet_full.display()
            );
            "UNVERIFIABLE:packet-missing".to_string()
        } else {
            match crate::packet::sha256_file(packet_full.to_str().unwrap_or("")) {
                Ok(sha) => sha,
                Err(e) => {
                    eprintln!(
                        "warning: could not hash packet file ({}) — packet integrity unverifiable",
                        e
                    );
                    "UNVERIFIABLE:hash-error".to_string()
                }
            }
        }
    };

    // Advisory warning: no stored sha in assessment (AC-6c — does not block).
    if fm.reviewed_packet_sha256.is_empty() && !packet_sha_actual.starts_with("UNVERIFIABLE:") {
        eprintln!("warning: assessment has no reviewed_packet_sha256 — packet integrity unverifiable (no stored sha)");
    }
    // Advisory warning: packet hash mismatch (AC-4 — does not block).
    else if !fm.reviewed_packet_sha256.is_empty()
        && !packet_sha_actual.starts_with("UNVERIFIABLE:")
        && packet_sha_actual != fm.reviewed_packet_sha256
    {
        eprintln!(
            "warning: packet hash mismatch — stored {} / recomputed {}",
            short_sha(&fm.reviewed_packet_sha256),
            short_sha(&packet_sha_actual)
        );
    }

    // Advisory warning: commit drift (AC-5 — does not block; HEAD_UNKNOWN also advisory).
    if !fm.review_commit.is_empty() && head_sha != "(unknown)" && head_sha != fm.review_commit {
        eprintln!(
            "warning: HEAD has moved since review — review_commit {} / current HEAD {}",
            short_sha(&fm.review_commit),
            short_sha(&head_sha)
        );
    }

    Ok(Some(DecisionProvenance {
        assessment_path: path.display().to_string(),
        review_commit: fm.review_commit,
        head_sha,
        packet_sha_stored: fm.reviewed_packet_sha256,
        packet_sha_actual,
        coverage_state: fm.coverage_state,
        override_rationale: override_reason.map(|s| s.to_string()),
        provenance_error: None,
    }))
}

struct AssessmentFrontmatter {
    review_commit: String,
    coverage_state: String,
    reviewed_packet: String,
    reviewed_packet_sha256: String,
}

fn parse_assessment_frontmatter(content: &str) -> Option<AssessmentFrontmatter> {
    let mut in_fm = false;
    let mut review_commit = String::new();
    let mut coverage_state = String::new();
    let mut reviewed_packet = String::new();
    let mut reviewed_packet_sha256 = String::new();

    for line in content.lines() {
        if line == "---" {
            if !in_fm {
                in_fm = true;
                continue;
            } else {
                break;
            }
        }
        if !in_fm {
            continue;
        }
        let t = line.trim();
        if let Some(v) = t.strip_prefix("review_commit: ") {
            review_commit = v.to_string();
        } else if let Some(v) = t.strip_prefix("coverage_state: ") {
            coverage_state = v.to_string();
        } else if let Some(v) = t.strip_prefix("reviewed_packet: ") {
            reviewed_packet = v.to_string();
        } else if let Some(v) = t.strip_prefix("reviewed_packet_sha256: ") {
            reviewed_packet_sha256 = v.to_string();
        }
    }

    // Both fields are required. Partial provenance (one present, one absent) is treated
    // as malformed — the caller turns None into Err and fail-closes (AC-6b).
    if review_commit.is_empty() || coverage_state.is_empty() {
        return None;
    }
    Some(AssessmentFrontmatter {
        review_commit,
        coverage_state,
        reviewed_packet,
        reviewed_packet_sha256,
    })
}

fn short_sha(sha: &str) -> &str {
    &sha[..sha.len().min(12)]
}

/// Append a HUMAN DECISION entry to the log.
/// `provenance` is populated when an assessment file was found for this feature+stage.
/// When None, falls back to the legacy `verify_artifacts` path for backward compatibility.
#[allow(clippy::too_many_arguments)]
pub fn append_decision(
    log_path: &Path,
    feature: &str,
    stage: &str,
    decision: &str,
    reason: &str,
    reentry_after: Option<&str>,
    closure_findings: Option<&[String]>,
    event_evidence_sha: Option<&str>,
    terminal_scope: Option<&crate::terminal_closure::ValidatedScope>,
    applicability_record_sha: Option<&str>,
    provenance: Option<&DecisionProvenance>,
    repo_root: &Path,
    codex_dir: &Path,
) -> Result<()> {
    ensure_log_exists(log_path)?;

    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let sha = git_head_sha(repo_root).unwrap_or_else(|_| "(unknown)".to_string());

    let mut entry = String::new();
    entry.push('\n');
    entry.push_str(&format!(
        "## {} HUMAN DECISION — {} — Stage {}\n",
        ts, feature, stage
    ));
    entry.push_str(&format!("Commit at decision: {}\n", sha));
    entry.push_str(&format!("Decision: {}\n", decision));
    if let Some(review_id) = reentry_after {
        if decision == "AUTHORIZE_EVIDENCE_CLOSURE" {
            entry.push_str(&format!("Evidence closure after: {}\n", review_id));
        } else if decision == "AUTHORIZE_INFRASTRUCTURE_REVALIDATION" {
            entry.push_str(&format!(
                "Infrastructure revalidation after: {}\n",
                review_id
            ));
        } else if decision == "AUTHORIZE_TERMINAL_REVIEW_CLOSURE" {
            entry.push_str(&format!("Terminal closure after: {}\n", review_id));
        } else if decision == "AUTHORIZE_LINEAGE_REPLACEMENT" {
            entry.push_str(&format!("Lineage replacement after: {}\n", review_id));
        } else if decision == "AUTHORIZE_ASSESSMENT_INPUT_CORRECTION" {
            entry.push_str(&format!(
                "Assessment input correction after: {}\n",
                review_id
            ));
        } else {
            entry.push_str(&format!("Re-entry after: {}\n", review_id));
        }
    }
    if let Some(findings) = closure_findings {
        entry.push_str(&format!("Authorized findings: {}\n", findings.join(", ")));
    }
    if let Some(sha) = event_evidence_sha {
        entry.push_str(&format!("Infrastructure repair evidence: {}\n", sha));
    }
    if let Some(scope) = terminal_scope {
        entry.push_str(&format!(
            "Terminal closure assessment SHA-256: {}\n",
            scope.scope.assessment_sha256
        ));
        entry.push_str(&format!(
            "Terminal closure scope SHA-256: {}\n",
            scope.sha256
        ));
        entry.push_str(&format!(
            "Terminal closure scope JSON: {}\n",
            scope.raw.trim()
        ));
    }
    if let Some(sha) = applicability_record_sha {
        if decision == "AUTHORIZE_ASSESSMENT_INPUT_CORRECTION" {
            entry.push_str(&format!("Correction record SHA-256: {}\n", sha));
        } else {
            entry.push_str(&format!("Applicability record SHA-256: {}\n", sha));
        }
    }
    entry.push_str(&format!("Reason/next: {}\n", reason));

    let mut legacy_changed = false;

    if let Some(prov) = provenance {
        // Structured Provenance block (AC-3).
        entry.push_str("Provenance:\n");
        entry.push_str(&format!("  assessment: {}\n", prov.assessment_path));

        if let Some(ref err) = prov.provenance_error {
            // Assessment file was found but frontmatter could not be parsed.
            entry.push_str(&format!("  status: PROVENANCE_UNVERIFIABLE — {}\n", err));
            if let Some(ref rationale) = prov.override_rationale {
                entry.push_str(&format!("  override: {}\n", rationale));
            }
        } else {
            // Commit drift (AC-5).
            let commit_note = if prov.review_commit.is_empty() {
                "PROVENANCE_UNVERIFIABLE: review_commit absent".to_string()
            } else if prov.head_sha == "(unknown)" {
                "HEAD_UNKNOWN: git rev-parse failed".to_string()
            } else if prov.review_commit == prov.head_sha {
                "HEAD_MATCH".to_string()
            } else {
                format!("HEAD_DRIFT: current={}", short_sha(&prov.head_sha))
            };
            let rc_display = if prov.review_commit.is_empty() {
                "(unknown)"
            } else {
                &prov.review_commit
            };
            entry.push_str(&format!(
                "  review_commit: {}  [{}]\n",
                rc_display, commit_note
            ));

            // Packet hash (AC-4). UNVERIFIABLE cases are explicit, not silent.
            let hash_note = if prov.packet_sha_stored.is_empty() {
                "PROVENANCE_UNVERIFIABLE: no stored sha in assessment".to_string()
            } else if prov.packet_sha_actual.starts_with("UNVERIFIABLE:") {
                format!(
                    "PROVENANCE_UNVERIFIABLE: {}",
                    &prov.packet_sha_actual["UNVERIFIABLE:".len()..]
                )
            } else if prov.packet_sha_actual == prov.packet_sha_stored {
                "MATCH".to_string()
            } else {
                format!(
                    "MISMATCH: stored={} / recomputed={}",
                    short_sha(&prov.packet_sha_stored),
                    short_sha(&prov.packet_sha_actual)
                )
            };
            entry.push_str(&format!("  packet_sha256: [{}]\n", hash_note));

            // Coverage gate status (AC-2, AC-8, AC-7).
            // Gate markers only apply when decision == APPROVE_STAGE; other decisions are informational.
            let gate_note = if decision == "APPROVE_STAGE"
                && matches!(
                    prov.coverage_state.as_str(),
                    "CRITICAL_OMISSION" | "EMPTY_PACKET"
                ) {
                match &prov.override_rationale {
                    Some(rationale) => format!("COVERAGE_GATE_OVERRIDDEN: {}", rationale),
                    None => "COVERAGE_GATE_TRIGGERED".to_string(),
                }
            } else if decision == "APPROVE_STAGE" {
                "OK".to_string()
            } else {
                "INFORMATIONAL".to_string()
            };
            let cs_display = if prov.coverage_state.is_empty() {
                "(unknown)"
            } else {
                &prov.coverage_state
            };
            entry.push_str(&format!(
                "  coverage_state: {}  [{}]\n",
                cs_display, gate_note
            ));
        }
    } else {
        // Backward compat: no assessment found — legacy artifact integrity check.
        let (verified_against, verify_lines, changed) = verify_artifacts(feature, stage, codex_dir);
        if let Some(path) = &verified_against {
            entry.push_str(&format!("Verified against: {}\n", path));
        }
        if !verify_lines.is_empty() {
            entry.push_str("Artifact integrity (informational audit, not a gate):\n");
            entry.push_str(&verify_lines);
        }
        legacy_changed = changed;
    }

    append_to_log(log_path, &entry)?;

    if legacy_changed {
        eprintln!("WARNING: some reviewed artifacts changed since the review — recorded with that flagged (advisory only).");
    }
    Ok(())
}

fn append_to_log(log_path: &Path, entry: &str) -> Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(log_path)
        .with_context(|| format!("could not open log for appending: {}", log_path.display()))?;
    file.write_all(entry.as_bytes())
        .with_context(|| format!("could not write to log: {}", log_path.display()))?;
    file.flush().context("could not flush log")?;
    Ok(())
}

pub(crate) fn append_raw(log_path: &Path, entry: &str) -> Result<()> {
    ensure_log_exists(log_path)?;
    append_to_log(log_path, entry)
}

fn git_head_sha(repo_root: &Path) -> Result<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_root)
        .output()
        .context("git rev-parse HEAD")?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn verify_artifacts(
    feature: &str,
    stage: &str,
    codex_dir: &Path,
) -> (Option<String>, String, bool) {
    // Find most recent assessment for this feature/stage
    let pattern = format!("{}-stage-{}-", feature, stage);
    let entries: Vec<_> = match std::fs::read_dir(codex_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name().to_string_lossy().contains(&pattern)
                    && !e.file_name().to_string_lossy().contains(".packet")
            })
            .collect(),
        Err(_) => return (None, String::new(), false),
    };

    let latest = entries.into_iter().max_by_key(|e| e.file_name());
    let latest = match latest {
        Some(ref e) => e.path(),
        None => return (None, String::new(), false),
    };

    let content = match std::fs::read_to_string(&latest) {
        Ok(c) => c,
        Err(_) => return (Some(latest.display().to_string()), String::new(), false),
    };

    let mut verify_lines = String::new();
    let mut changed = false;
    let mut current_path = String::new();

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("    - path: ") {
            current_path = rest.trim().to_string();
        } else if line.trim_start().starts_with("sha256: ") && !current_path.is_empty() {
            let stored_sha = line.trim().trim_start_matches("sha256: ").to_string();
            let now_sha = if Path::new(&current_path).exists() {
                crate::packet::sha256_file(&current_path).unwrap_or_else(|_| "(error)".to_string())
            } else {
                "(missing)".to_string()
            };
            if now_sha == stored_sha {
                verify_lines.push_str(&format!("  MATCH   {}\n", current_path));
            } else {
                verify_lines.push_str(&format!(
                    "  CHANGED {} (reviewed {} / now {})\n",
                    current_path,
                    &stored_sha[..12.min(stored_sha.len())],
                    &now_sha[..12.min(now_sha.len())]
                ));
                changed = true;
            }
            current_path.clear();
        }
    }

    (Some(latest.display().to_string()), verify_lines, changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_log(dir: &std::path::Path, entries: &[&str]) -> std::path::PathBuf {
        let log_path = dir.join("review-log.md");
        let mut content = LOG_HEADER.to_string();
        for e in entries {
            content.push('\n');
            content.push_str(e);
        }
        std::fs::write(&log_path, content).expect("write test log");
        log_path
    }

    /// A minimal, correctly-shaped REVIEW header line, exactly as `append_review` emits it
    /// (timestamp value is irrelevant to matching — only the trailing suffix is checked).
    fn review_header(feature: &str, stage: &str) -> String {
        format!(
            "## 2026-01-01T00:00:00Z REVIEW — {} — Stage {}\n",
            feature, stage
        )
    }

    /// A REVIEW entry block exactly as `append_review` emits its first two lines — the only two
    /// lines `resolve_continued_round`'s lookup reads. `continues` mirrors the optional
    /// `Continues:` line `append_review` now writes when a predecessor was supplied.
    fn review_entry(
        feature: &str,
        stage: &str,
        review_id: &str,
        continues: Option<&str>,
    ) -> String {
        let mut e = review_header(feature, stage);
        e.push_str(&format!("Review ID: {}\n", review_id));
        if let Some(c) = continues {
            e.push_str(&format!("Continues: {}\n", c));
        }
        e
    }

    fn reentry_decision(feature: &str, stage: &str, reason: &str) -> String {
        format!(
            "## 2026-01-01T00:01:00Z HUMAN DECISION — {feature} — Stage {stage}\nDecision: AUTHORIZE_REENTRY\nRe-entry after: REV__{feature}__{stage}__R3\nReason/next: {reason}\n"
        )
    }

    fn human_decision(
        timestamp: &str,
        feature: &str,
        stage: &str,
        decision: &str,
        reason: &str,
    ) -> String {
        format!(
            "## {timestamp} HUMAN DECISION — {feature} — Stage {stage}\nDecision: {decision}\nReason/next: {reason}\n"
        )
    }

    #[test]
    fn resolve_continued_round_happy_path_is_predecessor_round_plus_one() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log_path = write_log(
            dir.path(),
            &[&review_entry("F-0004", "8", "REV__F-0004__8__R1", None)],
        );
        let round =
            resolve_continued_round(&log_path, "F-0004", "8", "REV__F-0004__8__R1").unwrap();
        assert_eq!(round, 2);
    }

    #[test]
    fn resolve_continued_round_chases_a_multi_hop_chain() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log_path = write_log(
            dir.path(),
            &[
                &review_entry("F-0004", "8", "REV__F-0004__8__R1", None),
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__isolated-branch__R1",
                    Some("REV__F-0004__8__R1"),
                ),
            ],
        );
        // The second entry is itself round 2 by position (regardless of what its own ID string
        // says — round is derived from position among matching headings, never parsed from the
        // ID), so continuing it resolves to round 3.
        let round =
            resolve_continued_round(&log_path, "F-0004", "8", "REV__isolated-branch__R1").unwrap();
        assert_eq!(round, 3);
    }

    #[test]
    fn resolve_continued_round_rejects_a_predecessor_for_a_different_feature() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log_path = write_log(
            dir.path(),
            &[&review_entry(
                "OTHER-FEATURE",
                "8",
                "REV__OTHER-FEATURE__8__R1",
                None,
            )],
        );
        let err = resolve_continued_round(&log_path, "F-0004", "8", "REV__OTHER-FEATURE__8__R1")
            .unwrap_err();
        assert!(err
            .to_string()
            .contains("does not resolve to an existing review"));
    }

    #[test]
    fn resolve_continued_round_rejects_a_predecessor_for_a_different_stage() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log_path = write_log(
            dir.path(),
            &[&review_entry("F-0004", "5", "REV__F-0004__5__R1", None)],
        );
        let err =
            resolve_continued_round(&log_path, "F-0004", "8", "REV__F-0004__5__R1").unwrap_err();
        assert!(err
            .to_string()
            .contains("does not resolve to an existing review"));
    }

    #[test]
    fn resolve_continued_round_rejects_a_nonexistent_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log_path = write_log(
            dir.path(),
            &[&review_entry("F-0004", "8", "REV__F-0004__8__R1", None)],
        );
        let err =
            resolve_continued_round(&log_path, "F-0004", "8", "REV__made-up__R9").unwrap_err();
        assert!(err
            .to_string()
            .contains("does not resolve to an existing review"));
    }

    #[test]
    fn resolve_continued_round_rejects_a_cycle() {
        let dir = tempfile::tempdir().expect("tempdir");
        // Two entries whose Continues links point at each other. This cannot arise from real use
        // of this mechanism (a predecessor's Continues is always fixed before the successor is
        // written), but the check must still refuse it defensively rather than loop forever.
        let log_path = write_log(
            dir.path(),
            &[
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__F-0004__8__A",
                    Some("REV__F-0004__8__B"),
                ),
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__F-0004__8__B",
                    Some("REV__F-0004__8__A"),
                ),
            ],
        );
        let err =
            resolve_continued_round(&log_path, "F-0004", "8", "REV__F-0004__8__A").unwrap_err();
        assert!(
            err.to_string().contains("cycle") || err.to_string().contains("round budget"),
            "expected a cycle/budget refusal, got: {err}"
        );
    }

    #[test]
    fn resolve_continued_round_rejects_exceeding_the_round_budget() {
        let dir = tempfile::tempdir().expect("tempdir");
        // Round 3 by position; continuing it would make round 4, over the 3-round budget.
        let log_path = write_log(
            dir.path(),
            &[
                &review_entry("F-0004", "8", "REV__F-0004__8__R1", None),
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__F-0004__8__R2",
                    Some("REV__F-0004__8__R1"),
                ),
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__F-0004__8__R3",
                    Some("REV__F-0004__8__R2"),
                ),
            ],
        );
        let err =
            resolve_continued_round(&log_path, "F-0004", "8", "REV__F-0004__8__R3").unwrap_err();
        assert!(
            err.to_string().contains("round budget") || err.to_string().contains("exceed"),
            "expected a round-budget refusal, got: {err}"
        );
    }

    #[test]
    fn ordinary_fourth_review_is_rejected() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log_path = write_log(
            dir.path(),
            &[
                &review_entry("F-0004", "8", "REV__F-0004__8__R1", None),
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__F-0004__8__R2",
                    Some("REV__F-0004__8__R1"),
                ),
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__F-0004__8__R3",
                    Some("REV__F-0004__8__R2"),
                ),
            ],
        );
        let err = compute_review_round(&log_path, "F-0004", "8").unwrap_err();
        assert!(err.to_string().contains("exceed"));
    }

    #[test]
    fn authorized_reentry_is_distinct_and_preserves_scope() {
        let dir = tempfile::tempdir().expect("tempdir");
        let reason = "Stage 8 review re-entry authorized after R3 budget exhaustion, limited to the named findings.";
        let log_path = write_log(
            dir.path(),
            &[
                &review_entry("F-0004", "8", "REV__F-0004__8__R1", None),
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__F-0004__8__R2",
                    Some("REV__F-0004__8__R1"),
                ),
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__F-0004__8__R3",
                    Some("REV__F-0004__8__R2"),
                ),
                &reentry_decision("F-0004", "8", reason),
            ],
        );
        let authorization = resolve_reentry_authorization(
            &log_path,
            "F-0004",
            "8",
            "REV__F-0004__8__R3",
            ReentryDecisionSemantics::CanonicalLogOrder,
        )
        .unwrap();
        assert_eq!(authorization.scope, reason);
        assert_eq!(
            format_reentry_id("F-0004", "8"),
            "REV__F-0004__8__REENTRY-1"
        );
        assert!(compute_review_round(&log_path, "F-0004", "8")
            .unwrap_err()
            .to_string()
            .contains("exceed"));
    }

    #[test]
    fn reentry_without_explicit_authorization_fails_closed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log_path = write_log(
            dir.path(),
            &[
                &review_entry("F-0004", "8", "REV__F-0004__8__R1", None),
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__F-0004__8__R2",
                    Some("REV__F-0004__8__R1"),
                ),
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__F-0004__8__R3",
                    Some("REV__F-0004__8__R2"),
                ),
                &human_decision(
                    "2026-01-01T00:01:00Z",
                    "F-0004",
                    "8",
                    "REQUEST_CHANGES",
                    "Fix the findings and continue.",
                ),
            ],
        );
        let err = resolve_reentry_authorization(
            &log_path,
            "F-0004",
            "8",
            "REV__F-0004__8__R3",
            ReentryDecisionSemantics::CanonicalLogOrder,
        )
        .unwrap_err();
        assert!(err.to_string().contains("no AUTHORIZE_REENTRY"));
    }

    #[test]
    fn canonical_order_allows_a_later_authorization_after_an_approval() {
        let dir = tempfile::tempdir().expect("tempdir");
        let same_timestamp = "2026-01-01T00:01:00Z";
        let scope = "Review re-entry authorized for the repaired findings.";
        let log_path = write_log(
            dir.path(),
            &[
                &review_entry("F-0001", "8", "REV__F-0001__8__R1", None),
                &review_entry("F-0001", "8", "REV__F-0001__8__R2", Some("REV__F-0001__8__R1")),
                &review_entry("F-0001", "8", "REV__F-0001__8__R3", Some("REV__F-0001__8__R2")),
                &human_decision(same_timestamp, "F-0001", "8", "APPROVE_STAGE", "Mistaken approval."),
                &human_decision(same_timestamp, "OTHER", "8", "STOP", "Unrelated."),
                &human_decision(same_timestamp, "F-0001", "7", "STOP", "Wrong stage."),
                &format!(
                    "## {same_timestamp} HUMAN DECISION — F-0001 — Stage 8\nDecision: AUTHORIZE_REENTRY\nRe-entry after: REV__F-0001__8__R3\nReason/next: {scope}\n"
                ),
            ],
        );

        let authorization = resolve_reentry_authorization(
            &log_path,
            "F-0001",
            "8",
            "REV__F-0001__8__R3",
            ReentryDecisionSemantics::CanonicalLogOrder,
        )
        .unwrap();
        assert_eq!(authorization.scope, scope);
        assert_eq!(authorization.decision_timestamp, same_timestamp);
    }

    #[test]
    fn canonical_order_denies_a_stop_appended_after_authorization() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log_path = write_log(
            dir.path(),
            &[
                &review_entry("F-0001", "8", "REV__F-0001__8__R1", None),
                &review_entry(
                    "F-0001",
                    "8",
                    "REV__F-0001__8__R2",
                    Some("REV__F-0001__8__R1"),
                ),
                &review_entry(
                    "F-0001",
                    "8",
                    "REV__F-0001__8__R3",
                    Some("REV__F-0001__8__R2"),
                ),
                &reentry_decision("F-0001", "8", "Review re-entry authorized."),
                &human_decision(
                    "2026-01-01T00:01:00Z",
                    "F-0001",
                    "8",
                    "STOP",
                    "Do not continue.",
                ),
            ],
        );

        let error = resolve_reentry_authorization(
            &log_path,
            "F-0001",
            "8",
            "REV__F-0001__8__R3",
            ReentryDecisionSemantics::CanonicalLogOrder,
        )
        .unwrap_err();
        assert!(error.to_string().contains("STOP appended after"));
    }

    #[test]
    fn canonical_order_ignores_stage_dispositions_after_authorization() {
        let dir = tempfile::tempdir().expect("tempdir");
        let scope = "One bounded reviewer execution for the repaired findings.";
        let log_path = write_log(
            dir.path(),
            &[
                &review_entry("F-0001", "8", "REV__F-0001__8__R1", None),
                &review_entry(
                    "F-0001",
                    "8",
                    "REV__F-0001__8__R2",
                    Some("REV__F-0001__8__R1"),
                ),
                &review_entry(
                    "F-0001",
                    "8",
                    "REV__F-0001__8__R3",
                    Some("REV__F-0001__8__R2"),
                ),
                &reentry_decision("F-0001", "8", scope),
                &human_decision(
                    "2026-01-01T00:02:00Z",
                    "F-0001",
                    "8",
                    "APPROVE_STAGE",
                    "Stage disposition only.",
                ),
                &human_decision(
                    "2026-01-01T00:03:00Z",
                    "F-0001",
                    "8",
                    "REQUEST_CHANGES",
                    "Stage disposition only.",
                ),
            ],
        );

        let authorization = resolve_reentry_authorization(
            &log_path,
            "F-0001",
            "8",
            "REV__F-0001__8__R3",
            ReentryDecisionSemantics::CanonicalLogOrder,
        )
        .unwrap();
        assert_eq!(authorization.scope, scope);
    }

    #[test]
    fn canonical_order_rejects_authorization_bound_to_another_review() {
        let dir = tempfile::tempdir().expect("tempdir");
        let authorization = "## 2026-01-01T00:01:00Z HUMAN DECISION — F-0001 — Stage 8\nDecision: AUTHORIZE_REENTRY\nRe-entry after: REV__F-0001__8__R2\nReason/next: Wrong boundary.\n";
        let log_path = write_log(
            dir.path(),
            &[
                &review_entry("F-0001", "8", "REV__F-0001__8__R1", None),
                &review_entry(
                    "F-0001",
                    "8",
                    "REV__F-0001__8__R2",
                    Some("REV__F-0001__8__R1"),
                ),
                &review_entry(
                    "F-0001",
                    "8",
                    "REV__F-0001__8__R3",
                    Some("REV__F-0001__8__R2"),
                ),
                authorization,
            ],
        );

        let error = resolve_reentry_authorization(
            &log_path,
            "F-0001",
            "8",
            "REV__F-0001__8__R3",
            ReentryDecisionSemantics::CanonicalLogOrder,
        )
        .unwrap_err();
        assert!(error.to_string().contains("no AUTHORIZE_REENTRY bound"));
    }

    #[test]
    fn legacy_contract_denies_multiple_post_r3_decisions_as_ambiguous() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log_path = write_log(
            dir.path(),
            &[
                &review_entry("F-0001", "8", "REV__F-0001__8__R1", None),
                &review_entry(
                    "F-0001",
                    "8",
                    "REV__F-0001__8__R2",
                    Some("REV__F-0001__8__R1"),
                ),
                &review_entry(
                    "F-0001",
                    "8",
                    "REV__F-0001__8__R3",
                    Some("REV__F-0001__8__R2"),
                ),
                &human_decision(
                    "2026-01-01T00:01:00Z",
                    "F-0001",
                    "8",
                    "APPROVE_STAGE",
                    "Mistaken approval.",
                ),
                &human_decision(
                    "2026-01-01T00:02:00Z",
                    "F-0001",
                    "8",
                    "REQUEST_CHANGES",
                    "Review re-entry authorized.",
                ),
            ],
        );

        let error = resolve_reentry_authorization(
            &log_path,
            "F-0001",
            "8",
            "REV__F-0001__8__R3",
            ReentryDecisionSemantics::LegacySingleDecision,
        )
        .unwrap_err();
        assert!(error.to_string().contains("does not define their ordering"));
    }

    #[test]
    fn a_second_reentry_is_rejected() {
        let dir = tempfile::tempdir().expect("tempdir");
        let existing = "## 2026-01-01T00:02:00Z RE-ENTRY REVIEW — F-0004 — Stage 8\nReview ID: REV__F-0004__8__REENTRY-1\n";
        let log_path = write_log(
            dir.path(),
            &[
                &review_entry("F-0004", "8", "REV__F-0004__8__R1", None),
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__F-0004__8__R2",
                    Some("REV__F-0004__8__R1"),
                ),
                &review_entry(
                    "F-0004",
                    "8",
                    "REV__F-0004__8__R3",
                    Some("REV__F-0004__8__R2"),
                ),
                &reentry_decision(
                    "F-0004",
                    "8",
                    "Review re-entry authorized after R3 budget exhaustion.",
                ),
                existing,
            ],
        );
        let err = resolve_reentry_authorization(
            &log_path,
            "F-0004",
            "8",
            "REV__F-0004__8__R3",
            ReentryDecisionSemantics::CanonicalLogOrder,
        )
        .unwrap_err();
        assert!(err.to_string().contains("already been consumed"));
    }

    #[test]
    fn no_continues_flag_leaves_ordinary_round_derivation_unchanged() {
        // The no-flag path (compute_review_round) is untouched by this feature's existence:
        // same log, same result as before --continues was added.
        let dir = tempfile::tempdir().expect("tempdir");
        let log_path = write_log(
            dir.path(),
            &[&review_entry("F-0004", "8", "REV__F-0004__8__R1", None)],
        );
        assert_eq!(compute_review_round(&log_path, "F-0004", "8").unwrap(), 2);
    }

    #[test]
    fn round_one_when_log_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("does-not-exist.md");
        assert_eq!(
            compute_review_round(&missing, "UPG-0046", "selfdev-step-1").unwrap(),
            1
        );
    }

    #[test]
    fn round_one_when_log_exists_with_no_matches() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log_path = write_log(dir.path(), &[]);
        assert_eq!(
            compute_review_round(&log_path, "UPG-0046", "selfdev-step-1").unwrap(),
            1
        );
    }

    #[test]
    fn round_increments_across_matching_entries() {
        let dir = tempfile::tempdir().expect("tempdir");
        let e1 = review_header("UPG-0046", "selfdev-step-1");
        let e2 = review_header("UPG-0046", "selfdev-step-1");
        let log_path = write_log(dir.path(), &[&e1, &e2]);
        assert_eq!(
            compute_review_round(&log_path, "UPG-0046", "selfdev-step-1").unwrap(),
            3
        );
    }

    #[test]
    fn round_does_not_collide_across_similarly_named_stages() {
        // "Stage 1" must not be counted when computing the round for "Stage 10", or vice versa —
        // the exact scenario the human flagged as a danger of substring-based log parsing.
        let dir = tempfile::tempdir().expect("tempdir");
        let e1 = review_header("FEAT", "10");
        let log_path = write_log(dir.path(), &[&e1]);
        assert_eq!(
            compute_review_round(&log_path, "FEAT", "1").unwrap(),
            1,
            "an entry for Stage 10 must not be counted toward Stage 1's round"
        );
        assert_eq!(compute_review_round(&log_path, "FEAT", "10").unwrap(), 2);
    }

    #[test]
    fn round_does_not_collide_across_features_sharing_a_prefix() {
        let dir = tempfile::tempdir().expect("tempdir");
        let e1 = review_header("UPG-0046", "selfdev-step-1");
        let log_path = write_log(dir.path(), &[&e1]);
        assert_eq!(
            compute_review_round(&log_path, "UPG-004", "selfdev-step-1").unwrap(),
            1,
            "an entry for UPG-0046 must not be counted toward UPG-004's round"
        );
    }

    #[test]
    fn round_is_scoped_to_the_given_log_path_scratch_vs_durable() {
        // Mirrors the existing scratch/durable log separation in review.rs: compute_review_round
        // simply reads whichever path it's given, so a scratch log and a durable log for the
        // same feature+stage are independent by construction.
        let dir = tempfile::tempdir().expect("tempdir");
        let e1 = review_header("UPG-0046", "selfdev-step-1");
        let durable = write_log(dir.path(), &[&e1]);
        let scratch = dir.path().join("scratch-review-log.md");
        std::fs::write(&scratch, LOG_HEADER).expect("write scratch log");
        assert_eq!(
            compute_review_round(&durable, "UPG-0046", "selfdev-step-1").unwrap(),
            2
        );
        assert_eq!(
            compute_review_round(&scratch, "UPG-0046", "selfdev-step-1").unwrap(),
            1
        );
    }

    #[test]
    fn round_fails_closed_when_log_path_is_unreadable() {
        // A path that exists but is a directory, not a file — read_to_string must error, and
        // that error must propagate rather than being silently treated as "no matches".
        let dir = tempfile::tempdir().expect("tempdir");
        let not_a_file = dir.path().join("review-log.md");
        std::fs::create_dir(&not_a_file).expect("create dir standing in for the log path");
        assert!(compute_review_round(&not_a_file, "UPG-0046", "selfdev-step-1").is_err());
    }

    #[test]
    #[cfg(unix)]
    fn round_fails_closed_on_permission_error_not_silently_treated_as_missing() {
        // The specific bug a naive `if !log_path.exists() { return Ok(1) }` pre-check has:
        // `Path::exists()` collapses "genuinely not found" and "cannot access due to a
        // permission error" into the same `false` result. Reproduced here by making the
        // *containing directory* unsearchable (mode 000), so `fs::metadata`/`exists()` on the
        // log path inside it fails with `PermissionDenied`, not `NotFound`. This must be
        // reported as an error, never silently treated as "round 1."
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("tempdir");
        let locked_subdir = dir.path().join("locked");
        std::fs::create_dir(&locked_subdir).expect("create subdir");
        let log_path = locked_subdir.join("review-log.md");
        std::fs::write(&log_path, LOG_HEADER).expect("write log before locking");

        std::fs::set_permissions(&locked_subdir, std::fs::Permissions::from_mode(0o000))
            .expect("lock subdir permissions");

        let result = compute_review_round(&log_path, "UPG-0046", "selfdev-step-1");

        // Restore permissions immediately so the tempdir can clean itself up, regardless of
        // the assertion outcome below.
        std::fs::set_permissions(&locked_subdir, std::fs::Permissions::from_mode(0o755))
            .expect("restore subdir permissions for cleanup");

        // Root runs bypass Unix permission checks entirely — skip in that environment rather
        // than asserting a false failure.
        if unsafe { libc_geteuid_is_zero() } {
            return;
        }

        assert!(
            result.is_err(),
            "a permission error must never be silently treated as round 1: {:?}",
            result
        );
    }

    /// Minimal, dependency-free euid check (avoids adding a `libc` dependency for one test).
    #[cfg(unix)]
    unsafe fn libc_geteuid_is_zero() -> bool {
        extern "C" {
            fn geteuid() -> u32;
        }
        geteuid() == 0
    }

    #[test]
    fn format_review_id_uses_raw_stage_verbatim() {
        // No S<N> conversion — the raw --stage argument is used exactly as passed, for both
        // self-dev step strings and downstream stage ids.
        assert_eq!(
            format_review_id("UPG-0046__CHG-20260713-001", "selfdev-step-1", 1),
            "REV__UPG-0046__CHG-20260713-001__selfdev-step-1__R1"
        );
        assert_eq!(
            format_review_id("checkout-flow", "decomposition", 2),
            "REV__checkout-flow__decomposition__R2"
        );
        assert_eq!(
            format_review_id("checkout-flow", "7", 1),
            "REV__checkout-flow__7__R1"
        );
    }

    /// Set up a temp git repo and build a real `ReviewPacket` against it (needs the real
    /// Codeos repo as `toolkit_root` to find `dba/03-prompts/review/codeos-reviewer-task.md`). Shared by the
    /// tests below that need an actual packet, not just a hand-seeded log file.
    fn build_test_packet(repo_root: &std::path::Path, feature: &str, stage: &str) -> ReviewPacket {
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo_root)
            .output()
            .expect("git init");
        std::process::Command::new("git")
            .args(["config", "user.email", "t@t.test"])
            .current_dir(repo_root)
            .output()
            .ok();
        std::process::Command::new("git")
            .args(["config", "user.name", "T"])
            .current_dir(repo_root)
            .output()
            .ok();
        std::fs::write(repo_root.join("tracked.md"), "# tracked\n").expect("write");
        std::process::Command::new("git")
            .args(["add", "tracked.md"])
            .current_dir(repo_root)
            .output()
            .expect("git add");
        std::process::Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(repo_root)
            .output()
            .expect("git commit");

        let mut toolkit_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for _ in 0..4 {
            toolkit_root.pop();
        }

        let opts = crate::packet::PacketBuildOptions {
            feature: feature.to_string(),
            stage: stage.to_string(),
            artifacts: vec!["tracked.md".to_string()],
            sha_only_paths: vec![],
            basis: crate::packet::ReviewBasis::Ordinary { base: None },
            repo_root: repo_root.to_string_lossy().into_owned(),
            toolkit_root: toolkit_root.to_string_lossy().into_owned(),
            scope: None,
        };
        crate::packet::build(&opts).expect("build packet")
    }

    const TEST_REPLY: &str =
        "LOG SUMMARY: NO OBJECTION — ok\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n";

    fn test_raw() -> ReviewerRun {
        ReviewerRun {
            text: TEST_REPLY.to_string(),
            elapsed_ms: 1,
            source: RunSource::Codex {
                session_id: "test-session".to_string(),
                reconnect_count: 0,
                effort: "high".to_string(),
                model: "test-model".to_string(),
                cli_version: "0.162.0".to_string(),
            },
        }
    }

    #[allow(dead_code)]
    fn test_external_raw() -> ReviewerRun {
        ReviewerRun {
            text: TEST_REPLY.to_string(),
            elapsed_ms: 0,
            source: RunSource::External {
                label: Some("deepseek-v4-flash".to_string()),
            },
        }
    }

    #[test]
    fn append_review_writes_review_id_line() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo_root = dir.path();
        let packet = build_test_packet(repo_root, "UPG-TEST", "selfdev-step-1");
        let raw = test_raw();
        let parsed =
            crate::assessment::parse_review_output(&raw.text, &packet.coverage_state, 0, 0);

        let log_path = repo_root.join("review-log.md");
        let review_id = format_review_id(&packet.feature, &packet.stage, 1);
        append_review(
            &log_path,
            &review_id,
            &packet,
            &raw,
            &parsed,
            Path::new("assessment.md"),
            "deadbeef",
            Path::new("packet.txt"),
            "cafefeed",
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("append_review");

        let content = std::fs::read_to_string(&log_path).expect("read log");
        assert!(
            content.contains(&format!("Review ID: {}\n", review_id)),
            "log entry must contain the Review ID line: {}",
            content
        );
    }

    #[test]
    fn two_sequential_review_cycles_increment_the_round() {
        // Simulates exactly what review.rs::run() does, twice in a row: compute the round,
        // format the id, append the entry — then repeat. This is AC-2's actual acceptance
        // contract (round increments across real, sequential invocations), tested without
        // requiring a live Codex call by driving the same compute-then-append sequence
        // review.rs itself uses.
        let dir = tempfile::tempdir().expect("tempdir");
        let repo_root = dir.path();
        let packet = build_test_packet(repo_root, "UPG-TEST", "selfdev-step-1");
        let raw = test_raw();
        let parsed =
            crate::assessment::parse_review_output(&raw.text, &packet.coverage_state, 0, 0);
        let log_path = repo_root.join("review-log.md");

        // Round 1 (log does not exist yet).
        let round1 = compute_review_round(&log_path, &packet.feature, &packet.stage).unwrap();
        assert_eq!(round1, 1);
        let review_id1 = format_review_id(&packet.feature, &packet.stage, round1);
        assert!(review_id1.ends_with("__R1"));
        append_review(
            &log_path,
            &review_id1,
            &packet,
            &raw,
            &parsed,
            Path::new("assessment1.md"),
            "deadbeef1",
            Path::new("packet1.txt"),
            "cafefeed1",
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("append_review round 1");

        // Round 2 (log now has exactly one matching entry from round 1).
        let round2 = compute_review_round(&log_path, &packet.feature, &packet.stage).unwrap();
        assert_eq!(
            round2, 2,
            "round must increment after a real append_review call"
        );
        let review_id2 = format_review_id(&packet.feature, &packet.stage, round2);
        assert!(review_id2.ends_with("__R2"));
        append_review(
            &log_path,
            &review_id2,
            &packet,
            &raw,
            &parsed,
            Path::new("assessment2.md"),
            "deadbeef2",
            Path::new("packet2.txt"),
            "cafefeed2",
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("append_review round 2");

        let content = std::fs::read_to_string(&log_path).expect("read log");
        assert!(content.contains(&format!("Review ID: {}\n", review_id1)));
        assert!(content.contains(&format!("Review ID: {}\n", review_id2)));

        // A third read must now report round 3, proving the cycle keeps advancing correctly.
        let round3 = compute_review_round(&log_path, &packet.feature, &packet.stage).unwrap();
        assert_eq!(round3, 3);
    }

    #[test]
    fn semantic_review_rounds_are_isolated_and_bounded_per_revision() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("review-log.md");
        std::fs::write(
            &log,
            "Review ID: REV__F-1__8__SR1__R1\nReview ID: REV__F-1__8__SR1__R2\nReview ID: REV__F-1__8__SR1__R3\n",
        )
        .unwrap();
        assert!(compute_semantic_review_round(&log, "F-1", "8", 1).is_err());
        assert_eq!(
            compute_semantic_review_round(&log, "F-1", "8", 2).unwrap(),
            1
        );
        assert_eq!(
            format_semantic_review_id("F-1", "8", 2, 1),
            "REV__F-1__8__SR2__R1"
        );
    }

    #[test]
    fn ordinary_review_id_parser_distinguishes_stage_revision_and_round() {
        let legacy = parse_ordinary_review_id("REV__F-0001__8__R3", "F-0001", "8").unwrap();
        assert_eq!(legacy.feature, "F-0001");
        assert_eq!(legacy.stage, "8");
        assert_eq!(legacy.semantic_revision, None);
        assert_eq!(legacy.round, 3);

        for revision in [1, 2, 3] {
            let review_id = format!("REV__F-0001__8__SR{revision}__R3");
            let parsed = parse_ordinary_review_id(&review_id, "F-0001", "8").unwrap();
            assert_eq!(parsed.feature, "F-0001");
            assert_eq!(parsed.stage, "8");
            assert_eq!(parsed.semantic_revision, Some(revision));
            assert_eq!(parsed.round, 3);
        }
    }

    #[test]
    fn semantic_r1_and_r2_parse_but_are_not_reentry_targets() {
        for round in [1, 2] {
            let review_id = format!("REV__F-0001__8__SR2__R{round}");
            let parsed = parse_ordinary_review_id(&review_id, "F-0001", "8").unwrap();
            assert_eq!(parsed.round, round);

            let dir = tempfile::tempdir().unwrap();
            let log = write_log(
                dir.path(),
                &[&review_entry("F-0001", "8", &review_id, None)],
            );
            let error =
                validate_reentry_authorization_target(&log, "F-0001", "8", &review_id).unwrap_err();
            assert!(error.to_string().contains("available only for R3"));
        }
    }

    #[test]
    fn malformed_ordinary_review_ids_fail_closed() {
        for review_id in [
            "REV__F-0001__R3__SR2__8",
            "REV__F-0001__8__R3__garbage",
            "REV__F-0001__8__SR2__R3__R3",
            "something__R3",
            "REV__F-0001__8__SR2__R4",
            "REV__F-0001__8__SR2",
            "REV__F-0001__8__SR0__R3",
            "REV__F-0001__8__SR02__R3",
            "REV__F-0001__8__SR2__R03",
        ] {
            assert!(
                parse_ordinary_review_id(review_id, "F-0001", "8").is_none(),
                "malformed ID parsed: {review_id}"
            );
        }
    }

    #[test]
    fn reentry_target_uses_structured_round_not_matching_record_ordinal() {
        let dir = tempfile::tempdir().unwrap();
        let log = write_log(
            dir.path(),
            &[
                &review_entry("F-0001", "8", "REV__F-0001__8__R1", None),
                &review_entry("F-0001", "8", "REV__F-0001__8__R2", None),
                &review_entry("F-0001", "8", "REV__F-0001__8__R3", None),
                "## 2026-01-01T00:03:00Z EVIDENCE CLOSURE REVIEW — F-0001 — Stage 8\nReview ID: unrelated-closure-1\n",
                "## 2026-01-01T00:04:00Z EVIDENCE CLOSURE REVIEW — F-0001 — Stage 8\nReview ID: unrelated-closure-2\n",
                &review_entry("F-0001", "8", "REV__F-0001__8__SR2__R1", None),
                &review_entry("F-0001", "8", "REV__F-0001__8__SR2__R2", None),
                &review_entry("F-0001", "8", "REV__F-0001__8__SR2__R3", None),
            ],
        );

        validate_reentry_authorization_target(&log, "F-0001", "8", "REV__F-0001__8__SR2__R3")
            .unwrap();
    }

    #[test]
    fn package_bound_reentry_consumption_is_exact_to_the_stage8_cycle() {
        let dir = tempfile::tempdir().unwrap();
        let log = write_log(
            dir.path(),
            &[
                &review_entry("F-0001", "8", "REV__F-0001__8__SR1__R3", None),
                "## t RE-ENTRY REVIEW — F-0001 — Stage 8\nReview ID: REV__F-0001__8__REENTRY-1\nRe-entry after: REV__F-0001__8__SR1__R3\n",
                &review_entry("F-0001", "8", "REV__F-0001__8__SR2__R3", None),
            ],
        );

        validate_reentry_authorization_target_with_semantics(
            &log,
            "F-0001",
            "8",
            "REV__F-0001__8__SR2__R3",
            ReentryDecisionSemantics::PackageBoundCanonicalLogOrder {
                current_semantic_revision: Some(2),
            },
        )
        .unwrap();

        let error = validate_reentry_authorization_target_with_semantics(
            &log,
            "F-0001",
            "8",
            "REV__F-0001__8__SR1__R3",
            ReentryDecisionSemantics::PackageBoundCanonicalLogOrder {
                current_semantic_revision: Some(1),
            },
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("already been consumed"),
            "{error}"
        );
    }

    #[test]
    fn package_bound_reentry_ids_are_deterministic_and_legacy_compatible() {
        assert_eq!(
            format_package_reentry_id("F-0001", "8", Some(2)),
            "REV__F-0001__8__SR2__REENTRY-1"
        );
        assert_eq!(
            format_package_reentry_id("F-0001", "8", None),
            "REV__F-0001__8__REENTRY-1"
        );
    }

    #[test]
    fn ordinary_round_count_ignores_compound_review_headings() {
        let dir = tempfile::tempdir().unwrap();
        let log = write_log(
            dir.path(),
            &[
                &review_entry("F-0001", "8", "REV__F-0001__8__R1", None),
                "## 2026-01-01T00:03:00Z RE-ENTRY REVIEW — F-0001 — Stage 8\nReview ID: REV__F-0001__8__REENTRY-1\n",
                "## 2026-01-01T00:04:00Z EVIDENCE CLOSURE REVIEW — F-0001 — Stage 8\nReview ID: REV__F-0001__8__EVIDENCE-CLOSURE-1\n",
            ],
        );

        assert_eq!(compute_review_round(&log, "F-0001", "8").unwrap(), 2);
    }

    #[test]
    fn evidence_closure_requires_exact_authorized_findings_and_is_single_use() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("review-log.md");
        let text = "# log\n\n## t REVIEW — F-1 — Stage 8\nReview ID: REV__F-1__8__R1\n\n## t REVIEW — F-1 — Stage 8\nReview ID: REV__F-1__8__R2\n\n## t REVIEW — F-1 — Stage 8\nReview ID: REV__F-1__8__R3\n\n## t RE-ENTRY REVIEW — F-1 — Stage 8\nReview ID: REV__F-1__8__REENTRY-1\nEffective concern: DO NOT ADVANCE\n\n## t HUMAN DECISION — F-1 — Stage 8\nDecision: AUTHORIZE_EVIDENCE_CLOSURE\nEvidence closure after: REV__F-1__8__REENTRY-1\nAuthorized findings: runtime-evidence\nReason/next: evidence only\n";
        std::fs::write(&log, text).unwrap();
        assert!(resolve_evidence_closure_authorization(
            &log,
            "F-1",
            "8",
            "REV__F-1__8__REENTRY-1",
            &["runtime-evidence".into()]
        )
        .is_ok());
        assert!(resolve_evidence_closure_authorization(
            &log,
            "F-1",
            "8",
            "REV__F-1__8__REENTRY-1",
            &["other".into()]
        )
        .is_err());
        std::fs::write(&log, format!("{text}\n## t EVIDENCE CLOSURE REVIEW — F-1 — Stage 8\nReview ID: REV__F-1__8__EVIDENCE-CLOSURE-1\n")).unwrap();
        assert!(
            validate_evidence_closure_target(&log, "F-1", "8", "REV__F-1__8__REENTRY-1").is_err()
        );
    }
}
