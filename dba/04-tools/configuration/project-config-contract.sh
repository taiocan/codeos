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
# The fixed mechanics block is a transparency view of the selected Codeos Mechanics policy, which is
# its single authoritative source. When the block is present it must be the EXACT set the policy
# defines: every policy mechanic with its exact value, and no additional fixed mechanic. A v1
# project must not silently carry (or display) a v2-only mechanic such as `data_integrity`.

# Canonicalize a YAML mechanics block (from the policy renderer or a real codeos.yaml) to sorted
# `group:name:value` lines, ignoring comments, blank lines, and trailing whitespace.
mechanics_triples() {
  awk '
    { sub(/[[:space:]]+$/, "") }
    /^[[:space:]]*#/ || /^[[:space:]]*$/ { next }
    /^  [a-z_]+:$/            { group = $1; sub(/:$/, "", group); next }
    /^    [a-z_]+: [a-z_]+$/  { name = $1; sub(/:$/, "", name); print group ":" name ":" $2 }
  ' | LC_ALL=C sort
}

if grep -q '^mechanics:' "${CONFIG}"; then
  expected_triples="$(bash "${HERE}/render-mechanics-block.sh" "${MECHANICS_POLICY}" | mechanics_triples)"
  [[ -n "${expected_triples}" ]] || fail 'selected Codeos Mechanics policy renders no mechanics'
  # The config's block runs from `^mechanics:` to the line before the next top-level key.
  actual_triples="$(sed -n '/^mechanics:/,/^[a-z]/p' "${CONFIG}" | sed '/^[a-z]/d' | mechanics_triples)"
  if [[ "${actual_triples}" != "${expected_triples}" ]]; then
    diff_out="$(diff <(printf '%s\n' "${expected_triples}") <(printf '%s\n' "${actual_triples}") || true)"
    fail "mechanics: block is not the exact set the selected policy defines${diff_out:+ ($(printf '%s' "${diff_out}" | tr '\n' ' '))}"
  fi
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
