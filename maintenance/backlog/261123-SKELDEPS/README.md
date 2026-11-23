# SKELDEPS — Vulnerable Dependencies in the Initializer Skeleton

Planning context only. Git remains authoritative. Approved by the human 2026-11-23.

The DBA Monitor static snapshot `20261122T222139Z-363d5838` (DBAMON WO-07) reported 8 cargo-audit
vulnerabilities and 11 warnings. The approved 2026-11 monthly assessment names dependency hygiene,
starting with the initializer skeleton, as the clearest improvement opportunity: every new CodeOS
project inherits the skeleton's lockfile.

## Findings after audit

| Advisory | Skeleton | JamFree backend | AII |
|---|---|---|---|
| url 2.5.8, RUSTSEC-2026-0592 (DoS in `Url::origin()` on nested `blob:` URLs; fixed in 2.5.9) | compiled (sqlx-core) | compiled (sqlx-core, reqwest) | fixed in uncommitted work (2.5.9) |
| rsa 0.9.10, RUSTSEC-2023-0071 (Marvin; fixed in >= 0.10) | lockfile only | lockfile only | absent |
| libsqlite3-sys 0.30.1, RUSTSEC-2026-0459 (fixed in >= 0.39) | lockfile only | lockfile only | used (rusqlite bundled); fixed in uncommitted work (0.40) |
| tokio 1.53.1, unsound RUSTSEC-2026-0386 (fixed in >= 1.53.3) | compiled | already 1.54.1 | already 1.54.1 |
| chrono, unmaintained RUSTSEC-2026-0543 | enabled through the sqlx `chrono` feature but unused | used | used |

"Lockfile only" means `cargo tree -i` shows the crate is never built. It enters `Cargo.lock` only
through sqlx 0.8's optional MySQL and SQLite drivers, and cargo-audit reads the lockfile.

## Audit conclusions

1. **The headline overstates exposure.** 4 of the 8 vulnerabilities (rsa and libsqlite3-sys in the
   skeleton and in JamFree) are lockfile only. The skeleton's real exposure is url, with low
   practical risk because the skeleton never calls `Url::origin()`, plus the unsound tokio.
2. **No suppression.** `.cargo/audit.toml` ignores would propagate to every new project and would
   hide the advisories if a project later enabled the MySQL or SQLite drivers.
3. **sqlx 0.9 is deferred.** It is the only way to remove the lockfile-only entries, but it needs
   Rust 1.94 while the host has 1.93 (clippy, the monitor and development run on the host), and it
   is a major-version migration.
4. **No standalone JamFree bump.** `backend/Cargo.lock` is a dependency input to candidate bundles
   and to evidence scope (`workflow/engine/src/candidate.rs`, `workflow/engine/src/project.rs`), so
   changing it forces revalidation, with its cost and TomTom calls. JamFree's backend platform
   assurance and supply-chain health are already STALE, so the bump belongs in JamFree's next
   feature cycle, under JamFree's workflow.
5. **Remove chrono rather than replace it.** The skeleton source uses no time type. Dropping the
   sqlx `chrono` feature removes chrono from the lockfile and from the warnings; one README line
   steers new projects to a maintained alternative.
6. **Out of scope:**
   - the CodeOS engines' warnings (unsafe-libyaml, rustix, chrono), because any engine change needs
     a qualified release;
   - the skeleton's npm lockfile, which the monitor does not audit;
   - AII, whose findings its uncommitted work already fixes.
7. **No new guard.** The monthly static snapshot already audits the skeleton.

A trial in a scratch copy (url 2.5.9, tokio 1.54.1, no `chrono`) passed `cargo build`, `clippy
--all-targets -D warnings` and `cargo test` with `--locked`. Afterwards `cargo audit` reported only
the two lockfile-only advisories and 0 warnings.

**Class: NORMAL.** The skeleton is copied only at init and never overwrites an existing project.
It belongs to no release or configuration identity, and no binaries change.

| WO | Class | Title | Status |
|---|---|---|---|
| [WO-01](WO-01-skeleton-patch-upgrades.md) | NORMAL | Patch url and tokio; remove unused chrono | Accepted 2026-11-23 |
| [WO-02](WO-02-compiled-advisory-flag.md) | NORMAL (DBAMON monitor and report) | Mark whether an advisory's crate is compiled | Approved 2026-11-23 — not started |
| [WO-03](WO-03-jamfree-url.md) | Project-owned | JamFree url 2.5.9 in its next feature cycle | Deferred to JamFree's next feature |
| [WO-04](WO-04-sqlx-0-9.md) | Deferred | sqlx 0.9 in the skeleton | Deferred (trigger in the WO) |

This table is the only place WO status is kept.
