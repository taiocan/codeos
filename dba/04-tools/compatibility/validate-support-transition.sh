#!/usr/bin/env bash
# Validate one explicit support-state transition between CodeOS releases.
set -euo pipefail
[[ $# -eq 3 ]] || { echo 'usage: validate-support-transition.sh <old.json> <new.json> <new-release-id>' >&2; exit 2; }
OLD="$1" NEW="$2" RELEASE="$3"
fail() { printf 'compatibility support transition failed: %s\n' "$1" >&2; exit 1; }
while IFS=$'\t' read -r id before after; do
  case "${before}->${after}" in
    "supported->supported"|"supported->deprecated"|"deprecated->deprecated"|"deprecated->historical-only"|"historical-only->historical-only") ;;
    *) fail "${id} cannot transition ${before} -> ${after}" ;;
  esac
  if [[ "${after}" == deprecated ]]; then
    deprecated_in="$(jq -r --arg id "${id}" '.lifecycle[$id].deprecated_in // empty' "${NEW}")"
    migration_target="$(jq -r --arg id "${id}" '.lifecycle[$id].migration_target // empty' "${NEW}")"
    [[ -n "${deprecated_in}" && -n "${migration_target}" ]] || fail "${id} deprecation requires deprecated_in and migration_target"
  fi
  if [[ "${after}" == historical-only ]]; then
    [[ "${before}" == deprecated ]] || fail "${id} must pass through deprecated"
    deprecated_in="$(jq -r --arg id "${id}" '.lifecycle[$id].deprecated_in // empty' "${OLD}")"
    [[ -n "${deprecated_in}" && "${deprecated_in}" != "${RELEASE}" ]] || fail "${id} must remain deprecated for at least one release"
  fi
done < <(jq -r --slurpfile new "${NEW}" '.configurations|to_entries[]|[.key,.value,($new[0].configurations[.key] // "removed")]|@tsv' "${OLD}")
printf 'compatibility support transition OK\n'
