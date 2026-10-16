//! Parsed append-only JSONL stores, reused within one command while the file is unchanged (SPERF).
//!
//! Every read compares the file's length and modification time with the parsed copy, so an append
//! made by this command or another process is seen on the next read. Nothing outlives the process.

use anyhow::{Context, Result};
use std::any::Any;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

type Entry = (u64, Option<SystemTime>, Arc<dyn Any + Send + Sync>);

static CACHE: Mutex<BTreeMap<PathBuf, Entry>> = Mutex::new(BTreeMap::new());

/// The records at `path`, parsed by `parse` only when the file changed since the last read.
pub fn load<T: Send + Sync + 'static>(
    path: &Path,
    parse: impl FnOnce(&str) -> Result<Vec<T>>,
) -> Result<Arc<Vec<T>>> {
    let meta = std::fs::metadata(path).with_context(|| format!("read {}", path.display()))?;
    let identity = (meta.len(), meta.modified().ok());
    if let Some((len, modified, records)) = CACHE.lock().unwrap().get(path) {
        if (*len, *modified) == identity {
            if let Ok(records) = records.clone().downcast::<Vec<T>>() {
                return Ok(records);
            }
        }
    }
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let records = Arc::new(parse(&text)?);
    CACHE.lock().unwrap().insert(
        path.to_path_buf(),
        (identity.0, identity.1, records.clone()),
    );
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unchanged_file_is_parsed_once_and_an_append_is_seen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("records.jsonl");
        std::fs::write(&path, "a\n").unwrap();
        let parses = std::cell::Cell::new(0);
        let read = || {
            load(&path, |text| {
                parses.set(parses.get() + 1);
                Ok(text.lines().map(str::to_owned).collect::<Vec<_>>())
            })
            .unwrap()
        };
        assert_eq!(*read(), ["a"]);
        assert_eq!(*read(), ["a"]);
        assert_eq!(parses.get(), 1);

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        writeln!(file, "b").unwrap();
        assert_eq!(*read(), ["a", "b"]);
        assert_eq!(parses.get(), 2);
    }
}
