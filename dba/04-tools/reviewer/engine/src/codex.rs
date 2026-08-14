use crate::config::Config;
use crate::packet::ReviewPacket;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

/// The reviewer-facing result of one Codex invocation. Every Codex CLI detail stays in this module.
pub struct CodexResult {
    pub text: String,
    pub session_id: String,
    pub elapsed_ms: u64,
    pub reconnect_count: u32,
    pub effort: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SessionState {
    feature: String,
    session_id: String,
    codex_version: String,
    created_at: String,
}

pub fn invoke(packet: &ReviewPacket, cfg: &Config, fresh: bool) -> Result<CodexResult> {
    let session_file = cfg.sessions_dir.join(format!("{}.json", packet.feature));
    let codex_version = codex_version()?;
    let saved = load_session(&session_file)?;
    let resume_id = if fresh {
        None
    } else if let Some(state) = saved {
        if !state.codex_version.is_empty() && state.codex_version != codex_version {
            eprintln!(
                "note: codex version changed ({} -> {}); starting a fresh session.",
                state.codex_version, codex_version
            );
            None
        } else {
            Some(state.session_id)
        }
    } else {
        None
    };

    let started_new_session = resume_id.is_none();
    let final_message = tempfile::NamedTempFile::new().context("create Codex output file")?;
    let start = Instant::now();
    let output = run_codex(
        resume_id.as_deref(),
        &cfg.repo_root,
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
    let event_session_id = parse_session_id(&events)?;
    let session_id = resume_id.unwrap_or_else(|| event_session_id.clone());
    let text = std::fs::read_to_string(final_message.path())
        .context("could not read Codex final message")?;
    if text.trim().is_empty() {
        bail!("codex exec completed without a final message");
    }

    if started_new_session {
        save_session(
            &session_file,
            &SessionState {
                feature: packet.feature.clone(),
                session_id: session_id.clone(),
                codex_version,
                created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            },
        )?;
    }

    Ok(CodexResult {
        text,
        session_id,
        elapsed_ms,
        reconnect_count: events
            .lines()
            .filter(|line| line.contains("stream disconnected"))
            .count() as u32,
        effort: cfg.reasoning_effort.clone(),
    })
}

fn run_codex(
    resume_id: Option<&str>,
    repo_root: &Path,
    effort: &str,
    packet: &str,
    final_message: &Path,
) -> Result<std::process::Output> {
    let mut command = Command::new("codex");
    command.arg("exec");
    if let Some(session_id) = resume_id {
        command.args(["resume", "--json"]);
        command.args(["-c", "sandbox_mode=read-only"]);
        command.args(["-c", &format!("model_reasoning_effort={effort}")]);
        command.args(["-o", path_arg(final_message)?]);
        command.args([session_id, "-"]);
    } else {
        command.args(["--json", "--sandbox", "read-only"]);
        command.args(["--cd", path_arg(repo_root)?]);
        command.args(["-c", &format!("model_reasoning_effort={effort}")]);
        command.args(["-o", path_arg(final_message)?, "-"]);
    }

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

fn load_session(path: &Path) -> Result<Option<SessionState>> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("could not read {}", path.display()));
        }
    };
    let state = serde_json::from_str(&content).with_context(|| {
        format!(
            "malformed session state {}; delete it or pass --fresh",
            path.display()
        )
    })?;
    Ok(Some(state))
}

fn save_session(path: &Path, state: &SessionState) -> Result<()> {
    let parent = path.parent().context("session state path has no parent")?;
    std::fs::create_dir_all(parent).context("create Codex sessions directory")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).context("create session state")?;
    serde_json::to_writer_pretty(&mut temporary, state).context("serialize session state")?;
    temporary.write_all(b"\n").context("finish session state")?;
    temporary
        .persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("save session state {}", path.display()))?;
    Ok(())
}

fn codex_version() -> Result<String> {
    let output = Command::new("codex")
        .arg("--version")
        .output()
        .context("codex CLI not found on PATH")?;
    if !output.status.success() {
        bail!("codex --version failed with status {}", output.status);
    }
    Ok(String::from_utf8(output.stdout)
        .context("codex --version output was not UTF-8")?
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string())
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
