# WO-CIF-02 — Shared Gate and Dependency Preflight

- **Class:** NORMAL.
- **Goal:** a local PASS predicts the CI result, and a missing dependency fails clearly at once.

## Scope

- `dba/04-tools/ci/gate.sh fast|full`, called by CI and locally:
  - preflight: `rg`, `jq`, the pinned `node` and `npm`, Rust 1.93.0 with clippy and rustfmt,
    `cargo-audit`, and the pinned Codex qualification dependency; a miss names the binary and the
    expected version;
  - `fast`: layout contract, format, clippy;
  - `full`: `fast`, then `dba/04-tools/tests/run.sh`, dependency audit and compatibility.
- CI steps call the gate instead of repeating its commands.

## Acceptance

- `gate.sh full` passes locally exactly when CI passes on the same commit.
- Removing a required binary fails at preflight, naming it.

## Implementation notes (2026-11-26)

- The gate is `dba/04-tools/tests/gate.sh`, beside `run.sh`, rather than a new `ci/` directory.
- Codex is not a CI dependency: `ce08985` isolated the reviewer tests from Codex credentials and
  removed it from CI, so the preflight does not require it.
- The preflight checks presence of `git`, `rg`, `jq`, `python3`, `sha256sum`, `tar`, `awk`, `sed`,
  `cargo`, rustfmt and clippy; the exact versions of Rust 1.93.0, and for `full` Node v20.19.2 and
  npm 9.2.0; and `cargo-audit` for `full`. A miss lists each dependency with its expected version.
- CI calls `gate.sh fast` before the slow installs and `gate.sh full` after them.
- Main run 45767552664 failed `lifecycle_contract::a_revalidated_feature_is_current_for_the_workflow_and_the_reviewer`
  with "verifier produced no readable evidence"; the same code passed in run 45767891878 and 10 of
  10 local runs. Cause: verifier, journey and runner temp files were named from pid, subject and a
  timestamp only, so parallel tests on one subject in one process could share files and delete
  each other's evidence. Fixed with `unique_nonce`, which adds a process-wide counter, plus a test.
- Locally verified: `gate.sh full` PASS; with `jq` hidden, the preflight fails naming `jq`.
- CI run 45776215740 on b0d16e5 passed both gates and the release build, matching the local
  `gate.sh full` PASS on the same commit. Main is green again.

Status: accepted on the evidence above.
