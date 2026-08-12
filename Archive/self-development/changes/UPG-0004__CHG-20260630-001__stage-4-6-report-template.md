# Self-Development Change: UPG-0004__CHG-20260630-001 — stage-4-6-report-template

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit
itself (prompts, templates, docs, patterns, scripts).

This is NOT a downstream DBA feature. It has no behavioral contract, no event schema,
and no replay. Trivial changes do not get a record.

Workflow: prompts/codeos-self-dev.md (4-step loop)
Each step requires explicit human approval; Codex review cadence is governed by the assigned review profile (see prompts/codeos-self-dev.md Step 0a).
The live status row lives in status/self-development.md, not here.

FILENAME CONVENTION (Feature Thread model — see backlog/UPG-0001-feature-thread-traceability.md):
  changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md
  - UPG-#### = the PRIMARY feature this change implements (visible grouping).
  - CHG-YYYYMMDD-NNN = the unique change id (execution).
  - slug describes the concrete change, not the whole roadmap.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0004
primary_feature_id: UPG-0004
change_id: CHG-20260630-001
slug: stage-4-6-report-template
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0004
related_features: []
review_series: null
review_profile: PROFILE-3
review_state: DRAFT
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY -->


## Change Intent

**Why (problem in the toolkit):**

Stages 4, 5, and 6 of the DBA pipeline (Implementation, Tests, Runtime Evidence) are where
most hidden work happens. The current toolkit provides no structured template for what must be
reported at each stage. Practitioners produce short prose summaries that do not show:

- which contract clauses were implemented vs skipped (Stage 4)
- which test categories exist, which are missing, and why gaps are acceptable (Stage 5)
- how the system was run, what was captured, and whether the log is safe to commit (Stage 6)

This leaves Stage 7 Reconciliation without reliable, comparable evidence to reconcile against.
It also makes reviewer scope assessment harder — the reviewer cannot tell what the practitioner
claims was done vs what is simply absent from the report.

**What changes:**

1. `templates/stage-4-6-report.md` — new file. Three structured report templates in one file:
   - Stage 4 Implementation Report
   - Stage 5 Test Report
   - Stage 6 Runtime Evidence Report

   Each template is a filled-field markdown form. Each template explicitly instructs
   practitioners that fields must not be left empty; when there is no content, the field must
   say `none`, `not run`, or `not applicable`. This CHG does not add script-level enforcement.

2. `backlog/UPG-0004-stage-4-6-reports.md` — Feature Thread Changes table row added for
   CHG-20260630-001.

3. `status/self-development.md` — operational row activated for `UPG-0004` / `CHG-20260630-001`
   at step 2-Acceptance.

**Bookkeeping artifact:**

- `changes/UPG-0004__CHG-20260630-001__stage-4-6-report-template.md` — this change record.

**Scope boundary — what stays the same:**

- `dba-system.md` — **not touched**. Mandating use of this template in the downstream doctrine
  is a follow-on `downstream-doctrine` CHG. This CHG makes the template available; adoption is
  a separate scope decision.
- All other templates, prompts, scripts, docs — unchanged.
- No stage names, approval rules, or DBA philosophy changes.

**Class:** `template`
**Scope axis:** `self-dev only`
**Backlog item:** `backlog/UPG-0004-stage-4-6-reports.md`

---

## Acceptance Criteria

<!-- Template existence and structure (A1–A3) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A1 | `templates/stage-4-6-report.md` exists and contains three clearly delimited report sections: Stage 4 Implementation Report, Stage 5 Test Report, Stage 6 Runtime Evidence Report | `ls templates/stage-4-6-report.md` exits 0; `grep "Stage 4\|Stage 5\|Stage 6" templates/stage-4-6-report.md` returns a heading for each |
| A2 | Each template section contains every field listed in the backlog design notes for that stage — no field from the design notes is omitted | Read-through: compare each section of `templates/stage-4-6-report.md` field-by-field against `backlog/UPG-0004-stage-4-6-reports.md` design notes; every listed field present |
| A3 | Each template section includes an explicit instruction that empty fields must say `none`, `not run`, or `not applicable` — and states that this is a template instruction, not script enforcement | `grep -c "none.*not run\|not applicable" templates/stage-4-6-report.md` → match in each section; no claim of automated validation |

<!-- Field quality (A4–A6) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A4 | Stage 4 fields cover every field in the backlog design note: feature identifier, approved artifacts used (intent / contract / event schema sub-fields), files changed, files inspected but not changed, contract clauses implemented, schema events emitted, correlation ID propagation, runtime artifacts touched, unimplemented clauses, assumptions, blocked items, requires earlier-stage change, unexpected complexity | Read-through against the Stage 4 design note in `backlog/UPG-0004-stage-4-6-reports.md`; every listed field present |
| A5 | Stage 5 fields cover every field in the backlog design note: feature identifier, approved artifacts used, behavioral tests added, failure-mode tests added, invariant tests added, telemetry/event tests added, replay tests added, tests run, tests passed, tests failed, tests skipped, tests not run, known test gaps, why gaps are acceptable or not acceptable | Read-through against the Stage 5 design note; every listed field present |
| A6 | Stage 6 fields cover every field in the backlog design note: feature identifier, how the system was run, input fixture/scenario, runtime command, runtime log path, events captured, unexpected events, missing expected events, correlation chains observed, sanitization status, raw logs committed (yes/no + if-yes reason), derived replay fixtures produced, ready for reconciliation, known runtime gaps | Read-through against the Stage 6 design note; every listed field present |

<!-- DBA traceability (A7) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A7 | The template header or preamble states which DBA stages the report applies to and that it feeds Stage 7 Reconciliation — so a practitioner encountering the file understands its purpose without reading the backlog brief | Read-through: header section names Stage 4, 5, 6 and mentions Stage 7 Reconciliation as consumer |

<!-- Scope boundary (A8) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A8 | `dba-system.md` is unchanged; no script changes; no prompt changes | Read through the files changed in this CHG; none should be from `dba-system.md`, `scripts/`, or `prompts/`. Confirm with `git diff <base_sha> -- dba-system.md scripts/ prompts/` → empty, where `<base_sha>` is the commit before this change began |
| A9 | `backlog/UPG-0004-stage-4-6-reports.md` Feature Thread Changes table contains a row for CHG-20260630-001 with file, purpose, and state | Read-through: row exists and is correctly filled |
| A10 | `status/self-development.md` has an operational row for UPG-0004 / CHG-20260630-001; Loop step reflects the current step at the time of each gate | Read-through: row exists and Loop step matches the step at which verification runs |

---

## Implementation Notes

`templates/stage-4-6-report.md` created. Three sections delimited by `---`, each with a
`*Complete after Stage N…*` lead and an explicit template instruction sentence. All fields
from the backlog design notes are present verbatim. The preamble names Stages 4, 5, 6 and
Stage 7 Reconciliation as consumer (A7). `status/self-development.md` loop step advanced
(see status dashboard for current step). No changes to `dba-system.md`, `scripts/`, or `prompts/`.

---

## Reconciliation

All acceptance criteria verified against the current artifacts.

| AC | Result | Evidence |
|---|---|---|
| A1 | PASS | `templates/stage-4-6-report.md` exists; `grep -c "## Stage [456]"` → 3 |
| A2 | PASS | Field-by-field comparison confirms no field from any design note is omitted (see A4–A6) |
| A3 | PASS | `grep -c "not script enforcement"` → 4 (preamble + one per section); no automation claim |
| A4 | PASS | All 13 Stage 4 fields present verbatim: Feature, Approved artifacts used (with Intent/Contract/Event schema sub-fields), Files changed, Files inspected but not changed, Contract clauses implemented, Schema events emitted, Correlation ID propagation, Runtime artifacts touched, Unimplemented clauses, Assumptions, Blocked items, Requires earlier-stage change, Unexpected complexity |
| A5 | PASS | All 14 Stage 5 fields present: Feature, Approved artifacts used, Behavioral/Failure-mode/Invariant/Telemetry-event/Replay tests added, Tests run/passed/failed/skipped/not run, Known test gaps, Why gaps acceptable or not |
| A6 | PASS | All 14 Stage 6 fields present: Feature, How run, Input fixture/scenario, Runtime command, Runtime log path, Events captured, Unexpected events, Missing expected events, Correlation chains, Sanitization status, Raw logs committed (yes/no + reason), Derived replay fixtures, Ready for reconciliation, Known runtime gaps |
| A7 | PASS | `grep -c "Stage 7 Reconciliation"` → 5 (preamble × 2, per-section × 3); Stages 4/5/6 named in preamble and section headings |
| A8 | PASS | `git diff HEAD -- dba-system.md scripts/ prompts/` → empty |
| A9 | PASS | `backlog/UPG-0004-stage-4-6-reports.md` Feature Thread Changes table has CHG-20260630-001 row |
| A10 | PASS | `status/self-development.md` UPG-0004 row shows Loop step `4-Reconcile` at the Reconcile gate |

**Scope sweep:** `git diff --name-only HEAD` shows only in-scope files (backlog brief, status dashboard); untracked additions are `templates/stage-4-6-report.md` and this change record. `reviews/` changes are review bookkeeping, not UPG-0004 scope. `status/roadmap.md` was reverted to HEAD (not part of this change).

**Stale-reference sweep:** No stale internal paths. `prompts/codeos-self-dev.md` references in lines 10–11 are inside the template comment block and correctly describe the workflow; they are not scope claims.

**OUT-OF-SCOPE finding carried forward:** Precheck script false positives on `UPG-####` in comment/legend sections — tracked to UPG-0031 (script-review-pipeline).
