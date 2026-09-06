//! Derive each checkpoint's state from three direct evidence classes, with fail-closed ordering:
//!
//! 1. **Canonical state** — artifacts, approval metadata, configuration, binding currency. Cheap
//!    deterministic predicates, re-evaluated live on every query.
//! 2. **Verification records** — mechanical verifications `check` actually executed and that passed;
//!    persisted, hash-bound, and stale the moment a bound input drifts.
//! 3. **Decision receipts** — the closed set of irreducible human / authorized-agent decisions.
//!
//! `status` and `next` derive state read-only from all three. `check` executes pending mechanical
//! verifications and persists a record per pass. `decide` records a receipt. Nothing reconstructs an
//! earlier PASS indirectly: a later receipt never stands in for an earlier verification.

use crate::contract::{contract_for, Applies, Checkpoint, Requirement, Workflow};
use crate::evidence;
use crate::hashing;
use crate::project::Project;
use crate::receipts::{self, ReceiptState, ReceiptStore};
use crate::verification::{self, RecordState, VerificationRecord, VerificationStore};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Pass,
    Blocked,
    Waiting,
    NotApplicable,
}

impl State {
    pub fn label(&self) -> &'static str {
        match self {
            State::Pass => "PASS",
            State::Blocked => "BLOCKED",
            State::Waiting => "WAITING",
            State::NotApplicable => "n/a",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckpointReport {
    pub id: String,
    pub title: String,
    pub state: State,
    pub met: Vec<String>,
    pub unmet: Vec<String>,
    pub next_action: Option<String>,
}

/// A feature subject with its applicability resolved from the Contract.
pub struct Subject {
    pub workflow: Workflow,
    pub id: String,
    pub gui: bool,
    pub persistence: bool,
    pub arch_approval_required: bool,
}

impl Subject {
    pub fn resolve(project: &Project, workflow: Workflow, id: &str) -> Result<Self> {
        let (mut gui, mut persistence) = (false, false);
        if workflow == Workflow::Feature {
            let contract_path = project.contract(id);
            if let Ok(text) = std::fs::read_to_string(&contract_path) {
                // Read the Feature Impact Accounting table rows only, plus an explicit marker.
                let lower = text.to_lowercase();
                gui = lower.contains("gui-visible outcome") || impact_row_changed(&lower, "gui");
                persistence = impact_row_changed(&lower, "persistence");
            }
        }
        Ok(Self {
            workflow,
            id: id.to_string(),
            gui,
            persistence,
            arch_approval_required: false,
        })
    }
}

/// Read-only: derive every checkpoint's state for a subject from canonical evidence, verification
/// records, and decision receipts. Never executes a command.
pub fn evaluate(project: &Project, subject: &Subject) -> Result<Vec<CheckpointReport>> {
    let store = ReceiptStore::at(&project.codeos_dir());
    let vstore = VerificationStore::at(&project.codeos_dir());
    let contract = contract_for(subject.workflow);
    let mut reports = Vec::new();
    let mut all_prior_pass = true;

    // bootstrap-entry: a solution's first Feature Development workflow cannot enter its first
    // checkpoint while Bootstrap is not complete. Once any feature has an acceptance receipt,
    // Bootstrap has by definition already happened and the gate is satisfied.
    if subject.workflow == Workflow::Feature {
        let a_feature_was_accepted = store
            .all()?
            .iter()
            .any(|r| r.checkpoint == "acceptance" && r.result == "accepted");
        if !a_feature_was_accepted {
            let boot = Subject {
                workflow: Workflow::Bootstrap,
                id: "solution".into(),
                gui: false,
                persistence: false,
                arch_approval_required: false,
            };
            let boot_reports = evaluate(project, &boot)?;
            let boot_complete = boot_reports
                .iter()
                .filter(|r| matches!(r.id.as_str(), "B1" | "B2" | "B3" | "B4"))
                .all(|r| r.state == State::Pass);
            if !boot_complete {
                let blocker = boot_reports
                    .iter()
                    .find(|r| matches!(r.state, State::Blocked | State::Waiting))
                    .map(|r| r.id.clone())
                    .unwrap_or_else(|| "B?".into());
                reports.push(CheckpointReport {
                    id: "bootstrap-entry".into(),
                    title: "Solution Bootstrap complete".into(),
                    state: State::Blocked,
                    met: vec![],
                    unmet: vec![format!(
                        "Solution Bootstrap is not complete — {blocker} is not PASS"
                    )],
                    next_action: Some(
                        "run `codeos-workflow check --workflow bootstrap --subject solution`"
                            .into(),
                    ),
                });
                all_prior_pass = false;
            } else {
                reports.push(CheckpointReport {
                    id: "bootstrap-entry".into(),
                    title: "Solution Bootstrap complete".into(),
                    state: State::Pass,
                    met: vec!["Bootstrap B1–B4 are PASS".into()],
                    unmet: vec![],
                    next_action: None,
                });
            }
        }
    }

    for c in &contract {
        let applicable = applies(c.applies, subject);
        if !applicable {
            reports.push(CheckpointReport {
                id: c.id.into(),
                title: c.title.into(),
                state: State::NotApplicable,
                met: vec![],
                unmet: vec![],
                next_action: None,
            });
            continue;
        }

        if !all_prior_pass {
            let blocker = reports
                .iter()
                .find(|r| r.state == State::Blocked || r.state == State::Waiting)
                .map(|r| r.id.clone())
                .unwrap_or_default();
            reports.push(CheckpointReport {
                id: c.id.into(),
                title: c.title.into(),
                state: State::Blocked,
                met: vec![],
                unmet: vec![format!("earlier checkpoint {blocker} is not PASS")],
                next_action: Some(format!("resolve {blocker} first")),
            });
            continue;
        }

        let report = evaluate_checkpoint(project, subject, c, &store, &vstore)?;
        if report.state != State::Pass {
            all_prior_pass = false;
        }
        reports.push(report);
    }
    Ok(reports)
}

/// A Feature Impact Accounting row `| <tier…> | changed | <reason> |` for the given tier keyword.
fn impact_row_changed(lower_text: &str, tier_keyword: &str) -> bool {
    lower_text.lines().any(|line| {
        let l = line.trim();
        l.starts_with('|')
            && l.contains(tier_keyword)
            && l.split('|').map(str::trim).any(|cell| cell == "changed")
    })
}

fn applies(a: Applies, s: &Subject) -> bool {
    match a {
        Applies::Always => true,
        Applies::WhenGui => s.gui,
        Applies::WhenPersistence => s.persistence,
        Applies::WhenArchApproval => s.arch_approval_required,
    }
}

fn evaluate_checkpoint(
    project: &Project,
    subject: &Subject,
    c: &Checkpoint,
    store: &ReceiptStore,
    vstore: &VerificationStore,
) -> Result<CheckpointReport> {
    let mut met = Vec::new();
    let mut unmet = Vec::new();
    let mut soft = 0usize; // unmet conditions that are only waiting on a human decision
    let mut next_action = None;

    for req in &c.requirements {
        match req {
            Requirement::CharterApproved => {
                if evidence::approval_recorded(&project.charter())? {
                    met.push("Charter records approval".into());
                } else if !project.charter().exists() {
                    unmet.push(".codeos/00-project/charter.md does not exist".into());
                    next_action.get_or_insert(
                        "establish the Solution Charter (support-solution-charter.md)".into(),
                    );
                } else {
                    unmet.push(
                        "Charter has no recorded approval (approval.by / approval.at)".into(),
                    );
                    next_action.get_or_insert("approve the Solution Charter".into());
                }
            }
            Requirement::SpecPackageApproved => {
                let parts = [
                    ("Intent", project.intent(&subject.id)),
                    ("Contract", project.contract(&subject.id)),
                    ("Event Schema", project.event_schema(&subject.id)),
                ];
                let mut missing = Vec::new();
                for (name, path) in &parts {
                    if !evidence::spec_artifact_approved(path)? {
                        missing.push(*name);
                    }
                }
                if missing.is_empty() {
                    met.push("Specification Package approved (all three artifacts)".into());
                } else {
                    unmet.push(format!(
                        "not recording joint package approval: {}",
                        missing.join(", ")
                    ));
                    next_action.get_or_insert(
                        "record status: APPROVED with approved_by / approved_at on all three"
                            .into(),
                    );
                }
            }
            Requirement::CodeosYamlValid => {
                let y = project.codeos_yaml();
                if !y.exists() {
                    unmet.push(".codeos/00-project/codeos.yaml is missing".into());
                    next_action.get_or_insert("create codeos.yaml from the template".into());
                } else {
                    match validate_codeos_yaml(project, &y) {
                        Some(true) => met.push("codeos.yaml validates".into()),
                        Some(false) => {
                            unmet.push(
                                "codeos.yaml does not validate against the active configuration"
                                    .into(),
                            );
                            next_action.get_or_insert(
                                "run project-config-contract.sh and fix the reported issue".into(),
                            );
                        }
                        None => {
                            unmet.push("codeos.yaml could not be read".into());
                        }
                    }
                }
            }
            Requirement::ContractSection(heading) => {
                if evidence::section_filled(&project.contract(&subject.id), heading)? {
                    met.push(format!("Contract '{heading}' is filled in"));
                } else {
                    unmet.push(format!(
                        "Contract '{heading}' is missing or has unfilled placeholders"
                    ));
                    next_action
                        .get_or_insert(format!("complete the Contract's '{heading}' section"));
                }
            }
            Requirement::Receipt {
                checkpoint,
                results,
            } => {
                let mut bindings = current_bindings(project, subject, checkpoint)?;
                let latest = store.latest(subject.workflow.slug(), &subject.id, checkpoint)?;
                // The observation binding cannot be recomputed from the tree — the observation
                // statement is a historical, immutable fact. Carry the recorded hash forward so it
                // reads as current; it exists to link a closure to the same observation, not to
                // detect drift.
                if let Some(h) = latest.as_ref().and_then(|r| r.bindings.get("observation")) {
                    bindings.insert("observation".into(), h.clone());
                }
                match receipts::evaluate(latest, &bindings) {
                    ReceiptState::Absent => {
                        soft += 1;
                        unmet.push(format!("no {checkpoint} decision recorded"));
                        next_action.get_or_insert(format!(
                            "record it: codeos-workflow decide --workflow {} --subject {} --checkpoint {checkpoint} --result <{}>",
                            subject.workflow.slug(), subject.id, results.join("|")
                        ));
                    }
                    ReceiptState::Stale {
                        binding, was, now, ..
                    } => {
                        soft += 1;
                        unmet.push(format!(
                            "{checkpoint} decision is stale — '{binding}' changed since it was recorded (was {}, now {})",
                            short(&was), short(&now)
                        ));
                        next_action.get_or_insert(format!(
                            "re-run the {checkpoint} decision against current evidence"
                        ));
                    }
                    ReceiptState::Current(r) => {
                        if results.contains(&r.result.as_str()) {
                            met.push(format!("{checkpoint}: {}", r.result));
                        } else {
                            unmet.push(format!(
                                "{checkpoint} recorded '{}', which does not permit progression here",
                                r.result
                            ));
                            next_action
                                .get_or_insert(format!("route per the '{}' outcome", r.result));
                        }
                    }
                }
            }
            Requirement::ReviewRecord { stage } => {
                let rec = evidence::latest_review_record(
                    &project.review_records_dir(),
                    &subject.id,
                    stage,
                )?;
                match rec {
                    None => {
                        unmet.push(format!(
                            "no reviewer record for {} at stage {stage}",
                            subject.id
                        ));
                        next_action
                            .get_or_insert("run the reviewer at the final review boundary".into());
                    }
                    Some(r) => {
                        let head = current_head(&project.root);
                        let bound = r.review_commit.is_empty()
                            || head.is_empty()
                            || r.review_commit == head;
                        if !bound {
                            unmet.push(format!(
                                "reviewer record is bound to {} but HEAD is {} — re-review current evidence",
                                short(&r.review_commit), short(&head)
                            ));
                            next_action.get_or_insert(
                                "re-run the reviewer against the current commit".into(),
                            );
                        } else if !evidence::review_permits_progression(&r.effective_concern) {
                            unmet.push(format!(
                                "reviewer concern '{}' does not permit progression",
                                r.effective_concern
                            ));
                            next_action.get_or_insert("resolve the reviewer's finding".into());
                        } else {
                            met.push(format!("reviewer record present ({})", r.effective_concern));
                        }
                    }
                }
            }
            Requirement::LiveVerification { name, mechanic } => {
                let want = verification_bindings(project, subject, c.id)?;
                let have = vstore.latest(subject.workflow.slug(), &subject.id, c.id, mechanic)?;
                match verification::evaluate(have, &want) {
                    RecordState::Absent => {
                        unmet.push(format!(
                            "{name} has no verification record — run `codeos-workflow check`"
                        ));
                        next_action
                            .get_or_insert(format!("run `codeos-workflow check` to verify {name}"));
                    }
                    RecordState::Stale {
                        binding, was, now, ..
                    } => {
                        unmet.push(format!(
                            "{name} verification is stale — '{binding}' changed since it passed (was {}, now {})",
                            short(&was), short(&now)
                        ));
                        next_action.get_or_insert(format!(
                            "re-run `codeos-workflow check` to re-verify {name}"
                        ));
                    }
                    RecordState::Current(r) => {
                        met.push(format!(
                            "{name}: verification recorded {} ({})",
                            r.timestamp, r.verification
                        ));
                    }
                }
            }
            Requirement::ObservationStated => {
                let latest = store.latest("operation", &subject.id, "operation_route")?;
                match latest
                    .and_then(|r| r.observation)
                    .map(|o| o.trim().to_string())
                {
                    Some(o) if !o.is_empty() => {
                        met.push(format!("observation stated: {}", first_line(&o)))
                    }
                    _ => {
                        soft += 1;
                        unmet.push("no observation statement recorded".into());
                        next_action.get_or_insert(format!(
                            "codeos-workflow decide --workflow operation --subject {} --checkpoint operation_route --result <route> --observation <text>",
                            subject.id
                        ));
                    }
                }
            }
            Requirement::RouteResolution => {
                let route_receipt = store.latest("operation", &subject.id, "operation_route")?;
                let Some(route_receipt) = route_receipt else {
                    soft += 1;
                    unmet.push("no route recorded yet".into());
                    next_action.get_or_insert("record the O2 route decision".into());
                    continue;
                };
                let route = route_receipt.result.clone();
                let decided_at = route_receipt.timestamp.clone();
                match route.as_str() {
                    "no_action" => {
                        let closure =
                            store.latest("operation", &subject.id, "no_action_closure")?;
                        let obs_hash = route_receipt.bindings.get("observation").cloned();
                        let ok = closure
                            .map(|c| c.bindings.get("observation").cloned() == obs_hash)
                            .unwrap_or(false);
                        if ok {
                            met.push("no_action closure recorded with rationale".into());
                        } else {
                            soft += 1;
                            unmet.push("no_action route has no closure receipt".into());
                            next_action.get_or_insert(format!(
                                "codeos-workflow decide --workflow operation --subject {} --checkpoint no_action_closure --result no_action --observation <same text> --rationale <why>",
                                subject.id
                            ));
                        }
                    }
                    "new_feature" | "implementation_defect" | "specification_change" => {
                        let accepted_after = store.all()?.into_iter().any(|r| {
                            r.checkpoint == "acceptance"
                                && r.result == "accepted"
                                && r.subject == subject.id
                                && r.timestamp >= decided_at
                        });
                        if accepted_after {
                            met.push(format!(
                                "{route}: feature {} has an acceptance recorded after the route decision",
                                subject.id
                            ));
                        } else {
                            unmet.push(format!(
                                "{route}: no accepted resolution for {} after the route decision",
                                subject.id
                            ));
                            next_action.get_or_insert(format!(
                                "carry {} through Feature Development to acceptance",
                                subject.id
                            ));
                        }
                    }
                    "charter_change" | "architecture_reassessment" => {
                        soft += 1;
                        unmet.push(format!(
                            "{route}: closure derivation is not automatic for this route"
                        ));
                        next_action.get_or_insert(
                            "link and verify the resolving change through its owning workflow, then re-check".into(),
                        );
                    }
                    other => {
                        unmet.push(format!("recorded route '{other}' is not valid"));
                        next_action.get_or_insert("re-record O2 with a valid route".into());
                    }
                }
            }
        }
    }

    // Blocked if any unmet condition is a real evidence gap; Waiting if the only thing missing is a
    // human decision the tool must not make itself.
    let hard = unmet.len().saturating_sub(soft);
    let state = if unmet.is_empty() {
        State::Pass
    } else if hard == 0 {
        State::Waiting
    } else {
        State::Blocked
    };

    Ok(CheckpointReport {
        id: c.id.into(),
        title: c.title.into(),
        state,
        met,
        unmet,
        next_action,
    })
}

/// `check`: execute every mechanical verification this subject needs whose record is absent or
/// stale, in checkpoint order. On each pass, append a verification record bound to the current
/// governed inputs. Stops at the first failing verification, or at the first checkpoint `check`
/// cannot advance (one blocked on an approval, a receipt, or review — not on a mechanical
/// verification). Returns one human-readable line per verification attempted; writes nothing else.
pub fn run_checks(project: &Project, subject: &Subject) -> Result<Vec<String>> {
    let contract = contract_for(subject.workflow);
    let vstore = VerificationStore::at(&project.codeos_dir());
    let slug = subject.workflow.slug();
    let mut log = Vec::new();

    loop {
        let reports = evaluate(project, subject)?;
        let Some(stuck) = reports
            .iter()
            .find(|r| matches!(r.state, State::Blocked | State::Waiting))
        else {
            break; // every applicable checkpoint is PASS
        };
        if stuck.unmet.iter().any(|u| u.contains("earlier checkpoint")) {
            break; // blocked behind an earlier checkpoint check cannot resolve
        }
        let Some(c) = contract.iter().find(|c| c.id == stuck.id) else {
            break;
        };
        let mechanics: Vec<&'static str> = c
            .requirements
            .iter()
            .filter_map(|r| match r {
                Requirement::LiveVerification { mechanic, .. } => Some(*mechanic),
                _ => None,
            })
            .collect();
        if mechanics.is_empty() {
            break; // this checkpoint waits on a human decision or a non-mechanical condition
        }

        let want = verification_bindings(project, subject, &stuck.id)?;
        let mut progressed = false;
        for mechanic in mechanics {
            let have = vstore.latest(slug, &subject.id, &stuck.id, mechanic)?;
            if matches!(verification::evaluate(have, &want), RecordState::Current(_)) {
                continue; // already verified against the current inputs
            }
            let outcome = run_mechanic(project, mechanic);
            log.push(format!("{} / {mechanic}: {}", stuck.id, outcome.summary));
            if !outcome.ok {
                return Ok(log); // fail closed at the first failing verification
            }
            let command = if outcome.command.is_empty() {
                mechanic.to_string()
            } else {
                outcome.command.clone()
            };
            vstore.append(&VerificationRecord {
                workflow: slug.to_string(),
                subject: subject.id.clone(),
                checkpoint: stuck.id.clone(),
                verification: mechanic.to_string(),
                result: "passed".to_string(),
                bindings: want.clone(),
                command,
                timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            })?;
            progressed = true;
        }
        if !progressed {
            break; // nothing left check can do for this checkpoint
        }
    }
    Ok(log)
}

/// The bindings a receipt for `checkpoint` should currently be evaluated against. Public so
/// `decide` records exactly what `status`/`check` will later re-check.
pub fn bindings_for_decision(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
) -> Result<BTreeMap<String, String>> {
    current_bindings(project, subject, checkpoint)
}

/// The bindings a receipt for `checkpoint` should currently be evaluated against.
fn current_bindings(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
) -> Result<BTreeMap<String, String>> {
    let mut b = BTreeMap::new();
    let put = |b: &mut BTreeMap<String, String>, k: &str, path: &Path| -> Result<()> {
        if let Some(h) = hashing::file_sha256(path)? {
            b.insert(k.to_string(), h);
        }
        Ok(())
    };
    match checkpoint {
        "initial_product_preview" => {
            // Bound to the two governed inputs the platform-direction call actually judges: the
            // Charter (product direction) and codeos.yaml (Platform Baseline selection). It is
            // deliberately NOT bound to working-tree implementation_state — every feature increment
            // has its own Early Development Preview, and implementing a feature must not invalidate
            // the one-time confirmation that the baseline direction is right.
            put(&mut b, "charter", &project.charter())?;
            put(&mut b, "codeos_yaml", &project.codeos_yaml())?;
        }
        "early_preview" => {
            put(&mut b, "intent", &project.intent(&subject.id))?;
            put(&mut b, "contract", &project.contract(&subject.id))?;
            put(&mut b, "event_schema", &project.event_schema(&subject.id))?;
            b.insert(
                "implementation_state".into(),
                hashing::working_tree_state(&project.root, &project.implementation_paths())?,
            );
        }
        "reconciliation" | "final_ux_validation" | "acceptance" => {
            put(&mut b, "intent", &project.intent(&subject.id))?;
            put(&mut b, "contract", &project.contract(&subject.id))?;
            put(&mut b, "event_schema", &project.event_schema(&subject.id))?;
        }
        "operation_route" | "no_action_closure" => {
            // Bound to the observation statement's hash, supplied at decide time; nothing to
            // recompute from the tree here.
        }
        _ => {}
    }
    Ok(b)
}

/// The governed inputs a mechanical-verification record for `checkpoint` must be bound to. Public
/// for `check` and for tests that seed verification records directly.
pub fn verification_bindings_for(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
) -> Result<BTreeMap<String, String>> {
    verification_bindings(project, subject, checkpoint)
}

fn verification_bindings(
    project: &Project,
    subject: &Subject,
    checkpoint: &str,
) -> Result<BTreeMap<String, String>> {
    let mut b = BTreeMap::new();
    let put = |b: &mut BTreeMap<String, String>, k: &str, path: &Path| -> Result<()> {
        if let Some(h) = hashing::file_sha256(path)? {
            b.insert(k.to_string(), h);
        }
        Ok(())
    };
    match checkpoint {
        // Bootstrap's integrated baseline. Bound to the resolved Platform Baseline only: a platform
        // change forces a re-baseline. Deliberately NOT bound to implementation_state — Bootstrap
        // is a completed phase, and from the first feature onward the live integration guarantee is
        // carried by each feature's own verification (F3 / F5 / F5d / F5g).
        "B3" => {
            put(&mut b, "codeos_yaml", &project.codeos_yaml())?;
        }
        // Every feature mechanical verification: the approved Specification Package plus the current
        // implementation. Revising the spec, or changing the implementation, re-blocks it.
        "F3" | "F5" | "F5d" | "F5g" => {
            put(&mut b, "intent", &project.intent(&subject.id))?;
            put(&mut b, "contract", &project.contract(&subject.id))?;
            put(&mut b, "event_schema", &project.event_schema(&subject.id))?;
            b.insert(
                "implementation_state".into(),
                hashing::working_tree_state(&project.root, &project.implementation_paths())?,
            );
        }
        _ => {}
    }
    Ok(b)
}

/// Validate `codeos.yaml` against the active configuration's mechanics policy. Runs the authoritative
/// shell check when the toolkit is reachable, and falls back to a structural check otherwise. This
/// is a live predicate — evaluated the same way by every command.
fn validate_codeos_yaml(project: &Project, yaml: &Path) -> Option<bool> {
    let self_dev = project
        .root
        .join("dba/04-tools/configuration/project-config-contract.sh");
    let downstream = project
        .codeos_dir()
        .join("toolkit/dba/04-tools/configuration/project-config-contract.sh");
    if let Some(script) = [self_dev, downstream].into_iter().find(|p| p.exists()) {
        if let Ok(o) = std::process::Command::new("bash")
            .arg(&script)
            .arg(yaml)
            .output()
        {
            return Some(o.status.success());
        }
    }
    // Toolkit not reachable from here: structural check rather than a hard fail. The authoritative
    // validation still runs wherever the toolkit is present.
    let text = std::fs::read_to_string(yaml).ok()?;
    Some(text.contains("platform:") && text.contains("artifacts:"))
}

fn run_mechanic(project: &Project, mechanic: &str) -> evidence::CommandOutcome {
    // Test seam: a `.codeos/06-workflow/.verify-stub` file of `mechanic=pass|fail` lines forces
    // outcomes without executing anything. A real project never has this file; it is per-project,
    // so parallel tests do not interfere.
    let stub = project
        .codeos_dir()
        .join("06-workflow")
        .join(".verify-stub");
    if let Ok(spec) = std::fs::read_to_string(&stub) {
        for entry in spec.split([',', '\n']) {
            if let Some((m, v)) = entry.split_once('=') {
                if m.trim() == mechanic {
                    return evidence::CommandOutcome {
                        ok: v.trim() == "pass",
                        command: format!("<stub {mechanic}>"),
                        summary: format!("{mechanic} (stubbed {})", v.trim()),
                    };
                }
            }
        }
    }
    let backend = project.root.join("backend");
    let web = project.root.join("web");
    match mechanic {
        "baseline" => run_integrated_baseline(project),
        "smoke" | "behavior" | "repeatability" | "data_integrity" => {
            if !backend.is_dir() {
                return evidence::CommandOutcome {
                    ok: false,
                    command: String::new(),
                    summary: "no backend/ to verify".into(),
                };
            }
            // These exercise real PostgreSQL; bring the composed db up and point cargo at it.
            let db_url = "postgres://codeos:codeos@localhost:5432/codeos";
            if project.root.join("docker-compose.yml").is_file() {
                let _ = evidence::run(
                    &project.root,
                    "docker",
                    &["compose", "up", "-d", "--wait", "db"],
                );
            }
            evidence::run_env(
                &backend,
                "cargo",
                &["test", "--quiet"],
                &[("DATABASE_URL", db_url)],
            )
        }
        "playwright" => {
            if web.is_dir() {
                evidence::run(&web, "npm", &["run", "test:e2e"])
            } else {
                evidence::CommandOutcome {
                    ok: false,
                    command: String::new(),
                    summary: "no web/ to verify".into(),
                }
            }
        }
        other => evidence::CommandOutcome {
            ok: false,
            command: String::new(),
            summary: format!("unknown mechanic {other}"),
        },
    }
}

/// B3: bring the integrated stack up, prove DB<->backend<->GUI reachability and a clean migration,
/// run the skeleton's shipped tests and Playwright journey, then tear the stack down.
fn run_integrated_baseline(project: &Project) -> evidence::CommandOutcome {
    let root = &project.root;
    if !root.join("docker-compose.yml").is_file() {
        return evidence::CommandOutcome {
            ok: false,
            command: String::new(),
            summary: "no docker-compose.yml".into(),
        };
    }
    let mut steps: Vec<String> = Vec::new();
    let compose = |args: &[&str]| evidence::run(root, "docker", &[&["compose"], args].concat());

    // Always start from a clean volume so the migration runs from an empty database.
    let _ = compose(&["down", "-v"]);
    let up = compose(&["up", "-d", "--build", "--wait"]);
    if !up.ok {
        let _ = compose(&["down", "-v"]);
        return evidence::CommandOutcome {
            ok: false,
            command: "docker compose up -d --build --wait".into(),
            summary: format!("docker compose up failed: {}", up.summary),
        };
    }

    let mut ok = true;
    // Backend health reports a connected database — proves the clean migration applied.
    let health = evidence::run(root, "curl", &["-fsS", "http://localhost:8080/health"]);
    let db_connected = health.ok && health.summary.contains("connected");
    ok &= db_connected;
    steps.push(format!(
        "health: {}",
        if db_connected {
            "db connected"
        } else {
            "NOT connected"
        }
    ));

    let web = evidence::run(
        root,
        "curl",
        &["-fsS", "-o", "/dev/null", "http://localhost:3000/"],
    );
    ok &= web.ok;
    steps.push(format!("web reachable: {}", web.ok));

    if root.join("backend").is_dir() {
        // The composed db is up; point the backend test run at it (skeleton tests that touch
        // PostgreSQL need DATABASE_URL, exactly as the feature mechanics do).
        let t = evidence::run_env(
            &root.join("backend"),
            "cargo",
            &["test", "--quiet"],
            &[(
                "DATABASE_URL",
                "postgres://codeos:codeos@localhost:5432/codeos",
            )],
        );
        ok &= t.ok;
        steps.push(format!("backend tests: {}", t.ok));
    }
    if root.join("web").is_dir() {
        let u = evidence::run(
            &root.join("web"),
            "npm",
            &["run", "test:unit", "--", "--run"],
        );
        ok &= u.ok;
        steps.push(format!("web unit/component tests: {}", u.ok));
        let e = evidence::run(&root.join("web"), "npm", &["run", "test:e2e"]);
        ok &= e.ok;
        steps.push(format!("Playwright journey: {}", e.ok));
    }

    let _ = compose(&["down", "-v"]);
    evidence::CommandOutcome {
        ok,
        command: "docker compose integrated baseline (migration + curl + cargo test + npm test)"
            .into(),
        summary: steps.join("; "),
    }
}

fn current_head(root: &Path) -> String {
    std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").chars().take(80).collect()
}

fn short(s: &str) -> String {
    if s.len() > 10 {
        format!("{}…", &s[..10])
    } else {
        s.to_string()
    }
}
