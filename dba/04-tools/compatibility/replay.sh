#!/usr/bin/env bash
# Qualification replay of one configuration in a toolkit copy (DQA WO-01D):
# 1. the generic lifecycle fixtures no consumer history reproduces;
# 2. every pinned consumer snapshot in support.json `replay_cases`, each in a disposable clone with
#    the configuration overlaid: normalized `status --json` plus the `check --revalidation --plan`
#    of every feature with a revalidation trigger, compared with the human-accepted golden. A case
#    with a `script` instead runs `<script> <clone> <workflow-binary>` there and compares its JSON
#    output (DQA WO-07: the WO-04 shared-validation cases).
# Never touches a consumer's working tree. CODEOS_REPLAY_ACTUAL_DIR, when set, receives each
# case's actual output for review.
set -euo pipefail
[[ $# -eq 2 ]] || { echo 'usage: replay.sh <toolkit-root> <DBA-N>' >&2; exit 2; }
TOOLKIT="$(cd "$1" && pwd -P)" ID="$2"
HERE="${TOOLKIT}/dba/04-tools/compatibility"
LOCAL_ROOT="${CODEOS_COMPATIBILITY_LOCAL_ROOT:-$(cd "${TOOLKIT}/.." && pwd -P)}"
fail() { printf 'qualification replay failed: %s\n' "$1" >&2; exit 1; }
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT
# Build inside the copy: the fixtures locate their toolkit from the compile-time manifest path, so
# a binary built in another copy would replay against the wrong toolkit.
unset CARGO_TARGET_DIR
manifest="${TOOLKIT}/dba/04-tools/workflow/engine/Cargo.toml"

CODEOS_QUALIFICATION_CONFIGURATION="${ID}" \
  cargo test -q --locked --manifest-path "${manifest}" --test qualification_replay \
  || fail "${ID} changes an accepted lifecycle outcome"

cargo build -q --locked --manifest-path "${manifest}"
workflow="${TOOLKIT}/dba/04-tools/workflow/engine/target/debug/codeos-workflow"
run() { env -u TOMTOM_API_KEY -u PROVIDER_MODE "${workflow}" --project "$@"; }

while IFS=$'\t' read -r name consumer commit golden script; do
  live="${LOCAL_ROOT}/${consumer}"
  project="${WORK}/${name}"
  [[ -d "${live}/.git" ]] && git -C "${live}" cat-file -e "${commit}^{commit}" 2>/dev/null \
    || fail "${name}: ${consumer} snapshot ${commit} is unavailable"
  git clone -q --no-checkout "${live}" "${project}"
  git -C "${project}" checkout -q --detach "${commit}"
  [[ "$(cd "${project}" && pwd -P)" != "$(cd "${live}" && pwd -P)" ]] \
    || fail "${name}: refusing to replay in ${consumer}'s working tree"
  rm -f "${project}/.codeos/toolkit"
  ln -s "${TOOLKIT}" "${project}/.codeos/toolkit"
  sed -i -E "s/^dba_configuration:.*/dba_configuration: ${ID}/" "${project}/.codeos/00-project/codeos.yaml"

  if [[ -n "${script}" ]]; then
    env -u TOMTOM_API_KEY -u PROVIDER_MODE bash "${HERE}/${script}" "${project}" "${workflow}" \
      | jq -S . > "${WORK}/${name}-actual.json" || fail "${name}: ${script} failed"
  else
  set +e
  run "${project}" status --json > "${WORK}/${name}-status.json"
  rc=$?
  set -e
  [[ ${rc} -eq 0 || ${rc} -eq 2 ]] || fail "${name}: status failed with ${rc}"
  plans='{}'
  while IFS=$'\t' read -r feature trigger; do
    plan="$(run "${project}" check --workflow feature --subject "${feature}" --revalidation \
      --trigger "${trigger}" --plan)" || fail "${name}: ${feature} plan failed"
    plans="$(jq -c --arg f "${feature}" --argjson p "${plan}" '.[$f] = $p.items' <<<"${plans}")"
  done < <(jq -r '.features[].id as $f | [$f, ([.. | strings | select(test("--subject \($f) .*--trigger [0-9a-f]{40}"))][0] // "" | capture("--trigger (?<t>[0-9a-f]{40})").t)] | select(.[1] != null) | @tsv' \
    "${WORK}/${name}-status.json" 2>/dev/null)
  jq -S --argjson plans "${plans}" '{
      bootstrap_status: .bootstrap.status,
      features: [.features[] | {id, current_validity, status, current_checkpoint}],
      plans: $plans}' "${WORK}/${name}-status.json" > "${WORK}/${name}-actual.json"
  fi
  if [[ -n "${CODEOS_REPLAY_ACTUAL_DIR:-}" ]]; then
    mkdir -p "${CODEOS_REPLAY_ACTUAL_DIR}"
    cp "${WORK}/${name}-actual.json" "${CODEOS_REPLAY_ACTUAL_DIR}/${name}.json"
  fi
  diff -u <(jq -S . "${HERE}/${golden}") "${WORK}/${name}-actual.json" \
    || fail "${name}: ${ID} differs from the accepted replay"
done < <(jq -r '.replay_cases // [] | .[] | [.name, .consumer, .commit, .golden, (.script // "")] | @tsv' "${HERE}/support.json")
printf 'qualification replay: PASS (%s)\n' "${ID}"
