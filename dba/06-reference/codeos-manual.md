# Codeos: Declarative Behavioral Architecture for Provable AI-Assisted Software Evolution

*An explanatory manual for human-governed, artifact-constrained, event-verifiable development with Claude Code.*

---

## Abstract

Codeos is a development toolkit and methodology for tightly controlled, AI-assisted
software evolution. It implements **Declarative Behavioral Architecture (DBA)** — also
called **Intent-Driven System (IDS)** mode in its own instructions. Its organizing idea
is simple to state and consequential in practice: software is not built starting from
code, but from governed behavioral artifacts. The selected doctrine defines semantic guarantees;
stage prompts marked `DOCTRINE ADAPTER` encode only their operational consequences.

This manual presents Codeos as a *method*, not as a prompt collection. It defines the
method, explains the dangers it answers, walks its full development loop with a single
worked example, states its doctrine kernel, and explains its verification machinery
— the event spine, reconciliation, and replay.

**One disclaimer governs the entire manual.** Codeos is **not formal verification** in
the formal-methods or theorem-proving sense. It does not prove software correct against a
mathematical specification. What it builds is an *operational evidence chain*: a staged,
inspectable, replayable, reconcilable record that runs from intent through contracts and
events to observed runtime behavior, so that behavioral claims can be checked rather than
assumed. Throughout this manual, words like "proof," "provable," and "verification" mean
*operational behavioral verification* in this sense — never formal proof of correctness.
Where this manual uses interpretive labels for Codeos (for example "proof-before-code" or
"anti-improvisation system"), those are this manual's framing, not terms the repository
itself defines.

This manual is not a normative doctrine surface. If it conflicts with the selected doctrine or a
boundary adapter, those sources prevail and this manual must be corrected.

---

## Preface

Modern AI tools can write code, fix bugs, refactor files, and propose architectures. That
very capability creates a new risk: an AI can produce a system quickly that *appears*
correct but has no clear relationship to the actual intent, the business rule it was meant
to serve, or any runtime evidence that it still behaves as intended.

Codeos answers this by changing the question. The default question of AI-assisted
development is: *"How should the AI implement this feature?"* Codeos asks instead:

> What intent must be preserved? What observable truth must be demonstrable? Which events
> are permitted to exist? And what evidence will show that the system is still aligned with
> that intent?

This shift moves development out of an improvised conversation with an AI and into a
structured verification loop. The human is not reduced to a terminal operator for an AI
agent; the human is the author of intent and the holder of judgment. The AI is not an
autonomous product decision-maker; it is a constrained proposer, implementer, and auditor
of artifacts.

### How to read this manual

This is a reference manual, not a linear tutorial. Read by purpose:

- **Shortest path to understanding the method** — read Parts I–III and the worked example
  in Part VI.
- **Adopting Codeos on a project** — read Parts II–V; the adoption levels are at the end
  of Part V.
- **Comparing methods or deciding whether to adopt** — read "Codeos vs OAP" and
  "Limitations & risks" in Part VII.
- **Acting as a reviewer of Codeos artifacts** — read Part II (roles, rules) and the
  reviewer subsection in Part V.

### What this manual enables

After reading this manual, a reader should be able to:

- explain Codeos as a method, not as a set of prompts;
- run the DBA loop on a new feature and know which gate they are at;
- identify which artifacts are required versus optional;
- review whether Claude is obeying the workflow, or quietly drifting from it;
- distinguish operational behavioral verification from formal proof;
- decide when Codeos is worth its cost and when it is not.

### Source basis

Every normative claim in this manual is grounded in a Codeos repository artifact:
the active DBA components selected through `dba-system.md`, `README.md`, `dba/05-guidance/terminology.md`, the
stage prompts in `dba/03-prompts/workflow/`, the artifact templates in `dba/05-guidance/templates/`, the architectural patterns
in `dba/05-guidance/patterns/`, and `dba/04-tools/initializer/dba-init.sh`. Major claims are mapped to their source files in
**Appendix A — Source Map**. This manual distinguishes two kinds of statement: *repo
facts* (what Codeos requires, traceable to a file) and *interpretation* (this manual's
reading of why those requirements matter). Interpretation is always marked as such.

A normative convention is used throughout:

- **MUST / NEVER / required** — a rule backed directly by an active governed component or a stage prompt.
- **should / may / this manual recommends** — guidance or interpretation that the
  repository does not itself mandate.

---

# Part I — Definition and Motivation

*Source basis: `README.md` (what this is, the 9-step loop); the `doctrine` component selected
through `dba-system.md` (authority, lifecycle, constraints, human control); `dba/05-guidance/terminology.md`
(canonical project glossary); `dba/03-prompts/workflow/00c-onboarding.md`
(intent laundering); `dba/05-guidance/patterns/shared-infrastructure-boundary.md` (vertical drift).*

## Why Codeos exists

Classic AI-assisted development has a recurring weakness: the leap from idea to code. A
human describes a wish, the AI writes a solution, and the two then chase bugs together.
This is fast but brittle. It is frequently unclear which requirement was actually
implemented, which edge cases are covered, whether the tests verify behavior or merely
internal structure, and whether the running system emits any evidence that can be analyzed
later.

Codeos exists to forbid that leap. Its mechanism is not a more powerful model — it is a
better *structure of truth*. The leap is replaced by an ordered loop in which each step produces
artifacts or evidence for the next. Decision semantics come from the selected doctrine.

## Failure / threat model — the dangers Codeos answers

*(This section is interpretation: it organizes the specific failure modes that the
repository's rules and stages are built to contain. Each danger is paired with the
mechanism that contains it.)*

Codeos is best understood as a response to concrete, recurring failures of AI-assisted
development:

| Danger | What it looks like | Contained by |
|---|---|---|
| **Intent-to-code leap** | AI jumps from a wish straight to an implementation | Stage 3 specification adapter and Stage 4 delivery-entry adapter |
| **Hidden abstractions** | AI adds helper layers, "service" abstractions, speculative generality | Stage 4 constraint: no abstractions the contract does not require |
| **Unapproved events** | Runtime emits behavior nobody specified | Stage 3 event spine: implementation may emit *only* schema events; extras are structurally visible |
| **Implementation-asserting tests** | Tests check private methods and internal state, so they pass while behavior is wrong | Stage 5 rule: tests assert observable outcomes only, in event-schema language |
| **Runtime/schema divergence** | The log contains fields or types the schema never declared | Stage 7 Schema Payload Drift check (MATCH/TYPE_MISMATCH/ABSENT/EXTRA) |
| **Rubber-stamping** | Human accepts artifacts mechanically; decisions become theater | Independent advisory review at applicable boundary adapters |
| **Vertical drift** | Domain logic seeps into a shared infrastructure module, coupling features through the hub | `shared-infrastructure-boundary` pattern; Stage 10 Impact Analysis gate |
| **Fake source of truth** | A generated plain-language summary is edited and silently becomes authoritative | Human Navigation rule: stored summaries carry provenance, are regenerated, are never DBA artifacts |

Each later Part returns to these dangers and shows the stage or rule that addresses it.

## Core definition

Codeos is a **human-controlled, artifact-constrained methodology for AI-assisted software
evolution**, in which every feature is developed through Intent, Contract, and Event Schema,
*from* governed specification *into* a minimal implementation, *from* implementation into behavioral and
replay tests, and then *through* runtime observation, reconciliation audit, and a targeted
refinement loop.

Stated more briefly *(interpretation)*: Codeos is a system that prevents an AI from "doing
something clever" until it is clear what must be true, how that truth will be observable,
and what runtime evidence will show whether the truth still holds.

## What Codeos is

Codeos is, simultaneously:

- a **methodology** — it prescribes an ordered sequence of stages and governed boundaries;
- a **toolkit** — it ships governed DBA components selected through `dba-system.md`, stage prompts (`dba/03-prompts/workflow/`),
  artifact templates (`dba/05-guidance/templates/`), architectural patterns (`dba/05-guidance/patterns/`), and an
  initializer (`dba/04-tools/initializer/dba-init.sh`), symlinked into a project as `.codeos`;
- a **workflow** — the 9-step DBA loop, plus an alternate 5-step architectural-refinement
  loop;
- an **architectural discipline** — it treats event schemas, contracts, and runtime events
  as part of the architecture, not as adjunct documentation;
- an **audit system** — it enables structural comparison of what was intended, specified,
  implemented, tested, and actually observed at runtime.

This manual additionally characterizes Codeos as an **anti-improvisation system**
*(interpretation)*: the AI may not add abstractions, events, helper layers, error handling,
or behavior unless it derives from an approved artifact.

## What Codeos is NOT

- **Not vibe coding.** It does not rest on a feeling that "the solution looks right." Stages
  produce inspectable artifacts and evidence.
- **Not ordinary AI pair programming.** Governed boundaries replace conversational improvisation.
- **Not an open-ended autonomous agent system.** Delivery follows the selected doctrine and its
  operational adapters rather than agent-defined authority.
- **Not merely a documentation standard.** The documents are not commentary beside the
  code; they are the superior behavioral truth from which code must be derived.
- **Not classic test-first development.** Tests matter, but they come *after* intent,
  contracts, and the event schema — because they must verify approved observable behavior,
  not incidental implementation details.
- **Not formal verification.** Codeos produces operational behavioral verification through
  staged artifacts, runtime evidence, reconciliation, and replay. It does not prove
  correctness in the formal-methods sense. It creates an evidence chain that makes
  behavioral claims inspectable, replayable, and reconcilable against approved artifacts —
  which is a different and more modest guarantee than mathematical proof.

---

# Part II — Operating Doctrine

*Source basis: the `doctrine` component selected through `dba-system.md` (authority, lifecycle,
behavioral constraints, human control, escalation);
stage prompts `01`–`09` (per-stage roles); `dba/03-prompts/review/pipeline-reviewer.md` (reviewer role).*

## Proof before code

In ordinary AI development the chain is usually:

```
wish → prompt → code → bug → patch → a feeling that it works
```

In Codeos the chain is:

```
intent → contract → event schema → implementation → tests →
runtime events → reconciliation → replay → targeted refinement
```

The difference is decisive. Codeos does not treat code as the first source of truth. Code
is one layer in a chain. If code, tests, the event log, and the contract disagree, the
system is not declared finished — reconciliation follows: a structural comparison of all
layers.

The word "proof" here means a staged *evidence chain*, not a formal-methods proof. The
chain's strength is that each link is independently inspectable and that a break between
any two links is surfaced rather than hidden.

## Role of the human

The human is the primary holder of intent and the decisions assigned by the selected doctrine.
This does not require the human to hand-write every contract, schema, or test.

When a stage is reviewed, this manual recommends the following checklist, distilled from the stage
prompts:

- **Intent** — does it describe *why*, not *how*? Is the actor a human or business role,
  not "the system"?
- **Contracts** — do they describe observable truth, verifiable without reading code?
- **Event schema** — does it cover every scenario, with one failure event per named
  failure, and introduce no observable beyond the contract?
- **Implementation** — does it add unapproved logic, abstractions, or events?
- **Tests** — do they verify behavior, not internals, in exact event-schema language?
- **Runtime events** — do they demonstrate the execution actually happened?
- **Reconciliation** — does it surface gaps, mismatches, or missing coverage?
- **Refinement** — does it solve an *observed* problem rather than introduce a redesign?

In Codeos the human does not "trust the AI." The human steers a sequence of evidence.

## Role of Claude

Claude is not a free architect. In each stage it has a precisely scoped role, defined by
the stage prompts:

- **Stage 1** — intent analyst: help turn a raw description into actor + outcome intent.
  Implements nothing.
- **Stage 2** — behavioral contract specialist: align observable contracts with the current Intent.
- **Stage 3** — define the event spine.
- **Stage 4** — implement governed behavior using normal internal engineering choices.
- **Stage 5** — write behavioral and replay tests.
- **Stage 7** — behavioral auditor performing reconciliation (explicitly *not* a code
  reviewer).
- **Stage 8** — verify replayability.
- **Stage 9** — propose the smallest evidence-backed fix.

Claude stays inside the governed artifact chain. When implementation cannot satisfy that chain,
the Stage 4 adapter applies the active doctrine rather than silently inventing behavior.

## The doctrine kernel

The selected doctrine is intentionally small and is the sole semantic authority. Read it through
`dba-system.md`; this manual does not maintain a parallel summary of its guarantees.

## Truth Authority and conflict resolution

Authority and conflict resolution are defined only by the selected doctrine. Operational prompts
must apply that text rather than this manual's interpretation.

---

# Part III — The DBA Loop

*Source basis: `README.md` (Stage Purposes, key constraints); the `doctrine` component selected
through `dba-system.md` (the 9-step loop);
stage prompts `00-session-start`, `01`–`09`; `dba/05-guidance/templates/` (artifact shapes).*

Codeos has a nine-stage *feature* loop. It also has a Stage 0 *session discipline* that is
not a feature stage. Current doctrine consequences appear only at the boundary adapters identified
in the stage descriptions below.

## Stage 0 — Session Start / Operating Preflight

Stage 0 is not part of the feature lifecycle; it is the operating preflight that places
Claude into DBA mode, scopes the session, names forbidden actions, and prevents accidental
autonomous work. The session-start prompt (`dba/03-prompts/workflow/00-session-start.md`) directs Claude to:

- read `.codeos/dba-system.md`, confirm the selected configuration and doctrine, and avoid deriving
  doctrine semantics from explanatory documents;
- read the project `CLAUDE.md` and note the Active Features table;
- read `docs/codebase-digest.md` if it exists (structural orientation), or state that none
  was found;
- determine and confirm the **session type**:
  - **A — Feature Brief** (new feature discovery → `00b-feature-brief.md`);
  - **B — Feature Stage Work** (advancing a feature through Stages 1–9);
  - **C — Architectural Refinement** (structural, non-behavioral change →
    `10-arch-refine.md`);
  - **D — Existing Codebase Onboarding** (working code with no DBA artifacts →
    `00c-onboarding.md`);
- read `features/registry.yaml` if present and report feature status;
- absorb the session context (today's goal, scope, session-specific forbidden actions);
- then **STOP** and wait for the human to begin.

The key constraint: in Stage 0, Claude does not produce artifacts, write code, or analyze
anything until the human explicitly says to proceed.

## Stages 1–5: Intent → Contracts → Event Schema → Implementation → Tests

Each stage below uses the same mini-template:

- **Purpose** — what the stage establishes.
- **Primary artifact** — what is produced.
- **Approval** — whether this output participates in a human decision.
- **Prevented failure mode** — the danger (from Part I) it contains.
- **Proof produced** — the evidence this stage adds to the chain.
- **Claude constraints** — what Claude may not do here.
- **Verification questions** — what a reviewer asks.

### Stage 1 — Intent

- **Purpose:** Capture *why* a feature exists, in actor + outcome form, implementation-free.
  This is the foundation every downstream artifact derives from.
- **Primary artifact:** `intents/[feature_id].md` (from `dba/05-guidance/templates/intent.md`): a purpose
  statement, a few actor-outcome statements, stable guarantees, and an explicit scope
  boundary.
- **Boundary owner:** —; the draft passes to Stage 2.
- **Prevented failure mode:** the intent-to-code leap, and intent ambiguity that would
  propagate forward.
- **Proof produced:** an approved, implementation-independent statement of intent — the
  anchor of the trace chain.
- **Claude constraints:** no APIs, databases, frameworks, file formats, observability
  mechanics, or feature decomposition; guarantees must be enforceable and testable; if it
  fills more than one screen it is too broad. The prompt watches for subtle leaks (timing
  language, mechanism language, overly broad backward-compatibility obligations).
- **Verification questions:** Is the actor a specific human role, or has the feature been
  misattributed to a system? Do the stable guarantees name the one thing the feature must
  never fail to do? Is the scope boundary explicit enough that a later stage cannot silently
  expand it?

### Stage 2 — Behavioral Contracts

- **Purpose:** Translate the current Intent into independently testable, observable-only
  truth.
- **Primary artifact:** `contracts/[feature_id]_contract.md` (from `dba/05-guidance/templates/contract.md`):
  Given/When/Then scenarios, invariants, an Invariant Falsification Scenarios table,
  pre/postconditions, a Failure Classifications table, and optional Runtime Context and
  Vocabulary Dependency sections.
- **Boundary owner:** —; the drafts pass to Stage 3.
- **Prevented failure mode:** implementation-asserting specification — clauses that can only
  be checked by reading code.
- **Proof produced:** a set of black-box-verifiable behavioral truths, including
  *falsification scenarios* that each target a specific plausible wrong implementation.
- **Claude constraints:** every clause must be answerable from emitted events and system
  state alone; no classes, functions, databases, APIs, or frameworks; all actors must come
  from the approved intent; event and module names belong in Notes, not inside scenarios.
  The prompt requires at least one boundary scenario and one falsification scenario (with a
  `Falsifies:` annotation), and at least one falsification row per invariant.
- **Verification questions:** Does the contract cover failure modes that matter under real
  operation, not just the happy path? Are the Then clauses verifiable from output alone? Is
  there a boundary or falsification scenario that would catch the most plausible wrong
  implementation?

### Stage 3 — Event Schema

- **Purpose:** Define the complete event spine that structurally constrains all future
  implementation. This is the most constraining artifact in the loop.
- **Primary artifact:** `events/[feature_id]_schema.md` (from `dba/05-guidance/templates/event-schema.md`):
  named events with categories and payloads, an event flow diagram, and a Coverage Check
  table mapping every contract failure to a FAILURE event.
- **Boundary owner:** Stage 3 `specification-approval` doctrine adapter.
- **Prevented failure mode:** hidden behavior — once approved, implementation may emit only
  the listed events, so undeclared behavior becomes architecturally impossible.
- **Proof produced:** the closed set of permitted runtime observations, against which the
  log will later be reconciled and replayed.
- **Claude constraints:** the schema must not be *stronger* than the contract — every new
  observable (payload field, event, ordering guarantee) must trace to an approved contract
  clause; `correlation_id` is mandatory on every event without exception; validation
  ordering is not prescribed unless the Contract requires it. For external observation, the
  Event Schema records the approved observation source and that no governed internal events apply.
- **Verification questions:** Does every event trace to an approved contract clause? Are
  failure events specific enough to support root-cause analysis? Does the flow diagram
  accurately represent what Stage 4 will be constrained to implement?

The four event categories are: **OBSERVATIONAL** (raw runtime facts), **BEHAVIORAL**
(verified outcomes), **FAILURE** (classified error conditions), and **EXTERNAL** (side
effects on outside systems). Every event carries six required base fields: `event_id`,
`event_type`, `timestamp`, `correlation_id`, `source_module`, and `payload`.

### Stage 4 — Implementation

- **Purpose:** Satisfy every approved contract clause and emit every approved schema event —
  nothing more.
- **Primary artifact:** code in `modules/[feature_id]/`, plus a Contract Satisfaction Table
  and an Event Emission Table.
- **Boundary owner:** Stage 4 `delivery-entry` doctrine adapter; testing adds no new boundary.
- **Prevented failure mode:** hidden abstractions, unapproved events, speculative error
  handling, undeclared runtime artifacts.
- **Proof produced:** code in which the first thing wired up is correlation-ID propagation
  and event emission, so all behavior is traceable.
- **Claude constraints:** internal helpers, types, validation, technical errors, and established
  patterns are allowed. They must not change approved behavior, authority, event semantics,
  architecture, safety, authorization, or integrity. All three specification artifacts must be
  approved before implementation.
- **Verification questions:** Are any clauses satisfied in a surprising or fragile way? Does
  the implementation introduce behavior not traceable to intent, contract, or schema? What
  is the most likely Stage 7 gap?

### Stage 5 — Tests

- **Purpose:** Write behavioral truth anchors that fail if observable behavior deviates from
  the contracts.
- **Primary artifact:** `tests/behavioral/[feature_id]_behavior.test.[ext]` and
  `tests/replay/[feature_id]_replay.test.[ext]`, plus a Contract Coverage Table.
- **Boundary owner:** —; test evidence passes to runtime verification.
- **Prevented failure mode:** tests that assert internals and pass while behavior is wrong.
- **Proof produced:** automated behavioral evidence (happy path, every named failure,
  telemetry correctness, idempotency if contracted, one test per invariant-falsification
  row) and a replay test that asserts schema conformance, correlation-chain integrity, and a
  deterministic event sequence.
- **Claude constraints:** tests must not touch private methods, internal state, or
  intermediate computations; all assertions use event names exactly as they appear in the
  approved schema. The determinism assertion runs the feature twice and asserts the same
  `event_type` sequence.
- **Verification questions:** Do the tests verify observable behavior or internal details?
  Would each invariant-falsification test actually fail if the named wrong assumption were
  present? Is any clause technically covered but not verifying the right outcome?

## Stages 6–9: Runtime Execution → Reconciliation → Replay → Targeted Refinement

### Stage 6 — Runtime Execution

- **Purpose:** Run the implementation and capture `events/runtime_events.jsonl` as runtime evidence.
- **Primary artifact:** the populated append-only runtime event log.
- **Boundary owner:** —; record a GAP when evidence cannot be obtained.
- **Prevented failure mode:** declaring a feature done with no runtime evidence that it
  executed.
- **Proof produced:** Evidence Level 1 (Direct) observations — real events in the log,
  including failure paths reproduced via runtime fixtures where practical.
- **Claude constraints:** Claude's role is advisory (help set up event capture);
  `runtime_events.jsonl` is append-only — never modified or deleted. Where direct
  observation of a failure path is impractical (cloud outages, third-party failures,
  production-only infrastructure), the reason is documented and the row is later classified
  `GAP (runtime evidence)` rather than forced.
- **Verification questions:** Do event types match the schema? Are correlation chains
  intact? Are there any unexpected events?

### Stage 7 — Reconciliation Review

- **Purpose:** A structural audit comparing six layers — intent, contract, event schema,
  implementation, tests, runtime events — to surface gaps, mismatches, and missing coverage.
  This is the heart of the method (see Part IV).
- **Primary artifact:** the reconciliation table and Findings Summary, with an optional
  Structural Alignment section.
- **Boundary owner:** —; route findings using the selected doctrine's escalation rules.
- **Prevented failure mode:** silent drift — divergence between what was intended,
  specified, built, tested, and observed.
- **Proof produced:** a per-item status across all layers (see the status vocabulary in Part
  IV), plus a Schema Payload Drift table and an Evidence Quality grading.
- **Claude constraints:** this is not a code review and never suggests rewrites; status is
  exactly one of ALIGNED / GAP(...) / MISMATCH / MISSING; structural observations are
  advisory and do not change verdicts.
- **Verification questions:** Do any ALIGNED findings have low evidence quality that masks a
  real gap? Are GAP sub-types correctly classified? Are non-ALIGNED items prioritized to
  help the human decide what to address first?

### Stage 8 — Replay Verification

- **Purpose:** Confirm the system is deterministically replayable.
- **Primary artifact:** the replay report (event-log summary, sequence conformance, replay
  test results).
- **Boundary owner:** Stage 8 `final-acceptance` doctrine adapter.
- **Prevented failure mode:** non-determinism and broken or orphaned correlation chains
  hiding behind green unit tests.
- **Proof produced:** the guarantee that *same inputs + same module version + same
  constraints → same resulting events*; every event conforms to schema; every chain starts
  with a trigger/observational event and ends with a BEHAVIORAL or FAILURE event.
- **Claude constraints:** out-of-order events, events without prerequisites, and event types
  absent from the schema are conformance failures that must be resolved.
- **Verification questions:** Do schema-conformance issues indicate implementation drift
  rather than test gaps? Do broken chains point to a specific module or event category?

### Stage 9 — Targeted Refinement

- **Purpose:** Apply the smallest effective, problem-driven fix to each observed issue.
- **Primary artifact:** refinement records (from `dba/05-guidance/templates/refinement.md`) plus minimal
  diffs and a list of stages to re-run.
- **Boundary owner:** No additional boundary; verified results return to Stage 8.
- **Prevented failure mode:** the temptation to answer a bug with a redesign.
- **Proof produced:** an evidence-linked change with an explicit trigger and a bounded
  re-run set.
- **Claude constraints:** valid triggers only — RECURRING_FAILURE, RECONCILIATION_GAP,
  REPLAY_FAILURE, OBSERVABILITY_GAP, or HUMAN_APPROVED_EVOLUTION. Forbidden triggers:
  elegance, theoretical improvement, "better architecture," single non-recurring incidents.
  Refinements are ordered by cost — observability first, structural last.
- **Verification questions:** Is each change the smallest effective fix? Should any be moved
  to Stage 10 (architectural refinement) instead? Are there observed problems not addressed?

---

# Part IV — The Proof System

*Source basis: the `doctrine` component selected through `dba-system.md` (vocabulary: Event Spine,
Correlation ID, event kinds);
`dba/03-prompts/workflow/03-event-schema.md`, `06-observe.md`, `07-reconcile.md`, `08-replay.md`,
`09-refine.md`; `maintenance/archive/terminology.md` (historical event JSON example).*

The strongest claim Codeos makes is operational, not formal. Its three load-bearing
mechanisms are the event spine, reconciliation, and replay.

## Event Spine as operational evidence

Every meaningful runtime action must emit an approved event, written as one JSON object per
line to the append-only `events/runtime_events.jsonl`. The system therefore produces not
only a result but a trail from which what happened can be reconstructed.

Every event carries the six required base fields:

```json
{
  "event_id": "uuid-v4",
  "event_type": "ListingIngested",
  "timestamp": 1710000000000,
  "correlation_id": "uuid-v4",
  "source_module": "feed_ingestion",
  "payload": {}
}
```

`correlation_id` is mandatory on every event without exception; it links all events from a
single feature execution chain. Events are typed by category — OBSERVATIONAL (a raw fact
like `RequestReceived`), BEHAVIORAL (a verified outcome like `CartItemAdded`), FAILURE (a
classified error like `CartItemAddFailed`), and EXTERNAL (a side effect like `EmailSent`).

The event spine does several things at once:

- it **constrains implementation** — code may emit only approved events;
- it enables **reconciliation** between schema and reality;
- it enables **replay verification**;
- it **exposes hidden behavior** — an event not in the schema is itself evidence of drift;
- it **correlates** events across a single execution via `correlation_id`;
- it turns the runtime from a black box into an evidentiary record.

This is the gap between "the tests are green" and "the system is demonstrably traceable."

## Reconciliation — the heart of the method

Stage 7 is not a code review. It is a structural audit across six layers: intent → contract
→ schema → implementation → tests → runtime events. The reconciliation table uses a fixed
column structure and one status per row.

**Status vocabulary (exact):**

- **ALIGNED** — all layers agree.
- **GAP (implementation)** — specified but not implemented or not tested.
- **GAP (runtime evidence)** — implemented and tested, but the path was never observed at
  runtime.
- **GAP (observability)** — behavior may be occurring but cannot be proven from events
  alone.
- **GAP (documentation)** — artifact text does not match implemented reality; no code change
  needed.
- **GAP (evidence quality)** — a test passes but the evidence level is below the contract's
  declared minimum; real-boundary or production observation is still required.
- **MISMATCH** — two layers disagree (contract says X, runtime shows Y).
- **MISSING** — a required artifact or event is absent.

Reconciliation also runs a **Schema Payload Drift** check, comparing observed event payloads
against the schema field by field, with statuses MATCH / TYPE_MISMATCH / ABSENT / EXTRA. An
ABSENT required base field defaults to MISSING; an EXTRA field is always a documentation gap
(undeclared payload evolution).

Two independent axes of evidence are tracked. The **runtime evidence level** (1 Direct, 2
Indirect, 3 Test, 4 Static, 5 None) records *where* the evidence came from; level 5 makes a
row MISSING. Separately, the **Evidence Quality scale** (1 Specification, 2 Static, 3
Simulated, 4 Real boundary, 5 Production) records *environment fidelity*. The key principle:
alignment and evidence quality are independent — a test can pass (ALIGNED) yet sit at
Evidence Quality 3 when the contract requires level 4, producing a `GAP (evidence quality)`.

The crucial reframing *(repo-backed in spirit, stated here as interpretation)*: when Stage 7
finds a GAP, MISMATCH, or MISSING, that is not a failure of the process — it is the method
working. The system detected drift before it hid inside the code as "working" functionality.

## Replay verification

Replay verification is the second half of the evidence regime. The runtime log is useful not
only as a journal but as repeatable evidence. Given the same inputs and the same approved
schema, the event history must conform to the schema, correlation chains must be complete
(start with a trigger/observational event, end with a BEHAVIORAL or FAILURE event), and the
event sequence must match the contract's expected flow. Replay therefore checks not only
*that* something happened but that it happened in a way that can be understood, repeated, and
compared.

Codeos's own scope note matters here: replay tests invoke the real feature and assert
conformance and determinism; they do **not** re-inject the JSONL back through a replay engine
(that is out of current scope). The guarantee is deterministic *event-sequence* conformance,
not full historical re-execution.

## Targeted refinement

Codeos explicitly forbids the largest temptation of AI development: answering a discovered
fault with a larger redesign. Stage 9 demands the smallest effective fix for a concrete
observed problem, drawn only from the valid triggers, ordered cheapest-first. Every
refinement carries an evidentiary reason and re-opens only the affected stages. This keeps
the system stable: change is bounded and justified rather than sweeping and aesthetic.

---

# Part V — Supporting Machinery and Anti-Drift Architecture

*Source basis: active DBA policies, prompts, and templates (artifact use,
Review Logging, Human Navigation, Architectural Refinement); `dba/05-guidance/templates/` (review-package,
review-file, handoff);
`dba/03-prompts/workflow/10-arch-refine.md`, `00c-onboarding.md`, `pipeline-reviewer.md`;
`dba/05-guidance/patterns/shared-infrastructure-boundary.md`; `dba/04-tools/initializer/dba-init.sh`.*

## Required vs optional artifacts

Artifact eligibility is enforced by the Stage 4 `delivery-entry` adapter. This manual records only
the supporting classifications:

- **Recommended:** Feature Registry (`features/registry.yaml`) for multi-feature projects.
- **Optional:** Feature Brief (`backlog/[id].md`), Codebase Digest
  (`docs/codebase-digest.md`), the Stage 7 Structural Alignment section, Architectural
  Refinement records.
- **Onboarding-only:** `HYPOTHESIZED_INTENT` drafts from Session Type D, routed through the Stage 3
  `specification-approval` adapter.

## Templates, prompts, and project setup

The toolkit ships fill-in templates for every artifact (intent, contract, event schema,
feature spec, refinement, arch-refinement, codebase digest, conventions, feature brief,
feature registry, handoff, project `AGENTS.md` and `CLAUDE.md`, review file, review package) and a
sequential prompt for each step.

`dba/04-tools/initializer/dba-init.sh`, run from a new project root, scaffolds the project: it creates the
`.codeos` symlink to the toolkit; creates `intents/`, `contracts/`, `events/`, `modules/`,
`tests/behavioral/`, `tests/replay/`, `docs/`, `features/`, `backlog/`, and
`refinements/arch/`; seeds `features/registry.yaml`, an empty `events/runtime_events.jsonl`,
a project `AGENTS.md` and `CLAUDE.md`, `docs/conventions.md`, and a codebase-digest placeholder; initializes
git on branch `main`; and optionally adds a remote. The human then fills in the project
intent and conventions and pastes `dba/03-prompts/workflow/00-session-start.md` to begin.

## The reviewer interface

Codeos separates the *implementer* role (Claude in the loop) from an *independent reviewer*.
The Reviewer Activation Package (`dba/03-prompts/review/pipeline-reviewer.md`) primes a separate reviewer —
human or a second LLM — as an **independent critical assessor, not a DBA compliance
auditor**. The reviewer may challenge anything: assumptions, architecture, scope, framing,
alternatives, or DBA itself. Its output uses an **Attention Level** (High / Medium / Low) as
a scannability signal — explicitly **not** APPROVED / REVISE / BLOCKING verdicts, which would
turn the reviewer into a gatekeeper. The human decides what to act on.

At review points selected by the review policy and applicable adapters, Claude presents a **Review
Package** inline (from `dba/05-guidance/templates/review-package.md`): the relevant artifacts or evidence, the
purpose, suggested examination areas, and known tensions. This convenience view is not
authoritative and is not written to disk; the durable review record lives in
`reviews/[feature_id].md`.

## Review logging: decision log and architecture journal

When the human provides a reviewer's assessment and their decision, Claude records it before
doing other work:

- **One row** to the `reviews/[feature_id].md` **Decision Log** (append-only — original
  findings and decisions are never rewritten; superseded decisions get a new row).
- **A Decision Rationale section** — only when the decision would be hard to reconstruct from
  artifact history alone (a reframing, an architectural pivot, a rejected direction).
- **One Architecture Journal entry** (`reviews/architecture-journal.md`, format `AJ-NNN`) —
  only if the insight is likely to remain useful six months later to a reader who has
  forgotten the feature entirely.

The journal is the long-term institutional-memory artifact; per-feature review files are
primarily traceability artifacts. Log fidelity rule: preserve the reviewer's core insight
close to verbatim; compress context, never the insight. Human overrides exist ("do not log
this review," "journal this," "do not journal this").

## Architectural Refinement (Stage 10) and anti-drift

Some changes alter no contract and emit no event — workspace restructuring, shared-library
extraction, dependency consolidation, test infrastructure, naming normalization. For these,
Codeos provides an alternate **5-step architectural-refinement loop** (`dba/03-prompts/workflow/10-arch-refine.md`):
**Scope Intent → Impact Analysis → Implement → Verify → Reconcile**, each with a human
approval gate. The deciding rule: a change is *behavioral* (and so belongs to the 9-step loop
or Stage 9) if it would change any row in a feature's contract or event schema; otherwise it
is architectural.

The most subtle structural danger Codeos names is **vertical drift**, addressed by the
`shared-infrastructure-boundary` pattern. Workspace topology can enforce *lateral* isolation
— if module A cannot import module B, they cannot couple directly. But both A and B can
import a shared hub and each deposit domain logic there, coupling through the shared layer
while the topology looks intact. The hub becomes "a God module from below." The pattern's
**Diagnostic Test** is fast: *"Would a pure infrastructure module — one with zero knowledge
of the domain vocabulary — need this?"* If no, the addition encodes domain knowledge and
does not belong in the hub. A genuine exception passes a **Justification Gate**: write one
sentence explaining why it cannot live in a feature module; if that sentence is hard to
write, the addition is drift. Stage 10's Impact Analysis applies this test before
implementation, catching drift at the design gate rather than at reconciliation.

## The patterns library

Beyond the shared-infrastructure boundary, Codeos ships a small library of architectural
patterns in `dba/05-guidance/patterns/`. Each is explicitly *conditional* — applied when its preconditions
hold, never forced. Three are worth understanding in depth because they recur in
AI-assisted, vocabulary-driven systems.

### Vocabulary-centric architecture and the concept leak

When a system has types, statuses, or domain concepts defined by a configurable schema and
shared across modules, the `vocabulary-architecture` pattern governs how those concepts are
handled. It draws a sharp distinction between three things:

- **Concept** — the semantic identity the system reasons about, independent of any string.
- **Canonical** — the single stable runtime identifier the vocabulary chooses for a concept
  (exactly one per concept; if two canonicals identify one concept, the vocabulary is
  broken).
- **Alias** — an alternative input form accepted for compatibility or migration, translated
  to its concept *before* any domain logic runs.

Two rules follow. The **Concept Dependency Rule**: domain comparisons occur on resolved
concept identity, not on representations. The **Representation Ban Rule** (a hard
constraint): domain layers must not store, compare, branch on, or pattern-match vocabulary
representations — only concept identity, resolved through the vocabulary owner's API, is
valid in domain logic. Exactly one module owns each vocabulary; everyone else is a consumer
that calls the resolution API and never resolves on its own.

The failure this prevents is the **concept leak** — a representation escaping the resolution
boundary into domain logic. The pattern documents a real instance (the "R8" bug): a consumer
using normalize-on-read resolved the item's stored type correctly but compared it against a
hardcoded `Some("risk")` — an alias — while the canonical was `"Risk"`. Every item of
concept `Risk` silently failed to match. The fix resolves *both* sides through the same API
(`resolve(item_type) == resolve("risk")`), making the comparison concept-to-concept. The
standard falsifier for this class of bug — required as a contract Invariant Falsification row
— is the **canonical-casing fixture**: define a concept with capitalized canonical (`Risk`)
and lowercase alias (`risk`), store items under both, and assert identical outcomes. A
feature must also pick *one* resolution strategy (normalize-on-read, normalize-on-write, or
concept identifiers) and apply it uniformly; mixing strategies within one feature is itself a
violation, because it produces partial-resolution bugs that are harder to detect than the
original leak.

This pattern is the reason Stage 1 asks about vocabulary ownership, Stage 2 may add a
Vocabulary Dependency section with the Concept Dependency Invariant, and Stage 4 enforces the
Representation Ban.

### AI-mediated filtering observability

When a feature calls an external AI/LLM to generate candidates and then applies domain rules
to filter them, a behavioral event reporting `proposal_count: 8` is ambiguous: it is
consistent with "the LLM generated 12 and the filter rejected 4" *and* with "the LLM
generated exactly 8 and the filter never fired." Runtime observation cannot tell them apart —
a textbook `GAP (observability)` in Stage 7 terms: the post-generation-filtering invariant is
structurally and test-verified but never *observed*. The pattern's fix is minimal and stays
inside the rules: add a `generated_count` field beside `proposal_count`, with the schema
invariant `generated_count >= proposal_count`. The gap closes with no new event type and no
live LLM in tests; the falsifier is a mock returning one valid and one invalid candidate,
asserting `generated_count=2, proposal_count=1`. (A heavier alternative — a
`ProposalRejectedByVocabulary` observational event per rejection — is offered when
per-candidate audit trails are contractually required.) This pattern is a clean illustration
of the targeted-refinement philosophy: the cheapest observability change that makes an
invariant provable from events alone.

### Schema failure and vocabulary exclusion

The `schema-failure-and-exclusion` conventions handle cross-module failure honestly. When a
vocabulary consumer command fails because schema loading failed *and* the command is a
distinct observable business operation, two events are emitted: the vocabulary owner
(`project_schema`) emits the root cause (`SchemaNotFound` / `SchemaParseError` /
`SchemaValidationFailed`), and the consuming module emits its own
`[CommandName]FailedSchemaInvalid` — because "why schema loading failed" and "that this
command failed" are different facts serving different consumers. Relatedly, when a stored type
resolves to no concept, the unrecognized-type signal is owned by the vocabulary module
(`source_module: "project_schema"`), not the consumer — because it is a vocabulary-resolution
fact, not a consumer business decision. These conventions are explicitly scoped to the
conditions that produced them and are not imposed on every feature.

A `rust-project-structure` reference rounds out the library for Rust workspaces. The library
as a whole embodies a Codeos stance *(interpretation)*: architectural knowledge is captured
as conditional, auditable patterns with explicit "when this does and does not apply" sections,
rather than as universal mandates.

## Onboarding existing code (Session Type D)

For working code that has no DBA artifacts, Session Type D (`dba/03-prompts/workflow/00c-onboarding.md`)
bootstraps the minimum needed to enter the pipeline: per module, a draft Feature Brief, a
draft Intent (`status: HYPOTHESIZED_INTENT`), and a registry entry. Its central warning is
against **intent laundering** — describing the code's current behavior as if that behavior
were the intent, converting accidents into stated goals. The remedy is an evidence priority
(human interview first; then runtime behavior; then tests; source code structure last, and
least trusted for intent). Draft governance is owned by the Stage 3
`specification-approval` adapter.

## Human Navigation

Intent files are precision artifacts optimized for contract derivation, not for fast reading.
When a plain-language explanation is needed, the preferred pattern is to ask Claude directly
("Explain `[feature_id]` in plain English") — Claude reads the intent and explains on demand,
saving no file and creating no second intent surface. If a stored summary is genuinely needed,
it carries provenance metadata, is treated as generated output (regenerated, never hand-edited),
and is explicitly **not** a DBA artifact. This is the mechanism that prevents a summary from
quietly becoming a fake source of truth.

## Adoption levels

*This manual recommends* thinking about adoption as a gradient, not an all-or-nothing
commitment. (These levels are this manual's practical guidance; only the onboarding level
maps to a named repo concept — Session Type D.)

- **Light** — intent + contract + behavioral tests. Captures the discipline of stating
  observable truth before coding, without the full event/replay machinery.
- **Standard** — the full 9-step loop with the event spine.
- **Strict** — the full loop plus event spine, replay verification, review packages, and the
  architecture journal. Appropriate for production-critical or high-ambiguity-cost domains.
- **Existing-code onboarding** — Session Type D to bring legacy modules under governance
  before advancing them through Stage 1.

---

# Part VI — Worked Example: Real-Estate Ingestion

*Source basis: `maintenance/archive/terminology.md` (the real-estate ingestion example and event JSON shape);
`dba/05-guidance/templates/intent.md`, `contract.md`, `event-schema.md`; `dba/03-prompts/workflow/06-observe.md`,
`07-reconcile.md`, `09-refine.md`.*

> **Provenance disclaimer.** The system and the event names below are **derived from the
> real-estate ingestion example in `maintenance/archive/terminology.md`**. That file presents the example as a
> conceptual illustration of the DBA model, not as a complete, implemented project. The mini
> artifacts in this section are of two kinds, marked inline: **[sourced]** elements come
> directly from `maintenance/archive/terminology.md`; **[illustrative]** elements are written here to demonstrate
> the method's artifact shapes and are *not* existing repository artifacts. Nothing below
> should be read as a claim that the repository contains a production implementation of this
> system.

The system **[sourced]**: a real-estate ingestion and searchable-listing platform that
ingests listings from multiple agencies, normalizes them, removes duplicates, makes them
searchable, and continuously verifies correctness.

### Stage 1 — Intent **[illustrative artifact; outcome sourced]**

```text
listing_platform exists to let a property seeker find trustworthy, current listings
from many agencies in one place.

Specifically:
- A property seeker can search listings drawn from multiple agencies as one set.
- A property seeker can trust that the same property is not shown as several listings.

## Stable Guarantees
- listing_id is immutable once assigned.
- A deleted listing never appears in search results.

## Scope Boundary
This feature does NOT: rank or score listings; contact agencies; handle payments.
```

Observe the rules: actor is a human role (property seeker), outcomes are abilities, and no
mechanism appears.

### Stage 2 — Behavioral Contract **[illustrative]**

Happy path:

```gherkin
Given a source sends a valid listing
When the platform ingests it
Then a ListingIngested observation is recorded
And the listing eventually becomes searchable
```

A falsification scenario targeting the deduplication invariant:

```gherkin
Given two listings share the same external_id and source
When both are ingested
Then exactly one canonical listing exists
Falsifies: dedup keyed on normalized address only → identical external_id with a
           slightly different address string would create two canonical listings
```

A boundary scenario (the kind a future maintainer might wrongly "fix"):

```gherkin
Given a source sends a listing whose normalized address matches an existing canonical
  listing but whose external_id and source differ
When it is ingested
Then a second canonical listing is created
And both remain searchable
```

This boundary case states deliberately that same-address-but-different-source listings are
*not* duplicates — without it, a later change could silently collapse two genuine listings.

Failure Classifications (excerpt): `InvalidListingRejected` — trigger: a source sends a
malformed listing; observable signal: a `ListingRejected` FAILURE event. Every named failure
here gets a scenario above and a FAILURE event in Stage 3.

### Stage 3 — Event Schema **[event names sourced; payloads illustrative]**

Behavioral and observational events, each with the six base fields:

```
RawFeedReceived       (OBSERVATIONAL)  ← a source delivered a feed item
  ↓
ListingIngested       (BEHAVIORAL)     ← a valid listing entered the system
  ↓
AddressNormalized     (BEHAVIORAL)
  ↓
CanonicalListingCreated (BEHAVIORAL)   ← dedup resolved to one canonical listing
  ↓
SearchProjectionUpdated (BEHAVIORAL)   ← listing is searchable
  ↓ (on invalid input)
ListingRejected       (FAILURE)        ← payload: failure_reason: string
```

Coverage check: the contract failure `InvalidListingRejected` maps to the FAILURE event
`ListingRejected` — COVERED.

### Stage 4 — Implementation (constrained) **[illustrative]**

The implementation emits exactly these events and no others, propagates `correlation_id`
from the moment a feed item arrives, and catches only the contracted failure
(`ListingRejected`). A Contract Satisfaction Table maps each clause to a code location; an
Event Emission Table maps each schema event to where it is emitted.

### Stage 5 — Behavioral and replay tests **[illustrative]**

A behavioral test asserts the observable outcome in exact schema language; a replay test
asserts determinism and chain integrity:

```text
test_valid_listing_ingested_succeeds:
  given a valid feed item
  when ingested
  then an event of type "ListingIngested" is emitted
  and its correlation_id is present and non-empty
  and the six base fields are all present

test_duplicate_resolves_to_one_canonical_falsifies_address_only_dedup:
  given two items sharing external_id + source, with differing address strings
  when both are ingested
  then exactly one "CanonicalListingCreated" event is emitted
  (fails if dedup keys on normalized address only)

test_listing_platform_event_sequence_is_deterministic:
  run the happy path twice with the same input
  assert both runs emit the same event_type sequence
```

Note what the tests do *not* do: they never assert on private functions or intermediate
state, only on emitted events and resulting state. The Contract Coverage Table then maps
every scenario and every invariant-falsification row to exactly one test.

### Stage 6 — A runtime event log line **[shape sourced from `maintenance/archive/terminology.md`]**

```json
{"event_id":"evt-001","event_type":"ListingIngested","timestamp":1710000000000,"correlation_id":"listing-flow-777","source_module":"feed_ingestion","payload":{"listing_id":"abc123","source":"agency_x"}}
```

### Stage 7 — A reconciliation table row **[illustrative]**

| Item | Intent | Contract | Schema | Impl | Tests | Runtime (evidence) | Status |
|---|---|---|---|---|---|---|---|
| Dedup resolves to one canonical listing | ✓ | ✓ | ✓ | ✓ | ✓ | 1 (Direct) | ALIGNED |
| Deleted listing never searchable | ✓ | ✓ | ✓ | ✓ | ✓ | 3 (Test) | GAP (runtime evidence) |

The second row shows the method working: the behavior is implemented and tested, but the
deletion path was never observed at runtime, so the gap is surfaced rather than assumed
closed.

### Stage 8 — A replay guarantee **[illustrative]**

For one `correlation_id`, the observed sequence
`RawFeedReceived → ListingIngested → AddressNormalized → CanonicalListingCreated →
SearchProjectionUpdated` matches the expected flow, the chain is complete (trigger →
terminal BEHAVIORAL event), and re-running the same input yields the same `event_type`
sequence.

### Stage 9 — A targeted refinement **[sourced from `maintenance/archive/terminology.md`]**

Runtime metrics reveal `duplicate_detection_accuracy = 82%`; the observed cause is that
address abbreviations bypass deduplication. The refinement does not redesign the system. It
narrows the dedup contract clause — from "match on exact normalized address" to "match on
normalized address + geo proximity + fuzzy similarity" — then updates the affected tests and
module and re-runs the affected stages. Trigger: RECURRING_FAILURE. Type: behavioral. This
is the smallest effective, evidence-backed change.

---

# Part VII — Evaluation

*Source basis: active DBA components selected through `dba-system.md`, `README.md`
(strengths/constraints); `dba/03-prompts/review/pipeline-reviewer.md`
(reviewer freedom to challenge DBA itself); user-supplied OAP framing (external).*

## Strengths

- **Traceability.** Every piece of code traces back to an event, a contract clause, and an
  intent statement; what is not in the chain is not supposed to exist.
- **AI constraint.** Claude may not create unapproved behavior, additional events, or
  speculative abstractions.
- **Runtime provability** *(operational sense)*. The system is judged not only by static
  code or tests but by the events it emits during execution.
- **Deterministic verification.** Replay tests check that the runtime event sequence stays
  consistent with the schema across runs.
- **Human control.** Authority and decision boundaries remain explicit in the selected doctrine and
  its adapters rather than being inferred from agent behavior.
- **Institutional memory.** Handoffs, review logs, and the architecture journal keep
  important decisions from being lost in conversation history.
- **Anti-drift architecture.** The shared-infrastructure-boundary pattern names and contains
  vertical drift.
- **Fit for complex domains.** Beginning from actor, outcome, contracts, and observable
  truth makes the method valuable where correct behavior matters more than speed of first
  implementation.

## Limitations and risks

The honest framing is the **cost of proof**: Codeos trades speed of first implementation for
lower behavioral ambiguity, stronger auditability, and safer AI-assisted evolution. It is
not always faster to first commit; it can be far cheaper at incident time, review time, and
during later maintenance.

- **Overhead for small or exploratory work.** For one-off scripts the process can be too
  heavy; for research prototyping where the intent is not yet known, it can be premature; for
  creative UI experimentation the constraints can feel too rigid.
- **Artifact fatigue.** If a human mechanically approves intent, contracts, and schema, the
  method loses its meaning. Codeos requires an active human, not a rubber stamp.
- **False completeness.** If the schema, tests, or reconciliation are written poorly, the
  system will prove the *wrong* truth. Structure does not substitute for judgment.
- **Scope tension.** The ban on unapproved abstractions is useful but, for some systems,
  requires a well-separated architectural-refinement workflow so that infrastructure changes
  are not forced into the behavioral feature loop.

A mature reading: Codeos is worth its cost when the cost of behavioral ambiguity is high. It
is excessive when speed of exploration matters more than traceable behavioral evidence.

## Codeos vs OAP — comparison and synthesis

OAP (Orchestrated Agentic Programming) and Codeos address a related problem — how to use AI
in software development without losing control — but their centers of gravity differ.

- **OAP** separates the human, a strategic AI, and an execution agent; its subject is the
  *orchestration* of an agentic delivery process, with emphasis on governance, validation
  debt, and human release authority.
- **Codeos** more strictly bounds a *single* AI collaborator with a chain of artifacts; its
  subject is *provable behavioral conformance* at the level of individual feature
  development.

They are not competitors so much as layers. OAP is the stronger general operating model for
organizing agentic delivery across roles; Codeos is the stronger micro-methodology for
preventing AI improvisation inside one feature's development. The synthesis:

> OAP tells the human how to orchestrate agentic delivery. Codeos tells the agentic
> collaborator how not to drift from approved behavior.

Read this way, Codeos can serve as the constitution-and-workflow layer *inside* an
OAP-orchestrated process: OAP decides who does what and when to release; Codeos decides what
the execution collaborator is permitted to build and how that work is proven.

## Conclusion

Codeos matters because it shows clearly what the next stage of AI-assisted programming looks
like. The future is not only stronger models, longer context, or faster code generation. It
is better evidence loops. When code becomes cheap, evidence becomes valuable. Codeos builds
that evidence systematically: intent, contract, event, implementation, test, runtime log,
reconciliation, replay, and targeted refinement.

That is why Codeos is not merely a set of prompts for Claude Code. It is an attempt to shape
a discipline of development in which the human establishes governed meaning while the AI retains
normal implementation freedom inside that boundary—and in which "it works" is not a final claim
but the beginning of verification.

---

# Appendix A — Source Map

This appendix maps the manual's major claims, normative rules, stage descriptions, named
events, and key file references to their repository sources. It maps load-bearing claims, not
every sentence.

| Manual claim / element | Repository artifact | Path |
|---|---|---|
| Codeos = DBA / IDS methodology; symlinkable toolkit | README / active DBA components | `README.md`, `dba-system.md` → active configuration |
| 9-step loop (Intent → … → Refinement) | README and stage prompts | `README.md`, `dba/03-prompts/workflow/` |
| Authority, package lifecycle, behavioral constraints, human control, escalation | Doctrine | `dba-system.md` → `doctrine` |
| Truth Authority model + safety/invariant exception | Doctrine | `dba-system.md` → `doctrine` |
| Operational artifact requirements | Prompts and templates | `dba/03-prompts/workflow/`, `dba/05-guidance/templates/` |
| DBA vocabulary (Event Spine, Correlation ID, event kinds) | Terminology, prompts, templates | `dba/05-guidance/terminology.md`, `dba/03-prompts/workflow/`, `dba/05-guidance/templates/` |
| Review Logging; Architecture Journal `AJ-NNN` format | Review policy | `dba-system.md` → `review_policy` |
| Human Navigation (no fake source of truth) | Manual guidance | `dba/06-reference/codeos-manual.md` |
| Stage 0 session types A/B/C/D; STOP discipline | Session-start prompt | `dba/03-prompts/workflow/00-session-start.md` |
| Stage 1 intent rules; cross-examination | Stage 1 prompt; template | `dba/03-prompts/workflow/01-intent.md`, `dba/05-guidance/templates/intent.md` |
| Stage 2 contracts; boundary + falsification scenarios | Stage 2 prompt; template | `dba/03-prompts/workflow/02-contract.md`, `dba/05-guidance/templates/contract.md` |
| Stage 3 event spine; six base fields; categories; observation mode | Stage 3 prompt; template | `dba/03-prompts/workflow/03-event-schema.md`, `dba/05-guidance/templates/event-schema.md` |
| Stage 4 constrained satisfier; no extra events/abstractions | Stage 4 prompt | `dba/03-prompts/workflow/04-implement.md` |
| Stage 5 behavioral + replay tests; determinism assertion | Stage 5 prompt | `dba/03-prompts/workflow/05-tests.md` |
| Stage 6 append-only log; runtime fixtures | Stage 6 prompt | `dba/03-prompts/workflow/06-observe.md` |
| Stage 7 statuses; Schema Payload Drift; Evidence Quality scale | Stage 7 prompt | `dba/03-prompts/workflow/07-reconcile.md` |
| Stage 8 replay guarantee; chain integrity; scope note | Stage 8 prompt | `dba/03-prompts/workflow/08-replay.md` |
| Stage 9 valid/forbidden triggers; cost order | Stage 9 prompt | `dba/03-prompts/workflow/09-refine.md` |
| Stage 10 5-step architectural refinement loop | Arch-refine prompt | `dba/03-prompts/workflow/10-arch-refine.md` |
| Session Type D onboarding; intent laundering; evidence priority | Onboarding prompt | `dba/03-prompts/workflow/00c-onboarding.md` |
| Reviewer = independent critical assessor; Attention Level | Reviewer package | `dba/03-prompts/review/pipeline-reviewer.md` |
| Review Package is inline, not written to disk | Review-package template | `dba/05-guidance/templates/review-package.md` |
| Vertical drift; Diagnostic Test; Justification Gate | Pattern | `dba/05-guidance/patterns/shared-infrastructure-boundary.md` |
| Project scaffolding; directories; git init | Init script | `dba/04-tools/initializer/dba-init.sh` |
| Real-estate example; event names; event JSON shape | Historical terminology example | `maintenance/archive/terminology.md` |
| OAP framing and synthesis | External (user-supplied OAP handbook) | not in repository |

---

# Appendix B — DBA Vocabulary

The canonical project vocabulary is defined once in `dba/05-guidance/terminology.md`. Read that glossary rather
than maintaining a duplicate definition table here.

---

*This manual is a generated documentation artifact. It is not itself a DBA artifact: it does
not pass through the 9-step loop and carries no `status`, `approved_by`, or
`derived_contracts` fields. Regenerate it from the repository when the toolkit changes.*
