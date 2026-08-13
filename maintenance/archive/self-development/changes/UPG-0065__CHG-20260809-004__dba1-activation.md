# Self-Development Change: UPG-0065__CHG-20260809-004 — dba1-activation

<!--
PURPOSE: Seventh change under UPG-0065. Second (and final) half of Phase A's sixth sub-step —
the actual Invariant 1(d) act: dba-system.md is atomically replaced with a thin manifest naming
DBA-1 as the active configuration. Safe now because CHG-20260809-003 already retargeted every
consumer citation to dba/*/v1.md — no consumer depends on dba-system.md's current section
structure anymore. Once this lands, Phase A is complete.
Workflow: prompts/codeos-self-dev.md (4-step loop).
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0065
primary_feature_id: UPG-0065
change_id: CHG-20260809-004
slug: dba1-activation
state: DRAFT             # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 1-Intent   # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0065
related_features: []
review_series: null      # Set once Step 1's review runs
review_profile: PROFILE-4   # downstream-doctrine (Step 0a)
review_state: DRAFT      # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round; resets per step)
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round. Reference the stable review SERIES (review_series) + review_state; exact rounds live
only in reviews/review-log.md and reviews/codex/*. -->

---

## Change Intent

**Why (problem in the toolkit):**

`DBA-1` is approved (`CHG-20260809-002`) and every consumer citation now points at its `dba/*/v1.md`
components (`CHG-20260809-003`), but `dba-system.md` itself is still the full 792-line monolith —
nothing actually reads `DBA-1` yet. Invariant 1(d) is the last step of Phase A: replace
`dba-system.md`'s content with a short manifest naming `DBA-1` as active, per the brief's own
illustrative sketch and the human's minimal-manifest decision (2026-08-09).

**What changes:**

- `changes/UPG-0065__CHG-20260809-004__dba1-activation.md` (this file) — the change record.
- `dba-system.md` — content replaced with the thin manifest. Per the brief's sketch:
  ```markdown
  # Active DBA Configuration
  active_configuration: dba/configurations/DBA-1.yaml

  All components named in the active configuration are jointly authoritative and must be loaded
  when applicable. Do not load a component version not named by the active configuration.
  ```
  The exact wording is drafted at Step 3; the binding requirement (Step 2 AC) is that it names
  `DBA-1.yaml` and instructs the reader to load every component it names — this is what makes the
  5 `WHOLE-FILE-LOAD` consumer instructions ("read `dba-system.md`") keep working without editing
  those consumer files, per `CHG-20260809-003`'s own scope-correction reasoning.
- `backlog/UPG-0065-modular-dba-configuration-architecture.md`, `status/self-development.md`,
  `status/roadmap.md` — updated as this change progresses. Once this change completes, Phase A is
  done and `backlog/features.md`/`status/roadmap.md`'s `UPG-0065` row moves from `IN_PROGRESS`
  toward reflecting Phase A complete (Phase B remains separate, not started by this change).

**Scope boundary — what stays the same:**

- No `dba/*/v1.md` file is edited. No `dba/configurations/DBA-1.yaml` edit (still `status:
  approved`, unchanged).
- No consumer file (`prompts/`, `scripts/`, `templates/`, `patterns/`) is edited — `CHG-20260809-003`
  already did that half; re-touching them here would be scope creep.
- `dba-system-lean.md` is not touched. No `DBA-2`, no Phase B work of any kind.
- The 12 `dba/*/v1.md` citation/wording defects `CHG-20260809-001` recorded as human-waived remain
  untouched and out of scope here too.
- No downstream project's actual `.codeos` symlink or local files are touched — this change is
  entirely within the Codeos toolkit repo; a downstream project's own re-sync (if any is ever
  needed) is not this change's concern, since the symlink already points at this repo and the
  manifest redirect is designed to need no downstream action.

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** backlog/UPG-0065-modular-dba-configuration-architecture.md

---

## Acceptance Criteria

_To be drafted at Step 2, after this Step 1 gate._

---

## Implementation Notes

_To be filled at Step 3._

---

## Reconciliation

_To be filled at Step 4._
