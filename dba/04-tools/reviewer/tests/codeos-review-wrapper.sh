#!/usr/bin/env bash
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-review-wrapper.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
fail() { printf 'reviewer wrapper tests failed: %s\n' "$1" >&2; exit 1; }

PROJECT="${WORK}/project"
TOOLKIT="${WORK}/toolkit"
BIN="${WORK}/bin"
mkdir -p "${PROJECT}" "${TOOLKIT}/dba/04-tools/reviewer" "${BIN}"
git -C "${PROJECT}" init -q -b main
cp "${CODEOS_ROOT}/dba/04-tools/reviewer/codeos-review.sh" "${TOOLKIT}/dba/04-tools/reviewer/codeos-review.sh"
printf '#!/usr/bin/env bash\nprintf "%%s\\n" "$@" > "${REVIEW_ARGS_FILE}"\nexit 23\n' > "${BIN}/codeos-reviewer"
chmod +x "${BIN}/codeos-reviewer"

set +e
(cd "${PROJECT}" && PATH="${BIN}:${PATH}" REVIEW_ARGS_FILE="${WORK}/args" \
  "${TOOLKIT}/dba/04-tools/reviewer/codeos-review.sh" plan demo step artifact.md)
status=$?
set -e
[[ ${status} -eq 23 ]] || fail 'wrapper did not preserve reviewer exit status'
[[ "$(tr '\n' ' ' < "${WORK}/args")" == 'plan demo step artifact.md ' ]] || fail 'wrapper changed reviewer arguments'

printf 'reviewer wrapper tests: PASS\n'
