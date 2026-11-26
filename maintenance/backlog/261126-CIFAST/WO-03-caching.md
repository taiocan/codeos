# WO-CIF-03 — Caching

- **Class:** NORMAL.
- **Goal:** warm runs skip dependency compilation.

## Scope

- `Swatinem/rust-cache` keyed for both engines (`workflow/engine`, `reviewer/engine`).
- A prebuilt `cargo-audit` (for example `taiki-e/install-action`) instead of `cargo install`,
  pinned to one version that the gate preflight also checks. CI currently installs the latest
  (0.23.0) while local runs use 0.22.2.

## Acceptance

- On a warm run the dependency install step takes under 30 s.
- Before and after timings of the install, test and build steps are recorded here.

## Implementation notes (2026-11-26)

- `Swatinem/rust-cache@v2` caches both engines' dependencies and build output, keyed after the
  locked toolchain install.
- `taiki-e/install-action@v2` installs `cargo-audit@0.23.0` prebuilt, replacing the 2.5–3.5 min
  `cargo install`. 0.23.0 is the version CI already ran green. The gate preflight now requires that
  exact version, and the local install was upgraded from 0.22.2.
- DBAMON static snapshots taken from now on record cargo-audit 0.23.0.

Baseline before WO-03 (run 45776215740, b0d16e5): install step 153 s, full gate 180 s, release build
76 s, about 8 min in total.

## Verification on GitHub (2026-11-26)

| Run | Kind | Installs | Fast gate | Full gate | Release build | Total |
|---|---|---|---|---|---|---|
| 45776215740 | baseline, no cache | 153 s + 21 s | 22 s | 180 s | 76 s | ~472 s |
| 45791481739 | cold (fills cache) | 15 s | 28 s | 240 s | 82 s | 391 s |
| 45795757662 | warm, manual | 28 s | 15 s | 229 s | (stopped at secret scan) | 281 s |
| 45796746341 | warm, manual, after the exemption | 35 s | 14 s | 234 s | 52 s | 347 s |
| 45796732883 | warm, push to main | — | — | — | — | 326 s |

- The warm run restored the 270 MB cache with a full key match, so installs took 28 s (target
  < 30 s). The prebuilt `cargo-audit` took 0 s on both runs.
- The full gate barely moved: tests, not compilation, dominate it. That remaining time is WO-05.
- The manual run failed `Committed secrets`. A manual run scans the whole history, while a push
  scans only its commits. Two historical false positives: a fake key in the redaction unit test
  (a66bda9, `tools/reviewer/src/precheck.rs:157`) and the prose "strategic/execution role split"
  (4edf2b5, `docs/oap-codeos-integration.md:50`). `.gitleaksignore` exempts exactly these two
  findings; a local full-history gitleaks scan then reports no leaks.

Status: accepted. Warm installs are under 30 s with a full cache hit, and a warm main run takes
about 5.5 min against about 8 min before. Both warm runs of the final commit are green.
