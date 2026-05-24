# Codeos — DBA Toolkit

A symlinkable toolkit for developing software using **Declarative Behavioral Architecture (DBA)** / **Intent-Driven System (IDS)** methodology with Claude Code.

## What This Is

A human-gated AI development workflow where:
- **Humans define** intent, approve every artifact, and control every stage transition
- **Claude proposes** contracts, event schemas, implementations, tests, and refinements
- **Runtime events** become the source of operational truth

The 9-step loop:
```
Intent → Contracts → Event Schema → Implementation → Tests →
Runtime Execution → Reconciliation Review → Replay Verification → Targeted Refinement
```

Every step requires explicit human approval before Claude advances.

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

```
Codeos/
├── CLAUDE.md          — Master DBA system instructions (Claude reads this)
├── README.md          — This file
│
├── prompts/           — Stage-gated prompts (paste to Claude at each stage)
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
├── CLAUDE.md          — Project-level instructions; references .codeos/CLAUDE.md
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

**Step 3 — Claude reads both CLAUDE.md files and confirms — verify it names both.**
1. `.codeos/CLAUDE.md` — master DBA rules and the 9-step loop (Claude reads this; you do not edit it)
2. Project `CLAUDE.md` — the file you just filled in; Claude reads the project intent and conventions

**Step 4 — Paste `prompts/01-intent.md` and describe the first feature.**
Claude produces `intents/[feature_id].md`. After you approve it, add a row to the Active Features table in project `CLAUDE.md` (Stage 1, status APPROVED) before telling Claude to proceed to Stage 2.

There are no existing artifacts to read at this point — Claude starts from scratch using only the two CLAUDE.md files as context. Do not paste a resumption prompt; go straight to the stage prompt.

### Resuming After a Crash or Session Break

When starting a fresh session on a project where work is already in progress:

**Step 1 — Update the project `CLAUDE.md` Active Features table before opening Claude.**
This table is the only cross-session state record. Verify every row is accurate:
- Feature ID and description are correct
- Current Stage reflects the last *approved* stage (not what was in progress when the session ended)
- Status is one of: `DRAFT` / `APPROVED` / `IN_PROGRESS` / `COMPLETE`

If a feature was mid-stage when the session ended, set its status to `IN_PROGRESS` and its stage to the last *completed and approved* stage number.

**Step 2 — Open Claude Code in the project directory and paste `prompts/00-session-start.md`.**
Fill in the [BRACKETS] before pasting:
- "Today's goal" — what you want to complete this session
- "Current feature states" — copy from the Active Features table you just updated
- "This session's scope" — be explicit about which features are in scope

**Step 3 — Claude reads these two files automatically (verify it confirms both):**
1. `.codeos/CLAUDE.md` — master DBA rules, the 9-step loop, and non-negotiable constraints
2. Project `CLAUDE.md` — your Active Features table and project-specific conventions

**Step 4 — For each in-progress feature, direct Claude to read the existing approved artifacts.**
Tell Claude which feature to resume, then say: "Read the existing artifacts for [feature_id] before proceeding."
Claude will read:
- `intents/[feature_id].md` — approved intent (do not re-derive)
- `contracts/[feature_id]_contract.md` — approved contract (do not re-derive)
- `events/[feature_id]_schema.md` — approved event schema (do not re-derive)
- `modules/[feature_id]/` — existing implementation (if Stage 4 was completed)
- `tests/` — existing tests (if Stage 5 was completed)

**Do not ask Claude to rewrite or re-derive any artifact that already has `status: APPROVED`.** Treat approved files as ground truth. The session resumes at the next unapproved stage.

**What Claude cannot recover automatically:** any decisions or clarifications that happened only in conversation and were never written into an artifact. If a decision was important, it should have been captured in an artifact at the time. If it wasn't, re-state it explicitly when you resume.

### For Each Feature
Work through stages in order, pausing for human approval at each gate:

| Stage | File | Purpose | Claude produces | Gate |
|---|---|---|---|---|
| 1 | `prompts/01-intent.md` | Capture *why* the feature exists — actor + outcome, no implementation details | `intents/[id].md` | Human approves intent |
| 2 | `prompts/02-contract.md` | Translate intent into independently testable, observable-only behavioral truth | `contracts/[id]_contract.md` | Human approves contract |
| 3 | `prompts/03-event-schema.md` | Define the event spine that structurally constrains all future implementation | `events/[id]_schema.md` | Human approves schema |
| 4 | `prompts/04-implement.md` | Satisfy every contract clause and emit every schema event — nothing more | Code in `modules/` | Human approves implementation |
| 5 | `prompts/05-tests.md` | Write behavioral truth anchors verifying observable outcomes, not internal structure | Tests in `tests/` | Human approves tests |
| 6 | `prompts/06-observe.md` | Human runs the system; `runtime_events.jsonl` becomes operational truth | Advisory only | Human confirms events captured |
| 7 | `prompts/07-reconcile.md` | Structural audit of all six layers (intent → runtime) to surface gaps and mismatches | Reconciliation table | Human approves or directs return |
| 8 | `prompts/08-replay.md` | Confirm deterministic replayability: schema conformance, complete chains, same inputs → same events | Replay report | Human approves or marks complete |
| 9 | `prompts/09-refine.md` | Apply the smallest evidence-driven fix — no rewrites, no speculative improvement | Targeted refinement proposals | Human approves each refinement |

## Stage Purposes

### Stage 0 — Session Start
**Purpose:** Orient Claude to DBA mode before any work begins. The human fills in today's goal, the current stage and status of every active feature, and any session-specific forbidden actions. Claude reads both `.codeos/CLAUDE.md` and the project `CLAUDE.md`, then stops and waits for the human to begin.
**Key constraint:** Claude does not produce artifacts, write code, or analyze anything until the human explicitly says to proceed.

### Stage 1 — Intent
**Purpose:** Capture *why* a feature exists — not how it works. Every statement uses actor + outcome form and stays completely implementation-free. This document is the foundation every downstream artifact derives from; ambiguity here propagates forward into contracts, schema, and code.
**Key constraint:** No implementation details, APIs, databases, frameworks, observability mechanics, or feature decomposition. Guarantees must be enforceable and testable. If the intent fills more than one screen it is too broad.

### Stage 2 — Behavioral Contracts
**Purpose:** Translate the approved intent into independently testable observable truth. Contracts describe only what can be seen from the outside — emitted events and system state — never internal logic or code structure. Every failure mode gets a named scenario that becomes a failure event in Stage 3.
**Key constraint:** Every clause must be answerable by looking at emitted events and system state alone. No mention of classes, functions, databases, APIs, or frameworks. All actors must come from the approved intent.

### Stage 3 — Event Schema
**Purpose:** Define the complete event spine that structurally constrains all future implementation. Once approved, the implementation may only emit events listed here — hidden behavior becomes architecturally impossible. Any new behavior requires updating the schema first, then re-approval.
**Key constraint:** Every contract scenario maps to at least one event; every named contract failure maps to exactly one FAILURE event. `correlation_id` is mandatory on every event without exception.

### Stage 4 — Implementation
**Purpose:** Satisfy every approved contract clause and emit every approved schema event — nothing more. Claude acts as a constrained satisfier, not a creative designer. The first thing wired up is correlation ID propagation and event emission; all behavior is traceable to an approved artifact.
**Key constraint:** No abstractions, helper layers, or error handling beyond what the contracts explicitly require. If a contract clause cannot be satisfied without adding something unapproved, Claude flags it and stops rather than silently adding it.

### Stage 5 — Tests
**Purpose:** Write behavioral truth anchors that fail if observable behavior deviates from contracts. Tests cover happy paths, every named failure mode, telemetry correctness (correlation IDs, required fields), and idempotency if the contract specifies it. A replay test captures and re-runs the event stream to verify determinism.
**Key constraint:** Tests must not touch private methods, internal state, or intermediate computations. All assertions use event names exactly as they appear in the approved schema.

### Stage 6 — Runtime Execution
**Purpose:** The human runs the implementation; `events/runtime_events.jsonl` becomes the operational source of truth. Claude's role is advisory — helping set up event capture infrastructure if needed. No stage transition happens until the human confirms events are in the log.
**Key constraint:** `runtime_events.jsonl` is append-only. Claude does not advance to Stage 7 until the human explicitly confirms runtime events are available.

### Stage 7 — Reconciliation Review
**Purpose:** Perform a structural audit comparing all six layers — intent, contract, event schema, implementation, tests, and runtime events — to surface gaps, mismatches, and missing coverage. Every non-ALIGNED finding names the stage(s) that must be re-run and the minimal targeted fix required.
**Key constraint:** This is not a code review and does not suggest rewrites. Status is one of ALIGNED, GAP, MISMATCH, or MISSING — nothing else.

### Stage 8 — Replay Verification
**Purpose:** Confirm the system is deterministically replayable. Every runtime event must conform to the approved schema, correlation chains must be complete (start + end), and re-running the same inputs must produce events consistent with the schema. The replay tests in `tests/replay/` are the executable form of this guarantee.
**Key constraint:** Broken correlation chains, out-of-order events, and event types absent from the schema are all conformance failures that must be resolved before proceeding.

### Stage 9 — Targeted Refinement
**Purpose:** Apply the smallest effective problem-driven fix to each observed issue. Valid triggers are recurring failures in the event log, reconciliation gaps, replay failures, observability gaps, or human-approved intent evolution. Refinements are ordered by cost — observability changes first, structural changes last.
**Key constraint:** No redesigns, no rewrites, no improvements not backed by observed evidence. Each refinement is approved individually before it is applied; after each approval, affected stages are re-run before the next refinement is addressed.

## Key Rules

1. Claude NEVER advances stages without explicit human approval
2. Claude NEVER implements before intent + contract + event schema are approved
3. Claude NEVER emits events not listed in the approved schema
4. `events/runtime_events.jsonl` is append-only — never modified, never deleted
5. Refinements are always targeted and problem-driven — never full rewrites

## Reference Material

Prior design sessions and DBA theory: `Archive/`

- `Archive/intent.md` — Loop description and minimal stack
- `Archive/terminology.md` — DBA terminology with worked examples
- `Archive/intent-S0.md` through `intent-S7.md` — Stage-by-stage philosophy
- `Archive/S1-intent/intent-examples.md` — Complete loop walkthrough (counter example)
