use anyhow::Result;
use clap::{Parser, Subcommand};
use codeos_workflow::checker::{self, Subject};
use codeos_workflow::contract::Workflow;
use codeos_workflow::project::Project;
use codeos_workflow::receipts::{Receipt, ReceiptStore, RECEIPT_CHECKPOINTS};
use codeos_workflow::{report, EXIT_ERROR, EXIT_SUCCESS, EXIT_USAGE};
use std::path::PathBuf;
use std::process;

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
    /// Report every checkpoint's state for a subject. Read-only; never executes a test.
    Status(Target),
    /// Report the first blocked or waiting checkpoint and its next action. Read-only.
    Next(Target),
    /// Execute the pending mechanical verifications for a subject and persist a record per pass,
    /// then report state. The only command that runs a verification.
    Check(Target),
    /// Append a decision receipt for one of the seven receipt-bearing checkpoints.
    Decide(DecideArgs),
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
}

fn main() {
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
        Cmd::Status(t) | Cmd::Next(t) | Cmd::Check(t) => {
            let Some(workflow) = Workflow::parse(&t.workflow) else {
                eprintln!("workflow must be bootstrap | feature | operation");
                return Ok(EXIT_USAGE);
            };
            let project = Project::discover(start)?;
            let subject = Subject::resolve(&project, workflow, &t.subject)?;
            if matches!(cli.command, Cmd::Check(_)) {
                let log = checker::run_checks(&project, &subject)?;
                for line in &log {
                    println!("{line}");
                }
                if !log.is_empty() {
                    println!();
                }
            }
            let reports = checker::evaluate(&project, &subject)?;
            match &cli.command {
                Cmd::Next(_) => println!("{}", report::render_next(&reports)),
                _ => println!(
                    "{}",
                    report::render_status(workflow.slug(), &t.subject, &reports)
                ),
            }
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
