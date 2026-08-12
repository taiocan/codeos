---
change_id: CHG-20260701-003
feature_id: UPG-0011
slug: readiness-checklist
triage_class: template
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0011__CHG-20260701-003__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0011 / CHG-20260701-003 — Lightweight PR / Pre-Release Readiness Checklist

## TRACE HEADER

```yaml
feature_id: UPG-0011
primary_feature_id: UPG-0011
change_id: CHG-20260701-003
slug: readiness-checklist
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0011
related_features: []
review_series: RVS__UPG-0011__CHG-20260701-003__S4
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
triage_class: template
scope_axis: self-dev only
corrects: ~
corrected_by: ~
follow_up_of: ~
fixes_findings: []
```

---

## Step 1 — Change Intent

### Problem

DBA Stage 7 (reconciliation) verifies behavioral alignment between artifacts and runtime
behavior. What it does not answer is whether a change is operationally ready to merge or
release: are tests complete, CI green, no unrelated files staged, no raw secrets in logs,
rollback path known? These questions sit outside the DBA behavioral stages but practitioners
need to answer them before merging.

Without a dedicated operational readiness artifact, teams either skip these checks
(introducing release risk) or improvise them differently each time (no shared standard).
Reconciliation is not the right place to add them — it would expand Stage 7 beyond its
behavioral scope and dilute its signal.

### What changes

| File | Change |
|---|---|
| `templates/readiness-checklist.md` | NEW — lightweight PR / pre-release readiness checklist template covering behavioral readiness (Stage 7/8/9 completion) and operational readiness (tests, CI, secrets, docs, rollback). |
| `backlog/UPG-0011-readiness-checklist.md` | Feature Thread: add this change. |
| `status/self-development.md` | Activate row for this change. |
| `status/roadmap.md` | Wave 3 UPG-0011 row: update planned change and state to IN_PROGRESS. |
| `changes/UPG-0011__CHG-20260701-003__readiness-checklist.md` | This change record. |

### What stays the same (scope boundary)

- `dba-system.md` — NOT in scope. The template is usable without a doctrine reference;
  integration into the doctrine is a follow-on `downstream-doctrine` change once the
  template is proven.
- Stage prompts `01–09` — NOT modified. The checklist is an operational artifact used
  after Stage 8/9; no stage prompt needs to reference it in this change.
- `prompts/` — no prompt changes. The checklist is a template practitioners fill in, not
  a session prompt.
- `scripts/` — no changes.
- `UPG-0024` (Pre-Release Evidence Package) — separate feature; not in scope here.

### Triage class: `template`

Creating one new template file (`templates/readiness-checklist.md`). Class is `template`.
4-step loop with PROFILE-3 review cadence (downstream-facing).

### Scope axis: `self-dev only`

No changes to `dba-system.md`. Toolkit template files only.

### Review profile: PROFILE-3

Template class, downstream-facing. Codex review before each step gate; human approval at
all four gates; reviewer output is advisory and non-gatekeeping.

### Originating backlog item

`backlog/UPG-0011-readiness-checklist.md` — Lightweight PR / Pre-Release Readiness Checklist.

---

## Step 2 — Acceptance Criteria

### AC-1: New template file exists and is well-formed

`templates/readiness-checklist.md` is created. It contains all four sections:
header, behavioral readiness, operational readiness, and decision.

Verification: file exists; all four sections present.

### AC-2: Header section contains the four identity fields

The template header section contains exactly these four fields:
- Feature
- Branch
- Commit
- PR

Verification: all four field labels present in the header section.

### AC-3: Behavioral readiness section covers Stage 7/8/9 completion without repeating Stage 7 content

The behavioral readiness section checks:
- Stage 7 reconciliation complete (yes/no — completion status only)
- No unresolved GAP / MISMATCH / MISSING items
- Stage 8 replay complete
- Stage 9 refinement complete or not needed

The section must not reproduce or duplicate Stage 7 reconciliation criteria — it checks
completion status, not re-runs behavioral alignment.

Verification: all four behavioral checks present; no Stage 7 criteria re-stated in this section.

### AC-4: Operational readiness section covers all ten items from the backlog brief

The operational readiness section contains all of the following:
1. Tests run
2. Tests skipped explained
3. CI status
4. No unrelated files
5. Runtime evidence sanitized
6. No raw secrets / PII in logs
7. Docs updated if needed
8. Stack / config manifest checked if dependency or config changed
9. Release notes do not overclaim
10. Rollback / revert path known

Verification: all ten items present in the operational readiness section.

### AC-5: Decision field supports exactly three outcomes

The decision section provides exactly three options:
- READY
- NOT READY
- READY WITH KNOWN LIMITATIONS

Verification: all three decision options present; no other decision values introduced.

### AC-6: Remaining risks field is present

The template includes a `Remaining risks:` field for the practitioner to record open
risks at decision time.

Verification: `Remaining risks` field present after the decision section.

### AC-7: Template does not present itself as a DBA stage or override Stage 7

The template must not be labeled as a stage, must not position itself as a replacement
for or extension of Stage 7 reconciliation, and must not introduce new behavioral
alignment criteria.

Verification: no "Stage 10" or equivalent stage label in the template; no behavioral
alignment criteria beyond completion-status checks.

### AC-8: `dba-system.md` is unchanged

The downstream doctrine is not modified in this change.

Verification: the file is not listed in the `What changes` table and does not appear
in the Step 3 implementation diff.

### AC-9: Stage prompts, `prompts/`, and `scripts/` are unchanged

No stage prompts (01–09), other prompt files, or script files are modified.

Verification: none of these paths are listed in the `What changes` table and none
appear in the Step 3 implementation diff.

---

## Step 3 — Implementation

### `templates/readiness-checklist.md` (NEW)

New template for operational PR / pre-release readiness. Key design decisions:

- **Header** contains the four identity fields from AC-2: Feature, Branch, Commit, PR.
- **Behavioral Readiness section** carries an inline note — "Completion status only — do
  not re-run Stage 7 criteria here" — to enforce the AC-3 / AC-7 guardrail against
  duplicating reconciliation. Covers all four behavioral checks: Stage 7 complete,
  no unresolved GAP/MISMATCH/MISSING, Stage 8 complete, Stage 9 complete or not needed.
- **Operational Readiness section** lists all ten items from AC-4 verbatim: tests run,
  tests skipped explained, CI status, no unrelated files, runtime evidence sanitized,
  no raw secrets/PII in logs, docs updated if needed, stack/config manifest checked,
  release notes do not overclaim, rollback/revert path known.
- **Decision section** presents exactly three options (AC-5) as checkboxes: READY /
  NOT READY / READY WITH KNOWN LIMITATIONS. Checkbox format makes the selected decision
  visible without requiring free-text interpretation.
- **Remaining risks** field follows the decision (AC-6).
- The template is not labeled as a DBA stage, not named "Stage 10," and introduces no
  behavioral alignment criteria — satisfying AC-7.
- `dba-system.md`, stage prompts, `prompts/`, and `scripts/` not touched (AC-8/9).

---

## Step 4 — Reconcile

### AC Verification

Note on AC-8/9: all implementation changes are uncommitted. `git diff HEAD -- <path>` confirms
no tracked file under that path was modified. `git status --short -- <path>` confirms no
untracked file was created there either. Both checks together prove the path is absent from
the implementation diff.

| AC | Verification | Result |
|---|---|---|
| AC-1 | `ls templates/readiness-checklist.md` → exists; header, behavioral, operational, and decision sections present | PASS |
| AC-2 | `grep -E "^(Feature\|Branch\|Commit\|PR):" templates/readiness-checklist.md` → 4 hits | PASS |
| AC-3 | Stage 7/8/9 completion checks present; inline note "Completion status only — do not re-run Stage 7 criteria here" guards against duplication | PASS |
| AC-4 | `grep -E "^- (Tests run\|Tests skipped…)" … \| wc -l` → 10 | PASS |
| AC-5 | `grep -E "READY\|NOT READY\|KNOWN LIMITATIONS"` → exactly 3 options as checkboxes | PASS |
| AC-6 | `grep "Remaining risks" templates/readiness-checklist.md` → hit | PASS |
| AC-7 | `grep -i "stage 10" templates/readiness-checklist.md` → no match; no behavioral alignment criteria introduced | PASS |
| AC-8 | `git diff HEAD -- dba-system.md \| wc -l` → 0 (no tracked changes); `git status --short -- dba-system.md` → empty (no untracked) | PASS |
| AC-9 | `git diff HEAD -- prompts/ scripts/ \| wc -l` → 0 (no tracked changes); `git status --short -- prompts/ scripts/` → 0 lines (no untracked files created) | PASS |

### Cross-reference sweep

| Reference | Target | Status |
|---|---|---|
| Change record → backlog brief | `backlog/UPG-0011-readiness-checklist.md` | OK |
| Change record → new template | `templates/readiness-checklist.md` | exists |
| backlog brief Feature Thread | CHG-20260701-003 row present, state IN_PROGRESS | OK |
| `status/self-development.md` UPG-0011 row | IN_PROGRESS → will mark COMPLETE on acceptance | OK |
| `status/roadmap.md` Wave 3 UPG-0011 row | IN_PROGRESS → will mark COMPLETE on acceptance | OK |

No stage-table↔prompt-file drift. No orphaned links. `dba-system.md` table intentionally
not updated (scope boundary declared in Step 1).

### Reviewer scope triage

| Finding | Round | Triage | Disposition |
|---|---|---|---|
| AC-8/9 verification only used `git diff HEAD` — doesn't prove absence of untracked files in prompts/ scripts/ | Step 4 R1 | IN-SCOPE BLOCKER | Fixed: added `git status --short -- <path>` check alongside diff; note updated to explain what each command proves |
