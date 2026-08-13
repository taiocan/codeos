#!/usr/bin/env bash
# Verify the minimum responsibility boundary for components selected by a candidate DBA config.
# Run: bash dba/04-tools/configuration/dba-config-boundaries.sh dba/00-entry/configurations/DBA-N.yaml
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${HERE}/../../.." && pwd -P)"
fail() {
  printf 'DBA boundary check failed: %s\n' "$1" >&2
  exit 1
}

[[ $# -eq 1 ]] || fail "expected one candidate configuration path"

case "$1" in
  /*) CONFIG="$1" ;;
  *)  CONFIG="${CODEOS_ROOT}/$1" ;;
esac

[[ -f "${CONFIG}" ]] || fail "candidate configuration does not exist: $1"

mapfile -t COMPONENTS < <(
  sed -n 's/^\([a-z_][a-z0-9_]*\):[[:space:]]*\([^[:space:]#][^[:space:]#]*\.md\)[[:space:]]*$/\1|\2/p' "${CONFIG}"
)

[[ ${#COMPONENTS[@]} -gt 0 ]] || fail "candidate configuration selects no Markdown components: $1"

checked=0

for selection in "${COMPONENTS[@]}"; do
  key="${selection%%|*}"
  path="${selection#*|}"
  component="${CODEOS_ROOT}/${path}"

  [[ -f "${component}" ]] || fail "${key} selects missing file: ${path}"

  mapfile -t header < <(sed -n '1,4p' "${component}")
  [[ ${#header[@]} -eq 4 ]] || fail "${key} (${path}) has no complete four-line boundary contract"
  [[ "${header[0]}" == '---' ]] || fail "${key} (${path}) boundary must start on line 1"
  [[ "${header[1]}" == 'component_question: '* ]] || fail "${key} (${path}) is missing component_question"
  [[ -n "${header[1]#component_question: }" ]] || fail "${key} (${path}) has an empty component_question"
  [[ "${header[2]}" == 'out_of_scope: '* ]] || fail "${key} (${path}) is missing out_of_scope"
  [[ -n "${header[2]#out_of_scope: }" ]] || fail "${key} (${path}) has an empty out_of_scope"
  [[ "${header[3]}" == '---' ]] || fail "${key} (${path}) has unsupported boundary metadata"

  checked=$((checked + 1))
done

printf 'DBA boundary contract OK: %s (%d checked)\n' "$1" "${checked}"
