#!/usr/bin/env bash
# Prove project-config-contract.sh derives its mechanics-policy version from the active
# configuration: a config selecting codeos-mechanics v1 validates a v1 codeos.yaml, and a config
# selecting v2 validates a v2 codeos.yaml (with data_integrity). A fake toolkit root is required
# because the script derives its own root from its location.
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-cfg.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
fail() { printf 'project config contract tests failed: %s\n' "$1" >&2; exit 1; }

SCRIPT="${CODEOS_ROOT}/dba/04-tools/configuration/project-config-contract.sh"

# 1. Real toolkit, active config selects v1: the shipped template validates.
bash "${SCRIPT}" >/dev/null || fail 'shipped template does not validate against the active configuration'

# 2. Fake root whose active configuration selects codeos-mechanics v2.
mk_fake_root() {
  local root="$1" mechver="$2"
  mkdir -p "${root}/dba/04-tools/configuration" \
           "${root}/dba/00-entry/configurations" \
           "${root}/dba/02-policies/codeos-mechanics" \
           "${root}/dba/05-guidance/templates"
  cp "${SCRIPT}" "${root}/dba/04-tools/configuration/project-config-contract.sh"
  cp "${CODEOS_ROOT}/dba/02-policies/codeos-mechanics/${mechver}.md" \
     "${root}/dba/02-policies/codeos-mechanics/${mechver}.md"
  printf 'codeos_mechanics_policy: dba/02-policies/codeos-mechanics/%s.md\n' "${mechver}" \
    > "${root}/dba/00-entry/configurations/DBA-X.yaml"
  printf 'Active configuration: `.codeos/toolkit/dba/00-entry/configurations/DBA-X.yaml`\n' \
    > "${root}/dba-system.md"
}

FAKE_V2="${WORK}/root-v2"
mk_fake_root "${FAKE_V2}" v2
cat > "${FAKE_V2}/dba/05-guidance/templates/codeos.yaml" <<'EOF'
platform:
  persistence: postgresql
  backend: rust
  webapp: svelte
  runtime: docker

mechanics:
  delivery:
    vertical_slice: always
    early_gui_preview: always_when_gui
  validation:
    smoke: always
    behavior: always
    repeatability: always
    data_integrity: always_when_persistence
    playwright: always_when_gui
    human_ux: always_when_gui
  communication:
    summary_first: always
    reader_oriented_writing: always
    terminology_consistency: always

artifacts:
  charter: governed
  intent: governed
  contract: governed
  event_schema: governed
  architecture: governed
EOF
bash "${FAKE_V2}/dba/04-tools/configuration/project-config-contract.sh" >/dev/null \
  || fail 'a v2 codeos.yaml does not validate when the active configuration selects v2'

# 3. Same v2-selecting root, but the codeos.yaml is missing data_integrity: must fail closed.
sed '/data_integrity/d' "${FAKE_V2}/dba/05-guidance/templates/codeos.yaml" \
  > "${WORK}/no-di.yaml"
if bash "${FAKE_V2}/dba/04-tools/configuration/project-config-contract.sh" "${WORK}/no-di.yaml" >/dev/null 2>&1; then
  fail 'a v2 codeos.yaml missing data_integrity was accepted'
fi

# 4. The shipped v1 template must NOT validate against a v2-selecting root (it lacks data_integrity).
if bash "${FAKE_V2}/dba/04-tools/configuration/project-config-contract.sh" \
     "${CODEOS_ROOT}/dba/05-guidance/templates/codeos.yaml" >/dev/null 2>&1; then
  fail 'the v1 template validated against a v2-selecting configuration'
fi

printf 'project config contract tests: PASS\n'
