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
