# Codeos — DBA Toolkit

A symlinkable toolkit for developing software using **Declarative Behavioral Architecture (DBA)** / **Intent-Driven System (IDS)** methodology with Claude Code.

## What This Is

A human-governed AI development workflow where people define governed meaning, Claude produces
artifacts and implementation evidence, and the selected doctrine supplies the current semantic
guarantees. README does not duplicate those guarantees.

An approved **Solution Charter** governs why the solution exists, what success means, where its
boundary lies, and which obligations apply across it. Feature work then runs the 9-step loop:

```
Intent → Contracts → Event Schema → Implementation → Tests →
Runtime Execution → Reconciliation Review → Replay Verification → Targeted Refinement
```

After acceptance, a fact from real system use is an **Operational Observation**. It returns to the
earliest governed authority whose truth must change — the Charter, a Specification Package, a new
Intent, architecture, or targeted refinement — and never changes approved behavior by itself.

Current operational consequences are located by searching for `DOCTRINE ADAPTER` in the stage
prompts. The doctrine remains authoritative when explanatory text disagrees.

## Where to Focus Limited Review Time

Every substantial artifact opens with a Summary; read the full artifact only where the summary's
Oversimplification Risk note or the table below says the stakes warrant it.

| Artifact | Question it answers | Governance | Focus review on |
|---|---|---|---|
| Charter | Why are we building this? | Always | Purpose, scope, outcomes, Platform Baseline exceptions |
| Intent | What capability are we adding? | Always | Is this what you want? |
| Contract | What must be true? | Always | Is the behavior correct? The Feature Impact Accounting table |
| Event Schema | What runtime facts prove it? | Always | Are the right facts observable? |
| Architecture Scope | How is the solution structured? | Configurable (default: governed) | Major boundaries/tradeoffs, Platform Baseline resolution |
| Module Design Note | How does this module work? | Configurable (default: nongoverned) | Main risks and choices — descriptive only, never authoritative |
| User Workflow Map | What will the user do and see? | Configurable (default: nongoverned) | Does the journey make sense? |

## Quick Start

```bash
# In a new project directory:
bash /path/to/Codeos/dba/04-tools/initializer/dba-init.sh [project-name] [remote-url]

# Example:
bash /path/to/Codeos/dba/04-tools/initializer/dba-init.sh my-project https://github.com/user/my-project.git

# Then in Claude Code:
# Paste the contents of .codeos/toolkit/dba/03-prompts/workflow/support-session-orientation.md
```

## Directory Structure

**Two CLAUDE.md surfaces.** The toolkit repo and the projects that use it operate in
different modes, so the doctrine and the toolkit's own operating guide are separate files:
- `dba-system.md` — the stable downstream **DBA entrypoint**. It selects the active configuration,
  which selects the authoritative doctrine, policies, and reviewer tool contract. Downstream
  projects load it via `.codeos/toolkit/dba-system.md`.
- `CLAUDE.md` (toolkit repo root) — the **Codeos Self-Development** guide: minimal governance for
  ordinary work, with explicit human control for consequential semantic changes.
  Claude auto-reads it when working in this repo.

```
Codeos/
├── AGENTS.md          — Codex route to CLAUDE.md
├── CLAUDE.md          — Codeos Self-Development guide (governs toolkit changes)
├── dba-system.md      — Downstream DBA entrypoint (loaded via .codeos/toolkit/dba-system.md)
├── README.md          — This file
├── dba/               — Downstream DBA package
│   ├── 00-entry/      — Active configuration selection
│   ├── 01-doctrine/   — Fundamental DBA guarantees
│   ├── 02-policies/   — Conditional governance mechanisms
│   ├── 03-prompts/    — Workflow, review, and delegation instructions
│   ├── 04-tools/      — Contracts and implementations grouped by capability
│   ├── 05-guidance/   — Templates, patterns, and terminology
│   └── 06-reference/  — Non-authoritative migration guidance
└── maintenance/       — Codeos self-development
    ├── backlog/
    ├── reviews/
    ├── config/
    └── archive/
```

## Per-Project Structure (created by dba-init.sh)

```
myproject/
├── AGENTS.md                    — Codex route to root CLAUDE.md
├── CLAUDE.md                    — Discovery adapter
├── events/                      — Runtime evidence, created only when needed
├── source and tests             — Project-native layout
└── .codeos/                     — Durable project-local DBA state
    ├── 00-project/
    │   ├── CLAUDE.md            — Canonical project instructions
    │   ├── codeos.yaml          — Project config (DBA-5+): Platform Baseline, Codeos Mechanics,
    │   │                          per-artifact-type governance
    │   ├── charter.md           — Solution Charter, before the first package approval
    │   ├── learnings.md         — Optional Learning Register, created when needed
    │   └── terminology.md       — Optional shared project terminology, created when needed
    ├── 01-specification/
    │   ├── intents/
    │   ├── contracts/
    │   └── event-schemas/
    └── toolkit -> /path/to/Codeos  — Ignored machine-local mount
```

Architecture, framing, module design notes, refinement, review, runtime-evidence, and
operational-state paths are created only when the project actually needs them. The full canonical location model is owned by
the Downstream Project Layout Contract in `dba-system.md`.

## The DBA Development Loop

### Before Every Session
Use the Session Orientation support workflow at
`.codeos/toolkit/dba/03-prompts/workflow/support-session-orientation.md`.

### Starting a New Project for the First Time

After running `dba-init.sh` (with an optional project name and remote URL), follow these steps before opening Claude:

**Step 1 — Fill in `.codeos/00-project/CLAUDE.md` that the script generated.**
Open it and complete **Working agreements** — only durable instructions about how work is carried
out here that are not owned elsewhere; delete the section when empty. Project purpose and
constraints do not go here; they belong to the Solution Charter created in Step 4.

Do not create a project glossary by default. When the first specialized term needs one stable
meaning across features, create `.codeos/00-project/terminology.md` from
`.codeos/toolkit/dba/05-guidance/templates/project-terminology.md`.

**Step 2 — Open Claude Code in the project directory and use
`.codeos/toolkit/dba/03-prompts/workflow/support-session-orientation.md`.** Name the target feature or structural task.

**Step 3 — Claude reads the active DBA components and the project `CLAUDE.md` and confirms — verify it names both.**
1. `.codeos/toolkit/dba-system.md` — stable entrypoint and downstream layout owner
2. `.codeos/00-project/CLAUDE.md` — the canonical project instructions you just filled in

If the solution itself is still unclear, use the optional, non-authoritative Solution Framing
workflow at `.codeos/toolkit/dba/03-prompts/workflow/support-solution-framing.md` before Step 4.

**Step 4 — Establish the Solution Charter.**
Use `.codeos/toolkit/dba/03-prompts/workflow/support-solution-charter.md` to write and approve
`.codeos/00-project/charter.md`. It owns the `purpose-approval` boundary and must be approved before
the first Specification Package approval.

**Step 5 — Draft the Specification Package.**
Start with `.codeos/toolkit/dba/03-prompts/workflow/01-intent.md`, recording which Charter outcomes
the feature serves, then continue through Contract and Event Schema. Stage 3 owns the current
`specification-approval` boundary.

There are no existing project artifacts to read at this point — Claude starts from the active DBA
components selected through `.codeos/toolkit/dba-system.md` and the canonical project instructions. Do not paste a
resumption prompt; go straight to the stage prompt.

### Resuming After a Crash or Session Break

When starting a fresh session on a project where work is already in progress:

**Step 1 — Open Claude Code in the project directory and use
`.codeos/toolkit/dba/03-prompts/workflow/support-session-orientation.md`.** Name the target feature or task. The prompt
reads the matching live artifacts; do not copy repository state into the prompt.

**Step 2 — Claude reads these two files (verify it confirms both):**
1. `.codeos/toolkit/dba-system.md` — entrypoint to the active DBA configuration and layout contract
2. `.codeos/00-project/CLAUDE.md` — project intent and durable project constraints

**Step 3 — For each in-progress feature, direct Claude to read the existing approved artifacts.**
Tell Claude which feature to resume, then say: "Read the existing artifacts for [feature_id] before proceeding."
Claude will read:
- `.codeos/01-specification/intents/[feature_id].md` — approved Intent
- `.codeos/01-specification/contracts/[feature_id]_contract.md` — approved Contract
- `.codeos/01-specification/event-schemas/[feature_id]_schema.md` — approved Event Schema
- project-native implementation and tests, when present

**Do not ask Claude to rewrite or re-derive unchanged governed artifacts.** Route resumed work
through the Stage 4 `delivery-entry` adapter, which owns compatibility and entry checks.

**What Claude cannot recover automatically:** any decisions or clarifications that happened only in conversation and were never written into an artifact. If a decision was important, it should have been captured in an artifact at the time. If it wasn't, re-state it explicitly when you resume.

### For Each Feature
Support workflows route or prepare work without becoming lifecycle stages:

| Support workflow | Current compatibility path | Purpose |
|---|---|---|
| Solution Framing | `support-solution-framing.md` | Propose problem, vision, outcomes, scope, and constraints without approving them |
| Solution Charter | `support-solution-charter.md` | Approve the minimum solution-level authority |
| Feature Decomposition | `support-feature-decomposition.md` | Divide approved solution scope into candidate features when needed |
| Existing-Codebase Intake | `support-existing-codebase-intake.md` | Route observed existing behavior into normal DBA work |
| Architecture Synthesis | `support-architecture-synthesis.md` | Approve required project-level structure |
| Session Orientation | `support-session-orientation.md` | Select the applicable workflow from live state |
| Session Handoff | `support-session-handoff.md` | Summarize current state for resumption |

Work through governed Stages 1–9 in order. Decision behavior comes from the selected doctrine and
the named boundary adapters:

| Stage | File | Purpose | Claude produces | Boundary owner |
|---|---|---|---|---|
| 1 | `.codeos/toolkit/dba/03-prompts/workflow/01-intent.md` | Capture *why* the feature exists — actor + outcome, no implementation details | Intent in `01-specification/intents/` | — |
| 2 | `.codeos/toolkit/dba/03-prompts/workflow/02-contract.md` | Translate Intent into independently testable behavior | Contract in `01-specification/contracts/` | — |
| 3 | `.codeos/toolkit/dba/03-prompts/workflow/03-event-schema.md` | Complete and cross-check the Specification Package | Event Schema in `01-specification/event-schemas/` | `specification-approval` adapter |
| 4 | `.codeos/toolkit/dba/03-prompts/workflow/04-implement.md` | Implement governed behavior | Project-native code | `delivery-entry` adapter |
| 5 | `.codeos/toolkit/dba/03-prompts/workflow/05-tests.md` | Verify observable outcomes | Project-native tests | — |
| 6 | `.codeos/toolkit/dba/03-prompts/workflow/06-observe.md` | Capture trustworthy runtime evidence | Runtime evidence | — |
| 7 | `.codeos/toolkit/dba/03-prompts/workflow/07-reconcile.md` | Surface gaps and mismatches | Inline reconciliation | — |
| 8 | `.codeos/toolkit/dba/03-prompts/workflow/08-replay.md` | Verify replay and conformance | Inline final review package | `final-acceptance` adapter |
| 9 | `.codeos/toolkit/dba/03-prompts/workflow/09-refine.md` | Apply targeted refinement and return to verification | Optional durable refinement | Stage 8 adapter |

## Workflow Purposes

### Support Workflow — Session Orientation
**Purpose:** Orient Claude from live repository state, classify the target work, and select the
applicable workflow. Claude reads the active DBA entrypoint, project instructions, and only the
artifacts relevant to the target. Incomplete Specification Packages are normal work in progress.
**Key constraint:** Claude does not produce artifacts, write code, or analyze anything until the human explicitly says to proceed.

### Stage 1 — Intent
**Purpose:** Capture *why* a feature exists — not how it works. Every statement uses actor + outcome form and stays completely implementation-free. This document is the foundation every downstream artifact derives from; ambiguity here propagates forward into contracts, schema, and code.
**Key constraint:** No implementation details, APIs, databases, frameworks, observability mechanics, or feature decomposition. Guarantees must be enforceable and testable. If the intent fills more than one screen it is too broad.

### Stage 2 — Behavioral Contracts
**Purpose:** Translate the current Intent into independently testable observable truth while both
remain open to revision. The Contract also selects `events` or `external-observation`, names any
external observation artifact, and owns this feature's quality requirements.
**Key constraint:** Contracts define observable behavior, not internal logic, code structure, or
unapproved event semantics. Every quality requirement declares a verification method, and every
threshold states its workload, operating context, and rationale. Cross-cutting obligations belong to
the Charter's System Constraints instead.

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
**Key constraint:** Apply the constraints selected by the active doctrine. Stage 4 owns feature-local
design inside approved architectural boundaries, recording inline any such decision that is costly to
reverse. It may also maintain a Module Design Note for a module complex enough to warrant one — a
descriptive explanation of how that module works, never an authority for behavior or structure.

### Stage 5 — Tests
**Purpose:** Write behavioral truth anchors that fail if observable behavior deviates from the
Contract. Event mode includes event and replay checks; external-observation mode verifies its
declared artifact.
**Key constraint:** Tests assert observable outcomes, not private methods, internal state, or
intermediate computations. A test presented as acceptance evidence must observe the boundary named
by the acceptance claim.

### Stage 6 — Runtime Execution
**Purpose:** Run representative scenarios when permitted and capture evidence through the
Contract's observation mode.
**Key constraint:** Stage 6 never changes implementation or prior evidence and never fabricates an
unavailable observation. Everything it produces is development evidence — pre-acceptance proof of
the candidate implementation, distinct from a post-acceptance Operational Observation. Diagnostic
measurements remain unrestricted; a performance measurement used for a governed requirement must
first demonstrate that the measured operation exercises the governed behavior.

### Stage 7 — Reconciliation Review
**Purpose:** Compare the applicable layers from Intent through runtime or external observation and
surface incomplete, disagreeing, or absent coverage.
**Key constraint:** Status is exactly `ALIGNED | GAP | MISMATCH | MISSING`; evidence source is
exactly `runtime | test | static | none`; the note explains the issue without subtype taxonomies or
scores. A reconciliation claim cannot be stronger than its cited observation.

### Stage 8 — Replay Verification
**Purpose:** Verify repeatable governed outcomes before final human acceptance. Event mode checks
schema, sequence, correlation, and deterministic payload content; external-observation mode reruns
its declared verification.
**Key constraint:** Generated identifiers, timestamps, and other nondeterministic envelope fields
are ignored unless contracted. A valid governed outcome may be a single-event chain.
Mechanically unready packets are refused before a reviewer round; evidence adequacy remains a
human, agent, and reviewer judgment.

### Stage 9 — Targeted Refinement
**Purpose:** Repair an implementation that does not satisfy already-approved behavior, using the
smallest effective change.
**Key constraint:** Stage 9 never redefines intended behavior, requirements, or architecture — those
return to their owning authority under the re-entry rule. Use the actual cause, not a fixed
refinement taxonomy or cost order. One safety, authorization, or integrity failure is sufficient
evidence. Return through reconciliation and final verification.

## Governing Rules

Read `dba-system.md` and its selected doctrine. This README describes navigation and stage purpose;
it does not independently define authority, approval cadence, escalation, or evidence semantics.

The current lifecycle has `purpose-approval`, `specification-approval`, `delivery-entry`,
`final-acceptance`, and the conditional `architecture-entry` doctrine adapters. This count is
descriptive, not an invariant: add or remove an adapter only when a genuine execution boundary
changes.

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

Migration from supported legacy downstream layouts:
`dba/06-reference/downstream-upgrade.md`

Prior design sessions and DBA theory: `maintenance/archive/`

- `maintenance/archive/initial-intent.md` — Early loop description and minimal stack
- `maintenance/archive/terminology.md` — DBA terminology with worked examples
- `maintenance/archive/intent-S0.md` through `intent-S7.md` — Stage-by-stage philosophy
- `maintenance/archive/S1-intent/intent-examples.md` — Complete loop walkthrough (counter example)
