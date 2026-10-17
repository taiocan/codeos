# WO-02 — Implement Compact Tables

NORMAL · after WO-01

## Goal

Render the approved terminal contract without changing status derivation.

## Work

- Refactor only `monitor::render` and presentation helpers.
- Add aligned summary, health, feature, and attention tables.
- Group identical feature reasons; move evidence and commands to compact detail rows.
- Wrap long values at terminal width; never truncate identifiers or commands silently.

## Acceptance criteria

1. `status --json` is byte-identical.
2. Exit codes and row ordering are unchanged.
3. JamFree's four identical stale reasons render once in health summary.
4. Each attention item retains changed surfaces, actor, reason, and command.
5. Existing and new renderer tests pass.

## Results (2026-10-17)

Implemented in `monitor.rs` (`render_width`, `table`, `wrap`, `grouped`, `bootstrap_summary`).
`status --json` on JamFree is byte-identical and the exit code is unchanged (2). JamFree's four
identical stale reasons render once per domain, and "same as" replaces three repeated domains.
New tests cover grouping, wrapping, and table fitting; the status integration tests check the
same facts in the new layout, determinism, and the 80-column fit.
