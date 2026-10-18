#!/usr/bin/env bash
# Shared by qualify-candidate.sh and issue-configuration.sh (DQA WO-01A). A candidate configuration
# is maintenance/candidates/<name>.yaml: the keys of a DBA-N file with `status: candidate`. It lives
# outside dba/00-entry, so no project can resolve it. Everything here reads the committed HEAD, so
# the qualified semantic identity names exactly what issuance would publish.

candidate_fail() { printf 'candidate configuration failed: %s\n' "$1" >&2; exit 1; }

# The committed candidate file's path, after checking its form.
candidate_file() {
  local root="$1" name="$2" rel="maintenance/candidates/$2.yaml"
  [[ "${name}" =~ ^[a-z0-9][a-z0-9-]*$ ]] || candidate_fail "invalid candidate name '${name}'"
  git -C "${root}" cat-file -e "HEAD:${rel}" 2>/dev/null || candidate_fail "${rel} is not committed"
  git -C "${root}" show "HEAD:${rel}" | grep -Eq '^status:[[:space:]]*candidate[[:space:]]*$' \
    || candidate_fail "${rel} must declare status: candidate"
  ! git -C "${root}" show "HEAD:${rel}" | grep -Eq '^approved_at:' \
    || candidate_fail "${rel} must not carry approved_at"
  printf '%s\n' "${rel}"
}

# The id the next issued configuration takes.
next_configuration_id() {
  local root="$1" last
  last="$(git -C "${root}" ls-tree --name-only HEAD dba/00-entry/configurations/ \
    | sed -n 's/^.*DBA-\([0-9]*\)\.yaml$/\1/p' | sort -n | tail -n1)"
  printf 'DBA-%s\n' "$((last + 1))"
}

# Export the committed tree into an empty directory.
export_head() {
  local root="$1" dest="$2"
  mkdir -p "${dest}"
  git -C "${root}" archive HEAD | tar -x -C "${dest}"
}

# The qualified semantic identity: selections, semantic closure, CodeOS source (dba/ and
# dba-system.md as committed) and replay definitions.
qualified_candidate_sha256() {
  local root="$1" name="$2" rel tree
  rel="$(candidate_file "${root}" "${name}")"
  tree="$(mktemp -d)"
  export_head "${root}" "${tree}"
  {
    printf 'selections\n'
    sed -n 's/^\([a-z_]*:[[:space:]]*dba\/[^[:space:]#]*\).*$/\1/p' "${tree}/${rel}" | LC_ALL=C sort
    printf 'closure\n'
    bash "${tree}/dba/04-tools/configuration/semantic-closure.sh" "${tree}" "${tree}/${rel}"
    printf 'source\n'
    git -C "${root}" ls-tree -r HEAD -- dba dba-system.md
    printf 'replay\n'
    jq -S -c '.replay_cases // []' "${tree}/dba/04-tools/compatibility/support.json"
    jq -r '.replay_cases // [] | .[].golden' "${tree}/dba/04-tools/compatibility/support.json" \
      | while IFS= read -r golden; do sha256sum "${tree}/dba/04-tools/compatibility/${golden}" | awk '{print $1}'; done
  } | sha256sum | awk '{print $1}'
  rm -rf "${tree}"
}

# Publish candidate <name> as configuration <id> in the tree at <root>: the configuration file, its
# frozen semantic closure, both integrity sums, and its support state.
materialize_candidate() {
  local root="$1" name="$2" id="$3" date="$4"
  local configurations="${root}/dba/00-entry/configurations"
  local support="${root}/dba/04-tools/compatibility/support.json"
  [[ ! -e "${configurations}/${id}.yaml" ]] || candidate_fail "${id} already exists"
  sed -E "s/^status:[[:space:]]*candidate[[:space:]]*$/status: approved\napproved_at: ${date}/" \
    "${root}/maintenance/candidates/${name}.yaml" > "${configurations}/${id}.yaml"
  mkdir -p "${configurations}/closures"
  bash "${root}/dba/04-tools/configuration/semantic-closure.sh" "${root}" "${configurations}/${id}.yaml" \
    > "${configurations}/closures/${id}.closure"
  (cd "${root}" && sha256sum "dba/00-entry/configurations/${id}.yaml" \
    "dba/00-entry/configurations/closures/${id}.closure") >> "${configurations}/SHA256SUMS"
  jq --arg id "${id}" '.configurations[$id] = "supported"' "${support}" > "${support}.tmp"
  mv "${support}.tmp" "${support}"
}
