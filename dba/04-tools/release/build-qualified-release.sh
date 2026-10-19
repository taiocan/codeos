#!/usr/bin/env bash
# Build the two governed executables and bind them to one qualified release manifest.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT="$(cd "${HERE}/../../.." && pwd -P)"
MANIFEST="${ROOT}/maintenance/release-manifest.json"
LOCK="${ROOT}/maintenance/release-lock.json"

fail() { printf 'qualified release failed: %s\n' "$1" >&2; exit 1; }
command -v jq >/dev/null || fail 'jq is required'
bash "${HERE}/release-inputs-clean.sh" "${ROOT}"

locked_rust="$(jq -r '.rust' "${LOCK}")"
locked_node="$(jq -r '.node' "${LOCK}")"
locked_npm="$(jq -r '.npm' "${LOCK}")"
[[ "$(rustc --version | awk '{print $2}')" == "${locked_rust}" ]] || fail 'Rust version differs from release lock'
[[ "$(node --version | sed 's/^v//')" == "${locked_node}" ]] || fail 'Node version differs from release lock'
[[ "$(npm --version)" == "${locked_npm}" ]] || fail 'npm version differs from release lock'

# Hash the committed release inputs, excluding the generated manifest to avoid a circular identity.
source_sha="$({
  git -C "${ROOT}" ls-files -z | while IFS= read -r -d '' path; do
    [[ "${path}" == 'maintenance/release-manifest.json' ]] && continue
    printf '%s\0' "${path}"
    sha256sum "${ROOT}/${path}" | awk '{print $1}'
  done
} | sha256sum | awk '{print $1}')"
release_id="codeos-${source_sha:0:16}"

export CODEOS_RELEASE_ID="${release_id}"
export CODEOS_RELEASE_SOURCE_SHA256="${source_sha}"
cargo build --release --locked --manifest-path "${ROOT}/dba/04-tools/workflow/engine/Cargo.toml"
cargo build --release --locked --manifest-path "${ROOT}/dba/04-tools/reviewer/engine/Cargo.toml"

workflow="${ROOT}/dba/04-tools/workflow/engine/target/release/codeos-workflow"
reviewer="${ROOT}/dba/04-tools/reviewer/engine/target/release/codeos-reviewer"
generated="$(mktemp)"
trap 'rm -f "${generated}"' EXIT
jq -n \
  --arg release_id "${release_id}" \
  --arg source_sha256 "${source_sha}" \
  --arg workflow_sha256 "$(sha256sum "${workflow}" | awk '{print $1}')" \
  --arg reviewer_sha256 "$(sha256sum "${reviewer}" | awk '{print $1}')" \
  --arg rust "${locked_rust}" --arg node "${locked_node}" --arg npm "${locked_npm}" \
  --arg codex_cli "$(jq -r '.codex_cli' "${LOCK}")" \
  --arg reviewer_model "$(jq -r '.reviewer_model' "${LOCK}")" \
  --arg reviewer_reasoning_effort "$(jq -r '.reviewer_reasoning_effort' "${LOCK}")" \
  '{schema:1, release_id:$release_id, source_sha256:$source_sha256,
    binaries:{"codeos-workflow":{sha256:$workflow_sha256,build_id:$source_sha256},
              "codeos-reviewer":{sha256:$reviewer_sha256,build_id:$source_sha256}},
    toolchains:{rust:$rust,node:$node,npm:$npm},
    reviewer:{codex_cli:$codex_cli,model:$reviewer_model,reasoning_effort:$reviewer_reasoning_effort}}' \
  > "${generated}"
mv "${generated}" "${MANIFEST}"
trap - EXIT
printf 'qualified release: %s\nmanifest: %s\n' "${release_id}" "${MANIFEST}"
