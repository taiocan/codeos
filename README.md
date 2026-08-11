# Codeos — DBA Toolkit

A symlinkable toolkit for developing software using **Declarative Behavioral Architecture (DBA)** / **Intent-Driven System (IDS)** methodology with Claude Code.

## What This Is

A human-governed AI development workflow where people define governed meaning, Claude produces
artifacts and implementation evidence, and the selected doctrine supplies the current semantic
guarantees. README does not duplicate those guarantees.

The 9-step loop:
```
Intent → Contracts → Event Schema → Implementation → Tests →
Runtime Execution → Reconciliation Review → Replay Verification → Targeted Refinement
```

Current operational consequences are located by searching for `DOCTRINE ADAPTER` in the stage
prompts. The doctrine remains authoritative when explanatory text disagrees.

## Quick Start

```bash
# In a new project directory:
bash /home/arc/projects/claude/Codeos/scripts/dba-init.sh [project-name] [remote-url]

# Example:
bash /home/arc/projects/claude/Codeos/scripts/dba-init.sh my-project https://github.com/user/my-project.git

# Then in Claude Code:
# Paste the contents of .codeos/prompts/00-session-start.md
```

## Directory Structure

**Two CLAUDE.md surfaces.** The toolkit repo and the projects that use it operate in
different modes, so the doctrine and the toolkit's own operating guide are separate files:
- `dba-system.md` — the stable downstream **DBA entrypoint**. It selects the active configuration,
  which selects the authoritative doctrine, policies, and reviewer tool contract. Downstream
  projects load it via `.codeos/dba-system.md`.
- `CLAUDE.md` (toolkit repo root) — the **Codeos Self-Development** guide: minimal governance for
  ordinary work, with explicit human control for consequential semantic changes.
  Claude auto-reads it when working in this repo.

```
Codeos/
├── CLAUDE.md          — Codeos Self-Development guide (governs toolkit changes)
├── dba-system.md      — Downstream DBA entrypoint (loaded via .codeos/dba-system.md)
├── dba/               — Version-selected doctrine, policies, tool contract, and configuration
├── README.md          — This file
│
├── prompts/           — Sequential stage prompts (paste to Claude at each stage)
│   ├── 00-session-start.md
│   ├── 01-intent.md
│   ├── 02-contract.md
│   ├── 03-event-schema.md
│   ├── 04-implement.md
│   ├── 05-tests.md
│   ├── 06-observe.md
│   ├── 07-reconcile.md
│   ├── 08-replay.md
│   └── 09-refine.md
│
├── templates/         — Fill-in-the-blank templates for each artifact
│   ├── intent.md
│   ├── contract.md
│   ├── event-schema.md
│   ├── feature-spec.md
│   ├── refinement.md
│   └── project-CLAUDE.md
│
├── scripts/
│   └── dba-init.sh    — Scaffolds a new project (args: [project-name] [remote-url])
│
└── Archive/           — Prior design artifacts and reference material
```

## Per-Project Structure (created by dba-init.sh)

```
myproject/
├── .codeos -> /home/arc/projects/claude/Codeos   (symlink)
├── CLAUDE.md          — Project-level instructions; references .codeos/dba-system.md
├── intents/           — Feature intents (one .md per feature)
├── contracts/         — Behavioral contracts (one .md per feature)
├── events/
│   ├── event_schema.md            — Event definitions
│   └── runtime_events.jsonl       — Append-only runtime log
├── modules/           — Implementation code
├── tests/
│   ├── behavioral/    — Behavioral outcome tests
│   └── replay/        — Replay verification tests
└── docs/
    └── conventions.md — Naming conventions
```

## The DBA Development Loop

### Before Every Session
Paste `prompts/00-session-start.md` to orient Claude.

### Starting a New Project for the First Time

After running `dba-init.sh` (with an optional project name and remote URL), follow these steps before opening Claude:

**Step 1 — Fill in the project `CLAUDE.md` that the script generated.**
Open it and complete:
- **Project intent** — one paragraph describing what this project exists to do, in actor + outcome language (not implementation details)
- **Language/runtime, test framework, event prefix** — the project-specific conventions block at the bottom
- Leave the Active Features table empty — you add rows as features are created and approved

**Step 2 — Open Claude Code in the project directory and paste `prompts/00-session-start.md`.**
Fill in the [BRACKETS] before pasting:
- "Today's goal" — the first feature you want to start (e.g., "Start Stage 1 Intent for user_login")
- "Current feature states" — write "No features started yet" or leave the table blank
- "This session's scope" — name the first feature; tell Claude not to start any others

**Step 3 — Claude reads the active DBA components and the project `CLAUDE.md` and confirms — verify it names both.**
1. `.codeos/dba-system.md` — stable entrypoint to the active DBA configuration and selected components
2. Project `CLAUDE.md` — the file you just filled in; Claude reads the project intent and conventions

**Step 4 — Draft the Specification Package.**
Start with `prompts/01-intent.md`, then continue through Contract and Event Schema. Stage 3 owns the
current `specification-approval` boundary.

There are no existing project artifacts to read at this point — Claude starts from the active DBA
components selected through `.codeos/dba-system.md` and the project `CLAUDE.md`. Do not paste a
resumption prompt; go straight to the stage prompt.

### Resuming After a Crash or Session Break

When starting a fresh session on a project where work is already in progress:

**Step 1 — Update the project `CLAUDE.md` Active Features table before opening Claude.**
This table is the only cross-session state record. Verify every row is accurate:
- Feature ID and description are correct
- Current Stage reflects the last completed stage (not what was in progress when the session ended)
- Status is one of: `DRAFT` / `APPROVED` / `IN_PROGRESS` / `COMPLETE`

If a feature was mid-stage when the session ended, set its status to `IN_PROGRESS` and its stage to the last completed stage number.

**Step 2 — Open Claude Code in the project directory and paste `prompts/00-session-start.md`.**
Fill in the [BRACKETS] before pasting:
- "Today's goal" — what you want to complete this session
- "Current feature states" — copy from the Active Features table you just updated
- "This session's scope" — be explicit about which features are in scope

**Step 3 — Claude reads these two files automatically (verify it confirms both):**
1. `.codeos/dba-system.md` — entrypoint to the active DBA configuration and selected rules
2. Project `CLAUDE.md` — your Active Features table and project-specific conventions

**Step 4 — For each in-progress feature, direct Claude to read the existing approved artifacts.**
Tell Claude which feature to resume, then say: "Read the existing artifacts for [feature_id] before proceeding."
Claude will read:
- `intents/[feature_id].md` — approved intent (do not re-derive)
- `contracts/[feature_id]_contract.md` — approved contract (do not re-derive)
- `events/[feature_id]_schema.md` — approved event schema (do not re-derive)
- `modules/[feature_id]/` — existing implementation (if Stage 4 was completed)
- `tests/` — existing tests (if Stage 5 was completed)

**Do not ask Claude to rewrite or re-derive unchanged governed artifacts.** Route resumed work
through the Stage 4 `delivery-entry` adapter, which owns compatibility and entry checks.

**What Claude cannot recover automatically:** any decisions or clarifications that happened only in conversation and were never written into an artifact. If a decision was important, it should have been captured in an artifact at the time. If it wasn't, re-state it explicitly when you resume.

### For Each Feature
Work through stages in order. Decision behavior comes from the selected doctrine and the named
boundary adapters:

| Stage | File | Purpose | Claude produces | Boundary owner |
|---|---|---|---|---|
| 1 | `prompts/01-intent.md` | Capture *why* the feature exists — actor + outcome, no implementation details | Draft Intent | — |
| 2 | `prompts/02-contract.md` | Translate Intent into independently testable behavior | Draft Contract | — |
| 3 | `prompts/03-event-schema.md` | Complete and cross-check the Specification Package | Draft Event Schema + package review | `specification-approval` adapter |
| 4 | `prompts/04-implement.md` | Implement governed behavior | Code in `modules/` | `delivery-entry` adapter |
| 5 | `prompts/05-tests.md` | Verify observable outcomes | Tests in `tests/` | — |
| 6 | `prompts/06-observe.md` | Capture trustworthy runtime evidence | Runtime evidence | — |
| 7 | `prompts/07-reconcile.md` | Surface gaps and mismatches | Reconciliation evidence | — |
| 8 | `prompts/08-replay.md` | Verify replay and conformance | Final review package | `final-acceptance` adapter |
| 9 | `prompts/09-refine.md` | Apply targeted refinement and return to verification | Targeted refinement | Stage 8 adapter |

## Stage Purposes

### Stage 0 — Session Start
**Purpose:** Orient Claude to DBA mode before any work begins. The human fills in today's goal, the current stage and status of every active feature, and any session-specific forbidden actions. Claude reads both `.codeos/dba-system.md` and the project `CLAUDE.md`, then stops and waits for the human to begin.
**Key constraint:** Claude does not produce artifacts, write code, or analyze anything until the human explicitly says to proceed.

### Stage 1 — Intent
**Purpose:** Capture *why* a feature exists — not how it works. Every statement uses actor + outcome form and stays completely implementation-free. This document is the foundation every downstream artifact derives from; ambiguity here propagates forward into contracts, schema, and code.
**Key constraint:** No implementation details, APIs, databases, frameworks, observability mechanics, or feature decomposition. Guarantees must be enforceable and testable. If the intent fills more than one screen it is too broad.

### Stage 2 — Behavioral Contracts
**Purpose:** Translate the current Intent into independently testable observable truth while both remain open to revision. Contracts describe only what can be seen from the outside — emitted events and system state — never internal logic or code structure.
**Key constraint:** Intent and Contract remain open to correction while the specification is being completed.

### Stage 3 — Event Schema
**Purpose:** Define the event spine and verify all three specification artifacts together. This
stage owns the `specification-approval` adapter.
**Key constraint:** Every contract scenario maps to at least one event; every named contract failure maps to exactly one FAILURE event. `correlation_id` is mandatory on every event without exception.

### Stage 4 — Implementation
**Purpose:** Satisfy governed specification using normal internal engineering choices. This stage
owns the `delivery-entry` adapter.
**Key constraint:** Apply the constraints selected by the active doctrine.

### Stage 5 — Tests
**Purpose:** Write behavioral truth anchors that fail if observable behavior deviates from contracts. Tests cover happy paths, every named failure mode, telemetry correctness (correlation IDs, required fields), and idempotency if the contract specifies it. A replay test captures and re-runs the event stream to verify determinism.
**Key constraint:** Tests must not touch private methods, internal state, or intermediate computations. All assertions use event names exactly as they appear in the approved schema.

### Stage 6 — Runtime Execution
**Purpose:** Run representative scenarios when permitted and capture evidence of what occurred.
**Key constraint:** Apply the evidence and authorization rules from the selected doctrine.

### Stage 7 — Reconciliation Review
**Purpose:** Perform a structural audit comparing all six layers — intent, contract, event schema, implementation, tests, and runtime events — to surface gaps, mismatches, and missing coverage. Every non-ALIGNED finding names the stage(s) that must be re-run and the minimal targeted fix required.
**Key constraint:** This is not a code review and does not suggest rewrites. Status is one of ALIGNED, GAP, MISMATCH, or MISSING — nothing else.

### Stage 8 — Replay Verification
**Purpose:** Confirm the system is deterministically replayable. Every runtime event must conform to the approved schema, correlation chains must be complete (start + end), and re-running the same inputs must produce events consistent with the schema. The replay tests in `tests/replay/` are the executable form of this guarantee.
**Key constraint:** Broken correlation chains, out-of-order events, and event types absent from the schema are all conformance failures that must be resolved before proceeding.

### Stage 9 — Targeted Refinement
**Purpose:** Apply the smallest effective problem-driven fix to each observed issue. Valid triggers are recurring failures in the event log, reconciliation gaps, replay failures, observability gaps, or human-approved intent evolution. Refinements are ordered by cost — observability changes first, structural changes last.
**Key constraint:** Apply the selected doctrine's escalation and verification rules, then return to Stage 8.

## Governing Rules

Read `dba-system.md` and its selected doctrine. This README describes navigation and stage purpose;
it does not independently define authority, approval cadence, escalation, or evidence semantics.

The current lifecycle has `specification-approval`, `delivery-entry`, `final-acceptance`, and the
conditional `architecture-entry` doctrine adapters. This count is descriptive, not an invariant:
add or remove an adapter only when a genuine execution boundary changes.

For a future doctrine version:

1. Create the new doctrine version.
2. Identify which doctrine semantics changed.
3. Find current adapters with `rg "DOCTRINE ADAPTER: [a-z-]+"`.
4. Change only adapters whose actual boundary behavior changed.
5. Version another governed component only if its own normative semantics changed.
6. Activate the new DBA configuration.

## Reference Material

Current project definitions: `terminology.md`

Prior design sessions and DBA theory: `Archive/`

- `Archive/intent.md` — Loop description and minimal stack
- `Archive/terminology.md` — DBA terminology with worked examples
- `Archive/intent-S0.md` through `intent-S7.md` — Stage-by-stage philosophy
- `Archive/S1-intent/intent-examples.md` — Complete loop walkthrough (counter example)
