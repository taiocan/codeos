---
feature_id: UPG-0027
slug: lean-review-runner-packet-architecture
title: Lean Review Runner and Packet Architecture
status: IN_PROGRESS
priority: P0
depends_on:
  - UPG-0030
related_features:
  - UPG-0001
  - UPG-0028
  - UPG-0029
supersedes: []
superseded_by: []
---

# Feature Brief: Lean Review Runner and Packet Architecture

## Problem

The Codeos review process sends oversized, unfocused packets to the reviewer, and the reviewer
prompt gives no focused task. Both problems compound each other.

**Packet problem:** `codeos-review.sh` includes full file content for every artifact passed to
it. A full change record, full backlog brief, and full prior context are sent even when the
reviewer only needs a diff of one section or a path+sha guard for an unchanged file. There is
no manifest, no mode selection, no budget report, and no diff-only mode.

**Reviewer problem:** The current prompt opens with "Critically assess:" and provides generic
scope contract and triage rules. It does not ask the reviewer to: answer whether acceptance
criteria are satisfied, evaluate evidence quality, grade the evidence, cap blocker findings,
or distinguish between style issues and real blockers. Without a focused task, the reviewer
acts as a general-purpose critic — flagging wording, prose consistency, and peripheral issues
at the same priority as in-scope blockers.

This causes the recurring review-loop problem: each fix creates new prose the reviewer can
re-litigate, round counts grow, and the human eventually has to decide by budget exhaustion
rather than by a clean NO OBJECTION.

## Principle

**Codex should review judgment and evidence.
The runner should handle deterministic checks and packet shaping.**

## Objective

Three targeted changes, sequenced by value:

1. Make the reviewer ask the right questions first.
2. Make packets visible and budgeted.
3. Make local checks and delta packets real.

## Scope

**Self-dev only.** No changes to `dba-system.md` or downstream doctrine.

Language migration (Rust/Go/Python typed runner) is explicitly deferred to a future feature.
It must not be attempted until the packet model is proven by at least two real changes.

## Change Sequence

### CHG-1 — Focused Reviewer Task Prompt

`changes/UPG-0027__CHG-20260629-002__focused-reviewer-prompt.md`

Scope:
- New `prompts/codeos-reviewer-task.md` — focused five-question reviewer task
- `scripts/codeos-review.sh` — inject template; remove inline static "Critically assess:" block
- `docs/reviewer-pipeline.md` — one reference line
- No packet structure changes in this CHG

### CHG-2 — Packet Manifest and Budget Report

`changes/UPG-0027__CHG-20260629-003__packet-manifest-and-budget.md`

Scope:
- `scripts/codeos-review.sh` — YAML manifest prepended to every packet; budget report appended
- Inclusion modes: `full_file`, `path_sha_only`, `omitted_with_reason` (`header_only` deferred)
- Budget threshold: `CODEOS_PACKET_BUDGET_BYTES` (default 50 000), warning only
- Manifest in assessment file only; review-log.md format unchanged

### CHG-3 — Local Prechecks and True Delta Mode

`changes/UPG-0027__CHG-20260629-004__prechecks-and-delta-mode.md`

Scope:
- `scripts/codeos-review.sh` — two-tier local check gate; `--mode delta --base <sha>` flag
- Hard fail (deterministic governance violations): literal placeholders, forbidden fields,
  scope-boundary diffs (`dba-system.md`, `scripts/codeos-review.sh` when scope is self-dev)
- Warning only (fuzzy checks): unresolved TBD/FIXME, broken §X section references,
  trace header/dashboard mismatch
- Delta packet: changed hunks only since `<sha>`, no full unchanged files

---

## Feature Thread

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260629-002 | changes/UPG-0027__CHG-20260629-002__focused-reviewer-prompt.md | Focused reviewer task prompt | COMPLETE |
| CHG-20260629-003 | changes/UPG-0027__CHG-20260629-003__packet-manifest-and-budget.md | Packet manifest and budget report | PLANNED |
| CHG-20260629-004 | changes/UPG-0027__CHG-20260629-004__prechecks-and-delta-mode.md | Local prechecks and true delta mode | PLANNED |

### Reviews

| Review series | Step | Verdict summary |
|---|---|---|

### Findings

(none yet)

### Follow-up

| Feature ID | Reason | Source finding |
|---|---|---|
| TBD | Typed runner (Rust/Go/Python) — after CHG-1–3 prove packet model | — |
