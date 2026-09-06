#!/usr/bin/env bash
# One thin entrypoint over capability-owned suites.
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"

bash "${CODEOS_ROOT}/dba/04-tools/configuration/layout-contract.sh"
bash "${CODEOS_ROOT}/dba/04-tools/configuration/guidance-contract.sh"
bash "${CODEOS_ROOT}/dba/04-tools/configuration/project-config-contract.sh"
bash "${CODEOS_ROOT}/dba/04-tools/configuration/tests/project-config-contract-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/initializer/tests/dba-init-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/reviewer/tests/codeos-review-wrapper.sh"
bash "${CODEOS_ROOT}/dba/04-tools/reviewer/tests/codeos-review-deepseek-tests.sh"
bash "${CODEOS_ROOT}/dba/04-tools/architecture-migration/tests/architecture-scope-migration.sh"
bash "${CODEOS_ROOT}/dba/04-tools/implementer/tests/codeos-implement-tests.sh"
cargo test --manifest-path "${CODEOS_ROOT}/dba/04-tools/reviewer/engine/Cargo.toml"
cargo test --manifest-path "${CODEOS_ROOT}/dba/04-tools/workflow/engine/Cargo.toml"

printf '04-tools tests: PASS\n'
