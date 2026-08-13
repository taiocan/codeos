---
change_id: CHG-20260702-005
feature_id: UPG-0035
slug: reviewer-sha-only-exit-code
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0035__CHG-20260702-005__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0035 / CHG-20260702-005 — Reviewer `--sha-only` Missing-Path Exit Code

## TRACE HEADER

```yaml
feature_id: UPG-0035
primary_feature_id: UPG-0035
change_id: CHG-20260702-005
slug: reviewer-sha-only-exit-code
state: IN_PROGRESS
current_step: 4-Reconcile
implements:
  - UPG-0035
related_features:
  - UPG-0032
review_series: RVS__UPG-0035__CHG-20260702-005__S1
review_profile: PROFILE-3
review_state: DRAFT
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: CHG-20260702-001
```

---

## Step 1 — Change Intent

### Problem

When a `--sha-only` path does not exist, the Rust reviewer exits 4 (`EXIT_PACKET`) because
`packet::build()` propagates the missing-path `Err` through the PACKET error handler. The Bash
pilot (`codeos-review.sh` before UPG-0032) exited 2 (config/usage error) for the same input.

Consumers scripting against the binary's exit codes (e.g., CI pipelines) observe exit 4 for a
bad CLI argument — an exit code that belongs to "packet construction failed on valid input," not
"the path you gave me doesn't exist." Exit 4 is also returned by EMPTY_PACKET and other internal
failures, masking the diagnostic signal.

There is already an explicit pre-`packet::build()` existence check for positional artifacts
(`cmd/review.rs:83–89`, exits `EXIT_PACKET`). `--sha-only` paths have no analogous check.

### What changes

| File | Change |
|---|---|
| `tools/reviewer/src/cmd/review.rs` | Add explicit existence check for each `--sha-only` path after positional artifact check; emit `error: --sha-only path not found: <path>` to stderr; exit `EXIT_USAGE` (1) |
| `backlog/UPG-0035-reviewer-sha-only-exit-code.md` | Feature Thread: CHG-20260702-005 activated |
| `status/self-development.md` | Row activated |
| `status/roadmap.md` | UPG-0035 → IN_PROGRESS |

### Scope boundary — what stays the same

- `packet::build()` internal missing-path check remains (defense-in-depth; still errors if called
  directly without going through `cmd/review.rs`)
- Exit codes for all other error classes unchanged
- Positional artifact check at line 83-89 unchanged (it exits `EXIT_PACKET`; that is intentional
  and documented in the existing comment: "AC-3: exit 4 for artifact-not-found")
- Bash shim `scripts/codeos-review.sh` — not touched
- No new CLI flags

### Decision: exit code

`EXIT_USAGE` (1) — a missing `--sha-only` path is a bad CLI argument. `EXIT_CONFIG` (2) would
imply a configuration problem, not a bad argument. Backlog brief notes either is acceptable;
USAGE is more semantically precise.

### Triage

- Class: `script-tooling`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-3` (1 Codex review per step, max 3 rounds)
- Originating backlog id: `UPG-0035`

---

## Step 2 — Acceptance Criteria

### AC-1 — `--sha-only <missing>` exits 1

`codeos-reviewer review --sha-only <non-existent-path> …` returns exit code 1 (`EXIT_USAGE`).

### AC-2 — stderr contains the missing path

The error message on stderr is exactly:
`error: --sha-only path not found: <path>`
where `<path>` is the argument that was passed.

### AC-3 — Positional artifact missing check unchanged

The existing check at `cmd/review.rs:83–89` (positional artifacts → `EXIT_PACKET`) is not
modified. A missing positional artifact still exits 4.

### AC-4 — `packet.rs` and bash shim not touched

`tools/reviewer/src/packet.rs` and `scripts/codeos-review.sh` contain no diff from this change.

### AC-5 — Other exit code classes unchanged

All other exit codes (`EXIT_CONFIG` = 2, `EXIT_PROVIDER` = 3, `EXIT_PACKET` = 4,
`EXIT_WRITE` = 5) retain their current semantics and call sites. No exit-code renumbering.

### AC-6 — Tests pass

`cargo test` passes. New test(s) cover:
- `--sha-only <missing>` → exit 1 and stderr contains `--sha-only path not found:`
- `--sha-only <existing>` → proceeds normally (no spurious exit 1)

---

## Step 3 — Implement

### Changes made

#### `tools/reviewer/src/cmd/review.rs`

Added 5-line block immediately after the positional artifact existence check (line 89):

```rust
// Missing --sha-only path is a bad CLI argument (exit 1, not packet error 4)
for so in &args.sha_only {
    if !Path::new(so).exists() {
        eprintln!("error: --sha-only path not found: {}", so);
        return Ok(crate::EXIT_USAGE);
    }
}
```

The positional artifact check at lines 83–89 (`EXIT_PACKET`) is untouched.

#### `tools/reviewer/tests/smoke.rs`

Two new tests appended:

| Test | AC | Asserts |
|---|---|---|
| `smoke_sha_only_missing_path_exits_usage` | AC-1, AC-2 | exit 1; stderr contains `--sha-only path not found: nonexistent-file.md` |
| `smoke_sha_only_existing_path_no_spurious_exit1` | AC-6 | existing `--sha-only` path does not exit 1; no `--sha-only path not found` in stderr |

### What was NOT changed

- `scripts/codeos-review.sh` — confirmed unchanged (AC-4)
- `tools/reviewer/src/packet.rs` — confirmed unchanged (AC-4)
- All other exit code call sites — unchanged (AC-5)

### Test results

```
test result: ok. 22 passed; 0 failed (unit)
test result: ok. 35 passed; 0 failed (smoke)
```

Total: 57 tests (22 unit + 35 smoke). All pass.

---

## Step 4 — Reconcile

### AC verification

| AC | Verdict | Evidence |
|---|---|---|
| AC-1: `--sha-only <missing>` exits 1 | PASS | `cmd/review.rs:91–96` returns `EXIT_USAGE`; `smoke_sha_only_missing_path_exits_usage` asserts exit code 1 |
| AC-2: stderr contains `--sha-only path not found: <path>` | PASS | `cmd/review.rs:94` `eprintln!("error: --sha-only path not found: {}", so)`; smoke test asserts stderr contains `--sha-only path not found: nonexistent-file.md` |
| AC-3: Positional artifact check unchanged | PASS | `cmd/review.rs:83–89` unmodified; still returns `EXIT_PACKET` (4) |
| AC-4: `packet.rs` and bash shim not touched | PASS | `git diff` shows no changes to `tools/reviewer/src/packet.rs` or `scripts/codeos-review.sh` |
| AC-5: Other exit code classes unchanged | PASS | Only new call site is the `--sha-only` block; no renumbering of existing constants or call sites |
| AC-6: Tests pass | PASS | 57/57 tests pass (`cargo test`: 22 unit + 35 smoke) |

### Scope sweep

- `scripts/codeos-review.sh` — grep confirms no diff; shim untouched
- `tools/reviewer/src/packet.rs` — grep confirms no diff
- `tools/reviewer/src/main.rs` — no changes; exit code constants unchanged
- No stale future-tense language in Steps 1/2
- `backlog/UPG-0035-reviewer-sha-only-exit-code.md` Feature Thread — updated to IN_PROGRESS in Step 1

### Step 3 R1 reviewer scope triage

| Finding | My triage | Disposition |
|---|---|---|
| AC-6 test execution not pinned to review commit | IN-SCOPE NON-BLOCKER / REJECTED | Structurally irresolvable within UPG-0035 scope; test execution output is absent from all review packets by design. Human accepted. |
