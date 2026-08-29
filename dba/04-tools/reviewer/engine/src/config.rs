use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketBudgetMode {
    Fail,
    Warn,
}

impl PacketBudgetMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fail => "fail",
            Self::Warn => "warn",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub reasoning_effort: String,
    pub packet_budget_mode: PacketBudgetMode,
    pub repo_root: PathBuf,
    pub toolkit_root: PathBuf,
    pub review_log: PathBuf,
    pub codex_dir: PathBuf,
    pub state_dir: PathBuf,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ReviewerToml {
    reasoning_effort: Option<String>,
}

/// Resolve reviewer configuration. Reasoning effort precedence is environment, TOML, default.
pub fn resolve(repo_root: &Path) -> Result<Config> {
    let reviewer_toml = load_reviewer_toml(repo_root)?;
    let reasoning_effort = std::env::var("CODEOS_REASONING_EFFORT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or(reviewer_toml.reasoning_effort)
        .unwrap_or_else(|| "high".to_string());
    let packet_budget_mode = resolve_packet_budget_mode()?;

    let toolkit_root = find_toolkit_root(repo_root);
    let state_dir = repo_root.join(".codeos-state");
    let self_development =
        repo_root.join("dba-system.md").is_file() && repo_root.join("dba/00-entry").is_dir();
    let review_root = if self_development {
        repo_root.join("maintenance/reviews")
    } else {
        repo_root.join(".codeos/05-review/reviews")
    };

    Ok(Config {
        reasoning_effort,
        packet_budget_mode,
        review_log: review_root.join("review-log.md"),
        codex_dir: review_root.join("codex"),
        state_dir: state_dir.clone(),
        toolkit_root: toolkit_root.clone(),
        repo_root: repo_root.to_path_buf(),
    })
}

fn resolve_packet_budget_mode() -> Result<PacketBudgetMode> {
    match std::env::var("CODEOS_PACKET_BUDGET_MODE") {
        Ok(value) => match value.trim() {
            "fail" => Ok(PacketBudgetMode::Fail),
            "warn" => Ok(PacketBudgetMode::Warn),
            other => anyhow::bail!(
                "invalid CODEOS_PACKET_BUDGET_MODE '{other}'; expected 'fail' or 'warn'"
            ),
        },
        Err(std::env::VarError::NotPresent) => Ok(PacketBudgetMode::Fail),
        Err(std::env::VarError::NotUnicode(_)) => {
            anyhow::bail!("CODEOS_PACKET_BUDGET_MODE is not valid Unicode")
        }
    }
}

fn load_reviewer_toml(repo_root: &Path) -> Result<ReviewerToml> {
    if let Some(path) = find_reviewer_toml(repo_root) {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("could not read {}", path.display()))?;
        let parsed: ReviewerToml = toml::from_str(&content)
            .with_context(|| format!("malformed reviewer.toml at {}", path.display()))?;
        return Ok(parsed);
    }
    Ok(ReviewerToml::default())
}

fn find_reviewer_toml(repo_root: &Path) -> Option<PathBuf> {
    let self_development =
        repo_root.join("dba-system.md").is_file() && repo_root.join("dba/00-entry").is_dir();
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

    /// CODEOS_REASONING_EFFORT is process-global, not thread-local, but Rust's default test
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
    fn default_reasoning_effort_is_high() {
        let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tmp_repo();
        std::env::remove_var("CODEOS_REASONING_EFFORT");
        let cfg = resolve(dir.path()).expect("resolve");
        assert_eq!(cfg.reasoning_effort, "high");
    }

    #[test]
    fn packet_budget_mode_defaults_to_fail() {
        let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tmp_repo();
        std::env::remove_var("CODEOS_PACKET_BUDGET_MODE");
        let cfg = resolve(dir.path()).expect("resolve");
        assert_eq!(cfg.packet_budget_mode, PacketBudgetMode::Fail);
    }

    #[test]
    fn packet_budget_warn_is_an_explicit_override() {
        let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tmp_repo();
        std::env::set_var("CODEOS_PACKET_BUDGET_MODE", "warn");
        let cfg = resolve(dir.path()).expect("resolve");
        assert_eq!(cfg.packet_budget_mode, PacketBudgetMode::Warn);
        std::env::remove_var("CODEOS_PACKET_BUDGET_MODE");
    }

    #[test]
    fn unknown_packet_budget_mode_is_rejected() {
        let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tmp_repo();
        std::env::set_var("CODEOS_PACKET_BUDGET_MODE", "ignore");
        let error = resolve(dir.path()).expect_err("unknown mode must fail");
        assert!(error.to_string().contains("expected 'fail' or 'warn'"));
        std::env::remove_var("CODEOS_PACKET_BUDGET_MODE");
    }

    #[test]
    fn env_overrides_toml() {
        let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tmp_repo();
        let config_dir = dir.path().join(".codeos/05-review");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(
            config_dir.join("reviewer.toml"),
            "reasoning_effort = \"medium\"\n",
        )
        .expect("write toml");
        std::env::set_var("CODEOS_REASONING_EFFORT", "high");
        let cfg = resolve(dir.path()).expect("resolve");
        assert_eq!(cfg.reasoning_effort, "high");
        std::env::remove_var("CODEOS_REASONING_EFFORT");
    }

    #[test]
    fn toml_overrides_default() {
        let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tmp_repo();
        let config_dir = dir.path().join(".codeos/05-review");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(
            config_dir.join("reviewer.toml"),
            "reasoning_effort = \"medium\"\n",
        )
        .expect("write toml");
        std::env::remove_var("CODEOS_REASONING_EFFORT");
        let cfg = resolve(dir.path()).expect("resolve");
        assert_eq!(cfg.reasoning_effort, "medium");
    }

    #[test]
    fn stale_provider_key_is_rejected() {
        let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tmp_repo();
        let config_dir = dir.path().join(".codeos/05-review");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(config_dir.join("reviewer.toml"), "provider = \"codex\"\n").expect("write toml");
        let result = resolve(dir.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("malformed reviewer.toml"));
    }
}
