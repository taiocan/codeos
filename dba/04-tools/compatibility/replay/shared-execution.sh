#!/usr/bin/env bash
# DQA WO-07 replay of JamFree state 7bcc9146 (commit 56d28a2, the map-credit fix): the
# post-acceptance revalidation that DBA-28 ran per feature (24 runs of four project-wide commands,
# 9 journeys, 7 qualifications, 4 production runs). Deterministic mechanics are stubbed through
# the workflow's `.verify-stub` seam and the production verifier by a provider-free stub, so the
# replay measures executions, never results. Prints JSON: what each feature executed or reused,
# the distinct Evidence Sets, the shared production run, topology and inherited-claim state.
set -euo pipefail
project="$1" workflow="$2"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
# Exit 2 is a governed BLOCKED/WAITING report, not a failure.
run() { "${workflow}" --project "${project}" "$@" || [[ $? -eq 2 ]]; }
trigger="$(git -C "${project}" rev-parse HEAD)"
printf 'smoke=pass\nproduct_journey=pass\nbehavior=pass\nrepeatability=pass\nplaywright=pass\ndata_integrity=pass\nproject_qualification=pass\n' \
  > "${project}/.codeos/06-workflow/.verify-stub"
cp "${here}/stub-production-verifier.sh" "${project}/scripts/verify-production-claims.mjs"
chmod +x "${project}/scripts/verify-production-claims.mjs"
sets() { find "${project}/.codeos/06-workflow/evidence-sets" -name '*.json' 2>/dev/null | wc -l; }

# One line per verification: "<feature> <checkpoint>/<verification> executed|reused".
lines() {
  sed -nE 's#^(F[0-9a-z]+) / ([a-z_]+): (reused Validation Evidence Set)?.*#\1/\2 \3#p' \
    | awk -v f="$1" '{print f, $1, ($2 == "" ? "executed" : "reused")}'
}
{
  for feature in F-0001 F-0002 F-0003 F-0004 F-0005; do
    run check --workflow feature --subject "${feature}" --revalidation --trigger "${trigger}" \
      --environment deterministic | lines "${feature}"
  done
  for feature in F-0002 F-0003 F-0004 F-0005; do
    run qualify-project --mode acceptance --subject "${feature}" | head -n1 \
      | sed -E 's/^reused Validation Evidence Set.*/reused/; t; s/.*/executed/' \
      | awk -v f="${feature}" '{print f, "candidate/project_qualification", $1}'
  done
} > "${project}/.replay-executions"
deterministic_sets="$(sets)"
production="$(run check --workflow feature --subject F-0002 --subject F-0003 --subject F-0004 \
  --subject F-0005 --revalidation --trigger "${trigger}" --environment production | grep '^F5 /' || true)"
ledgers="$(find "${project}/.codeos/06-workflow/provider-ledger" -name '*.totals.json')"
status="$(run status --json)"
inherited="$(run check --workflow feature --subject F-0006 --revalidation --trigger "${trigger}" --plan \
  2>/dev/null | jq -c '[.items[] | select(.item | startswith("inherited:")) | {item, action, upstream}]' || echo null)"
removed="$(git -C "${project}" diff --numstat -- .codeos/06-workflow/verifications.jsonl \
  .codeos/06-workflow/decisions.jsonl | awk '{s += $2} END {print s + 0}')"
frontier="$(run journey | sed -n 's/^Accepted frontier: //p')"

jq -n --rawfile executions "${project}/.replay-executions" --arg production "${production}" \
  --argjson deterministic_sets "${deterministic_sets}" --argjson status "${status}" \
  --arg frontier "${frontier}" --argjson inherited "${inherited}" --argjson removed "${removed}" \
  --argjson ledgers "$(printf '%s\n' "${ledgers}" | grep -c .)" \
  --argjson calls "$(cat ${ledgers} | jq -s '.')" '{
  executions: ($executions | split("\n") | map(select(. != "") | split(" ") | {feature: .[0], verification: .[1], outcome: .[2]})),
  executed: ($executions | split("\n") | map(select(endswith(" executed"))) | length),
  reused: ($executions | split("\n") | map(select(endswith(" reused"))) | length),
  deterministic_evidence_sets: $deterministic_sets,
  production: {
    records: ($production | split("\n") | map(select(test("-> ok"))) | length),
    one_shared_set: ($production | [scan("Validation Evidence Set ([0-9a-f]+)")] | unique | length == 1),
    provider_runs: $ledgers,
    provider_calls: $calls
  },
  accepted_frontier: $frontier,
  f0006_inherited_claims: $inherited,
  history_lines_removed: $removed,
  validity: [$status.features[] | {id, status: .current_validity.status, reasons: .current_validity.reasons}]
}'
