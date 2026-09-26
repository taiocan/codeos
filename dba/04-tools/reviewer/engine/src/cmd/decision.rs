use crate::config::Config;
use crate::log::{self as review_log, DecisionProvenance};
use anyhow::Result;
use std::path::Path;

const VALID_DECISIONS: &[&str] = &[
    "APPROVE_STAGE",
    "REQUEST_CHANGES",
    "STOP",
    "AUTHORIZE_REENTRY",
    "AUTHORIZE_EVIDENCE_CLOSURE",
    "AUTHORIZE_INFRASTRUCTURE_REVALIDATION",
    "AUTHORIZE_TERMINAL_REVIEW_CLOSURE",
];

pub fn run(
    feature: &str,
    stage: &str,
    decision: &str,
    reason: &str,
    after_review: Option<&str>,
    findings: &[String],
    event_evidence_sha: Option<&str>,
    closure_scope_path: Option<&Path>,
    override_reason: Option<&str>,
    cfg: &Config,
) -> Result<i32> {
    if !VALID_DECISIONS.contains(&decision) {
        eprintln!(
            "decision must be APPROVE_STAGE | REQUEST_CHANGES | STOP | AUTHORIZE_REENTRY | AUTHORIZE_EVIDENCE_CLOSURE | AUTHORIZE_INFRASTRUCTURE_REVALIDATION | AUTHORIZE_TERMINAL_REVIEW_CLOSURE"
        );
        return Ok(crate::EXIT_USAGE);
    }

    let is_reentry_authorization = decision == "AUTHORIZE_REENTRY";
    let is_closure_authorization = decision == "AUTHORIZE_EVIDENCE_CLOSURE";
    let is_infrastructure_authorization = decision == "AUTHORIZE_INFRASTRUCTURE_REVALIDATION";
    let is_terminal_closure_authorization = decision == "AUTHORIZE_TERMINAL_REVIEW_CLOSURE";
    let mut terminal_scope = None;
    if is_reentry_authorization {
        if reason.trim().is_empty() {
            eprintln!("error: AUTHORIZE_REENTRY requires a non-empty reason and scope");
            return Ok(crate::EXIT_USAGE);
        }
        let Some(review_id) = after_review else {
            eprintln!("error: AUTHORIZE_REENTRY requires --after-review <R3-review-id>");
            return Ok(crate::EXIT_USAGE);
        };
        let semantics = match crate::config::resolve_reviewer_tool_contract(cfg) {
            Ok(contract)
                if matches!(
                    contract.as_str(),
                    "dba/04-tools/reviewer/contract/v6.md"
                        | "dba/04-tools/reviewer/contract/v7.md"
                        | "dba/04-tools/reviewer/contract/v8.md"
                        | "dba/04-tools/reviewer/contract/v9.md"
                        | "dba/04-tools/reviewer/contract/v10.md"
                ) =>
            {
                review_log::ReentryDecisionSemantics::CanonicalLogOrder
            }
            Ok(contract)
                if matches!(
                    contract.as_str(),
                    "dba/04-tools/reviewer/contract/v11.md"
                        | "dba/04-tools/reviewer/contract/v12.md"
                        | "dba/04-tools/reviewer/contract/v13.md"
                ) =>
            {
                let current_semantic_revision = if stage == "8" {
                    match super::review::approved_stage8_cycle(cfg, feature) {
                        Ok(revision) => revision,
                        Err(error) => {
                            eprintln!("error: could not resolve the approved Stage 8 review cycle: {error}");
                            return Ok(crate::EXIT_CONFIG);
                        }
                    }
                } else {
                    None
                };
                review_log::ReentryDecisionSemantics::PackageBoundCanonicalLogOrder {
                    current_semantic_revision,
                }
            }
            Ok(contract) => {
                eprintln!(
                    "error: AUTHORIZE_REENTRY is not supported by the selected reviewer contract: {contract}"
                );
                return Ok(crate::EXIT_USAGE);
            }
            Err(error) => {
                eprintln!("error: could not resolve Review re-entry semantics: {error}");
                return Ok(crate::EXIT_CONFIG);
            }
        };
        if let Err(error) = review_log::validate_reentry_authorization_target_with_semantics(
            &cfg.review_log,
            feature,
            stage,
            review_id,
            semantics,
        ) {
            eprintln!("error: AUTHORIZE_REENTRY refused: {error}");
            return Ok(crate::EXIT_USAGE);
        }
    } else if is_closure_authorization {
        if reason.trim().is_empty()
            || findings.is_empty()
            || findings.iter().any(|f| f.trim().is_empty())
        {
            eprintln!(
                "error: AUTHORIZE_EVIDENCE_CLOSURE requires a non-empty reason and --findings"
            );
            return Ok(crate::EXIT_USAGE);
        }
        let Some(review_id) = after_review else {
            eprintln!(
                "error: AUTHORIZE_EVIDENCE_CLOSURE requires --after-review <REENTRY-1-review-id>"
            );
            return Ok(crate::EXIT_USAGE);
        };
        match crate::config::resolve_reviewer_tool_contract(cfg) {
            Ok(contract)
                if matches!(
                    contract.as_str(),
                    "dba/04-tools/reviewer/contract/v7.md" | "dba/04-tools/reviewer/contract/v8.md"
                ) => {}
            Ok(contract) => {
                eprintln!("error: AUTHORIZE_EVIDENCE_CLOSURE is not supported by the selected reviewer contract: {contract}");
                return Ok(crate::EXIT_USAGE);
            }
            Err(error) => {
                eprintln!("error: could not resolve Evidence Closure semantics: {error}");
                return Ok(crate::EXIT_CONFIG);
            }
        }
        if let Err(error) =
            review_log::validate_evidence_closure_target(&cfg.review_log, feature, stage, review_id)
        {
            eprintln!("error: AUTHORIZE_EVIDENCE_CLOSURE refused: {error}");
            return Ok(crate::EXIT_USAGE);
        }
    } else if is_infrastructure_authorization {
        let Some(review_id) = after_review else {
            eprintln!("error: AUTHORIZE_INFRASTRUCTURE_REVALIDATION requires --after-review");
            return Ok(crate::EXIT_USAGE);
        };
        let Some(evidence_sha) = event_evidence_sha else {
            eprintln!("error: AUTHORIZE_INFRASTRUCTURE_REVALIDATION requires --event-evidence-sha");
            return Ok(crate::EXIT_USAGE);
        };
        if reason.trim().is_empty()
            || findings.is_empty()
            || findings.iter().any(|f| f.trim().is_empty())
        {
            eprintln!(
                "error: AUTHORIZE_INFRASTRUCTURE_REVALIDATION requires a reason and --findings"
            );
            return Ok(crate::EXIT_USAGE);
        }
        match crate::config::resolve_reviewer_tool_contract(cfg) {
            Ok(contract) if contract == "dba/04-tools/reviewer/contract/v8.md" => {}
            Ok(contract) => {
                eprintln!(
                    "error: Infrastructure Repair Revalidation is not supported by {contract}"
                );
                return Ok(crate::EXIT_USAGE);
            }
            Err(error) => {
                eprintln!("error: could not resolve Infrastructure Repair Revalidation semantics: {error}");
                return Ok(crate::EXIT_CONFIG);
            }
        }
        if let Err(error) = review_log::validate_infrastructure_revalidation_target(
            &cfg.review_log,
            &cfg.repo_root,
            feature,
            stage,
            review_id,
            evidence_sha,
        ) {
            eprintln!("error: AUTHORIZE_INFRASTRUCTURE_REVALIDATION refused: {error}");
            return Ok(crate::EXIT_USAGE);
        }
    } else if is_terminal_closure_authorization {
        let Some(review_id) = after_review else {
            eprintln!("error: AUTHORIZE_TERMINAL_REVIEW_CLOSURE requires --after-review");
            return Ok(crate::EXIT_USAGE);
        };
        let Some(scope_path) = closure_scope_path else {
            eprintln!("error: AUTHORIZE_TERMINAL_REVIEW_CLOSURE requires --closure-scope");
            return Ok(crate::EXIT_USAGE);
        };
        if reason.trim().is_empty() {
            eprintln!("error: AUTHORIZE_TERMINAL_REVIEW_CLOSURE requires a bounded reason");
            return Ok(crate::EXIT_USAGE);
        }
        match crate::config::resolve_reviewer_tool_contract(cfg) {
            Ok(contract)
                if matches!(
                    contract.as_str(),
                    "dba/04-tools/reviewer/contract/v12.md"
                        | "dba/04-tools/reviewer/contract/v13.md"
                ) => {}
            Ok(contract) => {
                eprintln!("error: Terminal Review Closure is not supported by {contract}");
                return Ok(crate::EXIT_USAGE);
            }
            Err(error) => {
                eprintln!("error: could not resolve Terminal Review Closure semantics: {error}");
                return Ok(crate::EXIT_CONFIG);
            }
        }
        let current_revision = match super::review::approved_stage8_cycle(cfg, feature) {
            Ok(revision) => revision,
            Err(error) => {
                eprintln!("error: could not resolve the approved Stage 8 review cycle: {error}");
                return Ok(crate::EXIT_CONFIG);
            }
        };
        match crate::terminal_closure::load_and_validate_scope(
            &cfg.review_log,
            feature,
            stage,
            review_id,
            current_revision,
            scope_path,
        ) {
            Ok(scope) => terminal_scope = Some(scope),
            Err(error) => {
                eprintln!("error: Terminal Review Closure authorization refused: {error}");
                return Ok(crate::EXIT_USAGE);
            }
        }
    } else if after_review.is_some() {
        eprintln!("error: --after-review is valid only with AUTHORIZE_REENTRY, AUTHORIZE_EVIDENCE_CLOSURE, AUTHORIZE_INFRASTRUCTURE_REVALIDATION, or AUTHORIZE_TERMINAL_REVIEW_CLOSURE");
        return Ok(crate::EXIT_USAGE);
    } else if !findings.is_empty() || event_evidence_sha.is_some() || closure_scope_path.is_some() {
        eprintln!("error: --findings and --event-evidence-sha are valid only with their bounded authorization");
        return Ok(crate::EXIT_USAGE);
    }

    // Load provenance from the most recent assessment for this feature+stage.
    //   Ok(None)     → no assessment exists → legacy path (AC-6)
    //   Ok(Some(.))  → assessment parsed → full provenance check
    //   Err(..)      → assessment exists but is unreadable/unparseable → fail-closed
    let provenance = if is_reentry_authorization
        || is_closure_authorization
        || is_infrastructure_authorization
        || is_terminal_closure_authorization
    {
        None
    } else {
        match review_log::load_decision_provenance(
            feature,
            stage,
            override_reason,
            &cfg.codex_dir,
            &cfg.repo_root,
        ) {
            Ok(p) => p,
            Err(e) => {
                // Assessment file exists but provenance is broken. Fail-closed unless --override.
                if override_reason.is_none() {
                    eprintln!("error: assessment file exists but provenance cannot be verified:");
                    eprintln!("       {}", e);
                    eprintln!("       Broken provenance cannot be silently bypassed.");
                    eprintln!("       Fix or remove the malformed assessment file, or pass:");
                    eprintln!(
                        "       --override \"<rationale>\" to record explicit human acceptance."
                    );
                    return Ok(crate::EXIT_USAGE);
                }
                // Override present: log a PROVENANCE_UNVERIFIABLE entry and proceed.
                eprintln!("warning: broken provenance bypassed via --override");
                Some(DecisionProvenance {
                    assessment_path: extract_path_from_error(&e),
                    review_commit: String::new(),
                    head_sha: String::new(),
                    packet_sha_stored: String::new(),
                    packet_sha_actual: String::new(),
                    coverage_state: String::new(),
                    override_rationale: override_reason.map(|s| s.to_string()),
                    provenance_error: Some(e.to_string()),
                })
            }
        }
    };

    // Coverage gate — only for APPROVE_STAGE (AC-7: REQUEST_CHANGES / STOP are never gated).
    if decision == "APPROVE_STAGE" {
        if let Some(ref prov) = provenance {
            if prov.provenance_error.is_none()
                && matches!(
                    prov.coverage_state.as_str(),
                    "CRITICAL_OMISSION" | "EMPTY_PACKET"
                )
                && override_reason.is_none()
            {
                // Software-enforced stop (AC-1). Does NOT write to the log.
                eprintln!(
                    "error: APPROVE_STAGE refused — reviewer saw incomplete evidence \
                     (coverage_state: {})",
                    prov.coverage_state
                );
                eprintln!(
                    "       Automated progression requires complete evidence. To record explicit"
                );
                eprintln!(
                    "       human acceptance of the associated risk, \
                     pass: --override \"<rationale>\""
                );
                return Ok(crate::EXIT_USAGE);
            }
        }
    }

    if let Err(e) = review_log::append_decision(
        &cfg.review_log,
        feature,
        stage,
        decision,
        reason,
        after_review,
        if is_closure_authorization || is_infrastructure_authorization {
            Some(findings)
        } else {
            None
        },
        if is_infrastructure_authorization {
            event_evidence_sha
        } else {
            None
        },
        terminal_scope.as_ref(),
        provenance.as_ref(),
        &cfg.repo_root,
        &cfg.codex_dir,
    ) {
        eprintln!("error appending decision to log: {}", e);
        return Ok(crate::EXIT_WRITE);
    }

    println!("decision appended to {}", cfg.review_log.display());
    Ok(crate::EXIT_SUCCESS)
}

/// Extract a path hint from an anyhow error message (best-effort, for the provenance struct).
fn extract_path_from_error(e: &anyhow::Error) -> String {
    let msg = e.to_string();
    // The error messages from load_decision_provenance include the path after the last colon.
    if let Some(pos) = msg.rfind(": ") {
        msg[pos + 2..].trim().to_string()
    } else {
        "(path unknown)".to_string()
    }
}
