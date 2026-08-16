#!/usr/bin/env bash
# Initialize the minimum project-local Codeos structure.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_PATH="$(cd "$SCRIPT_DIR/../../.." && pwd -P)"
PROJECT_DIR="$(pwd -P)"
PROJECT_NAME="${1:-$(basename "$PROJECT_DIR")}"
REMOTE_URL="${2:-}"
CODEOS_DIR="$PROJECT_DIR/.codeos"
TOOLKIT_LINK="$CODEOS_DIR/toolkit"
ROOT_CLAUDE="$PROJECT_DIR/CLAUDE.md"
ROOT_AGENTS="$PROJECT_DIR/AGENTS.md"
ROOT_CLAUDE_TEMPLATE="$CODEOS_PATH/dba/05-guidance/templates/project-root-CLAUDE.md"
ROOT_AGENTS_TEMPLATE="$CODEOS_PATH/dba/05-guidance/templates/project-AGENTS.md"
PROJECT_CLAUDE="$CODEOS_DIR/00-project/CLAUDE.md"

fail() {
    echo "[error] $*" >&2
    exit 1
}

[[ "$PROJECT_NAME" != *$'\n'* && "$PROJECT_NAME" != *$'\r'* ]] || \
    fail "project name must be a single line"

# Git itself is the authority for repositories and linked worktrees. Initializing inside an
# existing repository would silently create a nested repository, so reject it before mutation.
PROJECT_HAS_GIT=false
if git_root_raw="$(git -C "$PROJECT_DIR" rev-parse --show-toplevel 2>/dev/null)"; then
    GIT_ROOT="$(cd "$git_root_raw" && pwd -P)"
    if [[ "$GIT_ROOT" != "$PROJECT_DIR" ]]; then
        fail "project directory is inside an existing Git repository at $GIT_ROOT; initialize the repository root instead"
    fi
    PROJECT_HAS_GIT=true
fi

echo "DBA Project Init"
echo "Project name : $PROJECT_NAME"
echo "Project dir  : $PROJECT_DIR"
echo "Toolkit path : $CODEOS_PATH"

# Preflight discovery surfaces before changing the project. Existing files are never merged or
# overwritten. A current adapter is safe to preserve; anything else needs a human migration.
if [[ -f "$ROOT_CLAUDE" ]] && ! cmp -s "$ROOT_CLAUDE" "$ROOT_CLAUDE_TEMPLATE"; then
    fail "CLAUDE.md already exists and is not the Codeos discovery adapter; preserve it and migrate its substantive instructions manually to .codeos/00-project/CLAUDE.md (see $CODEOS_PATH/dba/06-reference/downstream-upgrade.md)"
fi
if [[ -e "$ROOT_CLAUDE" && ! -f "$ROOT_CLAUDE" ]]; then
    fail "CLAUDE.md exists but is not a regular file"
fi
if [[ -f "$ROOT_AGENTS" ]] && ! cmp -s "$ROOT_AGENTS" "$ROOT_AGENTS_TEMPLATE"; then
    fail "AGENTS.md already exists and is not the Codeos root adapter; preserve it and migrate it manually (see $CODEOS_PATH/dba/06-reference/downstream-upgrade.md)"
fi
if [[ -e "$ROOT_AGENTS" && ! -f "$ROOT_AGENTS" ]]; then
    fail "AGENTS.md exists but is not a regular file"
fi

if [[ -L "$CODEOS_DIR" ]]; then
    fail ".codeos is a legacy toolkit symlink; migrate it to the project-local layout first (see $CODEOS_PATH/dba/06-reference/downstream-upgrade.md)"
fi
if [[ -e "$CODEOS_DIR" && ! -d "$CODEOS_DIR" ]]; then
    fail ".codeos exists but is not a directory"
fi
if [[ -L "$TOOLKIT_LINK" ]]; then
    if [[ ! -e "$TOOLKIT_LINK/dba-system.md" || ! -d "$TOOLKIT_LINK/dba/00-entry" ]]; then
        fail ".codeos/toolkit is broken or does not point to a valid Codeos toolkit; repair it explicitly"
    fi
elif [[ -e "$TOOLKIT_LINK" ]]; then
    fail ".codeos/toolkit exists but is not a symlink"
fi
if [[ -e "$PROJECT_CLAUDE" && ! -f "$PROJECT_CLAUDE" ]]; then
    fail ".codeos/00-project/CLAUDE.md exists but is not a regular file"
fi
if [[ -e "$PROJECT_DIR/.gitignore" && ! -f "$PROJECT_DIR/.gitignore" ]]; then
    fail ".gitignore exists but is not a regular file"
fi

mkdir -p \
    "$CODEOS_DIR/00-project" \
    "$CODEOS_DIR/01-specification/intents" \
    "$CODEOS_DIR/01-specification/contracts" \
    "$CODEOS_DIR/01-specification/event-schemas"

if [[ -L "$TOOLKIT_LINK" ]]; then
    echo "[skip] .codeos/toolkit already points to a valid toolkit"
else
    ln -s "$CODEOS_PATH" "$TOOLKIT_LINK"
    echo "[ok]   .codeos/toolkit -> $CODEOS_PATH"
fi

GITIGNORE="$PROJECT_DIR/.gitignore"
IGNORE_RULES=("/.codeos/toolkit" "/.codeos-state/")
MISSING_IGNORE_RULES=()
for ignore_rule in "${IGNORE_RULES[@]}"; do
    if [[ ! -f "$GITIGNORE" ]] || ! grep -Fxq "$ignore_rule" "$GITIGNORE"; then
        MISSING_IGNORE_RULES+=("$ignore_rule")
    fi
done
if (( ${#MISSING_IGNORE_RULES[@]} > 0 )); then
    if [[ -s "$GITIGNORE" ]]; then
        printf '\n' >> "$GITIGNORE"
    fi
    printf '%s\n' "${MISSING_IGNORE_RULES[@]}" >> "$GITIGNORE"
    echo "[ok]   .gitignore ignores the toolkit mount and operational state"
else
    echo "[skip] .gitignore already ignores the toolkit mount and operational state"
fi

if [[ -f "$PROJECT_CLAUDE" ]]; then
    echo "[skip] .codeos/00-project/CLAUDE.md already exists"
else
    escaped_project_name="${PROJECT_NAME//\\/\\\\}"
    escaped_project_name="${escaped_project_name//&/\\&}"
    escaped_project_name="${escaped_project_name//|/\\|}"
    sed "s|\[PROJECT_NAME\]|$escaped_project_name|g" \
        "$CODEOS_PATH/dba/05-guidance/templates/project-CLAUDE.md" > "$PROJECT_CLAUDE"
    echo "[ok]   .codeos/00-project/CLAUDE.md"
fi

if [[ -f "$ROOT_CLAUDE" ]]; then
    echo "[skip] CLAUDE.md already contains the Codeos discovery adapter"
else
    cp "$ROOT_CLAUDE_TEMPLATE" "$ROOT_CLAUDE"
    echo "[ok]   CLAUDE.md"
fi

if [[ -f "$ROOT_AGENTS" ]]; then
    echo "[skip] AGENTS.md already contains the Codeos root adapter"
else
    cp "$ROOT_AGENTS_TEMPLATE" "$ROOT_AGENTS"
    echo "[ok]   AGENTS.md"
fi

if [[ "$PROJECT_HAS_GIT" == true ]]; then
    echo "[skip] git repo already exists"
else
    git -C "$PROJECT_DIR" init -b main >/dev/null
    echo "[ok]   git init"
fi

if [[ -n "$REMOTE_URL" ]]; then
    if git -C "$PROJECT_DIR" remote get-url origin &>/dev/null; then
        echo "[skip] git remote 'origin' already set"
    else
        git -C "$PROJECT_DIR" remote add origin "$REMOTE_URL"
        echo "[ok]   git remote origin -> $REMOTE_URL"
    fi
fi

echo "Done. Fill in .codeos/00-project/CLAUDE.md, then use .codeos/toolkit/dba/03-prompts/workflow/support-solution-charter.md to establish and approve the Solution Charter before the first Specification Package."
