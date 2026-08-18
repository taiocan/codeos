use crate::assessment;
use crate::codex;
use crate::config::Config;
use crate::log as review_log;
use crate::packet::{self, PacketBuildOptions, ReviewPacket};
use crate::precheck;
use crate::run::{ReviewerRun, RunSource};
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct EvidenceArgs {
    pub feature: String,
    pub stage: String,
    pub artifacts: Vec<String>,
    pub sha_only: Vec<String>,
    pub guard_clean: Vec<String>,
    pub base: Option<String>,
    pub skip_prechecks: bool,
}

pub struct ReviewArgs {
    pub evidence: EvidenceArgs,
    pub fresh: bool,
    pub scratch: bool,
    /// Record an assessment produced outside Codeos instead of invoking Codex.
    pub assessment: Option<PathBuf>,
    /// The exported packet the external model actually read. Required with `--assessment`.
    pub packet: Option<PathBuf>,
    /// Descriptive label for the external model. Metadata only.
    pub reviewer_label: Option<String>,
}

pub struct PreparedReview {
    pub args: EvidenceArgs,
    pub packet: ReviewPacket,
}

pub struct PrepareFailure {
    pub code: i32,
    pub message: String,
}

impl PrepareFailure {
    fn usage(message: impl Into<String>) -> Self {
        Self {
            code: crate::EXIT_USAGE,
            message: message.into(),
        }
    }

    fn packet(message: impl Into<String>) -> Self {
        Self {
            code: crate::EXIT_PACKET,
            message: message.into(),
        }
    }

    fn config(message: impl Into<String>) -> Self {
        Self {
            code: crate::EXIT_CONFIG,
            message: message.into(),
        }
    }
}

/// Apply the validation, evidence selection, prechecks, and packet construction shared by
/// `plan` and `review`. Callers differ only after this preparation succeeds.
pub fn prepare(
    mut args: EvidenceArgs,
    cfg: &Config,
) -> std::result::Result<PreparedReview, PrepareFailure> {
    validate_identifier("feature", &args.feature)?;
    validate_identifier("workflow-or-stage", &args.stage)?;

    args.artifacts = normalize_paths(&args.artifacts, cfg, true, "artifact")?;
    args.sha_only = normalize_paths(&args.sha_only, cfg, true, "--sha-only")?;
    args.guard_clean = normalize_paths(&args.guard_clean, cfg, false, "--guard-clean")?;

    for path in &args.sha_only {
        if args.artifacts.contains(path) {
            return Err(PrepareFailure::usage(format!(
                "path '{path}' was passed both as an artifact and --sha-only"
            )));
        }
    }

    if let Some(base) = args.base.as_deref() {
        args.base = Some(resolve_base(base, cfg)?);
    }

    if !args.skip_prechecks {
        for artifact in &args.artifacts {
            let content = std::fs::read_to_string(artifact).map_err(|error| {
                PrepareFailure::packet(format!("could not read artifact {artifact}: {error}"))
            })?;
            precheck::check_no_unfilled_placeholders(Path::new(artifact), &content)
                .map_err(|error| PrepareFailure::packet(error.to_string()))?;
            precheck::check_no_forbidden_fields(Path::new(artifact), &content)
                .map_err(|error| PrepareFailure::packet(error.to_string()))?;
            precheck::check_draft_markers(Path::new(artifact), &content);
        }
        for path in &args.guard_clean {
            precheck::check_guard_clean(Path::new(path))
                .map_err(|error| PrepareFailure::packet(error.to_string()))?;
        }
    }

    let packet = packet::build(&PacketBuildOptions {
        feature: args.feature.clone(),
        stage: args.stage.clone(),
        artifacts: args.artifacts.clone(),
        sha_only_paths: args.sha_only.clone(),
        delta_mode: args.base.is_some(),
        delta_base: args.base.clone(),
        repo_root: cfg.repo_root.to_string_lossy().into_owned(),
        toolkit_root: cfg.toolkit_root.to_string_lossy().into_owned(),
    })
    .map_err(|error| PrepareFailure::packet(format!("error building packet: {error}")))?;

    Ok(PreparedReview { args, packet })
}

pub fn run(args: ReviewArgs, cfg: &Config) -> Result<i32> {
    // Two ways to obtain the packet, and only one of them builds anything. The import path adopts
    // the exported bytes: the packet the model read and the packet recorded as reviewed are the
    // same bytes, not two builds that happen to agree.
    let (evidence, review_packet) = if let Some(packet_path) = args.packet.as_ref() {
        match load_exported_packet(packet_path, &args.evidence) {
            Ok(packet) => (args.evidence.clone(), packet),
            Err(failure) => {
                eprintln!("error: {}", failure.message);
                return Ok(failure.code);
            }
        }
    } else {
        match prepare(args.evidence.clone(), cfg) {
            Ok(prepared) => (prepared.args, prepared.packet),
            Err(failure) => {
                eprintln!("error: {}", failure.message);
                return Ok(failure.code);
            }
        }
    };

    if matches!(
        review_packet.coverage_state,
        crate::packet::CoverageState::EmptyPacket
    ) {
        eprintln!("error: review packet is empty (EMPTY_PACKET) — no reviewable content found.");
        if evidence.base.is_some() {
            eprintln!("       Ensure tracked artifacts changed since --base, or omit --base for a full review.");
        }
        eprintln!("       Inspect the evidence with plan before rerunning.");
        return Ok(crate::EXIT_PACKET);
    }

    let review_log_path = if args.scratch {
        let scratch = cfg.state_dir.join("reviewer-scratch");
        if let Err(error) = std::fs::create_dir_all(&scratch) {
            eprintln!(
                "error: could not create scratch dir {}: {error}",
                scratch.display()
            );
            return Ok(crate::EXIT_WRITE);
        }
        scratch.join("review-log.md")
    } else {
        cfg.review_log.clone()
    };

    // An external assessment is sequenced separately from review rounds, so importing one can
    // never consume a round the human still owes the boundary.
    let review_id = if args.assessment.is_some() {
        match review_log::compute_assessment_sequence(
            &review_log_path,
            &evidence.feature,
            &evidence.stage,
        ) {
            Ok(sequence) => {
                review_log::format_assessment_id(&evidence.feature, &evidence.stage, sequence)
            }
            Err(error) => {
                eprintln!("error: could not determine assessment sequence: {error}");
                return Ok(crate::EXIT_WRITE);
            }
        }
    } else {
        match review_log::compute_review_round(&review_log_path, &evidence.feature, &evidence.stage)
        {
            Ok(round) => review_log::format_review_id(&evidence.feature, &evidence.stage, round),
            Err(error) => {
                eprintln!("error: could not determine review round: {error}");
                return Ok(crate::EXIT_WRITE);
            }
        }
    };

    let raw = match &args.assessment {
        // Import path: no process is started, so there is no read-only violation to detect and no
        // session to resume. The text is taken verbatim; it flows through the same parser and the
        // same fail-closed schema check as a Codex reply.
        Some(path) => match std::fs::read_to_string(path) {
            Ok(text) if text.trim().is_empty() => {
                eprintln!("error: external assessment file is empty: {}", path.display());
                return Ok(crate::EXIT_PROVIDER);
            }
            Ok(text) => ReviewerRun {
                text,
                elapsed_ms: 0,
                source: RunSource::External {
                    label: args.reviewer_label.clone(),
                },
            },
            Err(error) => {
                eprintln!(
                    "error: could not read external assessment {}: {error}",
                    path.display()
                );
                return Ok(crate::EXIT_PROVIDER);
            }
        },
        None => {
            let pre_invoke_status = git_status(&cfg.repo_root);
            let invoke_result = codex::invoke(&review_packet, cfg, args.fresh);
            if let (Some(before), Some(after)) = (pre_invoke_status, git_status(&cfg.repo_root)) {
                if before != after {
                    eprintln!(
                        "WARNING: working tree changed during review — reviewer should be read-only"
                    );
                }
            }
            match invoke_result {
                Ok(raw) => raw,
                Err(error) => {
                    eprintln!("error: Codex invocation failed: {error}");
                    return Ok(crate::EXIT_PROVIDER);
                }
            }
        }
    };

    // Findings first: whether they all parsed is an input to whether the assessment is usable.
    let (findings, unparsed_findings_count) = assessment::parse_findings(&raw.text, &review_id);
    let parsed = assessment::parse_review_output(
        &raw.text,
        &review_packet.coverage_state,
        findings.len(),
        unparsed_findings_count,
    );
    // The recorded count covers every finding the reply declared but that never reached the record:
    // malformed blocks plus declarations in a shape the parser does not accept.
    let declared_findings = assessment::count_declared_findings(&raw.text);
    let unrecorded_findings_count = unparsed_findings_count
        + declared_findings.saturating_sub(findings.len() + unparsed_findings_count);
    let outdir = if args.scratch {
        cfg.state_dir.join("reviewer-scratch")
    } else {
        cfg.codex_dir.clone()
    };
    let packets_dir = outdir.join("packets");
    if let Err(error) = std::fs::create_dir_all(&packets_dir) {
        eprintln!(
            "error: could not create packets dir {}: {error}",
            packets_dir.display()
        );
        return Ok(crate::EXIT_WRITE);
    }

    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let short_sha = &review_packet.review_sha[..7.min(review_packet.review_sha.len())];
    let packet_saved = packets_dir.join(format!(
        "{}-{}-stage-{}-{}.packet.txt",
        timestamp, evidence.feature, evidence.stage, short_sha
    ));
    if let Err(error) = std::fs::write(&packet_saved, review_packet.content()) {
        eprintln!(
            "error: could not write packet file {}: {error}",
            packet_saved.display()
        );
        return Ok(crate::EXIT_WRITE);
    }
    let packet_path = match packet_saved.to_str() {
        Some(path) => path,
        None => {
            eprintln!(
                "error: packet path contains non-UTF-8 characters: {}",
                packet_saved.display()
            );
            return Ok(crate::EXIT_WRITE);
        }
    };
    let packet_hash = match packet::sha256_file(packet_path) {
        Ok(hash) => hash,
        Err(error) => {
            eprintln!("error: could not hash packet file: {error}");
            return Ok(crate::EXIT_WRITE);
        }
    };

    if let Err(error) = assessment::validate_schema(&review_packet, &parsed, &packet_hash) {
        eprintln!("error: {error}");
        return Ok(crate::EXIT_PACKET);
    }
    let (assessment_file, assessment_hash) = match assessment::write_assessment(
        &review_id,
        &findings,
        unrecorded_findings_count,
        &review_packet,
        &raw,
        &parsed,
        &outdir,
        &packet_saved,
        &packet_hash,
    ) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("error: assessment write failed: {error}");
            eprintln!("  packet was written to: {}", packet_saved.display());
            return Ok(crate::EXIT_WRITE);
        }
    };

    if let Err(error) = review_log::append_review(
        &review_log_path,
        &review_id,
        &review_packet,
        &raw,
        &parsed,
        &assessment_file,
        &assessment_hash,
        &packet_saved,
        &packet_hash,
    ) {
        eprintln!("error: log append failed: {error}");
        eprintln!("  assessment was written to: {}", assessment_file.display());
        eprintln!("  packet was written to: {}", packet_saved.display());
        return Ok(crate::EXIT_WRITE);
    }

    match &raw.source {
        RunSource::Codex {
            reconnect_count,
            effort,
            ..
        } => {
            println!("review logged: {}", review_log_path.display());
            println!("  review_id: {review_id}");
            println!(
                "  codex concern: {}   effective concern: {}   evidence: {}",
                parsed.codex_concern, parsed.effective_concern, parsed.evidence
            );
            println!(
                "  effort: {}   elapsed: {}ms   reconnects: {}",
                effort, raw.elapsed_ms, reconnect_count
            );
        }
        RunSource::External { .. } => {
            println!("external assessment logged: {}", review_log_path.display());
            println!("  assessment_id: {review_id}");
            println!(
                "  reported concern: {}   effective concern: {}   evidence: {}",
                parsed.codex_concern, parsed.effective_concern, parsed.evidence
            );
            println!("  advisory only: this does NOT satisfy a required review round.");
        }
    }
    if parsed.assessment_status != "COMPLETE" {
        println!(
            "  assessment: {} — {} — recorded as DO NOT ADVANCE; this cannot clear a boundary.",
            parsed.assessment_status, parsed.incomplete_reason
        );
    }
    println!(
        "  coverage: {} (redactions: {})",
        review_packet.coverage_state.as_str(),
        review_packet.redaction_count
    );
    println!("  assessment: {}", assessment_file.display());
    println!("  packet: {}", packet_saved.display());
    Ok(crate::EXIT_SUCCESS)
}

/// Load an exported packet and its sidecar without rebuilding either. The sidecar is what makes
/// this possible; the checks below are what make it trustworthy.
fn load_exported_packet(
    packet_path: &Path,
    evidence: &EvidenceArgs,
) -> std::result::Result<ReviewPacket, PrepareFailure> {
    let content = std::fs::read_to_string(packet_path).map_err(|error| {
        PrepareFailure::packet(format!(
            "could not read exported packet {}: {error}",
            packet_path.display()
        ))
    })?;
    let sidecar_path = crate::cmd::plan::sidecar_path(packet_path);
    let sidecar = std::fs::read_to_string(&sidecar_path).map_err(|error| {
        PrepareFailure::packet(format!(
            "could not read packet sidecar {}: {error}\n                    the sidecar is written beside the packet by `plan --emit-packet`",
            sidecar_path.display()
        ))
    })?;
    let packet: ReviewPacket = serde_json::from_str(&sidecar).map_err(|error| {
        PrepareFailure::packet(format!(
            "malformed packet sidecar {}: {error}",
            sidecar_path.display()
        ))
    })?;

    // The sidecar describes a packet; these checks establish that it describes *this* packet, and
    // that the operator is importing the assessment against the evidence they named.
    if packet.feature != evidence.feature {
        return Err(PrepareFailure::packet(format!(
            "exported packet is for feature '{}', not '{}'",
            packet.feature, evidence.feature
        )));
    }
    if packet.stage != evidence.stage {
        return Err(PrepareFailure::packet(format!(
            "exported packet is for stage '{}', not '{}'",
            packet.stage, evidence.stage
        )));
    }
    let mut exported: Vec<&str> = packet.artifacts.iter().map(|a| a.path.as_str()).collect();
    let mut named: Vec<&str> = evidence.artifacts.iter().map(String::as_str).collect();
    exported.sort_unstable();
    named.sort_unstable();
    if exported != named {
        return Err(PrepareFailure::packet(format!(
            "exported packet covers artifacts [{}], not [{}]",
            exported.join(", "),
            named.join(", ")
        )));
    }

    Ok(packet.from_sidecar(content))
}

fn validate_identifier(label: &str, value: &str) -> std::result::Result<(), PrepareFailure> {
    let mut characters = value.chars();
    let valid = characters
        .next()
        .is_some_and(|character| character.is_ascii_alphanumeric())
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        });
    if valid {
        Ok(())
    } else {
        Err(PrepareFailure::usage(format!(
            "{label} must start with an ASCII letter or digit and contain only letters, digits, '.', '_', or '-'"
        )))
    }
}

fn normalize_paths(
    paths: &[String],
    cfg: &Config,
    require_file: bool,
    label: &str,
) -> std::result::Result<Vec<String>, PrepareFailure> {
    let root = cfg.repo_root.canonicalize().map_err(|error| {
        PrepareFailure::config(format!("could not resolve repository root: {error}"))
    })?;
    paths
        .iter()
        .map(|path| normalize_path(path, &root, require_file, label))
        .collect()
}

fn normalize_path(
    supplied: &str,
    root: &Path,
    require_file: bool,
    label: &str,
) -> std::result::Result<String, PrepareFailure> {
    let absolute = PathBuf::from(supplied).canonicalize().map_err(|error| {
        PrepareFailure::packet(format!(
            "{label} path does not resolve: {supplied}: {error}"
        ))
    })?;
    if require_file && !absolute.is_file() {
        return Err(PrepareFailure::packet(format!(
            "{label} path is not a regular file: {supplied}"
        )));
    }
    let relative = absolute.strip_prefix(root).map_err(|_| {
        PrepareFailure::packet(format!(
            "{label} path resolves outside the repository: {supplied}"
        ))
    })?;
    relative
        .to_str()
        .map(|path| path.to_string())
        .ok_or_else(|| {
            PrepareFailure::packet(format!("{label} path is not valid UTF-8: {supplied}"))
        })
}

fn resolve_base(base: &str, cfg: &Config) -> std::result::Result<String, PrepareFailure> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--verify", &format!("{base}^{{commit}}")])
        .current_dir(&cfg.repo_root)
        .output()
        .map_err(|error| PrepareFailure::config(format!("could not run git: {error}")))?;
    if !output.status.success() {
        return Err(PrepareFailure::usage(format!(
            "--base '{base}' does not resolve to a commit"
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_status(repo_root: &Path) -> Option<Vec<u8>> {
    std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| output.stdout)
}
