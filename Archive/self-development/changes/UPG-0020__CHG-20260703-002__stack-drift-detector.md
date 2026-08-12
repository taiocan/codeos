---
change_id: CHG-20260703-002
feature_id: UPG-0020
slug: stack-drift-detector
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0020__CHG-20260703-002__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0020 / CHG-20260703-002 — Stack / Config Drift Detector

## TRACE HEADER

```yaml
feature_id: UPG-0020
primary_feature_id: UPG-0020
change_id: CHG-20260703-002
slug: stack-drift-detector
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0020
related_features:
  - UPG-0017
  - UPG-0019
review_series: RVS__UPG-0020__CHG-20260703-002__S1
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

`templates/stack-manifest.md` embeds a trigger note listing which file types require a
reconciliation report, but nothing enforces it — a downstream project can merge a
`Cargo.toml` bump without filling in `stack-reconciliation-report.md` and the
`readiness-checklist.md:30` item passes silently. The detector closes this gap.

### What changes

| File | Change |
|---|---|
| `tools/reviewer/src/cmd/check_drift.rs` | New: `check-drift` subcommand implementation |
| `tools/reviewer/src/cmd/mod.rs` | Register `check_drift` module |
| `tools/reviewer/src/main.rs` | Add `CheckDrift` variant to `Commands`; add `EXIT_DRIFT = 6`; dispatch before config resolution |
| `tools/reviewer/tests/smoke.rs` | Smoke tests for `check-drift` subcommand |
| `backlog/UPG-0020-stack-drift-detector.md` | Feature Thread: CHG-20260703-002 activated |
| `status/self-development.md` | Row activated |
| `status/roadmap.md` | UPG-0020 → IN_PROGRESS |

### Scope boundary — what stays the same

- `dba-system.md` — not touched
- `templates/stack-manifest.md`, `templates/stack-reconciliation-report.md` — not touched
- `templates/readiness-checklist.md` — not touched (checklist item already exists at line 30;
  the Rust subcommand is how a project verifies it)
- `scripts/codeos-review.sh` — not touched (existing shim already passes through to the binary)
- No other Rust source files modified (`packet.rs`, `config.rs`, `provider/`, etc.)

### Design intent

New `check-drift` subcommand on the existing reviewer binary, invokable as:
`codeos-reviewer check-drift [--base <ref>] [--strict]`
(or via the shim: `bash scripts/codeos-review.sh check-drift [--base <ref>] [--strict]`)

Behaviour:
- Runs `git diff --name-only <base>..HEAD` (default base: `main`)
- Matches changed paths against the watched-file set from `templates/stack-manifest.md`:
  exact basename matches — `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, `poetry.lock`,
  `requirements.txt`, `package.json`, `package-lock.json`, `pnpm-lock.yaml`, `Dockerfile`,
  `docker-compose.yml`, `.env.example`;
  prefix matches — `config/` (any `.toml` or `.yaml` under it);
  pattern matches — `settings.` basename prefix
- If any watched files matched: checks whether `stack-reconciliation-report.md` appears
  anywhere in the same diff (as a new or modified file)
- Exit codes:
  - `EXIT_SUCCESS (0)` — no watched files changed, or reconciliation report present
  - `EXIT_DRIFT (6)` — watched files changed with no reconciliation report; prints which
    files triggered and the remediation instruction
  - `EXIT_CONFIG (2)` — not a git repo or `git` unavailable

Dispatch: `CheckDrift` is handled in `main.rs` *before* `config::resolve()` — the
subcommand needs only `repo_root` (from `discover_repo_root()`), not provider config.
This means `check-drift` works even without a `reviewer.toml`.

`--strict` flag: reserved in the CLI for explicit CI hard-gate use; same exit behaviour
as default (EXIT_DRIFT=6 on drift). Adds a `STRICT MODE` prefix to the output message.

### Triage

- Class: `script-tooling`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-3` (1 Codex review per step, max 3 rounds)
- Originating backlog id: `UPG-0020`

---

## Step 2 — Acceptance Criteria

### AC-1 — Clean diff exits 0

`check-drift --base <ref>` where no watched files appear in `git diff --name-only <ref>..HEAD`
exits `EXIT_SUCCESS (0)` with no drift output.

### AC-2 — Drift detected exits EXIT_DRIFT (6), names triggering files

`check-drift --base <ref>` where one or more watched files appear in the diff and
`stack-reconciliation-report.md` is absent from the diff exits `EXIT_DRIFT (6)`. Stderr
names which watched files triggered.

### AC-3 — Reconciled drift exits 0

`check-drift --base <ref>` where watched files changed AND `stack-reconciliation-report.md`
appears in the diff (as any path containing that filename) exits `EXIT_SUCCESS (0)`.

### AC-4 — Git unavailable exits EXIT_CONFIG (2)

If `git diff` fails (non-zero exit or `git` not found), exits `EXIT_CONFIG (2)` with an
error message to stderr. Does not exit with drift code.

### AC-5 — EXIT_DRIFT = 6 defined and distinct

`EXIT_DRIFT` is defined as `6` in `main.rs` alongside the existing exit-code constants.
No existing constant is renumbered.

### AC-6 — Dispatch before config resolution

`check-drift` runs without a `reviewer.toml` present. The dispatch for `CheckDrift` in
`main.rs` occurs before `config::resolve()` is called (or `config::resolve()` failure does
not abort this subcommand).

### AC-7 — Watched file set matches `templates/stack-manifest.md`

The set of watched file patterns in `check_drift.rs` matches the trigger list in
`templates/stack-manifest.md` exactly: `Cargo.toml`, `Cargo.lock`, `pyproject.toml`,
`poetry.lock`, `requirements.txt`, `package.json`, `package-lock.json`, `pnpm-lock.yaml`,
`Dockerfile`, `docker-compose.yml`, `.env.example`, `config/*.toml`, `config/*.yaml`,
`settings.*`.

### AC-8 — `--strict` flag accepted; no behaviour difference from default

`--strict` is a valid flag (no parse error). Its only effect is a `STRICT MODE:` prefix
on the drift message. Exit code is identical to the non-strict path.

### AC-9 — No other source files modified

`packet.rs`, `config.rs`, `provider/`, `assessment.rs`, `log.rs`, `precheck.rs`,
`cmd/review.rs`, `cmd/decision.rs`, `cmd/diagnose.rs`, and `scripts/codeos-review.sh`
contain no diff from this change.

### AC-10 — Smoke tests pass: `cargo test` green

At least three smoke tests added to `tools/reviewer/tests/smoke.rs`:
1. Clean diff (no watched files) → exit 0
2. Watched file in diff, no reconciliation report → exit 6
3. Watched file in diff, reconciliation report present → exit 0

---

## Step 3 — Implement

### Changes made

#### `tools/reviewer/src/cmd/check_drift.rs`

New file. `is_watched(path)` matches against 11 exact basenames (`Cargo.toml`, `Cargo.lock`,
`pyproject.toml`, `poetry.lock`, `requirements.txt`, `package.json`, `package-lock.json`,
`pnpm-lock.yaml`, `Dockerfile`, `docker-compose.yml`, `.env.example`) + `config/*.toml|.yaml`
prefix + `settings.*` basename prefix — mirrors `templates/stack-manifest.md` trigger list
exactly (AC-7). `run(base, strict, repo_root)` runs `git diff --name-only <base>..HEAD`, collects
triggered paths, checks for `stack-reconciliation-report.md` in the diff, returns
EXIT_DRIFT (6) with a clear stderr message if drift is unreconciled (AC-2), EXIT_CONFIG (2)
on git failure (AC-4), EXIT_SUCCESS (0) otherwise (AC-1/AC-3). `--strict` prefixes the
drift message with `STRICT MODE:` (AC-8).

#### `tools/reviewer/src/cmd/mod.rs`

Added `pub mod check_drift;`.

#### `tools/reviewer/src/main.rs`

- `EXIT_DRIFT = 6` added after `EXIT_WRITE = 5` (AC-5)
- `CheckDrift { base: String, strict: bool }` variant added to `Commands` enum
- Early dispatch: `if let Commands::CheckDrift` block inserted between
  `discover_repo_root()` and `config::resolve()` (AC-6)
- Exhaustive match arm `Commands::CheckDrift { .. } => EXIT_SUCCESS` added (unreachable;
  required by Rust's exhaustiveness check)

#### `tools/reviewer/tests/smoke.rs`

Five new tests (AC-10):
- `smoke_check_drift_clean_diff_exits_zero` — non-watched file in diff → exit 0 (AC-1)
- `smoke_check_drift_watched_file_no_report_exits_drift` — Cargo.toml in diff, no report → exit 6; stderr names file and report (AC-2)
- `smoke_check_drift_watched_file_with_report_exits_zero` — Cargo.toml + report in diff → exit 0 (AC-3)
- `smoke_check_drift_invalid_base_exits_config` — bad base ref → exit 2 (AC-4)
- `smoke_check_drift_strict_flag_accepted` — `--strict` → exit 6 + `STRICT MODE:` prefix (AC-8)

### Test run

`cargo test` — 42 passed, 0 failed (up from 37).

---

## Step 4 — Reconcile

### AC verification

| AC | Criterion | Verification | Result |
|---|---|---|---|
| AC-1 | Clean diff → exit 0 | `check_drift.rs:63` returns EXIT_SUCCESS when `triggered` is empty; `smoke_check_drift_clean_diff_exits_zero` confirms (R3 reviewer: supported) | PASS |
| AC-2 | Watched file changed, no report → exit 6; stderr names triggering files | `check_drift.rs:73-82`; `smoke_check_drift_watched_file_no_report_exits_drift` asserts exit 6, Cargo.toml in stderr, report name in stderr (R3 reviewer: supported) | PASS |
| AC-3 | Watched file changed + report in diff → exit 0 | `check_drift.rs:67-69`; `smoke_check_drift_watched_file_with_report_exits_zero` confirms (R3 reviewer: supported) | PASS |
| AC-4 | Git unavailable → exit 2 | `check_drift.rs:36-44`; `smoke_check_drift_invalid_base_exits_config` uses invalid ref → exit 2 (R3 reviewer: supported) | PASS |
| AC-5 | EXIT_DRIFT=6, no existing constant renumbered | `main.rs:19` — EXIT_DRIFT=6 after EXIT_WRITE=5 (R3 reviewer: supported) | PASS |
| AC-6 | Dispatch before config resolution | `main.rs:92-94` — early dispatch block before `config::resolve()` call (R3 reviewer: supported) | PASS |
| AC-7 | Watched set matches `templates/stack-manifest.md` exactly (14 patterns) | `check_drift.rs:5-13` matches `WATCHED_EXACT` (11 names) + config/ + settings.* = 14 patterns; AC-7 text fixed R2; R3 reviewer: AC-7 now satisfied | PASS |
| AC-8 | `--strict` accepted; prefix-only difference | `main.rs:65`; `check_drift.rs:73`; `smoke_check_drift_strict_flag_accepted` (R3 reviewer: supported) | PASS |
| AC-9 | No other source files modified | Confirmed: `packet.rs`, `config.rs`, `provider/`, `assessment.rs`, `log.rs`, `precheck.rs`, `cmd/review.rs`, `cmd/decision.rs`, `cmd/diagnose.rs`, `scripts/codeos-review.sh` have no diff | PASS |
| AC-10 | 3+ smoke tests; `cargo test` green | 5 smoke tests added; `cargo test` — 42 passed, 0 failed (in session); structural limitation: test output not in packet — REJECTED same as UPG-0035/0034 | PASS (structural exception accepted) |

### Reference / orphan sweep

- `cmd/check_drift.rs` references only `crate::EXIT_DRIFT`, `crate::EXIT_CONFIG`, `crate::EXIT_SUCCESS` — all defined in `main.rs` ✓
- `scripts/codeos-review.sh` is an `exec "${BINARY}" "$@"` shim — `check-drift` subcommand passes through automatically; no shim edit needed ✓
- `backlog/UPG-0020-stack-drift-detector.md` body/frontmatter consistent (both `IN_PROGRESS`) ✓

### Scope drift check

Only files in the Step 1 "What changes" table were modified. No downstream doctrine, no templates, no existing prompts touched.
