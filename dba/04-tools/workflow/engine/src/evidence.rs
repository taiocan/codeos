//! Live predicates over canonical artifacts, plus reviewer-record lookup and a command runner.
//! Nothing here is a receipt: these are conditions the tool evaluates now, or durable records it
//! reads. A historical exit code is never trusted on its own.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

/// Split a Markdown file into (yaml frontmatter, body). Frontmatter is the block between the first
/// two `---` lines when the file starts with `---`.
pub fn split_frontmatter(text: &str) -> (Option<&str>, &str) {
    let rest = match text.strip_prefix("---\n") {
        Some(r) => r,
        None => return (None, text),
    };
    match rest.find("\n---\n") {
        Some(end) => (Some(&rest[..end]), &rest[end + 5..]),
        None => (None, text),
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct SpecFrontmatter {
    pub status: Option<String>,
    pub approved_by: Option<String>,
    pub approved_at: Option<serde_yaml::Value>,
}

/// True when an Intent/Contract/Event-Schema records the joint package approval.
pub fn spec_artifact_approved(path: &Path) -> Result<bool> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(false);
    };
    let (Some(fm), _) = split_frontmatter(&text) else {
        return Ok(false);
    };
    let fm: SpecFrontmatter = serde_yaml::from_str(fm)
        .with_context(|| format!("parse frontmatter of {}", path.display()))?;
    let approved = fm.status.as_deref() == Some("APPROVED")
        && fm.approved_by.as_deref().map(str::trim).unwrap_or("").len() > 3
        && fm
            .approved_at
            .as_ref()
            .map(|v| {
                !matches!(v, serde_yaml::Value::Null)
                    && !v.as_str().map(str::trim).unwrap_or("x").is_empty()
            })
            .unwrap_or(false);
    Ok(approved)
}

/// True when a Charter or Architecture Scope records `approval.by` and `approval.at`.
pub fn approval_recorded(path: &Path) -> Result<bool> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(false);
    };
    let (Some(fm), _) = split_frontmatter(&text) else {
        return Ok(false);
    };
    #[derive(Deserialize)]
    struct Fm {
        approval: Option<serde_yaml::Value>,
    }
    let fm: Fm = serde_yaml::from_str(fm).unwrap_or(Fm { approval: None });
    let Some(serde_yaml::Value::Mapping(m)) = fm.approval else {
        return Ok(false);
    };
    let non_empty = |k: &str| {
        m.get(serde_yaml::Value::String(k.into()))
            .and_then(|v| {
                v.as_str()
                    .map(|s| s.trim().to_string())
                    .or_else(|| Some(v_to_string(v)))
            })
            .map(|s| !s.is_empty() && s != "~" && s != "null")
            .unwrap_or(false)
    };
    Ok(non_empty("by") && non_empty("at"))
}

fn v_to_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

/// Does the file contain a `## <heading>` section that has real content and no unfilled
/// `[placeholder]` token? An unfilled placeholder is bracketed text not immediately followed by
/// `(` (which would make it a Markdown link) and not just digits or `#`.
pub fn section_filled(path: &Path, heading: &str) -> Result<bool> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(false);
    };
    let want = format!("## {heading}");
    let mut found = false;
    let mut in_section = false;
    let mut in_comment = false;
    let mut saw_content = false;

    for line in text.lines() {
        let t = line.trim();
        if !in_comment && t.starts_with("## ") {
            if in_section {
                break;
            }
            in_section = t == want;
            found = found || in_section;
            continue;
        }
        if !in_section {
            continue;
        }
        if in_comment {
            if t.contains("-->") {
                in_comment = false;
            }
            continue;
        }
        if t.starts_with("<!--") {
            if !t.contains("-->") {
                in_comment = true;
            }
            continue;
        }
        if t.is_empty() || t.chars().all(|c| c == '|' || c == '-' || c == ' ') {
            continue;
        }
        if has_unfilled_placeholder(t) {
            return Ok(false);
        }
        saw_content = true;
    }
    Ok(found && saw_content)
}

fn has_unfilled_placeholder(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            if let Some(rel) = line[i + 1..].find(']') {
                let inner = &line[i + 1..i + 1 + rel];
                let after = line.as_bytes().get(i + 1 + rel + 1).copied();
                let is_link = after == Some(b'(');
                let trivial = inner.is_empty()
                    || inner
                        .chars()
                        .all(|c| c.is_ascii_digit() || c == '#' || c == '-' || c == 'O');
                if !is_link && !trivial {
                    return true;
                }
                i = i + 1 + rel + 1;
                continue;
            }
        }
        i += 1;
    }
    false
}

/// A reviewer assessment record, parsed from its frontmatter.
#[derive(Debug, Clone)]
pub struct ReviewRecord {
    pub review_commit: String,
    pub reviewed_packet_sha256: String,
    pub effective_concern: String,
}

/// The most recent reviewer record for `feature` at `stage`, by filename timestamp order.
pub fn latest_review_record(
    dir: &Path,
    feature: &str,
    stage: &str,
) -> Result<Option<ReviewRecord>> {
    if !dir.is_dir() {
        return Ok(None);
    }
    let mut candidates: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "md").unwrap_or(false))
        .collect();
    candidates.sort();
    for path in candidates.into_iter().rev() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let (Some(fm), _) = split_frontmatter(&text) else {
            continue;
        };
        let doc: serde_yaml::Value = match serde_yaml::from_str(fm) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let reviewed = doc.get("reviewed");
        let f = reviewed
            .and_then(|r| r.get("feature"))
            .and_then(|v| v.as_str());
        let s = reviewed
            .and_then(|r| r.get("stage"))
            .and_then(|v| v.as_str());
        if f != Some(feature) || s != Some(stage) {
            continue;
        }
        return Ok(Some(ReviewRecord {
            review_commit: reviewed
                .and_then(|r| r.get("review_commit"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            reviewed_packet_sha256: reviewed
                .and_then(|r| r.get("reviewed_packet_sha256"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            effective_concern: doc
                .get("effective_concern")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        }));
    }
    Ok(None)
}

/// A review outcome permits progression unless it is an unresolved "DO NOT ADVANCE".
pub fn review_permits_progression(concern: &str) -> bool {
    !concern.trim().eq_ignore_ascii_case("DO NOT ADVANCE")
}

/// Result of live-executing a verification command.
pub struct CommandOutcome {
    pub ok: bool,
    /// The command line that was run (empty when nothing could be executed).
    pub command: String,
    pub summary: String,
}

/// Run a shell command from `cwd`. Used only by `check`, never by `status`/`next`.
pub fn run(cwd: &Path, program: &str, args: &[&str]) -> CommandOutcome {
    run_env(cwd, program, args, &[])
}

/// As `run`, with extra environment variables.
pub fn run_env(cwd: &Path, program: &str, args: &[&str], env: &[(&str, &str)]) -> CommandOutcome {
    let mut cmd = Command::new(program);
    cmd.args(args).current_dir(cwd);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let command = format!("{program} {}", args.join(" "));
    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Prefer the most informative test-harness lines (a `test result:` that recorded
            // work, or any failure line) over the raw final lines, which for `cargo test` are
            // an empty trailing doctest summary.
            let salient: Vec<&str> = stdout
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    (t.starts_with("test result:") && !t.contains("0 passed; 0 failed"))
                        || t.starts_with("FAILED")
                        || t.contains("panicked")
                        || t.ends_with("FAILED")
                })
                .collect();
            let tail: String = if salient.is_empty() {
                stdout
                    .lines()
                    .rev()
                    .take(3)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join(" | ")
            } else {
                salient
                    .iter()
                    .rev()
                    .take(3)
                    .rev()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" | ")
            };
            CommandOutcome {
                ok: out.status.success(),
                summary: format!(
                    "{command} -> {}{}",
                    if out.status.success() { "ok" } else { "FAILED" },
                    if tail.is_empty() {
                        String::new()
                    } else {
                        format!(" ({tail})")
                    }
                ),
                command,
            }
        }
        Err(e) => CommandOutcome {
            ok: false,
            summary: format!("{command}: {e}"),
            command,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn frontmatter_split() {
        let (fm, body) = split_frontmatter("---\na: 1\n---\nhello\n");
        assert_eq!(fm.unwrap().trim(), "a: 1");
        assert_eq!(body.trim(), "hello");
        let (fm, _) = split_frontmatter("no frontmatter here");
        assert!(fm.is_none());
    }

    #[test]
    fn spec_approval_requires_all_three_fields() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("c.md");
        fs::write(&p, "---\nstatus: DRAFT\napproved_by:\napproved_at:\n---\nx").unwrap();
        assert!(!spec_artifact_approved(&p).unwrap());
        fs::write(
            &p,
            "---\nstatus: APPROVED\napproved_by: Primoz Gorjup\napproved_at: 2026-09-05\n---\nx",
        )
        .unwrap();
        assert!(spec_artifact_approved(&p).unwrap());
    }

    #[test]
    fn charter_approval_detection() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("charter.md");
        fs::write(&p, "---\napproval: null\n---\nx").unwrap();
        assert!(!approval_recorded(&p).unwrap());
        fs::write(
            &p,
            "---\napproval:\n  by: Primoz Gorjup\n  at: 2026-09-05\n---\nx",
        )
        .unwrap();
        assert!(approval_recorded(&p).unwrap());
    }

    #[test]
    fn review_progression_rule() {
        assert!(review_permits_progression("NO OBJECTION"));
        assert!(review_permits_progression("CHANGES ADVISED"));
        assert!(!review_permits_progression("DO NOT ADVANCE"));
    }
}
