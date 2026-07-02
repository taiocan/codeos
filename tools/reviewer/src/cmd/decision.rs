use crate::config::Config;
use crate::log as review_log;
use anyhow::Result;

const VALID_DECISIONS: &[&str] = &["APPROVE_STAGE", "REQUEST_CHANGES", "STOP"];

pub fn run(feature: &str, stage: &str, decision: &str, reason: &str, cfg: &Config) -> Result<i32> {
    if !VALID_DECISIONS.contains(&decision) {
        eprintln!("decision must be APPROVE_STAGE | REQUEST_CHANGES | STOP");
        return Ok(crate::EXIT_USAGE);
    }

    if let Err(e) = review_log::append_decision(
        &cfg.review_log,
        feature,
        stage,
        decision,
        reason,
        &cfg.repo_root,
        &cfg.codex_dir,
    ) {
        eprintln!("error appending decision to log: {}", e);
        return Ok(crate::EXIT_WRITE);
    }

    println!("decision appended to {}", cfg.review_log.display());
    Ok(crate::EXIT_SUCCESS)
}
