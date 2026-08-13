---
change_id: CHG-20260701-002
feature_id: UPG-0010
slug: verify-only-mode
triage_class: prompt
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0010__CHG-20260701-002__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0010 / CHG-20260701-002 — Verification-Only Mode

## TRACE HEADER

```yaml
feature_id: UPG-0010
primary_feature_id: UPG-0010
change_id: CHG-20260701-002
slug: verify-only-mode
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0010
related_features: []
review_series: RVS__UPG-0010__CHG-20260701-002__S4
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
triage_class: prompt
scope_axis: self-dev only
corrects: ~
corrected_by: ~
follow_up_of: ~
fixes_findings: []
```

---

## Step 1 — Change Intent

### Problem

During verification runs, Claude may "helpfully" edit files — fixing failing tests, rewriting
assertions, updating snapshots — so that checks pass. This silently mutates the state the
verification was supposed to measure, blurring evidence. The downstream project has no way
to distinguish a verification run where everything passed cleanly from one where Claude fixed
failures mid-run to make them pass. This erodes the evidence integrity that the DBA philosophy
depends on.

No prompt currently establishes a strict read-only constraint for verification contexts. Stage
prompts 07–09 assume verification is clean but do not enforce it.

### What changes

| File | Change |
|---|---|
| `prompts/verify-only.md` | NEW — read-only verification mode prompt. Defines the no-edit rule, anti-blur pre/post git checks, verification report format, where to use the mode, and what distinguishes it from reconciliation. |
| `backlog/UPG-0010-verify-only-mode.md` | Feature Thread: add this change. |
| `status/self-development.md` | Activate row for this change. |
| `status/roadmap.md` | Wave 3 UPG-0010 row: update planned change and state to IN_PROGRESS. |
| `changes/UPG-0010__CHG-20260701-002__verify-only-mode.md` | This change record. |

### What stays the same (scope boundary)

- `dba-system.md` — NOT in scope. The downstream doctrine's stage table is intentionally not
  updated here. The new prompt is usable without a doctrine reference; integration into the
  doctrine is a follow-on `downstream-doctrine` change once the prompt is proven. (Same
  deferral pattern used for UPG-0007.)
- Stage prompts `01–09` — NOT modified. No cross-links added in this change; the verify-only
  mode is a standalone prompt practitioners load by choice.
- `scripts/` — no changes. Verification mode is a prompt-level constraint, not tooling.
- `templates/` — no changes.
- `UPG-0025` (Verification Packet for Reviewer Agent) — separate feature; not in scope.

### Triage class: `prompt`

Creating one new prompt file (`prompts/verify-only.md`). Class is `prompt`. 4-step loop with
PROFILE-3 review cadence (downstream-facing; same as UPG-0007).

### Scope axis: `self-dev only`

No changes to `dba-system.md`. Toolkit prompt files only.

### Review profile: PROFILE-3

Prompt class, downstream-facing. Codex review before each step gate; human approval at all
four gates; reviewer output is advisory and non-gatekeeping.

### Originating backlog item

`backlog/UPG-0010-verify-only-mode.md` — Verification-Only Mode.

---

## Step 2 — Acceptance Criteria

### AC-1: New prompt file exists and is well-formed

`prompts/verify-only.md` is created. It contains:
- A clear purpose statement establishing this as a read-only verification mode.
- An explicit no-edit rule.
- Anti-blur pre/post git checks.
- A verification report format.
- Guidance on where to use the mode.
- A statement distinguishing verification-only from reconciliation.

Verification: file exists; all six sections are present and coherent.

### AC-2: No-edit rule is explicit and covers all forbidden actions

The prompt explicitly forbids all of the following during a verification run:

1. Editing files
2. Fixing failures
3. Rewriting tests
4. Updating snapshots
5. Changing runtime fixtures
6. Staging, committing, or formatting files

The rule must instruct practitioners to run only the requested checks and report exact
results without alteration.

Verification: each of the six prohibited actions is named in the prompt; the "run only
and report" instruction is present.

### AC-3: Anti-blur git checks are specified

The prompt instructs practitioners to run the following before verification begins:

```bash
git status --short
git rev-parse HEAD
```

And after verification completes:

```bash
git status --short
git diff --exit-code
git rev-parse HEAD
```

If the working tree changed between pre and post checks, the verification run is explicitly
declared invalid.

Verification: both pre-check and post-check command sets are present; the invalidity
condition (working tree changed) is stated.

### AC-4: Verification report format is present and complete

The prompt provides a report template (or instructs the practitioner to produce a report)
containing all of the following fields:

- Checks requested
- Commands run
- Pre-check commit hash
- Pre-check working tree state
- Results broken down as: passed / failed / skipped / blocked
- Post-check commit hash
- Post-check working tree state
- Files changed (if any)
- Verification validity: VALID or INVALID
- If INVALID: reason

Verification: all ten fields are present in the template or instructions.

### AC-5: Verification-only is distinguished from reconciliation

The prompt explicitly states:
- Verification-only is mechanical: run checks without edits.
- Reconciliation is semantic: verify that all artifacts and runtime behavior align.
- Verification-only feeds reconciliation; it does not replace it.

Verification: all three distinctions are present in the prompt.

### AC-6: `dba-system.md` is unchanged

The downstream doctrine is not modified in this change (deferring to a follow-on
`downstream-doctrine` change once the prompt is proven).

Verification: the file is not listed in the `What changes` table and does not appear
in the Step 3 implementation diff.

### AC-7: Stage prompts 01–09 and `templates/` are unchanged

No stage prompts or template files are modified.

Verification: none of these paths are listed in the `What changes` table and none
appear in the Step 3 implementation diff.

---

## Step 3 — Implementation

### `prompts/verify-only.md` (NEW)

New prompt file for read-only verification mode. Key design decisions:

- **Role section** opens with explicit "read-only verification pass" framing and the
  "optional and practitioner-loaded" qualifier — consistent with the advisory pattern
  established in UPG-0007. The mode is not a mandatory DBA stage.
- **No-Edit Rule** enumerates all six prohibited actions from AC-2 by name: Edit files,
  Fix failing tests or checks, Rewrite tests, Update snapshots, Change runtime fixtures,
  Stage/commit/format files. The closing instruction ("Run only the checks that were
  requested. Report the exact results.") satisfies AC-2's "run only and report" requirement.
- **Anti-Blur Checks** specifies the exact pre/post command sets from the backlog brief.
  The invalidity condition (working tree mutated) is stated explicitly — satisfying AC-3.
- **Verification Report** provides the full template with all ten required fields from
  AC-4, plus a clarifying instruction that `Files changed` must list any files appearing
  in the post-check git status that were absent in the pre-check.
- **Where to Use** table lists the four contexts from the backlog brief (Stage 7, Stage 8,
  Stage 9, PR gate) plus the reviewer agent context, without making any of them mandatory.
- **What This Mode Is NOT** section explicitly separates verification-only from
  reconciliation (mechanical vs. semantic; feeds, does not replace) and states the
  non-prerequisite rule — satisfying AC-5 fully.
- `dba-system.md` not touched — scope boundary held (AC-6).
- Stage prompts 01–09 and `templates/` not touched (AC-7).

---

## Step 4 — Reconcile

### AC Verification

Note on AC-6/7: all implementation changes are uncommitted working-tree edits. `git diff HEAD -- <file>`
returning 0 lines confirms the file is absent from the implementation diff.

| AC | Verification | Result |
|---|---|---|
| AC-1 | `ls prompts/verify-only.md` → exists; all six sections present (Role, No-Edit Rule, Anti-Blur Checks, Verification Report, Where to Use, What This Mode Is NOT) | PASS |
| AC-2 | `grep -E "^- (Edit\|Fix\|Rewrite\|Update\|Change\|Stage)" prompts/verify-only.md` → 6 hits; `grep "Run only the checks" prompts/verify-only.md` → hit | PASS |
| AC-3 | Both `git status --short` + `git rev-parse HEAD` present in pre-check block; all three post-check commands present; invalidity condition ("working tree was mutated … INVALID") stated | PASS |
| AC-4 | All ten fields present in report template: Checks requested, Commands run, Pre-check commit, Pre-check working tree, Results (passed/failed/skipped/blocked), Post-check commit, Post-check working tree, Files changed, Verification validity, If invalid why | PASS |
| AC-5 | "Verification-only is mechanical … Reconciliation is semantic"; "Verification-only feeds reconciliation; it does not replace it" — all three distinctions present in "What This Mode Is NOT" | PASS |
| AC-6 | `git diff HEAD -- dba-system.md \| wc -l` → 0 | PASS |
| AC-7 | `git diff HEAD -- prompts/01-intent.md … prompts/09-replay.md templates/ \| wc -l` → 0 | PASS |

### Cross-reference sweep

| Reference | Target | Status |
|---|---|---|
| Change record → backlog brief | `backlog/UPG-0010-verify-only-mode.md` | OK |
| Change record → new prompt | `prompts/verify-only.md` | exists |
| backlog brief Feature Thread | CHG-20260701-002 row present, state IN_PROGRESS | OK |
| `status/self-development.md` UPG-0010 row | IN_PROGRESS → will mark COMPLETE on acceptance | OK |
| `status/roadmap.md` Wave 3 UPG-0010 row | IN_PROGRESS → will mark COMPLETE on acceptance | OK |

No stage-table↔prompt-file drift. No orphaned links. `dba-system.md` prompt table intentionally
not updated (scope boundary declared in Step 1, same deferral pattern as UPG-0007).

### Reviewer scope triage

| Finding | Round | Triage | Disposition |
|---|---|---|---|
| Step 3 notes claimed "verbatim" AC-2 coverage but prompt used "Rewrite assertions or test logic" vs AC term "Rewriting tests" | Step 3 R1 | IN-SCOPE BLOCKER | Fixed: prompt updated to "Rewrite tests"; notes updated to remove verbatim claim |
| AC-2 still not met after R1 fix — notes still claimed substantive equivalence is sufficient | Step 3 R2 | IN-SCOPE BLOCKER | Fixed: aligned prompt to exact AC term "Rewrite tests"; Step 3 notes simplified |
