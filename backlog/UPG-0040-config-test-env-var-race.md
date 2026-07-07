---
feature_id: UPG-0040
slug: config-test-env-var-race
title: Fix Flaky config::tests Race on CODEOS_REVIEWER_PROVIDER Env Var
status: COMPLETE
priority: P2
depends_on: []
related_features: [UPG-0023]
supersedes: []
superseded_by: []
---

# Upgrade: config-test-env-var-race — Fix Flaky config::tests Race on CODEOS_REVIEWER_PROVIDER Env Var

**Priority**: P2
**Status**: COMPLETE
**Type**: script-tooling

## Problem

`tools/reviewer/src/config.rs`'s unit tests (`toml_overrides_default` and likely others in the
same `#[cfg(test)] mod tests`) call `std::env::remove_var("CODEOS_REVIEWER_PROVIDER")` directly.
Rust's default test harness runs tests in parallel threads within the same process, and
environment variables are process-global, not thread-local — so any test that reads/removes
this env var races against other tests doing the same thing concurrently.

Discovered during UPG-0023's Step 4 verification (2026-07-06): `cargo test` (default,
parallel) intermittently failed `config::tests::toml_overrides_default`, while `cargo test --
--test-threads=1` passed all 120 tests (26 unit + 94 smoke) deterministically. `config.rs` is
untouched by UPG-0023 — this is a pre-existing flakiness bug, not a regression, filed as an
out-of-scope-for-UPG-0023 follow-up.

## Upgrade

Make `config.rs`'s env-var-dependent tests immune to parallel execution — e.g. a
process-wide mutex guarding env-var mutation in tests, or restructuring `resolve()`'s tests
to inject the env var value directly (as a parameter) rather than mutating the real process
environment.

## Scope

`tools/reviewer/src/config.rs`'s test module only. No production-code behavior change.

## Value

Medium. Flaky tests erode trust in `cargo test`'s green/red signal and can mask a real
regression appearing at the same time as an unrelated race.

## Risk

Low. Test-only change.

## Guardrail

No production code path changes — `config::resolve()`'s actual precedence logic (CLI flag >
env var > reviewer.toml > default) must remain unchanged.

## DBA-philosophy note

Not applicable — pure test-infrastructure fix, no doctrine change.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260707-004 | `changes/UPG-0040__CHG-20260707-004__config-test-env-var-race.md` | Serialize `config.rs`'s env-var-mutating tests with a test-only mutex | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| REV__UPG-0040__CHG-20260707-004__S1__R1 | CHG-20260707-004 | 1-Intent | R1 | REQUEST CHANGES (unsupported exclusivity claim) |
| REV__UPG-0040__CHG-20260707-004__S1__R2 | CHG-20260707-004 | 1-Intent | R2 | NO OBJECTION |
| REV__UPG-0040__CHG-20260707-004__S2__R1 | CHG-20260707-004 | 2-Acceptance | R1 | REQUEST CHANGES (unsupported test-count) |
| REV__UPG-0040__CHG-20260707-004__S2__R2 | CHG-20260707-004 | 2-Acceptance | R2 | REQUEST CHANGES (syntax + unsafe stash) |
| REV__UPG-0040__CHG-20260707-004__S2__R3 | CHG-20260707-004 | 2-Acceptance | R3 | NO OBJECTION |
| REV__UPG-0040__CHG-20260707-004__S3__R1 | CHG-20260707-004 | 3-Implement | R1 | NO OBJECTION |
| REV__UPG-0040__CHG-20260707-004__S4__R1 | CHG-20260707-004 | 4-Reconcile | R1 | REQUEST CHANGES (unscoped verification command) |
| REV__UPG-0040__CHG-20260707-004__S4__R2 | CHG-20260707-004 | 4-Reconcile | R2 | NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
