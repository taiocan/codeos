---
feature_id: UPG-0033
slug: review-script-instrumentation
title: Review Script Instrumentation — Timing, Reconnect Count, Reasoning Effort
status: COMPLETE
priority: P1
depends_on: [UPG-0027]
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: review-script-instrumentation — Review Script Instrumentation

**Priority**: P1
**Status**: IN_PROGRESS
**Type**: script-tooling

## Problem

`codeos-review.sh` emits no operational metrics per review run:

1. **No timing.** There is no wall-clock record of how long a Codex call takes, making
   effort-vs-quality tradeoffs unanalysable and slow reviews invisible in the log.

2. **Wrong reasoning-effort key.** The script passed `-c reasoning_effort=...` which Codex
   silently ignores. The correct key is `model_reasoning_effort`. All reviews have therefore
   run at Codex's built-in default (`high`) regardless of intent, and callers had no
   controllable lever.

3. **No reconnect visibility.** When Codex's WebSocket disconnects and retries, the event is
   buried in the raw assessment file. There is no aggregatable metric to track reconnect rate
   across reviews or distinguish transient from systematic network failures.

Discovered during a post-incident investigation of a WebSocket idle-timeout (2026-07-01).

## Upgrade

Add three instrumentation outputs to every review run, all additive and backward-compatible:

- `CODEOS_REASONING_EFFORT` env var (default `high`) — passes the correct
  `model_reasoning_effort` key to `codex exec` fresh and resume calls.
- Wall-clock timing (`elapsed_ms`) around the Codex call.
- Reconnect count (`reconnect_count`) from `stream disconnected` occurrences in Codex output.
- Three new fields in the review YAML header: `reasoning_effort`, `reconnect_count`, `elapsed_ms`.
- One new line in the review log entry: `Effort: ... Wall time: ...ms Reconnects: ...`

## Scope

Primary change: `scripts/codeos-review.sh`. Bookkeeping: feature brief, change record, `backlog/features.md`, `status/self-development.md`, `status/roadmap.md`. No prompt, template, doctrine, or packet-format changes.

## Value

Enables: duration baseline across review history, effort-level A/B comparison (confirmed
`medium` is 65% faster than `high` with identical verdict for this artifact class), and
reliable disconnect rate monitoring without manual grep of raw assessment files.

## Feature Thread

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260701-001 | `changes/UPG-0033__CHG-20260701-001__review-script-instrumentation.md` | Add instrumentation to codeos-review.sh | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
