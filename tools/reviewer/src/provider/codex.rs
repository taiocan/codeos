use super::{ProviderConfig, RawAssessment, ReviewProvider};
use crate::packet::ReviewPacket;
use anyhow::{Context, Result, bail};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

pub struct CodexProvider;

impl ReviewProvider for CodexProvider {
    fn name(&self) -> &str {
        "codex"
    }

    fn invoke(&self, packet: &ReviewPacket, cfg: &ProviderConfig) -> Result<RawAssessment> {
        // Verify codex is on PATH before doing any work.
        which_codex().context("codex CLI not found on PATH — install it or choose a different provider")?;

        let sess_file = Path::new(&cfg.sessions_dir).join(format!("{}.json", packet.feature));
        let mut session_id: Option<String> = load_session_id(&sess_file)?;
        let codex_ver = codex_version();

        // Version drift: if stored version differs, start fresh to avoid cross-version anchoring.
        if let Some(sid) = session_id.clone() {
            let stored_ver = load_stored_version(&sess_file).unwrap_or_default();
            if !stored_ver.is_empty() && stored_ver != codex_ver {
                eprintln!(
                    "note: codex version changed ({} -> {}); starting a fresh session.",
                    stored_ver, codex_ver
                );
                session_id = None;
                let _ = std::fs::remove_file(&sess_file);
                let _ = sid;
            }
        }

        let packet_text = packet.content();
        let start = Instant::now();

        let output = if let Some(ref sid) = session_id {
            run_codex_resume(sid, &cfg.reasoning_effort, packet_text)?
        } else {
            run_codex_new(&cfg.repo_root, &cfg.reasoning_effort, packet_text)?
        };

        let elapsed_ms = start.elapsed().as_millis() as u64;
        let reconnect_count = output.lines().filter(|l| l.contains("stream disconnected")).count() as u32;

        let extracted_sid = self.extract_session_id(&output);
        if session_id.is_none() {
            match extracted_sid {
                Some(ref sid) => {
                    std::fs::create_dir_all(&cfg.sessions_dir)
                        .context("could not create sessions directory")?;
                    let json = format!(
                        "{{ \"feature\": \"{}\", \"session_id\": \"{}\", \"codex_version\": \"{}\", \"created_at\": \"{}\" }}\n",
                        packet.feature, sid, codex_ver, chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
                    );
                    std::fs::write(&sess_file, json)
                        .context("could not save session file")?;
                }
                None => {
                    bail!("could not capture a Codex session id from output — aborting (fail-closed). review NOT logged. Inspect the codex output and rerun.");
                }
            }
        }

        let final_session_id = session_id
            .or_else(|| self.extract_session_id(&output))
            .unwrap_or_else(|| "(unknown)".to_string());

        Ok(RawAssessment {
            text: output,
            session_id: final_session_id,
            elapsed_ms,
            reconnect_count,
            effort: cfg.reasoning_effort.clone(),
        })
    }

    fn extract_session_id(&self, raw: &str) -> Option<String> {
        for line in raw.lines() {
            if let Some(rest) = line.strip_prefix("session id: ") {
                let sid = rest.trim().to_string();
                if !sid.is_empty() {
                    return Some(sid);
                }
            }
        }
        None
    }
}

fn which_codex() -> Result<()> {
    Command::new("sh")
        .args(["-c", "command -v codex"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|_| ())
        .context("codex not found")
}

fn codex_version() -> String {
    Command::new("codex")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
        .unwrap_or_default()
}

fn load_session_id(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("could not read session file {}", path.display()))?;
    // Parse "session_id": "..." from the JSON manually to avoid a full JSON dep.
    for line in content.lines() {
        if line.contains("\"session_id\"") {
            if let Some(start) = line.find('"').and_then(|_| {
                let after_key = line.splitn(2, "\"session_id\"").nth(1)?;
                let after_colon = after_key.splitn(2, ':').nth(1)?;
                let trimmed = after_colon.trim();
                if trimmed.starts_with('"') { Some(trimmed) } else { None }
            }) {
                let val: String = start.chars().skip(1).take_while(|&c| c != '"').collect();
                if val.is_empty() {
                    bail!("{} exists but has no session_id (malformed) — delete it or pass --fresh.", path.display());
                }
                return Ok(Some(val));
            }
        }
    }
    bail!("{} exists but has no session_id (malformed) — delete it or pass --fresh.", path.display());
}

fn load_stored_version(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if line.contains("\"codex_version\"") {
            if let Some(start) = line.splitn(2, "\"codex_version\"").nth(1) {
                let after_colon = start.splitn(2, ':').nth(1)?.trim();
                if after_colon.starts_with('"') {
                    let val: String = after_colon.chars().skip(1).take_while(|&c| c != '"').collect();
                    return Some(val);
                }
            }
        }
    }
    None
}

fn run_codex_new(repo_root: &str, effort: &str, packet: &str) -> Result<String> {
    let mut child = Command::new("codex")
        .args([
            "exec",
            "-s", "read-only",
            "--cd", repo_root,
            "-c", &format!("model_reasoning_effort={}", effort),
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn codex exec")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(packet.as_bytes()).context("failed to write packet to codex stdin")?;
    }

    let output = child.wait_with_output().context("failed to wait for codex")?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    Ok(format!("{}{}", stdout, stderr))
}

fn run_codex_resume(session_id: &str, effort: &str, packet: &str) -> Result<String> {
    let mut child = Command::new("codex")
        .args([
            "exec",
            "resume",
            session_id,
            "-c", "sandbox_mode=read-only",
            "-c", &format!("model_reasoning_effort={}", effort),
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn codex exec resume")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(packet.as_bytes()).context("failed to write packet to codex stdin")?;
    }

    let output = child.wait_with_output().context("failed to wait for codex")?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    Ok(format!("{}{}", stdout, stderr))
}
