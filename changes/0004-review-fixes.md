# Self-Development Change: 0004-review-fixes

## Change Intent

**Why (problem in the toolkit):**
The retroactive doctrine-split series Codex review (run at HEAD `2563e37`, verdict
**CHANGES ADVISED**, evidence A; `reviews/review-log.md`) raised two IN-SCOPE BLOCKERs and one
NON-BLOCKER against the `0001`–`0003` work. They must be fixed before merge.

**What changes:**
- `status/self-development.md` — fix the dashboard header that still named `backlog/features.md`
  as "the roadmap" → name `status/roadmap.md` as the roadmap and `backlog/features.md` as the
  stable catalog (BLOCKER-1). Populate the `Review` column for `0001`–`0003`.
- `status/roadmap.md` — soften over-absolute wording about every backlog item being a numbered
  self-dev change, acknowledging `reviewer-decision-brief` landed outside the sequence
  (NON-BLOCKER-3).
- `changes/0001..0003-*.md` — update the "Codex review" lines to reference the series review
  and these fixes, so no record claims COMPLETE without a review record (BLOCKER-2).

**Scope boundary — what stays the same:**
- No doctrine, prompt, template, or script change. Pure bookkeeping/consistency fixes driven
  by the reviewer.

**Class:** documentation / self-dev-governance
**Scope axis:** self-dev only
**Backlog item:** — (reviewer findings on 0001–0003)

---

## Acceptance Criteria

| # | Criterion | How verified |
|---|---|---|
| 1 | No file calls `backlog/features.md` "the roadmap"; `status/roadmap.md` is named the roadmap | grep `the roadmap` across status/, read-through |
| 2 | `Review` column populated for 0001–0003 in the dashboard | read-through |
| 3 | No `changes/*` record claims COMPLETE while stating its review was "not run" | grep "not run" changes/ |
| 4 | Roadmap wording no longer over-claims universal numbered sequence | read-through |

---

## Implementation Notes

Header + Review-column edits in `status/self-development.md`; wording edit in
`status/roadmap.md`; "Codex review" line updates in the three prior change records. The series
review verdict (CHANGES ADVISED) and the human decision (REQUEST_CHANGES → fixes) are in
`reviews/review-log.md`.

---

## Reconciliation

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Roadmap pointer correct | PASS | dashboard header + roadmap header agree (status/roadmap.md = roadmap; features.md = catalog) |
| 2 | Review column populated | PASS | 0001–0003 show `CHANGES ADV →0004` |
| 3 | No COMPLETE-without-review claim | PASS | `grep "not run" changes/` → none |
| 4 | Wording tightened | PASS | roadmap intro acknowledges out-of-sequence pilot |

**Codex review:** reviewed iteratively; full round history is in `reviews/review-log.md`
(append-only). The first round caught a **substantive** issue — `0001`'s status-dashboard
header still named `backlog/features.md` as "the roadmap" — which is fixed. Subsequent rounds
returned **CHANGES ADVISED** only on bookkeeping-recursion: the reviewer reviewing the very
dashboard/record that tracks its own review (premature COMPLETE state, a round-counter that
lagged the narrative, legend wording). Each was tightened; the residual is non-substantive and
self-referential. Per the advisory-not-gatekeeping principle, this is a **human-decision** point:
the review informs, it does not gate. **Human decision: APPROVE_STAGE** — residual accepted as
non-blocking; recursion limitation filed to `backlog/reviewer-self-reference-recursion.md`.
See `reviews/review-log.md`.

---

<!-- METADATA -->
status: COMPLETE
change_id: 0004-review-fixes
type: SELF_DEVELOPMENT
class: documentation / self-dev-governance
scope: self-dev only
backlog_item: —
step_completed: 4
approved_by: human (in-session, 2026-06-27)
approved_at: 2026-06-27
