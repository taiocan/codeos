# WO-CIF-04 — Hermetic Tests

- **Class:** NORMAL.
- **Goal:** the full gate passes without developer-machine state.

## Scope

- Run `gate.sh full` in a clean container without `~/.codex`, `~/.claude`, Docker or a developer
  `$HOME`.
- Fix each host dependency found in the test that has it, so that tests never read installed or
  authenticated tools or personal configuration.

## Acceptance

- `gate.sh full` passes in the clean container.
- Each dependency fixed is listed here with its test.
