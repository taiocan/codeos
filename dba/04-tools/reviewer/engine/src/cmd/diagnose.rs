use crate::config::Config;

pub fn run(feature: Option<&str>, stage: Option<&str>, cfg: &Config) {
    println!("codeos-reviewer diagnostic");
    println!("  reviewer:        codex");
    println!("  reasoning_effort: {}", cfg.reasoning_effort);
    println!("  repo_root:       {}", cfg.repo_root.display());
    println!("  toolkit_root:    {}", cfg.toolkit_root.display());
    println!("  review_log:      {}", cfg.review_log.display());
    println!("  codex_dir:       {}", cfg.codex_dir.display());
    println!("  state_dir:       {}", cfg.state_dir.display());
    if let (Some(f), Some(s)) = (feature, stage) {
        println!("  feature:         {}", f);
        println!("  workflow/stage:  {}", s);
        let sess_file = cfg.sessions_dir.join(format!("{}.json", f));
        if sess_file.exists() {
            println!("  session file:    {} (exists)", sess_file.display());
        } else {
            println!(
                "  session file:    {} (not found — fresh session will be created)",
                sess_file.display()
            );
        }
    }
}
