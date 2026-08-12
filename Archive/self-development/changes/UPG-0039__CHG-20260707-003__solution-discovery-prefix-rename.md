---
change_id: CHG-20260707-003
feature_id: UPG-0039
slug: solution-discovery-prefix-rename
triage_class: downstream-doctrine
scope_axis: downstream doctrine only
review_profile: PROFILE-4
review_series: RVS__UPG-0039__CHG-20260707-003__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0039 / CHG-20260707-003 — Resolve the 00b Prompt-Filename Collision (Discovery -> 00a)

## TRACE HEADER

```yaml
feature_id: UPG-0039
primary_feature_id: UPG-0039
change_id: CHG-20260707-003
slug: solution-discovery-prefix-rename
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0039
related_features:
  - UPG-0007
  - UPG-0037
review_series: RVS__UPG-0039__CHG-20260707-003__S1
review_profile: PROFILE-4
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

`prompts/00b-feature-brief.md` and `prompts/00b-solution-discovery.md` share the `00b`
prefix despite being distinct, sequential pre-Stage-1 steps — Solution Discovery (Session
Type E) precedes Feature Brief (Session Type A) in the actual workflow, per UPG-0007's own
design. UPG-0037 explicitly left this untouched ("no prompt file renames" — it added
reviewer-invocation identifier vocabulary only, not filenames).

### What changes

| File | Change |
|---|---|
| `prompts/00b-solution-discovery.md` → `prompts/00a-solution-discovery.md` | Renamed (`git mv`); no internal content change — the file never self-references its own filename |
| `dba-system.md` | Stage ID table's File column updated to the new path (currently line 164 — shifted from the backlog brief's originally-noted line 153 due to UPG-0037's later insertions; re-confirmed via fresh grep before drafting this Step 1) |
| `prompts/00-session-start.md` | Session Type E's "Prompt to load" reference updated (line 114) |
| `backlog/UPG-0007-solution-discovery-00b.md` | Its own "Proposed artifact(s)" section's one internal path mention (line 37) updated — resolving the backlog brief's own flagged "open Step 1 decision" as **yes**: the brief's *content* should stay accurate; its *filename* is not renamed (feature id is the permanent identity, not the slug) |
| `backlog/UPG-0039-solution-discovery-prefix-rename.md` | Feature Thread: CHG-20260707-003 activated (done) |
| `backlog/features.md` | Row → IN_PROGRESS (done) |
| `status/self-development.md` | Row activated (done) |
| `status/roadmap.md` | UPG-0039 → IN_PROGRESS (done) |

### Scope boundary — what stays the same

- **No behavior change.** This is a rename plus reference updates only — no doctrine
  substance, no stage semantics, no reviewer behavior changes.
- **Historical/append-only records are never touched**, confirmed by a fresh repo-wide grep
  for `00b-solution-discovery` immediately before drafting this Step 1 (re-verifying the
  backlog brief's own list is still accurate, since time has passed since it was written):
  - `changes/UPG-0007__CHG-20260630-005__solution-discovery-00b.md` (and its filename/slug)
  - `changes/UPG-0008__CHG-20260701-006__config-discovery.md`
  - `changes/UPG-0022__CHG-20260703-004__00b-adr-generator.md`
  - `changes/UPG-0037__CHG-20260705-002__downstream-default-stage-review.md`
  - `backlog/UPG-0008-config-discovery.md`'s completed Feature Thread "Changes" row
    (describes what `CHG-20260701-006` actually touched — the file *was* named
    `00b-solution-discovery.md` at that time; rewriting this row would misdescribe history)
  - `backlog/UPG-0037-downstream-default-stage-review.md`'s own "Proposed artifact(s)"
    description of what `CHG-20260705-002` touched (same reasoning — historical, accurate to
    the name at the time; not previously listed in the backlog brief, found by this Step 1's
    fresh grep sweep, confirmed historical by the same logic)
  - All `reviews/codex/*.md` frozen assessments and `reviews/review-log.md` entries
  - This change's own backlog brief's **Problem** section (describes the collision as it
    existed; stays as historical description, not rewritten to use the new name)
- **`backlog/UPG-0007-solution-discovery-00b.md`'s filename is not renamed** — only its line
  37 content. Backlog filenames are stable historical identifiers referenced by dozens of
  other files; the `UPG-####` feature id is the permanent identity, not the slug.
- **No `CLAUDE.md` or self-dev-governance changes.**
- **Downstream compatibility confirmed** (re-checked fresh, not just trusting the backlog
  brief): `grep -rl "00b-solution-discovery" /home/rimo/projects/FundFlow` returns nothing —
  no downstream-project-side edit needed. `.codeos` is a live symlink, so the rename is
  immediately live for FundFlow on its next session, same posture as UPG-0036/UPG-0037.

### Design intent

Straightforward rename + reference sweep, exactly as the backlog brief specifies, with the
open Step-1 question it flagged now resolved (update `backlog/UPG-0007`'s content, don't
rename its file) and the historical-file list re-verified fresh rather than assumed stale.

### Triage

- Class: `downstream-doctrine`
- Scope axis: `downstream doctrine only`
- Review profile: `PROFILE-4`
- Originating backlog id: `UPG-0039`

---

## Step 2 — Acceptance Criteria

### Rename correctness

**AC-1 — File renamed, content preserved exactly**
`prompts/00a-solution-discovery.md` exists with content byte-identical to the old
`prompts/00b-solution-discovery.md`; `prompts/00b-solution-discovery.md` no longer exists.
_Verify in Step 4:_ `git status`/`git diff` shows a detected rename (or equivalent
delete+add with identical content); no content-level diff.

**AC-2 — `prompts/00b-feature-brief.md` untouched**
The sibling file that originally created the `00b` collision is unmodified — this change
resolves the collision by moving Discovery, not by touching Feature Brief.
_Verify in Step 4:_ `git diff --stat -- prompts/00b-feature-brief.md` is empty.

### Live reference updates

**AC-3 — `dba-system.md`'s Stage table updated**
The Stage ID table's File column for Solution Discovery reads
`.codeos/prompts/00a-solution-discovery.md`.
_Verify in Step 4:_ grep the current line; confirm the new path.

**AC-4 — `prompts/00-session-start.md` updated**
Session Type E's "Prompt to load" line references the new path.
_Verify in Step 4:_ grep the current line; confirm the new path.

**AC-5 — `backlog/UPG-0007-solution-discovery-00b.md`'s content updated, filename unchanged**
Line 37's path mention updates to the new name; the file's own filename
(`UPG-0007-solution-discovery-00b.md`) is **not** renamed.
_Verify in Step 4:_ grep the updated line; confirm the filename on disk is unchanged.

### Historical-record integrity (no rewrite)

**AC-6 — Every historical file is unmodified in its existing content, verified by fresh
grep, not assumed**
None of the following has any of its *existing* content rewritten:
`changes/UPG-0007__CHG-20260630-005__solution-discovery-00b.md`,
`changes/UPG-0008__CHG-20260701-006__config-discovery.md`,
`changes/UPG-0022__CHG-20260703-004__00b-adr-generator.md`,
`changes/UPG-0037__CHG-20260705-002__downstream-default-stage-review.md`,
`backlog/UPG-0008-config-discovery.md`, `backlog/UPG-0037-downstream-default-stage-review.md`,
any `reviews/codex/*.md`, or this change's own backlog brief's Problem section. `git diff
--stat` for each of these is empty — no exceptions. `reviews/review-log.md` is the one
explicit exception to "no diff at all": as an append-only log, it is expected to gain *new*
entries from this same change's own review rounds; the AC for that file is narrower —
no existing line is altered or deleted, only new lines appended.
_Verify in Step 4:_ `git diff --stat` for every listed path except `reviews/review-log.md` is
empty; for `reviews/review-log.md`, confirm the diff is purely additive (only `+` lines, no
`-` lines removing or changing prior content). A fresh repo-wide grep
for the old filename after the change shows hits **only** in this explicitly-historical set
(plus this change's own Problem-section description and change-record narrative) — no
unlisted live file still references the old name.

### Downstream compatibility

**AC-7 — FundFlow's symlink resolves the rename correctly**
From FundFlow's real `.codeos` symlink checkout, `.codeos/prompts/00a-solution-discovery.md`
resolves and is readable; `.codeos/prompts/00b-solution-discovery.md` no longer resolves.
_Verify in Step 4:_ `cat`/`ls` both paths through FundFlow's live symlink.

**AC-8 — No downstream-project-side edit needed**
Re-confirms the Step 1 finding: no file under `/home/rimo/projects/FundFlow` hardcodes the
old path.
_Verify in Step 4:_ `grep -rl "00b-solution-discovery" /home/rimo/projects/FundFlow` → empty.

### Cross-reference integrity

**AC-9 — `CLAUDE.md` untouched**
_Verify in Step 4:_ `git diff --stat -- CLAUDE.md` is empty.

**AC-10 — No behavior change**
No stage semantics, reviewer behavior, or artifact schema changes — this is a rename plus
reference updates only, matching the backlog's own Guardrail.
_Verify in Step 4:_ confirm no file outside the declared "What changes" list is touched.

**AC-11 — Reviewer scope-triage applied at Step 4**
Per `CLAUDE.md`'s `downstream-doctrine` rigor requirement, Step 4 classifies every Step 3
review finding using the five-category scope-triage.
_Verify in Step 4:_ the Step 4 "Reviewer scope triage" section explicitly classifies each
finding from every round.

---

## Step 3 — Implement

### What was done

| File | Change |
|---|---|
| `prompts/00b-solution-discovery.md` → `prompts/00a-solution-discovery.md` | Renamed via `git mv`; content unchanged |
| `dba-system.md` | Stage table's File column updated |
| `prompts/00-session-start.md` | Session Type E's "Prompt to load" line updated |
| `backlog/UPG-0007-solution-discovery-00b.md` | Line 37 updated to the new path (with a short note on why/when); filename unchanged |

### Verification (AC-1 through AC-11)

Every check below was run directly against the real filesystem/git state by the author (not
inferred from a reviewer packet) — `git status`, `git diff --stat`, `grep`, and `cat`
invocations, no mocks. Separately, the Step 3 Codex review's own packet reported
`SECRET_REDACTION` coverage for this round: both `prompts/00a-solution-discovery.md` and
`backlog/UPG-0007-solution-discovery-00b.md` contain a pre-existing, benign template field
literally named `Secret / non-secret:` (unchanged by this rename — present in both files
before this change too), which a content scanner flags as looking secret-like. This is the
same known structural limitation already accepted during `UPG-0037`'s close-out; it reduces
what the *reviewer* could see in this round, but does not affect the verification claims
below, which are the author's own direct checks, independent of the reviewer's packet
visibility.

- **AC-1**: `git status` shows a detected rename (`R`); new path exists, old path gone.
- **AC-2**: `git diff --stat -- prompts/00b-feature-brief.md` → empty.
- **AC-3/AC-4/AC-5**: fresh grep confirms all three live references now point to the new
  path; `backlog/UPG-0007-solution-discovery-00b.md`'s filename confirmed unchanged on disk.
- **AC-6**: fresh repo-wide grep for the old name after the change returns hits only in the
  explicitly-historical set (`changes/UPG-0007*`, `UPG-0008*`, `UPG-0022*`, `UPG-0037*`;
  `backlog/UPG-0008-config-discovery.md`; `backlog/UPG-0037-downstream-default-stage-
  review.md`; all `reviews/codex/*.md`; `reviews/review-log.md`), plus this change's own
  backlog brief (Problem section, describing the collision as it existed) and its own change
  record (narrative describing what was renamed from) — no unlisted live file. `git diff
  --stat` for every listed historical path confirmed empty except `reviews/review-log.md`,
  whose diff is purely additive (this change's own append-only decision-log entries).
- **AC-7**: `cat /home/rimo/projects/FundFlow/.codeos/prompts/00a-solution-discovery.md`
  resolves and prints real content; the old path through the same symlink correctly no
  longer resolves.
- **AC-8**: `grep -rl "00b-solution-discovery" /home/rimo/projects/FundFlow` → empty.
- **AC-9**: `git diff --stat -- CLAUDE.md` → empty.
- **AC-10**: `git status --short` matches Step 1's declared "What changes" list exactly —
  no undeclared file touched.
- **AC-11**: scope-triage will be applied formally at Step 4 to any findings across all
  rounds (procedural — see Step 4).

### Scope check

`git status --short` shows exactly the 4 declared content changes (1 rename + 3 reference
updates) plus the declared backlog/status bookkeeping — no other file touched.

---

## Step 4 — Reconcile

### Acceptance criteria verification (fresh evidence)

| AC | Verified by | Result |
|---|---|---|
| AC-1 Rename correctness | New path exists, old path gone, `git status` shows `R` | PASS |
| AC-2 Sibling untouched | `git diff --stat -- prompts/00b-feature-brief.md` empty | PASS |
| AC-3 `dba-system.md` updated | Line 164 references new path | PASS |
| AC-4 `00-session-start.md` updated | Line 114 references new path | PASS |
| AC-5 `UPG-0007` content updated, filename unchanged | Line 37 updated; filename confirmed on disk | PASS |
| AC-6 Historical files unmodified (content), append-only exception honored | 6 change-record/backlog paths empty diff; `review-log.md` diff purely additive (0 removed lines) | PASS |
| AC-7 FundFlow symlink resolves rename | `cat` through live symlink prints real content | PASS |
| AC-8 No FundFlow-side reference | `grep -rl` empty | PASS |
| AC-9 `CLAUDE.md` untouched | `git diff --stat` empty | PASS |
| AC-10 No undeclared file touched | `git status --short` matches declared list exactly | PASS |
| AC-11 Scope-triage applied | See below | PASS |

### Cross-reference sweep

- Fresh repo-wide grep for the old filename (re-run at Step 4) returns the same set as Step
  3: exclusively the historical set plus this change's own narrative — no drift since Step 3.
- No other doc (`docs/*.md`, other `prompts/*.md`) references the old path in a way requiring
  an update — swept and confirmed clean.

### Reviewer scope triage (Step 4 findings)

Step 1 R1 (NO OBJECTION): no findings. Step 2 R1 (REQUEST CHANGES) found one genuine
IN-SCOPE BLOCKER (TRACE HEADER `review_state` staleness — the recurring bug caught before on
other changes); fixed, R2 clean. Step 3 R1 (REQUEST CHANGES) found two genuine IN-SCOPE
BLOCKERs: an overclaiming verification-section heading given the packet's own
`SECRET_REDACTION` coverage, and AC-6's wording incorrectly forbidding
`reviews/review-log.md`'s expected append-only growth — both fixed. Step 3 R2 (CHANGES
ADVISED) surfaced no new substantive finding — the sole remaining item is the reviewer's own
coverage rule mechanically firing on `SECRET_REDACTION`, caused by a pre-existing, benign
`Secret / non-secret:` template field unchanged by this rename (the same structural
limitation accepted during `UPG-0037`'s close-out). Classified **REJECTED / structural
scanner false positive / not an in-scope blocker** by explicit human decision; no further
round run to chase `NO OBJECTION`, consistent with the prior instruction not to use
`--sha-only` merely to obtain a cleaner packet.

### Outcome

All 11 ACs verified against the final artifacts with fresh evidence (table above), including
a live FundFlow symlink confirmation. No in-scope blockers open. No scope drift — no
historical record rewritten, `CLAUDE.md` untouched, sibling `00b-feature-brief.md` untouched.
The only residual item across Step 3 R2 and Step 4 R1 was the same known structural
`SECRET_REDACTION` coverage limitation (a pre-existing, benign `Secret / non-secret:`
template field in both touched files, unrelated to this rename, already accepted during
`UPG-0037`'s close-out) — classified REJECTED / structural scanner false positive / not an
in-scope blocker by explicit human decision both times. Human APPROVE_STAGE recorded
(2026-07-07). Change record, `status/self-development.md`, `status/roadmap.md`,
`backlog/features.md`, and `backlog/UPG-0039-solution-discovery-prefix-rename.md` updated to
COMPLETE in this same pass, following that approval.
