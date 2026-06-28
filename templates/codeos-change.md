# Self-Development Change: UPG-####__CHG-YYYYMMDD-NNN — slug

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit
itself (prompts, templates, docs, patterns, scripts).

This is NOT a downstream DBA feature. It has no behavioral contract, no event schema,
and no replay. Trivial changes do not get a record.

Workflow: prompts/codeos-self-dev.md (4-step loop)
Each step requires explicit human approval and a compulsory (advisory) Codex review.
The live status row lives in status/self-development.md, not here.

FILENAME CONVENTION (Feature Thread model — see backlog/UPG-0001-feature-thread-traceability.md):
  changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md
  - UPG-#### = the PRIMARY feature this change implements (visible grouping).
  - CHG-YYYYMMDD-NNN = the unique change id (execution).
  - slug describes the concrete change, not the whole roadmap.
  - Multi-feature change: keep the primary UPG-#### in the filename, list the rest in
    `related_features`. Use `MULTI__CHG-…` only when there is genuinely no primary feature and
    the human explicitly approves it (rare).
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-####
primary_feature_id: UPG-####
change_id: CHG-YYYYMMDD-NNN
slug: slug
state: DRAFT            # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 1-Intent  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-####
related_features: []
review_series: null     # e.g. RVS__UPG-####__CHG-YYYYMMDD-NNN__S<N> = ALL Step-N reviews for this change (stable)
review_state: DRAFT     # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round)
review_history: reviews/review-log.md   # exact per-round REV__…__R<N> verdicts + human decisions live here, never in this artifact
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round (which does not exist until after the packet is built). Reference the stable review
SERIES (review_series) + review_state; exact rounds live only in reviews/review-log.md and
reviews/codex/*. See prompts/codeos-self-dev.md → "Feature Thread & IDs" / "Self-Reference Boundary". -->


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

<!-- METADATA (approval) — the canonical trace header is at the top of this file -->
status: DRAFT
feature_id: UPG-####
primary_feature_id: UPG-####
change_id: CHG-YYYYMMDD-NNN
type: SELF_DEVELOPMENT
class: [class]
scope: [self-dev only | downstream doctrine only | both]
backlog_item: backlog/UPG-####-slug.md
step_completed: 0
approved_by:
approved_at:
