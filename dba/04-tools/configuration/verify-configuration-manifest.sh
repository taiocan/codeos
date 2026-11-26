#!/usr/bin/env bash
# Regression guard against accidental in-place edits to approved DBA compositions and to the
# normative semantic closure each one froze at issuance (DQA WO-01B).
set -euo pipefail
ROOT="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)}"
cd "${ROOT}"
sha256sum --check --quiet dba/00-entry/configurations/SHA256SUMS

fail() { printf 'configuration integrity failed: %s\n' "$1" >&2; exit 1; }
configurations="$(awk '$2 ~ /\.yaml$/ {print $2}' dba/00-entry/configurations/SHA256SUMS)"
closures="$(mktemp -d)"
trap 'rm -rf "${closures}"' EXIT
# Closures are independent, so they are computed concurrently (CIFAST WO-05) and then compared in
# manifest order, so the first failure reported is the same as in a sequential run.
printf '%s\n' "${configurations}" | xargs -P "$(nproc)" -I{} \
  sh -c 'bash dba/04-tools/configuration/semantic-closure.sh . "$1" > "$2/$(basename "$1" .yaml)"' _ {} "${closures}" \
  || true
while IFS= read -r configuration; do
  name="$(basename "${configuration}" .yaml)"
  closure="dba/00-entry/configurations/closures/${name}.closure"
  grep -qF "  ${closure}" dba/00-entry/configurations/SHA256SUMS \
    || fail "${configuration} has no frozen semantic closure"
  diff -u "${closure}" "${closures}/${name}" \
    || fail "the semantic closure of ${configuration} changed"
done <<<"${configurations}"
printf 'configuration integrity OK\n'
