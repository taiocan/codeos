use crate::precheck::redact_secrets;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ReviewedSnapshot {
    pub lineage: Vec<ReviewedPacketRecord>,
    /// Reviewer Tool Contract v17: full-file artifacts embedded in a reviewed packet are pre-images
    /// too, so a dirty or untracked reviewed workspace remains reconstructible.
    pub full_file_preimages: bool,
}

#[derive(Debug, Clone)]
pub struct ReviewedPacketRecord {
    pub packet_path: PathBuf,
    pub packet_sha256: String,
    pub review_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedDelta {
    pub diff: String,
    pub changed_paths: BTreeSet<String>,
    pub new_paths: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub enum ReviewBasis {
    Ordinary {
        base: Option<String>,
    },
    Bounded {
        predecessor_packet_sha256: String,
        provenance: String,
        delta: BoundedDelta,
    },
}

impl ReviewBasis {
    fn is_bounded(&self) -> bool {
        matches!(self, Self::Bounded { .. })
    }

    pub(crate) fn is_delta(&self) -> bool {
        self.is_bounded() || matches!(self, Self::Ordinary { base: Some(_) })
    }
}

const SIZE_LIMIT_BYTES: u64 = 256 * 1024;

const PATH_EXCLUDES: &[&str] = &[
    "*.env",
    ".env*",
    "*.pem",
    "*.key",
    "secrets/*",
    "credentials/*",
    "*runtime_events*.jsonl",
    "*.log",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoverageState {
    FullCoverage,
    PartialCoverage,
    SecretRedaction,
    CriticalOmission,
    EmptyPacket,
}

impl CoverageState {
    pub fn as_str(&self) -> &'static str {
        match self {
            CoverageState::FullCoverage => "FULL_COVERAGE",
            CoverageState::PartialCoverage => "PARTIAL_COVERAGE",
            CoverageState::SecretRedaction => "SECRET_REDACTION",
            CoverageState::CriticalOmission => "CRITICAL_OMISSION",
            CoverageState::EmptyPacket => "EMPTY_PACKET",
        }
    }

    /// Severity floor for effective_concern escalation.
    pub fn concern_floor(&self) -> u8 {
        match self {
            CoverageState::FullCoverage => 0,
            CoverageState::SecretRedaction | CoverageState::PartialCoverage => 1,
            CoverageState::EmptyPacket => 2,
            CoverageState::CriticalOmission => 3,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactEntry {
    pub path: String,
    pub sha256: String,
    pub visibility: String,
    /// Size in bytes as reported by `fs::metadata` at build time. `0` for `missing` artifacts
    /// (no file to stat).
    pub bytes: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ReviewPacket {
    pub feature: String,
    pub stage: String,
    pub branch: String,
    pub review_sha: String,
    pub base_sha: String,
    pub diff_hash: String,
    pub coverage_state: CoverageState,
    pub workspace_dirty: bool,
    pub redaction_count: usize,
    pub secret_flag: bool,
    pub artifacts: Vec<ArtifactEntry>,
    pub excluded_paths: Vec<(String, String, String)>, // (path, reason, section)
    /// The single changed-path authority for a fresh bounded review. It is intentionally omitted
    /// from sidecars: exact retry replays immutable bytes and never rebuilds or revalidates a
    /// fresh repair manifest.
    #[serde(skip)]
    pub bounded_changed_paths: Option<BTreeSet<String>>,
    /// Total bytes counted toward the packet budget (full-mode artifact bytes + diff bytes;
    /// mirrors the same accumulation `build()` already performs for the budget warning).
    pub review_content_bytes: u64,
    /// `review_content_bytes / 4`, the same rough token estimate already shown in the packet's
    /// `PACKET MANIFEST` section.
    pub estimated_review_tokens: u64,
    /// `CODEOS_PACKET_BUDGET_BYTES` (or its default) resolved at build time.
    pub budget_bytes: u64,
    /// `review_content_bytes > budget_bytes`.
    pub over_budget: bool,
    /// Bytes of the secret/size-filtered diff section, already counted into
    /// `review_content_bytes`. `0` when there is no diff content.
    pub diff_bytes: u64,
    /// Exactly the (path, bytes) pairs that actually count toward `review_content_bytes` —
    /// full-mode artifact content plus `(diff)` when non-empty. Deliberately excludes
    /// `sha-only` and delta-mode entries, which are never counted into the budget even though
    /// `ArtifactEntry.bytes` reports their on-disk size. This is the same list `build()`'s own
    /// oversized-packet warning ranks, exposed so callers can reproduce that exact ranking
    /// instead of (incorrectly) ranking all of `artifacts` by raw file size.
    pub budget_contributors: Vec<(String, u64)>,
    /// sha256 of the packet bytes this sidecar describes. Without it the sidecar's coverage claims
    /// are unbound: any packet file could be imported under any sidecar and inherit its
    /// `coverage_state`, `artifacts`, and `redaction_count`. Defaulted so sidecars written before
    /// this field existed still import, with a warning rather than silent acceptance.
    #[serde(default)]
    pub content_sha256: String,
    /// The Review Input Manifest this packet was built and gated from (Reviewer Tool Contract
    /// v17). Absent for packets built under earlier contracts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<crate::manifest::ReviewInputManifest>,
    /// The packet bytes. Held out of the sidecar: they live in the exported packet file itself, and
    /// storing them twice would create two copies that could disagree about what was reviewed.
    #[serde(skip)]
    content: String,
}

impl ReviewPacket {
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Machine-owned packet construction: include the exact required verification records the
    /// manifest names, redacted like every other packet input.
    pub fn include_verification_records(&mut self, lines: &[String]) {
        if lines.is_empty() {
            return;
        }
        let (redacted, _) = redact_secrets(&lines.join("\n"));
        self.content.push_str(&format!(
            "\n\nREQUIRED VERIFICATION RECORDS\n```jsonl\n{redacted}\n```\n"
        ));
        self.content_sha256 = crate::packet::sha256_str(&self.content);
        let bytes = redacted.len() as u64;
        self.review_content_bytes += bytes;
        self.estimated_review_tokens = self.review_content_bytes / 4;
        self.over_budget = self.review_content_bytes > self.budget_bytes;
        self.budget_contributors
            .push(("(required verification records)".to_string(), bytes));
    }

    pub fn require_f7_blocker_classification(&mut self) {
        self.content.push_str(
            "\n\nF7 BLOCKER CLASSIFICATION\nFor every IN-SCOPE BLOCKER add a separate line `Blocker kind: SUBSTANTIVE` or `Blocker kind: EVIDENCE_INSUFFICIENT`. For EVIDENCE_INSUFFICIENT also add `Evidence required: <specific proof still missing>`. Use EVIDENCE_INSUFFICIENT only when the finding alleges inadequate proof and no defect in implementation, approved meaning, architecture, safety, integrity, or another governed input. Missing classification makes the assessment incomplete.\n"
        );
        self.content_sha256 = crate::packet::sha256_str(&self.content);
    }

    pub fn constrain_to_evidence_closure(&mut self, predecessor: &str, findings: &[String]) {
        self.content.push_str(&format!(
            "\n\nEVIDENCE CLOSURE SCOPE\nPredecessor: {predecessor}\nAuthorized findings: {}\nAssess only whether newly supplied evidence closes every named evidence-only finding against the same implementation and governed inputs. Do not perform a general review. Any scope mismatch, contradiction, application defect, incomplete assessment, or unclosed finding requires DO NOT ADVANCE. If and only if every named finding is closed and no blocker is revealed, return ADVANCE / NO OBJECTION in the standard response schema.\n",
            findings.join(", ")
        ));
        self.content_sha256 = crate::packet::sha256_str(&self.content);
    }

    pub fn constrain_to_infrastructure_revalidation(
        &mut self,
        predecessor: &str,
        findings: &[String],
        event_evidence_sha: &str,
    ) {
        self.content.push_str(&format!(
            "\n\nINFRASTRUCTURE REPAIR REVALIDATION SCOPE\nPredecessor: {predecessor}\nCanonical event evidence SHA-256: {event_evidence_sha}\nAuthorized findings: {}\nAssess only whether the named evidence-system findings are closed by the supplied canonical evidence against the unchanged implementation and governed inputs. Do not perform a general review. A mismatch, incomplete assessment, unclosed finding, substantive defect, contradiction, or unrelated change requires DO NOT ADVANCE. If and only if every named finding is closed and no blocker is revealed, return ADVANCE / NO OBJECTION in the standard response schema.\n",
            findings.join(", ")
        ));
        self.content_sha256 = crate::packet::sha256_str(&self.content);
    }

    pub fn constrain_to_evidence_reassessment(
        &mut self,
        predecessor: &str,
        findings: &[String],
        event_evidence_sha: &str,
    ) {
        self.content.push_str(&format!(
            "\n\nF7 EVIDENCE REASSESSMENT SCOPE\nPredecessor: {predecessor}\nCanonical event evidence SHA-256: {event_evidence_sha}\nOpen evidence findings: {}\nAssess only whether this canonical evidence materially addresses and closes the named evidence-insufficient findings. The engine has established evidence identity, bindings, and non-equivalence; do not infer mechanical validity. Do not perform a general review. If all named findings close and no substantive blocker is revealed, return ADVANCE / NO OBJECTION. If evidence is materially relevant but insufficient, return DO NOT ADVANCE, state the remaining evidence required, and emit `EVIDENCE MATERIALITY: MATERIAL` before the standard final three lines. If it does not address the requirement, emit `EVIDENCE MATERIALITY: NOT_MATERIAL`. If a substantive blocker is revealed, classify it explicitly as SUBSTANTIVE; that closes evidence-loop eligibility.\n",
            findings.join(", ")
        ));
        self.content_sha256 = crate::packet::sha256_str(&self.content);
    }

    pub fn constrain_to_terminal_review_closure(
        &mut self,
        terminal_review_id: &str,
        terminal_assessment: &str,
        scope_json: &str,
        repair_manifest_json: &str,
    ) {
        self.content.push_str(&format!(
            "\n\nTERMINAL REVIEW CLOSURE — EVIDENCE OF REPAIR ONLY\nTerminal review: {terminal_review_id}\nImmutable terminal assessment:\n{terminal_assessment}\nAuthorized immutable-source conditions: {scope_json}\nRepair manifest: {repair_manifest_json}\nAnswer only: (1) were every authorized condition repaired, (2) is the applicable evidence valid and fresh where implementation changed, and (3) did the repair introduce an obvious blocker within the reviewed repair surface? Judge whether the changes actually remain within the authorized repair scope. Do not reopen the feature, architecture, Contract, or unrelated implementation for general review. Any scope excess, unclosed condition, invalid evidence, new in-scope blocker, ambiguity, or incomplete response requires DO NOT ADVANCE. Only complete closure of every condition permits NO OBJECTION.\n"
        ));
        self.content_sha256 = crate::packet::sha256_str(&self.content);
    }

    pub fn constrain_to_lineage_replacement(
        &mut self,
        invalid_review_id: &str,
        applicability_record: &str,
    ) {
        self.content.push_str(&format!(
            "\n\nINVALID REVIEW INPUT LINEAGE — SINGLE REPLACEMENT ASSESSMENT\nInvalid review: {invalid_review_id}\nImmutable applicability record:\n{applicability_record}\nAssess the original authorized semantic scope anew from the complete current context and current F6-selected evidence. Do not inherit any conclusion, finding disposition, or progression state from the invalid review. Do not perform an unrestricted feature review or expand scope. Any input ambiguity, scope drift, blocker, malformed response, or incomplete response requires DO NOT ADVANCE. This is the only replacement assessment.\n"
        ));
        self.content_sha256 = crate::packet::sha256_str(&self.content);
    }

    pub fn constrain_to_assessment_input_correction(
        &mut self,
        failed_replacement: &str,
        correction_record: &str,
        verification_records: &str,
    ) {
        self.content.push_str(&format!(
            "\n\nASSESSMENT INPUT CORRECTION — SINGLE CORRECTED ASSESSMENT\nFailed replacement: {failed_replacement}\nImmutable correction record:\n{correction_record}\n\nUNDERLYING INSPECTABLE VERIFICATION RECORDS\n```jsonl\n{verification_records}\n```\nReassess only the same replacement-review question using these corrected pre-existing inputs. The failed assessment remains immutable and DO NOT ADVANCE. Do not expand scope, accept new evidence, or respond to substantive repairs. Any mismatch or blocker requires DO NOT ADVANCE.\n"
        ));
        self.content_sha256 = crate::packet::sha256_str(&self.content);
    }

    /// Restore a packet from an exported sidecar plus the exported bytes. The bytes are adopted
    /// verbatim: nothing is rebuilt, so the reviewed packet and the recorded packet are the same
    /// bytes by construction rather than by a later comparison.
    pub fn from_sidecar(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    /// Reconstruct an immutable packet from the metadata stored in its assessment and the exact
    /// packet bytes. Used only by the bounded Terminal Review Closure retry path.
    #[allow(clippy::too_many_arguments)]
    pub fn from_immutable_record(
        feature: String,
        stage: String,
        branch: String,
        review_sha: String,
        base_sha: String,
        diff_hash: String,
        coverage_state: CoverageState,
        workspace_dirty: bool,
        redaction_count: usize,
        secret_flag: bool,
        artifacts: Vec<ArtifactEntry>,
        excluded_paths: Vec<(String, String, String)>,
        content: String,
    ) -> Self {
        let review_content_bytes = content.len() as u64;
        let content_sha256 = crate::packet::sha256_str(&content);
        Self {
            feature,
            stage,
            branch,
            review_sha,
            base_sha,
            diff_hash,
            coverage_state,
            workspace_dirty,
            redaction_count,
            secret_flag,
            artifacts,
            excluded_paths,
            bounded_changed_paths: None,
            review_content_bytes,
            estimated_review_tokens: review_content_bytes / 4,
            budget_bytes: 512 * 1024,
            over_budget: review_content_bytes > 512 * 1024,
            diff_bytes: 0,
            budget_contributors: Vec::new(),
            content_sha256,
            manifest: None,
            content,
        }
    }
}

pub struct PacketBuildOptions {
    pub feature: String,
    pub stage: String,
    pub artifacts: Vec<String>,
    pub sha_only_paths: Vec<String>,
    pub basis: ReviewBasis,
    pub repo_root: String,
    pub toolkit_root: String,
    /// The required input set under Reviewer Tool Contract v17. Only these paths may enter the
    /// diff, untracked, exclusion, and budget accounting; `None` keeps whole-repository behavior.
    pub scope: Option<BTreeSet<String>>,
}

fn in_scope(opts: &PacketBuildOptions, path: &str) -> bool {
    opts.scope.as_ref().is_none_or(|scope| scope.contains(path))
}

fn is_stage8_mandatory_context(opts: &PacketBuildOptions, path: &str) -> bool {
    if opts.stage != "8" {
        return false;
    }
    path == ".codeos/00-project/charter.md"
        || path == format!(".codeos/01-specification/intents/{}.md", opts.feature)
        || path
            == format!(
                ".codeos/01-specification/contracts/{}_contract.md",
                opts.feature
            )
        || path
            == format!(
                ".codeos/01-specification/event-schemas/{}_schema.md",
                opts.feature
            )
}

struct CommunicationSource {
    label: &'static str,
    canonical_path: PathBuf,
    content: String,
}

fn read_communication_source(
    path: &Path,
    allowed_root: &Path,
    label: &'static str,
) -> Result<CommunicationSource> {
    let canonical_root = std::fs::canonicalize(allowed_root).with_context(|| {
        format!(
            "could not resolve communication-context root {}",
            allowed_root.display()
        )
    })?;
    let canonical_path = std::fs::canonicalize(path)
        .with_context(|| format!("could not resolve applicable {label}: {}", path.display()))?;
    if !canonical_path.starts_with(&canonical_root) {
        bail!(
            "applicable {label} resolves outside its owning repository: {}",
            path.display()
        );
    }
    if !canonical_path.is_file() {
        bail!(
            "applicable {label} is not a regular file: {}",
            path.display()
        );
    }
    let content = std::fs::read_to_string(&canonical_path)
        .with_context(|| format!("could not read applicable {label}: {}", path.display()))?;
    Ok(CommunicationSource {
        label,
        canonical_path,
        content,
    })
}

/// The reader-output guidance for the reviewed project, resolved from **that project's own toolkit
/// mount** — `<repo_root>/.codeos/toolkit` downstream, or the toolkit itself in self-development.
/// `toolkit_root` is already that mount (`config::find_toolkit_root`), so its `dba-system.md` is the
/// configuration the project is bound to: a project stays on DBA-6 by mounting a DBA-6 toolkit, and
/// this reads whatever that mount declares. This is the same project-scoped rule a normal agent
/// applies when it reads `.codeos/toolkit/dba-system.md`.
///
/// Returns the `reader_output_policy` that configuration selects, or the unversioned
/// `dba/05-guidance/reader-oriented-output.md` when it selects none or cannot be resolved.
/// Resolution is deterministic from the mount contents, so the frozen packet bytes stay
/// reproducible, and every candidate path is under `toolkit_root`, so packet isolation is unchanged.
fn resolve_reader_output_guidance(repo_root: &Path, toolkit_root: &Path) -> PathBuf {
    let fallback = toolkit_root.join("dba/05-guidance/reader-oriented-output.md");
    let resolver = toolkit_root.join("dba/04-tools/configuration/resolve-project-configuration.sh");
    if !resolver.is_file() {
        return fallback;
    }
    let Ok(output) = std::process::Command::new("bash")
        .arg(resolver)
        .arg(repo_root)
        .output()
    else {
        return fallback;
    };
    if !output.status.success() {
        return fallback;
    }
    let resolved = String::from_utf8_lossy(&output.stdout);
    let selected = resolved.lines().find_map(|line| {
        line.strip_prefix("reader_output_policy=")
            .map(str::to_string)
    });
    match selected {
        Some(rel) if toolkit_root.join(&rel).is_file() => toolkit_root.join(rel),
        _ => fallback,
    }
}

fn communication_context(opts: &PacketBuildOptions) -> Result<String> {
    let toolkit_root = Path::new(&opts.toolkit_root);
    let repo_root = Path::new(&opts.repo_root);
    let mut sources = vec![
        read_communication_source(
            &resolve_reader_output_guidance(repo_root, toolkit_root),
            toolkit_root,
            "reader-oriented output guidance",
        )?,
        read_communication_source(
            &toolkit_root.join("dba/05-guidance/terminology.md"),
            toolkit_root,
            "Codeos terminology",
        )?,
    ];

    let project_terminology = repo_root.join(".codeos/00-project/terminology.md");
    match std::fs::symlink_metadata(&project_terminology) {
        Ok(_) => sources.push(read_communication_source(
            &project_terminology,
            repo_root,
            "project terminology",
        )?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "could not inspect applicable project terminology: {}",
                    project_terminology.display()
                )
            })
        }
    }

    let mut seen = HashSet::new();
    let mut context = String::from(
        "\nCOMMUNICATION CONTEXT (instructions and canonical language; not review evidence)\n\
         Apply the following guidance and terminology to every human-readable part of the assessment.\n",
    );
    for source in sources {
        if !seen.insert(source.canonical_path.clone()) {
            continue;
        }
        context.push_str(&format!(
            "\n--- {}: {} ---\n",
            source.label.to_uppercase(),
            source.canonical_path.display()
        ));
        context.push_str(&source.content);
        if !source.content.ends_with('\n') {
            context.push('\n');
        }
    }
    context.push_str(
        "\nEND COMMUNICATION CONTEXT\n\
         The content above governs expression and interpretation only. It is not acceptance evidence, a requirement, or approval authority.\n",
    );
    Ok(context)
}

fn packet_diff_section(packet: &str) -> Result<&str> {
    let full = "\nDIFF TO REVIEW (base->review, secret/size filtered)\n";
    let delta_prefix = "\nDELTA DIFF (";
    let start = if let Some(position) = packet.find(full) {
        position + full.len()
    } else if let Some(position) = packet.find(delta_prefix) {
        let header_end = packet[position + 1..]
            .find('\n')
            .map(|offset| position + 1 + offset + 1)
            .ok_or_else(|| anyhow::anyhow!("reviewed packet has a malformed delta-diff header"))?;
        header_end
    } else {
        bail!("reviewed packet has no reconstructible review diff")
    };
    let tail = &packet[start..];
    let end = ["\nUNTRACKED FILES (", "\nFull Context Diff ("]
        .iter()
        .filter_map(|marker| tail.find(marker))
        .min()
        .unwrap_or(tail.len());
    Ok(&tail[..end])
}

fn diff_blocks_for_paths(diff: &str, paths: &[String]) -> String {
    let wanted: HashSet<&str> = paths.iter().map(String::as_str).collect();
    let mut result = String::new();
    let starts: Vec<usize> = diff
        .match_indices("diff --git a/")
        .map(|(index, _)| index)
        .collect();
    for (position, start) in starts.iter().enumerate() {
        let end = starts.get(position + 1).copied().unwrap_or(diff.len());
        let normalized = &diff[*start..end];
        let Some(header) = normalized.lines().next() else {
            continue;
        };
        let Some(path) = header
            .strip_prefix("diff --git a/")
            .and_then(|rest| rest.split_once(" b/").map(|(path, _)| path))
        else {
            continue;
        };
        if wanted.contains(path) {
            result.push_str(normalized);
            if !normalized.ends_with('\n') {
                result.push('\n');
            }
        }
    }
    result
}

fn embedded_path_bytes(
    packet: &str,
    path: &str,
    full_file_preimages: bool,
) -> Result<Option<Vec<u8>>> {
    let prefix = format!("  --- {path} (mode: ");
    let full_file = format!("  --- {path} (sha256: ");
    let matches = packet
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            line.starts_with(&prefix)
                || (full_file_preimages
                    && line.starts_with(&full_file)
                    && line.ends_with(", visibility: shown) ---"))
        })
        .collect::<Vec<_>>();
    anyhow::ensure!(
        matches.len() <= 1,
        "immutable packet contains duplicate embedded records for {path}"
    );
    let Some((line_index, header)) = matches.first().copied() else {
        return Ok(None);
    };
    if !header.contains("mode: added_file")
        && !header.contains("mode: full_current_context")
        && !header.starts_with(&full_file)
    {
        return Ok(None);
    }
    let expected = header
        .split("sha256: ")
        .nth(1)
        .and_then(|value| value.split([',', ')']).next())
        .context("embedded artifact header has no SHA-256")?;
    let mut content = String::new();
    for line in packet.lines().skip(line_index + 1) {
        if line.is_empty() || line.starts_with("  --- ") {
            break;
        }
        let value = line
            .strip_prefix("    ")
            .context("embedded artifact content has an ambiguous boundary")?;
        content.push_str(value);
        content.push('\n');
    }
    anyhow::ensure!(
        sha256_str(&content) == expected,
        "embedded bytes for {path} do not match their immutable hash"
    );
    Ok(Some(content.into_bytes()))
}

fn packet_manifest_hashes(packet: &str) -> std::collections::BTreeMap<String, String> {
    let manifest = packet
        .split_once("PACKET MANIFEST\n")
        .map(|(_, tail)| tail)
        .unwrap_or_default()
        .split_once("\nREVIEW CONTEXT\n")
        .map(|(value, _)| value)
        .unwrap_or_default();
    let mut hashes = std::collections::BTreeMap::new();
    let mut path = None;
    for line in manifest.lines() {
        if let Some(value) = line.strip_prefix("    - path: ") {
            path = (value != "(diff)").then(|| value.to_string());
        } else if let (Some(current), Some(hash)) =
            (path.take(), line.strip_prefix("      sha256: "))
        {
            hashes.insert(current, hash.to_string());
        }
    }
    hashes
}

/// Reconstruct the requested paths from the immutable packet's recorded commit plus the exact
/// working-tree patch the reviewer saw, then compare that reviewed truth with the current files.
/// A redacted, missing, or non-applicable patch fails closed instead of silently falling back to
/// repository HEAD.
pub fn reviewed_snapshot_diff(
    snapshot: &ReviewedSnapshot,
    paths: &[String],
    repo_root: &str,
) -> Result<BoundedDelta> {
    anyhow::ensure!(
        !snapshot.lineage.is_empty(),
        "reviewed packet lineage is empty"
    );
    let temp = tempfile::tempdir().context("create reviewed-snapshot workspace")?;
    let root = &snapshot.lineage[0];
    for path in paths {
        let output = Command::new("git")
            .args(["show", &format!("{}:{path}", root.review_commit)])
            .current_dir(repo_root)
            .output()
            .context("read reviewed snapshot base file")?;
        if output.status.success() {
            let destination = temp.path().join(path);
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(destination, output.stdout)?;
        }
    }

    for (index, record) in snapshot.lineage.iter().enumerate() {
        let actual_packet_sha = sha256_file(record.packet_path.to_string_lossy().as_ref())?;
        anyhow::ensure!(
            actual_packet_sha == record.packet_sha256,
            "reviewed packet integrity mismatch: recorded {} but {} hashes to {}",
            record.packet_sha256,
            record.packet_path.display(),
            actual_packet_sha
        );
        let packet = std::fs::read_to_string(&record.packet_path)?;
        let selected_patch = diff_blocks_for_paths(packet_diff_section(&packet)?, paths);
        anyhow::ensure!(
            !selected_patch.contains("[REDACTED]"),
            "reviewed snapshot patch for requested paths contains redaction"
        );
        if !selected_patch.is_empty() {
            let patch_path = temp.path().join(format!("reviewed-{index}.patch"));
            std::fs::write(&patch_path, selected_patch)?;
            let output = Command::new("git")
                .arg("apply")
                .arg(&patch_path)
                .current_dir(temp.path())
                .output()
                .context("apply immutable reviewed-workspace patch")?;
            if !output.status.success() {
                bail!(
                    "could not reconstruct reviewed workspace from immutable packet {}: {}",
                    record.packet_sha256,
                    String::from_utf8_lossy(&output.stderr).trim()
                );
            }
        }
        for path in paths {
            if let Some(bytes) = embedded_path_bytes(&packet, path, snapshot.full_file_preimages)? {
                let destination = temp.path().join(path);
                if let Some(parent) = destination.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(destination, bytes)?;
            }
        }
        let hashes = packet_manifest_hashes(&packet);
        for path in paths {
            if let Some(expected) = hashes.get(path) {
                let reconstructed = temp.path().join(path);
                anyhow::ensure!(
                    reconstructed.is_file(),
                    "immutable packet records {path} but its required pre-image bytes are unavailable"
                );
                let actual = sha256_file(reconstructed.to_string_lossy().as_ref())?;
                anyhow::ensure!(
                    &actual == expected,
                    "reconstructed path {path} does not match immutable packet hash"
                );
            }
        }
    }
    let mut combined = String::new();
    let mut changed = BTreeSet::new();
    let mut new_paths = BTreeSet::new();
    for path in paths {
        let before = temp.path().join(path);
        let after = Path::new(repo_root).join(path);
        let was_present = before.is_file();
        let output = Command::new("git")
            .args([
                "diff",
                "--no-index",
                "--",
                before.to_string_lossy().as_ref(),
                after.to_string_lossy().as_ref(),
            ])
            .output()
            .context("compare reviewed snapshot with current workspace")?;
        if !matches!(output.status.code(), Some(0 | 1)) {
            bail!("could not compare reviewed snapshot path {path}");
        }
        if output.status.code() == Some(1) {
            changed.insert(path.clone());
            if !was_present && after.is_file() {
                new_paths.insert(path.clone());
            }
            let text = String::from_utf8_lossy(&output.stdout)
                .replace(&format!("a{}", before.display()), &format!("a/{path}"))
                .replace(&format!("b{}", after.display()), &format!("b/{path}"));
            combined.push_str(&text);
        }
    }
    Ok(BoundedDelta {
        diff: combined,
        changed_paths: changed,
        new_paths,
    })
}

pub fn build(opts: &PacketBuildOptions) -> Result<ReviewPacket> {
    if let ReviewBasis::Bounded { provenance, .. } = &opts.basis {
        anyhow::ensure!(
            !provenance.trim().is_empty(),
            "bounded review basis has no provenance"
        );
    }
    // Validate sha_only paths exist before doing anything.
    for so in &opts.sha_only_paths {
        if !Path::new(so).exists() {
            bail!("--sha-only path not found: {}", so);
        }
    }

    let branch = git_branch(&opts.repo_root)?;
    let review_sha = git_rev_parse("HEAD", &opts.repo_root)?;
    let preceding_stage = if opts.stage.parse::<u32>().is_ok() {
        let n: u32 = opts.stage.parse().unwrap();
        format!("{}", n.saturating_sub(1))
    } else {
        "n/a (non-numeric review target)".to_string()
    };

    let base_sha = match &opts.basis {
        ReviewBasis::Ordinary { base } => base.clone().unwrap_or_default(),
        ReviewBasis::Bounded { .. } => String::new(),
    };
    let delta_mode = opts.basis.is_delta();

    // Build diff
    let (_raw_diff, changed_files) = if let ReviewBasis::Bounded { delta, .. } = &opts.basis {
        (
            delta.diff.clone(),
            delta.changed_paths.iter().cloned().collect(),
        )
    } else if let ReviewBasis::Ordinary { base: Some(base) } = &opts.basis {
        let diff = git_diff_range(base, &opts.artifacts, &opts.repo_root)?;
        let files = git_diff_names(base, &opts.artifacts, &opts.repo_root)?;
        (diff, files)
    } else if !base_sha.is_empty() && base_sha != "(no base pin)" {
        let diff = git_diff_range(&base_sha, &[], &opts.repo_root)?;
        let files = git_diff_names(&base_sha, &[], &opts.repo_root)?;
        (diff, files)
    } else {
        let diff = git_diff_head(&opts.repo_root)?;
        let files = git_diff_names_head(&opts.repo_root)?;
        (diff, files)
    };

    let changed_files: Vec<String> = changed_files
        .into_iter()
        .filter(|path| in_scope(opts, path))
        .collect();
    let mut excluded_paths: Vec<(String, String, String)> = Vec::new();
    let mut keep_files: Vec<String> = Vec::new();
    for f in &changed_files {
        if is_path_excluded(f) || is_oversize(f) {
            excluded_paths.push((
                f.clone(),
                "path/size excluded".to_string(),
                "diff".to_string(),
            ));
        } else {
            keep_files.push(f.clone());
        }
    }

    let filtered_diff = if !keep_files.is_empty() {
        if let ReviewBasis::Bounded { delta, .. } = &opts.basis {
            diff_blocks_for_paths(&delta.diff, &keep_files)
        } else if let ReviewBasis::Ordinary { base: Some(base) } = &opts.basis {
            git_diff_range(base, &keep_files, &opts.repo_root)?
        } else if !base_sha.is_empty() && base_sha != "(no base pin)" {
            git_diff_range(&base_sha, &keep_files, &opts.repo_root)?
        } else {
            git_diff_head_paths(&keep_files, &opts.repo_root)?
        }
    } else {
        String::new()
    };

    let (redacted_diff, mut redaction_count) = redact_secrets(&filtered_diff);
    let mut secret_flag = redaction_count > 0;
    if secret_flag {
        excluded_paths.push((
            "(diff)".to_string(),
            "secret-like content redacted".to_string(),
            "diff".to_string(),
        ));
    }

    let diff_bytes = redacted_diff.len() as u64;
    let diff_hash = sha256_str(&redacted_diff);

    // Process artifacts
    let mut artifacts: Vec<ArtifactEntry> = Vec::new();
    let mut artifacts_block = String::new();
    let mut manifest_full = String::new();
    let mut manifest_sha_only = String::new();
    let mut artifact_excluded = false;
    let mut shown_count = 0usize;
    let mut delta_diff_count = 0usize;
    let mut review_content_bytes: u64 = 0;
    let mut file_contributors: Vec<(String, u64)> = Vec::new(); // Track (path, bytes) for warning

    // sha_only artifacts first
    for so in &opts.sha_only_paths {
        let bytes = std::fs::metadata(so).map(|m| m.len()).unwrap_or(0);
        let sha = sha256_file(so)?;
        artifacts.push(ArtifactEntry {
            path: so.clone(),
            sha256: sha.clone(),
            visibility: "path_sha_only".to_string(),
            bytes,
        });
        manifest_sha_only.push_str(&format!(
            "    - path: {}\n      mode: path_sha_only\n      bytes: {}\n      sha256: {}\n",
            so, bytes, sha
        ));
    }

    // Full / delta artifacts
    for a in &opts.artifacts {
        if !Path::new(a).exists() {
            artifacts_block.push_str(&format!(
                "  --- {} (visibility: missing — not shown) ---\n\n",
                a
            ));
            artifacts.push(ArtifactEntry {
                path: a.clone(),
                sha256: String::new(),
                visibility: "missing".to_string(),
                bytes: 0,
            });
            manifest_full.push_str(&format!(
                "    - path: {}\n      mode: omitted_with_reason\n      reason: requested artifact missing\n", a
            ));
            artifact_excluded = true;
            excluded_paths.push((
                a.clone(),
                "requested artifact missing".to_string(),
                "artifact".to_string(),
            ));
            continue;
        }

        let bytes = std::fs::metadata(a).map(|m| m.len()).unwrap_or(0);
        if bytes > SIZE_LIMIT_BYTES {
            artifacts_block.push_str(&format!(
                "  --- {} (visibility: oversize_omitted — over size limit, not shown) ---\n\n",
                a
            ));
            artifacts.push(ArtifactEntry {
                path: a.clone(),
                sha256: String::new(),
                visibility: "oversize_omitted".to_string(),
                bytes,
            });
            manifest_full.push_str(&format!(
                "    - path: {}\n      mode: omitted_with_reason\n      reason: over size limit\n",
                a
            ));
            artifact_excluded = true;
            excluded_paths.push((
                a.clone(),
                "requested artifact over size limit".to_string(),
                "artifact".to_string(),
            ));
            continue;
        }

        let sha = sha256_file(a)?;

        if delta_mode {
            let base = match &opts.basis {
                ReviewBasis::Ordinary { base: Some(base) } => base.as_str(),
                _ => "HEAD",
            };
            let bounded_new = matches!(&opts.basis, ReviewBasis::Bounded { delta, .. } if delta.new_paths.contains(a));
            let ordinary_untracked =
                !opts.basis.is_bounded() && !git_is_tracked(a, &opts.repo_root);
            if bounded_new || ordinary_untracked {
                // An explicit untracked artifact is the added-file part of this delta. There is no
                // historical version to compare, so show its complete current content rather than
                // synthesizing a base or requiring a Git mutation before review.
                let raw = std::fs::read_to_string(a)
                    .with_context(|| format!("could not read added artifact {}", a))?;
                let (redacted, rc) = redact_secrets(&raw);
                if rc > 0 {
                    secret_flag = true;
                    redaction_count += rc;
                    excluded_paths.push((
                        a.clone(),
                        "secret value redacted in place".to_string(),
                        "artifact".to_string(),
                    ));
                }
                let indented: String = redacted.lines().map(|l| format!("    {}\n", l)).collect();
                artifacts_block.push_str(&format!(
                    "  --- {} (mode: added_file, sha256: {}, bytes: {}) ---\n{}\n",
                    a, sha, bytes, indented
                ));
                artifacts.push(ArtifactEntry {
                    path: a.clone(),
                    sha256: sha.clone(),
                    visibility: "added_file".to_string(),
                    bytes,
                });
                manifest_full.push_str(&format!(
                    "    - path: {}\n      mode: added_file\n      bytes: {}\n      sha256: {}{}\n",
                    a,
                    bytes,
                    sha,
                    if rc > 0 {
                        "\n      note: secret value redacted in place"
                    } else {
                        ""
                    }
                ));
                review_content_bytes += redacted.len() as u64;
                file_contributors.push((a.clone(), redacted.len() as u64));
                delta_diff_count += 1;
                shown_count += 1;
                continue;
            }
            if is_stage8_mandatory_context(opts, a) {
                let raw = std::fs::read_to_string(a)
                    .with_context(|| format!("could not read mandatory Stage-8 context {}", a))?;
                let (redacted, rc) = redact_secrets(&raw);
                let vis = if rc > 0 {
                    secret_flag = true;
                    redaction_count += rc;
                    excluded_paths.push((
                        a.clone(),
                        "secret value redacted in place".to_string(),
                        "artifact".to_string(),
                    ));
                    "shown_redacted"
                } else {
                    "shown"
                };
                let indented: String = redacted.lines().map(|l| format!("    {}\n", l)).collect();
                artifacts_block.push_str(&format!(
                    "  --- {} (mode: full_current_context, sha256: {}, bytes: {}, visibility: {}) ---\n{}\n",
                    a, sha, bytes, vis, indented
                ));
                artifacts.push(ArtifactEntry {
                    path: a.clone(),
                    sha256: sha.clone(),
                    visibility: vis.to_string(),
                    bytes,
                });
                manifest_full.push_str(&format!(
                    "    - path: {}\n      mode: full_current_context\n      bytes: {}\n      sha256: {}{}\n",
                    a,
                    bytes,
                    sha,
                    if rc > 0 {
                        "\n      note: secret value redacted in place"
                    } else {
                        ""
                    }
                ));
                review_content_bytes += redacted.len() as u64;
                file_contributors.push((a.clone(), redacted.len() as u64));
                shown_count += 1;
                continue;
            }
            let changed = match &opts.basis {
                ReviewBasis::Bounded { delta, .. } => delta.changed_paths.contains(a),
                ReviewBasis::Ordinary { .. } => !git_diff_quiet(base, a, &opts.repo_root)?,
            };
            let vis = if changed {
                "delta_diff"
            } else {
                "path_sha_only"
            };
            artifacts_block.push_str(&format!(
                "  --- {} (mode: {}, sha256: {}, bytes: {}) ---\n\n",
                a, vis, sha, bytes
            ));
            artifacts.push(ArtifactEntry {
                path: a.clone(),
                sha256: sha.clone(),
                visibility: vis.to_string(),
                bytes,
            });
            manifest_full.push_str(&format!(
                "    - path: {}\n      mode: {}\n      bytes: {}\n      sha256: {}\n",
                a, vis, bytes, sha
            ));
            if changed {
                delta_diff_count += 1;
            }
            shown_count += 1;
        } else {
            let raw = std::fs::read_to_string(a)
                .with_context(|| format!("could not read artifact {}", a))?;
            let (redacted, rc) = redact_secrets(&raw);
            let vis = if rc > 0 {
                secret_flag = true;
                redaction_count += rc;
                excluded_paths.push((
                    a.clone(),
                    "secret value redacted in place".to_string(),
                    "artifact".to_string(),
                ));
                manifest_full.push_str(&format!(
                    "    - path: {}\n      mode: full_file\n      bytes: {}\n      sha256: {}\n      note: secret value redacted in place\n",
                    a, bytes, sha
                ));
                "shown_redacted"
            } else {
                manifest_full.push_str(&format!(
                    "    - path: {}\n      mode: full_file\n      bytes: {}\n      sha256: {}\n",
                    a, bytes, sha
                ));
                "shown"
            };
            let indented: String = redacted.lines().map(|l| format!("    {}\n", l)).collect();
            artifacts_block.push_str(&format!(
                "  --- {} (sha256: {}, visibility: {}) ---\n{}\n",
                a, sha, vis, indented
            ));
            artifacts.push(ArtifactEntry {
                path: a.clone(),
                sha256: sha,
                visibility: vis.to_string(),
                bytes,
            });
            review_content_bytes += bytes;
            file_contributors.push((a.clone(), bytes));
            shown_count += 1;
        }
    }

    review_content_bytes += diff_bytes;
    if diff_bytes > 0 {
        file_contributors.push(("(diff)".to_string(), diff_bytes));
    }

    // Untracked files are part of the change under review but appear in no diff. Each one is either
    // shown here or recorded as an exclusion — an exclusion downgrades coverage, so the packet
    // cannot report FULL_COVERAGE while withholding a new file.
    let mut untracked_block = String::new();
    let mut untracked_shown = 0usize;
    let untracked = if opts.basis.is_bounded() {
        // Package-bound re-entry includes explicitly selected new evidence. Unchanged historical
        // untracked material stays bound by identity instead of being embedded again.
        Some(Vec::new())
    } else {
        git_untracked_files(&opts.repo_root)
    };
    let untracked_unavailable = untracked.is_none();
    if untracked_unavailable {
        excluded_paths.push((
            "(untracked-file discovery)".to_string(),
            "git could not be consulted; untracked files may be missing from this packet"
                .to_string(),
            "untracked".to_string(),
        ));
    }
    for path in untracked.unwrap_or_default() {
        if opts.artifacts.contains(&path)
            || opts.sha_only_paths.contains(&path)
            || !in_scope(opts, &path)
        {
            continue; // already shown in its own right, or outside the required review scope
        }
        if is_path_excluded(&path) {
            excluded_paths.push((
                path.clone(),
                "excluded path pattern".to_string(),
                "untracked".to_string(),
            ));
            continue;
        }
        let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if bytes > SIZE_LIMIT_BYTES {
            excluded_paths.push((
                path.clone(),
                format!("over size limit ({bytes} bytes)"),
                "untracked".to_string(),
            ));
            continue;
        }
        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(error) => {
                excluded_paths.push((
                    path.clone(),
                    format!("unreadable: {error}"),
                    "untracked".to_string(),
                ));
                continue;
            }
        };
        let (redacted, hits) = redact_secrets(&raw);
        if hits > 0 {
            redaction_count += hits;
            secret_flag = true;
        }
        untracked_block.push_str(&format!(
            "\n  --- {} (untracked — new file, not in any diff; sha256: {}, bytes: {}) ---\n{}\n",
            path,
            sha256_str(&raw),
            bytes,
            redacted
        ));
        review_content_bytes += redacted.len() as u64;
        file_contributors.push((path.clone(), redacted.len() as u64));
        untracked_shown += 1;
    }

    // Coverage state (most severe wins)
    let coverage_state = if untracked_shown == 0
        && ((delta_mode && delta_diff_count == 0 && redacted_diff.trim().is_empty())
            || (!delta_mode && shown_count == 0 && redacted_diff.trim().is_empty()))
    {
        CoverageState::EmptyPacket
    } else if artifact_excluded || untracked_unavailable {
        CoverageState::CriticalOmission
    } else if secret_flag {
        CoverageState::SecretRedaction
    } else if !excluded_paths.is_empty() {
        CoverageState::PartialCoverage
    } else {
        CoverageState::FullCoverage
    };

    // workspace_dirty check
    let workspace_dirty = git_is_dirty(&opts.repo_root);

    // Packet construction measures and explains size. Codex invocation policy is enforced at the
    // model-spawning boundary, not here.
    let budget = std::env::var("CODEOS_PACKET_BUDGET_BYTES")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(50_000);
    if review_content_bytes > budget {
        // Enhanced warning: show overage, top contributors, and actionable suggestions
        let overage_multiple = (review_content_bytes as f64 / budget as f64).ceil() as u64;
        let budget_kb = budget / 1024;
        let packet_kb = review_content_bytes / 1024;

        eprintln!(
            "warning: packet is {} KB ({}× over {} KB budget)",
            packet_kb, overage_multiple, budget_kb
        );

        // Sort contributors by size (descending) and show top 3
        let mut sorted_contributors = file_contributors.clone();
        sorted_contributors.sort_by(|a, b| b.1.cmp(&a.1));
        let top_n = sorted_contributors.iter().take(3).collect::<Vec<_>>();

        if !top_n.is_empty() {
            eprintln!("  largest inputs:");
            for (path, bytes) in &top_n {
                let kb = bytes / 1024;
                let pct = (*bytes as f64 / review_content_bytes as f64 * 100.0) as u64;
                eprintln!("    {}: {} KB ({}%)", path, kb, pct);
            }
        }

        eprintln!("  suggest for R2+:");
        eprintln!(
            "    codeos-reviewer review {} {} --base <last-review-commit> <artifacts>",
            opts.feature, opts.stage
        );
        eprintln!("  optional:");
        eprintln!("    use --sha-only <path> only for large unchanged context files that are not the primary artifact under review; this reduces review evidence");
    }

    // Stage-specific checks
    let checks = stage_checks(&opts.stage);
    let expected = stage_expected(&opts.stage);
    let task_prompt_path = format!(
        "{}/dba/03-prompts/review/codeos-reviewer-task.md",
        opts.toolkit_root
    );
    let task_prompt = std::fs::read_to_string(&task_prompt_path)
        .with_context(|| format!("reviewer task template not found: {}", task_prompt_path))?;
    let communication_context = communication_context(opts)?;

    // Build the packet text
    let mut content = String::new();
    content.push_str(&task_prompt);
    content.push('\n');
    content.push_str(&communication_context);

    let budget_status = if review_content_bytes > budget {
        format!(
            "WARNING — {} bytes exceeds CODEOS_PACKET_BUDGET_BYTES={}",
            review_content_bytes, budget
        )
    } else {
        "OK".to_string()
    };

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let task_prompt_bytes = std::fs::metadata(&task_prompt_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let estimated_tokens = review_content_bytes / 4;

    content.push_str(&format!(
        "\nPACKET MANIFEST\n  generated: {now}\n  task_prompt: {task_prompt_path} ({task_prompt_bytes} bytes)\n  review_content_bytes: {review_content_bytes}\n  estimated_review_tokens: ~{estimated_tokens}\n  budget_status: {budget_status}\n  packet_mode: {mode}\n  delta_base: {delta_base}\n  items:\n{manifest_sha_only}{manifest_full}    - path: (diff)\n      mode: {diff_mode}\n      bytes: {diff_bytes}\n",
        mode = if delta_mode { "delta" } else { "full" },
        delta_base = match &opts.basis { ReviewBasis::Bounded { predecessor_packet_sha256, .. } => predecessor_packet_sha256.as_str(), ReviewBasis::Ordinary { base } => base.as_deref().unwrap_or("none") },
        diff_mode = if delta_mode { "delta_diff" } else { "full_file" },
    ));

    let dirty_note = if workspace_dirty {
        " (+ uncommitted workspace changes)"
    } else {
        ""
    };
    content.push_str(&format!(
        "\nREVIEW CONTEXT\n  Feature:                {feature}\n  Workflow/stage:         {stage}\n  Branch:                 {branch}\n  Base commit:            {base_sha}\n  Review commit:          {review_sha}{dirty_note}\n  Preceding numeric stage: {preceding_stage}\n  Evidence coverage:      {coverage}\n  Workspace dirty:        {dirty_text}\n",
        feature = opts.feature,
        stage = opts.stage,
        branch = branch,
        base_sha = match &opts.basis { ReviewBasis::Bounded { predecessor_packet_sha256, .. } => format!("reviewed-packet-sha256:{predecessor_packet_sha256}"), ReviewBasis::Ordinary { .. } => if base_sha.is_empty() { "(no base pin)".to_string() } else { base_sha.clone() } },
        review_sha = review_sha,
        preceding_stage = preceding_stage,
        coverage = coverage_state.as_str(),
        dirty_text = if workspace_dirty { "yes (uncommitted changes at review time)" } else { "no" },
    ));

    content.push_str("\nDBA RULES RELEVANT TO THIS WORKFLOW/STAGE\n");
    content.push_str("  - Your assessment is advisory and never makes a workflow decision.\n");
    content.push_str(
        "  - Memory is not truth — assess only what is provided, pinned to the review commit.\n",
    );
    content.push_str("  - Apply doctrine semantics only when the selected doctrine is included in the review evidence.\n");
    if matches!(
        coverage_state,
        CoverageState::PartialCoverage
            | CoverageState::SecretRedaction
            | CoverageState::CriticalOmission
    ) {
        content.push_str(
            "  - COVERAGE IS PARTIAL: some content was excluded/redacted (see below). You are\n",
        );
        content.push_str(
            "    seeing an incomplete evidence set — do not issue NO OBJECTION on this basis.\n",
        );
    }

    content.push_str(&format!("\nWORKFLOW/STAGE-SPECIFIC CHECKS\n{}\n", checks));
    content.push_str(&format!(
        "\nEXPECTED WORKFLOW/STAGE OUTPUT\n  {}\n",
        expected
    ));
    content.push_str("\nARTIFACTS TO REVIEW\n");
    content.push_str(&artifacts_block);

    if delta_mode {
        let base = match &opts.basis {
            ReviewBasis::Bounded {
                predecessor_packet_sha256,
                ..
            } => predecessor_packet_sha256.as_str(),
            ReviewBasis::Ordinary { base } => base.as_deref().unwrap_or("HEAD"),
        };
        content.push_str(&format!(
            "DELTA DIFF ({}->working tree, artifact paths only, secret/size filtered)\n",
            base
        ));
    } else {
        content.push_str("DIFF TO REVIEW (base->review, secret/size filtered)\n");
    }

    let excluded_summary: Vec<String> = excluded_paths.iter().map(|(p, _, _)| p.clone()).collect();
    if !excluded_summary.is_empty() {
        content.push_str(&format!(
            "  [excluded/redacted: {}] manual security review required\n",
            excluded_summary.join(", ")
        ));
    }
    content.push_str(&redacted_diff);

    if untracked_shown > 0 {
        content.push_str(&format!(
            "\nUNTRACKED FILES ({} new file(s) not present in any diff, shown in full)\n",
            untracked_shown
        ));
        content.push_str(&untracked_block);
    }

    // Full Context Diff (AC-1: auto-appended when delta_mode AND delta_base are both active).
    if let ReviewBasis::Ordinary { base: Some(base) } = &opts.basis {
        content.push_str(&format!(
            "\nFull Context Diff (informational — all changed files since {}):\n",
            base
        ));

        match git_diff_range(base, &[], &opts.repo_root) {
            Err(e) => {
                // Fail-closed: mark the section as unavailable rather than silently showing empty.
                content.push_str(&format!(
                    "[ERROR: git diff failed — full context diff unavailable: {}]\n",
                    e
                ));
            }
            Ok(full_raw) => {
                let (full_redacted, full_rc) = redact_secrets(&full_raw);
                if full_rc > 0 {
                    secret_flag = true;
                    redaction_count += full_rc;
                }

                // remaining = budget minus content bytes already counted (approximation;
                // task prompt and section headers are excluded from review_content_bytes).
                let remaining = budget.saturating_sub(review_content_bytes);
                let full_total = full_redacted.len() as u64;

                if full_total == 0 {
                    content.push_str("(no changes detected outside named artifacts)\n");
                } else if remaining == 0 {
                    content.push_str(&format!(
                            "[CLIPPED: full diff exceeded packet budget — showing first 0 of {} bytes]\n",
                            full_total
                        ));
                } else if full_total > remaining {
                    // Clip at a line boundary within the budget.
                    let head = &full_redacted[..remaining as usize];
                    let cutpoint = head
                        .rfind('\n')
                        .map(|n| n + 1)
                        .unwrap_or(remaining as usize);
                    content.push_str(&full_redacted[..cutpoint]);
                    content.push_str(&format!(
                            "\n[CLIPPED: full diff exceeded packet budget — showing first {} of {} bytes]\n",
                            cutpoint, full_total
                        ));
                } else {
                    content.push_str(&full_redacted);
                }
            }
        }
    }

    let final_base_sha = match &opts.basis {
        ReviewBasis::Bounded {
            predecessor_packet_sha256,
            ..
        } => format!("reviewed-packet-sha256:{predecessor_packet_sha256}"),
        ReviewBasis::Ordinary { .. } if base_sha.is_empty() => "(no base pin)".to_string(),
        ReviewBasis::Ordinary { .. } => base_sha,
    };

    Ok(ReviewPacket {
        feature: opts.feature.clone(),
        stage: opts.stage.clone(),
        branch,
        review_sha,
        base_sha: final_base_sha,
        diff_hash,
        coverage_state,
        workspace_dirty,
        redaction_count,
        secret_flag,
        artifacts,
        excluded_paths,
        bounded_changed_paths: match &opts.basis {
            ReviewBasis::Bounded { delta, .. } => Some(delta.changed_paths.clone()),
            _ => None,
        },
        review_content_bytes,
        estimated_review_tokens: estimated_tokens,
        budget_bytes: budget,
        over_budget: review_content_bytes > budget,
        diff_bytes,
        budget_contributors: file_contributors,
        content_sha256: sha256_str(&content),
        manifest: None,
        content,
    })
}

// --- git helpers ---

fn git_branch(repo_root: &str) -> Result<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_root)
        .output()
        .context("git rev-parse branch")?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_rev_parse(rev: &str, repo_root: &str) -> Result<String> {
    let out = Command::new("git")
        .args(["rev-parse", rev])
        .current_dir(repo_root)
        .output()
        .context("git rev-parse")?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_diff_range(base: &str, paths: &[String], repo_root: &str) -> Result<String> {
    let mut args = vec!["diff", base, "--"];
    for p in paths {
        args.push(p);
    }
    let out = Command::new("git")
        .args(&args)
        .current_dir(repo_root)
        .output()
        .context("git diff")?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn git_diff_names(base: &str, paths: &[String], repo_root: &str) -> Result<Vec<String>> {
    let mut args = vec!["diff", "--name-only", base, "--"];
    for p in paths {
        args.push(p);
    }
    args.extend_from_slice(&[
        ":(exclude).codeos/05-review/reviews",
        ":(exclude).codeos-state",
    ]);
    let out = Command::new("git")
        .args(&args)
        .current_dir(repo_root)
        .output()
        .context("git diff --name-only")?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect())
}

fn git_diff_head(repo_root: &str) -> Result<String> {
    let out = Command::new("git")
        .args(["diff", "HEAD", "--", "."])
        .current_dir(repo_root)
        .output()
        .context("git diff HEAD")?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn git_diff_names_head(repo_root: &str) -> Result<Vec<String>> {
    let out = Command::new("git")
        .args([
            "diff",
            "--name-only",
            "HEAD",
            "--",
            ".",
            ":(exclude).codeos/05-review/reviews",
            ":(exclude).codeos-state",
        ])
        .current_dir(repo_root)
        .output()
        .context("git diff --name-only HEAD")?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect())
}

fn git_diff_head_paths(paths: &[String], repo_root: &str) -> Result<String> {
    let mut args = vec!["diff", "HEAD", "--"];
    for p in paths {
        args.push(p);
    }
    let out = Command::new("git")
        .args(&args)
        .current_dir(repo_root)
        .output()
        .context("git diff HEAD paths")?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn git_diff_quiet(base: &str, path: &str, repo_root: &str) -> Result<bool> {
    let status = Command::new("git")
        .args(["diff", "--quiet", base, "--", path])
        .current_dir(repo_root)
        .status()
        .context("git diff --quiet")?;
    Ok(status.success())
}

fn git_is_tracked(path: &str, repo_root: &str) -> bool {
    Command::new("git")
        .args(["ls-files", "--error-unmatch", path])
        .current_dir(repo_root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Files git knows nothing about yet, honouring .gitignore. A diff cannot show them, so without
/// this they would be invisible to review while coverage still claimed to be full — which is how a
/// new implementation module can be added and reviewed as if it did not exist.
pub(crate) fn git_untracked_files(repo_root: &str) -> Option<Vec<String>> {
    let output = Command::new("git")
        .args([
            "ls-files",
            "--others",
            "--exclude-standard",
            "--",
            ".",
            ":(exclude).codeos/05-review/reviews",
            ":(exclude).codeos-state",
        ])
        .current_dir(repo_root)
        .output()
        .ok()?;
    // A non-zero exit is not an empty working tree: it means the answer is unknown. Returning an
    // empty list here is what let a packet claim full coverage while omitting every untracked file.
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect(),
    )
}

fn git_is_dirty(repo_root: &str) -> bool {
    Command::new("git")
        .args([
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--",
            ".",
            ":(exclude).codeos/05-review/reviews",
            ":(exclude).codeos-state",
        ])
        .current_dir(repo_root)
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

fn is_path_excluded(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);
    for pat in PATH_EXCLUDES {
        if glob_match(pat, path) || glob_match(pat, name) {
            return true;
        }
    }
    false
}

/// Whether `path` looks like a full runtime-event-log artifact, by the same glob this module
/// already uses to auto-exclude such files from untracked-file discovery (UPG-0074). Exposed so a
/// caller can warn when this kind of file is passed explicitly as a positional artifact, which is
/// how it reaches the packet at all — `PATH_EXCLUDES` only stops auto-discovery, not an explicit
/// pass.
pub(crate) fn looks_like_events_log(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);
    glob_match("*runtime_events*.jsonl", path) || glob_match("*runtime_events*.jsonl", name)
}

/// The full, unrestricted set of paths that changed in the reviewed diff — against `base` when
/// delta mode is in use, or against `HEAD` for a full review. Used to detect evidence outside the
/// declared artifact set (UPG-0074); deliberately not scoped to any path list, unlike
/// `git_diff_names`'s other callers, since scoping would hide exactly the drift this exists to
/// find.
pub(crate) fn diff_changed_paths(base: Option<&str>, repo_root: &str) -> Result<Vec<String>> {
    match base {
        Some(b) => git_diff_names(b, &[], repo_root),
        None => git_diff_names_head(repo_root),
    }
}

/// Minimal glob matching: `*` as a wildcard, any number of them, no other special characters.
/// UPG-0074 discovered that the previous single-wildcard-only implementation silently never
/// matched `PATH_EXCLUDES`'s own `"*runtime_events*.jsonl"` entry — a pattern with a leading and a
/// middle wildcard — because it only stripped one leading or one trailing `*` and compared the
/// rest literally, including the second `*` character. That exclusion has therefore never actually
/// applied since it was written; this restores the behavior the constant's own name and every
/// caller already assumed, not a new exclusion rule.
fn glob_match(pattern: &str, s: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return s == pattern;
    }
    let last = parts.len() - 1;
    let mut rest = s;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !rest.starts_with(part) {
                return false;
            }
            rest = &rest[part.len()..];
        } else if i == last {
            return rest.ends_with(part);
        } else {
            match rest.find(part) {
                Some(idx) => rest = &rest[idx + part.len()..],
                None => return false,
            }
        }
    }
    true
}

fn is_oversize(path: &str) -> bool {
    std::fs::metadata(path)
        .map(|m| m.len() > SIZE_LIMIT_BYTES)
        .unwrap_or(false)
}

pub fn sha256_file(path: &str) -> Result<String> {
    let bytes =
        std::fs::read(path).with_context(|| format!("could not read file for SHA: {}", path))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

pub fn sha256_str(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

fn stage_expected(stage: &str) -> &'static str {
    match stage {
        "framing" => "Solution Framing — problem, vision, candidate outcomes, scope, candidate constraints, and open architecture concerns; non-authoritative banner present; promoted claims are reviewed at the next applicable review point.",
        "decomposition" => "Feature Decomposition — one approved solution problem decomposed into candidate feature outcomes and boundaries; no feature IDs, lifecycle state, approval, or implementation detail.",
        "intake" => "Existing-Codebase Intake — draft Intent inputs derived from observation plus human intent; existing partial packages are valid and work continues through the normal Specification Package flow.",
        "charter" => "Solution Charter — approved problem, vision, primary actor and primary decision supported, measurable O-# outcomes, explicit scope boundary, and System Constraints each carrying a verification route; no feature behavior, guarantees, or architecture.",
        "architecture" => "Architecture Scope — the minimum project-level structure required before governed implementation, with non-empty duplicate-free feature membership; every decision derives from approved requirements or an explicit human architectural decision.",
        "1" => "Intent — actor+outcome statements, stable guarantees, explicit scope boundary; NO implementation detail.",
        "2" => "Behavioral contract — observable Given/When/Then scenarios, named failure modes, invariants; independently testable; no white-box claims.",
        "3" => "Specification Package — Intent, Contract, and Event Schema reviewed together for mutual consistency; observation mode is explicit; every Contract rule traces to Intent and every governed event traces to Contract.",
        "4" => "Implementation — code satisfying every Contract clause under the approved event or external-observation boundary; nothing governed is untraceable.",
        "5" => "Tests — applicable Contract scenarios and falsifiers covered through behavioral tests plus event replay or declared external-observation verification.",
        "6" => "Runtime evidence — representative bounded/sanitized evidence captured through the Contract's observation mode; unavailable evidence reported.",
        "7" => "Reconciliation — applicable layers compared using ALIGNED/GAP/MISMATCH/MISSING, evidence source runtime/test/static/none, and a plain explanatory note.",
        "8" => "Final verification — repeatable governed outcomes checked through event replay or external observation; nondeterminism and missing fixtures reported.",
        "9" => "Refinement — smallest effective change per observed trigger; no redesign disguised as refinement; affected artifacts named.",
        _ => "(no expected-output template for stage)",
    }
}

fn stage_checks(stage: &str) -> String {
    match stage {
        "framing" => "  - candidate outcomes and constraints are not approved; non-authoritative banner present; architecture concerns remain open; no components, flows, interfaces, technologies, governed artifacts, or implementation structure.".to_string(),
        "decomposition" => "  - approved Charter boundary is preserved; candidate outcomes and boundaries are distinct; no IDs, lifecycle state, approval, or implementation detail; each accepted candidate can proceed independently to Intent.".to_string(),
        "intake" => "  - normal draft Intent inputs used; observed behavior is not laundered into intent; infrastructure is not treated as a feature; normal Specification Package route named.".to_string(),
        "charter" => "  - for a new or materially revised Charter, does Decision Supported name the primary actor and primary decision, agree with Problem, Vision, Outcomes, and Scope, and remain an alignment lens rather than an acceptance requirement or feature behavior?\n  - outcomes are measurable results with stable O-# identities rather than features; the scope boundary is explicit in and out; every System Constraint is a product or system obligation with a stated verification route; every threshold states workload, operating context, and rationale; no feature behavior, guarantees, or architecture decision leaked in.\n  - when an approved Charter is being revised, is approval returned to null and is the impact assessment specific — an outcome change naming affected features through their recorded serves_outcomes, and a scope, boundary, or System Constraint change naming the approved artifacts that may be affected? judge the assessment's substance, not its presence.".to_string(),
        "architecture" => "  - only project-level structure is recorded; decisions local to one component are left to implementation even when reversing them would be costly; no behavior or quality requirement is invented, only the structural consequences approved requirements force; every listed feature has an approved Specification Package; unresolved responsibility, dependency-direction, data-authority, lifecycle, or integration conflicts keep approval null.\n  - a missing requirement is returned to its owning source — a behavioral gap to the affected Specification Package, a feature-specific quality requirement to that feature's Contract, a cross-cutting constraint to the Solution Charter — rather than resolved here.".to_string(),
        "1" => "  - actor/outcome clarity; no implementation detail; scope boundary explicit; stable guarantees clear; ambiguity flagged.".to_string(),
        "2" => "  - every intent outcome has observable contract coverage; failure paths named; invariants testable; no white-box claims.".to_string(),
        "3" => "  - Intent, Contract, and Event Schema are all present; observation mode is explicit; every Contract rule traces to Intent; every governed event traces to Contract; external-observation mode invents no placeholder events.".to_string(),
        "4" => "  - code traces to approved contract/schema only; no unapproved events; no hidden behavior; no unrelated files; evidence is sufficient for the next stage.\n  - did implementation resolve any question an approved artifact EXPLICITLY deferred? if so, is each material resolution recorded in a Deferral -> Resolution Trace (source deferral, resolution, where implemented, final/interim, expected superseder)? judge deferral by meaning, not by phrase; a missing record is a traceability finding, not an implementation failure.".to_string(),
        "5" => "  - behavior tested rather than private internals; applicable failures and invariants covered; observation-mode-specific verification present.".to_string(),
        "6" => "  - evidence captured without changing implementation; event or external-observation evidence bounded/sanitized; unavailable paths reported; a performance measurement used for a governed requirement demonstrates that the measured operation exercises the governed behavior.".to_string(),
        "7" => "  - only ALIGNED/GAP/MISMATCH/MISSING used; evidence source is runtime/test/static/none; each note supports the result and routes the gap; no ALIGNED claim is stronger than its cited observation.".to_string(),
        "8" => "  - verification follows observation mode; determinism excludes generated envelope fields unless contracted; missing fixtures reported; acceptance evidence observes the boundary named by the claim; performance evidence exercises the governed behavior; reconciliation claims no more than cited evidence proves.".to_string(),
        "9" => "  - trigger valid; proposed fix minimal; no redesign disguised as refinement; affected artifacts identified.".to_string(),
        _ => format!("  - (no stage-specific checklist for stage {})", stage),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_str_is_deterministic() {
        let h1 = sha256_str("hello");
        let h2 = sha256_str("hello");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn sha256_str_differs_for_different_inputs() {
        assert_ne!(sha256_str("hello"), sha256_str("world"));
    }

    #[test]
    fn coverage_state_str_values() {
        assert_eq!(CoverageState::FullCoverage.as_str(), "FULL_COVERAGE");
        assert_eq!(CoverageState::EmptyPacket.as_str(), "EMPTY_PACKET");
        assert_eq!(
            CoverageState::CriticalOmission.as_str(),
            "CRITICAL_OMISSION"
        );
    }

    #[test]
    fn path_excluded_env_file() {
        assert!(is_path_excluded("secrets/.env.local"));
        assert!(is_path_excluded("config.key"));
        assert!(!is_path_excluded("src/main.rs"));
    }

    #[test]
    fn glob_match_handles_a_pattern_with_leading_and_middle_wildcards() {
        // The regression this exists to guard: PATH_EXCLUDES' own "*runtime_events*.jsonl" never
        // matched anything before this fix, because the old implementation only stripped a single
        // leading OR trailing '*' and compared the remainder literally -- including the second
        // '*' character itself.
        assert!(glob_match("*runtime_events*.jsonl", "runtime_events.jsonl"));
        assert!(glob_match(
            "*runtime_events*.jsonl",
            "events/runtime_events.jsonl"
        ));
        assert!(!glob_match("*runtime_events*.jsonl", "runtime_events.txt"));
        assert!(!glob_match("*runtime_events*.jsonl", "other.jsonl"));
    }

    #[test]
    fn glob_match_single_wildcard_patterns_unchanged() {
        assert!(glob_match("*.env", "config.env"));
        assert!(!glob_match("*.env", "config.envelope"));
        assert!(glob_match("secrets/*", "secrets/api.key"));
        assert!(glob_match("foo", "foo"));
        assert!(!glob_match("foo", "foobar"));
    }

    #[test]
    fn looks_like_events_log_matches_the_full_path_and_the_bare_filename() {
        assert!(looks_like_events_log("events/runtime_events.jsonl"));
        assert!(looks_like_events_log("runtime_events.jsonl"));
        assert!(looks_like_events_log(
            ".codeos/05-review/events/runtime_events.jsonl"
        ));
    }

    #[test]
    fn looks_like_events_log_does_not_match_an_ordinary_artifact() {
        assert!(!looks_like_events_log("modules/source_fitness/src/lib.rs"));
        assert!(!looks_like_events_log(
            "tests/behavioral/F-0004_behavior.test.rs"
        ));
    }

    #[test]
    fn sha256_file_matches_content_hash() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().expect("tempfile");
        f.write_all(b"test content").expect("write");
        let path = f.path().to_str().expect("path");
        let file_hash = sha256_file(path).expect("hash");
        let str_hash = sha256_str("test content");
        assert_eq!(file_hash, str_hash);
    }

    #[test]
    fn stage_expected_new_downstream_identifiers_are_real_not_placeholder() {
        for stage in [
            "framing",
            "decomposition",
            "intake",
            "charter",
            "architecture",
        ] {
            let text = stage_expected(stage);
            assert_ne!(
                text, "(no expected-output template for stage)",
                "stage '{}' must have real expected-output text",
                stage
            );
        }
    }

    #[test]
    fn stage_checks_new_downstream_identifiers_are_real_not_placeholder() {
        for stage in [
            "framing",
            "decomposition",
            "intake",
            "charter",
            "architecture",
        ] {
            let text = stage_checks(stage);
            assert!(
                !text.contains("no stage-specific checklist"),
                "stage '{}' must have real checklist text",
                stage
            );
        }
    }

    #[test]
    fn framing_token_reviews_solution_framing_without_architecture_decisions() {
        let expected = stage_expected("framing");
        let checks = stage_checks("framing");
        assert!(expected.contains("Solution Framing"));
        assert!(expected.contains("candidate outcomes"));
        assert!(expected.contains("open architecture concerns"));
        assert!(checks.contains("architecture concerns remain open"));
        for forbidden in ["components", "flows", "interfaces", "technologies"] {
            assert!(checks.contains(forbidden));
        }
    }

    #[test]
    fn charter_token_reviews_purpose_without_feature_behavior_or_architecture() {
        let expected = stage_expected("charter");
        let checks = stage_checks("charter");
        assert!(expected.contains("Solution Charter"));
        assert!(expected.contains("primary actor and primary decision supported"));
        assert!(expected.contains("O-# outcomes"));
        assert!(expected.contains("verification route"));
        assert!(checks.contains("new or materially revised Charter"));
        assert!(checks.contains("Problem, Vision, Outcomes, and Scope"));
        assert!(checks.contains("alignment lens rather than an acceptance requirement"));
        // The Charter owns purpose, never feature behavior or structure.
        assert!(checks.contains("no feature behavior, guarantees, or architecture decision"));
        // Revision is the same boundary, so the impact assessment is reviewable from the
        // Charter itself rather than from a packet carrying every approved Intent.
        assert!(checks.contains("serves_outcomes"));
        assert!(checks.contains("approval returned to null"));
    }

    #[test]
    fn architecture_token_reviews_project_level_structure_only() {
        let expected = stage_expected("architecture");
        let checks = stage_checks("architecture");
        assert!(expected.contains("Architecture Scope"));
        assert!(expected.contains("feature membership"));
        // Reversal cost decides visibility, not authority (doctrine v3 Structural Ownership).
        assert!(checks.contains("even when reversing them would be costly"));
        // Architecture responds to requirements; it never originates them.
        assert!(checks.contains("no behavior or quality requirement is invented"));
        assert!(checks.contains("returned to its owning source"));
    }

    #[test]
    fn decision_boundary_tokens_are_distinct_from_each_other() {
        // A shared vocabulary is only useful if each identifier selects its own text.
        let all = [
            stage_checks("charter"),
            stage_checks("architecture"),
            stage_checks("framing"),
        ];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b, "decision-boundary checklists must not collide");
            }
        }
    }

    #[test]
    fn stage_expected_numeric_1_to_9_all_have_templates() {
        for stage in ["1", "2", "3", "4", "5", "6", "7", "8", "9"] {
            let text = stage_expected(stage);
            assert_ne!(
                text, "(no expected-output template for stage)",
                "numeric stage '{}' must still have its existing expected-output text",
                stage
            );
        }
    }

    #[test]
    fn stage_10_has_no_current_reviewer_protocol() {
        assert_eq!(
            stage_expected("10"),
            "(no expected-output template for stage)"
        );
        assert!(stage_checks("10").contains("no stage-specific checklist"));
    }

    #[test]
    fn stage_3_reviews_the_complete_specification_package() {
        let expected = stage_expected("3");
        let checks = stage_checks("3");
        assert!(expected.contains("Specification Package"));
        assert!(expected.contains("Intent, Contract, and Event Schema"));
        assert!(checks.contains("every Contract rule traces to Intent"));
        assert!(checks.contains("every governed event traces to Contract"));
        assert!(checks.contains("observation mode is explicit"));
        assert!(checks.contains("external-observation mode invents no placeholder events"));
    }

    #[test]
    fn stage_8_reviewer_checks_semantics_without_making_them_preflight_logic() {
        let checks = stage_checks("8");
        assert!(checks.contains("acceptance evidence observes the boundary named by the claim"));
        assert!(checks.contains("performance evidence exercises the governed behavior"));
        assert!(checks.contains("reconciliation claims no more than cited evidence proves"));
    }

    #[test]
    fn numeric_reviewer_checks_do_not_define_doctrine_decisions() {
        for stage in ["1", "2", "3", "4", "5", "6", "7", "8", "9"] {
            let text = format!("{}\n{}", stage_expected(stage), stage_checks(stage)).to_lowercase();
            for duplicated_rule in ["human approval", "final acceptance", "intermediate gate"] {
                assert!(
                    !text.contains(duplicated_rule),
                    "stage {stage} independently defines doctrine rule {duplicated_rule}: {text}"
                );
            }
        }
    }

    #[test]
    fn stage_4_checklist_asks_for_the_deferral_resolution_trace() {
        // UPG-0063: the Stage 4 reviewer asks the question actively rather than relying on the
        // implementation author remembering the obligation.
        let text = stage_checks("4");
        assert!(
            text.contains("EXPLICITLY deferred"),
            "must ask about explicit deferrals: {text}"
        );
        assert!(
            text.contains("Deferral -> Resolution Trace"),
            "must name the trace: {text}"
        );
        assert!(
            text.contains("final/interim"),
            "must ask for the interim marker: {text}"
        );
        assert!(
            text.contains("expected superseder"),
            "must ask what supersedes an interim: {text}"
        );
        // Advisory, not a behavioral gate.
        assert!(
            text.contains("traceability finding, not an implementation failure"),
            "must keep the finding advisory: {text}"
        );
        // Judged semantically — the checklist must not hand the reviewer a phrase list.
        assert!(
            text.contains("by meaning, not by phrase"),
            "must forbid phrase-matching: {text}"
        );
        // The pre-existing Stage 4 checks survive.
        assert!(text.contains("code traces to approved contract/schema only"));
    }

    #[test]
    fn stage_4_deferral_question_is_scoped_to_stage_4_only() {
        // No other stage's checklist mentions the trace.
        for stage in [
            "framing",
            "decomposition",
            "intake",
            "charter",
            "architecture",
            "1",
            "2",
            "3",
            "5",
            "6",
            "7",
            "8",
            "9",
            "10",
        ] {
            assert!(
                !stage_checks(stage).contains("Deferral -> Resolution Trace"),
                "stage '{stage}' must not carry the Stage 4 deferral question"
            );
        }
    }

    #[test]
    fn stage_checks_unrecognized_identifier_still_falls_back_to_placeholder() {
        let text = stage_checks("nonexistent-stage");
        assert!(text.contains("no stage-specific checklist for stage nonexistent-stage"));
    }

    #[test]
    fn reader_output_guidance_follows_the_selected_reader_output_policy() {
        let tk = tempfile::tempdir().expect("toolkit dir");
        let root = tk.path();
        std::fs::create_dir_all(root.join("dba/00-entry/configurations")).unwrap();
        std::fs::create_dir_all(root.join("dba/05-guidance")).unwrap();
        std::fs::create_dir_all(root.join("dba/02-policies/reader-output")).unwrap();
        std::fs::write(
            root.join("dba/05-guidance/reader-oriented-output.md"),
            "# unversioned fallback\n",
        )
        .unwrap();
        std::fs::write(
            root.join("dba/02-policies/reader-output/v1.md"),
            "# selected reader output policy\n",
        )
        .unwrap();

        // No resolver -> fallback.
        assert_eq!(
            resolve_reader_output_guidance(root, root),
            root.join("dba/05-guidance/reader-oriented-output.md")
        );

        std::fs::create_dir_all(root.join("dba/04-tools/configuration")).unwrap();
        std::fs::create_dir_all(root.join(".codeos/00-project")).unwrap();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../configuration/resolve-project-configuration.sh"),
            root.join("dba/04-tools/configuration/resolve-project-configuration.sh"),
        )
        .unwrap();

        // A legacy schema-less project resolves to DBA-7; without a selected policy it falls back.
        std::fs::write(
            root.join("dba/00-entry/configurations/DBA-7.yaml"),
            "doctrine: dba/01-doctrine/v7.md\nstatus: approved\n",
        )
        .unwrap();
        assert_eq!(
            resolve_reader_output_guidance(root, root),
            root.join("dba/05-guidance/reader-oriented-output.md")
        );

        // The same canonical resolver supplies the selected policy.
        std::fs::write(
            root.join("dba/00-entry/configurations/DBA-7.yaml"),
            "doctrine: dba/01-doctrine/v7.md\n\
             reader_output_policy: dba/02-policies/reader-output/v1.md\nstatus: approved\n",
        )
        .unwrap();
        assert_eq!(
            resolve_reader_output_guidance(root, root),
            root.join("dba/02-policies/reader-output/v1.md")
        );
    }

    #[test]
    fn reviewed_dirty_workspace_is_the_delta_baseline_instead_of_head() {
        let repo = tempfile::tempdir().unwrap();
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
        ] {
            assert!(Command::new("git")
                .args(args)
                .current_dir(repo.path())
                .status()
                .unwrap()
                .success());
        }
        std::fs::write(repo.path().join("target.txt"), "head\n").unwrap();
        std::fs::write(repo.path().join("historical.txt"), "head history\n").unwrap();
        assert!(Command::new("git")
            .args(["add", "."])
            .current_dir(repo.path())
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .args(["commit", "-qm", "base"])
            .current_dir(repo.path())
            .status()
            .unwrap()
            .success());
        let commit = git_rev_parse("HEAD", repo.path().to_str().unwrap()).unwrap();
        std::fs::write(repo.path().join("target.txt"), "reviewed\n").unwrap();
        std::fs::write(repo.path().join("historical.txt"), "reviewed history\n").unwrap();
        let reviewed_diff = git_diff_range("HEAD", &[], repo.path().to_str().unwrap()).unwrap();
        let packet =
            format!("header\nDIFF TO REVIEW (base->review, secret/size filtered)\n{reviewed_diff}");
        let packet_path = repo.path().join("reviewed.packet.txt");
        std::fs::write(&packet_path, &packet).unwrap();
        let snapshot = ReviewedSnapshot {
            full_file_preimages: false,
            lineage: vec![ReviewedPacketRecord {
                packet_path,
                packet_sha256: sha256_str(&packet),
                review_commit: commit,
            }],
        };
        std::fs::write(repo.path().join("target.txt"), "reviewed\nrepair\n").unwrap();

        let delta = reviewed_snapshot_diff(
            &snapshot,
            &["target.txt".into()],
            repo.path().to_str().unwrap(),
        )
        .unwrap();

        assert_eq!(
            delta.changed_paths,
            BTreeSet::from(["target.txt".to_string()])
        );
        assert!(delta.diff.contains("+repair"), "{}", delta.diff);
        assert!(!delta.diff.contains("-head"), "{}", delta.diff);
        let head_diff = git_diff_range("HEAD", &[], repo.path().to_str().unwrap()).unwrap();
        assert!(head_diff.contains("historical.txt"));
        assert!(head_diff.len() > delta.diff.len());
    }

    #[test]
    fn reviewed_untracked_full_file_is_reconstructible_only_under_v17() {
        let repo = tempfile::tempdir().unwrap();
        let root = repo.path().to_str().unwrap();
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
        ] {
            assert!(Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success());
        }
        std::fs::write(repo.path().join("tracked.txt"), "base\n").unwrap();
        for args in [vec!["add", "."], vec!["commit", "-qm", "base"]] {
            assert!(Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success());
        }
        let commit = git_rev_parse("HEAD", root).unwrap();
        // The reviewed workspace held an untracked evidence file, embedded in full.
        let reviewed = "{\"observation\":1}\n";
        std::fs::write(repo.path().join("evidence.json"), reviewed).unwrap();
        let sha = sha256_str(reviewed);
        let packet = format!(
            "PACKET MANIFEST\n  items:\n    - path: evidence.json\n      mode: full_file\n      bytes: {}\n      sha256: {sha}\n\nARTIFACTS TO REVIEW\n  --- evidence.json (sha256: {sha}, visibility: shown) ---\n    {{\"observation\":1}}\n\nDIFF TO REVIEW (base->review, secret/size filtered)\n",
            reviewed.len()
        );
        let packet_path = repo.path().join("reviewed.packet.txt");
        std::fs::write(&packet_path, &packet).unwrap();
        let snapshot = |full_file_preimages| ReviewedSnapshot {
            full_file_preimages,
            lineage: vec![ReviewedPacketRecord {
                packet_path: packet_path.clone(),
                packet_sha256: sha256_str(&packet),
                review_commit: commit.clone(),
            }],
        };
        let paths = ["evidence.json".to_string()];

        // Without full-file pre-images the reviewed file is misreported as new material.
        let legacy = reviewed_snapshot_diff(&snapshot(false), &paths, root).unwrap();
        assert!(legacy.new_paths.contains("evidence.json"));

        let unchanged = reviewed_snapshot_diff(&snapshot(true), &paths, root).unwrap();
        assert!(unchanged.changed_paths.is_empty(), "{}", unchanged.diff);

        std::fs::write(repo.path().join("evidence.json"), "{\"observation\":2}\n").unwrap();
        let repaired = reviewed_snapshot_diff(&snapshot(true), &paths, root).unwrap();
        assert_eq!(
            repaired.changed_paths,
            BTreeSet::from(["evidence.json".to_string()])
        );
        assert!(
            repaired.new_paths.is_empty(),
            "reviewed file must not count as new"
        );
        assert!(
            repaired.diff.contains("+{\"observation\":2}"),
            "{}",
            repaired.diff
        );
    }
}
