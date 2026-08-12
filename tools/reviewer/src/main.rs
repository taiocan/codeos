use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod assessment;
mod cmd;
mod config;
mod log;
mod packet;
mod precheck;
mod provider;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_USAGE: i32 = 1;
pub const EXIT_CONFIG: i32 = 2;
pub const EXIT_PROVIDER: i32 = 3;
pub const EXIT_PACKET: i32 = 4;
pub const EXIT_WRITE: i32 = 5;
pub const EXIT_DRIFT: i32 = 6;

#[derive(Parser)]
#[command(name = "codeos-reviewer", version = "0.1.0")]
#[command(about = "Codeos advisory reviewer pipeline — advisory, read-only, non-gatekeeping")]
struct Cli {
    /// Override provider for this invocation (highest priority)
    #[arg(long, global = true)]
    provider: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build packet, invoke reviewer, save assessment, append log
    ///
    /// Artifact paths and options:
    ///   <paths>               Files to include in packet
    ///   --mode delta          Include only diff since base; requires --base and tracked files
    ///   --base <commit-sha>   Base commit for delta mode
    ///   --sha-only <path>     Include path/hash only; reduces review evidence
    ///   --print-packet        Print packet to stdout instead of invoking reviewer
    ///   --fresh               Force fresh session
    ///
    /// Examples:
    ///   # Round 1: full review
    ///   codeos-reviewer review UPG-0042 selfdev-step-1 \
    ///     changes/UPG-0042__CHG-*.md src/packet.rs
    ///
    ///   # Round 2+: delta review after fixes
    ///   codeos-reviewer review UPG-0042 selfdev-step-1 \
    ///     --mode delta --base abc123 \
    ///     changes/UPG-0042__CHG-*.md src/packet.rs
    ///
    ///   # Large unchanged context file; reduces evidence for that path
    ///   codeos-reviewer review UPG-0042 selfdev-step-3 \
    ///     --sha-only tests/smoke.rs \
    ///     changes/UPG-0042__CHG-*.md src/packet.rs
    Review {
        feature: String,
        stage: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        rest: Vec<String>,
    },
    /// Preview what `review` would send — resolved artifacts, evidence mode, packet size vs.
    /// budget — without invoking Codex or writing anything. Accepts the exact same arguments as
    /// `review`.
    ///
    /// Examples:
    ///   codeos-reviewer plan UPG-0042 selfdev-step-1 changes/UPG-0042__CHG-*.md src/packet.rs
    ///   codeos-reviewer plan UPG-0042 selfdev-step-1 --mode delta --base abc123 changes/UPG-0042__CHG-*.md
    Plan {
        feature: String,
        stage: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        rest: Vec<String>,
    },
    /// Append a human decision entry to the log
    Decision {
        feature: String,
        stage: String,
        verdict: String,
        reason: String,
        /// Override the coverage gate for CRITICAL_OMISSION / EMPTY_PACKET; requires a rationale
        #[arg(long = "override", value_name = "RATIONALE")]
        override_reason: Option<String>,
    },
    /// Show config resolution and diagnostic info
    Diagnose {
        feature: Option<String>,
        stage: Option<String>,
    },
    /// Record base commit for a stage (for future delta reviews)
    StageStart {
        feature: String,
        stage: String,
        #[arg(long)]
        base: Option<String>,
    },
    /// Detect stack/config drift — exits 6 if watched dependency/config files changed without a reconciliation report
    CheckDrift {
        /// Git ref to diff against (default: main)
        #[arg(long, default_value = "main")]
        base: String,
        /// Prefix output with STRICT MODE (same exit behaviour)
        #[arg(long)]
        strict: bool,
    },
    /// Generate a Stage 4/5/6 report skeleton with mechanically inferred fields
    GenerateReport {
        /// Report stage: 4, 5, or 6
        #[arg(long)]
        stage: String,
        /// Feature id (e.g. UPG-0021) to populate the Feature field
        #[arg(long)]
        feature: Option<String>,
        /// Git ref to diff against for Stage 4 Files changed (git diff --name-only <base>..HEAD)
        #[arg(long)]
        base: Option<String>,
        /// Path to a cargo test output file to parse Stage 5 test counts from
        #[arg(long = "test-output")]
        test_output: Option<String>,
        /// Path to a JSONL events file to count for Stage 6 Events captured
        #[arg(long)]
        events: Option<String>,
    },
    /// Extract "## Architectural Risks" bullets from a 00b Solution Discovery doc into
    /// non-authoritative ADR candidate skeletons
    GenerateAdrCandidates {
        /// Path to the 00b Solution Discovery source document
        #[arg(long)]
        source: String,
    },
    /// Generate a human approval dashboard from a feature-registry.yaml file
    GenerateApprovalDashboard {
        /// Path to the feature registry YAML file
        #[arg(long)]
        registry: String,
    },
    /// Generate a pre-release evidence package skeleton for a feature
    GenerateReleaseEvidence {
        /// Feature id (e.g. UPG-0024) to populate the Feature field
        #[arg(long)]
        feature: String,
        /// Optional path to a feature-registry.yaml file to enrich PR / Approved artifacts
        #[arg(long)]
        registry: Option<String>,
    },
    /// Deterministically read and validate Architecture Scope metadata
    InspectArchitectureScopes {
        /// Resolve one feature to zero or one Architecture Scope
        #[arg(long)]
        feature: Option<String>,
    },
}

fn main() {
    let exit_code = run();
    process::exit(exit_code);
}

fn run() -> i32 {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            // Help and version print to stdout and exit 0; real argument errors exit 1 (AC-3).
            let code = if e.use_stderr() { EXIT_USAGE } else { EXIT_SUCCESS };
            e.print().unwrap_or(());
            return code;
        }
    };

    // Discover repo root
    let repo_root = match discover_repo_root() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {}", e);
            return EXIT_CONFIG;
        }
    };

    // Dispatch check-drift before config resolution — needs no provider config.
    if let Commands::CheckDrift { base, strict } = &cli.command {
        return cmd::check_drift::run(base, *strict, &repo_root);
    }

    // Dispatch generate-report before config resolution — needs no provider config.
    if let Commands::GenerateReport { stage, feature, base, test_output, events } = &cli.command {
        let args = cmd::generate_report::GenerateReportArgs {
            stage,
            feature: feature.as_deref(),
            base: base.as_deref(),
            test_output: test_output.as_deref(),
            events: events.as_deref(),
        };
        return cmd::generate_report::run(args, &repo_root);
    }

    // Dispatch generate-adr-candidates before config resolution — needs no provider config.
    if let Commands::GenerateAdrCandidates { source } = &cli.command {
        return cmd::generate_adr_candidates::run(cmd::generate_adr_candidates::GenerateAdrCandidatesArgs {
            source,
        });
    }

    // Dispatch generate-approval-dashboard before config resolution — needs no provider config.
    if let Commands::GenerateApprovalDashboard { registry } = &cli.command {
        return cmd::generate_approval_dashboard::run(cmd::generate_approval_dashboard::GenerateApprovalDashboardArgs {
            registry,
        });
    }

    // Dispatch generate-release-evidence before config resolution — needs no provider config.
    if let Commands::GenerateReleaseEvidence { feature, registry } = &cli.command {
        return cmd::generate_release_evidence::run(
            cmd::generate_release_evidence::GenerateReleaseEvidenceArgs {
                feature,
                registry: registry.as_deref(),
            },
            &repo_root,
        );
    }

    // This is a deterministic reader hosted by the existing binary. It invokes no reviewer and
    // produces no verdict or approval.
    if let Commands::InspectArchitectureScopes { feature } = &cli.command {
        return cmd::inspect_architecture_scopes::run(feature.as_deref(), &repo_root);
    }

    // Resolve config
    let cfg = match config::resolve(cli.provider.as_deref(), &repo_root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {}", e);
            return EXIT_CONFIG;
        }
    };

    match cli.command {
        Commands::Review { feature, stage, rest } => {
            let parsed = match cmd::review::parse_rest(&rest) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{}", e);
                    return EXIT_USAGE;
                }
            };
            let (artifacts, sha_only, guard_clean, fresh, scratch, print_only,
                 skip_prechecks, delta_mode, delta_base) = parsed;

            let args = cmd::review::ReviewArgs {
                feature, stage, artifacts, sha_only, guard_clean,
                fresh, scratch, print_only, skip_prechecks, delta_mode, delta_base,
            };

            match cmd::review::run(args, &cfg, &cfg.provider_name) {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("internal error: {}", e);
                    EXIT_WRITE
                }
            }
        }

        Commands::Plan { feature, stage, rest } => {
            let parsed = match cmd::review::parse_rest(&rest) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{}", e);
                    return EXIT_USAGE;
                }
            };
            let (artifacts, sha_only, guard_clean, fresh, scratch, print_only,
                 skip_prechecks, delta_mode, delta_base) = parsed;

            let args = cmd::review::ReviewArgs {
                feature, stage, artifacts, sha_only, guard_clean,
                fresh, scratch, print_only, skip_prechecks, delta_mode, delta_base,
            };

            match cmd::plan::run(args, &cfg) {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("internal error: {}", e);
                    EXIT_WRITE
                }
            }
        }

        Commands::Decision { feature, stage, verdict, reason, override_reason } => {
            match cmd::decision::run(&feature, &stage, &verdict, &reason, override_reason.as_deref(), &cfg) {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("error: {}", e);
                    EXIT_WRITE
                }
            }
        }

        Commands::Diagnose { feature, stage } => {
            cmd::diagnose::run(
                feature.as_deref(),
                stage.as_deref(),
                &cfg,
                cfg.provider_source,
            );
            EXIT_SUCCESS
        }

        Commands::StageStart { feature, stage, base } => {
            match run_stage_start(&feature, &stage, base.as_deref(), &cfg) {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("error: {}", e);
                    EXIT_CONFIG
                }
            }
        }

        // Handled above before config resolution; unreachable here.
        Commands::CheckDrift { .. } => EXIT_SUCCESS,
        Commands::GenerateReport { .. } => EXIT_SUCCESS,
        Commands::GenerateAdrCandidates { .. } => EXIT_SUCCESS,
        Commands::GenerateApprovalDashboard { .. } => EXIT_SUCCESS,
        Commands::GenerateReleaseEvidence { .. } => EXIT_SUCCESS,
        Commands::InspectArchitectureScopes { .. } => EXIT_SUCCESS,
    }
}

fn discover_repo_root() -> anyhow::Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|_| anyhow::anyhow!("not inside a git repository"))?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

fn run_stage_start(
    feature: &str,
    stage: &str,
    base: Option<&str>,
    cfg: &config::Config,
) -> anyhow::Result<i32> {
    let base_sha = if let Some(b) = base {
        b.to_string()
    } else {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&cfg.repo_root)
            .output()
            .map_err(|e| anyhow::anyhow!("git rev-parse: {}", e))?;
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };

    let branch_out = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&cfg.repo_root)
        .output()
        .map_err(|e| anyhow::anyhow!("git branch: {}", e))?;
    let branch = String::from_utf8_lossy(&branch_out.stdout).trim().to_string();

    let dir = cfg.stage_start_dir.join(feature);
    std::fs::create_dir_all(&dir)
        .map_err(|e| anyhow::anyhow!("create stage-start dir: {}", e))?;

    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let json = format!(
        "{{\n  \"feature\": \"{}\",\n  \"stage\": \"{}\",\n  \"base_commit\": \"{}\",\n  \"branch\": \"{}\",\n  \"started_at\": \"{}\"\n}}\n",
        feature, stage, base_sha, branch, ts
    );
    let out_file = dir.join(format!("stage-{}.json", stage));
    std::fs::write(&out_file, &json)
        .map_err(|e| anyhow::anyhow!("write stage-start file: {}", e))?;

    println!("stage-start recorded: {} (base {})", out_file.display(), base_sha);
    Ok(EXIT_SUCCESS)
}
