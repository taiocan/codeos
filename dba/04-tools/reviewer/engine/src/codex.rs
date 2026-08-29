use crate::config::Config;
use crate::packet::ReviewPacket;
use crate::run::{ReviewerRun, RunSource};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

const ISOLATION_PROFILE: &str = "codeos-review";
const ISOLATION_OK: &str = "CODEOS_ISOLATION_OK";

#[derive(Debug, Clone, Copy)]
pub struct BudgetRefusal {
    pub review_content_bytes: u64,
    pub budget_bytes: u64,
}

pub enum InvokeOutcome {
    Completed(ReviewerRun),
    RefusedOverBudget(BudgetRefusal),
}

struct CodexIsolation {
    executable: PathBuf,
    workdir: tempfile::TempDir,
    filesystem_config: String,
}

pub fn budget_refusal(packet: &ReviewPacket, cfg: &Config) -> Option<BudgetRefusal> {
    (cfg.packet_budget_mode == crate::config::PacketBudgetMode::Fail
        && packet.review_content_bytes > packet.budget_bytes)
        .then_some(BudgetRefusal {
            review_content_bytes: packet.review_content_bytes,
            budget_bytes: packet.budget_bytes,
        })
}

pub fn invoke(packet: &ReviewPacket, cfg: &Config) -> Result<InvokeOutcome> {
    // Keep the guard at the common model-spawning boundary so future callers cannot bypass it.
    if let Some(refusal) = budget_refusal(packet, cfg) {
        return Ok(InvokeOutcome::RefusedOverBudget(refusal));
    }

    let isolation = CodexIsolation::establish(&cfg.repo_root)?;
    let final_message = tempfile::NamedTempFile::new().context("create Codex output file")?;
    let start = Instant::now();
    let output = run_codex(
        &isolation,
        &cfg.reasoning_effort,
        packet.content(),
        final_message.path(),
    )?;
    let elapsed_ms = start.elapsed().as_millis() as u64;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(
            "codex exec failed with status {}{}",
            output.status,
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {stderr}")
            }
        );
    }

    let events = String::from_utf8(output.stdout).context("Codex JSONL output was not UTF-8")?;
    let session_id = parse_session_id(&events)?;
    let text = std::fs::read_to_string(final_message.path())
        .context("could not read Codex final message")?;
    if text.trim().is_empty() {
        bail!("codex exec completed without a final message");
    }

    Ok(InvokeOutcome::Completed(ReviewerRun {
        text,
        elapsed_ms,
        source: RunSource::Codex {
            session_id,
            reconnect_count: events
                .lines()
                .filter(|line| line.contains("stream disconnected"))
                .count() as u32,
            effort: cfg.reasoning_effort.clone(),
        },
    }))
}

fn run_codex(
    isolation: &CodexIsolation,
    effort: &str,
    packet: &str,
    final_message: &Path,
) -> Result<std::process::Output> {
    let mut command = Command::new(&isolation.executable);
    command.args([
        "exec",
        "--json",
        "--ephemeral",
        "--ignore-user-config",
        "--ignore-rules",
        "--strict-config",
        "--skip-git-repo-check",
        "--cd",
        path_arg(isolation.workdir.path())?,
        "-c",
        "project_doc_max_bytes=0",
        "-c",
        &format!("default_permissions=\"{ISOLATION_PROFILE}\""),
        "-c",
        "permissions.codeos-review.description=\"Packet-only Codeos reviewer\"",
        "-c",
        &isolation.filesystem_config,
        "-c",
        "permissions.codeos-review.network.enabled=false",
        "-c",
        "shell_environment_policy.inherit=\"none\"",
        "-c",
        &format!("model_reasoning_effort={effort}"),
        "-o",
        path_arg(final_message)?,
        "-",
    ]);

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn codex exec")?;
    child
        .stdin
        .take()
        .context("codex stdin unavailable")?
        .write_all(packet.as_bytes())
        .context("failed to write packet to codex stdin")?;
    child
        .wait_with_output()
        .context("failed to wait for codex exec")
}

impl CodexIsolation {
    fn establish(repo_root: &Path) -> Result<Self> {
        let executable = resolve_codex_executable()?;
        let executable_dir = executable
            .parent()
            .context("resolved Codex executable has no parent directory")?;
        let workdir = tempfile::tempdir().context("create isolated Codex working directory")?;
        let allowed_canary = workdir.path().join("runtime-canary");
        std::fs::write(&allowed_canary, "codeos isolation probe\n")
            .context("write allowed isolation canary")?;

        let filesystem_config = filesystem_config(executable_dir, workdir.path())?;
        verify_effective_isolation(
            &executable,
            workdir.path(),
            &filesystem_config,
            &allowed_canary,
            repo_root,
        )?;

        Ok(Self {
            executable,
            workdir,
            filesystem_config,
        })
    }
}

fn verify_effective_isolation(
    executable: &Path,
    workdir: &Path,
    filesystem_config: &str,
    allowed_canary: &Path,
    repo_root: &Path,
) -> Result<()> {
    let denied_canary = tempfile::NamedTempFile::new().context("create denied isolation canary")?;
    let mut denied_paths = vec![
        repository_probe(repo_root)?,
        denied_canary.path().to_path_buf(),
    ];
    denied_paths.extend(codex_state_probes());

    let script = r#"set -eu
if ! cat "$1" >/dev/null 2>&1; then
  printf '%s\n' CODEOS_RUNTIME_UNREADABLE
  exit 42
fi
shift
for path do
  if cat "$path" >/dev/null 2>&1; then
    printf 'CODEOS_ISOLATION_READABLE %s\n' "$path"
    exit 41
  fi
done
printf '%s\n' CODEOS_ISOLATION_OK"#;

    let mut command = Command::new(executable);
    command.args([
        "sandbox",
        "--permission-profile",
        ISOLATION_PROFILE,
        "--cd",
        path_arg(workdir)?,
        "-c",
        "permissions.codeos-review.description=\"Packet-only Codeos reviewer\"",
        "-c",
        filesystem_config,
        "--",
        "/bin/sh",
        "-c",
        script,
        "codeos-isolation-probe",
        path_arg(allowed_canary)?,
    ]);
    for path in &denied_paths {
        command.arg(path_arg(path)?);
    }

    let output = command
        .output()
        .context("failed to run Codex isolation preflight")?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() || stdout != ISOLATION_OK {
        bail!(
            "Codex isolation preflight failed with status {}{}{}",
            output.status,
            if stdout.is_empty() {
                String::new()
            } else {
                format!(": {stdout}")
            },
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {stderr}")
            }
        );
    }
    Ok(())
}

fn repository_probe(repo_root: &Path) -> Result<PathBuf> {
    for relative in [".git/HEAD", ".git", "AGENTS.md", "CLAUDE.md"] {
        let candidate = repo_root.join(relative);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    let candidate = std::fs::read_dir(repo_root)
        .with_context(|| {
            format!(
                "read repository root {} for isolation probe",
                repo_root.display()
            )
        })?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .find(|path| path.is_file())
        .context("repository has no regular file available for the isolation preflight")?;
    Ok(candidate)
}

fn codex_state_probes() -> Vec<PathBuf> {
    let codex_home = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".codex")));
    let Some(codex_home) = codex_home else {
        return Vec::new();
    };
    ["auth.json", "config.toml"]
        .into_iter()
        .map(|name| codex_home.join(name))
        .filter(|path| path.is_file())
        .collect()
}

fn filesystem_config(executable_dir: &Path, workdir: &Path) -> Result<String> {
    let executable_dir = serde_json::to_string(path_arg(executable_dir)?)?;
    let workdir = serde_json::to_string(path_arg(workdir)?)?;
    Ok(format!(
        "permissions.{ISOLATION_PROFILE}.filesystem={{\":minimal\"=\"read\",{executable_dir}=\"read\",{workdir}=\"read\"}}"
    ))
}

fn resolve_codex_executable() -> Result<PathBuf> {
    let path = std::env::var_os("PATH").context("PATH is not set; cannot locate Codex CLI")?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join("codex");
        if candidate.is_file() {
            return std::fs::canonicalize(&candidate)
                .with_context(|| format!("resolve Codex executable {}", candidate.display()));
        }
    }
    bail!("codex CLI not found on PATH")
}

fn parse_session_id(events: &str) -> Result<String> {
    for (index, line) in events.lines().enumerate() {
        let event: Value = serde_json::from_str(line)
            .with_context(|| format!("malformed Codex JSONL event on line {}", index + 1))?;
        if event.get("type").and_then(Value::as_str) == Some("thread.started") {
            if let Some(id) = event.get("thread_id").and_then(Value::as_str) {
                if !id.is_empty() {
                    return Ok(id.to_string());
                }
            }
        }
    }
    bail!("Codex JSONL output did not contain a thread.started session id")
}

fn path_arg(path: &Path) -> Result<&str> {
    path.to_str()
        .with_context(|| format!("path is not valid UTF-8: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_thread_started_id() {
        let events = "{\"type\":\"thread.started\",\"thread_id\":\"abc-123\"}\n";
        assert_eq!(parse_session_id(events).unwrap(), "abc-123");
    }

    #[test]
    fn rejects_malformed_jsonl() {
        assert!(parse_session_id("not-json\n").is_err());
    }

    #[test]
    fn rejects_missing_session_id() {
        assert!(parse_session_id("{\"type\":\"turn.completed\"}\n").is_err());
    }
}
