#!/usr/bin/env bash
# Deterministic configuration matrix plus pinned, read-only downstream projections.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT="$(cd "${HERE}/../../.." && pwd -P)"
SUPPORT="${HERE}/support.json"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT

bash "${HERE}/validate-support-manifest.sh" "${SUPPORT}" "${ROOT}"

while IFS=$'\t' read -r id state; do
  [[ "${state}" == supported ]] || continue
  project="${WORK}/${id}"
  mkdir -p "${project}/.codeos/00-project"
  cat > "${project}/.codeos/00-project/codeos.yaml" <<YAML
codeos_schema: 2
dba_configuration: ${id}
platform:
  persistence: postgresql
  backend: rust
  webapp: svelte
  runtime: docker
artifacts:
  charter: governed
  intent: governed
  contract: governed
  event_schema: governed
YAML
  resolved="$(bash "${ROOT}/dba/04-tools/configuration/resolve-project-configuration.sh" "${project}")"
  [[ "${resolved}" == *"dba_configuration=${id}"* ]] || { echo "${id}: wrong resolved identity" >&2; exit 1; }
  while IFS= read -r selected; do
    [[ -z "${selected}" || -f "${ROOT}/${selected}" ]] || { echo "${id}: missing ${selected}" >&2; exit 1; }
  done < <(printf '%s\n' "${resolved}" | sed -n 's/^[a-z_]*=\(dba\/.*\)$/\1/p')
  if [[ "${resolved}" == *'codeos_mechanics_policy='* ]]; then
    bash "${ROOT}/dba/04-tools/configuration/project-config-contract.sh" "${project}/.codeos/00-project/codeos.yaml" >/dev/null
  fi
done < <(jq -r '.configurations|to_entries[]|[.key,.value]|@tsv' "${SUPPORT}")

if [[ "${CODEOS_SKIP_REAL_CONSUMERS:-0}" != 1 ]]; then
  cargo build --quiet --locked --manifest-path "${ROOT}/dba/04-tools/workflow/engine/Cargo.toml"
  workflow="${ROOT}/dba/04-tools/workflow/engine/target/debug/codeos-workflow"
  while IFS=$'\t' read -r name repository commit bundle bundle_sha256 golden; do
    local_candidate="${CODEOS_COMPATIBILITY_LOCAL_ROOT:-$(cd "${ROOT}/.." && pwd -P)}/${name}"
    project="${WORK}/${name}"
    if [[ -d "${local_candidate}/.git" ]] \
      && git -C "${local_candidate}" cat-file -e "${commit}^{commit}" 2>/dev/null; then
      git clone --quiet --no-checkout "${local_candidate}" "${project}"
    elif git clone --quiet --no-checkout "${repository}" "${project}" 2>/dev/null \
      && git -C "${project}" cat-file -e "${commit}^{commit}" 2>/dev/null; then
      :
    else
      rm -rf "${project}"
      bundle_path="${HERE}/${bundle}"
      printf '%s  %s\n' "${bundle_sha256}" "${bundle_path}" | sha256sum --check --status \
        || { echo "${name}: compatibility bundle identity mismatch" >&2; exit 1; }
      git clone --quiet --no-checkout "${bundle_path}" "${project}"
    fi
    git -C "${project}" checkout --quiet --detach "${commit}"
    rm -f "${project}/.codeos/toolkit"
    ln -s "${ROOT}" "${project}/.codeos/toolkit"
    set +e
    "${workflow}" --project "${project}" status --json > "${WORK}/${name}-status.json"
    rc=$?
    set -e
    [[ ${rc} -eq 0 || ${rc} -eq 2 ]] || { echo "${name}: status failed with ${rc}" >&2; exit 1; }
    jq -S -c '{schema_version,project,bootstrap_status:.bootstrap.status,features:[.features[]|{id,historically_accepted,current_acceptance_valid,current_validity,status,current_checkpoint}]}' \
      "${WORK}/${name}-status.json" > "${WORK}/${name}-normalized.json"
    jq -S -c . "${HERE}/${golden}" > "${WORK}/${name}-golden.json"
    diff -u "${WORK}/${name}-golden.json" "${WORK}/${name}-normalized.json"
  done < <(jq -r '.consumers[]|[.name,.repository,.commit,.bundle,.bundle_sha256,.golden]|@tsv' "${SUPPORT}")
fi
printf 'downstream compatibility: PASS\n'
