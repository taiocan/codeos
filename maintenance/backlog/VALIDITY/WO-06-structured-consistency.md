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
