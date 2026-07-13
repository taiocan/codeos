use crate::assessment;
use crate::config::Config;
use crate::log as review_log;
use crate::packet::{self, PacketBuildOptions};
use crate::precheck;
use crate::provider::{self, ProviderConfig};
use anyhow::{Result, bail};
use std::path::Path;

pub struct ReviewArgs {
    pub feature: String,
    pub stage: String,
    pub artifacts: Vec<String>,
    pub sha_only: Vec<String>,
    pub guard_clean: Vec<String>,
    pub fresh: bool,
    pub scratch: bool,
    pub print_only: bool,
    pub skip_prechecks: bool,
    pub delta_mode: bool,
    pub delta_base: Option<String>,
}

pub fn run(args: ReviewArgs, cfg: &Config, provider_name: &str) -> Result<i32> {
    if args.artifacts.is_empty() {
        eprintln!("review: provide at least one artifact path");
        return Ok(crate::EXIT_USAGE);
    }

    // Validate delta mode
    if args.delta_mode {
        match &args.delta_base {
            None => {
                eprintln!("review: --mode delta requires --base <sha>");
                return Ok(crate::EXIT_USAGE);
            }
            Some(base) => {
                if !base.chars().all(|c| c.is_ascii_hexdigit()) || base.len() < 7 {
                    eprintln!("review: --base value is not a valid hex SHA: '{}'", base);
                    return Ok(crate::EXIT_USAGE);
                }
                // Verify commit exists
                let git_status = std::process::Command::new("git")
                    .args(["rev-parse", "--verify", &format!("{}^{{commit}}", base)])
                    .current_dir(&cfg.repo_root)
                    .status();
                match git_status {
                    Err(e) => {
                        eprintln!("error: could not run git to verify --base commit: {}", e);
                        return Ok(crate::EXIT_CONFIG);
                    }
                    Ok(s) if !s.success() => {
                        eprintln!("review: --base '{}' does not resolve to a valid commit", base);
                        return Ok(crate::EXIT_USAGE);
                    }
                    Ok(_) => {}
                }
            }
        }
    }

    // Conflict check: same path in both positional and --sha-only
    for so in &args.sha_only {
        for a in &args.artifacts {
            if a == so {
                eprintln!("review: path '{}' passed both as positional artifact and --sha-only; pass as one or the other", so);
                return Ok(crate::EXIT_USAGE);
            }
        }
    }

    let review_log_path = if args.scratch {
        let scratch = cfg.codex_dir.join("_scratch");
        if let Err(e) = std::fs::create_dir_all(&scratch) {
            eprintln!("error: could not create scratch dir {}: {}", scratch.display(), e);
            return Ok(crate::EXIT_WRITE);
        }
        scratch.join("review-log.md")
    } else {
        cfg.review_log.clone()
    };

    // Fail-closed on missing positional artifacts (AC-3: exit 4 for artifact-not-found)
    for a in &args.artifacts {
        if !Path::new(a).exists() {
            eprintln!("error: artifact not found: {}", a);
            return Ok(crate::EXIT_PACKET);
        }
    }

    // Missing --sha-only path is a bad CLI argument (exit 1, not packet error 4)
    for so in &args.sha_only {
        if !Path::new(so).exists() {
            eprintln!("error: --sha-only path not found: {}", so);
            return Ok(crate::EXIT_USAGE);
        }
    }

    // Prechecks
    if !args.skip_prechecks {
        for a in &args.artifacts {
            let content = match std::fs::read_to_string(a) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error: could not read artifact {}: {}", a, e);
                    return Ok(crate::EXIT_PACKET);
                }
            };
            if let Err(e) = precheck::check_no_unfilled_placeholders(Path::new(a), &content) {
                eprintln!("{}", e);
                return Ok(crate::EXIT_PACKET);
            }
            if let Err(e) = precheck::check_no_forbidden_fields(Path::new(a), &content) {
                eprintln!("{}", e);
                return Ok(crate::EXIT_PACKET);
            }
            precheck::check_draft_markers(Path::new(a), &content);
        }
        for gc in &args.guard_clean {
            if let Err(e) = precheck::check_guard_clean(Path::new(gc)) {
                eprintln!("{}", e);
                return Ok(crate::EXIT_PACKET);
            }
        }
    }

    // Build packet
    let build_opts = PacketBuildOptions {
        feature: args.feature.clone(),
        stage: args.stage.clone(),
        artifacts: args.artifacts.clone(),
        sha_only_paths: args.sha_only.clone(),
        delta_mode: args.delta_mode,
        delta_base: args.delta_base.clone(),
        fresh_session: args.fresh,
        repo_root: cfg.repo_root.to_string_lossy().into_owned(),
        toolkit_root: cfg.toolkit_root.to_string_lossy().into_owned(),
    };

    let review_packet = match packet::build(&build_opts) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error building packet: {}", e);
            return Ok(crate::EXIT_PACKET);
        }
    };

    if args.print_only {
        print!("{}", review_packet.content());
        if matches!(review_packet.coverage_state, crate::packet::CoverageState::EmptyPacket) {
            return Ok(crate::EXIT_PACKET);
        }
        return Ok(crate::EXIT_SUCCESS);
    }

    // Fail-closed on empty packet
    if matches!(review_packet.coverage_state, crate::packet::CoverageState::EmptyPacket) {
        eprintln!("error: review packet is empty (EMPTY_PACKET) — no reviewable content found.");
        if args.delta_mode {
            eprintln!("       Delta mode: ensure tracked artifacts have working-tree changes since --base,");
            eprintln!("       or use --mode full with explicit artifact paths.");
        }
        eprintln!("       Inspect the packet with --print-packet before rerunning.");
        return Ok(crate::EXIT_PACKET);
    }

    // Compute review_id before any Codex invocation (AC-10: fail closed on an unreadable log,
    // never guess a round). A missing log or zero prior matches both correctly yield round 1.
    let round = match review_log::compute_review_round(&review_log_path, &args.feature, &args.stage) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: could not determine review round: {}", e);
            return Ok(crate::EXIT_WRITE);
        }
    };
    let review_id = review_log::format_review_id(&args.feature, &args.stage, round);

    // Resolve provider and invoke
    let prov = match provider::resolve_provider(provider_name) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {}", e);
            return Ok(crate::EXIT_CONFIG);
        }
    };

    let prov_cfg = ProviderConfig {
        provider_name: provider_name.to_string(),
        reasoning_effort: cfg.reasoning_effort.clone(),
        repo_root: cfg.repo_root.to_string_lossy().into_owned(),
        sessions_dir: cfg.sessions_dir.to_string_lossy().into_owned(),
    };

    // Read-only invariant: snapshot working tree before invoke (AC-1); compare after.
    // Non-zero git exit (e.g. not a git repo) yields None — check silently skipped (AC-4).
    let pre_invoke_status = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&cfg.repo_root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| o.stdout);

    let invoke_result = prov.invoke(&review_packet, &prov_cfg);

    // Post-invoke check runs regardless of invoke success or failure (AC-2/AC-3/AC-4).
    if let Some(pre) = pre_invoke_status {
        let post_status = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&cfg.repo_root)
            .output()
            .ok()
            .filter(|o| o.status.success());
        if let Some(post) = post_status {
            if post.stdout != pre {
                eprintln!("WARNING: working tree changed during review — reviewer should be read-only");
            }
        }
    }

    let raw = match invoke_result {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: provider invocation failed: {}", e);
            return Ok(crate::EXIT_PROVIDER);
        }
    };

    // Parse review output
    let parsed = assessment::parse_review_output(&raw.text, &review_packet.coverage_state);

    // Save packet
    let outdir = if args.scratch { cfg.codex_dir.join("_scratch") } else { cfg.codex_dir.clone() };
    let packets_dir = outdir.join("packets");
    if let Err(e) = std::fs::create_dir_all(&packets_dir) {
        eprintln!("error: could not create packets dir {}: {}", packets_dir.display(), e);
        return Ok(crate::EXIT_WRITE);
    }

    let ts_clean = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let short_sha = &review_packet.review_sha[..7.min(review_packet.review_sha.len())];
    let packet_filename = format!("{}-{}-stage-{}-{}.packet.txt", ts_clean, args.feature, args.stage, short_sha);
    let packet_saved = packets_dir.join(&packet_filename);
    if let Err(e) = std::fs::write(&packet_saved, review_packet.content()) {
        eprintln!("error: could not write packet file {}: {}", packet_saved.display(), e);
        return Ok(crate::EXIT_WRITE);
    }
    let packet_saved_str = match packet_saved.to_str() {
        Some(s) => s,
        None => {
            eprintln!("error: packet path contains non-UTF-8 characters: {}", packet_saved.display());
            return Ok(crate::EXIT_WRITE);
        }
    };
    let packet_hash = match packet::sha256_file(packet_saved_str) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("error: could not hash packet file: {}", e);
            return Ok(crate::EXIT_WRITE);
        }
    };

    // Pre-compute expected assessment path for diagnostic use on write failure
    let assessment_filename = format!("{}-{}-stage-{}-{}.md", ts_clean, args.feature, args.stage, short_sha);
    let expected_assessment_path = outdir.join(&assessment_filename);

    // Schema validation (fail-closed)
    if let Err(e) = assessment::validate_schema(&review_packet, &parsed, &packet_hash) {
        eprintln!("error: {}", e);
        return Ok(crate::EXIT_PACKET);
    }

    // Write assessment
    let (assessment_file, assessment_hash) = match assessment::write_assessment(
        &review_id, &review_packet, &raw, &parsed, &outdir, &packet_saved, &packet_hash
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: assessment write failed: {}", e);
            eprintln!("  packet was written to: {}", packet_saved.display());
            eprintln!("  assessment target: {} (not written)", expected_assessment_path.display());
            return Ok(crate::EXIT_WRITE);
        }
    };

    // Append to log
    if let Err(e) = review_log::append_review(
        &review_log_path, &review_id, &review_packet, &raw, &parsed,
        &assessment_file, &assessment_hash, &packet_saved, &packet_hash,
    ) {
        eprintln!("error: log append failed: {}", e);
        eprintln!("  assessment was written to: {}", assessment_file.display());
        eprintln!("  packet was written to: {}", packet_saved.display());
        eprintln!("  log target: {} (not updated)", review_log_path.display());
        return Ok(crate::EXIT_WRITE);
    }

    // Print summary (matches Bash script stdout format)
    println!("review logged: {}", review_log_path.display());
    println!("  review_id: {}", review_id);
    println!("  codex concern: {}   effective concern: {}   evidence: {}",
        parsed.codex_concern, parsed.effective_concern, parsed.evidence);
    println!("  effort: {}   elapsed: {}ms   reconnects: {}",
        raw.effort, raw.elapsed_ms, raw.reconnect_count);
    println!("  coverage: {} (redactions: {})",
        review_packet.coverage_state.as_str(), review_packet.redaction_count);
    println!("  assessment: {}", assessment_file.display());
    println!("  packet: {}", packet_saved.display());

    Ok(crate::EXIT_SUCCESS)
}

/// Parse the trailing args after feature and stage from the review subcommand.
pub fn parse_rest(rest: &[String]) -> Result<(Vec<String>, Vec<String>, Vec<String>, bool, bool, bool, bool, bool, Option<String>)> {
    let mut artifacts = Vec::new();
    let mut sha_only = Vec::new();
    let mut guard_clean = Vec::new();
    let mut fresh = false;
    let mut scratch = false;
    let mut print_only = false;
    let mut skip_prechecks = false;
    let mut delta_mode = false;
    let mut delta_base: Option<String> = None;

    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--fresh" => { fresh = true; i += 1; }
            "--scratch" => { scratch = true; i += 1; }
            "--print-packet" | "--dry-run" => { print_only = true; i += 1; }
            "--skip-prechecks" => { skip_prechecks = true; i += 1; }
            "--sha-only" => {
                if i + 1 >= rest.len() { bail!("review: --sha-only requires a PATH argument"); }
                sha_only.push(rest[i+1].clone());
                i += 2;
            }
            "--guard-clean" => {
                if i + 1 >= rest.len() { bail!("review: --guard-clean requires a PATH argument"); }
                guard_clean.push(rest[i+1].clone());
                i += 2;
            }
            "--mode" => {
                if i + 1 >= rest.len() { bail!("review: --mode requires an argument (full or delta)"); }
                match rest[i+1].as_str() {
                    "full" => delta_mode = false,
                    "delta" => delta_mode = true,
                    other => bail!("review: --mode must be 'full' or 'delta', got '{}'", other),
                }
                i += 2;
            }
            "--base" => {
                if i + 1 >= rest.len() { bail!("review: --base requires a SHA argument"); }
                delta_base = Some(rest[i+1].clone());
                i += 2;
            }
            arg if arg.starts_with('-') => {
                bail!("review: unknown argument '{}'", arg);
            }
            path => {
                artifacts.push(path.to_string());
                i += 1;
            }
        }
    }

    Ok((artifacts, sha_only, guard_clean, fresh, scratch, print_only, skip_prechecks, delta_mode, delta_base))
}
