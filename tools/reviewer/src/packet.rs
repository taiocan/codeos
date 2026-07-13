use crate::precheck::redact_secrets;
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;

const SIZE_LIMIT_BYTES: u64 = 256 * 1024;

const PATH_EXCLUDES: &[&str] = &[
    "*.env", ".env*", "*.pem", "*.key", "secrets/*", "credentials/*",
    "*runtime_events*.jsonl", "*.log",
];

#[derive(Debug, Clone, PartialEq)]
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
            CoverageState::FullCoverage     => "FULL_COVERAGE",
            CoverageState::PartialCoverage  => "PARTIAL_COVERAGE",
            CoverageState::SecretRedaction  => "SECRET_REDACTION",
            CoverageState::CriticalOmission => "CRITICAL_OMISSION",
            CoverageState::EmptyPacket      => "EMPTY_PACKET",
        }
    }

    /// Severity floor for effective_concern escalation.
    pub fn concern_floor(&self) -> u8 {
        match self {
            CoverageState::FullCoverage                             => 0,
            CoverageState::SecretRedaction | CoverageState::PartialCoverage => 1,
            CoverageState::EmptyPacket                              => 2,
            CoverageState::CriticalOmission                         => 3,
        }
    }
}

#[derive(Debug)]
pub struct ArtifactEntry {
    pub path: String,
    pub sha256: String,
    pub visibility: String,
    /// Size in bytes as reported by `fs::metadata` at build time. `0` for `missing` artifacts
    /// (no file to stat).
    pub bytes: u64,
}

pub struct PacketManifestEntry {
    pub path: String,
    pub mode: String,
    pub bytes: u64,
    pub sha256: String,
    pub reason: Option<String>,
}

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
    content: String,
}

impl ReviewPacket {
    pub fn content(&self) -> &str {
        &self.content
    }
}

pub struct PacketBuildOptions {
    pub feature: String,
    pub stage: String,
    pub artifacts: Vec<String>,
    pub sha_only_paths: Vec<String>,
    pub delta_mode: bool,
    pub delta_base: Option<String>,
    pub fresh_session: bool,
    pub repo_root: String,
    pub toolkit_root: String,
}

pub fn build(opts: &PacketBuildOptions) -> Result<ReviewPacket> {
    // Validate sha_only paths exist before doing anything.
    for so in &opts.sha_only_paths {
        if !Path::new(so).exists() {
            bail!("--sha-only path not found: {}", so);
        }
    }

    let branch = git_branch(&opts.repo_root)?;
    let review_sha = git_rev_parse("HEAD", &opts.repo_root)?;
    let approved_stage = if opts.stage.parse::<u32>().is_ok() {
        let n: u32 = opts.stage.parse().unwrap();
        format!("{}", n.saturating_sub(1))
    } else {
        "n/a (non-numeric stage)".to_string()
    };

    let base_sha = load_base_sha(
        &opts.feature, &opts.stage, &opts.repo_root, opts.delta_base.as_deref()
    )?;

    // Build diff
    let (_raw_diff, changed_files) = if opts.delta_mode {
        let base = opts.delta_base.as_deref().unwrap_or("HEAD");
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

    let mut excluded_paths: Vec<(String, String, String)> = Vec::new();
    let mut keep_files: Vec<String> = Vec::new();
    for f in &changed_files {
        if is_path_excluded(f) || is_oversize(f) {
            excluded_paths.push((f.clone(), "path/size excluded".to_string(), "diff".to_string()));
        } else {
            keep_files.push(f.clone());
        }
    }

    let filtered_diff = if !keep_files.is_empty() {
        if opts.delta_mode {
            let base = opts.delta_base.as_deref().unwrap_or("HEAD");
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
        excluded_paths.push(("(diff)".to_string(), "secret-like content redacted".to_string(), "diff".to_string()));
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
            artifacts_block.push_str(&format!("  --- {} (visibility: missing — not shown) ---\n\n", a));
            artifacts.push(ArtifactEntry { path: a.clone(), sha256: String::new(), visibility: "missing".to_string(), bytes: 0 });
            manifest_full.push_str(&format!(
                "    - path: {}\n      mode: omitted_with_reason\n      reason: requested artifact missing\n", a
            ));
            artifact_excluded = true;
            excluded_paths.push((a.clone(), "requested artifact missing".to_string(), "artifact".to_string()));
            continue;
        }

        let bytes = std::fs::metadata(a).map(|m| m.len()).unwrap_or(0);
        if bytes > SIZE_LIMIT_BYTES {
            artifacts_block.push_str(&format!("  --- {} (visibility: oversize_omitted — over size limit, not shown) ---\n\n", a));
            artifacts.push(ArtifactEntry { path: a.clone(), sha256: String::new(), visibility: "oversize_omitted".to_string(), bytes });
            manifest_full.push_str(&format!(
                "    - path: {}\n      mode: omitted_with_reason\n      reason: over size limit\n", a
            ));
            artifact_excluded = true;
            excluded_paths.push((a.clone(), "requested artifact over size limit".to_string(), "artifact".to_string()));
            continue;
        }

        let sha = sha256_file(a)?;

        if opts.delta_mode {
            let base = opts.delta_base.as_deref().unwrap_or("HEAD");
            // fail-closed: untracked files can't be compared to a base commit; clear diagnostic.
            if !git_is_tracked(a, &opts.repo_root) {
                bail!(
                    "artifact is untracked; delta review cannot compare it to base: {}\n       Stage the file, commit it, or rerun with --mode full for explicit artifacts.",
                    a
                );
            }
            let changed = !git_diff_quiet(base, a, &opts.repo_root)?;
            let vis = if changed { "delta_diff" } else { "path_sha_only" };
            artifacts_block.push_str(&format!(
                "  --- {} (mode: {}, sha256: {}, bytes: {}) ---\n\n", a, vis, sha, bytes
            ));
            artifacts.push(ArtifactEntry { path: a.clone(), sha256: sha.clone(), visibility: vis.to_string(), bytes });
            manifest_full.push_str(&format!(
                "    - path: {}\n      mode: {}\n      bytes: {}\n      sha256: {}\n", a, vis, bytes, sha
            ));
            if changed { delta_diff_count += 1; }
            shown_count += 1;
        } else {
            let raw = std::fs::read_to_string(a)
                .with_context(|| format!("could not read artifact {}", a))?;
            let (redacted, rc) = redact_secrets(&raw);
            let vis = if rc > 0 {
                secret_flag = true;
                redaction_count += rc;
                excluded_paths.push((a.clone(), "secret value redacted in place".to_string(), "artifact".to_string()));
                manifest_full.push_str(&format!(
                    "    - path: {}\n      mode: full_file\n      bytes: {}\n      sha256: {}\n      note: secret value redacted in place\n",
                    a, bytes, sha
                ));
                "shown_redacted"
            } else {
                manifest_full.push_str(&format!(
                    "    - path: {}\n      mode: full_file\n      bytes: {}\n      sha256: {}\n", a, bytes, sha
                ));
                "shown"
            };
            let indented: String = redacted.lines().map(|l| format!("    {}\n", l)).collect();
            artifacts_block.push_str(&format!(
                "  --- {} (sha256: {}, visibility: {}) ---\n{}\n", a, sha, vis, indented
            ));
            artifacts.push(ArtifactEntry { path: a.clone(), sha256: sha, visibility: vis.to_string(), bytes });
            review_content_bytes += bytes;
            file_contributors.push((a.clone(), bytes));
            shown_count += 1;
        }
    }

    review_content_bytes += diff_bytes;
    if diff_bytes > 0 {
        file_contributors.push(("(diff)".to_string(), diff_bytes));
    }

    // Coverage state (most severe wins)
    let coverage_state = if (opts.delta_mode && delta_diff_count == 0 && redacted_diff.trim().is_empty())
        || (!opts.delta_mode && shown_count == 0 && redacted_diff.trim().is_empty())
    {
        CoverageState::EmptyPacket
    } else if artifact_excluded {
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

    // Budget check (warning only)
    let budget = std::env::var("CODEOS_PACKET_BUDGET_BYTES")
        .ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(50_000);
    if review_content_bytes > budget {
        // Enhanced warning: show overage, top contributors, and actionable suggestions
        let overage_multiple = (review_content_bytes as f64 / budget as f64).ceil() as u64;
        let budget_kb = budget / 1024;
        let packet_kb = review_content_bytes / 1024;

        eprintln!("warning: packet is {} KB ({}× over {} KB budget)", packet_kb, overage_multiple, budget_kb);

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
        eprintln!("    codeos-reviewer review {} {} --mode delta --base <last-review-commit> <artifacts>", opts.feature, opts.stage);
        eprintln!("  optional:");
        eprintln!("    use --sha-only <path> only for large unchanged context files that are not the primary artifact under review; this reduces review evidence");
    }

    // Stage-specific checks
    let checks = stage_checks(&opts.stage);
    let expected = stage_expected(&opts.stage);
    let task_prompt_path = format!("{}/prompts/codeos-reviewer-task.md", opts.toolkit_root);
    let task_prompt = std::fs::read_to_string(&task_prompt_path)
        .with_context(|| format!("reviewer task template not found: {}", task_prompt_path))?;

    // Build the packet text
    let mut content = String::new();
    content.push_str(&task_prompt);
    content.push('\n');

    let budget_status = if review_content_bytes > budget {
        format!("WARNING — {} bytes exceeds CODEOS_PACKET_BUDGET_BYTES={}", review_content_bytes, budget)
    } else {
        "OK".to_string()
    };

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let task_prompt_bytes = std::fs::metadata(&task_prompt_path).map(|m| m.len()).unwrap_or(0);
    let estimated_tokens = review_content_bytes / 4;

    content.push_str(&format!(
        "\nPACKET MANIFEST\n  generated: {now}\n  task_prompt: {task_prompt_path} ({task_prompt_bytes} bytes)\n  review_content_bytes: {review_content_bytes}\n  estimated_review_tokens: ~{estimated_tokens}\n  budget_status: {budget_status}\n  packet_mode: {mode}\n  delta_base: {delta_base}\n  items:\n{manifest_sha_only}{manifest_full}    - path: (diff)\n      mode: {diff_mode}\n      bytes: {diff_bytes}\n",
        mode = if opts.delta_mode { "delta" } else { "full" },
        delta_base = opts.delta_base.as_deref().unwrap_or("none"),
        diff_mode = if opts.delta_mode { "delta_diff" } else { "full_file" },
    ));

    let dirty_note = if workspace_dirty { " (+ uncommitted workspace changes)" } else { "" };
    content.push_str(&format!(
        "\nREVIEW CONTEXT\n  Feature:                {feature}\n  Stage:                  {stage}\n  Branch:                 {branch}\n  Base commit:            {base_sha}\n  Review commit:          {review_sha}{dirty_note}\n  Current approved stage: {approved_stage}\n  Evidence coverage:      {coverage}\n  Workspace dirty:        {dirty_text}\n",
        feature = opts.feature,
        stage = opts.stage,
        branch = branch,
        base_sha = if base_sha.is_empty() { "(no base pin)".to_string() } else { base_sha.clone() },
        review_sha = review_sha,
        approved_stage = approved_stage,
        coverage = coverage_state.as_str(),
        dirty_text = if workspace_dirty { "yes (uncommitted changes at review time)" } else { "no" },
    ));

    content.push_str("\nDBA RULES RELEVANT TO THIS STAGE\n");
    content.push_str("  - Human approval is required for every stage transition; you are advisory only.\n");
    content.push_str("  - Memory is not truth — assess only what is provided, pinned to the review commit.\n");
    content.push_str("  - Implementation must trace to approved artifacts; no behavior beyond intent+contract+schema.\n");
    content.push_str("  - No events outside the approved event schema; no hidden behavior.\n");
    if matches!(coverage_state, CoverageState::PartialCoverage | CoverageState::SecretRedaction | CoverageState::CriticalOmission) {
        content.push_str("  - COVERAGE IS PARTIAL: some content was excluded/redacted (see below). You are\n");
        content.push_str("    seeing an incomplete evidence set — do not issue NO OBJECTION on this basis.\n");
    }

    content.push_str(&format!("\nSTAGE-SPECIFIC CHECKS\n{}\n", checks));
    content.push_str(&format!("\nEXPECTED STAGE OUTPUT\n  {}\n", expected));
    content.push_str("\nARTIFACTS TO REVIEW\n");
    content.push_str(&artifacts_block);

    if opts.delta_mode {
        let base = opts.delta_base.as_deref().unwrap_or("HEAD");
        content.push_str(&format!(
            "DELTA DIFF ({}->working tree, artifact paths only, secret/size filtered)\n", base
        ));
    } else {
        content.push_str("DIFF TO REVIEW (base->review, secret/size filtered)\n");
    }

    let excluded_summary: Vec<String> = excluded_paths.iter().map(|(p, _, _)| p.clone()).collect();
    if !excluded_summary.is_empty() {
        content.push_str(&format!("  [excluded/redacted: {}] manual security review required\n", excluded_summary.join(", ")));
    }
    content.push_str(&redacted_diff);

    // Full Context Diff (AC-1: auto-appended when delta_mode AND delta_base are both active).
    if opts.delta_mode {
        if let Some(ref base) = opts.delta_base {
            content.push_str(&format!(
                "\nFull Context Diff (informational — all changed files since {}):\n",
                base
            ));

            match git_diff_range(base, &[], &opts.repo_root) {
                Err(e) => {
                    // Fail-closed: mark the section as unavailable rather than silently showing empty.
                    content.push_str(&format!(
                        "[ERROR: git diff failed — full context diff unavailable: {}]\n", e
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
                        let cutpoint = head.rfind('\n').map(|n| n + 1).unwrap_or(remaining as usize);
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
    }

    let final_base_sha = if base_sha.is_empty() { "(no base pin)".to_string() } else { base_sha };

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
        review_content_bytes,
        estimated_review_tokens: estimated_tokens,
        budget_bytes: budget,
        over_budget: review_content_bytes > budget,
        diff_bytes,
        budget_contributors: file_contributors,
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
    for p in paths { args.push(p); }
    let out = Command::new("git")
        .args(&args)
        .current_dir(repo_root)
        .output()
        .context("git diff")?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn git_diff_names(base: &str, paths: &[String], repo_root: &str) -> Result<Vec<String>> {
    let mut args = vec!["diff", "--name-only", base, "--"];
    for p in paths { args.push(p); }
    args.extend_from_slice(&[":(exclude)reviews", ":(exclude).codeos-state"]);
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
        .args(["diff", "--name-only", "HEAD", "--", ".",
               ":(exclude)reviews", ":(exclude).codeos-state"])
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
    for p in paths { args.push(p); }
    let out = Command::new("git").args(&args).current_dir(repo_root).output().context("git diff HEAD paths")?;
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

fn git_is_dirty(repo_root: &str) -> bool {
    Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=all", "--",
               ".", ":(exclude)reviews", ":(exclude).codeos-state"])
        .current_dir(repo_root)
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

fn load_base_sha(feature: &str, stage: &str, repo_root: &str, delta_base: Option<&str>) -> Result<String> {
    if let Some(b) = delta_base { return Ok(b.to_string()); }
    let ss = format!("{}/.codeos-state/stage-start/{}/stage-{}.json", repo_root, feature, stage);
    if !Path::new(&ss).exists() { return Ok(String::new()); }
    let content = std::fs::read_to_string(&ss)
        .with_context(|| format!("could not read stage-start file {}", ss))?;
    for line in content.lines() {
        if line.contains("\"base_commit\"") {
            if let Some(rest) = line.splitn(2, "\"base_commit\"").nth(1) {
                if let Some(after_colon) = rest.splitn(2, ':').nth(1) {
                    let trimmed = after_colon.trim();
                    if trimmed.starts_with('"') {
                        let val: String = trimmed.chars().skip(1).take_while(|&c| c != '"').collect();
                        if !val.is_empty() { return Ok(val); }
                    }
                }
            }
        }
    }
    anyhow::bail!("{} exists but has no valid base_commit (malformed provenance) — aborting.", ss);
}

fn is_path_excluded(path: &str) -> bool {
    let name = Path::new(path).file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);
    for pat in PATH_EXCLUDES {
        if glob_match(pat, path) || glob_match(pat, name) {
            return true;
        }
    }
    false
}

fn glob_match(pattern: &str, s: &str) -> bool {
    if let Some(suffix) = pattern.strip_prefix('*') {
        s.ends_with(suffix)
    } else if let Some(prefix) = pattern.strip_suffix('*') {
        s.starts_with(prefix)
    } else {
        s == pattern
    }
}

fn is_oversize(path: &str) -> bool {
    std::fs::metadata(path)
        .map(|m| m.len() > SIZE_LIMIT_BYTES)
        .unwrap_or(false)
}

pub fn sha256_file(path: &str) -> Result<String> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("could not read file for SHA: {}", path))?;
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
        "discovery" => "Solution Discovery — candidate feature topology, shared vocabulary, event/config hypotheses, architectural risks, explicit non-decisions; every item labeled CANDIDATE/HYPOTHESIZED; non-authoritative banner present; only reviewed when carried into a Feature Brief or Stage 1 Intent.",
        "brief" => "Feature Brief — problem, upgrade, bounded scope, proposed artifact(s), value/risk/guardrail; a candidate for Stage 1, not yet approved; no implementation detail.",
        "onboarding" => "Onboarding (Session Type D) — HYPOTHESIZED_INTENT drafts + codebase digest derived from evidence only; explicitly labeled draft, not APPROVED; must pass Stage 1 review before advancing.",
        "1" => "Intent — actor+outcome statements, stable guarantees, explicit scope boundary; NO implementation detail.",
        "2" => "Behavioral contract — observable Given/When/Then scenarios, named failure modes, invariants; independently testable; no white-box claims.",
        "3" => "Event schema — named events with payloads, event flow, coverage map of contract scenarios to events; no speculative telemetry.",
        "4" => "Implementation — code satisfying every contract clause; emits only schema events; contract-satisfaction + event-emission tables; nothing untraceable.",
        "5" => "Tests — one behavioral test per contract scenario incl. failures; replay tests for schema conformance + chain integrity; coverage table.",
        "6" => "Runtime evidence — events in runtime_events.jsonl; correlation chains intact; bounded/sanitized; unexpected/missing events reported.",
        "7" => "Reconciliation — Intent->Contract->Schema->Impl->Tests->Runtime with ALIGNED/GAP/MISMATCH/MISSING per item, supported by evidence.",
        "8" => "Replay — schema conformance + correlation chain integrity + determinism check; nondeterminism explained; missing fixtures reported.",
        "9" => "Refinement — smallest effective change per observed trigger; no redesign disguised as refinement; affected artifacts named.",
        "10" => "Architectural Refinement — Scope->Impact->Implement->Verify->Reconcile for structural changes with no behavioral contract or event schema; smallest effective change, no full rewrite.",
        _ => "(no expected-output template for stage)",
    }
}

fn stage_checks(stage: &str) -> String {
    match stage {
        "discovery" => "  - every item labeled CANDIDATE/HYPOTHESIZED, not approved; non-authoritative banner present; no intent/contract/schema language; out-of-scope findings recorded as backlog candidates, not acted on.".to_string(),
        "brief" => "  - problem clearly stated; scope explicitly bounded; no implementation detail; ready to become a Stage 1 Intent; value/risk/guardrail present.".to_string(),
        "onboarding" => "  - hypothesized intents clearly labeled as drafts, not approved; digest derived from observed evidence only, no invented behavior; clear path to Stage 1 promotion named.".to_string(),
        "1" => "  - actor/outcome clarity; no implementation detail; scope boundary explicit; stable guarantees clear; ambiguity flagged.".to_string(),
        "2" => "  - every intent outcome has observable contract coverage; failure paths named; invariants testable; no white-box claims.".to_string(),
        "3" => "  - every relevant contract scenario has event coverage; event names stable; required fields clear; no speculative telemetry.".to_string(),
        "4" => "  - code traces to approved contract/schema only; no unapproved events; no hidden behavior; no unrelated files; report complete.".to_string(),
        "5" => "  - behavior tested not private internals; failure paths tested; event/telemetry tests present; replay tests where applicable.".to_string(),
        "6" => "  - runtime evidence captured; event log bounded/sanitized; correlation chains visible; unexpected/missing events reported.".to_string(),
        "7" => "  - ALIGNED/GAP/MISMATCH/MISSING judgments supported; no weak evidence hidden behind fluent summary; gaps routed to right action.".to_string(),
        "8" => "  - replay actually checks event sequence + schema conformance; nondeterminism explained; missing fixtures reported.".to_string(),
        "9" => "  - trigger valid; proposed fix minimal; no redesign disguised as refinement; affected artifacts identified.".to_string(),
        "10" => "  - genuinely structural (no contract/schema change); impact assessed before implementing; verification is real, not just described; no full rewrite disguised as refinement.".to_string(),
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
        assert_eq!(CoverageState::CriticalOmission.as_str(), "CRITICAL_OMISSION");
    }

    #[test]
    fn path_excluded_env_file() {
        assert!(is_path_excluded("secrets/.env.local"));
        assert!(is_path_excluded("config.key"));
        assert!(!is_path_excluded("src/main.rs"));
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
        for stage in ["discovery", "brief", "onboarding", "10"] {
            let text = stage_expected(stage);
            assert_ne!(
                text, "(no expected-output template for stage)",
                "stage '{}' must have real expected-output text", stage
            );
        }
    }

    #[test]
    fn stage_checks_new_downstream_identifiers_are_real_not_placeholder() {
        for stage in ["discovery", "brief", "onboarding", "10"] {
            let text = stage_checks(stage);
            assert!(
                !text.contains("no stage-specific checklist"),
                "stage '{}' must have real checklist text", stage
            );
        }
    }

    #[test]
    fn stage_expected_numeric_1_to_9_unchanged_by_extension() {
        for stage in ["1", "2", "3", "4", "5", "6", "7", "8", "9"] {
            let text = stage_expected(stage);
            assert_ne!(
                text, "(no expected-output template for stage)",
                "numeric stage '{}' must still have its existing expected-output text", stage
            );
        }
    }

    #[test]
    fn stage_checks_unrecognized_identifier_still_falls_back_to_placeholder() {
        let text = stage_checks("nonexistent-stage");
        assert!(text.contains("no stage-specific checklist for stage nonexistent-stage"));
    }
}
