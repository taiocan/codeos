use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod assessment;
mod cmd;
mod codex;
mod config;
mod log;
mod packet;
mod precheck;
mod run;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_USAGE: i32 = 1;
pub const EXIT_CONFIG: i32 = 2;
pub const EXIT_PROVIDER: i32 = 3;
pub const EXIT_PACKET: i32 = 4;
pub const EXIT_WRITE: i32 = 5;

#[derive(Parser)]
#[command(name = "codeos-reviewer", version)]
#[command(about = "Codeos advisory reviewer pipeline — advisory, read-only, non-gatekeeping")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Clone)]
struct EvidenceCli {
    feature: String,
    #[arg(value_name = "WORKFLOW_OR_STAGE")]
    stage: String,
    /// Include a path and hash but omit its content.
    #[arg(long = "sha-only", value_name = "PATH")]
    sha_only: Vec<String>,
    /// Require a path to have no uncommitted changes.
    #[arg(long = "guard-clean", value_name = "PATH")]
    guard_clean: Vec<String>,
    /// Review changes since this Git ref; omission selects a full review.
    #[arg(long, value_name = "REF")]
    base: Option<String>,
    /// Skip deterministic artifact prechecks.
    #[arg(long)]
    skip_prechecks: bool,
    /// Files to include as review evidence.
    #[arg(value_name = "ARTIFACT", required = true)]
    artifacts: Vec<String>,
}

#[derive(Args)]
struct ReviewCli {
    #[command(flatten)]
    evidence: EvidenceCli,
    /// Compatibility flag; Codex-backed reviews are always fresh and ephemeral.
    #[arg(long)]
    fresh: bool,
    /// Write reviewer records under ignored operational state.
    #[arg(long)]
    scratch: bool,
    /// Record an external assessment from this file instead of invoking Codex. The assessment is
    /// advisory evidence: it is recorded with source: external and never counts as a review round.
    /// Requires --packet: the assessment is bound to the exact packet the model read.
    #[arg(long = "assessment", value_name = "FILE", requires = "packet")]
    assessment: Option<PathBuf>,
    /// The packet exported by `plan --emit-packet` that the external model read. Its bytes are
    /// recorded as the reviewed packet; no packet is rebuilt. Requires --assessment.
    #[arg(long = "packet", value_name = "FILE", requires = "assessment")]
    packet: Option<PathBuf>,
    /// Descriptive label for the model that produced an external assessment. Metadata only: Codeos
    /// neither invoked nor verified it. Requires --assessment.
    #[arg(long = "reviewer-label", value_name = "LABEL", requires = "assessment")]
    reviewer_label: Option<String>,
    /// This review continues an existing one (UPG-0074) — e.g. it runs on an isolated branch or
    /// worktree whose local log predates the predecessor's entries, which would otherwise make the
    /// round-by-heading-count default mis-derive round 1 for what is substantively a later round.
    /// The referenced ID is validated against the log (same feature, same stage, actually exists,
    /// no cycle, and the resolved round does not exceed the three-round budget) rather than
    /// trusted from this flag alone. Omit for the ordinary case; behavior is unchanged.
    #[arg(long, value_name = "REVIEW_ID")]
    continues: Option<String>,
}

#[derive(Args)]
struct PlanCli {
    #[command(flatten)]
    evidence: EvidenceCli,
    /// Write the exact packet `review` would send to this file, for assessment by an external model.
    #[arg(long = "emit-packet", value_name = "FILE")]
    emit_packet: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build evidence, invoke Codex, save the assessment, and append the review log.
    Review(ReviewCli),
    /// Preview evidence selection and packet size without invoking Codex or writing records.
    Plan(PlanCli),
    /// Append a human decision associated with an automated review.
    Decision {
        feature: String,
        #[arg(value_name = "WORKFLOW_OR_STAGE")]
        stage: String,
        verdict: String,
        reason: String,
        #[arg(long = "override", value_name = "RATIONALE")]
        override_reason: Option<String>,
    },
    /// Show resolved reviewer configuration and state.
    Diagnose {
        feature: Option<String>,
        #[arg(value_name = "WORKFLOW_OR_STAGE")]
        stage: Option<String>,
    },
    /// Deterministically read and validate Architecture Scope metadata.
    InspectArchitectureScopes {
        #[arg(long)]
        feature: Option<String>,
    },
}

impl From<EvidenceCli> for cmd::review::EvidenceArgs {
    fn from(value: EvidenceCli) -> Self {
        Self {
            feature: value.feature,
            stage: value.stage,
            artifacts: value.artifacts,
            sha_only: value.sha_only,
            guard_clean: value.guard_clean,
            base: value.base,
            skip_prechecks: value.skip_prechecks,
        }
    }
}

fn main() {
    process::exit(run());
}

fn run() -> i32 {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let code = if error.use_stderr() {
                EXIT_USAGE
            } else {
                EXIT_SUCCESS
            };
            error.print().unwrap_or(());
            return code;
        }
    };

    let repo_root = match discover_repo_root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("error: {error}");
            return EXIT_CONFIG;
        }
    };
    if let Err(error) = std::env::set_current_dir(&repo_root) {
        eprintln!(
            "error: could not enter repository root {}: {error}",
            repo_root.display()
        );
        return EXIT_CONFIG;
    }

    if let Commands::InspectArchitectureScopes { feature } = &cli.command {
        return cmd::inspect_architecture_scopes::run(feature.as_deref(), &repo_root);
    }

    let cfg = match config::resolve(&repo_root) {
        Ok(cfg) => cfg,
        Err(error) => {
            eprintln!("error: {error}");
            return EXIT_CONFIG;
        }
    };

    match cli.command {
        Commands::Review(args) => {
            // Retained on the public CLI; all Codex-backed reviews now have this behavior.
            let _ = args.fresh;
            cmd::review::run(
                cmd::review::ReviewArgs {
                    evidence: args.evidence.into(),
                    scratch: args.scratch,
                    assessment: args.assessment,
                    packet: args.packet,
                    reviewer_label: args.reviewer_label,
                    continues: args.continues,
                },
                &cfg,
            )
            .unwrap_or_else(|error| {
                eprintln!("internal error: {error}");
                EXIT_WRITE
            })
        }
        Commands::Plan(args) => cmd::plan::run(args.evidence.into(), args.emit_packet, &cfg)
            .unwrap_or_else(|error| {
                eprintln!("internal error: {error}");
                EXIT_WRITE
            }),
        Commands::Decision {
            feature,
            stage,
            verdict,
            reason,
            override_reason,
        } => cmd::decision::run(
            &feature,
            &stage,
            &verdict,
            &reason,
            override_reason.as_deref(),
            &cfg,
        )
        .unwrap_or_else(|error| {
            eprintln!("error: {error}");
            EXIT_WRITE
        }),
        Commands::Diagnose { feature, stage } => {
            cmd::diagnose::run(feature.as_deref(), stage.as_deref(), &cfg);
            EXIT_SUCCESS
        }
        Commands::InspectArchitectureScopes { .. } => EXIT_SUCCESS,
    }
}

fn discover_repo_root() -> anyhow::Result<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|_| anyhow::anyhow!("not inside a git repository"))?;
    if !output.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}
