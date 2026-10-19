#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
WORK="$(mktemp -d)"; trap 'rm -rf "${WORK}"' EXIT
printf 'qualified binary\n' > "${WORK}/tool"
sha="$(sha256sum "${WORK}/tool" | awk '{print $1}')"
printf '{"schema":1,"binaries":{"demo":{"sha256":"%s","build_id":"source"}}}\n' "${sha}" > "${WORK}/manifest.json"
CODEOS_RELEASE_MANIFEST="${WORK}/manifest.json" bash "${HERE}/verify-qualified-binary.sh" demo "${WORK}/tool"
printf 'stale\n' >> "${WORK}/tool"
if CODEOS_RELEASE_MANIFEST="${WORK}/manifest.json" bash "${HERE}/verify-qualified-binary.sh" demo "${WORK}/tool" >/dev/null 2>&1; then
  echo 'stale binary was accepted' >&2; exit 1
fi
# The input guard: untracked files matter only inside the CodeOS source boundary.
repo="${WORK}/repo"; mkdir -p "${repo}/dba"
git -C "${repo}" init -q && printf 'x\n' > "${repo}/dba/tracked" && printf 'y\n' > "${repo}/dba-system.md"
git -C "${repo}" add -A && git -C "${repo}" -c user.name=t -c user.email=t@t commit -qm base
clean() { bash "${HERE}/release-inputs-clean.sh" "${repo}" >/dev/null 2>&1; }
printf 'notes\n' > "${repo}/unrelated.md"
clean || { echo 'an untracked file outside dba/ refused the build' >&2; exit 1; }
printf 'new\n' > "${repo}/dba/untracked.rs"
if clean; then echo 'an untracked file inside dba/ was accepted' >&2; exit 1; fi
rm "${repo}/dba/untracked.rs"
printf 'z\n' >> "${repo}/dba-system.md"
if clean; then echo 'a modified tracked release input was accepted' >&2; exit 1; fi
printf 'release identity tests: PASS\n'
