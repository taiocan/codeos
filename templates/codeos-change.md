# Self-Development Change: [change_id]

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit
itself (prompts, templates, docs, patterns, scripts).

This is NOT a downstream DBA feature. It has no behavioral contract, no event schema,
and no replay. Trivial changes do not get a record.

Workflow: prompts/codeos-self-dev.md (4-step loop)
Each step requires explicit human approval and a compulsory (advisory) Codex review.
The live status row lives in status/self-development.md, not here.
-->

## Change Intent

**Why (problem in the toolkit):**
[What is wrong / missing today]

**What changes:**
[Name every file to be touched]

**Scope boundary — what stays the same:**
[Anything not listed here is in scope for change]

**Class:** [trivial | backlog-only | documentation | template | prompt | script-tooling | downstream-doctrine | self-dev-governance]
**Scope axis:** [self-dev only | downstream doctrine only | both]
**Backlog item:** [backlog/[id].md or "—"]

---

## Acceptance Criteria

<!-- The consistency contracts this change must satisfy. Each must be checkable in Reconcile. -->

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | [e.g., generated project CLAUDE.md still loads .codeos/dba-system.md] | [grep / dba-init smoke / read-through] |

<!-- For downstream-doctrine or both: include downstream-compatibility criteria. -->
<!-- For script-tooling: include I/O behavior, exit-code / fail-closed cases, idempotency. -->

---

## Implementation Notes

<!-- Filled during Step 3. Summary only — the git diff is the source of truth.
Note decisions, discoveries, and anything deferred (and re-triaged as its own change). -->

[Summary of edits made. Confirm all cross-references updated. Note any out-of-scope items filed to backlog.]

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | [criterion] | PASS / FAIL | [command output / file ref] |

**Consistency sweep (grep):**
[Stale references / orphaned links / stage-table ↔ prompt-file drift — clean, or gaps fixed/filed]

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| [finding] | IN-SCOPE BLOCKER / IN-SCOPE NON-BLOCKER / OUT-OF-SCOPE BACKLOG / REJECTED | [fixed / filed to backlog / dismissed] |

---

<!-- METADATA -->
status: DRAFT
change_id: [change_id]
type: SELF_DEVELOPMENT
class: [class]
scope: [self-dev only | downstream doctrine only | both]
backlog_item: [backlog/[id].md or "—"]
step_completed: 0
approved_by:
approved_at:
