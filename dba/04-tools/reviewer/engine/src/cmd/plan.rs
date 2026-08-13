use crate::cmd::review::ReviewArgs;
use crate::config::Config;
use crate::packet::{self, CoverageState, PacketBuildOptions, ReviewPacket};
use crate::precheck;
use anyhow::Result;
use std::path::Path;

/// Preview what `review` would send, without invoking Codex or writing anything.
///
/// Reuses `review`'s own validation (delta-mode `--base` checks, missing-artifact guards,
/// prechecks) and `packet::build()` (the exact function `review`/`--print-packet` call) so the
/// preview cannot drift from what a real review would see. Never resolves a provider, never
/// invokes one, never writes to `reviews/` or any other tracked file.
pub fn run(args: ReviewArgs, cfg: &Config) -> Result<i32> {
    if args.artifacts.is_empty() {
        eprintln!("plan: provide at least one artifact path");
        return Ok(crate::EXIT_USAGE);
    }

    // Validate delta mode (mirrors review::run).
    if args.delta_mode {
        match &args.delta_base {
            None => {
                eprintln!("plan: --mode delta requires --base <sha>");
                return Ok(crate::EXIT_USAGE);
            }
            Some(base) => {
                if !base.chars().all(|c| c.is_ascii_hexdigit()) || base.len() < 7 {
                    eprintln!("plan: --base value is not a valid hex SHA: '{}'", base);
                    return Ok(crate::EXIT_USAGE);
                }
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
                        eprintln!("plan: --base '{}' does not resolve to a valid commit", base);
                        return Ok(crate::EXIT_USAGE);
                    }
                    Ok(_) => {}
                }
            }
        }
    }

    // Conflict check: same path in both positional and --sha-only.
    for so in &args.sha_only {
        for a in &args.artifacts {
            if a == so {
                eprintln!(
                    "plan: path '{}' passed both as positional artifact and --sha-only; pass as one or the other",
                    so
                );
                return Ok(crate::EXIT_USAGE);
            }
        }
    }

    // Fail-closed on missing positional artifacts (mirrors review::run).
    for a in &args.artifacts {
        if !Path::new(a).exists() {
            eprintln!("error: artifact not found: {}", a);
            return Ok(crate::EXIT_PACKET);
        }
    }

    for so in &args.sha_only {
        if !Path::new(so).exists() {
            eprintln!("error: --sha-only path not found: {}", so);
            return Ok(crate::EXIT_USAGE);
        }
    }

    // Prechecks — reported, never written anywhere (mirrors review::run's precheck calls).
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

    // Build packet — the exact same function review/--print-packet use (AC-2). No writes,
    // no provider resolution, no provider invocation happen anywhere in this command.
    let build_opts = PacketBuildOptions {
        feature: args.feature.clone(),
        stage: args.stage.clone(),
        artifacts: args.artifacts.clone(),
        sha_only_paths: args.sha_only.clone(),
        delta_mode: args.delta_mode,
        delta_base: args.delta_base.clone(),
        fresh_session: false,
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

    print_plan_summary(&args, &review_packet);

    if matches!(review_packet.coverage_state, CoverageState::EmptyPacket) {
        return Ok(crate::EXIT_PACKET);
    }
    Ok(crate::EXIT_SUCCESS)
}

fn print_plan_summary(args: &ReviewArgs, p: &ReviewPacket) {
    println!("review plan: {} {}", args.feature, args.stage);
    println!("  mode: {}", if args.delta_mode { "delta" } else { "full" });
    if let Some(base) = &args.delta_base {
        println!("  base: {}", base);
    }
    println!("  coverage: {}", p.coverage_state.as_str());
    println!("  artifacts:");
    for a in &p.artifacts {
        println!("    - {} ({}, {} bytes)", a.path, a.visibility, a.bytes);
    }

    let pct = if p.budget_bytes > 0 {
        (p.review_content_bytes as f64 / p.budget_bytes as f64) * 100.0
    } else {
        0.0
    };
    println!(
        "  review_content_bytes: {} / budget {} ({:.0}%)",
        p.review_content_bytes, p.budget_bytes, pct
    );
    println!("  estimated_review_tokens: ~{}", p.estimated_review_tokens);

    if p.over_budget {
        print_budget_warning(args, p);
    }

    if matches!(p.coverage_state, CoverageState::EmptyPacket) {
        println!();
        println!(
            "  EMPTY_PACKET: no reviewable content found. review would fail closed (exit before any Codex call)."
        );
    }
}

fn print_budget_warning(args: &ReviewArgs, p: &ReviewPacket) {
    let overage_multiple = (p.review_content_bytes as f64 / p.budget_bytes as f64).ceil() as u64;
    let packet_kb = p.review_content_bytes / 1024;
    let budget_kb = p.budget_bytes / 1024;

    println!();
    println!(
        "  WARNING: packet is {} KB ({}x over {} KB budget)",
        packet_kb, overage_multiple, budget_kb
    );

    // The exact same (path, bytes) pairs build()'s own stderr warning ranks — sha-only and
    // delta-mode entries are deliberately excluded because they are never counted toward
    // review_content_bytes, even though ArtifactEntry.bytes reports their on-disk size.
    let mut contributors: Vec<(&str, u64)> =
        p.budget_contributors.iter().map(|(path, bytes)| (path.as_str(), *bytes)).collect();
    contributors.sort_by(|a, b| b.1.cmp(&a.1));

    println!("  largest inputs:");
    for (path, bytes) in contributors.iter().take(3) {
        let kb = bytes / 1024;
        let item_pct = if p.review_content_bytes > 0 {
            (*bytes as f64 / p.review_content_bytes as f64 * 100.0) as u64
        } else {
            0
        };
        println!("    {}: {} KB ({}%)", path, kb, item_pct);
    }

    println!("  suggest for R2+:");
    println!(
        "    codeos-reviewer review {} {} --mode delta --base <last-review-commit> <artifacts>",
        args.feature, args.stage
    );
    println!("  optional:");
    println!(
        "    use --sha-only <path> only for large unchanged context files that are not the primary artifact under review; this reduces review evidence"
    );
}
