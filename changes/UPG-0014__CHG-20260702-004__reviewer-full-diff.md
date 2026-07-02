---
change_id: CHG-20260702-004
feature_id: UPG-0014
slug: reviewer-full-diff
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0014__CHG-20260702-004__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0014 / CHG-20260702-004 — Reviewer Full Diff Access

## TRACE HEADER

```yaml
feature_id: UPG-0014
primary_feature_id: UPG-0014
change_id: CHG-20260702-004
slug: reviewer-full-diff
state: IN_PROGRESS
current_step: 4-Reconcile
implements:
  - UPG-0014
related_features:
  - UPG-0032
  - UPG-0016
review_series: RVS__UPG-0014__CHG-20260702-004__S4
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

In delta-review mode (`--mode delta --base <sha>`), the packet diffs only the explicitly
named artifact files. Files changed outside the declared list are invisible to the reviewer.
This means the reviewer cannot detect scope drift — it cannot flag that `backlog/features.md`
changed but was not in the declared artifact list (as happened in UPG-0015 Step-4 R1).

The diff-review checklist in `docs/workflow-profiles.md` ("Unrelated files?", "Approved
artifacts modified after approval?") is unanswerable from the evidence alone.

Additionally, the architecture of the reviewer pipeline was misunderstood in earlier
iterations of this change: `scripts/codeos-review.sh` is a **15-line static locator shim**
(`exec "${BINARY}" "$@"`), not a behavior layer. All reviewer capability lives in the Rust
engine. This boundary is documented in `docs/reviewer-pipeline.md` (Section 10) as part of this change.

### What changes

| File | Change |
|---|---|
| `tools/reviewer/src/packet.rs` | When `delta_mode=true` AND `delta_base=Some(_)`: append `Full Context Diff (informational — all changed files since <base>):` section after named artifacts, size-clipped to remaining content budget |
| `tools/reviewer/tests/smoke.rs` | New tests: full diff present in delta+base; absent in full-mode and delta-without-base; named artifact section unmodified; clip marker present when budget=0 |
| `docs/reviewer-pipeline.md` | Add architecture note: `codeos-review.sh` is a static locator shim; all reviewer capability changes are Rust engine changes |
| `backlog/UPG-0014-reviewer-full-diff.md` | Feature Thread updated |
| `status/self-development.md` | Row activated |
| `status/roadmap.md` | UPG-0014 → IN_PROGRESS |

### Scope boundary — what stays the same

- `scripts/codeos-review.sh` — not touched; it is a static shim with no behavior to update
- `tools/reviewer/src/main.rs` — not touched; no new CLI flags added
- Named artifact evidence — backward-compatible; full context diff is additive
- Reviewer advisory status — unchanged
- Secret filtering and size budget — full context diff obeys `CODEOS_PACKET_BUDGET_BYTES`
- `dba-system.md` — not touched
- Decision/log/provenance code — not touched
- The five reviewer "mode" names — not formalized in this increment

### Behavior

When `--mode delta` AND `--base <sha>` are BOTH active in `codeos-reviewer review`:

1. Named-artifact delta content is built as today (unchanged).
2. After the named-artifact section, compute remaining packet budget.
3. Run `git diff <base>` with no path filter (all changed files, not just named artifacts).
4. If within remaining budget: append section:
   ```
   Full Context Diff (informational — all changed files since <base>):
   <diff output>
   ```
5. If over remaining budget: clip and append:
   ```
   Full Context Diff (informational — all changed files since <base>):
   <diff output clipped>
   [CLIPPED: full diff exceeded packet budget — showing first N of M bytes]
   ```

When `--mode delta` is used without `--base`, or `--mode full` is used: no Full Context
Diff section. Packet is byte-for-byte identical to pre-change behavior.

The "informational" label signals to the reviewer that this section is supplementary
context, not the authoritative scope declaration. The declared artifact list remains the
canonical scope.

### Triage

- Class: `script-tooling`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-3` (1 Codex review per step, max 3 rounds)
- Originating backlog id: `UPG-0014`

---

## Step 2 — Acceptance Criteria

### AC-1 — Full Context Diff section present when delta+base, absent otherwise

The `Full Context Diff (informational — all changed files since <base>):` section appears in
the packet if and only if BOTH `--mode delta` AND `--base <sha>` are active. Specifically:

- `--mode delta --base <sha>` → section present
- `--mode delta` without `--base` → section absent
- `--mode full --base <sha>` (or no `--mode`) → section absent
- No `--base` in any mode → section absent

No new CLI flag. Behavior is automatic.

### AC-2 — Without delta+base, packet contains no Full Context Diff section

Running `codeos-reviewer review` in any mode other than `--mode delta --base <sha>` — including
`--mode full`, `--mode delta` without `--base`, and `--mode full --base <sha>` — produces a
packet with no Full Context Diff section. The gate `opts.delta_mode && opts.delta_base.is_some()`
is the only activation path; all other modes are structurally unaffected.

### AC-3 — Full diff is additive; named-artifact section is not replaced or modified

The named-artifact delta content (per-artifact diffs in delta mode) is present and unmodified
when the Full Context Diff section is also appended. Full context diff appears after the
named-artifact section, not in place of it.

### AC-4 — Budget clipping is explicit

When the full context diff exceeds the remaining content budget:
- The section is clipped at `remaining = CODEOS_PACKET_BUDGET_BYTES - review_content_bytes`
- A marker is appended: `[CLIPPED: full diff exceeded packet budget — showing first N of M bytes]`
- No truncation is silent

`review_content_bytes` counts artifact content bytes and named-artifact diff bytes only —
consistent with the rest of the budget system, which has always excluded task prompt and
section headers from this measure. The clip is therefore an approximation of the real
remaining packet size, not an exact guarantee of total packet byte count.

When the full context diff fits within remaining content budget, no CLIPPED marker appears.

### AC-5 — Secret redaction applies to the full context diff

The full context diff passes through the same redaction pipeline as named artifacts. Any
redacted span increments the packet's `redaction_count` and is replaced with `[REDACTED]`.
Verified by grep of the redaction call site confirming the full diff bytes pass through it.

`coverage_state` is computed from named-artifact evidence only — consistent with the
"informational" label of the full diff, which is supplementary context, not primary evidence.
Full-diff redactions appear in `redaction_count` (visible in the assessment header). A git
error in the full diff call produces an explicit `[ERROR: git diff failed — …]` marker in
the packet (fail-closed, not silently empty). Neither case escapes the packet unannounced.

### AC-6 — Auto-enable condition is delta_mode=true AND delta_base=Some(_) in Rust

In `cmd/review.rs` (or `PacketOpts`), the condition for appending the Full Context Diff is
`delta_mode == true && delta_base.is_some()`. Neither condition alone is sufficient. This is
a Rust-internal behavioral rule, not a bash-level decision (the bash script is a static
passthrough and requires no change).

### AC-7 — Section label distinguishes informational from declared scope

The section header is exactly:
`Full Context Diff (informational — all changed files since <base>):`
where `<base>` is the actual base SHA or ref. The word "informational" must be present so
the reviewer knows this is supplementary context, not the canonical scope declaration.

### AC-8 — Pipeline doc records the shim boundary (verified at Reconcile)

`docs/reviewer-pipeline.md` Section 10 ("Architecture: `codeos-review.sh` is a static locator
shim") contains a note that `codeos-review.sh` is `exec "${BINARY}" "$@"` verbatim; all
reviewer capability changes are Rust engine changes. Verified: section exists in the file.

### AC-9 — Tests pass

`cargo test` passes with tests covering at minimum:
- AC-1: full diff section present in delta+base; absent in delta-only; absent in full mode (with and without base)
- AC-2: packet contains no Full Context Diff section in full mode (with and without base)
- AC-3: named artifacts section present and diff content unmodified when full diff also present
- AC-4: clip marker present when diff exceeds budget (budget=0); no clip marker when diff fits in budget

---

## Step 3 — Implement

### Changes made

#### `tools/reviewer/src/packet.rs`

Full Context Diff section added after named-artifact diff content, before `final_base_sha` computation. Activation guard: `opts.delta_mode && opts.delta_base.is_some()`. Calls `git_diff_range(base, &[], &opts.repo_root)` (all files, no path filter). Applies `redact_secrets` (AC-5). Clips at `remaining = budget.saturating_sub(review_content_bytes)` with `[CLIPPED: ...]` marker (AC-4). Section header: `Full Context Diff (informational — all changed files since {base}):` (AC-7).

Edge cases: empty diff → `(no changes detected outside named artifacts)\n`; zero remaining budget → CLIPPED with 0/N display.

#### `docs/reviewer-pipeline.md`

New section **"10. Architecture: `codeos-review.sh` is a static locator shim"** inserted before the old Section 10 (Usage, now Section 11). States that `codeos-review.sh` is `exec "${BINARY}" "$@"` verbatim, contains no argument preprocessing, and that all reviewer capability changes are Rust engine changes (AC-8).

#### `tools/reviewer/tests/smoke.rs`

5 new smoke tests (AC-9). Helper `setup_codeos_symlink` added to make temp repos find the real Codeos toolkit prompts via `.codeos` symlink. Tests evolved through R1–R3 to reach final strength.

| Test | AC | Asserts |
|---|---|---|
| `smoke_full_context_diff_present_in_delta_plus_base` | AC-1 | `--mode delta --base` → packet contains "Full Context Diff (informational" |
| `smoke_full_context_diff_absent_in_full_mode` | AC-1/AC-2 | `--mode full` (no `--base`) → packet does NOT contain "Full Context Diff (informational" |
| `smoke_full_context_diff_absent_in_delta_without_base` | AC-1 | `--mode delta` without `--base` → packet does NOT contain "Full Context Diff (informational" |
| `smoke_full_context_diff_named_artifact_section_unchanged` | AC-3 | ordering: ARTIFACTS TO REVIEW before Full Context Diff; content: DELTA DIFF section contains exact `git diff <base> -- tracked.md` output |
| `smoke_full_context_diff_clipping_marker` | AC-4 | `CODEOS_PACKET_BUDGET_BYTES=0` → packet contains "Full Context Diff (informational" AND "CLIPPED" |

### `packet.rs` R1–R3 fixes

- Replaced `unwrap_or_default()` with explicit `match` — git error → `[ERROR: git diff failed — full context diff unavailable: …]` in packet (fail-closed, AC-1)
- Separated `full_total == 0` from `remaining == 0` cases (edge-case clarity)
- Added comment: `remaining` is content-budget approximation, consistent with pre-existing budget system

### Test results

```
running 22 tests
test result: ok. 22 passed; 0 failed; 0 ignored
running 31 tests
test result: ok. 31 passed; 0 failed; 0 ignored
```

Total: 53 tests (22 unit + 31 smoke). All pass.

### What was NOT changed

- `scripts/codeos-review.sh` — confirmed unchanged (static shim)
- `tools/reviewer/src/main.rs` — no new CLI flags
- `tools/reviewer/src/cmd/review.rs` — `PacketBuildOptions` already had `delta_mode` and `delta_base`; no changes needed
- Decision/provenance code — untouched

---

## Step 4 — Reconcile

### AC verification

| AC | Verdict | Evidence |
|---|---|---|
| AC-1: Full Context Diff present iff delta+base | PASS | `packet.rs:366` guard `opts.delta_mode && opts.delta_base.is_some()`; tests: present (delta+base), absent (full-mode), absent (delta-without-base), absent (full-mode-with-base) |
| AC-2: No Full Context Diff section outside delta+base | PASS | Gate is `delta_mode && delta_base.is_some()`; `smoke_full_context_diff_absent_in_full_mode`, `smoke_full_context_diff_absent_in_full_mode_with_base`, `smoke_full_context_diff_absent_in_delta_without_base` |
| AC-3: Full diff additive; named artifacts unmodified | PASS | `packet.rs` appends after named-artifact section; `smoke_full_context_diff_named_artifact_section_unchanged` verifies ordering and exact content match against raw `git diff` output |
| AC-4: Budget clipping explicit (content-budget approximation) | PASS | `remaining = budget.saturating_sub(review_content_bytes)`; `[CLIPPED: ...]` marker; tests: CLIPPED present (budget=0), absent (default budget + tiny diff) |
| AC-5: Secret redaction applies; coverage_state reflects named artifacts | PASS | `redact_secrets(&full_raw)` at `packet.rs:376`; `redaction_count` incremented; `coverage_state` intentionally reflects named artifacts only (documented in `docs/reviewer-pipeline.md` §5); errors explicit as `[ERROR: …]` in packet |
| AC-6: Auto-enable condition is `delta_mode && delta_base.is_some()` | PASS | Code at `packet.rs:366–367`; no CLI flag added |
| AC-7: Section label contains "informational" | PASS | Header: `Full Context Diff (informational — all changed files since {base}):` |
| AC-8: Pipeline doc records shim boundary | PASS | `docs/reviewer-pipeline.md` Section 10 written; states `exec "${BINARY}" "$@"` verbatim |
| AC-9: Tests pass (7 full-diff tests; 55 total) | PASS | 55/55 tests pass (`cargo test`) |

### Scope sweep

- `scripts/codeos-review.sh` — grep confirms unchanged; no `--all-diff` references anywhere
- `tools/reviewer/src/cmd/review.rs` — unchanged; removed from "What changes" table (was incorrectly listed)
- `docs/reviewer-pipeline.md` — Section 10 present; old Section 10 (Usage) renumbered to Section 11; Appendices unchanged
- No stale future-tense claims remaining in Steps 1/2/AC-8 (fixed at Reconcile)
- `backlog/UPG-0014-reviewer-full-diff.md` Feature Thread — status/CHG already updated in Step 3 prep

### Reviewer scope triage (R1–R3 findings)

| Finding | Verdict | Disposition |
|---|---|---|
| R1: fail-open on git diff (`unwrap_or_default`) | IN-SCOPE BLOCKER | FIXED |
| R1: missing delta-only absence test | IN-SCOPE BLOCKER | FIXED |
| R1: weak clipping assertion (OR condition) | IN-SCOPE BLOCKER | FIXED |
| R2: AC-3 ordering-only test | IN-SCOPE BLOCKER | FIXED (content match against raw git diff) |
| R2/R3: budget claim imprecision | IN-SCOPE NON-BLOCKER | FIXED in AC-4 (now documents approximation) |
| R3: AC-3 baseline comparison residual | IN-SCOPE NON-BLOCKER | FIXED post-budget by comparing against `git diff` output; accepted |
| S4-R1: TRACE HEADER current_step stale | IN-SCOPE BLOCKER | FIXED (3-Implement → 4-Reconcile) |
| S4-R1: AC-2 "byte-for-byte" claim untestable | IN-SCOPE BLOCKER | FIXED (claim narrowed to absence of Full Context Diff section) |
| S4-R1: AC-9 missing full-mode-with-base + clip-absent tests | IN-SCOPE BLOCKER | FIXED (2 new tests added) |
| S4-R1/R2: coverage_state bypass for full-diff | IN-SCOPE BLOCKER | REJECTED → DOCUMENTED (coverage_state intentionally reflects named artifacts; doc clarification added to §5) |
