use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use codeos_workflow::candidate;
use codeos_workflow::checker::{self, Subject};
use codeos_workflow::contract::Workflow;
use codeos_workflow::event_evidence::{self, VerifyEventArgs};
use codeos_workflow::observations::{ClosureEvidence, ObservationStore};
use codeos_workflow::project::Project;
use codeos_workflow::receipts::{Receipt, ReceiptStore, RECEIPT_CHECKPOINTS};
use codeos_workflow::revalidation;
use codeos_workflow::{monitor, report, EXIT_BLOCKED, EXIT_ERROR, EXIT_SUCCESS, EXIT_USAGE};
use std::path::PathBuf;
use std::process;

mod release_identity;

#[derive(Parser)]
#[command(name = "codeos-workflow", version)]
#[command(about = "Mechanical checkpoint governance for the three DBA workflows")]
struct Cli {
    /// Project directory to resolve from (default: current directory).
    #[arg(long, global = true, value_name = "DIR")]
    project: Option<PathBuf>,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Record one immutable post-acceptance observation and issue its OBS identity.
    Observe(ObserveArgs),
    /// Route one Observation v2 identity to an explicit target.
    RouteObservation(RouteObservationArgs),
    /// Close one routed Observation v2 identity with validated evidence.
    CloseObservation(CloseObservationArgs),
    /// Run the project-owned qualification protocol without adding a lifecycle checkpoint.
    QualifyProject(QualifyProjectArgs),
    /// Report every checkpoint's state for a subject. Read-only; never executes a test.
    Status(StatusArgs),
    /// Report the first blocked or waiting checkpoint and its next action. Read-only.
    Next(Target),
    /// Execute the pending mechanical verifications for a subject and persist a record per pass,
    /// then report state. The only command that runs a verification.
    Check(CheckArgs),
    /// Append a decision receipt for one of the seven receipt-bearing checkpoints.
    Decide(DecideArgs),
    /// Show the primary journey, accepted/candidate frontiers, and first broken transition.
    Journey(JourneyArgs),
    /// Execute one verification command and canonically record its exact appended runtime events.
    VerifyEvents(VerifyEventsCli),
    /// Build the bounded packet for post-acceptance dependency revalidation.
    RevalidationPlan(RevalidationPlanArgs),
    /// Record one independent post-acceptance revalidation assessment.
    RevalidationRecord(RevalidationRecordArgs),
    /// Materialize the immutable, content-addressed candidate used by F7 through F9.
    PrepareCandidate(PrepareCandidateArgs),
    /// Report deterministic feature impact for an architecture proposal. Read-only.
    ArchitectureImpact(ArchitectureImpactArgs),
}

#[derive(clap::Args)]
struct ObserveArgs {
    /// Observation text, or @path to read it from a file.
    #[arg(long)]
    statement: String,
    /// Source or evidence reference. May be repeated.
    #[arg(long = "source")]
    sources: Vec<String>,
}

#[derive(clap::Args)]
struct RouteObservationArgs {
    #[arg(long)]
    observation: String,
    #[arg(long)]
    route: String,
    #[arg(long)]
    target_type: Option<String>,
    #[arg(long)]
    target_id: Option<String>,
}

#[derive(clap::Args)]
struct CloseObservationArgs {
    #[arg(long)]
    observation: String,
    /// Evidence in KIND=REFERENCE form. May be repeated.
    #[arg(long = "evidence")]
    evidence: Vec<String>,
    #[arg(long)]
    rationale: Option<String>,
}

#[derive(clap::Args)]
struct QualifyProjectArgs {
    #[arg(long, value_parser = ["acceptance", "deployment"])]
    mode: String,
}

#[derive(clap::Args)]
struct ArchitectureImpactArgs {
    /// Accepted Git tree used as the comparison baseline.
    #[arg(long)]
    base: String,
    /// Proposed Git tree, or WORKTREE for the current filesystem.
    #[arg(long, default_value = "WORKTREE")]
    proposed: String,
}

#[derive(clap::Args)]
struct PrepareCandidateArgs {
    #[arg(long)]
    subject: String,
    /// Add a dynamic dependency that cannot be derived mechanically. May be repeated.
    #[arg(long = "include", value_name = "PROJECT_RELATIVE_PATH")]
    include: Vec<String>,
    /// Build a post-acceptance revalidation candidate rather than an ordinary F7 candidate.
    #[arg(long)]
    revalidation: bool,
    /// Latest commit changing the accepted surface; required with --revalidation.
    #[arg(long, requires = "revalidation")]
    trigger: Option<String>,
}

#[derive(clap::Args)]
struct RevalidationPlanArgs {
    #[arg(long)]
    subject: String,
    /// The latest commit changing the accepted surface; later commits must leave it unchanged.
    #[arg(long)]
    trigger: String,
    #[arg(long)]
    output: PathBuf,
}

#[derive(clap::Args)]
struct RevalidationRecordArgs {
    #[arg(long)]
    subject: String,
    #[arg(long)]
    trigger: String,
    /// Independent JSON assessment containing only `decision` and `summary`.
    #[arg(long)]
    assessment: PathBuf,
}

#[derive(clap::Args)]
struct StatusArgs {
    /// bootstrap | feature | operation; omit together with --subject for a project-wide summary.
    #[arg(long, requires = "subject", conflicts_with = "json")]
    workflow: Option<String>,
    /// The workflow subject; omit together with --workflow for a project-wide summary.
    #[arg(long, requires = "workflow", conflicts_with = "json")]
    subject: Option<String>,
    /// Emit the project-wide status projection as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct VerifyEventsCli {
    #[arg(long)]
    subject: String,
    #[arg(long, value_name = "PROJECT_RELATIVE_JSONL")]
    event_log: String,
    #[arg(long, value_name = "PROJECT_RELATIVE_JQ")]
    validator: String,
    /// Verification command and arguments, following `--`.
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

#[derive(clap::Args)]
struct JourneyArgs {
    /// Restrict candidate reporting to one feature; accepted reachability is always shown.
    #[arg(long)]
    subject: Option<String>,
}

#[derive(clap::Args)]
struct Target {
    /// bootstrap | feature | operation
    #[arg(long)]
    workflow: String,
    /// The subject: a feature id for `feature`, an observation slug for `operation`, `solution` for `bootstrap`.
    #[arg(long)]
    subject: String,
}

#[derive(clap::Args)]
struct CheckArgs {
    #[arg(long)]
    workflow: String,
    #[arg(long)]
    subject: String,
    /// Execute only stale post-acceptance verification without reopening F1-F9.
    #[arg(long)]
    revalidation: bool,
    /// Latest commit changing the accepted surface; required with --revalidation.
    #[arg(long, requires = "revalidation")]
    trigger: Option<String>,
}

#[derive(clap::Args)]
struct DecideArgs {
    #[arg(long)]
    workflow: String,
    #[arg(long)]
    subject: String,
    /// One of: initial_product_preview, early_preview, reconciliation, final_ux_validation, acceptance, operation_route, no_action_closure
    #[arg(long)]
    checkpoint: String,
    /// The recorded outcome (e.g. direction_confirmed, completed, accepted, new_feature, no_action).
    #[arg(long)]
    result: String,
    /// Required for no_action_closure / a no_action result.
    #[arg(long)]
    rationale: Option<String>,
    /// The observation statement (or @path to a file). Required for operation_route / no_action_closure;
    /// its hash becomes the receipt's `observation` binding and the text is stored in the receipt.
    #[arg(long)]
    observation: Option<String>,
    /// Extra bindings as name=hash, repeatable.
    #[arg(long = "bind", value_name = "NAME=HASH")]
    bind: Vec<String>,
    /// Content identity of the production-source evidence explicitly selected by F6.
    #[arg(long, value_name = "SHA256")]
    production_evidence: Option<String>,
}

fn main() {
    if let Err(error) = release_identity::verify_release_binary("codeos-workflow") {
        eprintln!("error: {error:#}");
        process::exit(EXIT_ERROR);
    }
    let cli = Cli::parse();
    let start = cli.project.clone().unwrap_or_else(|| PathBuf::from("."));
    let code = match run(&cli, &start) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e:#}");
            EXIT_ERROR
        }
    };
    process::exit(code);
}

fn run(cli: &Cli, start: &std::path::Path) -> Result<i32> {
    match &cli.command {
        Cmd::Observe(args) => {
            let project = Project::discover(start)?;
            anyhow::ensure!(
                project.operation_workflow_version()? >= 2,
                "Observation identities require Operation workflow v2"
            );
            let statement = if let Some(path) = args.statement.strip_prefix('@') {
                std::fs::read_to_string(path)?
            } else {
                args.statement.clone()
            };
            let item = ObservationStore::at(&project.codeos_dir())
                .observe(statement, args.sources.clone())?;
            println!("{}", item.id);
            Ok(EXIT_SUCCESS)
        }
        Cmd::RouteObservation(args) => {
            let project = Project::discover(start)?;
            anyhow::ensure!(
                project.operation_workflow_version()? >= 2,
                "Observation routing requires Operation workflow v2"
            );
            let item = ObservationStore::at(&project.codeos_dir()).route_observation(
                &project,
                &args.observation,
                &args.route,
                args.target_type.clone(),
                args.target_id.clone(),
            )?;
            println!(
                "routed {} -> {} {}",
                item.observation,
                item.route,
                item.target_id.as_deref().unwrap_or("")
            );
            Ok(EXIT_SUCCESS)
        }
        Cmd::CloseObservation(args) => {
            let project = Project::discover(start)?;
            anyhow::ensure!(
                project.operation_workflow_version()? >= 2,
                "Observation closure requires Operation workflow v2"
            );
            let evidence = args
                .evidence
                .iter()
                .map(|item| {
                    let (kind, reference) = item
                        .split_once('=')
                        .context("--evidence expects KIND=REFERENCE")?;
                    anyhow::ensure!(
                        !kind.is_empty() && !reference.is_empty(),
                        "--evidence expects non-empty KIND=REFERENCE"
                    );
                    Ok(ClosureEvidence {
                        kind: kind.into(),
                        reference: reference.into(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let item = ObservationStore::at(&project.codeos_dir()).close(
                &project,
                &args.observation,
                evidence,
                args.rationale.clone(),
            )?;
            println!("closed {} ({})", item.observation, item.route);
            Ok(EXIT_SUCCESS)
        }
        Cmd::QualifyProject(args) => {
            let project = Project::discover(start)?;
            anyhow::ensure!(
                project.feature_workflow_version()? >= 9,
                "project qualification requires Feature workflow v9"
            );
            let outcome = codeos_workflow::qualification::run(&project, &args.mode);
            println!("{}", outcome.summary);
            if let Some(details) = outcome.details {
                println!("{}", serde_json::to_string_pretty(&details)?);
            }
            Ok(if outcome.ok {
                EXIT_SUCCESS
            } else {
                EXIT_BLOCKED
            })
        }
        Cmd::ArchitectureImpact(args) => {
            let project = Project::discover(start)?;
            let impact =
                codeos_workflow::architecture::impact(&project, &args.base, &args.proposed)?;
            println!("{}", serde_json::to_string_pretty(&impact)?);
            Ok(EXIT_SUCCESS)
        }
        Cmd::PrepareCandidate(args) => {
            let project = Project::discover(start)?;
            let subject = Subject::resolve(&project, Workflow::Feature, &args.subject)?;
            let kind = if args.revalidation {
                "post_acceptance_revalidation"
            } else {
                "ordinary"
            };
            let (id, _) = candidate::prepare(
                &project,
                &subject,
                &args.include,
                kind,
                args.trigger.as_deref(),
            )?;
            println!("candidate bundle: {id}");
            Ok(EXIT_SUCCESS)
        }
        Cmd::RevalidationPlan(args) => {
            let project = Project::discover(start)?;
            anyhow::ensure!(
                project.feature_workflow_version()? < 8,
                "legacy revalidation packets are read-only under DBA-24; use check --revalidation and prepare-candidate --revalidation"
            );
            let case = revalidation::build_case(&project, &args.subject, &args.trigger)?;
            let packet = revalidation::packet(&project, &case)?;
            if let Some(parent) = args.output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&args.output, packet)?;
            println!(
                "POST_ACCEPTANCE_REVALIDATION packet: {}",
                args.output.display()
            );
            Ok(EXIT_SUCCESS)
        }
        Cmd::RevalidationRecord(args) => {
            let project = Project::discover(start)?;
            anyhow::ensure!(
                project.feature_workflow_version()? < 8,
                "legacy revalidation records are read-only under DBA-24; assess the immutable revalidation Candidate Bundle"
            );
            let case = revalidation::build_case(&project, &args.subject, &args.trigger)?;
            let record = revalidation::record(&project, case, &args.assessment)?;
            println!(
                "recorded {} / {} = {} -> {}",
                record.operation,
                record.subject,
                record.decision,
                revalidation::RevalidationStore::at(&project.codeos_dir())
                    .path()
                    .display()
            );
            Ok(if record.decision == "REVALIDATED" {
                EXIT_SUCCESS
            } else {
                EXIT_BLOCKED
            })
        }
        Cmd::VerifyEvents(args) => {
            let project = Project::discover(start)?;
            let subject = Subject::resolve(&project, Workflow::Feature, &args.subject)?;
            let record = event_evidence::verify(
                &project,
                VerifyEventArgs {
                    subject: &subject,
                    event_log: &args.event_log,
                    validator: &args.validator,
                    command: &args.command,
                },
            )?;
            let details = record
                .details
                .as_ref()
                .expect("event evidence always has details");
            println!(
                "runtime-event evidence recorded: {} events, sha256 {}, semantic {}",
                details["event_count"],
                details["event_set_sha256"].as_str().unwrap_or_default(),
                details["semantic_sha256"].as_str().unwrap_or_default()
            );
            Ok(EXIT_SUCCESS)
        }
        Cmd::Journey(args) => {
            let project = Project::discover(start)?;
            println!(
                "{}",
                checker::render_journey(&project, args.subject.as_deref())?
            );
            Ok(EXIT_SUCCESS)
        }
        Cmd::Status(args) => {
            let project = Project::discover(start)?;
            if let (Some(workflow), Some(subject_id)) =
                (args.workflow.as_deref(), args.subject.as_deref())
            {
                let Some(workflow) = Workflow::parse(workflow) else {
                    eprintln!("workflow must be bootstrap | feature | operation");
                    return Ok(EXIT_USAGE);
                };
                if workflow == Workflow::Feature && project.workflow_governance_version()? >= 3 {
                    let status = monitor::ProjectStatus::derive(&project)?;
                    let feature = status
                        .features
                        .iter()
                        .find(|feature| feature.id == subject_id)
                        .with_context(|| format!("unknown feature {subject_id}"))?;
                    println!("{}", monitor::render_targeted_feature(feature));
                    return Ok(
                        if feature.workflow.status == monitor::DisplayStatus::Complete {
                            EXIT_SUCCESS
                        } else {
                            EXIT_BLOCKED
                        },
                    );
                }
                let subject = Subject::resolve(&project, workflow, subject_id)?;
                let reports = checker::evaluate(&project, &subject)?;
                println!(
                    "{}",
                    report::render_status(workflow.slug(), subject_id, &reports)
                );
                Ok(report::overall_exit(&reports))
            } else {
                let status = monitor::ProjectStatus::derive(&project)?;
                if args.json {
                    println!("{}", serde_json::to_string_pretty(&status)?);
                } else {
                    println!("{}", monitor::render(&status));
                }
                Ok(if status.is_blocked() {
                    EXIT_BLOCKED
                } else {
                    EXIT_SUCCESS
                })
            }
        }
        Cmd::Next(t) => {
            let Some(workflow) = Workflow::parse(&t.workflow) else {
                eprintln!("workflow must be bootstrap | feature | operation");
                return Ok(EXIT_USAGE);
            };
            let project = Project::discover(start)?;
            if workflow == Workflow::Feature && project.workflow_governance_version()? >= 3 {
                let status = monitor::ProjectStatus::derive(&project)?;
                let feature = status
                    .features
                    .iter()
                    .find(|feature| feature.id == t.subject)
                    .with_context(|| format!("unknown feature {}", t.subject))?;
                match feature.workflow.next_action.as_deref() {
                    Some(action) => println!(
                        "{}\n{}",
                        feature
                            .workflow
                            .current_checkpoint
                            .as_deref()
                            .unwrap_or("CURRENT VALIDITY"),
                        action
                    ),
                    None => println!("nothing blocked — accepted snapshot is current"),
                }
                return Ok(
                    if feature.workflow.status == monitor::DisplayStatus::Complete {
                        EXIT_SUCCESS
                    } else {
                        EXIT_BLOCKED
                    },
                );
            }
            let subject = Subject::resolve(&project, workflow, &t.subject)?;
            let reports = checker::evaluate(&project, &subject)?;
            println!("{}", report::render_next(&reports));
            Ok(report::overall_exit(&reports))
        }
        Cmd::Check(t) => {
            let Some(workflow) = Workflow::parse(&t.workflow) else {
                eprintln!("workflow must be bootstrap | feature | operation");
                return Ok(EXIT_USAGE);
            };
            let project = Project::discover(start)?;
            let subject = Subject::resolve(&project, workflow, &t.subject)?;
            let log = if t.revalidation {
                checker::run_revalidation_checks(
                    &project,
                    &subject,
                    t.trigger.as_deref().expect("clap requires trigger"),
                )?
            } else {
                checker::run_checks(&project, &subject)?
            };
            for line in &log {
                println!("{line}");
            }
            if !log.is_empty() {
                println!();
            }
            let reports = checker::evaluate(&project, &subject)?;
            println!(
                "{}",
                report::render_status(workflow.slug(), &t.subject, &reports)
            );
            Ok(report::overall_exit(&reports))
        }
        Cmd::Decide(a) => {
            if Workflow::parse(&a.workflow).is_none() {
                eprintln!("workflow must be bootstrap | feature | operation");
                return Ok(EXIT_USAGE);
            }
            if !RECEIPT_CHECKPOINTS.contains(&a.checkpoint.as_str()) {
                eprintln!(
                    "checkpoint must be one of: {}",
                    RECEIPT_CHECKPOINTS.join(", ")
                );
                return Ok(EXIT_USAGE);
            }
            let project = Project::discover(start)?;
            let workflow = Workflow::parse(&a.workflow).unwrap();
            let subject = Subject::resolve(&project, workflow, &a.subject)?;

            // DBA-9 acceptance establishes an advancing feature's proposed frontier. Re-evaluate
            // every preceding checkpoint immediately before appending that receipt so a competing
            // acceptance cannot silently create a branched frontier.
            if workflow == Workflow::Feature
                && project.feature_workflow_version()? >= 2
                && a.checkpoint == "acceptance"
                && a.result == "accepted"
            {
                let reports = checker::evaluate(&project, &subject)?;
                let unresolved: Vec<_> = reports
                    .iter()
                    .take_while(|report| report.id != "F9")
                    .filter(|report| {
                        !matches!(
                            report.state,
                            codeos_workflow::checker::State::Pass
                                | codeos_workflow::checker::State::NotApplicable
                        )
                    })
                    .map(|report| report.id.as_str())
                    .collect();
                if !unresolved.is_empty() {
                    anyhow::bail!(
                        "cannot record DBA-9 acceptance while earlier checkpoints are unresolved: {}",
                        unresolved.join(", ")
                    );
                }
            }

            // An observation-bearing checkpoint must carry the observation statement; its hash is
            // the durable binding, since an Operational Observation has no other durable home.
            let needs_observation = matches!(
                a.checkpoint.as_str(),
                "operation_route" | "no_action_closure"
            );
            let observation_text = match &a.observation {
                Some(v) if v.starts_with('@') => Some(std::fs::read_to_string(&v[1..])?),
                Some(v) => Some(v.clone()),
                None => None,
            };
            if needs_observation
                && observation_text
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or("")
                    .is_empty()
            {
                eprintln!(
                    "checkpoint '{}' requires --observation <text|@file>",
                    a.checkpoint
                );
                return Ok(EXIT_USAGE);
            }

            // Compute the bindings the checker will later evaluate this receipt against, plus any
            // explicit --bind pairs.
            let mut bindings = checker::bindings_for_decision(&project, &subject, &a.checkpoint)?;
            if a.checkpoint == "reconciliation" && !subject.reality_claims.is_empty() {
                let Some(identity) = a.production_evidence.as_deref() else {
                    eprintln!("reconciliation for reality-dependent claims requires --production-evidence <sha256>");
                    return Ok(EXIT_USAGE);
                };
                checker::validate_production_evidence_selection(&project, &subject, identity)?;
                bindings.insert("production_evidence".into(), identity.to_string());
            } else if a.production_evidence.is_some() {
                eprintln!("--production-evidence is valid only for reconciliation");
                return Ok(EXIT_USAGE);
            }
            if let Some(text) = &observation_text {
                bindings.insert(
                    "observation".into(),
                    codeos_workflow::hashing::text_sha256(text),
                );
            }
            for pair in &a.bind {
                let Some((k, v)) = pair.split_once('=') else {
                    eprintln!("--bind expects NAME=HASH, got '{pair}'");
                    return Ok(EXIT_USAGE);
                };
                bindings.insert(k.to_string(), v.to_string());
            }

            let receipt = Receipt {
                workflow: workflow.slug().to_string(),
                subject: a.subject.clone(),
                checkpoint: a.checkpoint.clone(),
                result: a.result.clone(),
                bindings,
                timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                rationale: a.rationale.clone(),
                observation: observation_text,
            };
            let store = ReceiptStore::at(&project.codeos_dir());
            store.append(&receipt)?;
            println!(
                "recorded {} / {} / {} = {} ({} bindings) -> {}",
                receipt.workflow,
                receipt.subject,
                receipt.checkpoint,
                receipt.result,
                receipt.bindings.len(),
                store.path().display()
            );
            Ok(EXIT_SUCCESS)
        }
    }
}
