# [PROJECT_NAME] — CLAUDE.md

This project uses **Declarative Behavioral Architecture (DBA)** / **Intent-Driven System (IDS)** methodology.

## Toolkit

The DBA toolkit is at `.codeos/` (symlinked from `/home/arc/projects/claude/Codeos`).

**At the start of every Claude Code session — read `.codeos/dba-system.md` before doing any
work; it selects the authoritative Codeos DBA components.**
1. Read `.codeos/dba-system.md`, its active configuration, and the selected doctrine component
2. Read `.codeos/prompts/00-session-start.md` — session orientation template (Step 3 generates the Current Verified State: branch, commit, working tree, and registry/filesystem agreement)
3. Check the Active Features table below for current stage status
4. Ask what the human wants to work on this session, then STOP and wait

## Project Intent

<!-- Human fills in after running dba-init.sh — what this project exists to do -->
[Project intent goes here]

## Active Features

<!-- Human maintains this table — update stage and status as work progresses -->

| Feature ID | Description | Current Stage | Status |
|---|---|---|---|
| | | | |

Stages: 1-Intent / 2-Contract / 3-Schema / 4-Implement / 5-Tests / 6-Observe / 7-Reconcile / 8-Replay / 9-Refine

Status: DRAFT / APPROVED / IN_PROGRESS / COMPLETE

## Human Approval Gates

Every stage output requires explicit human approval before Claude advances.

After presenting any stage output, Claude **STOPS** and states: `AWAITING HUMAN APPROVAL`

Valid approval signals: `APPROVED`, `approved`, `yes proceed`, `lgtm`
Anything else is treated as a revision request.

## Runtime Events

All runtime events are appended to: `events/runtime_events.jsonl`
This file is **append-only**. Claude must never delete or modify existing lines.

## Project-Specific Conventions

<!-- Human fills in any project-specific event prefixes, module names, tech stack -->
Language/runtime: [e.g., Rust, Python, Node.js]
Test framework: [e.g., cargo test, pytest, vitest]
Event prefix: [e.g., all events prefixed with ProjectName]
