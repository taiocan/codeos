//! Shared test helpers for codeos-reviewer integration tests.
//!
//! These helpers are used across multiple test files (review, decision, check-drift,
//! generate-dashboard, generate-release-evidence).

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Create a minimal git repo in a temp directory and return (TempDir, base_sha).
pub fn setup_temp_git_repo() -> (TempDir, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let p = dir.path();
    Command::new("git").args(["init"]).current_dir(p).output().expect("git init");
    Command::new("git").args(["config", "user.email", "test@codeos.test"]).current_dir(p).output().ok();
    Command::new("git").args(["config", "user.name", "Codeos Test"]).current_dir(p).output().ok();
    std::fs::write(p.join("tracked.md"), "# tracked\n").expect("write tracked");
    Command::new("git").args(["add", "tracked.md"]).current_dir(p).output().expect("git add");
    Command::new("git").args(["commit", "-m", "initial"]).current_dir(p).output().expect("git commit");
    let sha_out = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(p)
        .output().expect("git rev-parse");
    let sha = String::from_utf8_lossy(&sha_out.stdout).trim().to_string();
    (dir, sha)
}

/// Path to the codeos-reviewer binary under test.
pub fn binary() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target/debug/codeos-reviewer");
    p
}

/// Path to the Codeos repository root (two levels up from tools/reviewer).
pub fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); p.pop(); // tools/reviewer -> Codeos/
    p
}

/// Run the binary with given args from the repo root, return (exit code, stdout, stderr).
pub fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(binary())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("failed to run codeos-reviewer binary");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

/// Run the binary from a specific directory, return (exit code, stdout, stderr).
pub fn run_in_dir(repo_path: &std::path::Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(binary())
        .args(args)
        .current_dir(repo_path)
        .output()
        .expect("run binary in dir");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

/// Helper: add a new commit with an extra file to the temp repo.
pub fn add_extra_commit(repo_path: &std::path::Path, filename: &str, content: &str) -> String {
    std::fs::write(repo_path.join(filename), content).expect("write extra file");
    Command::new("git").args(["add", filename]).current_dir(repo_path).output().expect("git add");
    Command::new("git").args(["commit", "-m", "extra"]).current_dir(repo_path).output().expect("git commit");
    let out = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(repo_path)
        .output().expect("git rev-parse");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
