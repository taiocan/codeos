#!/usr/bin/env bash
# codeos-review.sh — thin shim delegating to the compiled Rust binary.
# Subcommands: review / decision / diagnose / stage-start  (see: codeos-reviewer --help)
# To build: cargo build --release --manifest-path tools/reviewer/Cargo.toml
set -euo pipefail
# Preserve the original precondition: the shim requires the CALLER to be inside some git
# repository (a property of the project being reviewed) — unrelated to where the binary
# itself lives, so this check is deliberately kept separate from binary-path resolution
# below, running first, exactly as before this change.
git rev-parse --show-toplevel >/dev/null 2>&1 || { echo "error: not inside a git repository" >&2; exit 1; }

# Resolve the binary relative to this script's own physical location (following the
# .codeos symlink when invoked from a downstream project), not the calling repo's git
# root — git rev-parse --show-toplevel from within a downstream project resolves to that
# project's own root, not through .codeos to Codeos, which is where the binary actually
# lives. pwd -P (physical) is required, not plain pwd, to resolve through the symlink
# rather than preserve its logical name.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd -P)"
BINARY="${CODEOS_ROOT}/tools/reviewer/target/release/codeos-reviewer"
if [[ ! -x "${BINARY}" ]]; then
  command -v codeos-reviewer >/dev/null 2>&1 && BINARY="codeos-reviewer" || {
    echo "error: binary not found at ${BINARY} and not on PATH" >&2
    echo "       Build: cargo build --release --manifest-path ${CODEOS_ROOT}/tools/reviewer/Cargo.toml" >&2
    exit 2
  }
fi
exec "${BINARY}" "$@"
