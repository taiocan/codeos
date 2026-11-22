# WO-06 — JamFree Pilot Validation

Real use · NORMAL · after WO-05 · Charter: success criteria 1–10, C-9

## Goal

Confirm on a real project that the panel answers the Charter's ten questions without manual
repository reconstruction.

## In scope

- Run `status`, `status --json`, `status --history` read-only on JamFree.
- Rate each success criterion ANSWERED, PARTIAL, or NOT ANSWERED with supporting output.
- Deliver a diagnostic gap report (location agreed at start); the human routes each material gap
  to an Observation, a backlog WO, or no action.

## Out of scope

Creating Observations or other workflow state; changing JamFree; JamFree rules in CodeOS (C-9);
TomTom calls.

## Acceptance criteria

1. All ten criteria are covered with evidence.
2. JamFree `.codeos/` is byte-identical before and after.
3. No gap is fixed inside this WO.

## Verification

`.codeos/` checksums before and after; human review of the gap report.

## Results

**ACCEPTED with follow-up gaps routed** — 2026-10-10. Run on JamFree with the debug build:
`status`, `status --json`, `status --history`, `status --history --since all`. Exit 2 (blocked).
No TomTom calls. JamFree `.codeos/` byte-identical before and after (toolkit symlink excluded).

| # | Question | Rating | Evidence |
|---|---|---|---|
| 1 | Where are we | ANSWERED | B1–B3 ✓, B4 waiting HUMAN, B5 ×; no active feature (gap A) |
| 2 | Historically accepted | PARTIAL | JSON `historically_accepted`, `accepted_snapshot`; terminal lacks it (gap B) |
| 3 | Currently valid | ANSWERED | `current_validity` STALE for F-0001–F-0004 with reason codes |
| 4 | Foundations with current evidence | PARTIAL | 13 domains with JSON basis; terminal hides basis (gap D). Qualification domains UNOBSERVED: JamFree has no binding-current `project_qualification` record — correct absence, not a defect |
| 5 | Blocked/stale/unresolved/noteworthy | ANSWERED | 1 UNRESOLVED (bootstrap B4), 4 STALE accepted features |
| 6 | Why | ANSWERED | reason code, changed/unchanged surfaces, evidence ids, authority path |
| 7 | Next legitimate action | PARTIAL | typed action and actor; `revalidate_acceptance` has no mechanism (gap E) |
| 8 | AUTO/ASSISTED/HUMAN | ANSWERED | every attention item has an actor |
| 9 | Where effort is lost | PARTIAL | 74 additional decision receipts, 32 non-progressing assessments, 1 execution-invalid — totals only, not localized (gap F) |
| 10 | Observations needing improvement | ANSWERED | 2 opened, 2 closed, 0 open, 1 implementation-defect route, closure times |

Not a gap: Release `UNQUALIFIED` — the qualified release binary refused an unqualified build and
the panel exposed it.

| Gap | Route |
|---|---|
| A — accepted-but-stale features reported as "incomplete" | [WO-08](WO-08-terminal-acceptance-clarity.md) |
| B — terminal does not show historical acceptance | WO-08 |
| C — Product `CURRENT` while B4 preview is stale after a Charter change | NO ACTION — valid distinction: Product health covers the Charter; B4 covers dependent bootstrap evidence. Product does not aggregate downstream validity |
| D — terminal hides the health basis | WO-08 |
| E — `revalidate_acceptance` has no mechanism | Re-routed: the mechanism exists under Workflow Governance v4 ([REVAL](../../archive/self-development/backlog/completed/REVAL-bounded-acceptance-revalidation.md), closed); the defect was the non-atomic action, repaired in [WO-10](WO-10-atomic-revalidation-actions.md) |
| F — history totals are not localized | [WO-09](WO-09-history-attribution.md), high priority |

### Re-rating after WO-08 and WO-09 — 2026-10-10

Same read-only commands on JamFree; `.codeos/` byte-identical; no TomTom calls.

| # | Was | Now | Closed by |
|---|---|---|---|
| 1 | ANSWERED | ANSWERED | gap A closed (WO-08): `none derived — 4 accepted, currently STALE` |
| 2 | PARTIAL | ANSWERED | gap B closed (WO-08): feature lines show accepted snapshot and current validity |
| 4 | PARTIAL | ANSWERED | gap D closed (WO-08): every health status shows its basis |
| 7 | PARTIAL | PARTIAL | gap E remains, routed to REVAL outside ECP; the panel reports it truthfully |
| 9 | PARTIAL | ANSWERED | gap F closed (WO-09): friction signals located by subject and checkpoint |

Criteria 3, 5, 6, 8, 10 unchanged (ANSWERED).

### Re-rating after WO-10 — 2026-10-10

Same four read-only commands with a build at HEAD; exit 2 (blocked). JamFree `.codeos/`
byte-identical; no TomTom calls; no JamFree state or production evidence created.

| # | Was | Now | Evidence |
|---|---|---|---|
| 7 | PARTIAL | ANSWERED | gap E closed (WO-10): F-0003/F-0004 `[AUTO] run_verification` with JSON command `check --workflow feature --subject F-000x --revalidation --trigger c87c46d…`; F-0002 `[ASSISTED]` with the exact frontier refusal; F-0001 `[ASSISTED] reassess_architecture` |
| 9 | ANSWERED | ANSWERED | reconfirmed: friction signals still located by subject and checkpoint |

All ten criteria ANSWERED: read-only ECP-2–ECP-5 is complete. An exact refusal answers question 7;
the terminal names the action and trigger, and the exact command is in `status --json` (ECP-5).
ECP-6 (WO-07) remains unapproved pending real JamFree AUTO-command evidence.

**Correction (WO-11):** the F-0001 action above was a fixed string. F-0001 already had an ordinary
candidate assessed BLOCKED, so the legitimate next action was `resolve_review_finding`. WO-11
repairs this, and question 7 is ANSWERED again: F-0001 shows `[ASSISTED] resolve the assessment of
ordinary Candidate Bundle 6449208f…`.
