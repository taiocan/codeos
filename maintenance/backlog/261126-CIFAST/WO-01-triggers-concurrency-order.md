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

## Implementation notes (2026-11-26)

- Main pushes and manual runs use a per-run concurrency group (`github.run_id`). A shared group
  without cancellation would still let a newer pending main run replace an older pending one.
- The Rust toolchain, format, `jq`/`ripgrep`, layout contract and clippy now precede Node, npm and
  `cargo install cargo-audit`, so a format error fails before the slow installs.
- Locally verified: workflow YAML parses, `run.sh` PASS, both engines `cargo fmt --check` and
  `clippy -D warnings` clean. The acceptance items need GitHub runs and are verified after the push.

## Verification on GitHub (2026-11-26, probe PR #4, closed unmerged)

- Pushes to the probe branch started no push run; each PR update started exactly one run.
- Run 45767812540 was cancelled by the next commit on the PR.
- A deliberate format error failed at Format 15 s after the job started (runs 45767600845,
  45767639116, 45767698773).
- Main run 45767552664 (dd74a9e) reached Format after 12 s and ran to completion. It failed at
  `Test` on an intermittent fault, handled under WO-02. The probe run 45767891878 on the same code
  passed every step.
- Closing a PR does not cancel its in-progress run; only a newer commit does.

Status: accepted on the evidence above.
