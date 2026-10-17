# WO-03 — Verify Meaning and Readability

NORMAL · after WO-02

## Goal

Prove that compact output is faster to scan without hiding action or authority.

## Work

- Compare old and new JamFree reports fact by fact.
- Test complete, stale, blocked, in-progress, candidate, and no-attention projects.
- Test 80-column, 120-column, redirected, and non-colour output.
- Record line count and repeated-reason count before and after.

## Acceptance criteria

1. Fact comparison finds no semantic loss.
2. Summary identifies valid, stale, blocked, and unqualified state without scrolling.
3. Every blocker has one visible actor and next action.
4. Output is structurally readable at 80 columns.
5. Lines and repeated reason text materially decrease on the JamFree report.

## Results (2026-10-17)

JamFree at DBA-28, fixture mode, before and after:

| Measure | Before | After |
|---|---|---|
| Visual lines at 80 / 100 / 120 columns | 103 / 89 / 80 | 98 / 76 / 68 |
| "accepted snapshot is no longer current" | 12 | 1 |
| "historical F9 acceptance is preserved" | 4 | 1 |
| "run the stale revalidation verifications" | 4 | 1 |
| `REVALIDATION_REQUIRED` | 24 | 7 |

- **Fact comparison:** every token of the old report is in the new one except layout words and
  the five complete bootstrap checkpoint IDs, now summarized as `5/5`; incomplete or n/a
  checkpoints are still named.
- **80 columns:** every line fits except the COMMANDS lines, which are kept whole by design.
- **Commands:** executable commands appear in text output for the first time.
- **Project states:** complete, stale, blocked, in-progress, candidate, and current-feature
  projects are covered by the unit and integration tests. A no-attention project has no
  ATTENTION/DETAILS sections.
