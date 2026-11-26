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

## Result (2026-11-26)

`gate.sh full` was run locally under `env -i` with an empty `HOME` and only the system, Cargo and
Node paths, so `~/.codex`, `~/.claude` and user configuration were absent. Docker was not used: the
GitHub runner already provides a clean machine, and CI is green there.

| Dependency found | Test | Fix |
|---|---|---|
| Python module `yaml` (PyYAML), present locally only through `~/.local`, and in CI only through the runner image | `architecture-migration/tests/architecture-scope-migration.sh` | gate preflight checks `import yaml`; CI installs `python3-yaml` explicitly |

- With PyYAML visible, the full gate passed in the stripped environment. Codex and Claude
  credentials are not needed (`ce08985`).
- npm writes its cache to `$HOME/.npm`. It is a cache, not a dependency.
- With the module hidden, `gate.sh fast` now fails at preflight, naming PyYAML.

Status: accepted.
