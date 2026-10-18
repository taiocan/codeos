//! Per-command counts of the evaluator's expensive operations (SPERF WO-01).
//!
//! Counting never changes behavior. When `CODEOS_WORKFLOW_COUNTS` names a file, the command writes
//! its counts there as JSON at exit, so tests can assert them without touching the report.

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;
use std::sync::Mutex;

pub const COUNTS_ENV: &str = "CODEOS_WORKFLOW_COUNTS";

static COUNTS: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());
static BINDINGS: Mutex<BTreeSet<(String, String)>> = Mutex::new(BTreeSet::new());

pub fn count(key: &str) {
    *COUNTS.lock().unwrap().entry(key.to_owned()).or_default() += 1;
}

/// A subprocess, counted by program name.
pub fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let program = program.as_ref();
    let name = std::path::Path::new(program)
        .file_name()
        .unwrap_or(program)
        .to_string_lossy();
    count(&format!("process:{name}"));
    let mut command = Command::new(program);
    if let Some(environment) = crate::execution::current() {
        command.env(crate::execution::ENVIRONMENT_ENV, environment.as_str());
    }
    command
}

/// One computation of a binding of `kind` over `paths`. Repeats of the same input count as
/// computations but not as distinct bindings.
pub fn binding(kind: &str, paths: &[String]) {
    count(&format!("binding:{kind}"));
    BINDINGS
        .lock()
        .unwrap()
        .insert((kind.to_owned(), paths.join("\0")));
}

pub fn snapshot() -> serde_json::Value {
    let mut distinct: BTreeMap<String, u64> = BTreeMap::new();
    for (kind, _) in BINDINGS.lock().unwrap().iter() {
        *distinct.entry(kind.clone()).or_default() += 1;
    }
    serde_json::json!({
        "counts": *COUNTS.lock().unwrap(),
        "distinct_bindings": distinct,
    })
}

/// Write the counts if the environment asks for them. A failure to write is reported, never fatal.
pub fn write_if_requested() {
    let Some(path) = std::env::var_os(COUNTS_ENV) else {
        return;
    };
    let text = serde_json::to_string_pretty(&snapshot()).unwrap_or_default();
    if let Err(error) = std::fs::write(&path, text) {
        eprintln!("warning: could not write {COUNTS_ENV}: {error}");
    }
}
