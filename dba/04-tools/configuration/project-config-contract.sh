#!/usr/bin/env bash
# Verify a codeos.yaml project configuration and the single-frontmatter-governance rule it makes
# authoritative. Defaults to the shipped template; pass a path to check a real project's file.
# Run: bash dba/04-tools/configuration/project-config-contract.sh [path/to/codeos.yaml]
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${HERE}/../../.." && pwd -P)"
fail() { printf 'project config contract failed: %s\n' "$1" >&2; exit 1; }

CONFIG="${1:-${CODEOS_ROOT}/dba/05-guidance/templates/codeos.yaml}"
[[ -f "${CONFIG}" ]] || fail "configuration does not exist: ${CONFIG}"

# The mechanics-policy version is derived from the active configuration, not hardcoded, so a new
# DBA configuration selecting a newer Codeos Mechanics policy is validated against that version
# while an older configuration keeps validating against the one it selects.
ACTIVE_CONFIG_REL="$(sed -n 's#^Active configuration: `\.codeos/toolkit/\(.*\)`$#\1#p' "${CODEOS_ROOT}/dba-system.md")"
[[ -n "${ACTIVE_CONFIG_REL}" && -f "${CODEOS_ROOT}/${ACTIVE_CONFIG_REL}" ]] || \
  fail "cannot resolve the active configuration from dba-system.md"
MECHANICS_POLICY_REL="$(sed -n 's#^codeos_mechanics_policy:[[:space:]]*\([^[:space:]#]*\.md\).*#\1#p' "${CODEOS_ROOT}/${ACTIVE_CONFIG_REL}")"
[[ -n "${MECHANICS_POLICY_REL}" ]] || \
  fail "active configuration selects no codeos_mechanics_policy; codeos.yaml validation requires DBA-5 or later"
MECHANICS_POLICY="${CODEOS_ROOT}/${MECHANICS_POLICY_REL}"
[[ -f "${MECHANICS_POLICY}" ]] || fail "Codeos Mechanics policy is missing: ${MECHANICS_POLICY_REL}"

# --- artifacts: block -------------------------------------------------------
# Core-four governance is locked regardless of what the file says elsewhere; verify it reads
# exactly "governed" and reject a configuration that tries to loosen it.
for core in charter intent contract event_schema; do
  line="$(grep -E "^  ${core}: " "${CONFIG}" || true)"
  [[ -n "${line}" ]] || fail "artifacts: block omits core-four type: ${core}"
  [[ "${line}" == "  ${core}: governed" ]] || \
    fail "core-four artifact type is not locked governed: ${core}"
done

mapfile -t artifact_lines < <(sed -n '/^artifacts:/,/^[a-z]/p' "${CONFIG}" | grep -E '^  [a-z_]+: [^ ]+$')
[[ ${#artifact_lines[@]} -gt 0 ]] || fail 'artifacts: block has no entries'
for line in "${artifact_lines[@]}"; do
  value="${line##*: }"
  [[ "${value}" == "governed" || "${value}" == "nongoverned" ]] || \
    fail "artifacts: entry has an unsupported value: ${line}"
done

# --- mechanics: block -------------------------------------------------------
# Derived from the selected Codeos Mechanics policy, not hardcoded here, so the fixed set has one
# authoritative source. A configuration's mechanics block, if present, must match it exactly.
mapfile -t expected_mechanics < <(
  awk '
    /^### Delivery$/    { group = "delivery" }
    /^### Validation$/  { group = "validation" }
    /^### Communication$/ { group = "communication" }
    /^\| `[a-z_]+` \|/ {
      line = $0
      gsub(/^\| `/, "", line)
      split(line, parts, /` \| /)
      name = parts[1]
      split(parts[2], rest, / \|/)
      applies = rest[1]
      print group ":" name ":" applies
    }
  ' "${MECHANICS_POLICY}"
)
[[ ${#expected_mechanics[@]} -gt 0 ]] || fail 'Codeos Mechanics policy declares no mechanics'

if grep -q '^mechanics:' "${CONFIG}"; then
  for entry in "${expected_mechanics[@]}"; do
    group="${entry%%:*}"
    rest="${entry#*:}"
    name="${rest%%:*}"
    applies="${rest#*:}"
    mechanics_block="$(sed -n '/^mechanics:/,/^[a-z]/p' "${CONFIG}")"
    group_block="$(printf '%s\n' "${mechanics_block}" | sed -n "/^  ${group}:/,/^  [a-z]/p")"
    printf '%s\n' "${group_block}" | grep -qE "^    ${name}: ${applies}\$" || \
      fail "mechanics.${group}.${name} does not match the fixed policy value (${applies})"
  done
fi

# --- platform: block ---------------------------------------------------------
for tier in persistence backend webapp runtime; do
  line="$(sed -n "/^platform:/,/^[a-z]/p" "${CONFIG}" | grep -E "^  ${tier}: " | head -n1 || true)"
  [[ -n "${line}" ]] || fail "platform: block omits tier: ${tier}"
  if [[ "${line}" == *"not-applicable"* ]]; then
    [[ "${line}" == *"not-applicable: "?* ]] || \
      fail "platform.${tier} is marked not-applicable with no recorded reason"
  fi
done

# --- single-source-of-governance rule ----------------------------------------
# `governed:` is not a supported frontmatter key on any downstream artifact template — the
# artifacts: block above is the only authoritative source, so no template may restate it.
if grep -rn '^governed: ' "${CODEOS_ROOT}/dba/05-guidance/templates" >/dev/null 2>&1; then
  fail 'a template frontmatter restates governance with a governed: field'
fi

printf 'project config contract OK: %s\n' "${CONFIG#${CODEOS_ROOT}/}"
