#!/usr/bin/env bash
# codeos-review.sh — thin shim delegating to the compiled Rust binary.
# Subcommands: review / decision / diagnose / stage-start  (see: codeos-reviewer --help)
# To build: cargo build --release --manifest-path tools/reviewer/Cargo.toml
set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || { echo "error: not inside a git repository" >&2; exit 1; }
BINARY="${REPO_ROOT}/tools/reviewer/target/release/codeos-reviewer"
if [[ ! -x "${BINARY}" ]]; then
  command -v codeos-reviewer >/dev/null 2>&1 && BINARY="codeos-reviewer" || {
    echo "error: binary not found at ${BINARY} and not on PATH" >&2
    echo "       Build: cargo build --release --manifest-path ${REPO_ROOT}/tools/reviewer/Cargo.toml" >&2
    exit 2
  }
fi
exec "${BINARY}" "$@"
