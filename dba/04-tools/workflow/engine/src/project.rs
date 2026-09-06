//! Resolving a project's canonical paths and its active DBA configuration.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// The canonical locations a checkpoint reads evidence from. Downstream projects use `.codeos/…`;
/// Codeos self-development keeps reviewer records under `maintenance/reviews`.
pub struct Project {
    pub root: PathBuf,
    pub self_dev: bool,
}

impl Project {
    /// Resolve upward from `start` to the directory that holds `.codeos/` (downstream) or
    /// `dba-system.md` + `dba/00-entry` (Codeos self-development).
    pub fn discover(start: &Path) -> Result<Self> {
        let start = start
            .canonicalize()
            .with_context(|| format!("canonicalize {}", start.display()))?;
        for dir in start.ancestors() {
            let self_dev = dir.join("dba-system.md").is_file() && dir.join("dba/00-entry").is_dir();
            if self_dev {
                return Ok(Self {
                    root: dir.to_path_buf(),
                    self_dev: true,
                });
            }
            if dir.join(".codeos").is_dir() {
                return Ok(Self {
                    root: dir.to_path_buf(),
                    self_dev: false,
                });
            }
        }
        anyhow::bail!(
            "no Codeos project found at or above {} (looked for .codeos/ or a toolkit root)",
            start.display()
        )
    }

    pub fn codeos_dir(&self) -> PathBuf {
        self.root.join(".codeos")
    }

    pub fn charter(&self) -> PathBuf {
        self.codeos_dir().join("00-project/charter.md")
    }

    pub fn codeos_yaml(&self) -> PathBuf {
        self.codeos_dir().join("00-project/codeos.yaml")
    }

    pub fn intent(&self, feature: &str) -> PathBuf {
        self.codeos_dir()
            .join(format!("01-specification/intents/{feature}.md"))
    }

    pub fn contract(&self, feature: &str) -> PathBuf {
        self.codeos_dir()
            .join(format!("01-specification/contracts/{feature}_contract.md"))
    }

    pub fn event_schema(&self, feature: &str) -> PathBuf {
        self.codeos_dir().join(format!(
            "01-specification/event-schemas/{feature}_schema.md"
        ))
    }

    /// Directory holding the reviewer tool's assessment records.
    pub fn review_records_dir(&self) -> PathBuf {
        if self.self_dev {
            self.root.join("maintenance/reviews")
        } else {
            self.codeos_dir().join("05-review/reviews")
        }
    }

    /// Repo-relative paths, from the project root, that make up a feature's implementation state.
    /// Deliberately broad: the whole backend/web/migrations surface plus any project-native src.
    pub fn implementation_paths(&self) -> Vec<&'static str> {
        vec!["backend", "web", "migrations", "src", "docker-compose.yml"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn discovers_downstream_root_by_dot_codeos() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".codeos/01-specification")).unwrap();
        let deep = dir.path().join(".codeos/01-specification");
        let p = Project::discover(&deep).unwrap();
        assert!(!p.self_dev);
        assert_eq!(
            p.root.canonicalize().unwrap(),
            dir.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn discovers_self_dev_root() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("dba-system.md"), "x").unwrap();
        fs::create_dir_all(dir.path().join("dba/00-entry")).unwrap();
        let p = Project::discover(dir.path()).unwrap();
        assert!(p.self_dev);
        assert!(p.review_records_dir().ends_with("maintenance/reviews"));
    }
}
