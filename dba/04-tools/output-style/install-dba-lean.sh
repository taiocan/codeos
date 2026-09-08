#!/usr/bin/env bash
# Install the optional Claude Code adherence adapter without overriding an existing selection.
set -euo pipefail
PROJECT_ROOT="${1:-$(pwd -P)}"
TOOLKIT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"
SETTINGS="${PROJECT_ROOT}/.claude/settings.json"
STYLE_DIR="${PROJECT_ROOT}/.claude/output-styles"

mkdir -p "${STYLE_DIR}"
cp "${TOOLKIT_ROOT}/dba/04-tools/output-style/dba-lean.md" "${STYLE_DIR}/dba-lean.md"

if [[ -f "${SETTINGS}" ]] && grep -q '"outputStyle"[[:space:]]*:' "${SETTINGS}"; then
  printf 'DBA Lean installed but inactive: %s already selects an outputStyle; no setting was overwritten.\n' "${SETTINGS}"
  exit 2
fi

if [[ -f "${SETTINGS}" ]]; then
  printf 'DBA Lean installed but inactive: add "outputStyle": "dba-lean" to valid JSON in %s; existing settings were not rewritten.\n' "${SETTINGS}"
  exit 2
fi

mkdir -p "${PROJECT_ROOT}/.claude"
printf '{\n  "outputStyle": "dba-lean"\n}\n' > "${SETTINGS}"
printf 'DBA Lean active at project-shared precedence. Start a new session or run /clear.\n'
