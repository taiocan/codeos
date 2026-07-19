# Codeos DBA System — Master Doctrine (Downstream Projects)

> **What this file is.** This is the downstream Codeos DBA doctrine. It is loaded by
> downstream projects through their root `CLAUDE.md`, which points to `.codeos/dba-system.md`.
> It is **not** the operating guide for developing the Codeos toolkit repository itself —
> that is the toolkit repo's own root `CLAUDE.md` (the Codeos Self-Development guide).
> The 9-stage substance below is authoritative and unchanged; only its location moved.

## Mode Declaration

You are operating in **Declarative Behavioral Architecture (DBA)** mode, also called **Intent-Driven System (IDS)** mode.

This toolkit is located at `.codeos/` (symlinked from `/home/arc/projects/claude/Codeos`).

Read this file fully at the start of every session before doing anything else.

---

## Truth Authority and Conflict Resolution

When intent, runtime evidence, and structural analysis disagree, resolve as follows:

1. **Explicit human correction** (at any stage gate) overrides all other sources.
2. **Runtime behavior** (observed events) overrides intent text when behavior is more specific. Example: schema declares `"string"`, runtime consistently emits integer — this is empirical evidence of intent-text drift, not a runtime error.
3. **Safety, authorization, and invariant-enforcement logic** always preserves intent primacy regardless of runtime behavior. Example: runtime shows no authorization check was invoked — this is a contract violation, not an authorization redesign.
4. **Structural digest observations** (fan-in, god functions, known risk zones) do not override behavioral findings. They inform blast-radius estimates and remediation sequencing only.
5. **An approved Architecture Baseline** (see "Multi-Feature Architecture Synthesis Gate" below) is authoritative only for project-level structural decisions not fixed by rules 1–4's behavioral artifacts. It never overrides Intent, Contract, Event Schema, explicit human correction, or safety/authorization invariants. Conflicts with runtime evidence are handled through rule 2 above, not a separate baseline-specific rule — runtime behavior does not silently amend the baseline, and the baseline does not silently override runtime-confirmed intent drift either.

When a conflict cannot be resolved by these rules: surface it clearly to the human rather than silently resolving it.

---

## The Non-Negotiable Rules

1. **Every stage transition requires explicit human approval.** You NEVER advance to the next stage without a human "APPROVED", "approved", "yes proceed", or equivalent.
2. **You NEVER implement before intent + contract + event schema are all approved.**
3. **You NEVER add abstractions, patterns, or behaviors beyond what the current intent + contract + event schema specifies.**
4. **You NEVER emit events not listed in the approved event schema.**
5. **You NEVER invent hidden behavior** — all behavior must be traceable to an approved artifact.
6. **After producing any stage output, you STOP and state: `AWAITING HUMAN APPROVAL`.**

---

## Default Advisory Review

Advisory review runs by default at every reviewable gate across the whole workflow below —
not only the numbered Stage 1-9 loop, but also Feature Brief, Onboarding, Solution Discovery
(conditionally — see below), and Architectural Refinement. This mirrors the same default
that Codeos's own toolkit development already holds itself to.

**How to run it.** Before each gate's human-approval decision, run:
```
codeos-reviewer review <feature_id> <stage>
```
using the Stage ID from the table above (e.g. `codeos-reviewer review checkout-flow 2` before
approving Stage 2's contract; `codeos-reviewer review checkout-flow brief` before confirming
a Feature Brief). The reviewer is independent, read-only, and non-gatekeeping — its verdict
(NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE) informs the human's decision but never
auto-blocks. **The human decides at the gate; Non-Negotiable Rule #1 is unchanged.**

**Round budget.** Round 1 runs before the gate. Rounds 2-3 are allowed for fixes or material
deltas raised by the previous round. After 3 rounds, stop and require a human decision rather
than continuing to iterate automatically.

**Solution Discovery is reviewed conditionally, not unconditionally.** The Discovery session
itself stays optional and non-authoritative — running it is never required, and its output is
never an approved architecture. If its output is actually carried into a Feature Brief or a
Stage 1 Intent, *that handoff* gets the default advisory review (or a Review Waiver, below).
A Discovery session whose output nobody acts on is simply never reviewed — there is nothing
yet to review.

**Review Waiver.** If reviewer tooling is unavailable or not configured for this project, the
human records an explicit waiver with a reason and may continue — skipping the review
silently is not allowed, and neither is blocking the whole project over missing reviewer
setup. Record it as a plain entry in that feature's review log: "Review waiver: reviewer not
configured for this project; proceeding without advisory review at `<stage>`. Reason:
`<text>`." **The waiver applies only to the advisory review run. It never waives
Non-Negotiable Rule #1** — a waived review still requires the human to explicitly approve the
stage transition, exactly as if the review had run.

**Verification round-trip.** When a reviewer's assessment names a `HIGHEST-IMPACT
UNCERTAINTY` that is mechanically checkable — a specific file, command, or repository state
that can directly confirm or refute it — the acting agent may run `.codeos/prompts/
verify-only.md`'s read-only verification pass targeting exactly that uncertainty, then attach
its Verification-Only Report as evidence for the next review round. This is optional and
judged by the acting agent, not automatic or mandatory — not every uncertainty is
mechanically checkable, and declining to run it is always a valid choice. Verification
remains strictly read-only (see `verify-only.md`'s No-Edit Rule); it produces evidence for
the human's decision, it does not replace it. A verification pass is not itself a review
round and does not consume the round budget above — only the review round it feeds into does.

**Relationship to the Reviewer Activation Package** (`.codeos/prompts/pipeline-reviewer.md`).
That prompt remains available as an **optional, supplementary** independent critical-assessor
pass — a human can paste it into a separate reviewer session for a second opinion that is
free to challenge the artifact, the feature, or DBA itself, not just check it against
acceptance criteria. It does not replace the default review above.

---

## The 9-Step DBA Development Loop

Every feature follows this exact sequence. No skipping. Run the default advisory review
(see "Default Advisory Review" above) before each gate below.

`[feature_id]` below follows the `F-####` format defined in
`.codeos/templates/conventions.md` → Feature IDs.

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

## Multi-Feature Architecture Synthesis Gate

This is a **conditional, project-level structural approval gate** — not a numbered stage
inserted into the 9-Step loop above, and not required for single-feature or loosely-coupled
projects. It exists for multi-feature projects whose features could materially constrain each
other's implementation architecture.

**When it applies — the core cohort test.** A **core architecture cohort** is two or more
features whose independent implementation choices could *materially constrain* each other's
canonical ownership, dependency direction, persistence boundary, integration contract, shared
infrastructure, or deployment topology. Merely sharing a runtime or a database is evidence to
inspect, not automatic inclusion — a broader test (e.g. "shares runtime/persistence/events")
would pull nearly every feature in a project into one cohort and make this gate universal
rather than conditional, which it must not be.

**Declaring a cohort.** A human declares a core cohort by adding an `architecture_cohorts:`
entry to `features/registry.yaml` (see the template's schema comments) and setting each member
feature's `architecture_cohort` field to that cohort's id. **A feature belongs to at most one
active cohort**; a project may declare multiple cohorts, but their feature memberships must not
overlap. This declaration is where cohort membership and gate status live — the registry
remains an index (membership, status, a baseline version *reference*), never a second home for
the structural decisions the baseline itself owns.

**The rule.** Once a cohort is declared, every member feature reaching Stage 4 requires that
cohort's Architecture Baseline to be `approved` for the version applicable to that feature — see
`.codeos/prompts/04-implement.md`'s cohort eligibility check. Intent, Contract, and Event Schema
approval remain required exactly as before; the baseline is an *additional* requirement for
cohort members, not a replacement for the three approved artifacts.

**The gate sequence.**
1. Every cohort member completes Stage 1 (Intent). An **Intent Cohort Check** —
   duplicate outcomes, missing or circular ownership, inconsistent actor definitions — is
   *recommended* once the cohort's intents are all approved.
2. Every cohort member completes Stage 2 (Contract). A **Contract Cohort Check** — canonical
   artifact ownership, lifecycle/failure consistency, circular preconditions — is *recommended*.
3. Every cohort member completes Stage 3 (Event Schema). An **Event Cohort Check** — event
   ownership, envelope uniformity, correlation strategy, observational-vs-integration
   classification — is *required* as input to synthesis.
4. **Architecture Synthesis** (`.codeos/prompts/03b-architecture-synthesis.md`) produces
   `architecture/core-baseline.md` (template: `.codeos/templates/architecture-baseline.md`).
   Human approval of this baseline is **the single new mandatory gate** this workflow adds — not
   four separate mandatory gates.
5. Only after the baseline is `approved` for the applicable version may cohort members begin
   Stage 4, in whatever dependency order the baseline recommends.

**What the baseline may and may not do.** The baseline may constrain implementation structure —
crate/workspace topology, dependency direction, shared-infrastructure decisions, integration
style. It may **never** invent or alter behavior. Any behavioral gap discovered during synthesis
returns the affected feature to its owning Stage 1, 2, or 3 — it is never patched inside the
baseline directly.

**Authoritative decisions vs. derived views.** The baseline distinguishes decisions a human
manually approved (authoritative) from matrices and inventories mechanically derived from
already-approved artifacts (ownership matrix, dependency graph, event producer/consumer matrix —
regenerable, each carrying provenance to its source artifact). The derived views are never a
second canonical model that can silently drift from what they were built from.

**Cohort and baseline versioning.** `architecture_cohorts[].baseline_version` is a single,
**cohort-level** field — one current value, not tracked per member feature. A baseline approves a
specific *versioned cohort membership set*. Adding or removing a feature does not silently
rewrite the cohort or its approved baseline. A material membership change creates a new
cohort/baseline version and **requires an impact assessment**. Prior Stage 4 work already
approved under the earlier version is **not invalidated merely by the membership change itself**
— it must be reconciled only when that assessment identifies an **actual structural conflict**,
and reconciliation happens through Stage 9/10 for the specific affected feature, not by
re-running this gate. Approved baseline versions are never silently rewritten; a replacement
supersedes and is recorded — the superseded file moves to
`architecture/history/core-baseline-v<version>.md` *before* the new version is written to
`architecture/core-baseline.md`, so the two are never both "current-looking" at once; the registry
entry's `baseline_version` is updated to the new version as part of that same approval (see
`.codeos/prompts/03b-architecture-synthesis.md`'s Step 3). Historical files are a **provenance
record only** — they document which version governed a feature's already-completed Stage 4 work;
they are never consulted for a *new* Stage 4 eligibility decision.

**Verifying a `baseline_version` reference (live Stage 4 eligibility).** A cohort's
`baseline_version` in `features/registry.yaml` is valid, for the purpose of *entering or
re-entering Stage 4 right now*, only if it equals `architecture/core-baseline.md`'s **current**
`Baseline version` field exactly. A value that instead matches an
`architecture/history/core-baseline-v<version>.md` file is **stale, not valid** — it blocks Stage
4 exactly as an unapproved status would (see `.codeos/prompts/04-implement.md`'s cohort
eligibility check), until either the registry is updated to the current version (normally
automatic, as part of approving that version) or a human resolves the discrepancy. This is
deliberately stricter than "any version this feature was ever pinned to": the live check only
ever accepts the current version; historical files matter for audit and for the non-retroactive
protection above, not for gating new work.

**Reviewer coverage.** `codeos-reviewer` has no dedicated checklist for an `architecture-synthesis`
stage id today (it falls through to the tool's generic, untailored fallback). Until native
support exists, use the **Review Waiver** mechanism described under "Default Advisory Review"
above for gate reviews at this stage, recording the specific reason ("`architecture-synthesis`
stage id not yet supported by `codeos-reviewer`"). The waiver does not weaken Non-Negotiable Rule
#1 — the human still explicitly approves the baseline.

**Naming.** This is the **Architecture Synthesis Gate**, producing the **Core Architecture
Baseline** — deliberately not "Architecture Discovery." Solution Discovery
(`.codeos/prompts/00a-solution-discovery.md`) is optional, non-gating, and pre-Stage-1; its
output is never approved architecture. This gate is the opposite: conditional but, once
triggered, mandatory, and it runs only after Stage 3 approval across the whole cohort, consuming
approved artifacts rather than producing speculative ones.

---

## What You Do at Each Stage

Use the corresponding prompt file from `.codeos/prompts/` for detailed instructions. The
**Stage ID** column is the identifier vocabulary used both for documentation ordering and as
the `<stage>` argument to `codeos-reviewer review <feature_id> <stage>` — see "Default
Advisory Review" below.

| Stage | Stage ID | File |
|---|---|---|
| Session start | — | `.codeos/prompts/00-session-start.md` |
| Session end (handoff) | — | `.codeos/prompts/00-session-end.md` |
| Solution Discovery (Session Type E, pre-Feature-Brief) | `discovery` | `.codeos/prompts/00a-solution-discovery.md` |
| Feature Brief (pre-Stage 1) | `brief` | `.codeos/prompts/00b-feature-brief.md` |
| Existing Codebase Onboarding (Session Type D) | `onboarding` | `.codeos/prompts/00c-onboarding.md` |
| Stage 1: Intent | `1` | `.codeos/prompts/01-intent.md` |
| Stage 2: Contracts | `2` | `.codeos/prompts/02-contract.md` |
| Stage 3: Event Schema | `3` | `.codeos/prompts/03-event-schema.md` |
| **Architecture Synthesis Gate** (conditional, cohort-level) | `architecture-synthesis` | `.codeos/prompts/03b-architecture-synthesis.md` |
| Stage 4: Implementation | `4` | `.codeos/prompts/04-implement.md` |
| Stage 5: Tests | `5` | `.codeos/prompts/05-tests.md` |
| Stage 6: Observation | `6` | `.codeos/prompts/06-observe.md` |
| Stage 7: Reconcile | `7` | `.codeos/prompts/07-reconcile.md` |
| Stage 8: Replay | `8` | `.codeos/prompts/08-replay.md` |
| Stage 9: Refine | `9` | `.codeos/prompts/09-refine.md` |
| **Architectural Refinement** (alternate loop) | `10` | `.codeos/prompts/10-arch-refine.md` |
| Reviewer Activation Package (optional second opinion) | — | `.codeos/prompts/pipeline-reviewer.md` |

**On `onboarding`'s position in this list**: it is not a step every feature passes through
after `brief`. It is an **alternate entry point**, used *instead of* `discovery`/`brief` only
when bootstrapping an existing codebase that lacks DBA artifacts. The Stage ID sequence
(`discovery, brief, onboarding, 1, 2, ..., 10`) is identifier vocabulary and documentation
order — not a claim that every feature is a single linear path through all of it.

**On `architecture-synthesis`'s position in this list**: it is not a per-feature step every
feature passes through between Stage 3 and Stage 4. It is a **project-level gate** that applies
only when a human has declared a core architecture cohort (see "Multi-Feature Architecture
Synthesis Gate" above) — most features, and most projects, never trigger it. When it does apply,
every member of the declared cohort must reach Stage 3 before this gate runs, and the gate's
approval is required before *any* cohort member begins Stage 4.

The Architectural Refinement workflow is a 5-step alternative loop (Scope → Impact → Implement → Verify → Reconcile) for structural changes that have no behavioral contract or event schema. Use it for workspace restructuring, shared library extraction, dependency consolidation, test infrastructure, and naming normalization. Use the 9-step loop for any change that would alter a contract or schema. A structural-only correction to an already-approved Architecture Baseline (see "Multi-Feature Architecture Synthesis Gate" above) is Stage-10-eligible only when it does not change any feature's behavior — a change that would is not a refinement of the baseline, it returns the affected feature to its earlier stage instead.

Use the corresponding template from `.codeos/templates/` when producing artifacts:

| Artifact | Template |
|---|---|
| Feature brief | `.codeos/templates/feature-brief.md` |
| Feature intent | `.codeos/templates/intent.md` |
| Behavioral contract | `.codeos/templates/contract.md` |
| Event schema | `.codeos/templates/event-schema.md` |
| Feature specification | `.codeos/templates/feature-spec.md` |
| Refinement log | `.codeos/templates/refinement.md` |
| Architectural refinement | `.codeos/templates/arch-refinement.md` |
| Architecture Baseline (cohort-level, conditional) | `.codeos/templates/architecture-baseline.md` |
| Codebase digest | `.codeos/templates/codebase-digest.md` |
| Session handoff | `.codeos/templates/handoff.md` |
| Review Package | `.codeos/templates/review-package.md` |
| Per-feature review file | `.codeos/templates/review-file.md` |

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

## Artifact Classification

Not all artifacts are required. This table tells you which artifacts block stage
advancement and which improve decision quality without being prerequisites.

**Required artifacts block stage advancement. Optional and recommended artifacts
improve decision quality but are never prerequisites for stage transitions.**

`[id]` below is shorthand for `[feature_id]`, which follows the `F-####` format defined
in `.codeos/templates/conventions.md` → Feature IDs.

| Artifact | Classification | When it exists |
|---|---|---|
| Feature Brief (`backlog/[id].md`) | Optional | Pre-Stage-1 discovery; not required to start Stage 1 |
| Intent (`intents/[id].md`) | **Required** | Any behavioral work — must be APPROVED before Stage 2 |
| Contract (`contracts/[id]_contract.md`) | **Required** | Any behavioral work — must be APPROVED before Stage 3 |
| Event Schema (`events/[id]_schema.md`) | **Required** | Any behavioral work — must be APPROVED before Stage 4 |
| Feature Registry (`features/registry.yaml`) | Recommended | Multi-feature projects; not required for single-feature work |
| Codebase Digest (`docs/codebase-digest.md`) | Optional | Existing codebases and mature projects; absent digest is never a blocker |
| Structural Alignment (Stage 7 output section) | Optional output | Produced at Stage 7 only when architectural observations exist |
| Architectural Refinement (`refinements/arch/[id].md`) | Optional | Non-behavioral structural changes; uses the Stage 10 workflow |
| Architecture Baseline (`architecture/core-baseline.md`) | **Required for cohort members' Stage 4** | Only when a core architecture cohort is declared (see "Multi-Feature Architecture Synthesis Gate"); not applicable to single-feature or non-cohort projects |
| Onboarding artifacts (`HYPOTHESIZED_INTENT`) | Onboarding only | Produced by Session Type D; must pass Stage 1 review before advancing |

---

## File Layout

`[feature_id]` below follows the `F-####` format defined in
`.codeos/templates/conventions.md` → Feature IDs.

```
project/
├── .codeos/                      ← this toolkit (symlink)
├── CLAUDE.md                     ← project-level instructions (references this file)
├── features/
│   └── registry.yaml             ← authoritative feature status index (human-maintained)
├── architecture/                 ← only when a core architecture cohort is declared
│   ├── core-baseline.md          ← current approved Architecture Baseline (current version only)
│   └── history/
│       └── core-baseline-v[N].md ← superseded baseline versions, named for their own version
├── intents/
│   └── [feature_id].md           ← one per feature
├── contracts/
│   └── [feature_id]_contract.md  ← one per feature
├── events/
│   ├── [feature_id]_schema.md    ← event schema per feature (or shared event_schema.md)
│   └── runtime_events.jsonl      ← append-only runtime log
├── backlog/
│   └── [feature_id].md           ← feature briefs (pre-Stage-1 discovery)
├── handoffs/
│   └── [YYYY-MM-DD]-[desc].md    ← session handoffs (optional, not DBA artifacts)
├── reviews/
│   ├── [feature_id].md           ← per-feature: Decision Log + Decision Rationale (traceability)
│   └── architecture-journal.md   ← cross-feature institutional memory (AJ-NNN entries)
├── refinements/
│   └── arch/
│       └── [refine_id].md        ← architectural refinement records
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
| **Shared Infrastructure Module** | A module depended on by ≥2 feature modules that provides only mechanical infrastructure (event emission, DTOs, constants, re-exports). Never contains domain logic. See `patterns/shared-infrastructure-boundary.md`. |
| **Vertical Drift** | Accumulation of domain logic in a shared infrastructure module. Bypasses lateral isolation guarantees even when feature→feature imports are blocked by workspace topology. |

---

## Human Navigation

Intent files are precision artifacts optimized for contract derivation, not for fast reading.
When you need a quick plain-language explanation of what a feature does:

**Ask Claude directly:**
> "Explain [feature_id] in plain English."

Claude will read `intents/[feature_id].md` and produce a jargon-free explanation on demand.
No file is saved. No approval gate. No DBA lifecycle.

This is the preferred pattern. It solves the readability gap without creating a second
intent surface or introducing drift between stored summaries and authoritative intents.

**If a stored summary is needed** (e.g., for onboarding documentation or a project README):
Generate it on request, include provenance metadata (see below), and treat it as
generated output — never manually edit, regenerate from intent when the intent changes.

Provenance metadata for any stored generated summary:
```yaml
generated_from_intent: intents/[feature_id].md
generated_at: [ISO date]
generated_by: [Claude session / human]
```

Stored generated summaries are not DBA artifacts. They do not feed into any stage.
They do not carry `status`, `approved_by`, or `derived_contracts` fields.

---

## Review Logging

When the human provides a reviewer's assessment and their decision on it, before writing
any review artifacts, Claude shows a brief preview of what it will write (5 lines inline).
Then Claude writes — **do this before proceeding to any other work:**

1. **One row** to `reviews/[feature_id].md` Decision Log.
2. **A Decision Rationale section** to `reviews/[feature_id].md` — only when the decision
   would be difficult to reconstruct from artifact history alone: a reframing, an
   architectural pivot, a rejected direction. Not for wording revisions, contract
   clarifications, or any change the diff already explains. Most stages do not get a section.
3. **One entry** to `reviews/architecture-journal.md` — only if the insight is likely to
   remain useful six months from now to a reader who has forgotten this feature entirely.
   When uncertain, journal only if future usefulness is clear. Missing an important finding
   is more expensive than adding a few extra entries.

**Human overrides (override automatic classification):**
- "do not log this review" — suppress all review artifacts for this cycle
- "journal this" — force a journal entry regardless of criteria
- "do not journal this" — suppress only the journal entry

**Log fidelity:** Preserve the reviewer's core insight as close to verbatim as the format
allows. Compress explanation and context — never compress the insight itself.

**Log quality:** Record conclusions and rationale, not conversation history. Capture what
was learned, not what happened. Review artifacts must never become meeting minutes.

**Architecture Journal:** The journal is the long-term institutional knowledge artifact;
per-feature review files are primarily traceability artifacts. When an architectural
finding belongs equally in both, put it in the journal and keep the feature file entry
brief with a reference (e.g., `See AJ-014`). Journal entries must remain useful to a
future reader who has forgotten the feature entirely — if understanding requires feature
context, the entry belongs in `reviews/[feature_id].md` instead.

**Decision Log rows are append-only.** Original findings and decisions are never
rewritten. Superseded decisions are addressed by adding a new row. Decision Rationale
sections may gain cross-references, supersession references, or clarification notes —
but never rewrites of original findings or decisions.

Architecture Journal entry format (see `templates/review-file.md` for per-feature format):
```
## AJ-NNN — [topic]
Date: YYYY-MM-DD
Status: Active | Superseded | Rejected

Context: [what triggered this — feature, stage, or discussion]
Finding: [the key insight or reframing]
Decision: [accepted / rejected / deferred and why]
Action: [what changed — pattern created, architecture revised, etc.]
Supersedes: [AJ-NNN or "none"]
Related: [AJ-NNN, AJ-NNN or "none"]
```

---

## How to Use the Toolkit in a New Project

1. Run from the new project root: `bash /home/arc/projects/claude/Codeos/scripts/dba-init.sh`
2. This creates `.codeos` symlink, all required directories, and a project `CLAUDE.md`
3. Start Claude Code in the project directory
4. Claude reads the project `CLAUDE.md` which directs it to read this file
5. Human pastes `.codeos/prompts/00-session-start.md` to begin a session
