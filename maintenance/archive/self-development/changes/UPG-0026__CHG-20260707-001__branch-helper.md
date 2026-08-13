---
change_id: CHG-20260707-001
feature_id: UPG-0026
slug: branch-helper
triage_class: documentation
scope_axis: self-dev only
review_profile: PROFILE-2
review_series: RVS__UPG-0026__CHG-20260707-001__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0026 / CHG-20260707-001 — Optional Branch Creation Helper

## TRACE HEADER

```yaml
feature_id: UPG-0026
primary_feature_id: UPG-0026
change_id: CHG-20260707-001
slug: branch-helper
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0026
related_features:
  - UPG-0016
  - UPG-0013
  - UPG-0009
review_series: RVS__UPG-0026__CHG-20260707-001__S1
review_profile: PROFILE-2
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

`docs/workflow-profiles.md` (UPG-0016) already documents Profile B's single-branch
convention (`feature/<feature_id>`, line 53) and an explicit non-requirement for automated
branch creation ("What Profile B Does Not Require... Automated branch creation," line 128).
But Profile C's split-PR table (4 PRs: Artifacts / Implementation / Runtime evidence /
Refinement) names what each PR *contains* and never names the branches backing them —
exactly the gap this backlog brief's split-mode proposal (`feature/<feature_id>-artifacts`,
`-implementation`, `-runtime-replay`, `-refinement`) fills. The backlog's own Guardrail
("Start with documentation only") and Design notes ("Alternative: no script. Just document
branch convention.") point at this precise, narrow gap rather than new tooling.

### What changes

| File | Change |
|---|---|
| `docs/workflow-profiles.md` | Profile C's PR table gains a branch-name column (or an adjoining note) naming the 4 split-mode branches; a parallel "What Profile C Does Not Require" note states automated branch creation remains not required, mirroring Profile B's existing guardrail |
| `backlog/UPG-0026-branch-helper.md` | Feature Thread: CHG-20260707-001 activated (done) |
| `backlog/features.md` | Row → IN_PROGRESS (done) |
| `status/self-development.md` | Row activated (done) |
| `status/roadmap.md` | UPG-0026 → IN_PROGRESS (done) |

### Scope boundary — what stays the same

- **No script, no new subcommand, no CLI flag.** The backlog's own Guardrail and Design
  notes explicitly offer "no script, just document" as the accepted alternative; Profile B's
  existing "Automated branch creation... create the branch manually with `git checkout -b`"
  non-requirement is the direct precedent this change extends to Profile C, not overrides.
- **Profile B's existing single-branch convention (`feature/<feature_id>`, line 53) is
  unmodified** — this change only adds Profile C's missing branch-naming note; it does not
  touch Profile B's section.
- **`dba-system.md` — not touched.** Same self-dev-only classification as UPG-0016 (the
  file this change directly extends); branch/PR discipline has never been part of the
  downstream doctrine loaded via `.codeos/dba-system.md`.
- **No existing subcommand, template, or `CLAUDE.md` content touched.**
- **The 4-PR content table itself (what each PR contains, its merge gate) is unchanged** —
  only branch names are added alongside it.

### Design intent

Extend Profile C's existing table (currently: `| PR | What it contains | Merge gate |`) with
a fourth column, `Branch`, populated with the backlog's proposed split-mode names:

| PR | What it contains | Merge gate | Branch |
|---|---|---|---|
| PR 1 — Artifacts | Intent, contract, event schema (Stages 1–3) | Stages 1–3 approved | `feature/<feature_id>-artifacts` |
| PR 2 — Implementation | Code, behavioral tests, telemetry tests (Stages 4–6) | Stages 4–6 approved + CI green | `feature/<feature_id>-implementation` |
| PR 3 — Runtime evidence | Sanitized fixtures, replay tests, reconciliation reports (Stages 7–8) | Stages 7–8 approved | `feature/<feature_id>-runtime-replay` |
| PR 4 — Refinement | Only if Stage 9 required a substantive change | Stage 9 approved | `feature/<feature_id>-refinement` |

Immediately after the table (alongside the existing "Notes" bullets), add a short
"What Profile C Does Not Require" note mirroring Profile B's existing one verbatim in spirit:
branch creation stays manual (`git checkout -b`); no automated helper is introduced by this
change.

### Triage

- Class: `documentation`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-2`
- Originating backlog id: `UPG-0026`

---

## Step 2 — Acceptance Criteria

**AC-1 — Branch column added, correct 4 values, in order**
Profile C's PR table gains a `Branch` column with exactly:
`feature/<feature_id>-artifacts`, `feature/<feature_id>-implementation`,
`feature/<feature_id>-runtime-replay`, `feature/<feature_id>-refinement`, aligned to PR 1-4
respectively, in that order.
_Verify in Step 4:_ read the table; confirm all 4 values present, correctly aligned to their
PR row.

**AC-2 — Existing table content unchanged**
The `PR`, `What it contains`, and `Merge gate` columns' existing text is byte-identical to
before this change — only a new column is added, nothing in the existing three is reworded.
_Verify in Step 4:_ `git diff` shows only column-addition lines, no modification to existing
cell text.

**AC-3 — "What Profile C Does Not Require" note present**
A short note (mirroring Profile B's existing "Automated branch creation... create the branch
manually with `git checkout -b`" line in substance) states that automated branch creation is
not required for Profile C either — no script or tool is introduced by this change.
_Verify in Step 4:_ read the note; confirm it states manual creation remains sufficient and
no automation is implied.

**AC-4 — Profile B's section untouched**
Profile B's existing single-branch convention (`feature/<feature_id>`, currently at line 53)
and its own "What Profile B Does Not Require" list are unmodified by this diff.
_Verify in Step 4:_ `git diff` shows no hunks touching Profile B's section.

### Cross-reference integrity

**AC-5 — No new content file created**
`docs/workflow-profiles.md` is the only *content* file modified, and no new content file is
added anywhere in the repo. (The declared backlog/status/roadmap bookkeeping files — listed
in Step 1's "What changes" table — are expected to change as part of every non-trivial
change's activation/close-out and are not "content" in this AC's sense; this AC's target is
specifically ruling out a new doc/script/tool file, which the backlog's own Guardrail and
Design notes explicitly reject as this change's alternative.)
_Verify in Step 4:_ `git status --short` for this change shows `M` (modified) only on
`docs/workflow-profiles.md` among content files, plus the declared bookkeeping files; no new
(`??`/`A`) content file anywhere.

**AC-6 — No `dba-system.md` change**
_Verify in Step 4:_ `git diff --stat -- dba-system.md` is empty.

**AC-7 — No `CLAUDE.md` change**
_Verify in Step 4:_ `git diff --stat -- CLAUDE.md` is empty.

**AC-8 — No code change**
_Verify in Step 4:_ `git diff --stat -- tools/reviewer/` is empty.

---

## Step 3 — Implement

### What was done

| File | Change |
|---|---|
| `docs/workflow-profiles.md` | Profile C's PR table gained a `Branch` column with the 4 split-mode names; a "What Profile C Does Not Require" note added after the existing Notes, mirroring Profile B's automated-branch-creation non-requirement |

### Verification (AC-1 through AC-8)

- **AC-1**: table now reads `feature/<feature_id>-artifacts` / `-implementation` /
  `-runtime-replay` / `-refinement` aligned to PR 1-4 respectively, in order.
- **AC-2**: `git diff` shows the `PR`/`What it contains`/`Merge gate` cell text unchanged —
  only new column cells added to each row.
- **AC-3**: "What Profile C Does Not Require" note added, stating branch creation stays
  manual (`git checkout -b`), no script/helper introduced.
- **AC-4**: `git diff` hunks confined to lines 60+ (Profile C's table) and 74+ (the new
  note) — Profile B's section (~40-56) has zero hunks touching it.
- **AC-5**: `git status --short` shows `M docs/workflow-profiles.md` as the only content
  file, plus the declared bookkeeping files (`backlog/UPG-0026-branch-helper.md`,
  `backlog/features.md`, `status/roadmap.md`, `status/self-development.md`,
  `reviews/review-log.md`) and this change record — no new content file.
- **AC-6**: `git diff --stat -- dba-system.md` → empty.
- **AC-7**: `git diff --stat -- CLAUDE.md` → empty.
- **AC-8**: `git diff --stat -- tools/reviewer/` → empty.

### Scope check

Only `docs/workflow-profiles.md` modified as content, plus declared bookkeeping — matches
Step 1's "What changes" table exactly.

---

## Step 4 — Reconcile

### Acceptance criteria verification (fresh evidence)

| AC | Verified by | Result |
|---|---|---|
| AC-1 Branch column, 4 correct values, in order | Read table fresh — all 4 present, aligned to PR 1-4 | PASS |
| AC-2 Existing table content unchanged | Diff shows only column additions to existing cells | PASS |
| AC-3 "Does Not Require" note present | Read note fresh — manual creation, no automation | PASS |
| AC-4 Profile B untouched | `git diff` hunks confined to lines 60+/74+, both within Profile C | PASS |
| AC-5 No new content file | `git status --short` — only `docs/workflow-profiles.md` as content, rest is declared bookkeeping | PASS |
| AC-6 `dba-system.md` untouched | `git diff --stat` empty | PASS |
| AC-7 `CLAUDE.md` untouched | `git diff --stat` empty | PASS |
| AC-8 No code change | `git diff --stat -- tools/reviewer/` empty | PASS |

### Cross-reference sweep

- No other doc in the repo references Profile C's branch names in a way that could now
  conflict (swept `docs/*.md` and `prompts/*.md` for "runtime-replay"/"artifacts\`" — only
  this change's new table row uses it).
- `docs/ci-integration-profile.md` (UPG-0019) does not reference branch names at all —
  no cross-file drift to reconcile.

### Reviewer scope triage (Step 4 findings)

Step 1 R1 (NO OBJECTION): no findings. Step 2 R1 (DO NOT ADVANCE) found one genuine
self-contradiction (AC-5's overly broad "nothing else changes" claim, contradicting the
declared bookkeeping files) — IN-SCOPE BLOCKER, fixed; R2 came back clean. Step 3 R1 (NO
OBJECTION): no findings. This Step 4 round: no findings.

### Outcome

All 8 ACs verified against the final artifacts with fresh evidence (table above). No
in-scope blockers open. No scope drift — Profile B, `dba-system.md`, `CLAUDE.md`, and all
existing code untouched. Step 4 NO OBJECTION; human APPROVE_STAGE recorded (2026-07-07).
Change record, `status/self-development.md`, `status/roadmap.md`, `backlog/features.md`, and
`backlog/UPG-0026-branch-helper.md` updated to COMPLETE in this same pass, following that
approval. This closes out all of Wave 5.
