#!/usr/bin/env bash
set -euo pipefail
MANIFEST="$1"
ROOT="${2:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)}"
fail() { printf 'compatibility support manifest failed: %s\n' "$1" >&2; exit 1; }
jq -e '.schema == 1 and (.configurations|type == "object") and (.lifecycle|type == "object") and (.consumers|type == "array")' "${MANIFEST}" >/dev/null \
  || fail 'invalid top-level schema'
jq -e '.consumers | all(.name and .repository and .commit and .bundle and .bundle_sha256 and .golden)' "${MANIFEST}" >/dev/null \
  || fail 'consumer pin is incomplete'
jq -e '(.replay_cases // []) | all(.name and .consumer and (.commit | test("^[0-9a-f]{40}$")) and .golden)' "${MANIFEST}" >/dev/null \
  || fail 'replay case is incomplete'
while IFS= read -r golden; do
  [[ -f "$(dirname "${MANIFEST}")/${golden}" ]] || fail "replay golden ${golden} is missing"
done < <(jq -r '(.replay_cases // [])[].golden' "${MANIFEST}")
while IFS=$'\t' read -r id state; do
  [[ "${state}" =~ ^(supported|deprecated|historical-only)$ ]] || fail "${id} has invalid state ${state}"
  [[ -f "${ROOT}/dba/00-entry/configurations/${id}.yaml" ]] || fail "${id} has no retained configuration"
  if [[ "${state}" != supported ]]; then
    jq -e --arg id "${id}" '.lifecycle[$id].deprecated_in|type == "string"' "${MANIFEST}" >/dev/null \
      || fail "${id} lacks deprecation metadata"
    jq -e --arg id "${id}" '.lifecycle[$id].migration_target|type == "string"' "${MANIFEST}" >/dev/null \
      || fail "${id} lacks migration guidance"
  fi
done < <(jq -r '.configurations|to_entries[]|[.key,.value]|@tsv' "${MANIFEST}")
while IFS= read -r path; do
  id="$(basename "${path}" .yaml)"
  jq -e --arg id "${id}" '.configurations[$id] != null' "${MANIFEST}" >/dev/null \
    || fail "retained configuration ${id} has no support state"
done < <(find "${ROOT}/dba/00-entry/configurations" -maxdepth 1 -name 'DBA-*.yaml' | sort -V)
printf 'compatibility support manifest OK\n'
