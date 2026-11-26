#!/usr/bin/env bash
# The one quality gate CI and local runs share, so a local PASS predicts the CI result (CIFAST).
#   fast  dependency preflight, format, layout contract, clippy
#   full  fast, then the test suite, dependency audit and compatibility replay
# Release builds stay in dba/04-tools/release/ and are not part of this gate.
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"
MODE="${1:-}"
[[ "${MODE}" == fast || "${MODE}" == full ]] || {
  printf 'usage: %s fast|full\n' "$0" >&2
  exit 2
}

# Pinned versions CI installs; a local mismatch would let local and CI verdicts differ.
RUST_VERSION=1.93.0
NODE_VERSION=v20.19.2
NPM_VERSION=9.2.0
CARGO_AUDIT_VERSION=0.23.0
ENGINES=(dba/04-tools/workflow/engine dba/04-tools/reviewer/engine)

missing=()
need() { command -v "$1" >/dev/null 2>&1 || missing+=("$1${2:+ ($2)}"); }
version() { # name expected actual
  [[ "$3" == "$2" ]] || missing+=("$1 $2 (found ${3:-none})")
}

preflight() {
  for tool in git rg jq python3 sha256sum tar awk sed cargo; do need "${tool}"; done
  if command -v rustc >/dev/null 2>&1; then
    version rustc "${RUST_VERSION}" "$(cd "${CODEOS_ROOT}" && rustc --version | awk '{print $2}')"
  else
    need rustc "${RUST_VERSION}"
  fi
  (cd "${CODEOS_ROOT}" && cargo fmt --version >/dev/null 2>&1) || missing+=("rustfmt (rustup component)")
  (cd "${CODEOS_ROOT}" && cargo clippy --version >/dev/null 2>&1) || missing+=("clippy (rustup component)")
  if [[ "${MODE}" == full ]]; then
    if command -v node >/dev/null 2>&1; then
      version node "${NODE_VERSION}" "$(node --version)"
    else
      need node "${NODE_VERSION}"
    fi
    if command -v npm >/dev/null 2>&1; then
      version npm "${NPM_VERSION}" "$(npm --version)"
    else
      need npm "${NPM_VERSION}"
    fi
    if (cd "${CODEOS_ROOT}" && cargo audit --version >/dev/null 2>&1); then
      version cargo-audit "${CARGO_AUDIT_VERSION}" "$(cd "${CODEOS_ROOT}" && cargo audit --version | awk '{print $2}')"
    else
      need cargo-audit "${CARGO_AUDIT_VERSION}"
    fi
  fi
  if ((${#missing[@]})); then
    printf 'gate preflight FAIL: missing or mismatched dependencies:\n' >&2
    printf '  - %s\n' "${missing[@]}" >&2
    exit 1
  fi
  printf 'gate preflight: OK (%s)\n' "${MODE}"
}

step() {
  printf '\n== %s\n' "$1"
  shift
  "$@"
}

cd "${CODEOS_ROOT}"
preflight
for engine in "${ENGINES[@]}"; do
  step "format ${engine}" cargo fmt --manifest-path "${engine}/Cargo.toml" --check
done
step "layout contract" bash dba/04-tools/configuration/layout-contract.sh
for engine in "${ENGINES[@]}"; do
  step "clippy ${engine}" cargo clippy --manifest-path "${engine}/Cargo.toml" --all-targets -- -D warnings
done
if [[ "${MODE}" == full ]]; then
  step "test suite" bash dba/04-tools/tests/run.sh
  for engine in "${ENGINES[@]}"; do
    step "dependency audit ${engine}" cargo audit --file "${engine}/Cargo.lock"
  done
  step "compatibility" bash dba/04-tools/compatibility/run.sh
fi
printf '\ngate %s: PASS\n' "${MODE}"
