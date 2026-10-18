#!/usr/bin/env bash
# DQA WO-07 replay of one F-0001/F-0006 identical-surface F4 pair. Under DBA-28 the human recorded
# two separate F4 decisions on one Observed Surface. The replay removes those two receipts (neither
# judgment yet made), regenerates both journeys under the candidate with the Observed Surface the
# real journeys recorded (through the `.verify-stub` seam: no browser runs), then answers both
# questions in one Human Validation Session, with matching and with mixed outcomes, and checks that
# nothing else gains a judgment. Open questions are read from `session --subject`. Prints JSON.
set -euo pipefail
project="$1" workflow="$2"
# Exit 2 is a governed BLOCKED/WAITING report, not a failure.
run() { "${workflow}" --project "$1" "${@:2}" || [[ $? -eq 2 ]]; }
open() { run "$1" session "${@/#/--subject=}" | jq -c '[.[] | {feature, checkpoint}]'; }
decisions="${project}/.codeos/06-workflow/decisions.jsonl"

latest_surface() {
  jq -rs --arg f "$1" '[.[] | select(.subject == $f and .checkpoint == "early_preview")] | last
    | .bindings.observed_surface' "${decisions}"
}
surface="$(latest_surface F-0001)"
[[ "${surface}" == "$(latest_surface F-0006)" ]] || { echo 'the pair does not share a surface' >&2; exit 1; }
jq -c --arg s "${surface}" 'select((.checkpoint == "early_preview" and (.subject == "F-0001" or .subject == "F-0006")
  and .bindings.observed_surface == $s) | not)' "${decisions}" > "${decisions}.new"
mv "${decisions}.new" "${decisions}"
printf 'smoke=pass\nproduct_journey=pass;observed_surface=%s\nbehavior=pass\nrepeatability=pass\nplaywright=pass\n' \
  "${surface}" > "${project}/.codeos/06-workflow/.verify-stub"
for feature in F-0001 F-0006; do
  run "${project}" check --workflow feature --subject "${feature}" --environment deterministic >/dev/null
done

questions="$(run "${project}" session --subject F-0001 --subject F-0006)"
later_before="$(open "${project}" F-0002)"
mixed="${project}.mixed"
rm -rf "${mixed}"
cp -a "${project}" "${mixed}"
run "${project}" session --outcome F-0001:early_preview:direction_confirmed \
  --outcome F-0006:early_preview:direction_confirmed --rationale 'one look at the shared surface' >/dev/null
run "${mixed}" session --outcome F-0001:early_preview:direction_confirmed \
  --outcome F-0006:early_preview:direction_changed --rationale 'one look, two judgments' >/dev/null
matching_open="$(open "${project}" F-0001 F-0006)"
mixed_open="$(open "${mixed}" F-0001 F-0006)"
rm -rf "${mixed}"
sessions="$(find "${project}/.codeos/06-workflow/sessions" -name '*.json' | wc -l)"

# A changed question on the same pixels is not answered by the F4 session.
changed_question="$("${workflow}" --project "${project}" session \
  --outcome F-0006:final_ux_validation:accepted --rationale 'reuse' 2>&1 >/dev/null | tail -n1 || true)"
# A later feature gains nothing from the session.
later_after="$(open "${project}" F-0002)"
# Incomplete viewport coverage: a re-observed surface missing one viewport has another identity.
jq -c 'select(.subject == "F-0001" and .verification == "product_journey")' \
  "${project}/.codeos/06-workflow/verifications.jsonl" | tail -n1 \
  | jq -c '.details.observed_surface = ("one-viewport-" + .details.observed_surface)' \
  >> "${project}/.codeos/06-workflow/verifications.jsonl"
viewport_open="$(open "${project}" F-0001)"

jq -n --argjson questions "${questions}" --argjson sessions "${sessions}" \
  --argjson matching "${matching_open}" --argjson mixed "${mixed_open}" \
  --arg changed "${changed_question}" --argjson later_before "${later_before}" \
  --argjson later_after "${later_after}" --argjson viewport "${viewport_open}" '{
  questions: [$questions[] | {feature, checkpoint}],
  one_shared_surface: ([$questions[].observed_surface] | unique | length == 1),
  sessions: $sessions,
  open_after_matching_session: $matching,
  open_after_mixed_session: $mixed,
  changed_question_refused: ($changed | test("not waiting on final_ux_validation")),
  later_feature_unchanged: ($later_before == $later_after),
  missing_viewport_reopens: $viewport
}'
