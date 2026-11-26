# WO-CIF-01 — Triggers, Concurrency and Step Order

- **Class:** NORMAL.
- **Goal:** one run per change, superseded PR runs cancelled, and fast failures reported first.

## Scope

- `.github/workflows/codeos-release-qualification.yml`:
  - triggers `push: branches: [main]`, `pull_request` and `workflow_dispatch`;
  - `concurrency: group: <workflow>-<ref>`, `cancel-in-progress` true only for `pull_request`;
  - format, clippy and the layout contract run before `Test`;
  - the qualified release build and binary verification run only on a main push or a manual run.
- No change to what any check verifies.

## Acceptance

- One PR update yields exactly one run; a new commit cancels the superseded PR run.
- Every main push still runs to completion.
- A deliberate format error fails within about one minute of the job starting.
