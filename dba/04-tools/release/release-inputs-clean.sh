#!/usr/bin/env bash
# Refuse a qualified release build whose inputs are not exactly the committed tree (DQA WO-01).
# The release identity hashes every tracked file, so a tracked change anywhere is refused. An
# untracked file is never hashed, so it matters only where a build could read it: the CodeOS
# source boundary the qualification digest uses, `dba/` and `dba-system.md`.
set -euo pipefail
ROOT="$1"
fail() { printf 'qualified release failed: %s\n' "$1" >&2; exit 1; }
git -C "${ROOT}" diff --quiet -- . ':!maintenance/release-manifest.json' \
  || fail 'tracked source changes must be committed before a qualified release build'
git -C "${ROOT}" diff --cached --quiet -- . ':!maintenance/release-manifest.json' \
  || fail 'staged source changes must be committed before a qualified release build'
while IFS= read -r -d '' path; do
  fail "untracked release input must be committed or ignored: ${path}"
done < <(git -C "${ROOT}" ls-files --others --exclude-standard -z -- dba dba-system.md)
