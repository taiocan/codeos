use crate::config::Config;

pub fn run(feature: Option<&str>, stage: Option<&str>, cfg: &Config, provider_source: &str) {
    println!("codeos-reviewer diagnostic");
    println!("  provider:        {} (source: {})", cfg.provider_name, provider_source);
    println!("  reasoning_effort: {}", cfg.reasoning_effort);
    println!("  repo_root:       {}", cfg.repo_root.display());
    println!("  toolkit_root:    {}", cfg.toolkit_root.display());
    println!("  review_log:      {}", cfg.review_log.display());
    println!("  codex_dir:       {}", cfg.codex_dir.display());
    println!("  state_dir:       {}", cfg.state_dir.display());
    if let (Some(f), Some(s)) = (feature, stage) {
        println!("  feature:         {}", f);
        println!("  stage:           {}", s);
        let sess_file = cfg.sessions_dir.join(format!("{}.json", f));
        if sess_file.exists() {
            println!("  session file:    {} (exists)", sess_file.display());
        } else {
            println!("  session file:    {} (not found — fresh session will be created)", sess_file.display());
        }
        let stage_start = cfg.stage_start_dir.join(f).join(format!("stage-{}.json", s));
        if stage_start.exists() {
            println!("  stage-start:     {} (exists)", stage_start.display());
        } else {
            println!("  stage-start:     {} (not found — no base pin)", stage_start.display());
        }
    }
}
