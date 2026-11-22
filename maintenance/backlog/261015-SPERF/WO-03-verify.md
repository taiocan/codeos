# WO-03 — Verify Counts, Identity and Wall Time

NORMAL · after WO-02 · Charter: Acceptance criteria

## Goal

Prove the Charter's acceptance criteria.

## Work

- Tests assert the WO-01 counters against Charter criteria 1–4.
- The golden report is semantically identical (criterion 5).
- JamFree wall time is measured against the WO-01 baseline and recorded. It is a budget, not a
  deterministic assertion (criterion 6).

## Acceptance criteria

All six Charter criteria hold, and the human accepts the result.

## Results

**Verified (2026-10-16).** All six Charter criteria hold.

| # | Criterion | Evidence |
|---|---|---|
| 1 | One configuration resolution per command | Asserted in the golden test. JamFree: 1. |
| 2 | At most one successful cargo metadata resolution | Asserted. JamFree: 2 attempts, 1 resolved. |
| 3 | Git work independent of checkpoint count | `status_work_does_not_grow_with_checkpoints_and_persists_nothing`: two more in-progress features leave the Git, configuration and cargo counts unchanged. Disabling one memoization makes it fail. |
| 4 | Each distinct binding computed once | Asserted (computed = distinct). JamFree: 1 / 1, plus one guard check. |
| 5 | Report semantically identical | The golden report is unchanged. Both JamFree references at `d30545c` are byte-identical: `status --json` `03e017d9…` in three runs, and text `9f9625ee…`. |
| 6 | Wall time, measured separately from the counts | JamFree debug build, three runs: 6.1 s, 6.0 s and 5.9 s, against 108.9 s in WO-01. |

The design boundaries also hold.
- **One fresh snapshot per command, nothing persisted.** The snapshot lives in the process's
  `Project`, and the store cache lives in process memory. `status` leaves the JamFree and fixture
  trees unchanged, including ignored files (asserted for the fixture).
- **State-changing steps clear the snapshot.** This is tested by
  `snapshot_holds_within_a_command_until_refreshed`. The clearing is called at the three points that
  run project code: `run_mechanic`, `qualification::run` and `verify-events`.
- **The guard fails closed.** `a_report_changed_during_evaluation_is_rendered_again_then_fails_closed`
  covers an implementation edit and an appended record. A store append is reparsed:
  `an_unchanged_file_is_parsed_once_and_an_append_is_seen`.

**Accepted SPERF baseline (JamFree `d30545c`, `status --json`).**

| Measure | Value |
|---|---:|
| Wall time (user / system) | 6.1 s (5.1 / 1.0) |
| Configuration resolutions | 1 |
| cargo metadata attempts / resolved | 2 / 1 |
| Git subprocesses | 68 |
| `working_tree_state` / `product_state` | 1 / 1, plus 1 guard check |
| Store parses (verifications / decisions) | 1 / 1 |

The workflow suite (156 tests) and the reviewer suite (278 tests) pass, and clippy is clean. The
remaining ~5 s has no single hotspot (WO-02), so it is not pursued.
