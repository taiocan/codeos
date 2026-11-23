# WO-DM-08 — In-Report Guidance

- **Class:** NORMAL, PowerBI repository (`projects/dba-monitor`).
- **Goal:** a novice programmer and a CodeOS developer can each interpret every page without
  outside help.

## Scope

- Each page gets a "How to read this page" panel below its visuals (the page scrolls; collapsing
  would need hand-written bookmark state, which is fragile) with two short parts:
  - **Novice:** what question the page answers and how to read each visual.
  - **CodeOS developer:** which CodeOS record each figure comes from, which figures are labelled
    proxies, which patterns deserve a look, and where to drill down next.
- Each visual gets a one-line subtitle.
- Covers pages 1–4, and the WO-09 page once it exists.

## Rules

- Guidance explains interpretation. Definitions stay in `glossary.md`; the guidance names glossary
  terms and does not redefine them.
- No thresholds, targets, or "good" values (boundary 3). "Look for" statements are allowed.
- At most about 120 words per audience per page.

## Verification

- Every page has the box and every visual has a subtitle.
- Every glossary term a box names exists in `glossary.md`.
- The human checks the pages on the VM.

## Acceptance

The verification passes and the human accepts.
