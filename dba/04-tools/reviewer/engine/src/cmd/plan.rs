use crate::cmd::review::{self, EvidenceArgs};
use crate::config::Config;
use crate::packet::{CoverageState, ReviewPacket};
use anyhow::Result;

/// Preview the same validated evidence selection and packet construction used by `review`, without
/// invoking Codex or writing reviewer records.
pub fn run(args: EvidenceArgs, cfg: &Config) -> Result<i32> {
    let prepared = match review::prepare(args, cfg) {
        Ok(prepared) => prepared,
        Err(failure) => {
            eprintln!("error: {}", failure.message);
            return Ok(failure.code);
        }
    };

    print_plan_summary(&prepared.args, &prepared.packet);
    if matches!(prepared.packet.coverage_state, CoverageState::EmptyPacket) {
        return Ok(crate::EXIT_PACKET);
    }
    Ok(crate::EXIT_SUCCESS)
}

fn print_plan_summary(args: &EvidenceArgs, packet: &ReviewPacket) {
    println!("review plan: {} {}", args.feature, args.stage);
    println!(
        "  mode: {}",
        if args.base.is_some() { "delta" } else { "full" }
    );
    if let Some(base) = &args.base {
        println!("  base: {base}");
    }
    println!("  coverage: {}", packet.coverage_state.as_str());
    println!("  artifacts:");
    for artifact in &packet.artifacts {
        println!(
            "    - {} ({}, {} bytes)",
            artifact.path, artifact.visibility, artifact.bytes
        );
    }

    let percent = if packet.budget_bytes > 0 {
        packet.review_content_bytes as f64 / packet.budget_bytes as f64 * 100.0
    } else {
        0.0
    };
    println!(
        "  review_content_bytes: {} / budget {} ({percent:.0}%)",
        packet.review_content_bytes, packet.budget_bytes
    );
    println!(
        "  estimated_review_tokens: ~{}",
        packet.estimated_review_tokens
    );

    if packet.over_budget {
        print_budget_warning(args, packet);
    }
    if matches!(packet.coverage_state, CoverageState::EmptyPacket) {
        println!();
        println!("  EMPTY_PACKET: no reviewable content found; review would fail before Codex invocation.");
    }
}

fn print_budget_warning(args: &EvidenceArgs, packet: &ReviewPacket) {
    let multiple = (packet.review_content_bytes as f64 / packet.budget_bytes as f64).ceil() as u64;
    println!();
    println!(
        "  WARNING: packet is {} KB ({}x over {} KB budget)",
        packet.review_content_bytes / 1024,
        multiple,
        packet.budget_bytes / 1024
    );

    let mut contributors = packet.budget_contributors.clone();
    contributors.sort_by(|a, b| b.1.cmp(&a.1));
    println!("  largest inputs:");
    for (path, bytes) in contributors.iter().take(3) {
        let percent = if packet.review_content_bytes > 0 {
            *bytes as f64 / packet.review_content_bytes as f64 * 100.0
        } else {
            0.0
        };
        println!("    {path}: {} KB ({percent:.0}%)", bytes / 1024);
    }
    println!("  suggest for later rounds:");
    println!(
        "    codeos-reviewer review {} {} --base <last-review-commit> <artifacts>",
        args.feature, args.stage
    );
    println!("  use --sha-only only for unchanged context whose contents are not primary evidence");
}
