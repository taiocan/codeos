---
change_id: CHG-20260701-004
feature_id: UPG-0012
slug: repair-before-next-feature
triage_class: prompt
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0012__CHG-20260701-004__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0012 / CHG-20260701-004 — Repair-Before-Next-Feature Workflow Gate

## TRACE HEADER

```yaml
feature_id: UPG-0012
primary_feature_id: UPG-0012
change_id: CHG-20260701-004
slug: repair-before-next-feature
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0012
related_features: []
review_series: RVS__UPG-0012__CHG-20260701-004__S4
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

The DBA loop produces an evidence chain per feature. It is easy to start a new behavioral
feature while the current one still has unresolved issues — GAP/MISMATCH/MISSING from
Stage 7, replay failures, required refinements, structural blockers, CI failures, or
pre-release blockers. When this happens, the evidence chain for the in-flight feature is
silently abandoned and the next feature inherits an environment whose quality state is
unknown. This undermines the integrity of both features.

Before this change, no prompt told practitioners to stop and repair before starting fresh
work.

### What changes

| File | Change |
|---|---|
| `prompts/00-session-start.md` | Add a Repair-Before-Next-Feature rule: a named list of blocking conditions and a human-override clause. Placed at Step 3 (Current Verified State), where the practitioner already assesses in-flight work. |
| `backlog/UPG-0012-repair-before-next-feature.md` | Feature Thread: add this change. |
| `status/self-development.md` | Activate row for this change. |
| `status/roadmap.md` | Wave 3 UPG-0012 row: update planned change and state to IN_PROGRESS. |
| `changes/UPG-0012__CHG-20260701-004__repair-before-next-feature.md` | This change record. |

### What stays the same (scope boundary)

- `CLAUDE.md` — NOT modified. The self-dev guide governs Codeos toolkit development, not
  downstream DBA projects. The rule belongs in the downstream-facing session prompt.
- `dba-system.md` — NOT in scope. Doctrine update is a follow-on `downstream-doctrine`
  change once the rule is proven in the prompt.
- Stage prompts `01–09` — NOT modified.
- `templates/` — no changes.
- `scripts/` — no changes.

### Triage class: `prompt`

Updating one existing prompt file (`prompts/00-session-start.md`). Class is `prompt`.
4-step loop with PROFILE-3 review cadence (downstream-facing).

### Scope axis: `self-dev only`

No changes to `dba-system.md`. Toolkit prompt files only.

### Review profile: PROFILE-3

Prompt class, downstream-facing. Codex review before each step gate; human approval at
all four gates; reviewer output is advisory and non-gatekeeping.

### Originating backlog item

`backlog/UPG-0012-repair-before-next-feature.md` — Repair-Before-Next-Feature Workflow Gate.

---

## Step 2 — Acceptance Criteria

### AC-1: Rule is present in `prompts/00-session-start.md`

A named "Repair-Before-Next-Feature" rule (or equivalent heading) is added to
`prompts/00-session-start.md`. The rule must be locatable by a practitioner reading
the session-start prompt.

Verification: `grep -n "Repair-Before\|repair-before\|repair before" prompts/00-session-start.md`
returns a hit.

### AC-2: All seven blocking conditions are named

The rule lists all seven conditions that block starting a new behavioral feature:

1. Unresolved Stage 7 GAP / MISMATCH / MISSING
2. Stage 8 replay failure
3. Required Stage 9 refinement not yet done
4. Stage 10 structural blocker
5. Failing CI
6. Unresolved reviewer BLOCK
7. Unresolved pre-release blocker

Verification: all seven conditions are present in the rule text.

### AC-3: Human-override clause is present

The rule must include an explicit human-override clause stating:
- The human may suspend a feature and start another.
- The suspended feature must be marked as blocked / incomplete.
- The evidence chain for the suspended feature is not silently abandoned.

Verification: override clause is present; suspended-feature marking requirement is stated.

### AC-4: Routing guidance is present for each issue type

The rule provides routing guidance:
- Behavioral issue (Stage 7/8/9) → Stage 9 targeted refinement or rerun the affected
  earlier stage.
- Structural issue → Stage 10 architectural refinement.
- Release / package issue → Readiness checklist / resolve the release blocker.

Verification: all three routing paths are named.

### AC-5: Rule applies only to new behavioral features, not non-behavioral sessions

The rule must not block non-behavioral work:
- Backlog / planning sessions
- Documentation-only sessions
- Advisory / discovery sessions (e.g., Solution Discovery)

The constraint is scoped to starting a new **behavioral feature** (i.e., entering Stage 1
Intent for a new feature). Sessions that continue or review an existing feature in progress
are also not blocked by this rule.

Verification: rule text specifies "new behavioral feature" or equivalent scoping; no
language prevents non-behavioral sessions from proceeding.

### AC-6: Rule preserves human authority — it is a check, not an auto-block

The rule instructs practitioners to check and surface the blocking condition; it does not
auto-refuse to proceed. The human decides whether to repair first or exercise the override.
Advisory framing is used, not imperative enforcement.

Verification: rule text uses advisory language ("check", "before starting", "if any of the
following are unresolved") rather than hard refusal language.

### AC-7: Rule is placed at Step 3 (Current Verified State) in the session-start flow

The rule is added to or immediately after Step 3 of `00-session-start.md` — the step
where the practitioner already assesses in-flight work — not at a later step where the
context has shifted.

Verification: `grep -n "Step 3\|Current Verified State" prompts/00-session-start.md` shows
the rule lands near Step 3.

### AC-8: Out-of-scope files are unchanged

`dba-system.md`, `CLAUDE.md`, stage prompts 01–09, `templates/`, and `scripts/` are not
modified.

Verification: none of these paths are in the `What changes` table and none appear in the
Step 3 implementation diff (tracked changes via `git diff HEAD`, untracked via
`git status --short`).

---

## Step 3 — Implementation

### `prompts/00-session-start.md` (UPDATED)

Rule inserted between the `CURRENT STATE VERIFIED` statement and `Then proceed to Step 4.`
— after the practitioner has assessed current state, before session-type selection. Design
decisions:

- **Heading:** "Repair-Before-Next-Feature check:" — matches AC-1 grep target.
- **Seven blocking conditions** listed verbatim from the backlog brief and AC-2 — all
  present by exact name.
- **Scope guard:** "if the intent is to start a **new behavioral feature**" — satisfies
  AC-5; exploratory sessions, planning, and docs work are not blocked.
- **Routing guidance** covers all three paths (AC-4): behavioral → Stage 9 / rerun;
  structural → Stage 10; release → readiness checklist.
- **Advisory framing** ("check", "surface this to the human", "if any of the above are
  unresolved") — not a hard refusal; satisfies AC-6 (human authority preserved).
- **Human-override clause** explicit: suspend is allowed; suspended feature must be marked
  blocked/incomplete — satisfies AC-3.
- **Placement:** immediately before "Then proceed to Step 4." inside Step 3 — satisfies AC-7.
- `dba-system.md`, `CLAUDE.md`, stage prompts 01–09, `templates/`, `scripts/` not touched
  — satisfies AC-8.

---

## Step 4 — Reconcile

### AC Verification

Note on AC-8: `git diff HEAD -- <path>` confirms no tracked file modified; `git status --short -- <path>`
confirms no untracked file created in those paths.

| AC | Verification | Result |
|---|---|---|
| AC-1 | `grep -n "Repair-Before" prompts/00-session-start.md` → hit at line 61 | PASS |
| AC-2 | All seven conditions present: Stage 7 GAP/MISMATCH/MISSING, Stage 8 replay failure, Stage 9 refinement, Stage 10 structural blocker, Failing CI, reviewer BLOCK, pre-release blocker → `wc -l` → 7 | PASS |
| AC-3 | "Human override:" clause at line 78; "suspended feature must be marked blocked / incomplete" at line 79 | PASS |
| AC-4 | Three routing paths present: behavioral → Stage 9 (line 74); structural → Stage 10 (line 75); release → readiness checklist (line 76) | PASS |
| AC-5 | "the intent is to start a **new behavioral feature**" at line 72 — non-behavioral sessions not mentioned as blocked | PASS |
| AC-6 | "check whether", "surface this to the human before proceeding" at lines 61/73 — advisory framing, no hard refusal | PASS |
| AC-7 | Rule at line 61 (inside Step 3 block); "Then proceed to Step 4." at line 81 — rule sits between state verification and Step 4 | PASS |
| AC-8 | `git diff HEAD -- dba-system.md CLAUDE.md \| wc -l` → 0; `git diff HEAD -- prompts/01-intent.md templates/ scripts/ \| wc -l` → 0; `git status --short -- prompts/01-intent.md templates/ scripts/ \| wc -l` → 0 | PASS |

### Cross-reference sweep

| Reference | Target | Status |
|---|---|---|
| Change record → backlog brief | `backlog/UPG-0012-repair-before-next-feature.md` | OK |
| Change record → updated prompt | `prompts/00-session-start.md` | exists |
| backlog brief Feature Thread | CHG-20260701-004 row present, state IN_PROGRESS | OK |
| `status/self-development.md` UPG-0012 row | IN_PROGRESS → will mark COMPLETE on acceptance | OK |
| `status/roadmap.md` Wave 3 UPG-0012 row | IN_PROGRESS → will mark COMPLETE on acceptance | OK |

No stage-table↔prompt-file drift. No orphaned links. `dba-system.md` intentionally not
updated (scope boundary declared in Step 1).

### Reviewer scope triage

| Finding | Round | Triage | Disposition |
|---|---|---|---|
| Problem statement used present tense claiming no rule exists — false in same packet | Step 3 R1 | IN-SCOPE BLOCKER | Fixed: reframed to past tense ("Before this change…") |
| AC-5 listed Session Types A–E as unblocked — contradicts gating new feature (Type A) work | Step 3 R1 | IN-SCOPE BLOCKER | Fixed: replaced with non-behavioral session categories; removed session-type reference |
