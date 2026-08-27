//! Shared test helpers for codeos-reviewer integration tests.
//!
//! These helpers are used across reviewer command tests.

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

pub struct FakeCodex {
    pub dir: TempDir,
    pub args_log: PathBuf,
    pub packet_log: PathBuf,
}

/// A deterministic stand-in for the external Codex CLI. It implements only the stable
/// `exec` surface the reviewer consumes and keeps all captures outside the project repo.
pub fn setup_fake_codex() -> FakeCodex {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("fake codex tempdir");
    let script = dir.path().join("codex");
    std::fs::write(
        &script,
        r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "--version" ]]; then
  printf '%s\n' 'codex-cli fake-1.0'
  exit 0
fi
printf '%s\n' "$*" > "${CODEOS_FAKE_ARGS:?}"
output_file=''
while (( $# )); do
  case "$1" in
    -o|--output-last-message)
      output_file="$2"
      shift 2
      ;;
    *) shift ;;
  esac
done
cat > "${CODEOS_FAKE_PACKET:?}"
case "${CODEOS_FAKE_MODE:-success}" in
  failure) printf '%s\n' 'simulated Codex failure' >&2; exit 9 ;;
  malformed) printf '%s\n' 'not-json'; exit 0 ;;
  mutate) printf '%s\n' 'fake reviewer mutation' >> "${CODEOS_FAKE_REPO:?}/tracked.md" ;;
esac
printf '%s\n' 'LOG SUMMARY: NO OBJECTION — fixture review' 'EVIDENCE: A' 'HIGHEST-IMPACT UNCERTAINTY: none' > "$output_file"
printf '{"type":"thread.started","thread_id":"%s"}\n' "${CODEOS_FAKE_SESSION:-fake-session}"
"#,
    )
    .expect("write fake codex");
    let mut permissions = std::fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&script, permissions).unwrap();
    FakeCodex {
        args_log: dir.path().join("args.log"),
        packet_log: dir.path().join("packet.log"),
        dir,
    }
}

pub fn run_with_fake_codex(
    repo_path: &std::path::Path,
    fake: &FakeCodex,
    args: &[&str],
    mode: &str,
) -> (i32, String, String) {
    let old_path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{old_path}", fake.dir.path().display());
    let out = Command::new(binary())
        .args(args)
        .current_dir(repo_path)
        .env("PATH", path)
        .env("CODEOS_FAKE_ARGS", &fake.args_log)
        .env("CODEOS_FAKE_PACKET", &fake.packet_log)
        .env("CODEOS_FAKE_MODE", mode)
        .env("CODEOS_FAKE_REPO", repo_path)
        .output()
        .expect("run binary with fake Codex");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Create a minimal git repo in a temp directory and return (TempDir, base_sha).
pub fn setup_temp_git_repo() -> (TempDir, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let p = dir.path();
    Command::new("git")
        .args(["init"])
        .current_dir(p)
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "test@codeos.test"])
        .current_dir(p)
        .output()
        .ok();
    Command::new("git")
        .args(["config", "user.name", "Codeos Test"])
        .current_dir(p)
        .output()
        .ok();
    std::fs::write(p.join("tracked.md"), "# tracked\n").expect("write tracked");
    Command::new("git")
        .args(["add", "tracked.md"])
        .current_dir(p)
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(p)
        .output()
        .expect("git commit");
    let sha_out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .expect("git rev-parse");
    let sha = String::from_utf8_lossy(&sha_out.stdout).trim().to_string();
    (dir, sha)
}

/// Path to the codeos-reviewer binary under test.
pub fn binary() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target/debug/codeos-reviewer");
    p
}

/// Path to the Codeos repository root (four levels up from the reviewer engine).
pub fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..4 {
        p.pop();
    }
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

/// Run the binary from a directory with extra environment variables set. Used to place a failing
/// `git` shim ahead of the real one on PATH, so a discovery failure can be tested without a
/// mock layer inside the engine.
pub fn run_in_dir_with_env(
    repo_path: &std::path::Path,
    args: &[&str],
    env: &[(&str, &str)],
) -> (i32, String, String) {
    let mut cmd = Command::new(binary());
    cmd.args(args).current_dir(repo_path);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("run binary in dir with env");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

/// Helper: add a new commit with an extra file to the temp repo.
pub fn add_extra_commit(repo_path: &std::path::Path, filename: &str, content: &str) -> String {
    std::fs::write(repo_path.join(filename), content).expect("write extra file");
    Command::new("git")
        .args(["add", filename])
        .current_dir(repo_path)
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "extra"])
        .current_dir(repo_path)
        .output()
        .expect("git commit");
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .expect("git rev-parse");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
