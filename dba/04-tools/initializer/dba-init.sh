#!/usr/bin/env bash
# dba-init.sh — Initialize a new DBA/IDS project
#
# Usage (from new project root):
#   bash /path/to/Codeos/dba/04-tools/initializer/dba-init.sh [project-name] [remote-url]
#
# What it does:
#   1. Creates .codeos symlink → Codeos toolkit
#   2. Creates project directory structure
#   3. Generates CLAUDE.md and AGENTS.md from templates
#   4. Generates docs/conventions.md
#   5. Initializes git repo (if not already one)
#   6. Adds git remote (if remote URL provided)

set -euo pipefail

# Resolve paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEOS_PATH="$(cd "$SCRIPT_DIR/../../.." && pwd)"
PROJECT_DIR="$PWD"
PROJECT_NAME="${1:-$(basename "$PROJECT_DIR")}"
REMOTE_URL="${2:-}"

echo ""
echo "DBA Project Init"
echo "================"
echo "Project name : $PROJECT_NAME"
echo "Project dir  : $PROJECT_DIR"
echo "Toolkit path : $CODEOS_PATH"
if [ -n "$REMOTE_URL" ]; then
echo "Remote URL   : $REMOTE_URL"
fi
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
    "features"
    "backlog"
    "refinements/arch"
)

for dir in "${DIRS[@]}"; do
    if [ -d "$PROJECT_DIR/$dir" ]; then
        echo "[skip] $dir/ already exists"
    else
        mkdir -p "$PROJECT_DIR/$dir"
        echo "[ok]   $dir/"
    fi
done

# ── 3. Feature registry ────────────────────────────────────────────────────

REGISTRY="$PROJECT_DIR/features/registry.yaml"
REGISTRY_TEMPLATE="$CODEOS_PATH/dba/05-guidance/templates/feature-registry.yaml"

if [ -f "$REGISTRY" ]; then
    echo "[skip] features/registry.yaml already exists"
else
    cp "$REGISTRY_TEMPLATE" "$REGISTRY"
    echo "[ok]   features/registry.yaml (from template — edit to replace example entries)"
fi

# ── 4. Runtime event log ────────────────────────────────────────────────────

EVENTS_LOG="$PROJECT_DIR/events/runtime_events.jsonl"
if [ -f "$EVENTS_LOG" ]; then
    echo "[skip] events/runtime_events.jsonl already exists"
else
    touch "$EVENTS_LOG"
    echo "[ok]   events/runtime_events.jsonl"
fi

# ── 5. Project CLAUDE.md ────────────────────────────────────────────────────

PROJECT_CLAUDE="$PROJECT_DIR/CLAUDE.md"
TEMPLATE="$CODEOS_PATH/dba/05-guidance/templates/project-CLAUDE.md"

if [ -f "$PROJECT_CLAUDE" ]; then
    echo "[skip] CLAUDE.md already exists"
else
    sed "s/\[PROJECT_NAME\]/$PROJECT_NAME/g" "$TEMPLATE" > "$PROJECT_CLAUDE"
    echo "[ok]   CLAUDE.md (from template)"
fi

# ── 6. Project AGENTS.md ────────────────────────────────────────────────────

PROJECT_AGENTS="$PROJECT_DIR/AGENTS.md"
AGENTS_TEMPLATE="$CODEOS_PATH/dba/05-guidance/templates/project-AGENTS.md"

if [ -f "$PROJECT_AGENTS" ]; then
    echo "[skip] AGENTS.md already exists"
else
    cp "$AGENTS_TEMPLATE" "$PROJECT_AGENTS"
    echo "[ok]   AGENTS.md (from template)"
fi

# ── 7. Naming conventions ───────────────────────────────────────────────────

CONVENTIONS="$PROJECT_DIR/docs/conventions.md"
if [ -f "$CONVENTIONS" ]; then
    echo "[skip] docs/conventions.md already exists"
else
    cp "$CODEOS_PATH/dba/05-guidance/templates/conventions.md" "$CONVENTIONS"
    echo "[ok]   docs/conventions.md (from template)"
fi

# ── 8. Codebase digest placeholder ─────────────────────────────────────────

DIGEST="$PROJECT_DIR/docs/codebase-digest.md"
if [ -f "$DIGEST" ]; then
    echo "[skip] docs/codebase-digest.md already exists"
else
    cp "$CODEOS_PATH/dba/05-guidance/templates/codebase-digest.md" "$DIGEST"
    sed -i "s/\[PROJECT_NAME\]/$PROJECT_NAME/g" "$DIGEST"
    echo "[ok]   docs/codebase-digest.md (template — complete after first implementation)"
fi

# ── 9. Implementation Profile ────────────────────────────────────────────────

ARCH_DIR="$PROJECT_DIR/architecture"
PROFILE="$ARCH_DIR/implementation-profile.yaml"
PROFILE_TEMPLATE="$CODEOS_PATH/dba/05-guidance/templates/implementation-profile.yaml"

if [ -f "$PROFILE" ]; then
    echo "[skip] architecture/implementation-profile.yaml already exists"
else
    mkdir -p "$ARCH_DIR"
    cp "$PROFILE_TEMPLATE" "$PROFILE"
    echo "[ok]   architecture/implementation-profile.yaml (from template — status: proposed, non-binding)"
fi

# ── 10. Git init ────────────────────────────────────────────────────────────

if [ -d "$PROJECT_DIR/.git" ]; then
    echo "[skip] git repo already exists"
else
    git -C "$PROJECT_DIR" init -b main
    echo "[ok]   git init (branch: main)"
fi

# ── 11. Git remote ────────────────────────────────────────────────────────

if [ -n "$REMOTE_URL" ]; then
    if git -C "$PROJECT_DIR" remote get-url origin &>/dev/null; then
        echo "[skip] git remote 'origin' already set"
    else
        git -C "$PROJECT_DIR" remote add origin "$REMOTE_URL"
        echo "[ok]   git remote origin → $REMOTE_URL"
    fi
fi

# ── 12. Reviewer config ───────────────────────────────────────────────────

REVIEWER_TOML="$PROJECT_DIR/reviewer.toml"
REVIEWER_TEMPLATE="$CODEOS_PATH/dba/05-guidance/templates/reviewer.toml"

if [ -f "$REVIEWER_TOML" ]; then
    echo "[skip] reviewer.toml already exists"
elif [ -f "$REVIEWER_TEMPLATE" ]; then
    cp "$REVIEWER_TEMPLATE" "$REVIEWER_TOML"
    echo "[ok]   reviewer.toml (from template — edit to change provider)"
else
    echo "[warn] reviewer.toml template not found at $REVIEWER_TEMPLATE — skipping"
fi

# ── 13. Done ──────────────────────────────────────────────────────────────

echo ""
echo "Done. Project initialized."
echo ""
echo "Next steps:"
echo "  1. Open CLAUDE.md and fill in the project intent"
echo "  2. Edit features/registry.yaml — replace the example entry with real features"
echo "  3. Start Claude Code: claude"
echo "  4. Tell Claude: 'Read .codeos/dba-system.md' (the active DBA entrypoint)"
echo "  5. Paste .codeos/dba/03-prompts/workflow/00-session-start.md to begin"
echo ""
echo "Session type choices (from 00-session-start.md):"
echo "  A — Feature Brief (new feature discovery)"
echo "  B — Feature Stage Work (Stages 1–9)"
echo "  C — Architectural Refinement (structural changes)"
echo "  D — Existing Codebase Onboarding (working code, no DBA artifacts)"
echo ""
echo "To create your first feature brief:"
echo "  cp .codeos/dba/05-guidance/templates/feature-brief.md backlog/[feature_id].md"
echo "  # Complete the brief, then use it as Stage 1 input"
echo ""
echo "Optional — Codebase Digest (structural orientation for Claude):"
echo "  Complete docs/codebase-digest.md after your first implementation is in place."
echo "  Claude will read it at session start (Step 2b). Generate from openlore,"
echo "  static analysis, or manual inspection. See dba/05-guidance/templates/codebase-digest.md."
echo ""
echo "Implementation Profile (architecture/implementation-profile.yaml):"
echo "  Scaffolded as a non-binding proposal (status: proposed, primary_language: rust)."
echo "  Edit it, or leave it as-is, then have a human explicitly approve it before"
echo "  Stage 4 relies on it. See implementation_profile_policy via .codeos/dba-system.md."
echo ""
