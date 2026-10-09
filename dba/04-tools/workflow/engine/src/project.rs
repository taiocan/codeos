//! Resolving a project's canonical paths and its active DBA configuration.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_DATABASE_SERVICE: &str = "db";
const DEFAULT_DATABASE_TEST_URL: &str = "postgres://codeos:codeos@localhost:5432/codeos";
const DEFAULT_BACKEND_HEALTH_URL: &str = "http://localhost:8080/health";
const DEFAULT_WEB_URL: &str = "http://localhost:3000/";

/// Project facts needed to locate verification targets. These values describe where the
/// project's already-declared runtime can be observed; they do not define that runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeVerification {
    pub database_service: String,
    pub database_test_url_env: Option<String>,
    pub backend_health_url: String,
    pub web_url: String,
    pub journey_guard_script: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealityVerification {
    pub command: String,
}

#[derive(Debug, Default, Deserialize)]
struct ProjectConfiguration {
    #[serde(default)]
    runtime_verification: Option<RuntimeVerificationInput>,
    #[serde(default)]
    reality_verification: Option<RealityVerificationInput>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RealityVerificationInput {
    command: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeVerificationInput {
    database_service: String,
    database_test_url_env: String,
    backend_health_url: String,
    web_url: String,
    #[serde(default)]
    journey_guard_script: Option<String>,
}

/// The canonical locations a checkpoint reads evidence from. Downstream projects use `.codeos/…`;
/// Codeos self-development keeps reviewer records under `maintenance/reviews`.
pub struct Project {
    pub root: PathBuf,
    pub self_dev: bool,
}

impl Project {
    /// Resolve the immutable DBA configuration governing this project through the same adapter
    /// used for all selected-policy lookups.
    pub fn dba_configuration(&self) -> Result<String> {
        let resolved = self.resolved_project_configuration("dba_configuration")?;
        resolved
            .lines()
            .find_map(|line| line.strip_prefix("dba_configuration="))
            .map(str::to_owned)
            .context("resolved project configuration names no dba_configuration")
    }

    /// Resolve the selected feature-workflow policy through the project's adopted composition.
    /// Older projects with no reachable resolver retain workflow v1 compatibility.
    pub fn feature_workflow_version(&self) -> Result<u8> {
        self.selected_policy_version(
            "workflow_feature_policy",
            "dba/02-policies/workflow-feature/v1.md",
            &[1, 2, 3, 4, 5, 6, 7, 8, 9],
        )
    }

    pub fn workflow_governance_version(&self) -> Result<u8> {
        self.selected_policy_version(
            "workflow_governance_policy",
            "dba/02-policies/workflow-governance/v1.md",
            &[1, 2, 3, 4],
        )
    }

    pub fn operation_workflow_version(&self) -> Result<u8> {
        self.selected_policy_version(
            "workflow_operation_policy",
            "dba/02-policies/workflow-operation/v1.md",
            &[1, 2],
        )
    }

    pub fn codeos_mechanics_version(&self) -> Result<u8> {
        self.selected_policy_version(
            "codeos_mechanics_policy",
            "dba/02-policies/codeos-mechanics/v3.md",
            &[1, 2, 3, 4, 5, 6, 7],
        )
    }

    fn selected_policy_version(
        &self,
        key: &str,
        legacy_default: &str,
        supported: &[u8],
    ) -> Result<u8> {
        let resolved = self.resolved_project_configuration(key)?;
        let prefix = format!("{key}=");
        let selected = resolved
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap_or(legacy_default);
        version_from_policy(selected, supported)
    }

    fn resolved_project_configuration(&self, key: &str) -> Result<String> {
        let resolver = if self.self_dev {
            self.root
                .join("dba/04-tools/configuration/resolve-project-configuration.sh")
        } else {
            self.codeos_dir()
                .join("toolkit/dba/04-tools/configuration/resolve-project-configuration.sh")
        };
        if !resolver.is_file() {
            let explicitly_bound = std::fs::read_to_string(self.codeos_yaml())
                .map(|text| text.lines().any(|line| line.trim() == "codeos_schema: 2"))
                .unwrap_or(false);
            if explicitly_bound {
                anyhow::bail!(
                    "explicit project configuration cannot resolve {key} because the toolkit resolver is unavailable"
                );
            }
            return Ok("codeos_schema=legacy\nresolution_mode=legacy-dba-7-cutover\ndba_configuration=DBA-7\n".into());
        }
        let output = Command::new("bash")
            .arg(&resolver)
            .arg(&self.root)
            .arg(self.codeos_yaml())
            .output()
            .context("resolve project DBA configuration")?;
        if !output.status.success() {
            anyhow::bail!(
                "project DBA configuration did not resolve: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        String::from_utf8(output.stdout).context("project configuration output is not UTF-8")
    }

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

    pub fn runtime_verification(&self) -> Result<RuntimeVerification> {
        let text = std::fs::read_to_string(self.codeos_yaml()).context("read codeos.yaml")?;
        let config: ProjectConfiguration =
            serde_yaml::from_str(&text).context("parse runtime_verification in codeos.yaml")?;
        let Some(input) = config.runtime_verification else {
            return Ok(RuntimeVerification {
                database_service: DEFAULT_DATABASE_SERVICE.into(),
                database_test_url_env: None,
                backend_health_url: DEFAULT_BACKEND_HEALTH_URL.into(),
                web_url: DEFAULT_WEB_URL.into(),
                journey_guard_script: None,
            });
        };
        for (name, value) in [
            ("database_service", input.database_service.as_str()),
            (
                "database_test_url_env",
                input.database_test_url_env.as_str(),
            ),
            ("backend_health_url", input.backend_health_url.as_str()),
            ("web_url", input.web_url.as_str()),
        ] {
            if value.trim().is_empty() {
                anyhow::bail!("runtime_verification.{name} must be non-empty");
            }
        }
        if !valid_env_name(&input.database_test_url_env) {
            anyhow::bail!(
                "runtime_verification.database_test_url_env must be an environment-variable name"
            );
        }
        for (name, value) in [
            ("backend_health_url", input.backend_health_url.as_str()),
            ("web_url", input.web_url.as_str()),
        ] {
            if !(value.starts_with("http://") || value.starts_with("https://")) {
                anyhow::bail!("runtime_verification.{name} must be an http(s) URL");
            }
        }
        if let Some(path) = input.journey_guard_script.as_deref() {
            if self.codeos_mechanics_version()? < 5 {
                anyhow::bail!(
                    "runtime_verification.journey_guard_script requires Codeos Mechanics v5"
                );
            }
            validate_project_executable(&self.root, path)?;
        }
        Ok(RuntimeVerification {
            database_service: input.database_service,
            database_test_url_env: Some(input.database_test_url_env),
            backend_health_url: input.backend_health_url,
            web_url: input.web_url,
            journey_guard_script: input.journey_guard_script,
        })
    }

    pub fn reality_verification(&self) -> Result<RealityVerification> {
        let text = std::fs::read_to_string(self.codeos_yaml()).context("read codeos.yaml")?;
        let config: ProjectConfiguration =
            serde_yaml::from_str(&text).context("parse reality_verification in codeos.yaml")?;
        let input = config
            .reality_verification
            .context("reality-dependent claims require reality_verification.command")?;
        validate_project_executable(&self.root, &input.command)?;
        Ok(RealityVerification {
            command: input.command,
        })
    }

    /// Resolve the host-side database URL without persisting its value in configuration or
    /// verification evidence. Legacy projects retain the original canonical default.
    pub fn database_test_url(&self, runtime: &RuntimeVerification) -> Result<String> {
        match &runtime.database_test_url_env {
            Some(name) => {
                let value = std::env::var(name).with_context(|| {
                    format!(
                        "required runtime verification environment variable {name} is unavailable"
                    )
                })?;
                if value.trim().is_empty() {
                    anyhow::bail!(
                        "required runtime verification environment variable {name} is empty"
                    );
                }
                Ok(value)
            }
            None => Ok(DEFAULT_DATABASE_TEST_URL.into()),
        }
    }

    /// Cargo establishes the Rust workspace and package set. If the project has no usable Cargo
    /// metadata, retain the original skeleton paths for compatibility.
    pub fn cargo_workspace_root(&self) -> Option<PathBuf> {
        self.resolved_cargo_metadata()
            .and_then(|v| v.get("workspace_root")?.as_str().map(PathBuf::from))
    }

    pub fn compose_has_service(&self, service: &str) -> Result<bool> {
        let out = Command::new("docker")
            .args(["compose", "config", "--services"])
            .current_dir(&self.root)
            .output()
            .context("run docker compose config --services")?;
        if !out.status.success() {
            anyhow::bail!("docker compose config --services failed");
        }
        Ok(service_output_contains(
            &String::from_utf8_lossy(&out.stdout),
            service,
        ))
    }

    pub fn web_has_script(&self, script: &str) -> bool {
        let Ok(bytes) = std::fs::read(self.root.join("web/package.json")) else {
            return false;
        };
        serde_json::from_slice::<serde_json::Value>(&bytes)
            .ok()
            .and_then(|v| v.get("scripts")?.get(script).cloned())
            .and_then(|v| v.as_str().map(str::to_owned))
            .is_some()
    }

    pub fn intent(&self, feature: &str) -> PathBuf {
        self.codeos_dir()
            .join(format!("01-specification/intents/{feature}.md"))
    }

    pub fn contract(&self, feature: &str) -> PathBuf {
        self.codeos_dir()
            .join(format!("01-specification/contracts/{feature}_contract.md"))
    }

    pub fn feature_ids(&self) -> Result<Vec<String>> {
        let dir = self.codeos_dir().join("01-specification/contracts");
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut ids = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
                continue;
            };
            if let Some(id) = name.strip_suffix("_contract.md") {
                ids.push(id.to_string());
            }
        }
        ids.sort();
        Ok(ids)
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

    /// Repo-relative paths that make up the implementation evidence set: Cargo's resolved
    /// workspace/package targets plus the applicable web, migration, and Compose surfaces.
    pub fn implementation_paths(&self) -> Vec<String> {
        let Some(metadata) = self.resolved_cargo_metadata() else {
            return ["backend", "web", "migrations", "src", "docker-compose.yml"]
                .into_iter()
                .map(String::from)
                .collect();
        };
        let mut paths = BTreeSet::new();
        if let Some(root) = metadata.get("workspace_root").and_then(|v| v.as_str()) {
            add_relative(&self.root, Path::new(root).join("Cargo.toml"), &mut paths);
            add_relative(&self.root, Path::new(root).join("Cargo.lock"), &mut paths);
        }
        for package in metadata
            .get("packages")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            if let Some(manifest) = package.get("manifest_path").and_then(|v| v.as_str()) {
                if let Some(parent) = Path::new(manifest).parent() {
                    add_relative(&self.root, parent, &mut paths);
                    let migrations = parent.join("migrations");
                    if migrations.is_dir() {
                        add_relative(&self.root, migrations, &mut paths);
                    }
                }
            }
            for target in package
                .get("targets")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
            {
                if let Some(src) = target.get("src_path").and_then(|v| v.as_str()) {
                    add_relative(&self.root, src, &mut paths);
                }
            }
        }
        for conventional in ["web", "migrations", "docker-compose.yml"] {
            if self.root.join(conventional).exists() {
                paths.insert(conventional.into());
            }
        }
        paths.into_iter().collect()
    }

    fn resolved_cargo_metadata(&self) -> Option<serde_json::Value> {
        cargo_metadata(&self.root).or_else(|| cargo_metadata(&self.root.join("backend")))
    }
}

fn version_from_policy(path: &str, supported: &[u8]) -> Result<u8> {
    let filename = path.rsplit('/').next().unwrap_or(path);
    let Some(raw) = filename
        .strip_prefix('v')
        .and_then(|v| v.strip_suffix(".md"))
    else {
        anyhow::bail!("unsupported policy path '{path}'");
    };
    let version: u8 = raw
        .parse()
        .with_context(|| format!("unsupported policy version '{path}'"))?;
    if !supported.contains(&version) {
        anyhow::bail!("unsupported policy version '{path}'");
    }
    Ok(version)
}

fn validate_project_executable(root: &Path, supplied: &str) -> Result<()> {
    let path = Path::new(supplied);
    if path.is_absolute()
        || supplied.trim().is_empty()
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        anyhow::bail!("runtime_verification.journey_guard_script must be a non-traversing project-relative path");
    }
    let canonical = root
        .join(path)
        .canonicalize()
        .with_context(|| format!("resolve journey guard script {supplied}"))?;
    if !canonical.starts_with(root) || !canonical.is_file() {
        anyhow::bail!("runtime_verification.journey_guard_script must resolve to a regular file inside the project");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if canonical.metadata()?.permissions().mode() & 0o111 == 0 {
            anyhow::bail!("runtime_verification.journey_guard_script must be executable");
        }
    }
    Ok(())
}

fn cargo_metadata(root: &Path) -> Option<serde_json::Value> {
    let out = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(root)
        .output()
        .ok()?
        .stdout;
    serde_json::from_slice(&out).ok()
}

fn add_relative(root: &Path, path: impl AsRef<Path>, paths: &mut BTreeSet<String>) {
    let supplied = path.as_ref();
    let normalized = supplied
        .canonicalize()
        .unwrap_or_else(|_| supplied.to_path_buf());
    if let Ok(relative) = normalized.strip_prefix(root) {
        if !relative.as_os_str().is_empty() {
            paths.insert(relative.to_string_lossy().into_owned());
        }
    }
}

fn valid_env_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some('A'..='Z' | 'a'..='z' | '_'))
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn service_output_contains(output: &str, service: &str) -> bool {
    output.lines().any(|line| line.trim() == service)
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

    fn downstream_project() -> (tempfile::TempDir, Project) {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".codeos/00-project")).unwrap();
        let project = Project::discover(dir.path()).unwrap();
        (dir, project)
    }

    #[test]
    fn runtime_verification_defaults_preserve_the_skeleton_contract() {
        let (_dir, project) = downstream_project();
        fs::write(
            project.codeos_yaml(),
            "platform:\n  persistence: postgresql\nartifacts:\n  charter: governed\n",
        )
        .unwrap();
        let runtime = project.runtime_verification().unwrap();
        assert_eq!(runtime.database_service, "db");
        assert_eq!(runtime.database_test_url_env, None);
        assert_eq!(runtime.backend_health_url, "http://localhost:8080/health");
        assert_eq!(runtime.web_url, "http://localhost:3000/");
    }

    #[test]
    fn explicit_configuration_fails_closed_without_the_toolkit_resolver() {
        let (_dir, project) = downstream_project();
        fs::write(
            project.codeos_yaml(),
            "codeos_schema: 2\ndba_configuration: DBA-9\n",
        )
        .unwrap();
        let error = project.feature_workflow_version().unwrap_err().to_string();
        assert!(error.contains("toolkit resolver is unavailable"));
    }

    #[test]
    fn evidenceatlas_shaped_runtime_profile_resolves_without_project_branches() {
        let (_dir, project) = downstream_project();
        fs::write(
            project.codeos_yaml(),
            "runtime_verification:\n  database_service: postgres\n  database_test_url_env: EVIDENCEATLAS_TEST_DATABASE_URL\n  backend_health_url: http://127.0.0.1:3002/api/publications\n  web_url: http://127.0.0.1:4173/publications\n",
        )
        .unwrap();
        let runtime = project.runtime_verification().unwrap();
        assert_eq!(runtime.database_service, "postgres");
        assert_eq!(
            runtime.database_test_url_env.as_deref(),
            Some("EVIDENCEATLAS_TEST_DATABASE_URL")
        );
        assert_eq!(
            runtime.backend_health_url,
            "http://127.0.0.1:3002/api/publications"
        );
    }

    #[test]
    fn runtime_profile_rejects_unknown_keys_and_invalid_values() {
        let (_dir, project) = downstream_project();
        fs::write(
            project.codeos_yaml(),
            "runtime_verification:\n  database_service: postgres\n  database_test_url_env: not-valid!\n  backend_health_url: ftp://backend\n  web_url: http://web\n  project: evidenceatlas\n",
        )
        .unwrap();
        assert!(project.runtime_verification().is_err());
    }

    #[test]
    #[cfg(unix)]
    fn runtime_profile_accepts_only_an_executable_guard_inside_the_project() {
        use std::os::unix::fs::PermissionsExt;
        let (_dir, project) = downstream_project();
        fs::create_dir_all(project.root.join("scripts")).unwrap();
        let guard = project.root.join("scripts/guard.sh");
        fs::write(&guard, "#!/usr/bin/env bash\nexec \"$@\"\n").unwrap();
        let mut permissions = guard.metadata().unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&guard, permissions).unwrap();
        validate_project_executable(&project.root, "scripts/guard.sh").unwrap();
        assert!(validate_project_executable(&project.root, "../outside.sh").is_err());
    }

    #[test]
    fn configured_database_url_fails_closed_when_the_environment_value_is_missing() {
        let (_dir, project) = downstream_project();
        let variable = "CODEOS_TEST_INTENTIONALLY_MISSING_DATABASE_URL_7E7C";
        std::env::remove_var(variable);
        fs::write(
            project.codeos_yaml(),
            format!("runtime_verification:\n  database_service: postgres\n  database_test_url_env: {variable}\n  backend_health_url: http://backend/health\n  web_url: http://web/\n"),
        )
        .unwrap();
        let runtime = project.runtime_verification().unwrap();
        let error = project.database_test_url(&runtime).unwrap_err().to_string();
        assert!(error.contains(variable));
    }

    #[test]
    fn compose_service_matching_is_exact() {
        let services = "postgres\nserver\npublic-web\n";
        assert!(service_output_contains(services, "postgres"));
        assert!(!service_output_contains(services, "db"));
        assert!(!service_output_contains(services, "post"));
    }

    #[test]
    fn cargo_metadata_discovers_workspace_packages_and_external_targets() {
        let (dir, project) = downstream_project();
        fs::create_dir_all(dir.path().join("apps/server/src")).unwrap();
        fs::create_dir_all(dir.path().join("tests")).unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"apps/server\"]\nresolver = \"2\"\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("apps/server/Cargo.toml"),
            "[package]\nname = \"server\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[test]]\nname = \"outside\"\npath = \"../../tests/outside.rs\"\n",
        )
        .unwrap();
        fs::write(dir.path().join("apps/server/src/lib.rs"), "").unwrap();
        fs::write(dir.path().join("tests/outside.rs"), "#[test] fn ok() {}\n").unwrap();
        let paths = project.implementation_paths();
        assert!(paths.contains(&"Cargo.toml".to_string()));
        assert!(paths.contains(&"apps/server".to_string()));
        assert!(paths.contains(&"tests/outside.rs".to_string()));
        assert!(!paths.iter().any(|p| p.contains("EvidenceAtlas")));
    }

    #[test]
    fn cargo_metadata_preserves_the_legacy_backend_layout() {
        let (dir, project) = downstream_project();
        fs::create_dir_all(dir.path().join("backend/src")).unwrap();
        fs::write(
            dir.path().join("backend/Cargo.toml"),
            "[package]\nname = \"backend\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        fs::write(dir.path().join("backend/src/lib.rs"), "").unwrap();
        assert_eq!(
            project.cargo_workspace_root().unwrap(),
            dir.path().join("backend").canonicalize().unwrap()
        );
        assert!(project
            .implementation_paths()
            .contains(&"backend".to_string()));
    }
}
