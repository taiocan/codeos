# WO-SD-02 — Compiled-Advisory Flag

- **Class:** NORMAL. It changes the DBAMON static snapshot and the Code health report page.
- **Goal:** the monitor stops presenting lockfile-only advisories as exposure, without hiding them.

## Scope

- `codeos-static-analytics.sh` adds `compiled: true | false` to each `advisories.jsonl` row, from
  `cargo tree --target all -e normal,build,dev -i <package>@<version>`. If the lookup fails, the
  value is null, never a guess.
- The Code health page adds the column and a "Vulnerabilities (compiled)" card next to the
  unchanged total.
- The glossary defines a compiled advisory.

## Verification

- The stub-cargo test covers `true`, `false` and the null case.
- In the real snapshot, the 4 lockfile-only rows show `compiled: false` and JamFree's url row shows
  `compiled: true`.
- The human checks the page on the VM.

## Acceptance

The verification passes and the human accepts.
