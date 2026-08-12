#!/usr/bin/env bash
# Verify that supported Codeos paths no longer consume Controlled Plain English machinery.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${HERE}/../.." && pwd -P)"
TEST_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/codeos-cpe-retirement.XXXXXX")"
trap 'rm -rf "${TEST_ROOT}"' EXIT

fail() {
  printf 'CPE retirement check failed: %s\n' "$1" >&2
  exit 1
}

SUPPORTED_PATHS=(
  "${CODEOS_ROOT}/dba-system.md"
  "${CODEOS_ROOT}/dba/configurations/DBA-2.yaml"
  "${CODEOS_ROOT}/dba/tools/reviewer/v2.md"
  "${CODEOS_ROOT}/docs/reviewer-pipeline.md"
  "${CODEOS_ROOT}/prompts"
  "${CODEOS_ROOT}/scripts/codeos-review.sh"
  "${CODEOS_ROOT}/scripts/dba-init.sh"
  "${CODEOS_ROOT}/templates"
)

if rg -n -S \
  'Controlled Plain English|controlled-plain-english|controlled_plain_english|writing-discipline|codeos-cpe-status|CPE_STATUS|CPE_CONFIG|Layer [ABCD][12]?' \
  "${SUPPORTED_PATHS[@]}"; then
  fail "a supported path still references retired CPE machinery"
fi

[[ ! -e "${CODEOS_ROOT}/config/writing-discipline.yaml" ]] || fail "self-development status file remains"
[[ ! -e "${CODEOS_ROOT}/patterns/controlled-plain-english.md" ]] || fail "CPE pattern remains"
[[ ! -e "${CODEOS_ROOT}/dba/policies/controlled-plain-english/v1.md" ]] || fail "CPE policy remains"

PROJECT_ROOT="${TEST_ROOT}/project"
mkdir -p "${PROJECT_ROOT}"
(
  cd "${PROJECT_ROOT}"
  bash "${CODEOS_ROOT}/scripts/dba-init.sh" example >/dev/null
)

[[ -f "${PROJECT_ROOT}/CLAUDE.md" ]] || fail "initializer did not create CLAUDE.md"
[[ -f "${PROJECT_ROOT}/AGENTS.md" ]] || fail "initializer did not create AGENTS.md"
cmp -s "${PROJECT_ROOT}/AGENTS.md" "${CODEOS_ROOT}/templates/project-AGENTS.md" || \
  fail "generated AGENTS.md differs from its template"
[[ ! -e "${PROJECT_ROOT}/architecture/controlled-plain-english.yaml" ]] || \
  fail "initializer created a retired CPE status file"

printf 'owned CLAUDE\n' > "${PROJECT_ROOT}/CLAUDE.md"
printf 'owned AGENTS\n' > "${PROJECT_ROOT}/AGENTS.md"
(
  cd "${PROJECT_ROOT}"
  bash "${CODEOS_ROOT}/scripts/dba-init.sh" example >/dev/null
)
[[ "$(cat "${PROJECT_ROOT}/CLAUDE.md")" == "owned CLAUDE" ]] || fail "initializer overwrote CLAUDE.md"
[[ "$(cat "${PROJECT_ROOT}/AGENTS.md")" == "owned AGENTS" ]] || fail "initializer overwrote AGENTS.md"

WRAPPER_ROOT="${TEST_ROOT}/toolkit"
STUB_BIN="${TEST_ROOT}/bin"
mkdir -p "${WRAPPER_ROOT}/scripts" "${STUB_BIN}" "${PROJECT_ROOT}/architecture"
cp "${CODEOS_ROOT}/scripts/codeos-review.sh" "${WRAPPER_ROOT}/scripts/codeos-review.sh"
printf 'not: valid\n' > "${PROJECT_ROOT}/architecture/controlled-plain-english.yaml"
cat > "${STUB_BIN}/codeos-reviewer" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$@" > "${REVIEW_ARGS_FILE}"
exit 23
EOF
chmod +x "${STUB_BIN}/codeos-reviewer"

set +e
(
  cd "${PROJECT_ROOT}"
  PATH="${STUB_BIN}:${PATH}" REVIEW_ARGS_FILE="${TEST_ROOT}/review-args" \
    "${WRAPPER_ROOT}/scripts/codeos-review.sh" review demo 1 artifact.md
)
wrapper_status=$?
set -e

[[ ${wrapper_status} -eq 23 ]] || fail "wrapper did not delegate the reviewer's exit status"
mapfile -t review_args < "${TEST_ROOT}/review-args"
[[ ${#review_args[@]} -eq 4 ]] || fail "wrapper injected an unexpected argument"
[[ "${review_args[*]}" == "review demo 1 artifact.md" ]] || fail "wrapper changed reviewer arguments"

printf 'CPE retirement check OK\n'
