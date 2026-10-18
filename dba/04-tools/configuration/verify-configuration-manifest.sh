#!/usr/bin/env bash
# Regression guard against accidental in-place edits to approved DBA compositions and to the
# normative semantic closure each one froze at issuance (DQA WO-01B).
set -euo pipefail
ROOT="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)}"
cd "${ROOT}"
sha256sum --check --quiet dba/00-entry/configurations/SHA256SUMS

fail() { printf 'configuration integrity failed: %s\n' "$1" >&2; exit 1; }
while IFS= read -r configuration; do
  closure="dba/00-entry/configurations/closures/$(basename "${configuration}" .yaml).closure"
  grep -qF "  ${closure}" dba/00-entry/configurations/SHA256SUMS \
    || fail "${configuration} has no frozen semantic closure"
  diff -u "${closure}" <(bash dba/04-tools/configuration/semantic-closure.sh . "${configuration}") \
    || fail "the semantic closure of ${configuration} changed"
done < <(awk '$2 ~ /\.yaml$/ {print $2}' dba/00-entry/configurations/SHA256SUMS)
printf 'configuration integrity OK\n'
