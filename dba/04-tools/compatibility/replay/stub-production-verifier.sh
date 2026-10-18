#!/usr/bin/env bash
# Deterministic stand-in for the production verifier (no provider): one forecast leg per
# direction, each receipted and ledgered once, observing every claim of that direction.
set -euo pipefail
[[ "${CODEOS_EXECUTION_ENVIRONMENT:-}" == production && -n "${CODEOS_PROVIDER_LEDGER:-}" ]] || exit 9
d="$CODEOS_REALITY_DEFINITION_PATH"
leg() { case "$1" in F-0002/*|*return-summary) echo return ;; *) echo outbound ;; esac; }
claims=$(jq -r '.subjects[] | .subject as $f | .claims[] | "\($f)/\(.)"' "$d")
obs='[]'; n=0
for direction in outbound return; do
  ids=$(for c in $claims; do if [[ "$(leg "$c")" == "$direction" ]]; then echo "$c"; fi; done | jq -R . | jq -sc .)
  [[ "$ids" == "[]" ]] && continue
  n=$((n + 1))
  printf '{"schema":1,"attempt":%d,"event":"attempt","at":"2026-10-18T00:00:00Z"}\n{"schema":1,"attempt":%d,"event":"success","at":"2026-10-18T00:00:01Z"}\n' $n $n >> "$CODEOS_PROVIDER_LEDGER"
  h=$(printf '%s' "$direction" | sha256sum | cut -c1-64)
  r='[{"provider_host":"provider.stub","provider_path":"/'$direction'","request_sha256":"'$h'","response_sha256":"'$h'","provider_request_id":null,"requested_at":"2026-10-18T00:00:00Z","received_at":"2026-10-18T00:00:01Z","http_status":200}]'
  v='"'$direction'"'
  vs=$(printf '%s' "$v" | sha256sum | cut -c1-64); rs=$(printf '%s' "$r" | sha256sum | cut -c1-64)
  obs=$(jq -c --argjson ids "$ids" --argjson r "$r" --argjson v "$v" --arg vs "$vs" --arg rs "$rs" \
    '. + [{claim_ids: $ids, visible_result: $v, visible_result_sha256: $vs, provider_receipts_sha256: $rs, provider_request_response_receipts: $r, details: {}}]' <<<"$obs")
done
jq --argjson obs "$obs" '{schema: 3,
  subjects: [.subjects[] | {subject, claims, governed_inputs, claim_scopes: (reduce .claims[] as $c ({}; .[$c] = ["backend", "web"]))}],
  source_kind: "production", provider: "provider.stub", observed_at: "2026-10-18T00:00:00Z",
  observations: $obs}' "$d" > "$CODEOS_REALITY_EVIDENCE_PATH"
