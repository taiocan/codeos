# CIFAST — Fast, Trustworthy CodeOS CI

Planning context only. Git remains authoritative. Assessed and approved by the human 2026-11-26.
NORMAL: CI is not a governed authority. Qualification and issuance run locally through
`build-qualified-release.sh` and `issue-configuration.sh`.

## Origin

The human proposed splitting CI into fast PR checks, a full suite and release qualification: three
workflows, PR-only triggers, concurrency, caching, self-contained tests and a dependency preflight.

## Evidence (GitHub Actions, taiocan/codeos, 2026-11-14..26)

- 46 runs, 43 failed. 39 were push and 7 pull_request. Only PR #3 produced push+PR duplicates
  (7 pairs, about 15% of runs). The repository is public, so Actions minutes are free.
- main was red for 31 consecutive pushes (2026-11-14..23), every one failing at `Test`, while
  `run.sh` passed locally. PR #3 needed 7 attempts to close CI-environment gaps (ripgrep, Codex
  credentials, gitleaks permissions and history).
- One run takes about 8 min: dependency install 3 min (mostly `cargo install cargo-audit`), tests
  3 min, release build 76 s, format and clippy 17 s. Format and clippy run after the tests.
- Local `cargo fmt --check` reported diffs that CI did not. Resolved 2026-11-26: PR #3 had
  formatted `origin/main`, and local `main` had not yet merged it. After the merge, the only diffs
  were in unpushed local commits, now formatted. Local and CI formatting agree.

## Assessment

| Proposal item | Decision |
|---|---|
| PR checks only on `pull_request` | Modified: `push` on `main` only, plus `pull_request` and `workflow_dispatch`. Most work is pushed straight to main. |
| Concurrency with cancel-in-progress | Pull requests only; every main run is kept so a red commit can be identified |
| Fast checks first | Step order within the existing job |
| Full suite once after fast checks | Follows from the step order |
| Cache Cargo dependencies and targets | Accepted; the largest time win |
| Self-contained tests | Accepted; the actual cause of 9 days of red main |
| Dependency preflight | One script shared by CI and local runs |
| Release-only steps | Release build and binary verification on main and manual runs only. Audit and compatibility stay on every run. |
| Three workflow files | Rejected: each reinstalls the toolchain and duplicates configuration |
| "Roughly halve CI usage" | Not true here; the value is faster feedback and a trustworthy main signal |

Addition beyond the proposal: one gate script shared by local runs and CI, so that a local PASS
predicts the CI result.

## Work orders

| WO | Goal |
|---|---|
| [WO-01](WO-01-triggers-concurrency-order.md) | One run per change, superseded PR runs cancelled, fast failures first |
| [WO-02](WO-02-shared-gate-and-preflight.md) | Local and CI run the same gate with an explicit dependency preflight |
| [WO-03](WO-03-caching.md) | Warm runs skip dependency compilation |
| [WO-04](WO-04-hermetic-tests.md) | The full gate passes without developer-machine state |

Order: WO-01, WO-02, WO-03, WO-04. `origin/main` was merged into local `main` on 2026-11-26.
