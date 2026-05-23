#!/usr/bin/env bash
# dba-init.sh — Initialize a new DBA/IDS project
#
# Usage (from new project root):
#   bash /home/arc/projects/claude/Codeos/scripts/dba-init.sh [project-name]
#
# What it does:
#   1. Creates .codeos symlink → Codeos toolkit
#   2. Creates project directory structure
#   3. Generates CLAUDE.md from template
#   4. Generates docs/conventions.md

set -euo pipefail

# Resolve paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEOS_PATH="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_DIR="$PWD"
PROJECT_NAME="${1:-$(basename "$PROJECT_DIR")}"

echo ""
echo "DBA Project Init"
echo "================"
echo "Project name : $PROJECT_NAME"
echo "Project dir  : $PROJECT_DIR"
echo "Toolkit path : $CODEOS_PATH"
echo ""

# ── 1. Symlink ─────────────────────────────────────────────────────────────

if [ -L "$PROJECT_DIR/.codeos" ]; then
    echo "[skip] .codeos symlink already exists"
elif [ -e "$PROJECT_DIR/.codeos" ]; then
    echo "[warn] .codeos exists but is not a symlink — skipping"
else
    ln -s "$CODEOS_PATH" "$PROJECT_DIR/.codeos"
    echo "[ok]   .codeos → $CODEOS_PATH"
fi

# ── 2. Directory structure ──────────────────────────────────────────────────

DIRS=(
    "intents"
    "contracts"
    "events"
    "modules"
    "tests/behavioral"
    "tests/replay"
    "docs"
)

for dir in "${DIRS[@]}"; do
    if [ -d "$PROJECT_DIR/$dir" ]; then
        echo "[skip] $dir/ already exists"
    else
        mkdir -p "$PROJECT_DIR/$dir"
        echo "[ok]   $dir/"
    fi
done

# ── 3. Runtime event log ────────────────────────────────────────────────────

EVENTS_LOG="$PROJECT_DIR/events/runtime_events.jsonl"
if [ -f "$EVENTS_LOG" ]; then
    echo "[skip] events/runtime_events.jsonl already exists"
else
    touch "$EVENTS_LOG"
    echo "[ok]   events/runtime_events.jsonl"
fi

# ── 4. Project CLAUDE.md ────────────────────────────────────────────────────

PROJECT_CLAUDE="$PROJECT_DIR/CLAUDE.md"
TEMPLATE="$CODEOS_PATH/templates/project-CLAUDE.md"

if [ -f "$PROJECT_CLAUDE" ]; then
    echo "[skip] CLAUDE.md already exists"
else
    sed "s/\[PROJECT_NAME\]/$PROJECT_NAME/g" "$TEMPLATE" > "$PROJECT_CLAUDE"
    echo "[ok]   CLAUDE.md (from template)"
fi

# ── 5. Naming conventions ───────────────────────────────────────────────────

CONVENTIONS="$PROJECT_DIR/docs/conventions.md"
if [ -f "$CONVENTIONS" ]; then
    echo "[skip] docs/conventions.md already exists"
else
    cp "$CODEOS_PATH/templates/conventions.md" "$CONVENTIONS"
    echo "[ok]   docs/conventions.md (from template)"
fi

# ── Done ────────────────────────────────────────────────────────────────────

echo ""
echo "Done. Project initialized."
echo ""
echo "Next steps:"
echo "  1. Open CLAUDE.md and fill in the project intent"
echo "  2. Update the Active Features table as you create features"
echo "  3. Start Claude Code: claude"
echo "  4. Tell Claude: 'Read .codeos/CLAUDE.md'"
echo "  5. Paste .codeos/prompts/00-session-start.md to begin"
echo ""
echo "To create your first feature intent:"
echo "  cp .codeos/templates/intent.md intents/[feature_id].md"
echo "  # Edit the file, then tell Claude: 'Stage 1 for [feature_id]'"
echo ""
