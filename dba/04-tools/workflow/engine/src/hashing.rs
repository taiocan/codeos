//! Content hashing for evidence bindings.
//!
//! Two kinds of hash are needed: a stable hash of one file's bytes, and a hash of the current
//! working-tree state of a set of paths. The working-tree hash deliberately does not require a
//! commit — an Early Development Preview is allowed against uncommitted work.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;

/// Hex sha256 of a file's bytes. Returns `None` if the file does not exist.
pub fn file_sha256(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(Some(hex_digest(&bytes)))
}

/// A hash over the current working-tree state of `paths`, relative to `repo_root`. Combines
/// `git rev-parse HEAD`, the tracked diff for those paths, and the bytes of any untracked file
/// under them, so any change — committed or not — moves the hash.
pub fn working_tree_state(repo_root: &Path, paths: &[&str]) -> Result<String> {
    let mut hasher = Sha256::new();

    let head = git(repo_root, &["rev-parse", "HEAD"]).unwrap_or_default();
    hasher.update(b"HEAD\0");
    hasher.update(head.as_bytes());

    for p in paths {
        hasher.update(b"\0PATH\0");
        hasher.update(p.as_bytes());

        let diff = git(repo_root, &["diff", "HEAD", "--", p]).unwrap_or_default();
        hasher.update(b"\0DIFF\0");
        hasher.update(diff.as_bytes());

        let untracked = git(
            repo_root,
            &["ls-files", "--others", "--exclude-standard", "--", p],
        )
        .unwrap_or_default();
        for rel in untracked.lines().filter(|l| !l.is_empty()) {
            let full = repo_root.join(rel);
            if let Ok(bytes) = std::fs::read(&full) {
                hasher.update(b"\0UNTRACKED\0");
                hasher.update(rel.as_bytes());
                hasher.update(b"\0");
                hasher.update(&bytes);
            }
        }
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Hex sha256 of a string, after trimming — so trailing-whitespace edits don't count as drift.
pub fn text_sha256(s: &str) -> String {
    hex_digest(s.trim().as_bytes())
}

/// The first bound input whose recorded hash no longer matches the current map, as
/// `(name, was, now)`. `None` when every recorded binding still matches. Shared by decision-receipt
/// and verification-record staleness — a record is stale the moment one bound input drifts.
pub fn binding_drift(
    recorded: &std::collections::BTreeMap<String, String>,
    current: &std::collections::BTreeMap<String, String>,
) -> Option<(String, String, String)> {
    for (name, was) in recorded {
        match current.get(name) {
            Some(now) if now == was => {}
            Some(now) => return Some((name.clone(), was.clone(), now.clone())),
            None => {
                return Some((
                    name.clone(),
                    was.clone(),
                    "<input no longer present>".to_string(),
                ))
            }
        }
    }
    None
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn git(repo_root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .context("run git")?;
    if !out.status.success() {
        anyhow::bail!("git {:?} failed", args);
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn file_sha256_is_stable_and_content_sensitive() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("a.txt");
        fs::write(&f, "hello").unwrap();
        let h1 = file_sha256(&f).unwrap().unwrap();
        let h2 = file_sha256(&f).unwrap().unwrap();
        assert_eq!(h1, h2);
        fs::write(&f, "hello world").unwrap();
        let h3 = file_sha256(&f).unwrap().unwrap();
        assert_ne!(h1, h3);
    }

    #[test]
    fn file_sha256_missing_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(file_sha256(&dir.path().join("nope")).unwrap().is_none());
    }
}
