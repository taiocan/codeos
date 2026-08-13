---
change_id: CHG-20260706-003
feature_id: UPG-0025
slug: reviewer-verification-packet
triage_class: downstream-doctrine
scope_axis: downstream doctrine only
review_profile: PROFILE-4
review_series: RVS__UPG-0025__CHG-20260706-003__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0025 / CHG-20260706-003 — Verification Packet for Reviewer Agent

## TRACE HEADER

```yaml
feature_id: UPG-0025
primary_feature_id: UPG-0025
change_id: CHG-20260706-003
slug: reviewer-verification-packet
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0025
related_features:
  - UPG-0003
  - UPG-0010
  - UPG-0037
review_series: RVS__UPG-0025__CHG-20260706-003__S1
review_profile: PROFILE-4
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

The advisory reviewer (`codeos-reviewer`) already emits a `HIGHEST-IMPACT UNCERTAINTY:` line
on every single review round (mandated by `prompts/codeos-reviewer-task.md`'s output format).
Separately, `prompts/verify-only.md` already implements a full read-only verification mode
(no-edit rule, before/after anti-blur `git status`/`git diff --exit-code` checks, a structured
Verification-Only Report) — its own "Where to Use This Mode" table already names "Reviewer
agent — independent evidence" as an intended use. But **nothing connects the two**:
`verify-only.md` is referenced from no other file in the repo (confirmed:
`grep -rl "verify-only"` across `dba-system.md`, `docs/`, `prompts/` returns nothing), and
`dba-system.md`'s "Default Advisory Review" section (added by UPG-0037) never mentions
verification at all. This exact ad-hoc pattern — re-running a review with more read-only
evidence shown after the reviewer names a specific, checkable uncertainty — has been used
manually, repeatedly, this session (e.g. UPG-0019's Step 3/4 rounds, UPG-0024's Step 2 rounds)
without ever being written down as a named practice.

### What changes

| File | Change |
|---|---|
| `dba-system.md` | New subsection under "## Default Advisory Review": the verification round-trip — when a reviewer's `HIGHEST-IMPACT UNCERTAINTY` names something mechanically checkable, the acting agent may run `verify-only.md` targeting exactly that uncertainty, then feed its report back into the next review round. |
| `docs/reviewer-pipeline.md` | New "## 13. Verification round-trip" section, the self-dev-facing description of the same practice (parallel to how §12 documents downstream usage of the shared review mechanism). |
| `prompts/verify-only.md` | One-line cross-reference added to the existing "Reviewer agent — independent evidence" table row, pointing at the newly-documented workflow, so the file is no longer a dead end. |
| `backlog/UPG-0025-reviewer-verification-packet.md` | Feature Thread: CHG-20260706-003 activated (done) |
| `backlog/features.md` | Row → IN_PROGRESS (done) |
| `status/self-development.md` | Row activated (done) |
| `status/roadmap.md` | UPG-0025 → IN_PROGRESS (done) |

### Scope boundary — what stays the same

- **No new code.** Both mechanisms already exist and work today: the reviewer's uncertainty
  line (`prompts/codeos-reviewer-task.md`, unchanged) and the read-only verification mode
  (`prompts/verify-only.md`, changed only by one cross-reference sentence). This change
  documents how they connect; it builds no new subcommand, no new CLI flag, no new automated
  trigger.
- **No automation of the trigger.** Per the human's explicit direction: deciding whether an
  uncertainty is "mechanically checkable" and worth a verification pass stays a
  practitioner/agent judgment call, exactly like `verify-only.md`'s existing "optional and
  practitioner-loaded" framing — this change does not add code that parses a
  `HIGHEST-IMPACT UNCERTAINTY:` line and auto-triggers a shell command.
- **Non-Negotiable Rule #1 untouched.** A verification pass produces evidence for the human's
  decision; it never substitutes for the human's approval at the gate, exactly as
  `verify-only.md` already states ("Not a mandatory prerequisite... an evidence-quality
  tool, not a gate") and as UPG-0037's Review Waiver language already establishes for the
  parent Default Advisory Review practice.
- **No change to the round-budget table** (`docs/reviewer-pipeline.md` §4d, or
  `dba-system.md`'s flat R1-R3 downstream cadence). A verification pass is not itself a Codex
  review round — it is a read-only evidence-gathering step that happens *between* rounds; only
  the review round that follows it (informed by the new evidence) counts against the existing
  budget. This change states that explicitly rather than leaving it ambiguous.
- **No change to `prompts/codeos-reviewer-task.md`** — it already asks for the
  `HIGHEST-IMPACT UNCERTAINTY:` line; nothing about the reviewer's own output format changes.
- **`CLAUDE.md` / self-dev governance — not touched.** This mirrors UPG-0037's precedent
  exactly: one `downstream-doctrine`-classed change covering `dba-system.md` plus its
  supporting self-dev toolkit docs (`docs/reviewer-pipeline.md`), because neither touches
  `CLAUDE.md` or the self-dev loop itself.

### Design intent

**In `dba-system.md`**, under "## Default Advisory Review," add a subsection (placed after
"Review Waiver," before "Relationship to the Reviewer Activation Package" — a refinement of
how review plays out in practice, alongside the waiver, distinct from the "when review
applies" material above it):

> **Verification round-trip.** When a reviewer's assessment names a `HIGHEST-IMPACT
> UNCERTAINTY` that is mechanically checkable — a specific file, command, or repository state
> that can directly confirm or refute it — the acting agent may run `.codeos/prompts/
> verify-only.md`'s read-only verification pass targeting exactly that uncertainty, then
> attach its Verification-Only Report as evidence for the next review round. This is optional
> and judged by the acting agent, not automatic or mandatory — not every uncertainty is
> mechanically checkable, and declining to run it is always a valid choice. Verification
> remains strictly read-only (see `verify-only.md`'s No-Edit Rule); it produces evidence for
> the human's decision, it does not replace it. A verification pass is not itself a review
> round and does not consume the round budget above — only the review round it feeds into
> does.

**In `docs/reviewer-pipeline.md`**, add "## 13. Verification round-trip" (after §12, before
Appendix A) — the self-dev-facing mirror of the same paragraph, referencing the concrete
mechanics already used this session (e.g., re-running `codeos-review.sh review ...` with
additional files shown after a prior round's `HIGHEST-IMPACT UNCERTAINTY` named something
unverified, as happened for UPG-0019 Step 3 and UPG-0024 Step 2), and cross-referencing
`dba-system.md`'s parallel downstream-facing wording so the two never drift apart.

**In `prompts/verify-only.md`**, extend the existing row:
```
| Reviewer agent — independent evidence | Reviewer reads exact results without needing to
  re-run — see dba-system.md's "Verification round-trip" for when/how this connects to a
  review round. |
```

### Downstream-compatibility

- No artifact path, filename, schema, or stage identifier changes — this is additive prose
  only.
- A generated downstream project's `.codeos/dba-system.md` and `.codeos/prompts/
  verify-only.md` (both loaded via the live symlink) pick up this addition immediately, same
  as every prior `downstream-doctrine` change this session (UPG-0037). No version/sync step
  exists or is needed.
- `verify-only.md`'s existing content (No-Edit Rule, anti-blur checks, Verification Report
  template) is unchanged except for the one added cross-reference sentence in its usage
  table — every existing invocation of that mode still behaves identically.

### Triage

- Class: `downstream-doctrine`
- Scope axis: `downstream doctrine only`
- Review profile: `PROFILE-4`
- Originating backlog id: `UPG-0025`

---

## Step 2 — Acceptance Criteria

### Content

**AC-1 — `dba-system.md`'s new subsection present, correctly placed**
Under "## Default Advisory Review," a "**Verification round-trip.**" subsection exists,
placed after "Review Waiver" and before "Relationship to the Reviewer Activation Package."
Its content preserves, in substance (not necessarily verbatim), all of: (a) the trigger is
the reviewer's `HIGHEST-IMPACT UNCERTAINTY` line naming something mechanically checkable;
(b) running it is optional, judged by the acting agent, never automatic or mandatory; (c)
the mechanism is `verify-only.md`, strictly read-only; (d) the result is evidence for the
human's decision, not a replacement for it; (e) a verification pass does not itself consume
the round budget — only the review round it feeds into does.
_Verify in Step 4:_ read the subsection; confirm placement and all five substantive points
are present.

**AC-2 — `docs/reviewer-pipeline.md`'s new §13 present, correctly placed**
"## 13. Verification round-trip" exists after §12 and before "Appendix A," describing the
same practice in self-dev's own terms, cross-referencing `dba-system.md`'s parallel wording.
_Verify in Step 4:_ confirm section placement via `grep -n "^## "` ordering; confirm the
cross-reference to `dba-system.md` is present.

**AC-3 — `prompts/verify-only.md`'s table row extended, nothing else changed**
The "Reviewer agent — independent evidence" row in the "Where to Use This Mode" table gains
the cross-reference sentence. No other line in the file changes — the No-Edit Rule,
Anti-Blur Checks, and Verification Report template are byte-identical to before this change.
_Verify in Step 4:_ `git diff -- prompts/verify-only.md` shows only the one table-row line
changed.

**AC-4 — The two new sections are mutually consistent**
`dba-system.md`'s subsection and `docs/reviewer-pipeline.md`'s §13 agree on all five
substantive points from AC-1 — neither contradicts the other on whether verification is
mandatory, whether it consumes the round budget, or what mechanism it uses.
_Verify in Step 4:_ read both side by side; confirm no contradiction.

### No-regression guarantees

**AC-5 — No round-budget table change**
`docs/reviewer-pipeline.md`'s "## 4d. Review-round budget table" and `dba-system.md`'s flat
R1-R3 downstream cadence description (in "Default Advisory Review," above the new
subsection) are textually unchanged by this diff.
_Verify in Step 4:_ `git diff` restricted to those specific line ranges shows no
modification.

**AC-6 — Non-Negotiable Rule #1 unweakened**
`dba-system.md`'s "## The Non-Negotiable Rules" section is untouched by this diff, and no
new prose anywhere in this change states or implies that a verification pass can substitute
for, defer, or weaken the human-approval gate.
_Verify in Step 4:_ `git diff --stat` restricted to that section is empty;
`grep -in "replaces.*approval\|substitutes.*gate"` across the new prose returns nothing.

**AC-7 — `prompts/codeos-reviewer-task.md` untouched**
The reviewer's own output-format instructions (including the existing `HIGHEST-IMPACT
UNCERTAINTY:` line) are unmodified — this change documents a response to that line, not a
change to how it's produced.
_Verify in Step 4:_ `git diff --stat -- prompts/codeos-reviewer-task.md` is empty.

**AC-8 — No new code**
No file under `tools/reviewer/src/` changes; no new CLI flag or subcommand is added to
`main.rs`.
_Verify in Step 4:_ `git diff --stat -- tools/reviewer/` is empty.

**AC-9 — `CLAUDE.md` untouched**
Confirms the `downstream doctrine only` scope axis declaration.
_Verify in Step 4:_ `git diff --stat -- CLAUDE.md` is empty.

### Downstream-compatibility

**AC-10 — No path, filename, or schema changes**
No file is renamed or moved; `.codeos/dba-system.md` and `.codeos/prompts/verify-only.md`
resolve through the existing symlink to the same paths as before this change, now carrying
additive content only.
_Verify in Step 4:_ `git diff --stat` for this change shows only content modifications to
existing files, zero renames/moves; from a symlinked downstream checkout (or a simulated
equivalent), confirm `.codeos/dba-system.md` and `.codeos/prompts/verify-only.md` still
resolve and are readable.

### Cross-reference integrity

**AC-11 — Reviewer scope-triage applied at Step 4**
Per `CLAUDE.md`'s `downstream-doctrine` rigor requirement, Step 4 classifies every Step 3
review finding using the five-category scope-triage (IN-SCOPE BLOCKER / IN-SCOPE NON-BLOCKER
/ OUT-OF-SCOPE BACKLOG / REJECTED / SELF-REFERENCE), not just a flat accept/reject.
_Verify in Step 4:_ the Step 4 "Reviewer scope triage" section explicitly classifies each
finding from every round.

---

## Step 3 — Implement

### What was done

| File | Change |
|---|---|
| `dba-system.md` | Added "**Verification round-trip.**" paragraph under "## Default Advisory Review," between "Review Waiver" and "Relationship to the Reviewer Activation Package" |
| `docs/reviewer-pipeline.md` | Added "## 13. Verification round-trip" section, after §12, before Appendix A |
| `prompts/verify-only.md` | Extended the "Reviewer agent — independent evidence" table row with a cross-reference sentence; single-line diff |

### Verification (AC-1 through AC-11)

- **AC-1**: `grep -n` confirms the new paragraph sits between "Review Waiver" (line 71) and
  "Relationship to the Reviewer Activation Package" (line 91); read content confirms all 5
  substantive points present (trigger, optional/judged, read-only mechanism, evidence-not-
  replacement, doesn't consume round budget).
- **AC-2**: `grep -n "^## "` confirms §13 sits between §12 (line 339) and Appendix A (line
  378, was 378 pre-edit).
- **AC-3**: `git diff -- prompts/verify-only.md` shows exactly one changed line — the table
  row — nothing else in the file touched.
- **AC-4**: read both new sections side by side; both state the same 5 points (optional,
  judged by the acting agent, read-only, evidence not decision, doesn't consume the round
  budget) with no contradiction.
- **AC-5**: `git diff` restricted to `docs/reviewer-pipeline.md`'s §4d region and
  `dba-system.md`'s round-budget prose (above the new paragraph) shows no hits — confirmed by
  inspecting the single diff hunk's line range (77-91 in `dba-system.md`; the new §13 is a
  pure addition after existing content in `reviewer-pipeline.md`, not a modification to §4d).
- **AC-6**: the single diff hunk in `dba-system.md` starts at line 77, well after "## The
  Non-Negotiable Rules" (lines 32-42) — that section is untouched;
  `grep -iE "replaces.*approval|substitutes.*gate"` across the new prose → empty.
- **AC-7**: `git diff --stat -- prompts/codeos-reviewer-task.md` → empty.
- **AC-8**: `git diff --stat -- tools/reviewer/` → empty.
- **AC-9**: `git diff --stat -- CLAUDE.md` → empty.
- **AC-10**: `git status --short` shows only `M` (modified) entries, zero renames; from
  FundFlow's real `.codeos` symlink checkout, both `.codeos/dba-system.md` and
  `.codeos/prompts/verify-only.md` resolve and are readable with the new content live.
- **AC-11**: this Step 3 round returned clean; scope-triage will be applied formally at Step
  4 to any findings across all rounds (procedural — see Step 4).

### Scope check

`git status --short` shows exactly the 3 declared content files plus the declared
backlog/status bookkeeping and review artifacts — no other file touched.

---

## Step 4 — Reconcile

### Acceptance criteria verification (fresh evidence)

| AC | Verified by | Result |
|---|---|---|
| AC-1 `dba-system.md` placement + content | `grep -n` confirms "Verification round-trip" sits between "Review Waiver" and "Relationship to the Reviewer Activation Package"; content re-read, all 5 substantive points present | PASS |
| AC-2 `docs/reviewer-pipeline.md` §13 placement | `grep -n "^## "` confirms §13 between §12 and Appendix A | PASS |
| AC-3 `verify-only.md` single-line diff | `git diff | grep -E "^[+-][^+-]"` shows exactly one removed, one added line | PASS |
| AC-4 Mutual consistency | Both sections re-read side by side; same 5 points, no contradiction | PASS |
| AC-5 Round-budget tables untouched | Diff hunks confirmed outside §4d/round-budget prose regions | PASS |
| AC-6 Non-Negotiable Rule #1 unweakened | `dba-system.md` diff hunk starts at line 77, after the Non-Negotiable Rules section (32-42); no weakening language found | PASS |
| AC-7 `codeos-reviewer-task.md` untouched | `git diff --stat` empty | PASS |
| AC-8 No new code | `git diff --stat -- tools/reviewer/` empty | PASS |
| AC-9 `CLAUDE.md` untouched | `git diff --stat -- CLAUDE.md` empty | PASS |
| AC-10 Downstream compatibility | `diff` between `dba-system.md` and FundFlow's live `.codeos/dba-system.md` symlink target — byte-identical, confirming real resolution | PASS |
| AC-11 Scope-triage applied | See below | PASS |

### Cross-reference sweep

- `git status --short` — only the 3 declared content files plus declared bookkeeping/review
  artifacts; no stray files.
- Re-confirmed the precheck's draft-marker warning (Step 3) is pre-existing content from
  2026-06-29 (`git blame`), not introduced by this diff.
- No other file in the repo references "Verification round-trip" in a conflicting way
  (swept `*.md` for the phrase — only the two new sections and this change record use it).

### Reviewer scope triage (Step 4 findings)

Step 1 R1 (NO OBJECTION): no findings. Step 2 R1 (NO OBJECTION): no findings. Step 3 R1 (NO
OBJECTION): no IN-SCOPE BLOCKER findings; the only reviewer-surfaced item was the precheck's
draft-marker warning, which is SELF-REFERENCE / REVIEW-BOOKKEEPING (a scanner artifact on
pre-existing, unrelated prose from 2026-06-29, not a defect in this artifact) — resolved by
human decision at the Step 3 gate, consistent with the precedent set for UPG-0037's
SECRET_REDACTION false positive. This Step 4 round: no findings.

### Outcome

All 11 ACs verified against the final artifacts with fresh evidence (table above), including
a live downstream symlink confirmation. No in-scope blockers open across any of the 4 gates.
No scope drift — no code, no `CLAUDE.md`, no round-budget table, no Non-Negotiable Rule
touched. Step 4 NO OBJECTION; human APPROVE_STAGE recorded (2026-07-07). Change record,
`status/self-development.md`, `status/roadmap.md`, `backlog/features.md`, and
`backlog/UPG-0025-reviewer-verification-packet.md` updated to COMPLETE in this same pass,
following that approval.
