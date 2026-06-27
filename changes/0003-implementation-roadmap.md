# Self-Development Change: 0003-implementation-roadmap

## Change Intent

**Why (problem in the toolkit):**
The backlog (`backlog/features.md` + ~24 briefs) carries priorities but no current-state,
dependency-aware sequencing. With `0001` (split) and `0002` (docs) done and the reviewer
advisory pilot landed, the toolkit needs a single mutable roadmap so future sessions can pick
features off one at a time as self-dev changes, without re-deriving order each time.

**What changes:**
- New `status/roadmap.md` — current state + 5 dependency-aware waves mapping every active
  backlog item to a wave and its prerequisites, plus immediate next pickups and the backlog's
  "do NOT do yet" guardrails.

**Scope boundary — what stays the same:**
- `backlog/features.md` stays the stable brief catalog; the roadmap does not duplicate briefs.
- The roadmap is **planning state, not doctrine, and not authorization** — each item still
  needs its own self-dev change + human approval.
- No prompts, templates, scripts, or doctrine change.

**Class:** backlog-only / self-dev planning
**Scope axis:** self-dev only
**Backlog item:** — (planning artifact over the whole backlog)

---

## Acceptance Criteria

| # | Criterion | How verified |
|---|---|---|
| 1 | Carries the "not doctrine / not authorization" header disclaimer | read-through |
| 2 | Every active implementation item appears exactly once in a wave; piloted/done items (`0001`, `0002`, reviewer-decision-brief) are in Current State, not a wave | cross-check against backlog inventory |
| 3 | Dependencies shown and acyclic (e.g. ci-profile/stack-drift-detector → stack-manifest; 00b-adr-generator → solution-discovery-00b; approval-dashboard → feature-registry + reviewer) | read-through |
| 4 | Honest current state (reviewer-quality-scale shown partially piloted, scheduled once in Wave 1) | read-through |

---

## Implementation Notes

Created `status/roadmap.md`. Waves: 1 Transparency & state, 2 Discovery & registry, 3 Evidence
discipline & gates, 4 Reviewer hardening & delivery, 5 Advanced automation & generators.
`reviewer-quality-scale` is scheduled exactly once (Wave 1) with a partial-pilot annotation;
its status note in Current State explicitly points to Wave 1 rather than re-listing it.

---

## Reconciliation

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Disclaimer present | PASS | header blockquote |
| 2 | Each active item once; done/piloted in Current State | PASS | 22 active items across waves + reviewer-quality-scale (Wave 1); reviewer-decision-brief + 0001/0002 in Current State |
| 3 | Deps shown, acyclic | PASS | dependency column per wave |
| 4 | Honest current state | PASS | reviewer-quality-scale = partially piloted |

**Codex review:** RUN (retroactively, as the doctrine-split series review at HEAD 2563e37).
Verdict **CHANGES ADVISED** (evidence A, full coverage) — see
`reviews/codex/2026-06-27T163034Z-dsplit-0003-stage-selfdev-step-4-2563e37.md` and
`reviews/review-log.md`. Two IN-SCOPE BLOCKERs + one NON-BLOCKER raised; all addressed by
change `0004-review-fixes` (status-dashboard roadmap pointer, review-record compliance,
roadmap wording).

---

<!-- METADATA -->
status: COMPLETE
change_id: 0003-implementation-roadmap
type: SELF_DEVELOPMENT
class: backlog-only / self-dev planning
scope: self-dev only
backlog_item: —
step_completed: 4
approved_by: human (in-session, 2026-06-27)
approved_at: 2026-06-27
