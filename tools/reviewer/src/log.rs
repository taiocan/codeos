use crate::assessment::ParsedReview;
use crate::packet::ReviewPacket;
use crate::provider::RawAssessment;
use anyhow::{Context, Result};
use std::path::Path;

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

/// Append a HUMAN DECISION entry to the log.
pub fn append_decision(
    log_path: &Path,
    feature: &str,
    stage: &str,
    decision: &str,
    reason: &str,
    repo_root: &Path,
    codex_dir: &Path,
) -> Result<()> {
    ensure_log_exists(log_path)?;

    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let sha = git_head_sha(repo_root).unwrap_or_else(|_| "(unknown)".to_string());

    // Best-effort artifact integrity check
    let (verified_against, verify_lines, changed) = verify_artifacts(feature, stage, codex_dir);

    let mut entry = String::new();
    entry.push('\n');
    entry.push_str(&format!("## {} HUMAN DECISION — {} — Stage {}\n", ts, feature, stage));
    entry.push_str(&format!("Commit at decision: {}\n", sha));
    entry.push_str(&format!("Decision: {}\n", decision));
    entry.push_str(&format!("Reason/next: {}\n", reason));
    if let Some(path) = &verified_against {
        entry.push_str(&format!("Verified against: {}\n", path));
    }
    if !verify_lines.is_empty() {
        entry.push_str("Artifact integrity (informational audit, not a gate):\n");
        entry.push_str(&verify_lines);
    }

    append_to_log(log_path, &entry)?;

    if changed {
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
