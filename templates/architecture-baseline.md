# Core Architecture Baseline: [cohort_id]

<!--
PURPOSE: Records the approved project-level structural decisions for a declared core
architecture cohort — Rust workspace/crate topology, dependency direction, shared
infrastructure, integration style, persistence boundaries — synthesized from the whole
cohort's approved Intent, Contract, and Event Schema artifacts.

This is NOT a behavioral artifact. It may constrain implementation structure. It may
NEVER invent or alter behavior. Any behavioral gap discovered while producing this
document returns the affected feature to its owning Stage 1, 2, or 3 — it is not
resolved here.

Workflow: .codeos/prompts/03b-architecture-synthesis.md (4-step pipeline)
Decision behavior is owned by the conditional `architecture-entry` doctrine adapter and the
selected architecture policy; this template defines no additional boundary.
See: the architecture_synthesis_policy component selected by .codeos/dba-system.md
-->

## Identity and Version

| Field | Value |
|---|---|
| Cohort id | [cohort_id] |
| Baseline version | [N] |
| Status | draft / approved / superseded |
| Supersedes | [previous version or "none"] |
| Approved by | [human] |
| Approved at | [ISO date] |

<!-- This file (architecture/core-baseline.md) always holds only the CURRENT approved version.
When a new version supersedes this one, this file's content moves to
architecture/history/core-baseline-v<N>.md (named for the version it was current as) before
being overwritten with the new version — see the architecture_synthesis_policy component selected
by .codeos/dba-system.md →
"Verifying a `baseline_version` reference" for how registry entries reference a specific version. -->

**Cohort membership set (this version):**

| Feature ID | Role in cohort |
|---|---|
| [feature_id] | [e.g. canonical artifact owner / consumer / projection] |

<!-- Membership is versioned. Adding or removing a feature creates a new version and requires
an impact assessment — see "Cohort and baseline versioning" in
the architecture_synthesis_policy component selected by .codeos/dba-system.md. Prior Stage 4 work
approved under an earlier version is not invalidated merely by a membership change; it is
reconciled only if the impact assessment finds an actual structural conflict. -->

---

## Authoritative Decisions

<!-- Structural choices requiring explicit human sign-off. Not implementation detail — project-
level structure only. Every entry here must reflect a decision the human actually made, not an
inference. -->

**Crate / workspace topology:**
[Decision + rationale]

**Dependency direction:**
[Which crates/modules may depend on which — the DAG, and why]

**Shared infrastructure:**
[What is genuinely shared vs. feature-owned, and the boundary rule applied]

**Integration style:**
[Is the event log (`events/runtime_events.jsonl`) observational-only, or does something read it
to continue processing? State explicitly — do not default to event-sourcing by accident.]

**Persistence boundaries:**
[Decision + rationale]

**Implementation profile reference** (if one exists for this project):
[Reference to the project's Implementation Profile artifact, if declared — this baseline does
not itself decide implementation language]

---

## Derived Views

<!-- Mechanically derived from the approved artifacts above — regenerable, not itself a second
canonical source. Each view names the source artifacts it was built from. If a source artifact
changes, regenerate the view; do not hand-edit it out of sync. -->

**Canonical artifact ownership matrix** (derived from: [artifacts]):

| Canonical artifact | Owning feature | Source |
|---|---|---|

**Feature dependency graph** (derived from: [artifacts]):
[Narrative or diagram]

**Event producer/consumer matrix** (derived from: [event schemas]):

| Event | Producer | Consumers |
|---|---|---|

---

## Open Architectural Risks

[Named risks and their current status — not dismissed, not silently resolved]

## Revisit Triggers

[Explicit conditions under which this baseline must be revisited — e.g. a new core feature
proposed, a cross-feature event schema changes, an authoritative decision proves unworkable
during Stage 4]

---

<!-- METADATA -->
status: DRAFT
cohort_id: [cohort_id]
baseline_version: [N]
type: ARCHITECTURE_BASELINE
step_completed: 0
approved_by:
approved_at:
supersedes:
