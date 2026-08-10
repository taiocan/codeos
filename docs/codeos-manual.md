# Codeos: Declarative Behavioral Architecture for Provable AI-Assisted Software Evolution

*A doctrinal manual for human-gated, artifact-constrained, event-verifiable development with Claude Code.*

---

## Abstract

Codeos is a development toolkit and methodology for tightly controlled, AI-assisted
software evolution. It implements **Declarative Behavioral Architecture (DBA)** — also
called **Intent-Driven System (IDS)** mode in its own instructions. Its organizing idea
is simple to state and consequential in practice: software is not built starting from
code, but from *behavioral truth*. A feature begins as an approved statement of intent,
becomes a set of observable behavioral contracts, then a constraining event schema, and
only then is it implemented — and the implementation may contain nothing that does not
trace back to an approved artifact.

This manual presents Codeos as a *method*, not as a prompt collection. It defines the
method, explains the dangers it answers, walks its full development loop with a single
worked example, states its non-negotiable rules, and explains its verification machinery
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
the active DBA components selected through `dba-system.md`, `README.md`, `terminology.md`, the
stage prompts in `prompts/`, the artifact templates in `templates/`, the architectural patterns
in `patterns/`, and `scripts/dba-init.sh`. Major claims are mapped to their source files in
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
through `dba-system.md` (mode declaration, non-negotiable rules, vocabulary); `terminology.md`
(DBA/IDA framing); `prompts/00c-onboarding.md`
(intent laundering); `patterns/shared-infrastructure-boundary.md` (vertical drift).*

## Why Codeos exists

Classic AI-assisted development has a recurring weakness: the leap from idea to code. A
human describes a wish, the AI writes a solution, and the two then chase bugs together.
This is fast but brittle. It is frequently unclear which requirement was actually
implemented, which edge cases are covered, whether the tests verify behavior or merely
internal structure, and whether the running system emits any evidence that can be analyzed
later.

Codeos exists to forbid that leap. Its mechanism is not a more powerful model — it is a
better *structure of truth*. The leap is replaced by an ordered loop in which each step
produces an artifact and each artifact must be approved by a human before the next step
begins.

## Failure / threat model — the dangers Codeos answers

*(This section is interpretation: it organizes the specific failure modes that the
repository's rules and stages are built to contain. Each danger is paired with the
mechanism that contains it.)*

Codeos is best understood as a response to concrete, recurring failures of AI-assisted
development:

| Danger | What it looks like | Contained by |
|---|---|---|
| **Intent-to-code leap** | AI jumps from a wish straight to an implementation | Non-negotiable rule: no implementation before intent + contract + schema are approved (Stages 1–4 gates) |
| **Hidden abstractions** | AI adds helper layers, "service" abstractions, speculative generality | Stage 4 constraint: no abstractions the contract does not require |
| **Unapproved events** | Runtime emits behavior nobody specified | Stage 3 event spine: implementation may emit *only* schema events; extras are structurally visible |
| **Implementation-asserting tests** | Tests check private methods and internal state, so they pass while behavior is wrong | Stage 5 rule: tests assert observable outcomes only, in event-schema language |
| **Runtime/schema divergence** | The log contains fields or types the schema never declared | Stage 7 Schema Payload Drift check (MATCH/TYPE_MISMATCH/ABSENT/EXTRA) |
| **Rubber-stamping** | Human approves artifacts mechanically; gates become theater | Reviewer role as independent critical assessor; per-stage suggested examination areas |
| **Vertical drift** | Domain logic seeps into a shared infrastructure module, coupling features through the hub | `shared-infrastructure-boundary` pattern; Stage 10 Impact Analysis gate |
| **Fake source of truth** | A generated plain-language summary is edited and silently becomes authoritative | Human Navigation rule: stored summaries carry provenance, are regenerated, are never DBA artifacts |

Each later Part returns to these dangers and shows the stage or rule that addresses it.

## Core definition

Codeos is a **human-gated, artifact-constrained methodology for AI-assisted software
evolution**, in which every feature is developed *from* an approved intent *into*
observable behavioral contracts, *from* contracts *into* an event schema, *from* the
schema *into* a minimal implementation, *from* the implementation *into* behavioral and
replay tests, and then *through* runtime observation, reconciliation audit, and a targeted
refinement loop.

Stated more briefly *(interpretation)*: Codeos is a system that prevents an AI from "doing
something clever" until it is clear what must be true, how that truth will be observable,
and what runtime evidence will show whether the truth still holds.

## What Codeos is

Codeos is, simultaneously:

- a **methodology** — it prescribes an ordered sequence of stages and human approvals;
- a **toolkit** — it ships governed DBA components selected through `dba-system.md`, stage prompts (`prompts/`),
  artifact templates (`templates/`), architectural patterns (`patterns/`), and an
  initializer (`scripts/dba-init.sh`), symlinked into a project as `.codeos`;
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

- **Not vibe coding.** It does not rest on a feeling that "the solution looks right." Every
  stage requires an artifact and an approval.
- **Not ordinary AI pair programming.** The human does not co-write code line by line; the
  human approves intents, contracts, schemas, tests, and refinements.
- **Not a fully autonomous agent system.** The active doctrine selected through `dba-system.md` forbids autonomous planning,
  self-direction, and multi-step autonomous execution; Claude may not skip stages,
  implement before specification, or advance without explicit approval.
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

*Source basis: the `doctrine` component selected through `dba-system.md` (non-negotiable rules,
Truth Authority, "What You NEVER Do");
stage prompts `01`–`09` (per-stage roles); `prompts/pipeline-reviewer.md` (reviewer role).*

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

The human is the primary holder of intent and approvals. This does not require the human to
hand-write every contract, schema, or test. It requires the human to decide when an
artifact is acceptable and when Claude may advance.

A human reviewing each stage should attend to *(this manual recommends the following
checklist, distilled from the stage prompts)*:

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
- **Stage 2** — behavioral contract specialist: derive observable contracts from the
  approved intent.
- **Stage 3** — define the event spine.
- **Stage 4** — *constrained satisfier* (the prompt's own term): implement only what the
  three approved artifacts require.
- **Stage 5** — write behavioral and replay tests.
- **Stage 7** — behavioral auditor performing reconciliation (explicitly *not* a code
  reviewer).
- **Stage 8** — verify replayability.
- **Stage 9** — propose the smallest evidence-backed fix.

Claude's task is never to create a system by its own judgment, but to stay inside the
approved chain. When a contract clause cannot be satisfied without adding something
unapproved, the Stage 4 rule is explicit: Claude flags it and stops, rather than silently
adding it.

## The non-negotiable rules

The `doctrine` component selected through `dba-system.md` states six non-negotiable rules. They
are reproduced here in meaning (these are repo-backed MUST/NEVER rules):

1. **Every stage transition requires explicit human approval.** Claude NEVER advances
   without an explicit "APPROVED" / "yes proceed" or equivalent.
2. Claude NEVER implements before intent + contract + event schema are *all* approved.
3. Claude NEVER adds abstractions, patterns, or behaviors beyond what the current approved
   artifacts specify.
4. Claude NEVER emits events not listed in the approved event schema.
5. Claude NEVER invents hidden behavior — all behavior must trace to an approved artifact.
6. After producing any stage output, Claude STOPS and states `AWAITING HUMAN APPROVAL`.

The selected `doctrine` component also enumerates "What You NEVER Do," which restates and extends these:
implement before approval; add abstractions the contracts do not demand; add "just in case"
error handling not in the contract's failure modes; emit unlisted events; advance without
approval; suggest full rewrites; add autonomous planning or multi-step autonomous
execution; or modify `events/runtime_events.jsonl`, which is append-only.

The visible discipline of these rules is the `AWAITING HUMAN APPROVAL` stop. Every stage
prompt ends by emitting a stop token (for example
`AWAITING HUMAN APPROVAL TO PROCEED TO STAGE 2`). The stop is not decoration; it is the
gate.

## Truth Authority and conflict resolution

When intent, runtime evidence, and structural analysis disagree, the `doctrine` component selected
through `dba-system.md` defines an authority model. This manual presents it with its exact categories and caveats and does
**not** reduce it to a simple ranking — the safety caveat in particular inverts what a
naive ranking would suggest:

1. **Explicit human correction** at any stage gate overrides all other sources.
2. **Runtime behavior** (observed events) overrides intent *text* when behavior is more
   specific. Example from the selected `doctrine` component: schema declares `"string"` but runtime consistently
   emits an integer — this is empirical evidence of intent-text drift, not a runtime error.
3. **Safety, authorization, and invariant-enforcement logic** always preserves *intent
   primacy*, regardless of runtime behavior. Example: if runtime shows no authorization
   check was invoked, that is a contract violation — **not** a license to redesign
   authorization around the observed behavior. This is the critical exception: for
   safety/authorization/invariant logic, intent wins even over runtime.
4. **Structural digest observations** (fan-in, god functions, known risk zones) do *not*
   override behavioral findings. They inform blast-radius estimates and remediation
   sequencing only — they are advisory.

When a conflict cannot be resolved by these rules, the instruction is to surface it to the
human rather than silently resolving it. A flattened "human > runtime > intent" summary is
therefore wrong: it would discard the safety exception, which is the most important part of
the model.

---

# Part III — The DBA Loop

*Source basis: `README.md` (Stage Purposes, key constraints); the `doctrine` component selected
through `dba-system.md` (the 9-step loop);
stage prompts `00-session-start`, `01`–`09`; `templates/` (artifact shapes).*

Codeos has a nine-stage *feature* loop. It also has a Stage 0 *session discipline* that is
not a feature stage. The human-gated character of Codeos begins before Stage 1 — in how a
session is started.

## Stage 0 — Session Start / Operating Preflight

Stage 0 is not part of the feature lifecycle; it is the operating preflight that places
Claude into DBA mode, scopes the session, names forbidden actions, and prevents accidental
autonomous work. The session-start prompt (`prompts/00-session-start.md`) directs Claude to:

- read `.codeos/dba-system.md`, follow it to the selected doctrine, and state the non-negotiable rules;
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
- **Human gate** — what the human approves before advancing.
- **Prevented failure mode** — the danger (from Part I) it contains.
- **Proof produced** — the evidence this stage adds to the chain.
- **Claude constraints** — what Claude may not do here.
- **Verification questions** — what a reviewer asks.

### Stage 1 — Intent

- **Purpose:** Capture *why* a feature exists, in actor + outcome form, implementation-free.
  This is the foundation every downstream artifact derives from.
- **Primary artifact:** `intents/[feature_id].md` (from `templates/intent.md`): a purpose
  statement, a few actor-outcome statements, stable guarantees, and an explicit scope
  boundary.
- **Human gate:** Human approves the intent before Stage 2.
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

- **Purpose:** Translate the approved intent into independently testable, observable-only
  truth.
- **Primary artifact:** `contracts/[feature_id]_contract.md` (from `templates/contract.md`):
  Given/When/Then scenarios, invariants, an Invariant Falsification Scenarios table,
  pre/postconditions, a Failure Classifications table, and optional Runtime Context and
  Vocabulary Dependency sections.
- **Human gate:** Human approves the contract before Stage 3.
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
- **Primary artifact:** `events/[feature_id]_schema.md` (from `templates/event-schema.md`):
  named events with categories and payloads, an event flow diagram, and a Coverage Check
  table mapping every contract failure to a FAILURE event.
- **Human gate:** Human approves the schema before Stage 4. This is the gate that constrains
  everything that follows.
- **Prevented failure mode:** hidden behavior — once approved, implementation may emit only
  the listed events, so undeclared behavior becomes architecturally impossible.
- **Proof produced:** the closed set of permitted runtime observations, against which the
  log will later be reconciled and replayed.
- **Claude constraints:** the schema must not be *stronger* than the contract — every new
  observable (payload field, event, ordering guarantee) must trace to an approved contract
  clause; `correlation_id` is mandatory on every event without exception; validation
  ordering is not prescribed unless the contract requires it. *Observation-mode note:* if
  the contract's Runtime Context declares `observation_mode: external-observation`, Stage 3
  is skipped and evidence comes from test layers or an acceptance artifact instead — this is
  exceptional; event-emitting features use the schema.
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
- **Human gate:** Human approves the implementation before Stage 5.
- **Prevented failure mode:** hidden abstractions, unapproved events, speculative error
  handling, undeclared runtime artifacts.
- **Proof produced:** code in which the first thing wired up is correlation-ID propagation
  and event emission, so all behavior is traceable.
- **Claude constraints (all repo-backed):** no additional abstractions; no additional
  events (a needed new event means *stop and request a schema update*); no undeclared
  runtime artifacts (only `events/runtime_events.jsonl` unless the contract's Runtime
  Artifacts section names another); no speculative error handling (only contracted failure
  modes are caught; others propagate); implementation must be deterministic; if the feature
  consumes a vocabulary, the Representation Ban applies. All three artifacts must be APPROVED
  before this stage; implementation without all three is a DBA violation.
- **Verification questions:** Are any clauses satisfied in a surprising or fragile way? Does
  the implementation introduce behavior not traceable to intent, contract, or schema? What
  is the most likely Stage 7 gap?

### Stage 5 — Tests

- **Purpose:** Write behavioral truth anchors that fail if observable behavior deviates from
  the contracts.
- **Primary artifact:** `tests/behavioral/[feature_id]_behavior.test.[ext]` and
  `tests/replay/[feature_id]_replay.test.[ext]`, plus a Contract Coverage Table.
- **Human gate:** Human approves the tests (and runs them) before Stage 6.
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

- **Purpose:** The human runs the implementation; `events/runtime_events.jsonl` becomes
  operational truth.
- **Primary artifact:** the populated append-only runtime event log.
- **Human gate:** the human confirms events are captured ("events captured, ready for Stage
  7"); Claude does not advance until then.
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
- **Human gate:** Human approves, or directs return to an earlier stage.
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
- **Human gate:** Human approves to proceed to Stage 9 or marks the feature complete.
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
- **Primary artifact:** refinement records (from `templates/refinement.md`) plus minimal
  diffs and a list of stages to re-run.
- **Human gate:** the human approves each refinement *individually*; affected stages are
  re-run before the next refinement is addressed.
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
`prompts/03-event-schema.md`, `06-observe.md`, `07-reconcile.md`, `08-replay.md`,
`09-refine.md`; `terminology.md` (event JSON shape).*

The strongest claim Codeos makes is operational, not formal. Its three load-bearing
mechanisms are the event spine, reconciliation, and replay.

## Event Spine as operational truth

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

*Source basis: the active DBA components selected through `dba-system.md` (Artifact Classification,
Review Logging, Human Navigation, Architectural Refinement); `templates/` (review-package,
review-file, handoff);
`prompts/10-arch-refine.md`, `00c-onboarding.md`, `pipeline-reviewer.md`;
`patterns/shared-infrastructure-boundary.md`; `scripts/dba-init.sh`.*

## Required vs optional artifacts

Not every artifact blocks progress. The selected `doctrine` component draws a clear line:
**required artifacts block stage advancement; optional and recommended artifacts improve decision quality but are
never prerequisites.**

- **Required (block advancement):** Intent (before Stage 2), Contract (before Stage 3), Event
  Schema (before Stage 4).
- **Recommended:** Feature Registry (`features/registry.yaml`) for multi-feature projects.
- **Optional:** Feature Brief (`backlog/[id].md`), Codebase Digest
  (`docs/codebase-digest.md`), the Stage 7 Structural Alignment section, Architectural
  Refinement records.
- **Onboarding-only:** `HYPOTHESIZED_INTENT` drafts from Session Type D, which must pass
  Stage 1 review before they count as approved.

## Templates, prompts, and project setup

The toolkit ships fill-in templates for every artifact (intent, contract, event schema,
feature spec, refinement, arch-refinement, codebase digest, conventions, feature brief,
feature registry, handoff, project `CLAUDE.md`, review file, review package) and a
stage-gated prompt for each step.

`scripts/dba-init.sh`, run from a new project root, scaffolds the project: it creates the
`.codeos` symlink to the toolkit; creates `intents/`, `contracts/`, `events/`, `modules/`,
`tests/behavioral/`, `tests/replay/`, `docs/`, `features/`, `backlog/`, and
`refinements/arch/`; seeds `features/registry.yaml`, an empty `events/runtime_events.jsonl`,
a project `CLAUDE.md`, `docs/conventions.md`, and a codebase-digest placeholder; initializes
git on branch `main`; and optionally adds a remote. The human then fills in the project
intent and conventions and pastes `prompts/00-session-start.md` to begin.

## The reviewer interface

Codeos separates the *implementer* role (Claude in the loop) from an *independent reviewer*.
The Reviewer Activation Package (`prompts/pipeline-reviewer.md`) primes a separate reviewer —
human or a second LLM — as an **independent critical assessor, not a DBA compliance
auditor**. The reviewer may challenge anything: assumptions, architecture, scope, framing,
alternatives, or DBA itself. Its output uses an **Attention Level** (High / Medium / Low) as
a scannability signal — explicitly **not** APPROVED / REVISE / BLOCKING verdicts, which would
turn the reviewer into a gatekeeper. The human decides what to act on.

At the end of each stage, Claude presents a **Review Package** inline (from
`templates/review-package.md`): the artifact, the stage purpose, three suggested examination
areas, and known tensions. Crucially, this package is *not written to disk* — it is a
convenience view for copy-paste to a reviewer; the authoritative record lives in
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
Codeos provides an alternate **5-step architectural-refinement loop** (`prompts/10-arch-refine.md`):
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
patterns in `patterns/`. Each is explicitly *conditional* — applied when its preconditions
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

For working code that has no DBA artifacts, Session Type D (`prompts/00c-onboarding.md`)
bootstraps the minimum needed to enter the pipeline: per module, a draft Feature Brief, a
draft Intent (`status: HYPOTHESIZED_INTENT`), and a registry entry. Its central warning is
against **intent laundering** — describing the code's current behavior as if that behavior
were the intent, converting accidents into stated goals. The remedy is an evidence priority
(human interview first; then runtime behavior; then tests; source code structure last, and
least trusted for intent) and a hard rule that drafts are never APPROVED until Stage 1 review
processes them.

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

*Source basis: `terminology.md` (the real-estate ingestion example and event JSON shape);
`templates/intent.md`, `contract.md`, `event-schema.md`; `prompts/06-observe.md`,
`07-reconcile.md`, `09-refine.md`.*

> **Provenance disclaimer.** The system and the event names below are **derived from the
> real-estate ingestion example in `terminology.md`**. That file presents the example as a
> conceptual illustration of the DBA model, not as a complete, implemented project. The mini
> artifacts in this section are of two kinds, marked inline: **[sourced]** elements come
> directly from `terminology.md`; **[illustrative]** elements are written here to demonstrate
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

### Stage 6 — A runtime event log line **[shape sourced from `terminology.md`]**

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

### Stage 9 — A targeted refinement **[sourced from `terminology.md`]**

Runtime metrics reveal `duplicate_detection_accuracy = 82%`; the observed cause is that
address abbreviations bypass deduplication. The refinement does not redesign the system. It
narrows the dedup contract clause — from "match on exact normalized address" to "match on
normalized address + geo proximity + fuzzy similarity" — then updates the affected tests and
module and re-runs the affected stages. Trigger: RECURRING_FAILURE. Type: behavioral. This
is the smallest effective, evidence-backed change.

---

# Part VII — Evaluation

*Source basis: active DBA components selected through `dba-system.md`, `README.md`
(strengths/constraints); `prompts/pipeline-reviewer.md`
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
- **Human control.** Every stage transition requires explicit human approval.
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
a discipline of development in which the AI is granted no freedom until a human has
established what is true — and in which "it works" is not a final claim but the beginning of
verification.

---

# Appendix A — Source Map

This appendix maps the manual's major claims, normative rules, stage descriptions, named
events, and key file references to their repository sources. It maps load-bearing claims, not
every sentence.

| Manual claim / element | Repository artifact | Path |
|---|---|---|
| Codeos = DBA / IDS methodology; symlinkable toolkit | README / active DBA components | `README.md`, `dba-system.md` → active configuration |
| 9-step loop (Intent → … → Refinement) | Doctrine; README | `dba-system.md` → `doctrine`, `README.md` |
| Six non-negotiable rules; "What You NEVER Do" | Doctrine | `dba-system.md` → `doctrine` |
| Truth Authority model + safety/invariant exception | Doctrine | `dba-system.md` → `doctrine` |
| Artifact Classification (required/recommended/optional) | Doctrine | `dba-system.md` → `doctrine` |
| DBA vocabulary (Event Spine, Correlation ID, event kinds) | Doctrine | `dba-system.md` → `doctrine` |
| Review Logging; Architecture Journal `AJ-NNN` format | Review policy | `dba-system.md` → `review_policy` |
| Human Navigation (no fake source of truth) | Doctrine | `dba-system.md` → `doctrine` |
| Stage 0 session types A/B/C/D; STOP discipline | Session-start prompt | `prompts/00-session-start.md` |
| Stage 1 intent rules; cross-examination | Stage 1 prompt; template | `prompts/01-intent.md`, `templates/intent.md` |
| Stage 2 contracts; boundary + falsification scenarios | Stage 2 prompt; template | `prompts/02-contract.md`, `templates/contract.md` |
| Stage 3 event spine; six base fields; categories; observation mode | Stage 3 prompt; template | `prompts/03-event-schema.md`, `templates/event-schema.md` |
| Stage 4 constrained satisfier; no extra events/abstractions | Stage 4 prompt | `prompts/04-implement.md` |
| Stage 5 behavioral + replay tests; determinism assertion | Stage 5 prompt | `prompts/05-tests.md` |
| Stage 6 append-only log; runtime fixtures | Stage 6 prompt | `prompts/06-observe.md` |
| Stage 7 statuses; Schema Payload Drift; Evidence Quality scale | Stage 7 prompt | `prompts/07-reconcile.md` |
| Stage 8 replay guarantee; chain integrity; scope note | Stage 8 prompt | `prompts/08-replay.md` |
| Stage 9 valid/forbidden triggers; cost order | Stage 9 prompt | `prompts/09-refine.md` |
| Stage 10 5-step architectural refinement loop | Arch-refine prompt | `prompts/10-arch-refine.md` |
| Session Type D onboarding; intent laundering; evidence priority | Onboarding prompt | `prompts/00c-onboarding.md` |
| Reviewer = independent critical assessor; Attention Level | Reviewer package | `prompts/pipeline-reviewer.md` |
| Review Package is inline, not written to disk | Review-package template | `templates/review-package.md` |
| Vertical drift; Diagnostic Test; Justification Gate | Pattern | `patterns/shared-infrastructure-boundary.md` |
| Project scaffolding; directories; git init | Init script | `scripts/dba-init.sh` |
| Real-estate example; event names; event JSON shape | Terminology | `terminology.md` |
| OAP framing and synthesis | External (user-supplied OAP handbook) | not in repository |

---

# Appendix B — DBA Vocabulary

The canonical vocabulary, as defined in `dba-system.md` and `terminology.md`. These terms are
repo-backed; this appendix reproduces their meaning for reference.

| Term | Definition |
|---|---|
| **Intent** | Why a feature exists. Actor + outcome form. No implementation details. |
| **Behavioral Contract** | Observable truths derived from intent. BDD Given/When/Then. |
| **Event Spine** | The complete ordered set of events a feature is permitted to emit. |
| **Observational Event** | A raw runtime fact (e.g., `RequestReceived`). |
| **Behavioral Event** | A verified outcome (e.g., `CartItemAdded`). |
| **Failure Event** | A classified error condition (e.g., `CartItemAddFailed`). |
| **External Event** | A side effect on an outside system (e.g., `EmailSent`). |
| **Reconciliation Review** | Structural comparison of all artifacts against each other for gaps and mismatches. |
| **Replay Verification** | Confirming the runtime event log conforms to schema and contract sequence. |
| **Targeted Refinement** | The smallest effective change for a specific observed problem — not a rewrite. |
| **Correlation ID** | A UUID linking all events from a single feature execution chain. |
| **Shared Infrastructure Module** | A module depended on by ≥2 feature modules providing only mechanical infrastructure (event emission, DTOs, constants, re-exports); never domain logic. |
| **Vertical Drift** | Accumulation of domain logic in a shared infrastructure module, bypassing lateral isolation even when feature→feature imports are blocked. |
| **Concept / Canonical / Alias** | The semantic identity reasoned about; its one stable runtime identifier; an alternative input form resolved to the concept before domain logic runs. |
| **Concept Leak** | A bug where a vocabulary representation escapes the resolution boundary into domain logic. |
| **Evidence Quality** | Environment-fidelity scale (1 Specification → 5 Production); independent of alignment. |
| **HYPOTHESIZED_INTENT** | Status of an onboarding draft intent that has not yet passed Stage 1 review. |

Status vocabulary used across the loop: artifact status is one of `DRAFT` / `APPROVED` /
`IN_PROGRESS` / `COMPLETE` (plus `HYPOTHESIZED_INTENT` for onboarding drafts); reconciliation
status is one of `ALIGNED` / `GAP (…)` / `MISMATCH` / `MISSING`; payload-drift status is one
of `MATCH` / `TYPE_MISMATCH` / `ABSENT` / `EXTRA`.

---

*This manual is a generated documentation artifact. It is not itself a DBA artifact: it does
not pass through the 9-step loop and carries no `status`, `approved_by`, or
`derived_contracts` fields. Regenerate it from the repository when the toolkit changes.*
