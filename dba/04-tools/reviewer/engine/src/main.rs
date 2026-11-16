use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod accounting;
mod assessment;
mod assessment_input_correction;
mod build_identity;
mod cmd;
mod codex;
mod config;
mod lineage_replacement;
mod log;
mod manifest;
mod packet;
mod precheck;
mod projection;
mod readiness;
mod release_identity;
mod run;
mod terminal_closure;

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
    #[arg(value_name = "ARTIFACT")]
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
    /// Run the single exceptional review re-entry authorized by the human decision following an
    /// exhausted third round. The referenced review must be that feature/stage's R3.
    #[arg(
        long = "reentry-after",
        value_name = "REVIEW_ID",
        conflicts_with = "continues",
        conflicts_with = "assessment"
    )]
    reentry_after: Option<String>,
    /// Run the single evidence-only closure authorized after a consumed REENTRY-1.
    #[arg(long = "evidence-closure-after", value_name = "REVIEW_ID", conflicts_with_all = ["continues", "reentry_after", "assessment"])]
    evidence_closure_after: Option<String>,
    /// Run the terminal revalidation authorized after a negative evidence closure.
    #[arg(long = "infrastructure-revalidation-after", value_name = "REVIEW_ID", conflicts_with_all = ["continues", "reentry_after", "evidence_closure_after", "assessment"])]
    infrastructure_revalidation_after: Option<String>,
    /// Reassess named open F7 evidence findings against materially new canonical evidence.
    #[arg(long = "evidence-reassessment-after", value_name = "REVIEW_ID", conflicts_with_all = ["continues", "reentry_after", "evidence_closure_after", "infrastructure_revalidation_after", "assessment"])]
    evidence_reassessment_after: Option<String>,
    /// Close the finite repaired conditions from one consumed terminal REENTRY-1.
    #[arg(long = "terminal-review-closure-after", value_name = "REVIEW_ID", conflicts_with_all = ["continues", "reentry_after", "evidence_closure_after", "infrastructure_revalidation_after", "evidence_reassessment_after", "assessment"])]
    terminal_review_closure_after: Option<String>,
    /// Retry one structurally failed Terminal Review Closure with its exact saved payload.
    #[arg(long = "retry-terminal-review-closure-after", value_name = "REVIEW_ID", conflicts_with_all = ["continues", "reentry_after", "evidence_closure_after", "infrastructure_revalidation_after", "evidence_reassessment_after", "terminal_review_closure_after", "repair_manifest", "assessment", "packet", "findings", "event_evidence_sha"])]
    retry_terminal_review_closure_after: Option<String>,
    /// Replace one complete assessment whose mandatory input lineage was mechanically invalid.
    #[arg(long = "replace-invalid-lineage-after", value_name = "REVIEW_ID", conflicts_with_all = ["continues", "reentry_after", "evidence_closure_after", "infrastructure_revalidation_after", "evidence_reassessment_after", "terminal_review_closure_after", "retry_terminal_review_closure_after", "assessment"])]
    replace_invalid_lineage_after: Option<String>,
    /// Reassess one consumed lineage replacement using only corrected pre-existing inputs.
    #[arg(long = "correct-lineage-replacement-after", value_name = "REVIEW_ID", conflicts_with_all = ["continues", "reentry_after", "evidence_closure_after", "infrastructure_revalidation_after", "evidence_reassessment_after", "terminal_review_closure_after", "retry_terminal_review_closure_after", "replace_invalid_lineage_after", "assessment"])]
    correct_lineage_replacement_after: Option<String>,
    /// Supply one bounded evidence-representation correction for a closure retry.
    #[arg(
        long = "assessability-correction",
        value_name = "JSON",
        requires = "retry_terminal_review_closure_after"
    )]
    assessability_correction: Option<PathBuf>,
    /// Repair manifest for Terminal Review Closure.
    #[arg(
        long = "repair-manifest",
        value_name = "JSON",
        requires = "terminal_review_closure_after"
    )]
    repair_manifest: Option<PathBuf>,
    /// Exact evidence finding identities authorized for closure (comma-separated or repeated).
    #[arg(long = "findings", value_delimiter = ',')]
    findings: Vec<String>,
    /// Exact canonical runtime-event evidence set authorized for terminal revalidation.
    #[arg(long = "event-evidence-sha", value_name = "SHA256")]
    event_evidence_sha: Option<String>,
}

#[derive(Args)]
struct PlanCli {
    #[command(flatten)]
    evidence: EvidenceCli,
    /// Write the exact packet `review` would send to this file, for assessment by an external model.
    #[arg(long = "emit-packet", value_name = "FILE")]
    emit_packet: Option<PathBuf>,
    /// Preview a package-bound re-entry against the exact immutable workspace reviewed by R3.
    #[arg(long = "reentry-after", value_name = "REVIEW_ID")]
    reentry_after: Option<String>,
    /// Preview Terminal Review Closure against its immutable terminal predecessor.
    #[arg(
        long = "terminal-review-closure-after",
        value_name = "REVIEW_ID",
        conflicts_with = "reentry_after"
    )]
    terminal_review_closure_after: Option<String>,
}

#[allow(clippy::large_enum_variant)]
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
        /// Bind a review-control authorization to the identified exhausted R3 boundary.
        #[arg(long = "after-review", value_name = "REVIEW_ID")]
        after_review: Option<String>,
        #[arg(long = "findings", value_delimiter = ',')]
        findings: Vec<String>,
        #[arg(long = "event-evidence-sha", value_name = "SHA256")]
        event_evidence_sha: Option<String>,
        /// Immutable-source condition manifest for Terminal Review Closure authorization.
        #[arg(long = "closure-scope", value_name = "JSON")]
        closure_scope: Option<PathBuf>,
        /// Exact immutable applicability record authorizing a lineage replacement.
        #[arg(long = "applicability-record-sha", value_name = "SHA256")]
        applicability_record_sha: Option<String>,
        /// Exact immutable eligibility record authorizing an assessment-input correction.
        #[arg(long = "correction-record-sha", value_name = "SHA256")]
        correction_record_sha: Option<String>,
        #[arg(long = "override", value_name = "RATIONALE")]
        override_reason: Option<String>,
    },
    /// Read-only review cost and composition, derived from the review log and its records.
    ReviewStatus {
        /// Print the status as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Show resolved reviewer configuration and state.
    Diagnose {
        feature: Option<String>,
        #[arg(value_name = "WORKFLOW_OR_STAGE")]
        stage: Option<String>,
    },
    /// Deterministically record that a complete review had invalid mandatory input lineage.
    AuditReviewApplicability {
        feature: String,
        #[arg(value_name = "WORKFLOW_OR_STAGE")]
        stage: String,
        #[arg(long = "review-id")]
        review_id: String,
    },
    /// Audit whether a consumed lineage replacement failed only from correctable pre-existing inputs.
    AuditAssessmentInputCorrection {
        feature: String,
        #[arg(value_name = "WORKFLOW_OR_STAGE")]
        stage: String,
        #[arg(long = "review-id")]
        review_id: String,
    },
    /// Deterministically read and validate Architecture Scope metadata.
    InspectArchitectureScopes {
        #[arg(long)]
        feature: Option<String>,
    },
}

impl From<EvidenceCli> for cmd::review::EvidenceArgs {
    fn from(value: EvidenceCli) -> Self {
        let basis = crate::packet::ReviewBasis::Ordinary {
            base: value.base.clone(),
        };
        Self {
            feature: value.feature,
            stage: value.stage,
            artifacts: value.artifacts,
            sha_only: value.sha_only,
            guard_clean: value.guard_clean,
            base: value.base,
            skip_prechecks: value.skip_prechecks,
            basis,
        }
    }
}

fn main() {
    if let Err(error) = release_identity::verify_release_binary("codeos-reviewer") {
        eprintln!("error: {error:#}");
        process::exit(EXIT_CONFIG);
    }
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
                    reentry_after: args.reentry_after,
                    evidence_closure_after: args.evidence_closure_after,
                    infrastructure_revalidation_after: args.infrastructure_revalidation_after,
                    evidence_reassessment_after: args.evidence_reassessment_after,
                    terminal_review_closure_after: args.terminal_review_closure_after,
                    retry_terminal_review_closure_after: args.retry_terminal_review_closure_after,
                    replace_invalid_lineage_after: args.replace_invalid_lineage_after,
                    correct_lineage_replacement_after: args.correct_lineage_replacement_after,
                    assessability_correction: args.assessability_correction,
                    repair_manifest: args.repair_manifest,
                    findings: args.findings,
                    event_evidence_sha: args.event_evidence_sha,
                },
                &cfg,
            )
            .unwrap_or_else(|error| {
                eprintln!("internal error: {error}");
                EXIT_WRITE
            })
        }
        Commands::Plan(args) => {
            if args.evidence.artifacts.is_empty() {
                eprintln!("error: plan requires at least one ARTIFACT");
                EXIT_USAGE
            } else {
                let mut evidence: cmd::review::EvidenceArgs = args.evidence.into();
                if let Some(predecessor) = args.reentry_after.as_deref() {
                    match cmd::review::resolve_bounded_basis(
                        &mut evidence,
                        &cfg,
                        predecessor,
                        "package-bound REENTRY plan",
                    ) {
                        Ok(()) => {}
                        Err(error) => {
                            eprintln!("error: package-bound re-entry has no valid reviewed-workspace snapshot: {error}");
                            return EXIT_PACKET;
                        }
                    }
                }
                if let Some(predecessor) = args.terminal_review_closure_after.as_deref() {
                    match cmd::review::resolve_bounded_basis(
                        &mut evidence,
                        &cfg,
                        predecessor,
                        "Terminal Review Closure plan",
                    ) {
                        Ok(()) => {}
                        Err(error) => {
                            eprintln!("error: Terminal Review Closure has no valid immutable predecessor state: {error}");
                            return EXIT_PACKET;
                        }
                    }
                }
                cmd::plan::run(evidence, args.emit_packet, &cfg).unwrap_or_else(|error| {
                    eprintln!("internal error: {error}");
                    EXIT_WRITE
                })
            }
        }
        Commands::Decision {
            feature,
            stage,
            verdict,
            reason,
            after_review,
            findings,
            event_evidence_sha,
            closure_scope,
            applicability_record_sha,
            correction_record_sha,
            override_reason,
        } => cmd::decision::run(
            &feature,
            &stage,
            &verdict,
            &reason,
            after_review.as_deref(),
            &findings,
            event_evidence_sha.as_deref(),
            closure_scope.as_deref(),
            applicability_record_sha.as_deref(),
            correction_record_sha.as_deref(),
            override_reason.as_deref(),
            &cfg,
        )
        .unwrap_or_else(|error| {
            eprintln!("error: {error}");
            EXIT_WRITE
        }),
        Commands::ReviewStatus { json } => cmd::status::run(&cfg, json),
        Commands::Diagnose { feature, stage } => {
            cmd::diagnose::run(feature.as_deref(), stage.as_deref(), &cfg);
            EXIT_SUCCESS
        }
        Commands::AuditReviewApplicability {
            feature,
            stage,
            review_id,
        } => {
            match lineage_replacement::audit(&cfg, &feature, &stage, &review_id)
                .and_then(|record| lineage_replacement::append_audit(&cfg, &record))
            {
                Ok(sha) => {
                    println!("review applicability recorded: INVALID_REVIEW_INPUT_LINEAGE ({sha})");
                    EXIT_SUCCESS
                }
                Err(error) => {
                    eprintln!("error: review applicability audit refused: {error}");
                    EXIT_WRITE
                }
            }
        }
        Commands::AuditAssessmentInputCorrection {
            feature,
            stage,
            review_id,
        } => {
            match assessment_input_correction::audit(&cfg, &feature, &stage, &review_id)
                .and_then(|record| assessment_input_correction::append_audit(&cfg, &record))
            {
                Ok(sha) => {
                    println!("assessment input correction eligibility recorded: {sha}");
                    EXIT_SUCCESS
                }
                Err(error) => {
                    eprintln!("error: assessment input correction audit refused: {error}");
                    EXIT_WRITE
                }
            }
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
