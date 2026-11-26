#!/usr/bin/env bash
# One thin entrypoint over capability-owned suites.
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"

bash "${CODEOS_ROOT}/dba/04-tools/configuration/layout-contract.sh"
bash "${CODEOS_ROOT}/dba/04-tools/configuration/guidance-contract.sh"
bash "${CODEOS_ROOT}/dba/04-tools/configuration/project-config-contract.sh"
bash "${CODEOS_ROOT}/dba/04-tools/configuration/tests/project-config-contract-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/configuration/tests/project-configuration-resolution-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/configuration/verify-configuration-manifest.sh"
bash "${CODEOS_ROOT}/dba/04-tools/configuration/tests/semantic-closure-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/compatibility/validate-support-manifest.sh" \
  "${CODEOS_ROOT}/dba/04-tools/compatibility/support.json" "${CODEOS_ROOT}"
bash "${CODEOS_ROOT}/dba/04-tools/compatibility/tests/support-lifecycle-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/release/tests/release-identity-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/release/tests/candidate-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/initializer/tests/dba-init-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/reviewer/tests/codeos-review-wrapper.sh"
bash "${CODEOS_ROOT}/dba/04-tools/reviewer/tests/codeos-review-deepseek-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/architecture-migration/tests/architecture-scope-migration.sh"
bash "${CODEOS_ROOT}/dba/04-tools/implementer/tests/codeos-implement-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/learning/tests/codeos-learning-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/monitor/tests/codeos-monitor-snapshot-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/monitor/tests/codeos-toolkit-analytics-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/monitor/tests/codeos-static-analytics-tests.sh"
# The engine suites are independent and mostly wait on git and subprocesses, so they run
# concurrently (CIFAST WO-05). The reviewer output is printed after the workflow output.
reviewer_log="$(mktemp)"
trap 'rm -f "${reviewer_log}"' EXIT
cargo test --manifest-path "${CODEOS_ROOT}/dba/04-tools/reviewer/engine/Cargo.toml" >"${reviewer_log}" 2>&1 &
reviewer_pid=$!
workflow_status=0
cargo test --manifest-path "${CODEOS_ROOT}/dba/04-tools/workflow/engine/Cargo.toml" || workflow_status=$?
reviewer_status=0
wait "${reviewer_pid}" || reviewer_status=$?
cat "${reviewer_log}"
if ((workflow_status || reviewer_status)); then
  printf '04-tools tests: FAIL (workflow engine exit %s, reviewer engine exit %s)\n' \
    "${workflow_status}" "${reviewer_status}" >&2
  exit 1
fi

printf '04-tools tests: PASS\n'
