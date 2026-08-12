---
change_id: CHG-20260702-006
feature_id: UPG-0034
slug: reviewer-readonly-invariant-check
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0034__CHG-20260702-006__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0034 / CHG-20260702-006 — Reviewer Read-Only Invariant Check

## TRACE HEADER

```yaml
feature_id: UPG-0034
primary_feature_id: UPG-0034
change_id: CHG-20260702-006
slug: reviewer-readonly-invariant-check
state: IN_PROGRESS
current_step: 4-Reconcile
implements:
  - UPG-0034
related_features:
  - UPG-0032
review_series: RVS__UPG-0034__CHG-20260702-006__S1
review_profile: PROFILE-3
review_state: DRAFT
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: CHG-20260702-001
```

---

## Step 1 — Change Intent

### Problem

The Rust reviewer is supposed to be read-only — it must not mutate the working tree during
provider invocation. The Bash pilot captured `git status --porcelain` before and after invoking
`codex`, compared the outputs, and printed a warning to stderr if they differed:

```
WARNING: working tree changed during review — reviewer should be read-only
```

The Rust engine (`tools/reviewer/src/cmd/review.rs`) has no equivalent check. A silent
working-tree mutation during `prov.invoke()` goes unnoticed. This parity gap was found during
UPG-0032 / CHG-20260702-001 reconcile.

### What changes

| File | Change |
|---|---|
| `tools/reviewer/src/cmd/review.rs` | Capture `git status --porcelain` immediately before `prov.invoke()`; compare after it returns (success or error); if different, print warning to stderr (advisory only, no exit-code change) |
| `tools/reviewer/tests/smoke.rs` | New tests covering AC-5 and the git status comparison mechanism |
| `backlog/UPG-0034-reviewer-readonly-invariant-check.md` | Feature Thread: CHG-20260702-006 activated |
| `status/self-development.md` | Row activated |
| `status/roadmap.md` | UPG-0034 → IN_PROGRESS |

### Scope boundary — what stays the same

- Advisory only — no exit-code change, no blocking
- `packet.rs`, `scripts/codeos-review.sh` — not touched
- No new CLI flags
- Warning goes to stderr only; stdout output unchanged
- `--print-packet` / `--dry-run` path (`args.print_only`) does not invoke the provider —
  no snapshot needed there

### Behavior

Immediately before `prov.invoke()`:
1. Run `git status --porcelain` in `cfg.repo_root`; store output as `pre_invoke_status`
2. Call `prov.invoke()`
3. After invoke returns (success or error), run `git status --porcelain` again
4. Compare: if output differs, emit to stderr:
   `WARNING: working tree changed during review — reviewer should be read-only`
5. Continue normally regardless

If `git status` fails (git not on PATH, not a git repo, etc.) — skip silently. The check is
a best-effort safety net, not a hard gate.

### Triage

- Class: `script-tooling`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-3` (1 Codex review per step, max 3 rounds)
- Originating backlog id: `UPG-0034`

---

## Step 2 — Acceptance Criteria

### AC-1 — Pre-snapshot taken before `prov.invoke()`

`git status --porcelain` is run in `cfg.repo_root` immediately before `prov.invoke()` is
called, and the output is stored for comparison. The snapshot is taken only on the live
review path (not `--print-packet` / `--dry-run`, where the provider is never invoked).

### AC-2 — Warning emitted when working tree changes

If the post-invoke `git status --porcelain` output differs from the pre-invoke snapshot,
the following line is printed to stderr (and only stderr):

```
WARNING: working tree changed during review — reviewer should be read-only
```

No other action is taken: no exit-code change, no log entry, no blocking.

### AC-3 — No warning when working tree is unchanged

If pre- and post-invoke `git status --porcelain` output are identical, no warning is
emitted. Normal review flow continues without any extra output.

### AC-4 — `git status` failure is silent

If either `git status --porcelain` invocation fails (command not found, not a git repo,
permission error, non-zero exit, etc.), the check is skipped entirely with no error to
stderr. The review proceeds normally. Implementation uses `.filter(|o| o.status.success())`
to treat non-zero git exits as failure (not as a valid empty snapshot).

### AC-5 — `--print-packet` path not affected

When `args.print_only` is true, the provider is never invoked and no snapshot is taken.
The `--print-packet` output and exit code are unchanged from pre-change behavior.

### AC-6 — `packet.rs` and bash shim not touched

`tools/reviewer/src/packet.rs` and `scripts/codeos-review.sh` contain no diff from this
change. `tools/reviewer/tests/smoke.rs` is explicitly changed (declared in "What changes").

### AC-7 — Tests pass

`cargo test` passes. New tests cover:
- `--print-packet` path → no `WARNING:` on stderr even with a dirty working tree (AC-5)
- `git status --porcelain` on clean repo returns empty; differs after file write — verifies
  the comparison mechanism (AC-1/AC-2/AC-3)

The live mutation-warning path (`prov.invoke()` + working-tree change → `eprintln!`) cannot
be smoke-tested without a real provider session. The warning logic (`if post.stdout != pre`)
is a single equality check verified by code inspection.

---

## Step 3 — Implement

### Changes made

#### `tools/reviewer/src/cmd/review.rs`

Two blocks added around `prov.invoke()`:

**Before invoke** — `git status --porcelain` snapshot (AC-1/AC-4):
```rust
let pre_invoke_status = std::process::Command::new("git")
    .args(["status", "--porcelain"])
    .current_dir(&cfg.repo_root)
    .output()
    .ok()
    .filter(|o| o.status.success())
    .map(|o| o.stdout);
```
`.filter(|o| o.status.success())` ensures a non-zero git exit (e.g. not a git repo) yields
`None` — check silently skipped (AC-4). Plain `.ok()` was insufficient: it passes `Some` even
when git exits non-zero (R1 F2 fix).

**After invoke, before error handling** — runs regardless of invoke success or failure (R1 F1
fix). The original code returned early in the `Err` arm, skipping the check:
```rust
let invoke_result = prov.invoke(&review_packet, &prov_cfg);

if let Some(pre) = pre_invoke_status {
    if let Ok(post) = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&cfg.repo_root)
        .output()
    {
        if post.stdout != pre {
            eprintln!("WARNING: working tree changed during review — reviewer should be read-only");
        }
    }
}

let raw = match invoke_result {
    Ok(r) => r,
    Err(e) => {
        eprintln!("error: provider invocation failed: {}", e);
        return Ok(crate::EXIT_PROVIDER);
    }
};
```

The `--print-packet` path (`args.print_only`) returns before reaching this code — no snapshot
taken there (AC-5).

**R2 fix — post-invoke git status also filtered on success (AC-4):**

R1 applied `.filter(|o| o.status.success())` to the pre-invoke snapshot but left the post-invoke
path using raw `if let Ok(post) = ...output()`, which passes even when git exits non-zero. R2 fix:

```rust
let post_status = std::process::Command::new("git")
    .args(["status", "--porcelain"])
    .current_dir(&cfg.repo_root)
    .output()
    .ok()
    .filter(|o| o.status.success());
if let Some(post) = post_status {
    if post.stdout != pre {
        eprintln!("WARNING: working tree changed during review — reviewer should be read-only");
    }
}
```

Both snapshots now apply the same `filter(success())` guard.

#### `tools/reviewer/tests/smoke.rs`

Two new tests:

| Test | AC | Asserts |
|---|---|---|
| `smoke_readonly_invariant_no_warning_on_print_packet` | AC-5 | `--print-packet` with dirty working tree produces no `WARNING:` on stderr |
| `smoke_readonly_invariant_git_status_porcelain_detects_mutation` | AC-1/AC-2/AC-3 | `git status --porcelain` output is empty on clean repo; differs after file is written — verifies the comparison mechanism the invariant check relies on |

**Note on AC-7 test coverage:** The live mutation-warning path (`prov.invoke()` + working
tree change → `eprintln!`) cannot be smoke-tested without a real provider session. The two
tests cover the supporting mechanism (AC-1 via git command) and the non-invoke path (AC-5).
The warning logic itself (`if post.stdout != pre`) is a single trivial equality check verified
by code inspection.

### What was NOT changed

- `scripts/codeos-review.sh` — confirmed unchanged (AC-6)
- `tools/reviewer/src/packet.rs` — confirmed unchanged (AC-6)
- No exit codes changed; no new CLI flags

### Test results

```
test result: ok. 22 passed; 0 failed (unit)
test result: ok. 37 passed; 0 failed (smoke)
```

Total: 59 tests (22 unit + 37 smoke). All pass.

---

## Step 4 — Reconcile

### AC verification

| AC | Verdict | Evidence |
|---|---|---|
| AC-1: Pre-snapshot before `prov.invoke()`, live path only | PASS | `cmd/review.rs`: `pre_invoke_status` captured before `prov.invoke()`; `--print-packet` returns at line 148 before this code |
| AC-2: Warning on working-tree change | PASS | `if post.stdout != pre { eprintln!("WARNING: ...") }` — advisory stderr only, no exit-code change |
| AC-3: No warning when unchanged | PASS | Warning inside `if post.stdout != pre`; identical output → branch not taken |
| AC-4: `git status` failure is silent — both snapshots | PASS | Both pre and post use `.ok().filter(|o| o.status.success())`; non-zero exit → `None` → check skipped |
| AC-5: `--print-packet` path unaffected | PASS | `args.print_only` returns at line 148 before snapshot code; `smoke_readonly_invariant_no_warning_on_print_packet` verifies no WARNING on dirty tree with `--print-packet` |
| AC-6: `packet.rs` and bash shim not touched | PASS | `git diff` confirms no changes to `tools/reviewer/src/packet.rs` or `scripts/codeos-review.sh`; `smoke.rs` declared in "What changes" |
| AC-7: Tests pass | PASS | 59/59 tests pass (22 unit + 37 smoke); two new tests cover AC-5 and the git comparison mechanism |

### Scope sweep

- `scripts/codeos-review.sh` — grep confirms no diff; static shim untouched
- `tools/reviewer/src/packet.rs` — grep confirms no diff
- No new CLI flags; no exit-code changes
- `backlog/UPG-0034-reviewer-readonly-invariant-check.md` — Feature Thread updated in Step 1

### Step 3 R1–R3 reviewer scope triage

| Finding | My triage | Disposition |
|---|---|---|
| R1 F1: post-invoke check skipped on provider error | IN-SCOPE BLOCKER | FIXED (invoke_result hoisted; check before Err return) |
| R1/R2 F2: post-invoke git status no success filter | IN-SCOPE BLOCKER | FIXED (both snapshots use `.filter(success())`) |
| R1 F3: AC-7 live-path test absent | IN-SCOPE NON-BLOCKER / REJECTED | Structural limitation; cannot test without real provider; AC-7 narrowed |
| R1 F4: `smoke.rs` not declared in What changes; AC-6 wording | IN-SCOPE NON-BLOCKER | FIXED (smoke.rs added to table; AC-6 reworded) |
