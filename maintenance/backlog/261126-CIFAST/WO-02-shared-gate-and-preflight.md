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
- Find and fix the cause of the local and CI format verdicts disagreeing.

## Acceptance

- `gate.sh full` passes locally exactly when CI passes on the same commit.
- Removing a required binary fails at preflight, naming it.
- Local and CI agree on format for the whole tree.
