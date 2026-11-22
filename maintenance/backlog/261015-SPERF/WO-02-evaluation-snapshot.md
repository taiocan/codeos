# WO-02 — Command-Scoped Evaluation Snapshot

NORMAL · after WO-01 · Charter: Goal, Invariant

## Goal

Evaluate every checkpoint against one snapshot that is built once per command.

## Work

1. Resolve the project configuration once.
2. Run cargo metadata once and derive the implementation paths from it.
3. Memoize each read-only Git query instead of batching it. Accepted evidence binds the exact
   per-path hash, and a split repository-wide diff is not equivalent to `git diff HEAD -- <path>`
   (rename detection and pathspec matching differ).
4. Memoize each binding by path set and binding type.
5. Pass the Bootstrap result into feature evaluation.
6. Reuse each feature's assessment and checkpoint report for the health and attention sections.

Steps 5 and 6 run only if measurement after 1–4 shows they still matter.

## Acceptance criteria

The workflow and reviewer suites pass, and the WO-01 golden report is unchanged.

## Results

**Implemented (2026-10-16).**
- **Snapshot.** `Project` holds a command-scoped snapshot: configuration, cargo metadata,
  implementation paths, implementation and product state, and read-only Git history queries.
- **Refresh points.** It is cleared after each step that runs project code: `run_mechanic`,
  `qualification::run` and the `verify-events` command.
- **Concurrent-mutation guard.** `status`, `next` and `journey` render inside
  `Project::consistent`. If the governed records or the implementation changed during the render,
  it renders once more; a second change fails closed.
- **Store cache.** `src/store_cache.rs` parses each JSONL store once per process and reparses when
  the file's length or mtime changes.
- **Deferred.** Steps 5 and 6 are not needed.

**What dominated.** With subprocess fan-out removed, JamFree `status` still took 78 s. Stack sampling
put about 87% of it in re-parsing the 4.1 MB `verifications.jsonl` (353 parses per command;
`decisions.jsonl` 139). WO-01 had not counted store parses. The Charter's attribution to subprocess
fan-out was therefore incomplete.

| JamFree `d30545c`, `status --json` | WO-01 | WO-02 |
|---|---:|---:|
| Wall (user / system) | 108.9 s (100.5 / 10.6) | 6.1 s (5.1 / 1.0) |
| Configuration resolutions | 295 | 1 |
| cargo metadata attempts / resolved | 102 / 51 | 2 / 1 |
| Git subprocesses | 1,131 | 68 |
| `working_tree_state` computed / distinct | 45 / 1 | 1 / 1, plus 1 guard check |
| `verifications.jsonl` / `decisions.jsonl` parses | 353 / 139 | 1 / 1 |

Of the 68 Git calls, 42 are the implementation state and its guard check (7 paths × 3 calls, twice).
The remaining time is spread across record cloning, hashing of governed files and review-record
scans; no single hotspot remains.

**Identity.** The golden report is unchanged, and so are both JamFree references
(`status --json` sha256 `03e017d9…`, text `9f9625ee…`). The workflow suite (155 tests) and the
reviewer suite (278 tests) pass. Two tests that reuse one `Project` across simulated commands now
refresh it explicitly.
