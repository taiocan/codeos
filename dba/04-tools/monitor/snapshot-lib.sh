#!/usr/bin/env bash
# Shared snapshot mechanics for the DBAMON monitor scripts (sourced, never run). One path builds,
# verifies, and activates every snapshot: <root>/snapshots/<id>/{manifest.json, files} and an
# atomically written <root>/current.json. A failure before activation leaves current.json as it was.

SNAPSHOT_SCHEMA_VERSION=1
fail() { printf 'monitor snapshot failed: %s\n' "$1" >&2; exit 2; }
command -v jq >/dev/null || fail 'jq is required'

# The rows a data file holds: lines of a .jsonl file, 1 for a .json document.
rows() { [[ "$1" == *.jsonl ]] && { grep -c '' "$1" || true; } || echo 1; }

# A new snapshot id and its staging folder under <root>, removed on exit unless activated.
snapshot_stage() {
  SNAPSHOT_ID="$(date -u +%Y%m%dT%H%M%SZ)-$(od -An -N4 -tx1 /dev/urandom | tr -d ' \n')"
  SNAPSHOT_STAGE="$1/.staging-${SNAPSHOT_ID}"
  mkdir -p "${SNAPSHOT_STAGE}" "$1/snapshots"
  trap 'rm -rf "${SNAPSHOT_STAGE}"' EXIT
}

# The manifest's `files` object: sha256 and row count of every staged file.
snapshot_files() {
  local path
  for path in "$1"/*; do
    jq -n --arg name "$(basename "${path}")" --arg sha "$(sha256sum "${path}" | cut -d' ' -f1)" \
      --argjson rows "$(rows "${path}")" '{($name): {sha256: $sha, rows: $rows}}'
  done | jq -s 'add'
}

# Re-hash and re-count every file a manifest names; any difference or extra file fails.
verify_snapshot() {
  local dir="$1" manifest="$1/manifest.json"
  [[ -f "${manifest}" ]] || fail "no manifest in ${dir}"
  jq -e --argjson v "${SNAPSHOT_SCHEMA_VERSION}" '.schema_version == $v and .status == "complete"' \
    "${manifest}" >/dev/null || fail "manifest is not a complete schema-${SNAPSHOT_SCHEMA_VERSION} snapshot"
  local name sha count
  while IFS=$'\t' read -r name sha count; do
    [[ -f "${dir}/${name}" ]] || fail "missing ${name}"
    [[ "$(sha256sum "${dir}/${name}" | cut -d' ' -f1)" == "${sha}" ]] || fail "hash mismatch: ${name}"
    [[ "$(rows "${dir}/${name}")" == "${count}" ]] || fail "row count mismatch: ${name}"
  done < <(jq -r '.files | to_entries[] | [.key, .value.sha256, .value.rows] | @tsv' "${manifest}")
  [[ "$(cd "${dir}" && ls | grep -v '^manifest.json$' | sort)" == \
     "$(jq -r '.files | keys[]' "${manifest}" | sort)" ]] || fail "files differ from the manifest"
}

verify_active() {
  local root="$1" current="$1/current.json"
  [[ -f "${current}" ]] || fail "no active snapshot: ${current}"
  local id expected
  id="$(jq -er '.snapshot_id' "${current}")" || fail 'current.json names no snapshot'
  expected="$(jq -er '.manifest_sha256' "${current}")" || fail 'current.json has no manifest hash'
  [[ "${id}" =~ ^[0-9]{8}T[0-9]{6}Z-[0-9a-f]{8}$ ]] || fail "malformed snapshot id: ${id}"
  local dir="${root}/snapshots/${id}"
  [[ "$(sha256sum "${dir}/manifest.json" 2>/dev/null | cut -d' ' -f1)" == "${expected}" ]] \
    || fail "manifest hash differs from current.json for ${id}"
  verify_snapshot "${dir}"
  printf 'PASS: active snapshot %s is complete and hash-verified\n' "${id}"
}

# Verify the staged snapshot, move it into place, then activate it atomically.
snapshot_activate() {
  local root="$1"
  verify_snapshot "${SNAPSHOT_STAGE}"
  mv "${SNAPSHOT_STAGE}" "${root}/snapshots/${SNAPSHOT_ID}"
  trap - EXIT
  jq -n --arg id "${SNAPSHOT_ID}" \
    --arg sha "$(sha256sum "${root}/snapshots/${SNAPSHOT_ID}/manifest.json" | cut -d' ' -f1)" \
    --arg at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    '{snapshot_id: $id, manifest_sha256: $sha, activated_at: $at}' > "${root}/current.json.tmp"
  mv "${root}/current.json.tmp" "${root}/current.json"
  verify_active "${root}"
}
