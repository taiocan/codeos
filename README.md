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
bash /path/to/Codeos/dba/04-tools/initializer/dba-init.sh [project-name] [remote-url]

# Example:
bash /path/to/Codeos/dba/04-tools/initializer/dba-init.sh my-project https://github.com/user/my-project.git

# Then in Claude Code:
# Paste the contents of .codeos/dba/03-prompts/workflow/00-session-start.md
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
├── AGENTS.md          — Codex route to CLAUDE.md
├── CLAUDE.md          — Codeos Self-Development guide (governs toolkit changes)
├── dba-system.md      — Downstream DBA entrypoint (loaded via .codeos/dba-system.md)
├── README.md          — This file
├── dba/               — Downstream DBA package
│   ├── 00-entry/      — Active configuration selection
│   ├── 01-doctrine/   — Fundamental DBA guarantees
│   ├── 02-policies/   — Conditional governance mechanisms
│   ├── 03-prompts/    — Workflow, review, and delegation instructions
│   ├── 04-tools/      — Contracts and implementations grouped by capability
│   ├── 05-guidance/   — Templates, patterns, and terminology
│   └── 06-reference/  — Explanatory and migration documentation
└── maintenance/       — Codeos self-development
    ├── backlog/
    ├── reviews/
    ├── config/
    └── archive/
```

## Per-Project Structure (created by dba-init.sh)

```
myproject/
├── .codeos -> /home/arc/projects/claude/Codeos   (symlink)
├── AGENTS.md          — Codex route to CLAUDE.md
├── CLAUDE.md          — Project-level instructions; references .codeos/dba-system.md
├── intents/           — Feature intents (one .md per feature)
├── contracts/         — Behavioral contracts (one .md per feature)
├── events/
│   ├── [feature_id]_schema.md      — Governed events or external-observation mapping
│   └── runtime_events.jsonl        — Append-only log for features using events mode
├── modules/           — Implementation code
├── tests/
│   ├── behavioral/    — Behavioral outcome tests
│   └── replay/        — Replay verification tests
└── docs/
    └── conventions.md — Naming conventions
```

## The DBA Development Loop

### Before Every Session
Paste `.codeos/dba/03-prompts/workflow/00-session-start.md` to orient Claude.

### Starting a New Project for the First Time

After running `dba-init.sh` (with an optional project name and remote URL), follow these steps before opening Claude:

**Step 1 — Fill in the project `CLAUDE.md` that the script generated.**
Open it and complete:
- **Project intent** — one paragraph describing what this project exists to do, in actor + outcome language (not implementation details)
- **Language/runtime, test framework, event prefix** — the project-specific conventions block at the bottom
- Leave the Active Features table empty — you add rows as features are created and approved

**Step 2 — Open Claude Code in the project directory and use
`.codeos/dba/03-prompts/workflow/00-session-start.md`.** Name the target feature or structural task.

**Step 3 — Claude reads the active DBA components and the project `CLAUDE.md` and confirms — verify it names both.**
1. `.codeos/dba-system.md` — stable entrypoint to the active DBA configuration and selected components
2. Project `CLAUDE.md` — the file you just filled in; Claude reads the project intent and conventions

**Step 4 — Draft the Specification Package.**
Start with `.codeos/dba/03-prompts/workflow/01-intent.md`, then continue through Contract and Event Schema. Stage 3 owns the
current `specification-approval` boundary.

There are no existing project artifacts to read at this point — Claude starts from the active DBA
components selected through `.codeos/dba-system.md` and the project `CLAUDE.md`. Do not paste a
resumption prompt; go straight to the stage prompt.

### Resuming After a Crash or Session Break

When starting a fresh session on a project where work is already in progress:

**Step 1 — Open Claude Code in the project directory and use
`.codeos/dba/03-prompts/workflow/00-session-start.md`.** Name the target feature or task. The prompt
reads its matching registry entry and live artifacts; do not copy repository state into the prompt.

**Step 2 — Claude reads these two files (verify it confirms both):**
1. `.codeos/dba-system.md` — entrypoint to the active DBA configuration and selected rules
2. Project `CLAUDE.md` — your Active Features table and project-specific conventions

**Step 3 — For each in-progress feature, direct Claude to read the existing approved artifacts.**
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
| 1 | `.codeos/dba/03-prompts/workflow/01-intent.md` | Capture *why* the feature exists — actor + outcome, no implementation details | Draft Intent | — |
| 2 | `.codeos/dba/03-prompts/workflow/02-contract.md` | Translate Intent into independently testable behavior | Draft Contract | — |
| 3 | `.codeos/dba/03-prompts/workflow/03-event-schema.md` | Complete and cross-check the Specification Package | Draft Event Schema + package review | `specification-approval` adapter |
| 4 | `.codeos/dba/03-prompts/workflow/04-implement.md` | Implement governed behavior | Code in `modules/` | `delivery-entry` adapter |
| 5 | `.codeos/dba/03-prompts/workflow/05-tests.md` | Verify observable outcomes | Tests in `tests/` | — |
| 6 | `.codeos/dba/03-prompts/workflow/06-observe.md` | Capture trustworthy runtime evidence | Runtime evidence | — |
| 7 | `.codeos/dba/03-prompts/workflow/07-reconcile.md` | Surface gaps and mismatches | Reconciliation evidence | — |
| 8 | `.codeos/dba/03-prompts/workflow/08-replay.md` | Verify replay and conformance | Final review package | `final-acceptance` adapter |
| 9 | `.codeos/dba/03-prompts/workflow/09-refine.md` | Apply targeted refinement and return to verification | Targeted refinement | Stage 8 adapter |

## Stage Purposes

### Stage 0 — Session Start
**Purpose:** Orient Claude from live repository state, classify the target work, and select the
applicable workflow. Claude reads the active DBA entrypoint, project instructions, and only the
registry entry and artifacts relevant to the target.
**Key constraint:** Claude does not produce artifacts, write code, or analyze anything until the human explicitly says to proceed.

### Stage 1 — Intent
**Purpose:** Capture *why* a feature exists — not how it works. Every statement uses actor + outcome form and stays completely implementation-free. This document is the foundation every downstream artifact derives from; ambiguity here propagates forward into contracts, schema, and code.
**Key constraint:** No implementation details, APIs, databases, frameworks, observability mechanics, or feature decomposition. Guarantees must be enforceable and testable. If the intent fills more than one screen it is too broad.

### Stage 2 — Behavioral Contracts
**Purpose:** Translate the current Intent into independently testable observable truth while both
remain open to revision. The Contract also selects `events` or `external-observation` and names any
external observation artifact.
**Key constraint:** Contracts define observable behavior, not internal logic, code structure, or
unapproved event semantics.

### Stage 3 — Event Schema
**Purpose:** Complete the third Specification Package artifact and verify all three artifacts
together. In `events` mode it defines governed events; in `external-observation` mode it records
that no governed internal events apply and maps outcomes to the Contract's observation artifact.
This stage owns the `specification-approval` adapter.
**Key constraint:** Governed events must trace to the Contract. External-observation mode must not
invent placeholder events.

### Stage 4 — Implementation
**Purpose:** Satisfy governed specification using normal internal engineering choices. This stage
owns the `delivery-entry` adapter.
**Key constraint:** Apply the constraints selected by the active doctrine.

### Stage 5 — Tests
**Purpose:** Write behavioral truth anchors that fail if observable behavior deviates from the
Contract. Event mode includes event and replay checks; external-observation mode verifies its
declared artifact.
**Key constraint:** Tests assert observable outcomes, not private methods, internal state, or
intermediate computations.

### Stage 6 — Runtime Execution
**Purpose:** Run representative scenarios when permitted and capture evidence through the
Contract's observation mode.
**Key constraint:** Stage 6 never changes implementation or prior evidence and never fabricates an
unavailable observation.

### Stage 7 — Reconciliation Review
**Purpose:** Compare the applicable layers from Intent through runtime or external observation and
surface incomplete, disagreeing, or absent coverage.
**Key constraint:** Status is exactly `ALIGNED | GAP | MISMATCH | MISSING`; evidence source is
exactly `runtime | test | static | none`; the note explains the issue without subtype taxonomies or
scores.

### Stage 8 — Replay Verification
**Purpose:** Verify repeatable governed outcomes before final human acceptance. Event mode checks
schema, sequence, correlation, and deterministic payload content; external-observation mode reruns
its declared verification.
**Key constraint:** Generated identifiers, timestamps, and other nondeterministic envelope fields
are ignored unless contracted. A valid governed outcome may be a single-event chain.

### Stage 9 — Targeted Refinement
**Purpose:** Apply the smallest effective change justified by an observed problem or explicit human
evolution decision.
**Key constraint:** Use the actual cause, not a fixed refinement taxonomy or cost order. One safety,
authorization, or integrity failure is sufficient evidence. Return through reconciliation and final
verification.

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
6. Run `bash dba/04-tools/configuration/dba-config-boundaries.sh dba/00-entry/configurations/DBA-N.yaml`. Only after it
   passes, activate that configuration in `dba-system.md` and verify the pointer names the tested
   candidate.

## Reference Material

Current project definitions: `dba/05-guidance/terminology.md`

Prior design sessions and DBA theory: `maintenance/archive/`

- `maintenance/archive/initial-intent.md` — Early loop description and minimal stack
- `maintenance/archive/terminology.md` — DBA terminology with worked examples
- `maintenance/archive/intent-S0.md` through `intent-S7.md` — Stage-by-stage philosophy
- `maintenance/archive/S1-intent/intent-examples.md` — Complete loop walkthrough (counter example)
