#!/usr/bin/env bash
# DQA WO-01B falsifiers: a change to any normative semantic dependency an issued configuration
# reaches, directly or indirectly, fails the integrity check; an advisory file stays free.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-closure.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
fail() { printf 'semantic closure tests failed: %s\n' "$1" >&2; exit 1; }

git -C "${ROOT}" ls-files -z --cached --others --exclude-standard | (cd "${ROOT}" && xargs -0 cp --parents -t "${WORK}")
git -C "${WORK}" init -q && git -C "${WORK}" add -A
closure() { bash "${WORK}/dba/04-tools/configuration/semantic-closure.sh" "${WORK}" "${WORK}/dba/00-entry/configurations/$1.yaml"; }
frozen() { cat "${WORK}/dba/00-entry/configurations/closures/$1.closure"; }
restore() { cp "${ROOT}/$1" "${WORK}/$1"; }

[[ -n "$(frozen DBA-28)" && "$(closure DBA-28)" == "$(frozen DBA-28)" ]] || fail 'the unmodified DBA-28 closure differs from its baseline'

# A glossary definition DBA-28's policies use without defining it themselves.
sed -i 's/^\(| \*\*Evidence Scope\*\* |\)/\1 Redefined./' "${WORK}/dba/05-guidance/terminology.md"
[[ "$(closure DBA-28)" != "$(frozen DBA-28)" ]] || fail 'a redefined Evidence Scope left DBA-28 unchanged'
if bash "${WORK}/dba/04-tools/configuration/verify-configuration-manifest.sh" "${WORK}" >/dev/null 2>&1; then
  fail 'the integrity check accepted a redefined term'
fi
restore dba/05-guidance/terminology.md

# A definition an issued configuration froze cannot be broadened in place for a later one (WO-06:
# v13 records may cite shared evidence, so v13 adds a new term instead).
sed -i 's/^\(| \*\*Verification record\*\* |\)/\1 Broadened./' "${WORK}/dba/05-guidance/terminology.md"
[[ "$(closure DBA-22)" != "$(frozen DBA-22)" ]] || fail 'a broadened Verification record left DBA-22 unchanged'
if bash "${WORK}/dba/04-tools/configuration/verify-configuration-manifest.sh" "${WORK}" >/dev/null 2>&1; then
  fail 'the integrity check accepted a broadened frozen term'
fi
restore dba/05-guidance/terminology.md

# A normative file DBA-7 reaches only by reference from its selected Reader Output policy.
printf '\nChanged.\n' >> "${WORK}/dba/05-guidance/reader-oriented-output.md"
[[ "$(closure DBA-7)" != "$(frozen DBA-7)" ]] || fail 'an indirect normative dependency of DBA-7 changed unnoticed'
restore dba/05-guidance/reader-oriented-output.md

# Advisory guidance and glossary growth outside the closure stay free.
printf '\nChanged.\n' >> "${WORK}/dba/05-guidance/patterns/rust-project-structure.md"
printf '| **Unused Fixture Term** | Only this test defines it. |\n' >> "${WORK}/dba/05-guidance/terminology.md"
[[ "$(closure DBA-28)" == "$(frozen DBA-28)" ]] || fail 'an advisory pattern or an unused term entered the DBA-28 closure'

printf 'semantic closure tests: PASS\n'
