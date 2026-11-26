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
