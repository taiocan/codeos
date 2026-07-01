---
change_id: CHG-20260701-001
feature_id: UPG-0033
slug: review-script-instrumentation
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0033__CHG-20260701-001__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0033 / CHG-20260701-001 — Review Script Instrumentation

## TRACE HEADER

```yaml
feature_id: UPG-0033
primary_feature_id: UPG-0033
change_id: CHG-20260701-001
slug: review-script-instrumentation
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0033
related_features: []
review_series: RVS__UPG-0033__CHG-20260701-001__S4
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
triage_class: script-tooling
scope_axis: self-dev only
corrects: ~
corrected_by: ~
follow_up_of: ~
fixes_findings: []
```

---

## Step 1 — Change Intent

### Problem

`codeos-review.sh` provides no operational visibility into individual review runs:

1. **Silent wrong reasoning-effort key.** The script passed `-c reasoning_effort=...` to
   `codex exec` which Codex silently ignores. The correct key is `model_reasoning_effort`.
   Consequently all reviews have run at Codex's built-in default (`high`) regardless of any
   caller intent, and there is no controllable effort lever exposed to operators.

2. **No wall-clock timing.** Review duration is not recorded anywhere — not in the YAML
   header, not in the log entry. This makes effort-vs-quality tradeoffs invisible and blocks
   any systematic duration analysis.

3. **No reconnect metric.** When Codex's WebSocket disconnects and retries during a review,
   the event is buried inside the full assessment file. There is no aggregatable field to
   track reconnect rate across reviews or distinguish a transient network failure from a
   systematic problem.

These gaps were discovered during a post-incident investigation of a WebSocket idle-timeout
(2026-07-01 05:14Z, `session 019f1a65`). The investigation confirmed: the disconnect was a
one-off transient; all reviews ran at `high` effort despite the presence of the (broken) key
in the script; and `medium` effort produces equivalent verdicts 65% faster for this artifact
class.

**Implementation note:** This is an already-implemented investigation fix. The script
changes were applied during the investigation that uncovered the problems. Steps 1–2 document
intent and acceptance criteria; Step 3 claims and verifies the existing implementation.

### What changes

| File | Change |
|---|---|
| `scripts/codeos-review.sh` | (1) Fix reasoning-effort key from `reasoning_effort` to `model_reasoning_effort` in both `codex exec` calls (fresh + resume). (2) Expose `CODEOS_REASONING_EFFORT` env var (default `high`) so callers can override without editing the script. (3) Record wall-clock time around the Codex call as `REVIEW_ELAPSED_MS`. (4) Count `stream disconnected` occurrences in Codex output as `REVIEW_RECONNECT_COUNT`. (5) Add `reasoning_effort`, `reconnect_count`, `elapsed_ms` to the review YAML header. (6) Add `Effort: ... Wall time: ...ms Reconnects: ...` line to each review log entry. (7) Add `effort: ... elapsed: ...ms reconnects: ...` line to stdout summary. |
| `backlog/UPG-0033-review-script-instrumentation.md` | NEW — feature brief for this upgrade. |
| `changes/UPG-0033__CHG-20260701-001__review-script-instrumentation.md` | NEW — this change record. |
| `backlog/features.md` | UPG-0033 row added to feature-ID map. |
| `status/self-development.md` | UPG-0033 row activated at Step 3-Implement. |
| `status/roadmap.md` | UPG-0033 row added to Unsequenced section. |

### What stays the same (scope boundary)

- **Default effort level unchanged.** `CODEOS_REASONING_EFFORT` defaults to `high`, matching
  Codex's built-in default. The reasoning-effort key was previously silently ignored by Codex,
  so this change makes the key effective for the first time rather than altering default
  behavior. Historical review verdicts are unaffected by the key fix.
- **Packet format** — unchanged. No packet content, size limits, or coverage logic touched.
- **Verdict format** — unchanged. New YAML fields are strictly additive; existing fields
  unchanged.
- **Log format** — append-only invariant preserved. One new informational line per entry;
  existing line order and content unchanged.
- **v0 schema validation** — unchanged. New fields are not part of the required schema.
- **`stage-start` and `decision` subcommands** — not touched.
- **All prompt, template, doctrine, and packet files** — NOT in scope.
- **`dba-system.md`** — NOT in scope.

### Triage class: `script-tooling`

Adds instrumentation to `codeos-review.sh` and fixes a silently-broken config key.
Bookkeeping files (feature brief, change record, index/status rows) are also modified.
4-step loop, PROFILE-3.

### Scope axis: `self-dev only`

No downstream doctrine changes.

### Review profile: PROFILE-3

Script-tooling class. Codex review before each step gate; human approval at all four gates.

### Originating backlog item

`backlog/UPG-0033-review-script-instrumentation.md`

---

## Step 2 — Acceptance Criteria

### AC-1: Reasoning-effort key is correct

`scripts/codeos-review.sh` passes `-c model_reasoning_effort=<value>` (not
`-c reasoning_effort=<value>`) to both `codex exec` (fresh) and `codex exec resume` calls.
Verification: `grep -n 'model_reasoning_effort' scripts/codeos-review.sh` returns hits in
both the fresh-session and resume-session branches inside `run_codex()`.

### AC-2: Env-var default preserves historical behavior

`CODEOS_REASONING_EFFORT` defaults to `high` when unset. Callers can override by exporting
the variable before invoking the script. The default means existing pipelines that do not
set the var produce the same reasoning effort as before this change.
Verification: `grep 'CODEOS_REASONING_EFFORT:-high' scripts/codeos-review.sh` returns a hit.

### AC-3: Elapsed time is recorded

Every review run populates `REVIEW_ELAPSED_MS` with a non-negative integer (milliseconds)
measured by `date +%s%N` before and after the `codex exec` call.
Verification: inspect `run_codex()` for `_t_start`/`_t_end` and the assignment
`REVIEW_ELAPSED_MS=$(( (_t_end - _t_start) / 1000000 ))`.

### AC-4: Reconnect count is recorded

Every review run sets `REVIEW_RECONNECT_COUNT` by counting occurrences of
`stream disconnected` in the Codex output. Zero is a valid value (no disconnects).
Verification: `grep 'REVIEW_RECONNECT_COUNT' scripts/codeos-review.sh` returns the
`grep -c 'stream disconnected'` assignment.

### AC-5: New fields appear in review YAML header

The assessment file written by `write_assessment()` contains three new additive fields
after the `evidence` line: `reasoning_effort`, `reconnect_count`, `elapsed_ms`.
Verification: `grep -n 'reasoning_effort\|reconnect_count\|elapsed_ms' scripts/codeos-review.sh`
returns hits inside the assessment YAML block.

### AC-6: New line appears in review log entry

Each entry appended to `reviews/review-log.md` contains one new line of the form
`Effort: <value>   Wall time: <ms>ms   Reconnects: <count>` after the `Reviewer:` line.
Verification: grep the log-append block in the script.

### AC-7: New line appears in stdout summary

The `review` subcommand prints `  effort: <value>   elapsed: <ms>ms   reconnects: <count>`
to stdout alongside the existing summary lines.
Verification: grep the stdout-summary block.

### AC-8: No behavioral change to existing subcommands

`stage-start` and `decision` subcommands do not call `run_codex()` and are unaffected.
Packet construction, verdict parsing, coverage logic, and log-append invariants are
unchanged. Verification: diff shows no edits outside `run_codex()` and the three output
blocks (YAML header, log entry, stdout summary).

---

## Step 3 — Implementation

**Status:** Implementation complete (applied during investigation, 2026-07-01).

### Changes in `scripts/codeos-review.sh`

**`run_codex()` — instrumentation additions (around lines 447–478):**

Added before the `codex exec` calls:
```bash
local out _t_start _t_end
local _effort="${CODEOS_REASONING_EFFORT:-high}"
_t_start="$(date +%s%N)"
```

Changed in both branches (fresh and resume):
```bash
# before: -c reasoning_effort="${CODEOS_REASONING_EFFORT:-high}"
# after:
-c model_reasoning_effort="${_effort}"
```

Added after the `codex exec` calls:
```bash
_t_end="$(date +%s%N)"
REVIEW_SESSION="${session_id}"
REVIEW_OUTPUT="${out}"
REVIEW_ELAPSED_MS=$(( (_t_end - _t_start) / 1000000 ))
REVIEW_RECONNECT_COUNT="$(printf '%s\n' "${out}" | grep -c 'stream disconnected' || true)"
REVIEW_EFFORT="${_effort}"
```

**Assessment YAML header — new fields (after `evidence` line):**
```bash
echo "  reasoning_effort: ${REVIEW_EFFORT}"
echo "  reconnect_count: ${REVIEW_RECONNECT_COUNT}"
echo "  elapsed_ms: ${REVIEW_ELAPSED_MS}"
```

**Review log entry — new line (after `Reviewer:` line):**
```bash
echo "Effort: ${REVIEW_EFFORT}   Wall time: ${REVIEW_ELAPSED_MS}ms   Reconnects: ${REVIEW_RECONNECT_COUNT}"
```

**Stdout summary — new line:**
```bash
echo "  effort: ${REVIEW_EFFORT}   elapsed: ${REVIEW_ELAPSED_MS}ms   reconnects: ${REVIEW_RECONNECT_COUNT}"
```

All other script logic (packet construction, precheck, session management, verdict parsing,
coverage check, log-append, `stage-start`, `decision`) is unchanged.

---

## Step 4 — Reconcile

### AC Verification

| AC | Check | Result |
|---|---|---|
| AC-1 | `grep -n 'model_reasoning_effort' scripts/codeos-review.sh` → hits at lines 452, 454 (both fresh + resume branches) | PASS |
| AC-2 | `grep -n 'CODEOS_REASONING_EFFORT:-high' scripts/codeos-review.sh` → hit at line 448 | PASS |
| AC-3 | `_t_start`/`_t_end` at lines 449/464; `REVIEW_ELAPSED_MS` assigned at line 467 | PASS |
| AC-4 | `REVIEW_RECONNECT_COUNT` assigned via `grep -c 'stream disconnected'` at line 468 | PASS |
| AC-5 | `reasoning_effort`/`reconnect_count`/`elapsed_ms` emitted at lines 709–711 | PASS |
| AC-6 | `Effort: ... Wall time: ...ms Reconnects: ...` at line 727 | PASS |
| AC-7 | `effort: ... elapsed: ...ms reconnects: ...` at line 749 | PASS |
| AC-8 | `run_codex` called only at line 599 (review subcommand); `stage-start` (line 79) and `decision` (line 761) do not call it | PASS |

### Cross-reference sweep

| Reference | Target | Status |
|---|---|---|
| Change record → `backlog/UPG-0033-review-script-instrumentation.md` | exists | OK |
| Backlog brief Feature Thread → change record | exists | OK |
| `backlog/features.md` UPG-0033 row | exists at line 66 | OK |
| `status/self-development.md` UPG-0033 row | exists | OK |
| `status/roadmap.md` UPG-0033 row | exists at line 94 | OK |

No stage-table↔prompt-file drift (script-tooling change; no prompt files touched). No orphaned links.

### Reviewer scope triage

| Finding | Source | Triage | Disposition |
|---|---|---|---|
| F1/F-A: implementation predated Step 1 | Step 1 R1, Step 3 R1 | REVIEW-BOOKKEEPING | Reviewer applies standard loop rule; human accepted investigation-first as a one-time exception, not a new rule. |
| F2a: "What changes" table incomplete | Step 1 R1 | IN-SCOPE NON-BLOCKER | Fixed: expanded table to all six files. |
| F2b: "only codeos-review.sh" prose contradiction | Step 1 R2 | IN-SCOPE NON-BLOCKER | Fixed: updated triage description and backlog scope. |
| F3/F-B: UPG-0007 diff drift in packet | All rounds | OUT-OF-SCOPE BACKLOG | AJ-009 — workspace constraint; root cause is both UPGs uncommitted simultaneously. Not a UPG-0033 scope defect. |
| F-C: overclaiming compatibility guarantee | Step 3 R1 | IN-SCOPE NON-BLOCKER | Fixed: weakened claim to accurately describe ignored-key behavior. |
