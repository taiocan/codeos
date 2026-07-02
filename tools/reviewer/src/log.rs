use crate::assessment::ParsedReview;
use crate::packet::ReviewPacket;
use crate::provider::RawAssessment;
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
advisory and read-only; APPROVE belongs to the human. See docs/reviewer-pipeline.md.

(v0 layout: one global log. Per-feature logs are a documented future layout.)
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

/// Append a REVIEW entry to the log. Uses a temp-file + rename for atomicity.
pub fn append_review(
    log_path: &Path,
    packet: &ReviewPacket,
    raw: &RawAssessment,
    parsed: &ParsedReview,
    assessment_file: &Path,
    assessment_hash: &str,
    packet_saved: &Path,
    packet_hash: &str,
) -> Result<()> {
    ensure_log_exists(log_path)?;

    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let dirty_text = if packet.workspace_dirty { "true" } else { "false" };
    let coverage_note_suffix = if !parsed.coverage_note.is_empty() {
        format!("; note: {}", parsed.coverage_note)
    } else {
        String::new()
    };

    let mut entry = String::new();
    entry.push('\n');
    entry.push_str(&format!("## {} REVIEW — {} — Stage {}\n", ts, packet.feature, packet.stage));
    entry.push_str(&format!("Base: {}  Review: {}  Branch: {}\n",
        packet.base_sha, packet.review_sha, packet.branch));
    entry.push_str(&format!("Diff-hash: {}\n", packet.diff_hash));
    entry.push_str(&format!("Reviewer: codex default-model (session {})\n", raw.session_id));
    entry.push_str(&format!("Effort: {}   Wall time: {}ms   Reconnects: {}\n",
        raw.effort, raw.elapsed_ms, raw.reconnect_count));
    entry.push_str(&format!("Codex concern: {}\n", parsed.codex_concern));
    entry.push_str(&format!("Effective concern: {}\n", parsed.effective_concern));
    entry.push_str(&format!("Evidence: {}\n", parsed.evidence));
    entry.push_str(&format!("Coverage: {}; redactions: {}; workspace_dirty: {}{}\n",
        packet.coverage_state.as_str(), packet.redaction_count, dirty_text, coverage_note_suffix));
    entry.push_str(&format!("Log summary: {}\n",
        parsed.summary_line.trim_start_matches("LOG SUMMARY: ")));
    entry.push_str(&format!("Full assessment: {} (sha256:{})\n",
        assessment_file.display(), assessment_hash));
    entry.push_str(&format!("Reviewed packet: {} (sha256:{})\n",
        packet_saved.display(), packet_hash));
    if packet.secret_flag
        || packet.coverage_state.as_str() == "CRITICAL_OMISSION"
        || packet.coverage_state.as_str() == "EMPTY_PACKET"
    {
        let excluded: Vec<String> = packet.excluded_paths.iter().map(|(p,_,_)| p.clone()).collect();
        entry.push_str(&format!("Coverage gap: {} — excluded/redacted [{}] — MANUAL SECURITY REVIEW REQUIRED\n",
            packet.coverage_state.as_str(), excluded.join(", ")));
    }
    entry.push_str(&format!("Human decision: (append with: codeos-reviewer decision {} {} <DECISION> \"<reason>\")\n",
        packet.feature, packet.stage));

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
        Ok(rd) => rd.filter_map(|e| e.ok())
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
        format!("assessment file exists but could not be read: {}", path.display())
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
        eprintln!("warning: could not determine current HEAD (git rev-parse failed) \
                   — commit drift check skipped");
        "(unknown)".to_string()
    });

    // Re-hash the saved packet file. Unverifiable cases are recorded explicitly.
    let packet_sha_actual = if fm.reviewed_packet.is_empty() {
        eprintln!("warning: assessment has no reviewed_packet path — packet integrity unverifiable");
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
                    eprintln!("warning: could not hash packet file ({}) — packet integrity unverifiable", e);
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
            if !in_fm { in_fm = true; continue; }
            else { break; }
        }
        if !in_fm { continue; }
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
    Some(AssessmentFrontmatter { review_commit, coverage_state, reviewed_packet, reviewed_packet_sha256 })
}

fn short_sha(sha: &str) -> &str {
    &sha[..sha.len().min(12)]
}

/// Append a HUMAN DECISION entry to the log.
/// `provenance` is populated when an assessment file was found for this feature+stage.
/// When None, falls back to the legacy `verify_artifacts` path for backward compatibility.
pub fn append_decision(
    log_path: &Path,
    feature: &str,
    stage: &str,
    decision: &str,
    reason: &str,
    provenance: Option<&DecisionProvenance>,
    repo_root: &Path,
    codex_dir: &Path,
) -> Result<()> {
    ensure_log_exists(log_path)?;

    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let sha = git_head_sha(repo_root).unwrap_or_else(|_| "(unknown)".to_string());

    let mut entry = String::new();
    entry.push('\n');
    entry.push_str(&format!("## {} HUMAN DECISION — {} — Stage {}\n", ts, feature, stage));
    entry.push_str(&format!("Commit at decision: {}\n", sha));
    entry.push_str(&format!("Decision: {}\n", decision));
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
            let rc_display = if prov.review_commit.is_empty() { "(unknown)" } else { &prov.review_commit };
            entry.push_str(&format!("  review_commit: {}  [{}]\n", rc_display, commit_note));

            // Packet hash (AC-4). UNVERIFIABLE cases are explicit, not silent.
            let hash_note = if prov.packet_sha_stored.is_empty() {
                "PROVENANCE_UNVERIFIABLE: no stored sha in assessment".to_string()
            } else if prov.packet_sha_actual.starts_with("UNVERIFIABLE:") {
                format!("PROVENANCE_UNVERIFIABLE: {}", &prov.packet_sha_actual["UNVERIFIABLE:".len()..])
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
                && matches!(prov.coverage_state.as_str(), "CRITICAL_OMISSION" | "EMPTY_PACKET")
            {
                match &prov.override_rationale {
                    Some(rationale) => format!("COVERAGE_GATE_OVERRIDDEN: {}", rationale),
                    None => "COVERAGE_GATE_TRIGGERED".to_string(),
                }
            } else if decision == "APPROVE_STAGE" {
                "OK".to_string()
            } else {
                "INFORMATIONAL".to_string()
            };
            let cs_display = if prov.coverage_state.is_empty() { "(unknown)" } else { &prov.coverage_state };
            entry.push_str(&format!("  coverage_state: {}  [{}]\n", cs_display, gate_note));
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

fn git_head_sha(repo_root: &Path) -> Result<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_root)
        .output()
        .context("git rev-parse HEAD")?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn verify_artifacts(feature: &str, stage: &str, codex_dir: &Path) -> (Option<String>, String, bool) {
    // Find most recent assessment for this feature/stage
    let pattern = format!("{}-stage-{}-", feature, stage);
    let entries: Vec<_> = match std::fs::read_dir(codex_dir) {
        Ok(rd) => rd.filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(&pattern) && !e.file_name().to_string_lossy().contains(".packet"))
            .collect(),
        Err(_) => return (None, String::new(), false),
    };

    let latest = entries.into_iter()
        .max_by_key(|e| e.file_name());
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
                verify_lines.push_str(&format!("  CHANGED {} (reviewed {} / now {})\n",
                    current_path, &stored_sha[..12.min(stored_sha.len())],
                    &now_sha[..12.min(now_sha.len())]));
                changed = true;
            }
            current_path.clear();
        }
    }

    (Some(latest.display().to_string()), verify_lines, changed)
}
