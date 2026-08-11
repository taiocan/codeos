# Cohort Logical Design: [cohort_id]

<!--
PURPOSE: Records the approved logical design shared across a declared core architecture cohort's
member features — identity/key strategy, revision/supersession pattern, module interface
boundaries, transaction and event-emission ownership, read-model design, indexing/spatial
principles, migration strategy — synthesized from the whole cohort's approved Intent, Contract,
and Event Schema artifacts, and from the cohort's own approved Architecture Baseline.

This is a SECOND, separate output of Architecture Synthesis, distinct from the Architecture
Baseline (`architecture/core-baseline.md`). It elaborates logical detail one level below the
Baseline's strategic decisions — it does not restate or re-decide the Baseline's own authoritative
decisions (topology, dependency direction, persistence technology, integration style).

This is NOT a behavioral artifact. It may constrain implementation structure at logical detail. It
may NEVER invent or alter behavior. Any behavioral gap discovered while producing this document
returns the affected feature to its owning Stage 1, 2, or 3 — it is not resolved here.

Workflow: .codeos/prompts/03b-architecture-synthesis.md (4-step pipeline, Step 3 drafts this,
Step 4 approves it together with the Baseline)
Decision behavior is owned by the conditional `architecture-entry` doctrine adapter and the
selected architecture policy; this template defines no additional boundary.
See: the architecture_synthesis_policy component selected by .codeos/dba-system.md
-->

## Identity and Version

| Field | Value |
|---|---|
| Cohort id | [cohort_id] |
| Logical design version | [N] |
| Status | draft / approved / superseded |
| Supersedes | [previous version or "none"] |
| Baseline version this design elaborates | [N — must reference the Baseline version approved in the same Step 4, or the current Baseline version if this is a later, standalone revision] |
| Approved by | [human] |
| Approved at | [ISO date] |

<!-- This file (architecture/cohort-logical-design.md) always holds only the CURRENT approved
version. When a new version supersedes this one, this file's content moves to
architecture/history/cohort-logical-design-v<N>.md (named for the version it was current as)
before being overwritten with the new version — see
the architecture_synthesis_policy component selected by .codeos/dba-system.md →
"Verifying a `baseline_version` or
`logical_design_version` reference". -->

**Cohort membership set (this version):**

| Feature ID | Role in cohort |
|---|---|
| [feature_id] | [e.g. canonical entity owner / consumer / projection] |

<!-- Membership is versioned the same way the Baseline's is. A material membership change creates
a new version and requires an impact assessment — see "Cohort, baseline, and logical design
versioning" in the architecture_synthesis_policy component selected by .codeos/dba-system.md. -->

---

## Logical Design Decisions

<!-- Logical structure requiring explicit human sign-off — one level below the Baseline's
strategic decisions. Every entry here must reflect a decision the human actually made, not an
inference. Mark an area "Not applicable to this cohort" explicitly rather than omitting it
silently. -->

**Logical ERD:**
[Entities, relationships, cardinality — narrative or diagram]

**Entity / aggregate ownership:**
[Which feature owns which canonical entity or aggregate]

**Identity and key strategy** (for shared/canonical entities — local per-feature types may still
be decided at Stage 4):
[Decision + rationale]

**Revision / supersession model** (if the Baseline or approved artifacts already establish
append-only/revision-based persistence):
[The shared pattern — e.g. logical_record_id / revision_id / revision_number /
supersedes_revision_id — and its rules: scoping, single-current-revision, branching, rejected/
withdrawn representation, which tables use it]

**Module interface map:**
[What each module boundary exposes and may consume]

**Command / query responsibilities:**
[Operation categories and ownership]

**Transaction boundaries:**
[Which operations must be atomic; which module owns the transaction; whether one command may
write records owned by two feature modules; whether the event log write is in the same
transaction]

**Validation ownership:**
[Which module validates each shared invariant]

**Event-emission rules:**
[When events are emitted relative to validation and transaction commit]

**Read-model design:**
[Ownership, source-of-truth relationship, refresh semantics]

**Indexing and spatial principles** (required access paths and indexing policy — not final index
definitions):
[Decision + rationale; geometry model / SRID policy / spatial index requirement if applicable]

**Migration strategy** (ordering, ownership, compatibility, rollback policy — not concrete
migration scripts):
[Decision + rationale]

**Integration-test obligations** (named boundaries requiring coverage — implementation belongs to
Stage 5):
[List]

---

## Mapping to Source Feature Artifacts

<!-- Every design element above traces to the approved artifact(s) it derives from. This is not a
second canonical model — if a source artifact changes, this mapping (and possibly the design
decision itself) must be revisited. -->

| Design element | Source feature artifact(s) |
|---|---|

---

## Open Architectural Risks

[Named risks and their current status — not dismissed, not silently resolved]

## Revisit Triggers

[Explicit conditions under which this logical design must be revisited — e.g. a new cohort member
proposed, a cross-feature event schema changes, an authoritative decision proves unworkable during
Stage 4]

---

<!-- METADATA -->
status: DRAFT
cohort_id: [cohort_id]
logical_design_version: [N]
type: COHORT_LOGICAL_DESIGN
step_completed: 0
approved_by:
approved_at:
supersedes:
