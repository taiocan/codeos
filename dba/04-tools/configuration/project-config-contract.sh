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

# All consumers use the canonical resolver. For a canonical downstream path, derive its project;
# for templates and test fixtures, the supplied file itself is still the resolution input.
if [[ "${CONFIG}" == */.codeos/00-project/codeos.yaml ]]; then
  PROJECT_ROOT="${CONFIG%/.codeos/00-project/codeos.yaml}"
else
  PROJECT_ROOT="${CODEOS_ROOT}"
fi
RESOLVED="$(bash "${HERE}/resolve-project-configuration.sh" "${PROJECT_ROOT}" "${CONFIG}")" || exit 1
CONFIGURATION_PATH="$(printf '%s\n' "${RESOLVED}" | sed -n 's/^configuration_path=//p')"
MECHANICS_POLICY_REL="$(printf '%s\n' "${RESOLVED}" | sed -n 's/^codeos_mechanics_policy=//p')"
[[ -n "${MECHANICS_POLICY_REL}" ]] || \
  fail "resolved configuration selects no codeos_mechanics_policy; codeos.yaml validation requires DBA-5 or later"
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

# --- runtime_verification: block -------------------------------------------
# Optional project facts used only to locate the runtime that mechanical checks observe. The
# profile never selects platform architecture or changes mechanic semantics.
if grep -q '^runtime_verification:' "${CONFIG}"; then
  mapfile -t runtime_lines < <(
    sed -n '/^runtime_verification:/,/^[a-z]/p' "${CONFIG}" |
      grep -E '^  [a-z_]+: ' || true
  )
  expected_keys=$'backend_health_url\ndatabase_service\ndatabase_test_url_env\nweb_url'
  if [[ "${MECHANICS_POLICY_REL}" == 'dba/02-policies/codeos-mechanics/v5.md' ]] && \
     printf '%s\n' "${runtime_lines[@]}" | grep -q '^  journey_guard_script:'; then
    expected_keys=$'backend_health_url\ndatabase_service\ndatabase_test_url_env\njourney_guard_script\nweb_url'
  fi
  actual_keys="$(printf '%s\n' "${runtime_lines[@]}" | sed -E 's/^  ([a-z_]+):.*/\1/' | LC_ALL=C sort)"
  [[ "${actual_keys}" == "${expected_keys}" ]] ||
    fail 'runtime_verification must contain exactly database_service, database_test_url_env, backend_health_url, and web_url'

  runtime_value() {
    local key="$1"
    printf '%s\n' "${runtime_lines[@]}" | sed -n "s/^  ${key}:[[:space:]]*//p" | head -n1
  }
  database_service="$(runtime_value database_service)"
  database_test_url_env="$(runtime_value database_test_url_env)"
  backend_health_url="$(runtime_value backend_health_url)"
  web_url="$(runtime_value web_url)"
  [[ -n "${database_service}" ]] || fail 'runtime_verification.database_service must be non-empty'
  [[ "${database_test_url_env}" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] ||
    fail 'runtime_verification.database_test_url_env must be an environment-variable name'
  [[ "${backend_health_url}" =~ ^https?://[^[:space:]]+$ ]] ||
    fail 'runtime_verification.backend_health_url must be an http(s) URL'
  [[ "${web_url}" =~ ^https?://[^[:space:]]+$ ]] ||
    fail 'runtime_verification.web_url must be an http(s) URL'
  if [[ "${MECHANICS_POLICY_REL}" == 'dba/02-policies/codeos-mechanics/v5.md' ]]; then
    journey_guard_script="$(runtime_value journey_guard_script)"
    if [[ -n "${journey_guard_script}" ]]; then
      [[ "${journey_guard_script}" != /* && "${journey_guard_script}" != *'..'* ]] ||
        fail 'runtime_verification.journey_guard_script must be a non-traversing project-relative path'
    fi
  fi
fi

# --- single-source-of-governance rule ----------------------------------------
# `governed:` is not a supported frontmatter key on any downstream artifact template — the
# artifacts: block above is the only authoritative source, so no template may restate it.
if grep -rn '^governed: ' "${CODEOS_ROOT}/dba/05-guidance/templates" >/dev/null 2>&1; then
  fail 'a template frontmatter restates governance with a governed: field'
fi

printf 'project config contract OK: %s\n' "${CONFIG#${CODEOS_ROOT}/}"
