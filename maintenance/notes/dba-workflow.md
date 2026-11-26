# 0. Overall Codeos lifecycle

The whole new Codeos model can therefore be understood with one diagram:

┌───────────────────────────────────────┐
│ 1. SOLUTION BOOTSTRAP                 │
│                                       │
│ Charter → Architecture → Config       │
│ → integrated Postgres/Rust/Svelte     │
│ → baseline verification → preview     │
└───────────────────┬───────────────────┘
                    ↓
┌───────────────────────────────────────┐
│ 2. FEATURE DEVELOPMENT                │
│                                       │
│ Specify → design → vertical slice     │
│ → early preview → full V&V            │
│ → reconciliation → acceptance         │
└───────────────────┬───────────────────┘
                    ↓
┌───────────────────────────────────────┐
│ 3. OPERATION & LEARNING               │
│                                       │
│ use → observe → classify              │
│ → repair / new feature / revision     │
└───────────────────┬───────────────────┘
                    │
                    ├──── new feature ────────────┐
                    │                             │
                    └──── feature revision ──────→│
                                                  ↓
                                      FEATURE DEVELOPMENT

And occasionally:

Operation & Learning
        ↓
solution-level change
        ↓
Charter / Architecture revision
        ↓
Feature Development


# 1. Solution Bootsrap workflow

Product / solution idea
      ↓
Solution discovery
  - problem
  - target user
  - primary decision supported
  - desired outcomes
  - important boundaries / exclusions
      ↓
Terminology seed
  - define semantics-critical project terms
  - avoid creating a large glossary upfront
      ↓
Solution Charter
  - Problem
  - Vision
  - Decision Supported
  - Outcomes
  - Scope
  - Platform Baseline exceptions, if any
  - product-level validation expectations
      ↓
Human approval of Charter
      ↓
Solution Architecture
  - PostgreSQL persistence boundary
  - Rust/Axum backend boundary
  - SvelteKit browser boundary
  - Docker execution topology
  - external systems
  - major module boundaries
  - integrated verification boundaries
      ↓
Consequential architecture approval
      ↓
Project configuration
  - Platform Baseline
  - fixed Codeos mechanics
  - artifact governance choices
      ↓
Integrated reference skeleton
  - PostgreSQL starts
  - SQLx migrations run
  - Axum backend starts
  - SvelteKit app starts
  - DB ↔ backend ↔ GUI connected
      ↓
Baseline verification
  - clean bootstrap works
  - migration from zero works
  - integration smoke passes
  - minimal Rust tests pass
  - minimal browser/component tests pass
  - minimal Playwright journey passes
      ↓
INITIAL USER PRODUCT PREVIEW
  - user sees the running empty/minimal product shell
  - confirms overall product direction
      ↓
Ready for Feature Development

---
What this workflow should deliberately NOT do

Do not design the future product in detail.

Avoid:

building feature-specific database schemas;
designing every future module;
creating complete navigation;
defining a large component library;
speculative infrastructure;
Redis/queues/auth/search/etc. without current need;
building the final GUI.

The bootstrap should answer only:

Can Codeos now deliver the first real feature through the complete solution?

A useful bootstrap invariant

I would make this explicit:

Feature development may start only when the integrated product baseline is runnable and its end-to-end verification path has been demonstrated once.

That prevents the old failure mode where backend development begins while the GUI remains hypothetical.


# 2. Feature Development Loop

```text
Feature selected
      ↓
Intent + Contract + Event Schema
      ↓
Terminology check/update
      ↓
Design
  - define persisted-data invariants where applicable
  - define migration approach where schema changes
  - define transaction boundary for atomic multi-step writes
  - define how behavior, repeatability, data integrity,
    browser behavior, and preview will be proven
      ↓
DB/backend/Svelte vertical implementation
  - PostgreSQL constraints enforce durable invariants where practical
  - schema changes use versioned SQLx migrations
  - SQL receiving external values is parameterized
  - atomic multi-step writes use explicit transactions
      ↓
Integration smoke
  - Postgres ↔ Rust ↔ Svelte actually runs
      ↓
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
EARLY USER GUI PREVIEW
direction / UX only — not acceptance
verification is incomplete
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
      ↓
direction wrong?
      ├─ changes approved behavior
      │      → revise Specification Package
      │      → human re-approval
      │      → implement again
      │
      └─ implementation / UX only
             → refine implementation
             → repeat preview
      ↓
direction accepted
      ↓
Behavior tests
  - prove approved Contract behavior
  - use real PostgreSQL where database behavior matters
      ↓
Data-integrity tests
  [required when the feature persists or changes data]
  - valid required state can be stored
  - relevant invalid state is rejected
  - migrations succeed from a clean database
  - transaction rollback preserves atomicity where applicable
      ↓
Repeatability tests
  - replay, idempotency, stable repeated results,
    or another feature-appropriate repeatability property
      ↓
Component / browser tests
      ↓
Playwright integrated journeys
  - real PostgreSQL
  - real Rust backend
  - real Svelte application
  - critical user-visible outcome demonstrated end to end
      ↓
Conditional validation mechanics
  - performance
  - security
  - accessibility
  - concurrency
  - migration compatibility / data migration
  - other mechanics justified by the feature
      ↓
Reconciliation
      ↓
Reviewer
      ↓
FINAL USER GUI VALIDATION
      ↓
Acceptance
```

# 3. operational workflow

Feature accepted
      ↓
Release / integrated product baseline updated
      ↓
Operational use
  - user uses the feature
  - system produces runtime evidence
  - real data accumulates
      ↓
Observe
  - defects
  - unexpected behavior
  - UX friction
  - performance problems
  - data-quality problems
  - changed external conditions
  - new user needs
      ↓
Classify the observation
      │
      ├── implementation defect
      │      ↓
      │   existing approved behavior still correct
      │      ↓
      │   targeted repair
      │      ↓
      │   proportional verification
      │      ↓
      │   restore accepted behavior
      │
      ├── UX refinement within approved behavior
      │      ↓
      │   targeted implementation refinement
      │      ↓
      │   browser / human validation
      │
      ├── new product capability
      │      ↓
      │   create/select next Feature
      │      ↓
      │   Feature Development workflow
      │
      ├── approved feature meaning must change
      │      ↓
      │   revise Specification Package
      │      ↓
      │   human approval
      │      ↓
      │   Feature Development workflow
      │
      ├── solution purpose / outcome / scope changed
      │      ↓
      │   revise Solution Charter
      │      ↓
      │   impact existing features
      │      ↓
      │   human approval
      │
      └── consequential architecture change
             ↓
          architecture reassessment
             ↓
          human approval
             ↓
          affected feature work
      ↓
Terminology update when meaning changed
      ↓
Continue operation


## Database reliability rules carried by the loop

These are implementation and verification rules, not additional lifecycle stages:

* Persisted invariants use PostgreSQL constraints where PostgreSQL can express them cleanly.
* Schema changes use versioned SQLx migrations.
* External values are passed to SQL through parameters, not string concatenation.
* Database-sensitive behavior is tested against real PostgreSQL rather than repository mocks alone.
* Atomic multi-step state changes use explicit transaction boundaries.
* Repeatability and concurrency properties are tested when the feature depends on them.
* When source identity affects trust or interpretation, canonical persisted data retains a stable link to its source evidence.

4. The most important post-feature routing rule

Keep the decision very simple:

Observation
    ↓
Does approved meaning change?

If no:

repair / refinement
→ proportional verification

If yes at feature level:

Specification Package revision
→ approval
→ feature loop

If yes at solution level:

Charter / Architecture revision
→ approval
→ affected feature work

This avoids creating a large issue taxonomy.

5. Add one product-baseline concept after acceptance

After each accepted feature, the integrated application should become the new accepted product baseline:

Baseline N
Postgres + Rust + Svelte
        ↓
Feature F-0012 accepted
        ↓
Baseline N+1

That gives the next feature a known starting point.

It also means every feature is added to a working product, rather than accumulating as isolated backend modules.

You do not necessarily need a new artifact for this. Git commit/release state plus existing accepted artifacts can identify the baseline.

6. Post-acceptance regression should be selective, not “rerun everything forever”

One risk is making the post-feature workflow too expensive.

Before accepting a feature:

new feature tests
+
affected existing feature tests
+
critical integrated journeys

Do not automatically rerun every expensive test for every feature unless cheap enough.

Use impact:

DB schema changed
→ affected persistence/data-integrity tests

shared backend behavior changed
→ affected behavioral/repeatability tests

shared GUI/navigation changed
→ affected Playwright journeys

But keep a small product critical-path suite that always runs.

For example:

application starts
user reaches main workflow
critical DB→backend→GUI journey works

This gives high protection at low cost.

7. Where performance, security and accessibility fit

Do not make separate workflows.

They are mechanics attached to either:

Feature development

when a feature introduces relevant risk:

new query at scale
→ performance validation

authentication/authorization
→ security validation

new user interaction
→ accessibility validation

or:

Operation

when real evidence exposes the need:

production latency deteriorates
→ performance investigation

accessibility issue reported
→ refinement / feature change

This preserves modularity.


9. I would make these three workflows explicit in Codeos

Not as three new lifecycle stages.

They are three scopes of work:

Workflow	Frequency	Main human question
Solution Bootstrap	Usually once	Are we building the right product foundation?
Feature Development	Repeated	Is this the right capability and does it work?
Operation & Learning	Continuous	What have we learned and what should change next?

The existing DBA stages continue to implement the detailed behavior inside these scopes.

This is important because it gives a user a much easier mental model than thinking primarily in terms of nine engineering stages.

The key result

The complete Codeos development philosophy then becomes:

Start with a runnable integrated product, grow it through small visible and fully validated vertical features, and let real operational evidence drive the next feature, repair, or approved solution change.

That is much closer to the product-development behavior you originally wanted Codeos to enforce.


------

**Yes — I think making all three workflows mechanically enforced could make Codeos materially stronger, provided you mechanize the process, not the judgment.** The best form is a **small governed workflow contract + one generic runner + evidence-derived checklist** for each workflow.

The crucial distinction is:

> **Codeos mechanically controls whether a required step was performed and whether its required evidence exists. Humans/agents still decide semantic questions such as “is this the right product direction?” or “does this evidence actually satisfy the intent?”**

If you cross that boundary and make the tool infer semantic adequacy, complexity and false confidence will rise quickly.

# 1. I would make the three workflows first-class governed contracts

Codeos would have:

```text
Solution Bootstrap Workflow
Feature Development Workflow
Operation & Learning Workflow
```

Each workflow contract defines only:

| Element           | Meaning                                      |
| ----------------- | -------------------------------------------- |
| Entry             | What must already be true                    |
| Step              | What must happen                             |
| Required evidence | What proves it happened                      |
| Human decision    | Only where semantic judgment is necessary    |
| Exit              | Deterministic conditions required to proceed |
| Failure route     | Where to go if the condition fails           |

The **workflow definition itself is governed** and versioned with DBA.

But individual workflow runs should not create another large hand-maintained artifact.

---

# 2. Do not use manual checkboxes

This is important.

Avoid:

```text
[x] Contract approved
[x] Playwright run
[x] User preview completed
[x] Reviewer passed
```

because an agent can tick a box without the underlying truth being present.

Instead:

```text
Contract approved
→ derived from Contract approval metadata

Playwright passed
→ derived from test evidence

Reviewer passed
→ derived from reviewer record

User preview completed
→ derived from recorded human decision

Migration passed
→ derived from migration/test evidence
```

So the checklist is **generated from evidence**.

Conceptually:

```text
Workflow Contract
       +
Current project evidence
       ↓
Workflow engine
       ↓
STATUS

✓ Specification Package approved
✓ Terminology checked
✓ Design available
✓ Vertical implementation runnable
✓ Integration smoke passed
✓ Early preview completed
✗ Behavioral validation missing
○ Playwright waiting
○ Final UX waiting
```

That gives you the strictness you want without maintaining another source of truth.

# 3. One generic workflow mechanism, not three engines

I would avoid separate scripts for bootstrap, feature and operation.

One small interface is enough:

```text
codeos workflow status
codeos workflow check
codeos workflow next
```

Possibly:

```text
codeos workflow status F-0012
```

The workflow definition determines what is applicable.

`status` explains where you are.

`check` evaluates the current checkpoint.

`next` tells the agent/human what can happen next.

The existing prompts/tools still do the actual work.

Codeos does **not** need a generic BPM/workflow platform.

---

# 4. Solution Bootstrap can become highly mechanical

I would make its governed checklist approximately:

| Checkpoint         | Mechanical requirement                                  | Semantic decision                         |
| ------------------ | ------------------------------------------------------- | ----------------------------------------- |
| B1 Purpose         | Charter exists, terminology seed exists                 | Human approves Charter                    |
| B2 Architecture    | Architecture + `codeos.yaml` valid                      | Human approves consequential architecture |
| B3 Skeleton        | Postgres/Rust/Svelte/Docker generated                   | None                                      |
| B4 Baseline        | migration + smoke + backend + browser + Playwright pass | None                                      |
| B5 Product preview | running baseline exposed                                | Human confirms direction                  |
| B6 Ready           | all prior evidence present                              | Human authorizes feature development      |

The important exit invariant:

> **Feature development cannot begin until Codeos has demonstrated a runnable end-to-end product baseline.**

This is exactly the kind of property worth enforcing mechanically.

# 5. Feature Development should be the strongest mechanical workflow

Your feature workflow is particularly well suited to this.

I would structure its checkpoints like this:

| Checkpoint                 | Required evidence                                                                                             |
| -------------------------- | ------------------------------------------------------------------------------------------------------------- |
| F1 Specification           | approved Intent + Contract + Event Schema                                                                     |
| F2 Preparation             | terminology consistent + validation questions answered + Design                                               |
| F3 Vertical implementation | tier-impact accounting + runnable implementation                                                              |
| F4 Direction               | integration smoke + Early Development Preview human decision                                                  |
| F5 Verification            | behavior + data integrity + repeatability + component/browser + Playwright + applicable conditional mechanics |
| F6 Reconciliation          | evidence-backed reconciliation complete                                                                       |
| F7 Independent review      | required reviewer result                                                                                      |
| F8 Final UX                | human validation of completed feature                                                                         |
| F9 Acceptance              | all required evidence present + human acceptance                                                              |

Notice what this does **not** do.

It does not introduce nine new artifacts.

Most checklist items are predicates over artifacts/evidence that already exist.

---

# 6. Preserve the preview branch mechanically

The early preview is an excellent candidate for strict routing.

```text
F4 Early Preview
       ↓
human decision
       │
       ├── RIGHT DIRECTION
       │       ↓
       │    verification
       │
       └── CHANGE NEEDED
               ↓
       Does feedback change approved behavior?
               │
          ┌────┴────┐
          YES       NO
           ↓         ↓
     Specification   Implementation/
     revision        UX refinement
           ↓         ↓
     human approval  repeat preview
```

Codeos should **not allow F5 verification to begin** until the preview route has resolved.

That is a mechanical rule with very high practical value.

# 7. Database reliability fits naturally into the feature checklist

Your new database practices should not become another workflow.

They become predicates inside F2/F3/F5.

For a persistence-affecting feature:

```text
F2
Define persisted invariants,
migration and transaction needs

F3
SQLx migration exists
constraints implemented
queries parameterized
transactions established

F5
real Postgres tests
data-integrity tests
migration test
repeatability/concurrency where applicable
```

And applicability is derived:

```text
feature touches persistence?
    ├── YES → data-integrity mechanic required
    └── NO  → not applicable
```

The project cannot simply switch it off.

# 8. Operation & Learning can also be mechanical—but differently

This workflow is inherently less linear.

Codeos should mechanically require every significant operational observation to be **resolved into a route**, but it should not automatically determine the route.

For example:

| Checkpoint | Requirement                          |
| ---------- | ------------------------------------ |
| O1 Observe | concrete operational evidence exists |
| O2 Assess  | observation has been interpreted     |
| O3 Route   | exactly one valid route selected     |
| O4 Execute | routed workflow/change completed     |
| O5 Verify  | resulting change demonstrated        |
| O6 Close   | observation linked to outcome        |

The semantic decision at O3 is something like:

```text
implementation defect
UX refinement
new feature
Specification change
Charter change
Architecture reassessment
no action + rationale
```

Once selected, Codeos mechanically routes it.

For example:

```text
NEW FEATURE
    ↓
Feature Workflow F1

SPECIFICATION CHANGE
    ↓
Specification revision
    ↓
Feature Workflow

SOLUTION PURPOSE CHANGE
    ↓
Charter revision
    ↓
impact assessment

IMPLEMENTATION DEFECT
    ↓
repair
    ↓
proportional verification
```

This is a strong use of mechanics: **the system ensures nothing disappears into an informal note**, without pretending it can decide what the observation means.

# 9. Govern the process definitions, not all process data

I would make this distinction very explicit.

### Governed

```text
Bootstrap Workflow Contract
Feature Workflow Contract
Operation Workflow Contract
Codeos Mechanics definitions
routing rules
mandatory evidence rules
human-control boundaries
```

### Derived / operational

```text
current workflow step
PASS / FAIL / WAITING
which checks currently pass
what the next allowed action is
temporary execution state
```

Do not create approval ceremonies around the generated checklist itself.

That would add no value.

# 10. A tiny durable workflow receipt may be justified

There is one small missing piece.

Some steps—particularly:

```text
Early Development Preview
Final Human UX Validation
```

have no deterministic file/test result proving the human interaction happened.

For these I would allow a **minimal human decision receipt**, not another narrative artifact.

For example:

```yaml
feature: F-0012
decision: early_preview
result: direction_confirmed
at: 2026-09-05T10:42:00+02:00
```

or for a correction:

```yaml
feature: F-0012
decision: early_preview
result: behavior_revision_required
```

No rationale unless materially useful.

No essay.

No new approval package.

The workflow engine can then derive progression from this receipt.

Whether this can reuse an existing Codeos decision/event mechanism should be checked before creating a new format.

# 11. Efficiency depends on a strict rule: checkpoints must be cheap

This is where mechanical systems usually go wrong.

I would impose a design constraint:

> **A workflow checkpoint should evaluate existing evidence; it should rarely generate new evidence solely for the checkpoint.**

Bad:

```text
Create checklist document
→ write completion report
→ write evidence mapping
→ approve checklist
→ update status file
```

Good:

```text
Contract approval exists?
migration passed?
Playwright evidence exists?
reviewer passed?
human decision exists?
        ↓
derive status
```

The second is much cheaper.

# 12. Do not make the mechanical sequence unnecessarily rigid inside a checkpoint

Another important distinction:

```text
checkpoint ordering = strict
work within checkpoint = flexible
```

For example, during implementation, Claude may move between:

```text
Rust
SQL
Svelte
Rust
Svelte
SQL
```

however it wants.

Codeos only cares that before leaving the vertical implementation checkpoint:

```text
all affected tiers are integrated
+
smoke passes
+
preview is reachable
```

This preserves implementation freedom.

# 13. The workflow engine should fail with actionable information

Avoid:

```text
F5 CHECK FAILED
```

Use:

```text
F5 — Full Verification: BLOCKED

PASS
✓ behavior: 18 tests
✓ data integrity: 6 tests
✓ repeatability: replay PASS
✓ component/browser: 9 tests

MISSING
✗ Playwright integrated journey
  Required because Contract contains GUI-visible behavior.

NEXT
Run the approved Playwright journey for F-0012.
```

This turns governance into assistance rather than bureaucracy.

# 14. Future mechanics become easy to add

This is one of the strongest arguments for your idea.

Suppose later you decide accessibility is fundamental.

Today:

```text
F5 Verification
├── behavior
├── data integrity
├── repeatability
├── component/browser
└── Playwright
```

Later:

```text
F5 Verification
├── behavior
├── data integrity
├── repeatability
├── component/browser
├── Playwright
└── accessibility
```

You change:

```text
Codeos Mechanics policy
+
Feature Workflow Contract
+
check implementation
```

You **do not redesign the lifecycle**.

The same applies to:

* security;
* migration compatibility;
* recovery;
* resilience;
* performance;
* accessibility.

That is the modularity you were looking for.

# 15. The main adversarial risks

I would watch only five failure modes:

| Risk                                                      | Protection                                                |
| --------------------------------------------------------- | --------------------------------------------------------- |
| Checklist becomes huge                                    | 5–9 checkpoints per workflow maximum                      |
| Semantic judgment gets automated                          | mechanical checks verify evidence existence/state only    |
| New duplicate state appears                               | checklist derived from canonical evidence                 |
| Every feature runs every expensive test                   | applicability + affected-scope rules                      |
| Agents optimize for passing checklist rather than product | early and final human product validation remain mandatory |

If those boundaries hold, mechanical workflows can actually **reduce** cognitive load.

# Recommended architecture

I would now evolve DBA-5 toward:

```text
                   CODEOS
                      │
        ┌─────────────┼─────────────┐
        ↓             ↓             ↓
    BOOTSTRAP      FEATURE       OPERATION
    WORKFLOW       WORKFLOW      WORKFLOW
        │             │             │
        └─────────────┼─────────────┘
                      ↓
              CODEOS MECHANICS
                      ↓
     artifacts + tests + evidence + decisions
                      ↓
              WORKFLOW CHECKER
                      ↓
           PASS / BLOCKED / NEXT
```

The underlying DBA artifacts and nine-stage discipline remain.

The workflows provide the **mechanical outer control structure**.

---

## My recommendation

I would pursue this.

In fact, for the new Codeos direction I think it may be stronger than relying mainly on prompts to tell an agent what sequence to follow.

But I would define **“fully governed”** precisely as:

> **The workflow definition, required checkpoints, routing rules and evidence requirements are governed. Progression is mechanically enforced. Semantic judgments remain explicitly human/reviewer-owned and are recorded as decisions rather than inferred by tooling.**

That combination gives you strictness **without turning Codeos into a bureaucratic workflow engine**.

And I would build it incrementally: first mechanize the **Feature Workflow**, because it repeats most often and has the clearest evidence predicates. Once that proves low-friction in the DBA-5 pilot, apply the same small engine to Bootstrap and Operation rather than designing all three engines upfront.
