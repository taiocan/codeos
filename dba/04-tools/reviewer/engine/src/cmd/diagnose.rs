use crate::config::Config;

pub fn run(feature: Option<&str>, stage: Option<&str>, cfg: &Config) {
    println!("codeos-reviewer diagnostic");
    println!("  reviewer:        codex");
    println!("  reasoning_effort: {}", cfg.reasoning_effort);
    println!("  packet_budget_mode: {}", cfg.packet_budget_mode.as_str());
    println!("  repo_root:       {}", cfg.repo_root.display());
    println!("  toolkit_root:    {}", cfg.toolkit_root.display());
    println!("  review_log:      {}", cfg.review_log.display());
    println!("  codex_dir:       {}", cfg.codex_dir.display());
    println!("  state_dir:       {}", cfg.state_dir.display());
    println!("  codex_isolation: preflight required before every Codex review");
    if let (Some(f), Some(s)) = (feature, stage) {
        println!("  feature:         {}", f);
        println!("  workflow/stage:  {}", s);
    }
}
