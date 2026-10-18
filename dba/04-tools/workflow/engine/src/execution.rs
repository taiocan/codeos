//! Feature Workflow v13 execution environments and provider-call accounting (DQA WO-03).
//!
//! A `check` runs in one stated environment and starts only the verifications that environment
//! authorizes. Codeos never sets, reads or changes the project's provider mode. A production-source
//! run is accounted from the project adapter's per-attempt ledger, never from evidence receipts.

use crate::project::Project;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const ENVIRONMENT_ENV: &str = "CODEOS_EXECUTION_ENVIRONMENT";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Deterministic,
    Production,
}

impl Environment {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "deterministic" => Some(Self::Deterministic),
            "production" => Some(Self::Production),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deterministic => "deterministic",
            Self::Production => "production",
        }
    }

    /// The only environment that authorizes `mechanic`.
    pub fn of(mechanic: &str) -> Self {
        if mechanic == "production_source_evidence" {
            Self::Production
        } else {
            Self::Deterministic
        }
    }
}

thread_local! {
    static CURRENT: Cell<Option<Environment>> = const { Cell::new(None) };
}

/// Run `f` with every subprocess it starts told the stated environment.
pub fn within<T>(environment: Environment, f: impl FnOnce() -> T) -> T {
    let previous = CURRENT.with(|current| current.replace(Some(environment)));
    let result = f();
    CURRENT.with(|current| current.set(previous));
    result
}

/// The stated environment of the running verification, if any.
pub fn current() -> Option<Environment> {
    CURRENT.with(Cell::get)
}

/// Record details with the environment the verification executed in.
pub fn stamp(details: Option<serde_json::Value>, environment: Environment) -> serde_json::Value {
    let mut details = match details {
        Some(serde_json::Value::Object(object)) => object,
        Some(other) => {
            let mut object = serde_json::Map::new();
            object.insert("result".into(), other);
            object
        }
        None => serde_json::Map::new(),
    };
    details.insert("execution_environment".into(), environment.as_str().into());
    serde_json::Value::Object(details)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCalls {
    pub attempted: u64,
    pub successful: u64,
    pub failed: u64,
    /// Attempts with no ending line: the run ended, or was aborted, before the adapter recorded
    /// an outcome. Counted in the total, never guessed.
    pub unresolved: u64,
    pub total: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LedgerLine {
    schema: u8,
    attempt: u64,
    event: String,
    at: String,
}

fn ledger_dir(project: &Project) -> PathBuf {
    project.codeos_dir().join("06-workflow/provider-ledger")
}

/// Start an accounted production run: create its empty ledger before anything executes, so an
/// abort at any later point leaves the run discoverable. Returns the run identity and ledger path.
pub fn start_run(project: &Project, subject: &str) -> Result<(String, PathBuf)> {
    let dir = ledger_dir(project);
    std::fs::create_dir_all(&dir).context("create provider ledger directory")?;
    let run = format!(
        "{}-{}-{}",
        subject,
        chrono::Utc::now().format("%Y%m%dT%H%M%S%9fZ"),
        std::process::id()
    );
    let path = dir.join(format!("{run}.jsonl"));
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .with_context(|| format!("create provider ledger {}", path.display()))?;
    Ok((run, path))
}

/// The provider calls a run's ledger records.
pub fn count(path: &std::path::Path) -> Result<ProviderCalls> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read provider ledger {}", path.display()))?;
    let mut outcomes: BTreeMap<u64, Option<bool>> = BTreeMap::new();
    for (index, line) in text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
    {
        let entry: LedgerLine = serde_json::from_str(line)
            .with_context(|| format!("provider ledger line {} is malformed", index + 1))?;
        anyhow::ensure!(
            entry.schema == 1 && !entry.at.trim().is_empty(),
            "provider ledger line {} is malformed",
            index + 1
        );
        match entry.event.as_str() {
            "attempt" => anyhow::ensure!(
                outcomes.insert(entry.attempt, None).is_none(),
                "provider ledger repeats attempt {}",
                entry.attempt
            ),
            "success" | "failure" => {
                let outcome = outcomes.get_mut(&entry.attempt).with_context(|| {
                    format!(
                        "provider ledger ends attempt {} it never began",
                        entry.attempt
                    )
                })?;
                anyhow::ensure!(
                    outcome.is_none(),
                    "provider ledger ends attempt {} twice",
                    entry.attempt
                );
                *outcome = Some(entry.event == "success");
            }
            other => anyhow::bail!("provider ledger event '{other}' is unknown"),
        }
    }
    let attempted = outcomes.len() as u64;
    let successful = outcomes.values().filter(|o| **o == Some(true)).count() as u64;
    let failed = outcomes.values().filter(|o| **o == Some(false)).count() as u64;
    Ok(ProviderCalls {
        attempted,
        successful,
        failed,
        unresolved: attempted - successful - failed,
        total: attempted,
    })
}

/// Close a run's accounting: its totals are written beside the ledger, and only then is the run
/// accounted.
pub fn finish_run(path: &std::path::Path, calls: &ProviderCalls) -> Result<()> {
    std::fs::write(
        path.with_extension("totals.json"),
        serde_json::to_vec_pretty(calls)?,
    )
    .context("record provider-call totals")
}

/// Account every run that started but never recorded its totals (an aborted run), returning
/// each run's identity with its totals, or the reason its ledger cannot be counted.
pub fn account_unfinished(project: &Project) -> Result<Vec<(String, Result<ProviderCalls>)>> {
    let dir = ledger_dir(project);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut runs: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "jsonl")
                && !path.with_extension("totals.json").exists()
        })
        .collect();
    runs.sort();
    let mut accounted = Vec::new();
    for path in runs {
        let run = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let calls = count(&path);
        if let Ok(calls) = &calls {
            finish_run(&path, calls)?;
        }
        accounted.push((run, calls));
    }
    Ok(accounted)
}

impl std::fmt::Display for ProviderCalls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "attempted={} successful={} failed={} unresolved={} total={}",
            self.attempted, self.successful, self.failed, self.unresolved, self.total
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ledger(lines: &str) -> Result<ProviderCalls> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run.jsonl");
        std::fs::write(&path, lines).unwrap();
        count(&path)
    }

    const AT: &str = r#""at":"2026-10-18T00:00:00Z""#;

    #[test]
    fn a_timed_out_request_counts_as_one_failed_call() {
        let calls = ledger(&format!(
            "{{\"schema\":1,\"attempt\":1,\"event\":\"attempt\",{AT}}}\n{{\"schema\":1,\"attempt\":1,\"event\":\"failure\",{AT}}}\n"
        ))
        .unwrap();
        assert_eq!(
            calls,
            ProviderCalls {
                attempted: 1,
                successful: 0,
                failed: 1,
                unresolved: 0,
                total: 1
            }
        );
    }

    #[test]
    fn an_attempt_without_an_outcome_is_unresolved() {
        let calls = ledger(&format!(
            "{{\"schema\":1,\"attempt\":1,\"event\":\"attempt\",{AT}}}\n{{\"schema\":1,\"attempt\":1,\"event\":\"success\",{AT}}}\n{{\"schema\":1,\"attempt\":2,\"event\":\"attempt\",{AT}}}\n"
        ))
        .unwrap();
        assert_eq!(
            calls,
            ProviderCalls {
                attempted: 2,
                successful: 1,
                failed: 0,
                unresolved: 1,
                total: 2
            }
        );
    }

    #[test]
    fn an_inconsistent_ledger_cannot_be_counted() {
        for lines in [
            format!("{{\"schema\":1,\"attempt\":1,\"event\":\"success\",{AT}}}\n"),
            format!("{{\"schema\":1,\"attempt\":1,\"event\":\"attempt\",{AT}}}\n{{\"schema\":1,\"attempt\":1,\"event\":\"attempt\",{AT}}}\n"),
            format!("{{\"schema\":1,\"attempt\":1,\"event\":\"retry\",{AT}}}\n"),
            "not json\n".to_string(),
        ] {
            assert!(ledger(&lines).is_err(), "{lines}");
        }
    }
}
