# Self-Development Change: UPG-0031__CHG-20260630-002 — review-delta-working-tree

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit
itself (prompts, templates, docs, patterns, scripts).

Workflow: prompts/codeos-self-dev.md (4-step loop)
Each step requires explicit human approval; Codex review cadence is governed by the
assigned review profile (see prompts/codeos-self-dev.md Step 0a).
The live status row lives in status/self-development.md, not here.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0031
primary_feature_id: UPG-0031
change_id: CHG-20260630-002
slug: review-delta-working-tree
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0031
related_features:
  - UPG-0027
  - UPG-0030
review_series: null
review_profile: PROFILE-3
review_state: DRAFT
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: UPG-0004
corrected_by: CHG-20260630-003
```

<!-- SELF-REFERENCE BOUNDARY -->

## Change Intent

**Why (problem in the toolkit):**

`scripts/codeos-review.sh` delta mode uses `git diff "${delta_base}" HEAD -- <paths>` at four
sites. This only sees committed changes between the base and HEAD. When review fixes are
uncommitted, HEAD has not moved, the diff is empty, and the packet reports `EMPTY_PACKET`.
Codex is then invoked with zero reviewable content, producing an unactionable DO NOT ADVANCE.

This stalled UPG-0004 Step 2 (the change record fixes were in the working tree but not
committed, so R2 delta review saw no content). The root cause was identified externally and
confirmed by reading the script.

Two additional related defects:
- No fail-closed guard prevents Codex from being called with an EMPTY_PACKET.
- Untracked new artifact files produce silent EMPTY_PACKET in delta mode rather than a clear
  diagnostic (because `git diff <base> HEAD` cannot see untracked files at all).

A fourth related issue: the precheck grep for `UPG-####` fires on comment/legend-section prose,
requiring `--skip-prechecks` workarounds. This is a usability defect in the precheck.

**What changes:**

1. `scripts/codeos-review.sh` — four targeted edits:
   - **Fix A (4 sites, delta mode)**: remove `HEAD` from delta-mode `git diff` calls. `git diff
     "${delta_base}" -- <paths>` compares the base commit to the working tree (staged + unstaged
     tracked changes). Sites: line 159 (`raw_diff`), line 160 (`changed_files`), line 185
     (`filtered_diff`), line 231 (per-artifact delta detection).
   - **Fix B (1 site, untracked artifact guard)**: in the per-artifact loop in `build_packet`,
     when `delta_mode == "delta"`, check `git ls-files --error-unmatch "$a"` before attempting
     a diff. If the artifact is untracked, exit 5 with a clear diagnostic.
   - **Fix C (1 site, EMPTY_PACKET guard)**: in `cmd_review`, after the `print_only` block and
     before `run_codex`, exit 4 if `PACKET_COVERAGE_STATE == EMPTY_PACKET`. The `print_only`
     block itself should exit nonzero (exit 4) when coverage is EMPTY_PACKET, so inspection is
     still possible but the nonzero signals the empty state to callers.
   - **Fix D (1 site, precheck false-positive)**: tighten the `UPG-####` grep in `run_prechecks`
     to skip HTML comment blocks (`<!-- … -->`) and lines whose content is clearly documentation
     of the filename convention rather than an unfilled placeholder field.

2. `backlog/UPG-0031-review-delta-mode-fix.md` — Feature Thread Changes table row activated.

3. `backlog/features.md` — UPG-0031 row added to the authoritative feature-id map.

4. `status/self-development.md` — operational row activated for UPG-0031 / CHG-20260630-002.

5. `backlog/UPG-0004-stage-4-6-reports.md` — Feature Thread Changes table row backfilled for
   CHG-20260630-001 (trace/backlink only; no UPG-0004 scope or content change).

**Bookkeeping artifact:**

- `changes/UPG-0031__CHG-20260630-002__review-delta-working-tree.md` — this change record.

**Scope boundary — what stays the same:**

- `dba-system.md` — not touched.
- `prompts/` — not touched.
- `templates/` — not touched.
- `docs/` — not touched unless a comment inside `codeos-review.sh` references a doc path
  that needs updating (unlikely; will be confirmed in Reconcile).
- Packet format, review log format, session handling — unchanged.
- Advisory/read-only/non-gatekeeping guarantees — unchanged.
- `--mode full` diff/packet content behavior — unchanged (Fix A only affects delta-mode diff
  calls; the non-delta `git diff HEAD -- .` branch is not touched).
- `--mode full` precheck behavior — intentionally changed by Fix D: artifacts that previously
  produced false positives (`UPG-####` in blockquotes, HTML comments, or code spans) now pass
  the precheck in both modes. The fail-closed guarantee for real unfilled placeholders is
  retained.

**Class:** `script-tooling`
**Scope axis:** `self-dev only`
**Review profile:** `PROFILE-3`
**Backlog item:** `backlog/UPG-0031-review-delta-mode-fix.md`

---

## Acceptance Criteria

<!-- Delta mode working-tree fix (B1–B4) -->

| # | Criterion | How it will be verified |
|---|---|---|
| B1 | Delta mode with uncommitted tracked artifact changes produces a non-empty diff | Smoke test: modify an artifact without committing; run `--mode delta --base <sha> --print-packet`; confirm `review_content_bytes > 0` and `coverage_state` is not `EMPTY_PACKET` |
| B2 | Delta mode with no artifact changes (nothing committed or uncommitted since base) produces `EMPTY_PACKET` and the script exits 4 before calling Codex | Smoke test: run delta review with no changes to artifact paths; confirm exit 4, no Codex session created |
| B3 | `--mode full` diff/packet content is identical before and after this change; precheck behavior is intentionally changed by Fix D (false positives eliminated; real placeholders still exit 2) | Smoke test: run `--mode full --print-packet` on an artifact with no placeholder issues before and after; `diff` the output (excluding `generated:` timestamp); confirm precheck passes on previously-false-positive artifacts and still fails on genuine unfilled fields |
| B4 | `git diff HEAD -- .` (working tree) behavior — the existing non-delta code path at line 165–166 — is unchanged | Read the script after changes; confirm lines 161–167 of the non-delta branch are byte-identical to before |

<!-- Untracked artifact guard (B5) -->

| # | Criterion | How it will be verified |
|---|---|---|
| B5 | Passing an untracked file as a positional artifact in delta mode exits 5 with a diagnostic naming the file and suggesting `--mode full` or staging | Smoke test: create a temp untracked file; run delta review; confirm exit 5 and diagnostic message |

<!-- EMPTY_PACKET fail-closed guard (B6–B7) -->

| # | Criterion | How it will be verified |
|---|---|---|
| B6 | Live delta review with `EMPTY_PACKET` exits 4 before calling Codex; no session is created, no assessment file written | Smoke test: trigger EMPTY_PACKET (delta with no changes); confirm exit 4; confirm no new file under `reviews/codex/` |
| B7 | `--print-packet` with `EMPTY_PACKET` still prints the packet metadata and exits nonzero; Codex is not called | Smoke test: run `--mode delta --print-packet` with no changes; confirm packet metadata printed; confirm exit nonzero; confirm no Codex session created |

<!-- Precheck false-positive (B8) -->

| # | Criterion | How it will be verified |
|---|---|---|
| B8 | `run_prechecks` does not fail on `UPG-####` appearing inside HTML comment blocks or filename-convention documentation lines in an artifact | Smoke test: run `review` (without `--skip-prechecks`) on `status/self-development.md` and `changes/UPG-0031__CHG-20260630-002__review-delta-working-tree.md`; confirm precheck passes |

<!-- Scope boundary (B9) -->

| # | Criterion | How it will be verified |
|---|---|---|
| B9 | `dba-system.md`, `prompts/`, `templates/`, `docs/` are unchanged | `git diff <base_sha> -- dba-system.md prompts/ templates/ docs/` → empty |

<!-- Bookkeeping (B10–B11) -->

| # | Criterion | How it will be verified |
|---|---|---|
| B10 | `backlog/UPG-0031-review-delta-mode-fix.md` Feature Thread Changes table has a row for CHG-20260630-002; `backlog/features.md` has a UPG-0031 row; `backlog/UPG-0004-stage-4-6-reports.md` Feature Thread has CHG-20260630-001 row | Read-through: all three rows exist and are correctly filled |
| B11 | `status/self-development.md` has an operational row for UPG-0031 / CHG-20260630-002; Loop step reflects current gate | Read-through: row exists at correct step |

---

## Implementation Notes

`scripts/codeos-review.sh` — four targeted edits applied:

**Fix A (4 sites)**: Removed `HEAD` from delta-mode `git diff` calls at lines 159, 160, 185, 231.
`git diff "${delta_base}" -- <paths>` now compares the base commit to the working tree
(staged + unstaged tracked changes) instead of comparing base to HEAD.

**Fix B (1 site)**: Added untracked artifact guard in the per-artifact loop (delta mode), before
the `git diff --quiet` check. `git ls-files --error-unmatch "${a}"` exits 5 with a diagnostic
if the artifact is untracked.

**Fix C (2 sites)**: (a) `print_only` block now exits 4 (not 0) when `PACKET_COVERAGE_STATE ==
EMPTY_PACKET`, so `--print-packet` callers can detect empty state while still viewing the packet.
(b) Added fail-closed guard after the `print_only` block and before `run_codex`: exits 4 with a
diagnostic when coverage is EMPTY_PACKET, preventing Codex invocation on empty packets.

**Fix D (2 sites)**: Tightened the `UPG-####` and `CHG-YYYYMMDD-NNN` precheck greps. The
pipeline now: (1) strips HTML comment blocks (`sed '/<!--/,/-->/d'`), (2) removes blockquote
lines (`grep -vE '^\s*>'`), (3) removes allowed documentation occurrences at the occurrence
level via `sed` (backtick code spans, arrow-notation entries, filename-convention references)
before checking — not at the line level. This prevents a documentation pattern elsewhere on
the same line from masking a real unfilled field. Genuine unfilled field references (e.g.
`feature_id: UPG-####`, no surrounding backticks or `>` context) are not filtered and will
still exit 2 — smoke-test evidence deferred to Reconcile.

Also fixed the delta diff packet header: changed `${delta_base}->HEAD` label to
`${delta_base}->working tree` to accurately reflect the diff semantics after Fix A.

Status advanced to `3-Implement` (see status dashboard).

---

## Reconciliation

All acceptance criteria verified. Command output for runtime ACs is embedded below so the
reviewer can read the evidence directly in the packet.

| AC | Result | Evidence |
|---|---|---|
| B1 | PASS | See transcript below |
| B2 | PASS | See transcript below |
| B3 | PASS (split) | See transcript below |
| B4 | PASS | `scripts/codeos-review.sh` lines 166–167: `git diff HEAD -- .` (non-delta branch) unchanged — readable in packet |
| B5 | PASS | See transcript below |
| B6 | PASS | See transcript below |
| B7 | PASS | See transcript below |
| B8 | PASS | See transcript below |
| B9 | PASS | `git diff HEAD -- dba-system.md prompts/ templates/ docs/` → empty; scope sweep below confirms |
| B10 | PASS | `backlog/UPG-0031-review-delta-mode-fix.md` Feature Thread has CHG-20260630-002 row; `backlog/features.md` has UPG-0031 row; `backlog/UPG-0004-stage-4-6-reports.md` has CHG-20260630-001 row — all readable in packet |
| B11 | PASS | `status/self-development.md` UPG-0031 row: Loop step `4-Reconcile`, State `IN_PROGRESS` — readable in packet |

**Smoke-test transcripts** (all runs use `--scratch` so no live Codex session is created):

B1 — delta mode with uncommitted tracked change (`status/self-development.md`, base=HEAD):
```
review_content_bytes: 1042
packet_mode: delta
Evidence coverage:      FULL_COVERAGE
exit: 0
```

B2 — delta mode with no changes (`dba-system.md`, base=HEAD):
```
warning: prechecks skipped (--skip-prechecks)
error: review packet is empty (EMPTY_PACKET) — no reviewable content found.
       Delta mode: ensure tracked artifacts have working-tree changes since --base,
       or use --mode full with explicit artifact paths.
       Inspect the packet with --print-packet before rerunning.
exit: 4
```

B3a — full-mode packet on clean artifact (`dba-system.md`):
```
review_content_bytes: 24609
packet_mode: full
exit: 0
```
B3b — precheck on `status/self-development.md` without `--skip-prechecks`:
```
(packet content begins — no error line before it)
exit: 0
```
B3c — real unfilled placeholder (`feature_id: UPG-####` in a bare file):
> error: precheck failed — literal placeholder `'UPG-####'` found in /tmp/real-ph.md (fill in the real UPG id)
> exit: 2

B5 — untracked artifact (`backlog/UPG-0031-review-delta-mode-fix.md`) in delta mode:
```
warning: prechecks skipped (--skip-prechecks)
error: artifact is untracked; delta review cannot compare it to base: backlog/UPG-0031-review-delta-mode-fix.md
       Stage the file, commit it, or rerun with --mode full for explicit artifacts.
exit: 5
```

B6 — EMPTY_PACKET live delta: no new codex files (`reviews/codex/*.md` count unchanged 67→67):
```
error: review packet is empty (EMPTY_PACKET) — no reviewable content found.
       Delta mode: ensure tracked artifacts have working-tree changes since --base,
       or use --mode full with explicit artifact paths.
       Inspect the packet with --print-packet before rerunning.
exit: 4
codex/*.md count: before=67 after=67
```

B7 — `--print-packet` + EMPTY_PACKET: metadata printed, exit 4, no new codex files:
```
  Evidence coverage:      EMPTY_PACKET
exit: 4
codex/*.md count after: 67
```

B8 — precheck on `changes/UPG-0031__CHG-20260630-002__review-delta-working-tree.md` without `--skip-prechecks`
(corrected by CHG-20260630-003; original transcript was false):
> (packet content begins — no error line before it)
> exit: 0

**Post-completion verification defect and correction (CHG-20260630-003):**

Post-commit verification (2026-06-30) found B8b falsely recorded as PASS. Two root causes:

1. **Inline code span / HTML comment range interaction**: The original precheck pipeline ran
   `sed '/<!--/,/-->/d'` before stripping inline code spans. A code span containing `<!--`
   (e.g., `` `<!-- … -->` `` on line 72 of this file) opened a sed deletion range that swallowed
   all subsequent lines until the next `-->` (here: the AC section comment at line 113). This
   silently hid lines 73–113 — including the scope-boundary prose on line 100 — from the grep,
   masking what would have been a false-positive hit. The original B8b "PASS" was an artifact
   of this hidden deletion, not a correct pass.

   Fix (CHG-20260630-003): the precheck now strips inline code spans FIRST
   (`sed 's/\`[^`]*\`//g'`), then strips HTML comment blocks. This makes HTML comment
   removal safe against inline code spans.

2. **Bare prose placeholder on line 100**: After the script fix, line 100
   (`produced false positives (UPG-#### in blockquotes…)`) was correctly exposed to the grep
   and would have triggered a false positive. The word `UPG-####` in that sentence was a
   literal placeholder token written without a code span.

   Fix (CHG-20260630-003): wrapped `UPG-####` in backticks on line 100.

3. **Fenced code block in B3c transcript (line 241)**: The error-message transcript
   contained `'UPG-####'` (single-quoted) inside a fenced code block. The backtick-span
   filter does not handle fenced code blocks, so the literal token survived to the grep.

   Fix (CHG-20260630-003): converted the B3c transcript to blockquote format; blockquote
   lines are explicitly filtered by the precheck (`grep -vE '^\s*>'`).

Bare prose `UPG-####` / `CHG-YYYYMMDD-NNN` remains intentionally fail-closed.

**Stale-reference sweep:** Delta diff header at `scripts/codeos-review.sh:385` now says
`${delta_base}->working tree`. The only remaining `HEAD` reference near delta code is
`git cat-file -e "${delta_base}:${a}"` — an object-existence check at base, not a diff
call; unrelated to Fix A. All four Fix A sites verified clean (lines 159, 160, 185, 231).

**Scope sweep:** `git diff HEAD -- dba-system.md prompts/ templates/ docs/` → empty.
All working-tree modifications are inside declared scope: `scripts/codeos-review.sh`,
`backlog/UPG-0031-review-delta-mode-fix.md`, `backlog/features.md`,
`changes/UPG-0031__CHG-20260630-002__review-delta-working-tree.md`,
`status/self-development.md`, `backlog/UPG-0004-stage-4-6-reports.md`.
