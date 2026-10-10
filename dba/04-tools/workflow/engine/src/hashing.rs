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
/// the committed tree entries for those paths, the tracked diff for those paths, and the bytes of
/// any untracked file under them. An unrelated commit does not invalidate scoped evidence.
pub fn working_tree_state(repo_root: &Path, paths: &[String]) -> Result<String> {
    let mut hasher = Sha256::new();

    for p in paths {
        hasher.update(b"\0PATH\0");
        hasher.update(p.as_bytes());

        let tree = git(repo_root, &["ls-tree", "-r", "HEAD", "--", p.as_str()]).unwrap_or_default();
        hasher.update(b"\0TREE\0");
        hasher.update(tree.as_bytes());

        let diff = git(repo_root, &["diff", "HEAD", "--", p.as_str()]).unwrap_or_default();
        hasher.update(b"\0DIFF\0");
        hasher.update(diff.as_bytes());

        let untracked = git(
            repo_root,
            &[
                "ls-files",
                "--others",
                "--exclude-standard",
                "--",
                p.as_str(),
            ],
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

/// Whether a repository-relative path is a test file rather than product code.
pub fn is_test_path(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    [".spec.", ".test.", ".e2e."]
        .iter()
        .any(|marker| name.contains(marker))
        || rel
            .split('/')
            .any(|part| matches!(part, "tests" | "__tests__" | "e2e"))
}

/// A hash over the current content of every non-test file under `paths`: the product a human
/// judges at Early Development Preview and Final UX Validation. Tracked, modified, and untracked
/// files count alike; a deleted tracked file is recorded as absent.
pub fn product_state(repo_root: &Path, paths: &[String]) -> Result<String> {
    let mut args = vec![
        "ls-files",
        "--cached",
        "--others",
        "--exclude-standard",
        "--",
    ];
    args.extend(paths.iter().map(String::as_str));
    let listed = git(repo_root, &args)?;
    let files: std::collections::BTreeSet<&str> = listed
        .lines()
        .filter(|line| !line.is_empty() && !is_test_path(line))
        .collect();
    let mut hasher = Sha256::new();
    for rel in files {
        hasher.update(b"\0FILE\0");
        hasher.update(rel.as_bytes());
        hasher.update(b"\0");
        match std::fs::read(repo_root.join(rel)) {
            Ok(bytes) => hasher.update(&bytes),
            Err(_) => hasher.update(b"\0ABSENT\0"),
        }
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Hex sha256 of a string, after trimming — so trailing-whitespace edits don't count as drift.
pub fn text_sha256(s: &str) -> String {
    hex_digest(s.trim().as_bytes())
}

pub fn bytes_sha256(bytes: &[u8]) -> String {
    hex_digest(bytes)
}

/// One recorded bound input compared with its current value. A **surface** is a named bound input:
/// a binding key (e.g. `intent`, `implementation_state`) or a Candidate Bundle implementation path.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Comparison {
    pub surface: String,
    /// `None` when the input was not recorded but is present now.
    pub was: Option<String>,
    /// `None` when the input is no longer present.
    pub now: Option<String>,
}

impl Comparison {
    pub fn changed(&self) -> bool {
        self.was != self.now
    }
}

/// Compare every recorded binding with the current map. A current key that was never recorded is
/// not compared.
pub fn binding_comparisons(
    recorded: &std::collections::BTreeMap<String, String>,
    current: &std::collections::BTreeMap<String, String>,
) -> Vec<Comparison> {
    recorded
        .iter()
        .map(|(name, was)| Comparison {
            surface: name.clone(),
            was: Some(was.clone()),
            now: current.get(name).cloned(),
        })
        .collect()
}

/// The first bound input whose recorded hash no longer matches the current map, as
/// `(name, was, now)`.
pub fn binding_drift(
    recorded: &std::collections::BTreeMap<String, String>,
    current: &std::collections::BTreeMap<String, String>,
) -> Option<(String, String, String)> {
    first_drift(&binding_comparisons(recorded, current))
}

/// The first changed comparison, as `(name, was, now)`.
pub fn first_drift(comparisons: &[Comparison]) -> Option<(String, String, String)> {
    comparisons.iter().find(|c| c.changed()).map(|c| {
        (
            c.surface.clone(),
            c.was
                .clone()
                .unwrap_or_else(|| "<input not recorded>".to_string()),
            c.now
                .clone()
                .unwrap_or_else(|| "<input no longer present>".to_string()),
        )
    })
}

/// What kind of persisted evidence an identity names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    AcceptanceReceipt,
    CandidateBundle,
    DecisionReceipt,
    VerificationRecord,
}

/// An existing evidence identity: a `record_identity` or a Candidate Bundle ID.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EvidenceRef {
    pub kind: EvidenceKind,
    pub id: String,
}

/// CodeOS's record identity: the sha256 of a record's JSON serialization. Candidate Bundles,
/// accepted snapshots, and revalidation persist and compare this identity.
pub fn record_identity<T: serde::Serialize>(record: &T) -> Result<String> {
    Ok(hex_digest(&serde_json::to_vec(record)?))
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

    #[test]
    fn implementation_hash_ignores_unrelated_commits_but_tracks_scoped_content() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub fn one() {}\n").unwrap();
        fs::write(dir.path().join("docs/note.md"), "one\n").unwrap();
        let run = |args: &[&str]| {
            let status = Command::new("git")
                .args(args)
                .current_dir(dir.path())
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@t")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@t")
                .status()
                .unwrap();
            assert!(status.success(), "git {args:?}");
        };
        run(&["init", "-q"]);
        run(&["add", "-A"]);
        run(&["commit", "-q", "-m", "initial"]);
        let paths = vec!["src".to_string()];
        let first = working_tree_state(dir.path(), &paths).unwrap();

        fs::write(dir.path().join("docs/note.md"), "two\n").unwrap();
        run(&["add", "-A"]);
        run(&["commit", "-q", "-m", "docs"]);
        assert_eq!(first, working_tree_state(dir.path(), &paths).unwrap());

        fs::write(dir.path().join("src/lib.rs"), "pub fn two() {}\n").unwrap();
        run(&["add", "-A"]);
        run(&["commit", "-q", "-m", "implementation"]);
        assert_ne!(first, working_tree_state(dir.path(), &paths).unwrap());
    }
}

#[cfg(test)]
mod comparison_tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn comparisons_cover_recorded_keys_only_and_drift_is_their_first_change() {
        let recorded = map(&[("a", "1"), ("b", "2"), ("c", "3")]);
        let current = map(&[("a", "1"), ("b", "X"), ("new", "9")]);
        let comparisons = binding_comparisons(&recorded, &current);
        assert_eq!(
            comparisons
                .iter()
                .map(|c| c.surface.as_str())
                .collect::<Vec<_>>(),
            ["a", "b", "c"]
        );
        assert_eq!(comparisons[2].now, None);
        assert_eq!(
            binding_drift(&recorded, &current),
            Some(("b".into(), "2".into(), "X".into()))
        );
        assert_eq!(
            binding_drift(&map(&[("c", "3")]), &map(&[])),
            Some(("c".into(), "3".into(), "<input no longer present>".into()))
        );
        assert_eq!(binding_drift(&recorded, &recorded), None);
    }
}
