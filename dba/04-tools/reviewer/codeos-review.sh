#!/usr/bin/env bash
# codeos-review.sh — thin shim delegating to the compiled Rust binary.
# Subcommands: review / plan / decision / diagnose / stage-start /
#              inspect-architecture-scopes
# (see: codeos-reviewer --help)
# To build: cargo build --release --manifest-path dba/04-tools/reviewer/engine/Cargo.toml
#
# Exit codes: 1 = not a git repo, 2 = binary not found. Other codes belong to the Rust binary
# (dba/04-tools/reviewer/engine/src/main.rs) and are passed through unchanged.
set -euo pipefail
# Preserve the original precondition: the shim requires the CALLER to be inside some git
# repository (a property of the project being reviewed) — unrelated to where the binary
# itself lives, so this check is deliberately kept separate from binary-path resolution
# below, running first, exactly as before this change.
git rev-parse --show-toplevel >/dev/null 2>&1 || { echo "error: not inside a git repository" >&2; exit 1; }

# Resolve the binary relative to this script's own physical location (following the
# nested .codeos/toolkit symlink when invoked from a downstream project), not the calling repo's git
# root — git rev-parse --show-toplevel from within a downstream project resolves to that
# project's own root, not through .codeos/toolkit to Codeos, which is where the binary actually
# lives. pwd -P (physical) is required, not plain pwd, to resolve through the symlink
# rather than preserve its logical name.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd -P)"
BINARY="${CODEOS_ROOT}/dba/04-tools/reviewer/engine/target/release/codeos-reviewer"
if [[ ! -x "${BINARY}" ]]; then
  command -v codeos-reviewer >/dev/null 2>&1 && BINARY="codeos-reviewer" || {
    echo "error: binary not found at ${BINARY} and not on PATH" >&2
    echo "       Build: cargo build --release --manifest-path ${CODEOS_ROOT}/dba/04-tools/reviewer/engine/Cargo.toml" >&2
    exit 2
  }
fi

exec "${BINARY}" "$@"
