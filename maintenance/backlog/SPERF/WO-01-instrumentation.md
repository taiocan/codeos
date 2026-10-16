# WO-01 — Instrumentation and Golden Report

NORMAL · Charter: Acceptance criteria 1–5

## Goal

Make the cost measurable and the semantics checkable before anything is optimized.

## Work

- **Counters.** A test-visible count of configuration resolutions, cargo metadata runs, Git
  subprocesses and binding computations per command.
- **Golden report.** A representative fixture project: Bootstrap plus several features, with at
  least one accepted and one stale. Capture its full `status --json` report as the identity
  reference.
- **Baseline.** Record JamFree's counts and wall time.

## Acceptance criteria

The counters and golden report are committed, and the baseline is recorded below.

## Results

**Instrumentation.** `src/instrument.rs` counts every production subprocess by program,
configuration resolutions, cargo metadata attempts and resolutions, and `working_tree_state` and
`product_state` computations, both total and distinct by path set. Setting `CODEOS_WORKFLOW_COUNTS`
to a file makes the command write the counts there at exit. The report itself is unchanged.

**Golden report.** `tests/status_snapshot.rs` builds a DBA-26 fixture shaped like JamFree: a Cargo
backend below the root, Bootstrap 5/5, F-0001 VALID, F-0002 STALE (Contract changed after
acceptance) and F-0003 in progress. Its `status --json` must equal `tests/golden/status.json`, and
the counts are stable across runs.

**Baseline (2026-10-16).** Debug build, CodeOS `8d6f268` plus this instrumentation, `status --json`.

| Count | JamFree `d30545c` | Fixture |
|---|---:|---:|
| Configuration resolutions (bash) | 295 | 70 |
| cargo metadata attempts / resolved | 102 / 51 | 24 / 12 |
| Git subprocesses | 1,131 | 213 |
| `working_tree_state` computed / distinct | 45 / 1 | 11 / 1 |
| `product_state` computed / distinct | 2 / 1 | 1 / 1 |

JamFree wall time was 108.9 s (user 100.5 s, system 10.6 s). Half the cargo runs fail, because each
resolution tries the project root, which has no `Cargo.toml`, before `backend/`. The JamFree reports
at `d30545c` are a second identity reference: `status --json` sha256 `03e017d9…f099f09e`, text
`status` sha256 `9f9625ee…99920e17`.
