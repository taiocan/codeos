---
change_id: CHG-20260707-004
feature_id: UPG-0040
slug: config-test-env-var-race
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0040__CHG-20260707-004__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0040 / CHG-20260707-004 — Fix Flaky config::tests Race on CODEOS_REVIEWER_PROVIDER Env Var

## TRACE HEADER

```yaml
feature_id: UPG-0040
primary_feature_id: UPG-0040
change_id: CHG-20260707-004
slug: config-test-env-var-race
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0040
related_features:
  - UPG-0023
review_series: RVS__UPG-0040__CHG-20260707-004__S1
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

`tools/reviewer/src/config.rs`'s `#[cfg(test)] mod tests` has 5 tests
(`default_provider_is_codex`, `cli_flag_overrides_env`, `env_var_overrides_toml`,
`toml_overrides_default`, `unknown_provider_returns_err`) — **all five** call
`std::env::remove_var("CODEOS_REVIEWER_PROVIDER")` and/or `std::env::set_var(...)` directly
(confirmed by reading the file fresh before drafting this Step 1, not just trusting the
backlog's claim). Rust's default test harness runs tests in parallel threads within the same
process; environment variables are process-global, not thread-local, so any two of these
tests running concurrently race on the same variable. Discovered during UPG-0023's Step 4
verification: default `cargo test` intermittently failed `toml_overrides_default`, while
`cargo test -- --test-threads=1` passed all 120 tests deterministically.

### What changes

| File | Change |
|---|---|
| `tools/reviewer/src/config.rs` | `#[cfg(test)] mod tests` gains a process-wide `static Mutex<()>`; each of the 5 tests acquires it before touching the env var |
| `backlog/UPG-0040-config-test-env-var-race.md` | Feature Thread: CHG-20260707-004 activated (done) |
| `backlog/features.md` | Row → IN_PROGRESS (done) |
| `status/self-development.md` | Row activated (done) |
| `status/roadmap.md` | UPG-0040 → IN_PROGRESS (done) |

### Scope boundary — what stays the same

- **No production code changes.** `resolve()` and `resolve_provider()` (lines 1-121, outside
  `#[cfg(test)]`) are untouched — same signatures, same precedence logic (CLI flag > env var
  > `reviewer.toml` > compiled-in default). Matches the backlog's Guardrail exactly.
- **No test assertions weakened or removed.** All 5 tests keep their exact existing
  assertions; the only addition is a lock-acquisition line at the top of each.
- **No new dependency.** `std::sync::Mutex` is standard library — no `Cargo.toml` change.
- **No other test file touched.** `grep -rln "CODEOS_REVIEWER_PROVIDER" tools/reviewer/src/
  tools/reviewer/tests/` returns only `config.rs` — confirmed fresh, not assumed — so
  `tests/smoke.rs` and every other `#[cfg(test)]` module in the crate genuinely cannot race
  on this specific variable; a mutex scoped to `config.rs`'s own test module is sufficient.

### Design intent

Chose the backlog's first candidate approach (a process-wide mutex guarding env-var mutation
in tests) over the second (restructuring `resolve()` to accept an injected value) because the
mutex is a **pure test-only change** — it doesn't touch `resolve()`/`resolve_provider()`'s
signatures at all, matching the backlog's own Scope statement ("`config.rs`'s test module
only. No production-code behavior change") more literally than the injection alternative
would (which requires adding a parameter to production-code function signatures, even if the
precedence *logic* itself stays identical).

```rust
static ENV_VAR_LOCK: Mutex<()> = Mutex::new(());
```

Each test acquires the lock first, recovering from a poisoned mutex rather than propagating a
panic into unrelated tests if an earlier test in the module fails while holding it:

```rust
let _guard = ENV_VAR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
```

The guard is held for the test's full body (RAII, released at function end via `Drop`),
serializing every env-var-touching test in this module against every other one — eliminating
the race without changing what any individual test verifies.

### Triage

- Class: `script-tooling`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-3`
- Originating backlog id: `UPG-0040`

---

## Step 2 — Acceptance Criteria

### Race eliminated

**AC-1 — Default (parallel) `cargo test` passes reliably across repeated runs**
`cargo test` (default parallel threading, the mode that previously failed intermittently)
passes with zero failures across at least 20 consecutive runs.
_Verify in Step 4:_ a shell loop running `cargo test` 20+ times, confirming 0 failures.

**AC-2 — Single-threaded baseline still passes**
`cargo test -- --test-threads=1` (the pre-existing deterministic workaround) continues to
pass in full — the fix must not regress the mode that already worked.
_Verify in Step 4:_ run it once; confirm 0 failures, same test count as before this change.

### No production-code change

**AC-3 — Diff confined to the test module**
Every changed line in `config.rs` is inside `#[cfg(test)] mod tests` (starting at the
existing `#[cfg(test)]` attribute); zero lines outside it change.
_Verify in Step 4:_ `git diff` line-range inspection confirming all hunks fall after the
`#[cfg(test)]` marker.

**AC-4 — `resolve()`/`resolve_provider()` signatures and precedence logic byte-identical**
Both functions' signatures and bodies (lines 30-91, per the current file) are unchanged.
_Verify in Step 4:_ diff those specific line ranges; confirm empty.

**AC-5 — All 5 existing test assertions unchanged**
Every `assert_eq!`/`assert!` in the 5 tests is textually identical to before this change —
the only addition is a lock-acquisition line per test.
_Verify in Step 4:_ diff each test function; confirm only an added line, no assertion text
changed.

### Design correctness

**AC-6 — Poisoned-mutex recovery, not propagation**
The lock-acquisition pattern uses `.unwrap_or_else(|e| e.into_inner())`, not a bare
`.unwrap()` — so one test panicking while holding the lock doesn't cascade-poison every
subsequent test in the module.
_Verify in Step 4:_ grep the actual pattern used in the diff.

**AC-7 — No new dependency**
`std::sync::Mutex` is standard library; `Cargo.toml`/`Cargo.lock` are unchanged.
_Verify in Step 4:_ `git diff --stat -- tools/reviewer/Cargo.toml tools/reviewer/Cargo.lock`
is empty.

### Cross-reference integrity

**AC-8 — Full suite unaffected in count or outcome, both threading modes**
The complete test suite (unit tests in `config.rs` and elsewhere, plus `tests/smoke.rs`)
has the identical number of `#[test]` functions before and after this change — the fix adds
only lock-acquisition lines inside existing test bodies, never a new or removed `#[test]`
function — and passes fully in both default-parallel and single-threaded modes. (No specific
total count is hardcoded here, since it drifts as unrelated changes land elsewhere in the
crate; what matters is that *this* change doesn't alter it. No `git stash` or other
working-tree mutation is used to verify this, to avoid disturbing unrelated uncommitted work
in a workspace that may not be clean.)
_Verify in Step 4:_ `git diff -- tools/reviewer/src/config.rs` — count added/removed
`#[test]` occurrences (expect 0 added, 0 removed); run `cargo test -- --test-threads=1`
(note: harness flags go after `--`) and confirm 0 failures; separately run default `cargo
test` (parallel) and confirm 0 failures.

---

## Step 3 — Implement

### What was done

| File | Change |
|---|---|
| `tools/reviewer/src/config.rs` | `#[cfg(test)] mod tests` gains `static ENV_VAR_LOCK: Mutex<()>`; each of the 5 tests acquires it (`.unwrap_or_else(\|e\| e.into_inner())`) as its first line |

### Verification (AC-1 through AC-8), all against real runs, no mocks

- **AC-1**: 20 consecutive default-parallel `cargo test` runs — **0 failures** (previously
  intermittent).
- **AC-2**: `cargo test -- --test-threads=1` — 26 unit + 110 smoke, 0 failures, unchanged
  from before this change.
- **AC-3**: `git diff` hunk inspection — all `+`/`-` content lines fall inside
  `#[cfg(test)] mod tests`; the "fn find_toolkit_root" label on the first hunk header is
  git's nearest-enclosing-item context marker, not a changed line inside that function.
- **AC-4**: `resolve()`/`resolve_provider()` (and every other production function) have zero
  diff lines — confirmed by the hunk-location check above.
- **AC-5**: `git diff -- tools/reviewer/src/config.rs | grep "^-"` (excluding the file
  header; scoped to this one file — the repo-wide diff also includes this change's own
  declared bookkeeping edits, e.g. `-status: PROPOSED` in the backlog/status files, which is
  expected and not what this AC is about) shows nothing removed from `config.rs` — every
  test's existing assertions are untouched, only a lock-acquisition line added per test.
- **AC-6**: `grep -c "unwrap_or_else(|e| e.into_inner())"` → 5, one per test, confirming
  poison-recovery (not a bare `.unwrap()`) is used uniformly.
- **AC-7**: `git diff --stat -- Cargo.toml Cargo.lock` → empty.
- **AC-8**: `#[test]` occurrence count in the diff → 0 added/removed; full suite is 136
  (26+110) in both single-threaded and default-parallel modes, both 0 failures.

### Scope check

`git status --short` shows only `tools/reviewer/src/config.rs` as content, plus the declared
backlog/status bookkeeping — no other file touched.

---

## Step 4 — Reconcile

### Acceptance criteria verification (fresh evidence)

| AC | Verified by | Result |
|---|---|---|
| AC-1 Race eliminated | Fresh 20-run default-parallel stress test, 0 failures | PASS |
| AC-2 Single-threaded baseline | `cargo test -- --test-threads=1`, 136 passing, 0 failed | PASS |
| AC-3 Diff confined to test module | Hunk-location inspection, all changes inside `mod tests` | PASS |
| AC-4 Production signatures/logic unchanged | Zero diff lines outside the test module | PASS |
| AC-5 Assertions unchanged | `git diff -- tools/reviewer/src/config.rs \| grep "^-"` (scoped to this file) shows nothing removed | PASS |
| AC-6 Poison recovery used | 5× `unwrap_or_else(\|e\| e.into_inner())`, one per test | PASS |
| AC-7 No new dependency | `Cargo.toml`/`Cargo.lock` diff empty | PASS |
| AC-8 Suite count/outcome unaffected | 0 `#[test]` added/removed; 136 passing both modes | PASS |

### Cross-reference sweep

- `git status --short` — only `tools/reviewer/src/config.rs` as content, plus declared
  bookkeeping.
- No other file references `ENV_VAR_LOCK` or depends on this test module's internals — swept
  `tools/reviewer/tests/smoke.rs` and confirmed it doesn't touch `CODEOS_REVIEWER_PROVIDER`
  at all (matches Step 1's fresh-grep finding).

### Reviewer scope triage (Step 4 findings)

Step 1 R1 (REQUEST CHANGES) found one genuine IN-SCOPE BLOCKER: an unsupported repo-wide
exclusivity claim, fixed by backing it with fresh grep evidence; R2 clean. Step 2 R1/R2 found
three genuine IN-SCOPE BLOCKERs across two rounds: a hardcoded test-count number sourced from
outside the packet, a command-syntax typo (missing `--` separator), and an unsafe `git
stash`-based verification procedure in what is actually a dirty workspace — all fixed,
AC-8 redesigned around a stash-free comparison; R3 clean. Step 3 R1: no findings. This Step 4
round: no findings.

### Outcome

All 8 ACs verified against the final artifacts with fresh evidence (table above), including
two independent 20-run stress tests (Step 3 and Step 4) both showing 0 failures. No in-scope
blockers open. No scope drift — zero production-code lines touched, no new dependency. Step
4 R2 NO OBJECTION; human APPROVE_STAGE recorded (2026-07-07). Change record,
`status/self-development.md`, `status/roadmap.md`, `backlog/features.md`, and
`backlog/UPG-0040-config-test-env-var-race.md` updated to COMPLETE in this same pass,
following that approval.
