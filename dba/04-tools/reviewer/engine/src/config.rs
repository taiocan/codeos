use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub provider_name: String,
    pub provider_source: &'static str,
    pub reasoning_effort: String,
    pub repo_root: PathBuf,
    pub toolkit_root: PathBuf,
    pub review_log: PathBuf,
    pub codex_dir: PathBuf,
    pub state_dir: PathBuf,
    pub sessions_dir: PathBuf,
    pub stage_start_dir: PathBuf,
}

#[derive(Debug, Deserialize, Default)]
struct ReviewerToml {
    provider: Option<String>,
    reasoning_effort: Option<String>,
}

/// Resolve config. Precedence (highest to lowest):
///   1. `cli_provider` argument (--provider flag)
///   2. `CODEOS_REVIEWER_PROVIDER` environment variable
///   3. `.codeos/05-review/reviewer.toml` downstream or `reviewer.toml` at the toolkit root
///   4. Compiled-in default: "codex"
pub fn resolve(cli_provider: Option<&str>, repo_root: &Path) -> Result<Config> {
    let (provider_name, provider_source) = resolve_provider(cli_provider, repo_root)?;
    let reasoning_effort = std::env::var("CODEOS_REASONING_EFFORT")
        .unwrap_or_else(|_| "high".to_string());

    let toolkit_root = find_toolkit_root(repo_root);
    let state_dir = repo_root.join(".codeos-state");
    let self_development = repo_root.join("dba-system.md").is_file()
        && repo_root.join("dba/00-entry").is_dir();
    let review_root = if self_development {
        repo_root.join("maintenance/reviews")
    } else {
        repo_root.join(".codeos/05-review/reviews")
    };

    Ok(Config {
        provider_name,
        provider_source,
        reasoning_effort,
        review_log: review_root.join("review-log.md"),
        codex_dir: review_root.join("codex"),
        state_dir: state_dir.clone(),
        sessions_dir: state_dir.join("codex-sessions"),
        stage_start_dir: state_dir.join("stage-start"),
        toolkit_root: toolkit_root.clone(),
        repo_root: repo_root.to_path_buf(),
    })
}

fn resolve_provider(cli_provider: Option<&str>, repo_root: &Path) -> Result<(String, &'static str)> {
    // Priority 1: CLI flag
    if let Some(p) = cli_provider {
        validate_provider_name(p)?;
        return Ok((p.to_string(), "cli flag"));
    }
    // Priority 2: environment variable
    if let Ok(p) = std::env::var("CODEOS_REVIEWER_PROVIDER") {
        if !p.is_empty() {
            validate_provider_name(&p)?;
            return Ok((p, "env CODEOS_REVIEWER_PROVIDER"));
        }
    }
    // Priority 3: reviewer.toml
    let toml_path = find_reviewer_toml(repo_root);
    if let Some(path) = toml_path {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("could not read {}", path.display()))?;
        let parsed: ReviewerToml = toml::from_str(&content)
            .with_context(|| format!("malformed reviewer.toml at {}", path.display()))?;
        if let Some(p) = parsed.provider {
            if !p.is_empty() {
                validate_provider_name(&p)?;
                return Ok((p, "reviewer.toml"));
            }
        }
    }
    // Priority 4: default
    Ok(("codex".to_string(), "compiled-in default"))
}

fn validate_provider_name(name: &str) -> Result<()> {
    match name {
        "codex" | "opencode" | "gemini" | "kimi" => Ok(()),
        other => anyhow::bail!(
            "unknown provider '{}'; supported: codex, opencode, gemini, kimi",
            other
        ),
    }
}

fn find_reviewer_toml(repo_root: &Path) -> Option<PathBuf> {
    let self_development = repo_root.join("dba-system.md").is_file()
        && repo_root.join("dba/00-entry").is_dir();
    let path = if self_development {
        repo_root.join("reviewer.toml")
    } else {
        repo_root.join(".codeos/05-review/reviewer.toml")
    };
    path.exists().then_some(path)
}

fn find_toolkit_root(repo_root: &Path) -> PathBuf {
    // Downstream: .codeos is project-local and its toolkit child is the shared mount.
    let toolkit = repo_root.join(".codeos/toolkit");
    if toolkit.is_symlink() {
        if let Ok(target) = std::fs::read_link(&toolkit) {
            if target.is_absolute() {
                return target;
            }
            return repo_root.join(".codeos").join(target);
        }
    }
    // Self-dev: repo_root is the toolkit root
    repo_root.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;

    /// CODEOS_REVIEWER_PROVIDER is process-global, not thread-local, but Rust's default test
    /// harness runs tests in parallel threads within the same process. Every test in this
    /// module that reads/mutates this env var acquires this lock first, serializing them
    /// against each other and eliminating the race (UPG-0040). Recovering from a poisoned
    /// mutex (via `into_inner()` rather than a bare `.unwrap()`) means one test panicking
    /// while holding the lock doesn't cascade-block every subsequent test in this module.
    static ENV_VAR_LOCK: Mutex<()> = Mutex::new(());

    fn tmp_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        // Create a minimal git repo structure
        fs::create_dir_all(dir.path().join(".git")).expect("create .git");
        dir
    }

    #[test]
    fn default_provider_is_codex() {
        let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tmp_repo();
        // Remove env var if set
        std::env::remove_var("CODEOS_REVIEWER_PROVIDER");
        let cfg = resolve(None, dir.path()).expect("resolve");
        assert_eq!(cfg.provider_name, "codex");
        assert_eq!(cfg.provider_source, "compiled-in default");
    }

    #[test]
    fn cli_flag_overrides_env() {
        let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tmp_repo();
        std::env::set_var("CODEOS_REVIEWER_PROVIDER", "opencode");
        let cfg = resolve(Some("codex"), dir.path()).expect("resolve");
        assert_eq!(cfg.provider_name, "codex");
        assert_eq!(cfg.provider_source, "cli flag");
        std::env::remove_var("CODEOS_REVIEWER_PROVIDER");
    }

    #[test]
    fn env_var_overrides_toml() {
        let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tmp_repo();
        let config_dir = dir.path().join(".codeos/05-review");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(config_dir.join("reviewer.toml"), "provider = \"gemini\"\n")
            .expect("write toml");
        std::env::set_var("CODEOS_REVIEWER_PROVIDER", "opencode");
        let cfg = resolve(None, dir.path()).expect("resolve");
        assert_eq!(cfg.provider_name, "opencode");
        assert_eq!(cfg.provider_source, "env CODEOS_REVIEWER_PROVIDER");
        std::env::remove_var("CODEOS_REVIEWER_PROVIDER");
    }

    #[test]
    fn toml_overrides_default() {
        let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tmp_repo();
        std::env::remove_var("CODEOS_REVIEWER_PROVIDER");
        let config_dir = dir.path().join(".codeos/05-review");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(config_dir.join("reviewer.toml"), "provider = \"opencode\"\n")
            .expect("write toml");
        let cfg = resolve(None, dir.path()).expect("resolve");
        assert_eq!(cfg.provider_name, "opencode");
        assert_eq!(cfg.provider_source, "reviewer.toml");
    }

    #[test]
    fn unknown_provider_returns_err() {
        let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tmp_repo();
        std::env::remove_var("CODEOS_REVIEWER_PROVIDER");
        let result = resolve(Some("unknown-ai"), dir.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("unknown provider"));
    }
}
