#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
RESOLVER="${ROOT}/dba/04-tools/configuration/resolve-project-configuration.sh"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-resolve.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
fail() { printf 'project configuration resolution tests failed: %s\n' "$1" >&2; exit 1; }

mkdir -p "${WORK}/.codeos/00-project"

legacy="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'resolution_mode=legacy-dba-7-cutover' <<<"${legacy}" || fail 'schema-less mode changed'
grep -Fxq 'dba_configuration=DBA-7' <<<"${legacy}" || fail 'schema-less project did not resolve to DBA-7'

printf 'codeos_schema: 2\ndba_configuration: DBA-8\n' > "${WORK}/.codeos/00-project/codeos.yaml"
explicit="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'resolution_mode=explicit' <<<"${explicit}" || fail 'schema-2 mode is not explicit'
grep -Fxq 'reader_output_policy=dba/02-policies/reader-output/v3.md' <<<"${explicit}" || fail 'DBA-8 did not select Reader Output v3'

printf 'codeos_schema: 2\n' > "${WORK}/.codeos/00-project/codeos.yaml"
if bash "${RESOLVER}" "${WORK}" >/dev/null 2>&1; then fail 'missing schema-2 binding was accepted'; fi

printf 'codeos_schema: 2\ndba_configuration: DBA-999\n' > "${WORK}/.codeos/00-project/codeos.yaml"
if bash "${RESOLVER}" "${WORK}" >/dev/null 2>&1; then fail 'unknown binding was accepted'; fi

printf 'dba_configuration: DBA-8\n' > "${WORK}/.codeos/00-project/codeos.yaml"
if bash "${RESOLVER}" "${WORK}" >/dev/null 2>&1; then fail 'binding without schema was accepted'; fi

printf 'project configuration resolution tests: PASS\n'
