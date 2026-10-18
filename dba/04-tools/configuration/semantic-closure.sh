#!/usr/bin/env bash
# Print the normative semantic closure of one DBA configuration, one "<sha256>  <member>" line per
# member, sorted. Members: every selected file; every normative file a member references by path,
# transitively; and, as "term:<Name>", the glossary row of every canonical term a member uses.
# Advisory patterns, templates, prompts and tool scripts are not normative: tools keep their own
# source and build identity (DQA WO-01B).
set -euo pipefail
[[ $# -eq 2 ]] || { echo 'usage: semantic-closure.sh <toolkit-root> <configuration.yaml>' >&2; exit 2; }
ROOT="$(cd "$1" && pwd -P)" CONFIG="$2"
GLOSSARY='dba/05-guidance/terminology.md'
fail() { printf 'semantic closure failed: %s\n' "$1" >&2; exit 1; }
[[ -f "${CONFIG}" ]] || fail "missing configuration ${CONFIG}"

normative() {
  case "$1" in
    "${GLOSSARY}") return 1 ;; # frozen per term below, never as a whole file
    dba/01-doctrine/*.md | dba/02-policies/*.md | dba/04-tools/*/contract/*.md) return 0 ;;
    dba/05-guidance/*/*) return 1 ;;
    dba/05-guidance/*.md) return 0 ;;
    *) return 1 ;;
  esac
}

# The path a reference names inside the toolkit, if it is one tracked file.
resolve() {
  local from="$1" ref="$2" candidate
  ref="${ref#.codeos/toolkit/}"
  if [[ "${ref}" == dba/* ]]; then
    candidate="${ref}"
  elif [[ "${ref}" == */* || "${ref}" == ../* ]]; then
    candidate="$(realpath -m --relative-to="${ROOT}" "${ROOT}/$(dirname "${from}")/${ref}")"
  else
    candidate="$(awk -F/ -v n="${ref}" '$NF == n' <<<"${TRACKED}")"
    [[ "$(printf '%s\n' "${candidate}" | grep -c .)" -eq 1 ]] || return 1
  fi
  [[ -f "${ROOT}/${candidate}" ]] && printf '%s\n' "${candidate}"
}

TRACKED="$(git -C "${ROOT}" ls-files -- 'dba/*')"
declare -A seen=()
queue=()
while IFS= read -r selected; do
  [[ -f "${ROOT}/${selected}" ]] || fail "selected file missing: ${selected}"
  queue+=("${selected}")
done < <(sed -n 's/^[a-z_]*:[[:space:]]*\(dba\/[^[:space:]#]*\).*$/\1/p' "${CONFIG}")
[[ ${#queue[@]} -gt 0 ]] || fail "${CONFIG} selects no files"

while [[ ${#queue[@]} -gt 0 ]]; do
  file="${queue[0]}"
  queue=("${queue[@]:1}")
  [[ -z "${seen[${file}]:-}" ]] || continue
  seen["${file}"]=1
  [[ "${file}" == *.md ]] || continue
  while IFS= read -r ref; do
    target="$(resolve "${file}" "${ref}")" || continue
    normative "${target}" && queue+=("${target}")
  done < <(grep -oE '(\.codeos/toolkit/)?dba/[A-Za-z0-9_./-]+\.(md|json|yaml)|`[A-Za-z0-9_.-]+\.md`|\]\([^)#[:space:]]+\.md' "${ROOT}/${file}" \
    | sed -E 's/^`//; s/`$//; s/^\]\(//' | sort -u)
done

{
  for file in "${!seen[@]}"; do
    printf '%s  %s\n' "$(sha256sum "${ROOT}/${file}" | awk '{print $1}')" "${file}"
  done
  text=''
  for file in "${!seen[@]}"; do text+="$(<"${ROOT}/${file}")"$'\n'; done
  while IFS= read -r row; do
    term="${row#'| **'}"
    term="${term%%'**'*}"
    [[ "${text}" == *"${term}"* ]] || continue
    printf '%s  term:%s\n' "$(printf '%s' "${row}" | sha256sum | awk '{print $1}')" "${term}"
  done < <(grep -E '^\| \*\*[^*]+\*\*' "${ROOT}/${GLOSSARY}")
} | LC_ALL=C sort -k2
