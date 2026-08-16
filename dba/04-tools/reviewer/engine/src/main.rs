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
    /// Start a new Codex session and replace saved session state only after success.
    #[arg(long)]
    fresh: bool,
    /// Write reviewer records under ignored operational state.
    #[arg(long)]
    scratch: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Build evidence, invoke Codex, save the assessment, and append the review log.
    Review(ReviewCli),
    /// Preview evidence selection and packet size without invoking Codex or writing records.
    Plan(EvidenceCli),
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
        Commands::Review(args) => cmd::review::run(
            cmd::review::ReviewArgs {
                evidence: args.evidence.into(),
                fresh: args.fresh,
                scratch: args.scratch,
            },
            &cfg,
        )
        .unwrap_or_else(|error| {
            eprintln!("internal error: {error}");
            EXIT_WRITE
        }),
        Commands::Plan(args) => cmd::plan::run(args.into(), &cfg).unwrap_or_else(|error| {
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
