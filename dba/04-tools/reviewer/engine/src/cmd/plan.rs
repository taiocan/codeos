use crate::cmd::review::{self, EvidenceArgs};
use crate::config::Config;
use crate::packet::{CoverageState, ReviewPacket};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Preview the same validated evidence selection and packet construction used by `review`, without
/// invoking Codex or writing reviewer records.
///
/// `emit_packet` writes the packet bytes `review` would send. There is deliberately no second
/// construction path: both commands share `review::prepare`, so the exported file is the canonical
/// packet and stays byte-identical to what a Codex-backed review would receive.
pub fn run(args: EvidenceArgs, emit_packet: Option<PathBuf>, cfg: &Config) -> Result<i32> {
    let prepared = match review::prepare(args, cfg) {
        Ok(prepared) => prepared,
        Err(failure) => {
            eprintln!("error: {}", failure.message);
            return Ok(failure.code);
        }
    };

    print_plan_summary(&prepared.args, &prepared.packet, cfg.packet_budget_mode);
    prepared.readiness.print_plan();
    if !prepared.readiness.passes() {
        return Ok(crate::EXIT_PACKET);
    }
    if matches!(prepared.packet.coverage_state, CoverageState::EmptyPacket) {
        // An empty packet fails before invocation, so there is nothing legitimate to export either.
        return Ok(crate::EXIT_PACKET);
    }
    if let Some(path) = emit_packet {
        warn_if_export_pollutes_evidence(&path, &cfg.repo_root, &cfg.state_dir);
        if let Err(error) = std::fs::write(&path, prepared.packet.content()) {
            eprintln!(
                "error: could not write packet file {}: {error}",
                path.display()
            );
            return Ok(crate::EXIT_WRITE);
        }
        // The sidecar carries everything about this packet except its bytes, so `review
        // --assessment` can record the exported packet without rebuilding one. Generating the
        // packet twice is what let the recorded evidence drift from the reviewed evidence.
        let sidecar = sidecar_path(&path);
        match serde_json::to_string_pretty(&prepared.packet) {
            Ok(json) => {
                if let Err(error) = std::fs::write(&sidecar, format!("{json}\n")) {
                    eprintln!(
                        "error: could not write packet sidecar {}: {error}",
                        sidecar.display()
                    );
                    return Ok(crate::EXIT_WRITE);
                }
            }
            Err(error) => {
                eprintln!("error: could not serialize packet sidecar: {error}");
                return Ok(crate::EXIT_WRITE);
            }
        }
        println!("  packet written: {}", path.display());
        println!("  sidecar written: {}", sidecar.display());
    }
    Ok(crate::EXIT_SUCCESS)
}

/// The sidecar sits beside the packet as `<packet>.meta.json`. Derived rather than passed so the
/// two files cannot be separated by an operator naming them independently.
pub fn sidecar_path(packet_path: &Path) -> PathBuf {
    let mut name = packet_path.as_os_str().to_os_string();
    name.push(".meta.json");
    PathBuf::from(name)
}

/// An exported packet written into tracked working-tree space becomes evidence about itself: the
/// next build sees a dirty tree and a new untracked file, so the packet actually assessed and the
/// packet later recorded diverge. `.codeos-state/` is already excluded from the diff, which makes
/// it the safe destination; anywhere outside the repository is equally safe.
fn warn_if_export_pollutes_evidence(path: &Path, repo_root: &Path, state_dir: &Path) {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };
    if absolute.starts_with(repo_root) && !absolute.starts_with(state_dir) {
        eprintln!(
            "WARNING: exporting the packet to {} adds an untracked file to the reviewed tree,",
            path.display()
        );
        eprintln!(
            "         so the next packet build will differ from this one. Prefer {}/ or a path outside the repository.",
            state_dir.display()
        );
    }
}

fn print_plan_summary(
    args: &EvidenceArgs,
    packet: &ReviewPacket,
    budget_mode: crate::config::PacketBudgetMode,
) {
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
        print_budget_warning(args, packet, budget_mode);
    }
    if matches!(packet.coverage_state, CoverageState::EmptyPacket) {
        println!();
        println!("  EMPTY_PACKET: no reviewable content found; review would fail before Codex invocation.");
    }
}

fn print_budget_warning(
    args: &EvidenceArgs,
    packet: &ReviewPacket,
    budget_mode: crate::config::PacketBudgetMode,
) {
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
    match budget_mode {
        crate::config::PacketBudgetMode::Fail => println!(
            "  Codex review will refuse this packet; reduce evidence or intentionally set CODEOS_PACKET_BUDGET_MODE=warn"
        ),
        crate::config::PacketBudgetMode::Warn => println!(
            "  operator override active: CODEOS_PACKET_BUDGET_MODE=warn permits this oversized Codex invocation"
        ),
    }
}
