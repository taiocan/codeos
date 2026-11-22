#!/usr/bin/env bash
# codeos-toolkit-analytics.sh — immutable, hash-verified snapshot of CodeOS governance growth and of
# git churn for the toolkit and the given projects (DBAMON WO-04). Read-only: it reads git history
# only, never checks out, compiles, or contacts the network.
#
#   codeos-toolkit-analytics.sh <external-root> [project ...]
#   codeos-toolkit-analytics.sh --verify-active <external-root>
#
# Tables (one JSON object per line):
#   configurations.jsonl     each issued DBA configuration with its recorded approval and the commit
#                            that added it; `corrective` is null because it is not recorded
#   governance_growth.jsonl  governance-surface counts at each configuration's issue commit and at
#                            HEAD; a count is null where its source does not exist at that commit
#   adoptions.jsonl          each commit where a project's codeos.yaml selected a configuration
#   churn.jsonl              lines added/deleted per file per non-merge commit, bookkeeping excluded
#   file_sizes.jsonl         current line count of every tracked text file, bookkeeping excluded
# Downstream defects attributable to CodeOS are not recorded in structured form; their column is null.
#
# Exit codes: 0 success, 1 usage, 2 failure (nothing activated / verification failed).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd -P)"
# shellcheck source=snapshot-lib.sh
source "${SCRIPT_DIR}/snapshot-lib.sh"
usage() { sed -n '6,7p' "${BASH_SOURCE[0]}" | sed 's/^#  *//' >&2; exit 1; }

# Paths whose churn is records or generated state, not engineering change.
BOOKKEEPING=(':(exclude)maintenance/**' ':(exclude).codeos/**' ':(exclude)events/**' ':(exclude)evidence/**'
  ':(exclude,glob)**/Cargo.lock' ':(exclude,glob)**/package-lock.json')

if [[ "${1:-}" == "--verify-active" ]]; then
  [[ $# -eq 2 ]] || usage
  verify_active "$2"
  exit 0
fi
[[ $# -ge 1 ]] || usage
ROOT="$(realpath -m "$1")"; shift
PROJECTS=()
for project in "$@"; do
  path="$(cd "${project}" && pwd -P)" || fail "project not found: ${project}"
  [[ -f "${path}/.codeos/00-project/codeos.yaml" ]] || fail "not a CodeOS project: ${path}"
  git -C "${path}" rev-parse --git-dir >/dev/null 2>&1 || fail "not a git repository: ${path}"
  PROJECTS+=("${path}")
done
for repo in "${CODEOS_ROOT}" "${PROJECTS[@]}"; do
  case "${ROOT}/" in "${repo}/"*) fail 'the external root must be outside every analysed repository' ;; esac
done
mkdir -p "${ROOT}"
snapshot_stage "${ROOT}"
STAGE="${SNAPSHOT_STAGE}"
git_codeos() { git -C "${CODEOS_ROOT}" "$@"; }

# A sum of `git grep -c` counts at a commit, or 0 when nothing matches.
grep_sum() { { git_codeos grep -I -c -E "$@" || true; } | awk -F: '{s+=$NF} END {print s+0}'; }

# 1. Issued configurations.
: > "${STAGE}/configurations.jsonl"
for file in "${CODEOS_ROOT}"/dba/00-entry/configurations/DBA-*.yaml; do
  name="$(basename "${file}" .yaml)"
  rel="dba/00-entry/configurations/${name}.yaml"
  commit=''; committed=''
  read -r commit committed < <(git_codeos log --diff-filter=A --format='%H %cI' -- "${rel}" | tail -1) || true
  jq -nc --arg c "${name}" --argjson n "${name#DBA-}" \
    --arg status "$(sed -n 's/^status: *//p' "${file}" | head -1)" \
    --arg approved "$(sed -n 's/^approved_at: *//p' "${file}" | head -1)" \
    --arg commit "${commit:-}" --arg committed "${committed:-}" \
    '{configuration: $c, number: $n, status: (if $status == "" then null else $status end),
      approved_at: (if $approved == "" then null else $approved end),
      issue_commit: (if $commit == "" then null else $commit end),
      issue_committed_at: (if $committed == "" then null else $committed end),
      corrective: null, corrective_note: "not recorded in structured form"}' >> "${STAGE}/configurations.jsonl"
done
jq -sc 'sort_by(.number)[]' "${STAGE}/configurations.jsonl" > "${STAGE}/configurations.sorted"
mv "${STAGE}/configurations.sorted" "${STAGE}/configurations.jsonl"

# 2. Governance-surface counts at each issue commit and at HEAD.
count_versions() { # <commit> <dir> <regex>: versioned files under <dir>, or null if <dir> is absent
  local listing
  listing="$(git_codeos ls-tree -r --name-only "$1" -- "$2" 2>/dev/null || true)"
  [[ -n "${listing}" ]] || { echo null; return; }
  grep -c -E "$3" <<< "${listing}" || true
}
growth_row() { # <commit> <label>
  local commit="$1" glossary terms
  glossary="$(git_codeos show "${commit}:dba/05-guidance/terminology.md" 2>/dev/null || true)"
  if [[ -n "${glossary}" ]]; then terms="$(grep -c '^| \*\*' <<< "${glossary}" || true)"; else terms=null; fi
  jq -nc --arg at "$2" --arg commit "${commit}" \
    --arg committed "$(git_codeos show -s --format=%cI "${commit}")" \
    --argjson doctrine "$(count_versions "${commit}" dba/01-doctrine '^dba/01-doctrine/v[0-9]+\.md$')" \
    --argjson policies "$(count_versions "${commit}" dba/02-policies '^dba/02-policies/[^/]+/v[0-9]+\.md$')" \
    --argjson contracts "$(count_versions "${commit}" dba/04-tools/reviewer/contract '/v[0-9]+\.md$')" \
    --argjson configurations "$(count_versions "${commit}" dba/00-entry/configurations '/DBA-[0-9]+\.yaml$')" \
    --argjson terms "${terms}" \
    --argjson flags "$(grep_sum '#\[arg\(.*long' "${commit}" -- ':(glob)**/*.rs')" \
    --argjson lines "$(grep_sum '' "${commit}" -- ':(glob)**/*.rs')" \
    --argjson tests "$(grep_sum '#\[(tokio::)?test' "${commit}" -- ':(glob)**/*.rs')" \
    '{as_of: $at, commit: $commit, committed_at: $committed, doctrine_versions: $doctrine,
      policy_versions: $policies, reviewer_contract_versions: $contracts,
      configurations: $configurations, glossary_terms: $terms, cli_flags: $flags,
      rust_lines: $lines, rust_tests: $tests, codeos_attributable_defects: null}'
}
: > "${STAGE}/governance_growth.jsonl"
while IFS=$'\t' read -r name commit; do
  [[ "${commit}" == null ]] || growth_row "${commit}" "${name}" >> "${STAGE}/governance_growth.jsonl"
done < <(jq -r '[.configuration, (.issue_commit // "null")] | @tsv' "${STAGE}/configurations.jsonl")
growth_row "$(git_codeos rev-parse HEAD)" HEAD >> "${STAGE}/governance_growth.jsonl"

# 3. Adoptions: each project commit whose codeos.yaml selects a configuration different from before.
: > "${STAGE}/adoptions.jsonl"
for project in "${PROJECTS[@]}"; do
  previous=''
  while read -r commit committed; do
    selected="$(git -C "${project}" show "${commit}:.codeos/00-project/codeos.yaml" 2>/dev/null \
      | sed -n 's/^dba_configuration: *//p' | head -1)"
    [[ -n "${selected}" && "${selected}" != "${previous}" ]] || continue
    jq -nc --arg p "$(basename "${project}")" --arg c "${selected}" --arg commit "${commit}" \
      --arg at "${committed}" '{project: $p, configuration: $c, commit: $commit, committed_at: $at}' \
      >> "${STAGE}/adoptions.jsonl"
    previous="${selected}"
  done < <(git -C "${project}" log --reverse --format='%H %cI' -- .codeos/00-project/codeos.yaml)
done

# 4. Churn and current size, per repository, bookkeeping excluded.
: > "${STAGE}/churn.jsonl"
: > "${STAGE}/file_sizes.jsonl"
for repo in "${CODEOS_ROOT}" "${PROJECTS[@]}"; do
  name="$(basename "${repo}")"
  # --no-renames keeps every path a plain current or former file path, joinable to file sizes.
  git -C "${repo}" log --no-merges --no-renames --numstat --format='@%H %cI' -- . "${BOOKKEEPING[@]}" \
    | awk -F'\t' -v repo="${name}" '
        /^@/ { split(substr($0, 2), h, " "); commit = h[1]; at = h[2]; next }
        NF == 3 { added = ($1 == "-") ? "null" : $1; deleted = ($2 == "-") ? "null" : $2
                  path = $3
                  gsub(/\\/, "\\\\", path); gsub(/"/, "\\\"", path)
                  printf "{\"repo\":\"%s\",\"commit\":\"%s\",\"at\":\"%s\",\"path\":\"%s\",\"added\":%s,\"deleted\":%s}\n",
                         repo, commit, at, path, added, deleted }' >> "${STAGE}/churn.jsonl"
  { git -C "${repo}" grep -I -c '' HEAD -- . "${BOOKKEEPING[@]}" || true; } \
    | sed 's/^HEAD://' | awk -F: -v repo="${name}" '{
        lines = $NF; path = substr($0, 1, length($0) - length(lines) - 1)
        gsub(/\\/, "\\\\", path); gsub(/"/, "\\\"", path)
        printf "{\"repo\":\"%s\",\"path\":\"%s\",\"lines\":%s}\n", repo, path, lines }' \
    >> "${STAGE}/file_sizes.jsonl"
done
jq -c . "${STAGE}/churn.jsonl" "${STAGE}/file_sizes.jsonl" > /dev/null || fail 'malformed churn or size rows'

# 5. Manifest and activation.
repos="$(for repo in "${CODEOS_ROOT}" "${PROJECTS[@]}"; do
  jq -nc --arg n "$(basename "${repo}")" --arg h "$(git -C "${repo}" rev-parse HEAD)" '{name: $n, head: $h}'
done | jq -sc .)"
jq -n --argjson v "${SNAPSHOT_SCHEMA_VERSION}" --arg id "${SNAPSHOT_ID}" \
  --arg generated "$(date -u +%Y-%m-%dT%H:%M:%SZ)" --argjson repos "${repos}" \
  --argjson files "$(snapshot_files "${STAGE}")" \
  --argjson excluded "$(printf '%s\n' "${BOOKKEEPING[@]}" | jq -R . | jq -sc .)" \
  '{schema_version: $v, snapshot_id: $id, status: "complete", kind: "toolkit-analytics",
    generated_at: $generated, repositories: $repos, churn_excludes: $excluded, files: $files}' \
  > "${STAGE}/manifest.json"
snapshot_activate "${ROOT}"
