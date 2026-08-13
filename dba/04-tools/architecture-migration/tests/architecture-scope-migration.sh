#!/usr/bin/env bash
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT

fail() { printf 'FAIL: %s\n' "$1" >&2; exit 1; }
make_project() {
  local root="$1"
  mkdir -p "${root}/features" "${root}/architecture"
  git -C "${root}" init -q -b main
  git -C "${root}" config user.email test@example.com
  git -C "${root}" config user.name Test
}

PROJECT="${WORK}/ok"
make_project "${PROJECT}"
FIXTURES="${CODEOS_ROOT}/dba/04-tools/architecture-migration/tests/fixtures"
cp "${FIXTURES}/architecture-migration-registry.yaml" "${PROJECT}/features/registry.yaml"
cp "${FIXTURES}/architecture-migration-baseline.md" "${PROJECT}/architecture/core-baseline.md"
cp "${FIXTURES}/architecture-migration-logical.md" "${PROJECT}/architecture/cohort-logical-design.md"
cp "${FIXTURES}/architecture-migration-profile.yaml" "${PROJECT}/architecture/implementation-profile.yaml"
git -C "${PROJECT}" add . && git -C "${PROJECT}" commit -qm legacy

python3 "${CODEOS_ROOT}/dba/04-tools/architecture-migration/migrate-architecture-synthesis-v2.py" "${PROJECT}" >/dev/null
[[ ! -e "${PROJECT}/architecture/scopes/example.md" ]] || fail "dry run mutated project"
python3 "${CODEOS_ROOT}/dba/04-tools/architecture-migration/migrate-architecture-synthesis-v2.py" "${PROJECT}" --apply >/dev/null
SCOPE="${PROJECT}/architecture/scopes/example.md"
[[ -f "${SCOPE}" ]] || fail "scope was not created"
grep -q '^features:$' "${SCOPE}" || fail "membership was not preserved"
grep -q '^approval:$' "${SCOPE}" || fail "approval was not preserved"
! grep -qE '^(scope_id|status|approved_by|approved_at):' "${SCOPE}" || fail "obsolete scope metadata remains"
grep -q 'Workspace owns dependency direction.' "${SCOPE}" || fail "baseline decision was not preserved"
grep -q 'Records use stable ids.' "${SCOPE}" || fail "logical decision was not preserved"
[[ ! -e "${PROJECT}/architecture/core-baseline.md" && ! -e "${PROJECT}/architecture/cohort-logical-design.md" ]] || fail "legacy artifacts remain"
! grep -q 'architecture_cohort' "${PROJECT}/features/registry.yaml" || fail "legacy registry state remains"
python3 - "${PROJECT}/architecture/implementation-profile.yaml" <<'PY'
import sys, yaml
profile = yaml.safe_load(open(sys.argv[1], encoding="utf-8"))
assert profile["applies_to"] == {"scope": "feature_ids", "feature_ids": ["F-0001", "F-0002"]}
assert [item["id"] for item in profile["exceptions"]] == ["F-0001", "F-0002"]
assert all(item["scope"] == "feature_id" for item in profile["exceptions"])
PY

BAD="${WORK}/bad"
make_project "${BAD}"
cp -R "${FIXTURES}/architecture-migration-registry.yaml" "${BAD}/features/registry.yaml"
sed 's/F-0002/F-0003/' "${FIXTURES}/architecture-migration-baseline.md" > "${BAD}/architecture/core-baseline.md"
cp "${FIXTURES}/architecture-migration-logical.md" "${BAD}/architecture/cohort-logical-design.md"
git -C "${BAD}" add . && git -C "${BAD}" commit -qm ambiguous
if python3 "${CODEOS_ROOT}/dba/04-tools/architecture-migration/migrate-architecture-synthesis-v2.py" "${BAD}" >/dev/null 2>&1; then
  fail "ambiguous membership was accepted"
fi

printf 'architecture scope migration tests: PASS\n'
