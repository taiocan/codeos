---
feature_id: UPG-0031
slug: review-delta-mode-fix
title: Review script delta-mode working-tree fix + fail-closed EMPTY_PACKET guard
status: COMPLETE
priority: P1
depends_on:
  - UPG-0027
related_features:
  - UPG-0030
supersedes: []
superseded_by: []
---

# Upgrade: review-delta-mode-fix — Review script delta-mode working-tree fix + fail-closed EMPTY_PACKET guard

**Priority**: P1
**Status**: COMPLETE
**Type**: script-tooling
**Related**: scripts/codeos-review.sh

## Problem

`scripts/codeos-review.sh` delta mode computes diffs with `git diff "${delta_base}" HEAD -- <paths>`.
This compares the base commit to HEAD only. When review fixes are uncommitted, `HEAD` has not moved
and the diff is empty, producing `EMPTY_PACKET`. Codex is then called with zero reviewable content
and returns a DO NOT ADVANCE that cannot be actioned.

Two defects:
1. Delta mode is commit-only — excludes working-tree (staged/unstaged) changes to tracked files.
2. No fail-closed guard before Codex invocation when the packet is empty.

A third edge case: untracked new artifact files cannot be compared in delta mode at all; delta
mode should fail closed with a clear diagnostic rather than silently producing EMPTY_PACKET.

Additionally: the precheck script greps for literal `UPG-####` but hits false positives in
comment/legend sections of files like `status/self-development.md` and change record template
comments (filed as OUT-OF-SCOPE from UPG-0004; tracked here).

## Upgrade

Fix all four issues in `scripts/codeos-review.sh`:

1. **Delta mode working-tree diff** — remove `HEAD` from the four delta-mode `git diff` calls
   so they compare `<base>` to the working tree (staged + unstaged tracked changes).

2. **Untracked artifact guard** — in the per-artifact loop (delta mode), fail closed before
   Codex with a clear diagnostic if a positional artifact is untracked:
   ```
   error: artifact is untracked; delta review cannot compare it to base: <path>
          Stage, commit, or rerun with --mode full restricted to explicit artifacts.
   ```

3. **EMPTY_PACKET fail-closed guard** — after `build_packet` and after the `print_only` block,
   exit before calling `run_codex` if `PACKET_COVERAGE_STATE == EMPTY_PACKET`. The guard must
   not block `--print-packet` inspection.

4. **Precheck false-positive** — tighten the `UPG-####` grep to exclude comment/HTML-comment
   blocks and legend sections. The grep currently fires on documentation-only uses of the
   placeholder pattern (e.g. `changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md` in a comment).

## Scope

`self-dev only`. Functional change: `scripts/codeos-review.sh`. Self-dev bookkeeping:
`backlog/features.md`, `backlog/UPG-0031-review-delta-mode-fix.md`,
`changes/UPG-0031__CHG-20260630-002__review-delta-working-tree.md`,
`status/self-development.md`. Cross-feature trace link: `backlog/UPG-0004-stage-4-6-reports.md`
(Feature Thread row backfill only; no UPG-0004 scope or content change).
No downstream doctrine changes. No prompt changes. No template changes.

## Value

Prevents silent EMPTY_PACKET Codex calls that produce unactionable DO NOT ADVANCE artifacts.
Fixes the delta-mode commit-only limitation that stalled UPG-0004. Reduces --skip-prechecks
workarounds for comment-section false positives.

## Guardrail

All four fixes must be narrowly targeted. No packet format changes, no new subcommands, no
advisory/read-only/non-gatekeeping guarantee changes. `--mode full` diff/packet content behavior
unchanged. `--mode full` precheck behavior is intentionally changed by Fix D: false positives
in blockquotes, HTML comments, and code spans are eliminated; real unfilled placeholders still
exit 2.

---

## Feature Thread

> Canonical thread rollup for this feature.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260630-002 | changes/UPG-0031__CHG-20260630-002__review-delta-working-tree.md | Fix delta-mode working-tree diff, untracked guard, EMPTY_PACKET guard, precheck false-positive | COMPLETE |
| CHG-20260630-003 | changes/UPG-0031__CHG-20260630-003__precheck-filter-correction.md | Corrective: fix precheck filter ordering (code spans before HTML comments) + artifact formatting | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
