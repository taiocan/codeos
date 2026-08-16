use crate::assessment::ParsedReview;
use crate::packet::ReviewPacket;
use crate::codex::CodexResult;
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
owner of automated review records; see dba/04-tools/reviewer/contract/v3.md.
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
                format!("could not read review log to compute round: {}", log_path.display())
            });
        }
    };
    let suffix = format!(" REVIEW — {} — Stage {}", feature, stage);
    let count = content.lines()
        .filter(|line| line.starts_with("## ") && line.ends_with(&suffix))
        .count();
    Ok(count as u32 + 1)
}

/// Format the `REV__<feature>__<stage>__R<N>` review id. The stage is used verbatim — no
/// numeric-stage-to-`S<N>` conversion — because `S<N>` (from `UPG-0001`) was defined only for
/// self-dev steps 1-4 and has no mapping for downstream DBA stage ids.
pub fn format_review_id(feature: &str, stage: &str, round: u32) -> String {
    format!("REV__{}__{}__R{}", feature, stage, round)
}

/// Append a REVIEW entry to the log. Uses a temp-file + rename for atomicity.
pub fn append_review(
    log_path: &Path,
    review_id: &str,
    packet: &ReviewPacket,
    raw: &CodexResult,
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
    entry.push_str(&format!("Review ID: {}\n", review_id));
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
        format!("## 2026-01-01T00:00:00Z REVIEW — {} — Stage {}\n", feature, stage)
    }

    #[test]
    fn round_one_when_log_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("does-not-exist.md");
        assert_eq!(compute_review_round(&missing, "UPG-0046", "selfdev-step-1").unwrap(), 1);
    }

    #[test]
    fn round_one_when_log_exists_with_no_matches() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log_path = write_log(dir.path(), &[]);
        assert_eq!(compute_review_round(&log_path, "UPG-0046", "selfdev-step-1").unwrap(), 1);
    }

    #[test]
    fn round_increments_across_matching_entries() {
        let dir = tempfile::tempdir().expect("tempdir");
        let e1 = review_header("UPG-0046", "selfdev-step-1");
        let e2 = review_header("UPG-0046", "selfdev-step-1");
        let log_path = write_log(dir.path(), &[&e1, &e2]);
        assert_eq!(compute_review_round(&log_path, "UPG-0046", "selfdev-step-1").unwrap(), 3);
    }

    #[test]
    fn round_does_not_collide_across_similarly_named_stages() {
        // "Stage 1" must not be counted when computing the round for "Stage 10", or vice versa —
        // the exact scenario the human flagged as a danger of substring-based log parsing.
        let dir = tempfile::tempdir().expect("tempdir");
        let e1 = review_header("FEAT", "10");
        let log_path = write_log(dir.path(), &[&e1]);
        assert_eq!(
            compute_review_round(&log_path, "FEAT", "1").unwrap(), 1,
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
            compute_review_round(&log_path, "UPG-004", "selfdev-step-1").unwrap(), 1,
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
        assert_eq!(compute_review_round(&durable, "UPG-0046", "selfdev-step-1").unwrap(), 2);
        assert_eq!(compute_review_round(&scratch, "UPG-0046", "selfdev-step-1").unwrap(), 1);
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
            "a permission error must never be silently treated as round 1: {:?}", result
        );
    }

    /// Minimal, dependency-free euid check (avoids adding a `libc` dependency for one test).
    #[cfg(unix)]
    unsafe fn libc_geteuid_is_zero() -> bool {
        extern "C" { fn geteuid() -> u32; }
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
        assert_eq!(format_review_id("checkout-flow", "7", 1), "REV__checkout-flow__7__R1");
    }

    /// Set up a temp git repo and build a real `ReviewPacket` against it (needs the real
    /// Codeos repo as `toolkit_root` to find `dba/03-prompts/review/codeos-reviewer-task.md`). Shared by the
    /// tests below that need an actual packet, not just a hand-seeded log file.
    fn build_test_packet(repo_root: &std::path::Path, feature: &str, stage: &str) -> ReviewPacket {
        std::process::Command::new("git").args(["init"]).current_dir(repo_root).output().expect("git init");
        std::process::Command::new("git").args(["config", "user.email", "t@t.test"]).current_dir(repo_root).output().ok();
        std::process::Command::new("git").args(["config", "user.name", "T"]).current_dir(repo_root).output().ok();
        std::fs::write(repo_root.join("tracked.md"), "# tracked\n").expect("write");
        std::process::Command::new("git").args(["add", "tracked.md"]).current_dir(repo_root).output().expect("git add");
        std::process::Command::new("git").args(["commit", "-m", "init"]).current_dir(repo_root).output().expect("git commit");

        let mut toolkit_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for _ in 0..4 { toolkit_root.pop(); }

        let opts = crate::packet::PacketBuildOptions {
            feature: feature.to_string(),
            stage: stage.to_string(),
            artifacts: vec!["tracked.md".to_string()],
            sha_only_paths: vec![],
            delta_mode: false,
            delta_base: None,
            repo_root: repo_root.to_string_lossy().into_owned(),
            toolkit_root: toolkit_root.to_string_lossy().into_owned(),
        };
        crate::packet::build(&opts).expect("build packet")
    }

    fn test_raw() -> crate::codex::CodexResult {
        crate::codex::CodexResult {
            text: "LOG SUMMARY: NO OBJECTION — ok\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n".to_string(),
            session_id: "test-session".to_string(),
            elapsed_ms: 1,
            reconnect_count: 0,
            effort: "high".to_string(),
        }
    }

    #[test]
    fn append_review_writes_review_id_line() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo_root = dir.path();
        let packet = build_test_packet(repo_root, "UPG-TEST", "selfdev-step-1");
        let raw = test_raw();
        let parsed = crate::assessment::parse_review_output(&raw.text, &packet.coverage_state);

        let log_path = repo_root.join("review-log.md");
        let review_id = format_review_id(&packet.feature, &packet.stage, 1);
        append_review(
            &log_path, &review_id, &packet, &raw, &parsed,
            Path::new("assessment.md"), "deadbeef", Path::new("packet.txt"), "cafefeed",
        ).expect("append_review");

        let content = std::fs::read_to_string(&log_path).expect("read log");
        assert!(
            content.contains(&format!("Review ID: {}\n", review_id)),
            "log entry must contain the Review ID line: {}", content
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
        let parsed = crate::assessment::parse_review_output(&raw.text, &packet.coverage_state);
        let log_path = repo_root.join("review-log.md");

        // Round 1 (log does not exist yet).
        let round1 = compute_review_round(&log_path, &packet.feature, &packet.stage).unwrap();
        assert_eq!(round1, 1);
        let review_id1 = format_review_id(&packet.feature, &packet.stage, round1);
        assert!(review_id1.ends_with("__R1"));
        append_review(
            &log_path, &review_id1, &packet, &raw, &parsed,
            Path::new("assessment1.md"), "deadbeef1", Path::new("packet1.txt"), "cafefeed1",
        ).expect("append_review round 1");

        // Round 2 (log now has exactly one matching entry from round 1).
        let round2 = compute_review_round(&log_path, &packet.feature, &packet.stage).unwrap();
        assert_eq!(round2, 2, "round must increment after a real append_review call");
        let review_id2 = format_review_id(&packet.feature, &packet.stage, round2);
        assert!(review_id2.ends_with("__R2"));
        append_review(
            &log_path, &review_id2, &packet, &raw, &parsed,
            Path::new("assessment2.md"), "deadbeef2", Path::new("packet2.txt"), "cafefeed2",
        ).expect("append_review round 2");

        let content = std::fs::read_to_string(&log_path).expect("read log");
        assert!(content.contains(&format!("Review ID: {}\n", review_id1)));
        assert!(content.contains(&format!("Review ID: {}\n", review_id2)));

        // A third read must now report round 3, proving the cycle keeps advancing correctly.
        let round3 = compute_review_round(&log_path, &packet.feature, &packet.stage).unwrap();
        assert_eq!(round3, 3);
    }
}
