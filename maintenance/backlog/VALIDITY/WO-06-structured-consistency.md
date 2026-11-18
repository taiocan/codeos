# WO-06 — Structured-Value Consistency in Preflight

NORMAL · after WO-03 · approved by the human 2026-11-18 (from JamFree proposal U3a)

## Goal

Preflight reports a contradiction between two governed structured fields that state the same
value, before approval.

## Rules

- Compare only values CodeOS already identifies as structured: parsed frontmatter, table cells
  with a known column, declared ranges.
- No number extraction from prose and no heuristic semantic matching.

## Work

First establish which structured fields in the Specification Package and Architecture Scope state
the same value. If none do, report that and stop. The measured case (one-hour vs two-hour Reality
Basis rows, F-0001 rev 7 / F-0002 rev 9) was a contradiction between Reality Basis cells and
scenario prose; WO-06 covers it only if both sides are structured.

## Acceptance criteria

1. A fixture where two structured fields disagree → preflight is `BLOCKED(DEFECT)`, naming both.
2. A prose-only difference → no finding.

## Result (2026-11-18): no applicable structured values; stopped as the WO prescribes

Examined in JamFree F-0001 (the measured case) and its Event Schema:

| Where the threshold appears | Form |
|---|---|
| Reality Basis "Supported conditions" / "Unavailable behavior" | prose sentences in table cells ("every 2 hours … (12 departure times)", "more than two such hours") |
| Event Schema payload | a typed field name (`integer`) with a prose range ("from 10 through 12", "from 0 through 2") |
| Scenarios and falsification rows | Gherkin and prose |

No two governed structured fields state the same value, so there is nothing WO-06 can compare
without extracting numbers from prose, which the human excluded. Nothing is implemented.

Making the class checkable would require a structured declaration, for example a typed range syntax
for Event Schema payload fields that a Reality Basis row could reference. That changes the
Specification Package format, so it is a separate, consequential decision, not part of WO-06. The
measured contradiction stays with the deferred "spec-only Stage 8 review and prose-number lint"
item.
