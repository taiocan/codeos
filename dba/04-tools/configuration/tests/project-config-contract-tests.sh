#!/usr/bin/env bash
# Prove project-config-contract.sh derives its mechanics-policy version from the active
# configuration and validates the fixed `mechanics:` block as the EXACT set that policy defines:
#
#   DBA-5 + exact v1 mechanics            -> PASS
#   DBA-5 + a v2-only mechanic            -> FAIL (no extra fixed mechanic)
#   DBA-6 + exact v2 mechanics            -> PASS
#   DBA-6 without data_integrity          -> FAIL (missing a policy mechanic)
#   an existing DBA-5 project, unchanged  -> PASS
#
# A fake toolkit root is required because the script derives its own root from its location.
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-cfg.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
fail() { printf 'project config contract tests failed: %s\n' "$1" >&2; exit 1; }

CFG_DIR="${CODEOS_ROOT}/dba/04-tools/configuration"

# A fake toolkit root whose active configuration selects codeos-mechanics <mechver>.
mk_fake_root() {
  local root="$1" mechver="$2"
  mkdir -p "${root}/dba/04-tools/configuration" \
           "${root}/dba/00-entry/configurations" \
           "${root}/dba/02-policies/codeos-mechanics"
  cp "${CFG_DIR}/project-config-contract.sh" "${CFG_DIR}/render-mechanics-block.sh" \
     "${root}/dba/04-tools/configuration/"
  cp "${CODEOS_ROOT}/dba/02-policies/codeos-mechanics/${mechver}.md" \
     "${root}/dba/02-policies/codeos-mechanics/${mechver}.md"
  printf 'codeos_mechanics_policy: dba/02-policies/codeos-mechanics/%s.md\n' "${mechver}" \
    > "${root}/dba/00-entry/configurations/DBA-X.yaml"
  printf 'Active configuration: `.codeos/toolkit/dba/00-entry/configurations/DBA-X.yaml`\n' \
    > "${root}/dba-system.md"
}

# Wrap a mechanics block (on stdin) into a complete codeos.yaml.
wrap_config() {
  local out="$1"
  {
    printf 'platform:\n  persistence: postgresql\n  backend: rust\n  webapp: svelte\n  runtime: docker\n\n'
    cat
    printf '\nartifacts:\n  charter: governed\n  intent: governed\n  contract: governed\n  event_schema: governed\n  architecture: governed\n'
  } > "${out}"
}

check() { bash "$1/dba/04-tools/configuration/project-config-contract.sh" "$2" >/dev/null 2>&1; }

FAKE_V1="${WORK}/root-v1"; mk_fake_root "${FAKE_V1}" v1
FAKE_V2="${WORK}/root-v2"; mk_fake_root "${FAKE_V2}" v2

# 0. Shipped template validates against the repo's own active configuration.
bash "${CFG_DIR}/project-config-contract.sh" >/dev/null \
  || fail 'shipped template does not validate against the active configuration'

# 1. DBA-5 + exact v1 mechanics -> PASS.
bash "${CFG_DIR}/render-mechanics-block.sh" "${CODEOS_ROOT}/dba/02-policies/codeos-mechanics/v1.md" \
  | wrap_config "${WORK}/v1-exact.yaml"
check "${FAKE_V1}" "${WORK}/v1-exact.yaml" \
  || fail 'DBA-5 + exact v1 mechanics was rejected'

# 2. DBA-5 + a v2-only mechanic (data_integrity) -> FAIL.
sed 's/^\(    repeatability: always\)$/\1\n    data_integrity: always_when_persistence/' \
  "${WORK}/v1-exact.yaml" > "${WORK}/v1-plus-di.yaml"
if check "${FAKE_V1}" "${WORK}/v1-plus-di.yaml"; then
  fail 'DBA-5 config carrying the v2-only data_integrity mechanic was accepted'
fi

# 3. DBA-6 + exact v2 mechanics -> PASS.
bash "${CFG_DIR}/render-mechanics-block.sh" "${CODEOS_ROOT}/dba/02-policies/codeos-mechanics/v2.md" \
  | wrap_config "${WORK}/v2-exact.yaml"
check "${FAKE_V2}" "${WORK}/v2-exact.yaml" \
  || fail 'DBA-6 + exact v2 mechanics was rejected'

# 4. DBA-6 without data_integrity -> FAIL.
sed '/data_integrity/d' "${WORK}/v2-exact.yaml" > "${WORK}/v2-no-di.yaml"
if check "${FAKE_V2}" "${WORK}/v2-no-di.yaml"; then
  fail 'DBA-6 config missing data_integrity was accepted'
fi

# 5. An existing DBA-5 project (exact v1 block, comments and all) remains valid unchanged.
check "${FAKE_V1}" "${WORK}/v1-exact.yaml" \
  || fail 'an existing DBA-5-shaped codeos.yaml no longer validates against a v1-selecting configuration'

# 6. The exact v1 block must NOT satisfy a v2-selecting configuration (it lacks data_integrity),
#    and the exact v2 block must NOT satisfy a v1-selecting one (extra fixed mechanic).
if check "${FAKE_V2}" "${WORK}/v1-exact.yaml"; then
  fail 'a v1 mechanics block validated against a v2-selecting configuration'
fi
if check "${FAKE_V1}" "${WORK}/v2-exact.yaml"; then
  fail 'a v2 mechanics block validated against a v1-selecting configuration'
fi

printf 'project config contract tests: PASS\n'
