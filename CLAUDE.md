# Codeos DBA Toolkit — Master System Instructions

## Mode Declaration

You are operating in **Declarative Behavioral Architecture (DBA)** mode, also called **Intent-Driven System (IDS)** mode.

This toolkit is located at `.codeos/` (symlinked from `/home/arc/projects/claude/Codeos`).

Read this file fully at the start of every session before doing anything else.

---

## The Non-Negotiable Rules

1. **Every stage transition requires explicit human approval.** You NEVER advance to the next stage without a human "APPROVED", "approved", "yes proceed", or equivalent.
2. **You NEVER implement before intent + contract + event schema are all approved.**
3. **You NEVER add abstractions, patterns, or behaviors beyond what the current intent + contract + event schema specifies.**
4. **You NEVER emit events not listed in the approved event schema.**
5. **You NEVER invent hidden behavior** — all behavior must be traceable to an approved artifact.
6. **After producing any stage output, you STOP and state: `AWAITING HUMAN APPROVAL`.**

---

## The 9-Step DBA Development Loop

Every feature follows this exact sequence. No skipping.

```
STEP 1 — Intent
  Human writes raw feature description.
  AI verifies, corrects format, flags missing information.
  Output: intents/[feature_id].md
  Gate: human approves intent before step 2.

STEP 2 — Behavioral Contracts
  AI derives BDD-style contracts from approved intent.
  Output: contracts/[feature_id]_contract.md
  Gate: human approves contracts before step 3.

STEP 3 — Event Schema
  AI defines the complete event spine from approved intent + contracts.
  This is the most constraining artifact — implementation is locked to it.
  Output: events/[feature_id]_schema.md (or events/event_schema.md)
  Gate: human approves schema before step 4.

STEP 4 — AI Implementation
  AI implements ONLY what is specified by the three approved artifacts.
  Output: code in modules/
  Gate: human approves implementation before step 5.

STEP 5 — Tests
  AI writes behavioral tests and replay tests.
  Output: tests/behavioral/ and tests/replay/
  Gate: human approves tests before step 6.

STEP 6 — Runtime Execution
  Human runs the implementation.
  System emits events to events/runtime_events.jsonl (append-only).

STEP 7 — AI Reconciliation Review
  AI compares intent / contracts / event schema / implementation / tests / runtime events.
  Produces reconciliation table with ALIGNED / GAP / MISMATCH / MISSING status.
  Gate: human approves before step 8 or directs return to earlier step.

STEP 8 — Replay Verification
  AI verifies runtime_events.jsonl conforms to schema and contract sequence.
  Gate: human approves before step 9 or directs return.

STEP 9 — Targeted Refinement
  AI proposes the smallest effective change for each observed problem.
  Affected stages are re-run. No full rewrites.
  Gate: human approves each refinement individually.
```

---

## What You Do at Each Stage

Use the corresponding prompt file from `.codeos/prompts/` for detailed instructions:

| Stage | File |
|---|---|
| Session start | `.codeos/prompts/00-session-start.md` |
| Stage 1: Intent | `.codeos/prompts/01-intent.md` |
| Stage 2: Contracts | `.codeos/prompts/02-contract.md` |
| Stage 3: Event Schema | `.codeos/prompts/03-event-schema.md` |
| Stage 4: Implementation | `.codeos/prompts/04-implement.md` |
| Stage 5: Tests | `.codeos/prompts/05-tests.md` |
| Stage 6: Observation | `.codeos/prompts/06-observe.md` |
| Stage 7: Reconcile | `.codeos/prompts/07-reconcile.md` |
| Stage 8: Replay | `.codeos/prompts/08-replay.md` |
| Stage 9: Refine | `.codeos/prompts/09-refine.md` |

Use the corresponding template from `.codeos/templates/` when producing artifacts:

| Artifact | Template |
|---|---|
| Feature intent | `.codeos/templates/intent.md` |
| Behavioral contract | `.codeos/templates/contract.md` |
| Event schema | `.codeos/templates/event-schema.md` |
| Feature specification | `.codeos/templates/feature-spec.md` |
| Refinement log | `.codeos/templates/refinement.md` |

---

## What You NEVER Do

- Implement before intent + contract + event schema are all APPROVED
- Add abstractions not demanded by the contracts
- Add "just in case" error handling not listed in the contract's failure modes
- Emit events not in the approved event schema
- Move to the next stage without explicit human approval
- Suggest full rewrites — only targeted, localized changes
- Add autonomous planning, self-direction, or multi-step autonomous execution
- Modify `events/runtime_events.jsonl` — it is append-only

---

## Naming Conventions

See `.codeos/templates/conventions.md` for the authoritative naming convention reference.

---

## File Layout

```
project/
├── .codeos/                      ← this toolkit (symlink)
├── CLAUDE.md                     ← project-level instructions (references this file)
├── intents/
│   └── [feature_id].md           ← one per feature
├── contracts/
│   └── [feature_id]_contract.md  ← one per feature
├── events/
│   ├── [feature_id]_schema.md    ← event schema per feature (or shared event_schema.md)
│   └── runtime_events.jsonl      ← append-only runtime log
├── modules/                      ← actual implementation code
└── tests/
    ├── behavioral/               ← behavioral outcome tests
    └── replay/                   ← replay verification tests
```

---

## DBA Vocabulary

| Term | Definition |
|---|---|
| **Intent** | Why a feature exists. Actor + outcome form. No implementation details. |
| **Behavioral Contract** | Observable truths derived from intent. BDD Given/When/Then. |
| **Event Spine** | The complete ordered set of events a feature is permitted to emit. |
| **Observational Event** | Raw runtime fact (e.g., `RequestReceived`). |
| **Behavioral Event** | Verified outcome (e.g., `CartItemAdded`). |
| **Failure Event** | Classified error condition (e.g., `CartItemAddFailed`). |
| **Reconciliation Review** | Structural comparison of all artifacts against each other for gaps/mismatches. |
| **Replay Verification** | Confirming runtime event log conforms to schema and contract sequence. |
| **Targeted Refinement** | Smallest effective change for a specific observed problem. Not a rewrite. |
| **Correlation ID** | UUID that links all events from a single feature execution chain. |

---

## How to Use the Toolkit in a New Project

1. Run from the new project root: `bash /home/arc/projects/claude/Codeos/scripts/dba-init.sh`
2. This creates `.codeos` symlink, all required directories, and a project `CLAUDE.md`
3. Start Claude Code in the project directory
4. Claude reads the project `CLAUDE.md` which directs it to read this file
5. Human pastes `.codeos/prompts/00-session-start.md` to begin a session
