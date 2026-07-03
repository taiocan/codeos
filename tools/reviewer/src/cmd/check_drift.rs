use std::path::Path;

// Exact basename matches (any directory depth) — mirrors templates/stack-manifest.md trigger list.
const WATCHED_EXACT: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "pyproject.toml",
    "poetry.lock",
    "requirements.txt",
    "package.json",
    "package-lock.json",
    "pnpm-lock.yaml",
    "Dockerfile",
    "docker-compose.yml",
    ".env.example",
];

fn is_watched(path: &str) -> bool {
    let basename = path.rsplit('/').next().unwrap_or(path);
    if WATCHED_EXACT.contains(&basename) {
        return true;
    }
    // config/*.toml or config/*.yaml
    if path.starts_with("config/") && (path.ends_with(".toml") || path.ends_with(".yaml")) {
        return true;
    }
    // settings.* — any file whose basename starts with "settings."
    if basename.starts_with("settings.") {
        return true;
    }
    false
}

pub fn run(base: &str, strict: bool, repo_root: &Path) -> i32 {
    let spec = format!("{}..HEAD", base);
    let diff_out = match std::process::Command::new("git")
        .args(["diff", "--name-only", &spec])
        .current_dir(repo_root)
        .output()
    {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            eprintln!("error: git diff failed: {}", String::from_utf8_lossy(&o.stderr).trim());
            return crate::EXIT_CONFIG;
        }
        Err(e) => {
            eprintln!("error: git not available: {}", e);
            return crate::EXIT_CONFIG;
        }
    };

    let changed: Vec<String> = String::from_utf8_lossy(&diff_out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect();

    let triggered: Vec<&str> = changed.iter()
        .filter(|p| is_watched(p))
        .map(String::as_str)
        .collect();

    if triggered.is_empty() {
        return crate::EXIT_SUCCESS;
    }

    let has_report = changed.iter().any(|p| p.ends_with("stack-reconciliation-report.md"));

    if has_report {
        return crate::EXIT_SUCCESS;
    }

    let prefix = if strict { "STRICT MODE: " } else { "" };
    eprintln!(
        "{}stack drift detected — dependency/config files changed without a stack-reconciliation-report.md",
        prefix
    );
    eprintln!("triggered by: {}", triggered.join(", "));
    eprintln!("action: fill in templates/stack-reconciliation-report.md and include it in this diff");

    crate::EXIT_DRIFT
}
