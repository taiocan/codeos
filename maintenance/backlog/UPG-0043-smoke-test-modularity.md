---
feature_id: UPG-0043
slug: smoke-test-modularity
title: Split Monolithic Smoke Test File by Tool Area
status: PROPOSED
priority: P3
depends_on: []
related_features: [UPG-0042]
supersedes: []
superseded_by: []
---

# Upgrade: smoke-test-modularity — Split Monolithic Smoke Test File by Tool Area

**Priority**: P3
**Status**: PROPOSED
**Type**: script-tooling

## Problem

`dba/04-tools/reviewer/engine/tests/smoke.rs` is a single 121 KB file (2,789 lines, 119 tests) covering all reviewer tool functionality. As more tools and test coverage are added, this file will continue growing, causing:

- **Review inefficiency**: Every review packet includes the full 121 KB file, even when changes affect only one tool area (see UPG-0042)
- **Cognitive load**: 2,789 lines in one file makes it harder to find relevant tests
- **Merge conflicts**: Higher risk as the file grows and multiple changes touch it
- **Slow test discovery**: IDEs and grep must scan the entire file

Observed during UPG-0041: 9 new tests for registry v2 schema required including the full 121 KB smoke.rs in every review packet, even though only ~200 lines were relevant.

## Current Structure

Single `smoke.rs` organized by feature/tool:
```
smoke.rs (121 KB, 2,789 lines, 119 tests):
  - Common helpers (setup_temp_git_repo, binary(), run(), etc.)
  - UPG-0023: generate-approval-dashboard (15 tests)
  - UPG-0024: generate-release-evidence (15 tests)
  - UPG-0027: review command (20+ tests)
  - UPG-0032: check-drift (10 tests)
  - UPG-0034: read-only invariant (5 tests)
  - UPG-0035: sha-only mode (5 tests)
  - ... and more
```

All tests import from the same namespace, share helpers, but are conceptually independent by tool area.

## Upgrade

Not decided by this brief — questions for implementer to resolve:

### 1. Split Strategy

Should smoke tests be split:
- **By tool area**: `smoke_dashboard.rs`, `smoke_release_evidence.rs`, `smoke_review.rs`, etc.?
- **By feature**: `smoke_upg0023.rs`, `smoke_upg0024.rs`, etc.?
- **By test type**: `smoke_generators.rs`, `smoke_review_commands.rs`, `smoke_drift.rs`?
- **By fixture complexity**: `smoke_simple.rs`, `smoke_integration.rs`?

Which grouping makes the most sense for:
- Review packet efficiency (modified files only)
- Test discoverability (cargo test --test smoke_X)
- Conceptual cohesion

### 2. Helper Extraction

Should common helpers be:
- Extracted to `tests/common/mod.rs`? (standard Rust test pattern)
- Duplicated per-file? (no shared state)
- Left in a single `smoke_common.rs`?

Trade-off: shared helpers reduce duplication but create a dependency that might be included in every review packet.

### 3. Fixture Management

Should test fixtures (registry YAML, etc.) be:
- Inline in each test file (current pattern)?
- Extracted to `tests/fixtures/` directory?
- Generated programmatically from templates?

### 4. Coverage Preservation

How to ensure no test coverage is lost during the split:
- Run full test suite before and after, verify 119 → 119
- Check test names are preserved (may need renaming for uniqueness)
- Verify all UPG feature areas still have smoke coverage

### 5. Incremental vs. Big Bang

Should the split happen:
- All at once (one large refactor)?
- Incrementally (split one tool area at a time)?
- Only for new tests going forward (leave existing tests in place)?

Incremental risks having tests in two places; big bang is disruptive but cleaner.

## Scope

`dba/04-tools/reviewer/engine/tests/smoke.rs` split into multiple test files, with possible extraction of:
- Common test helpers
- Fixture data
- Test utilities

No change to test behavior, coverage, or passing rate (119 tests → 119 tests, all passing).

May also touch:
- CI configuration if test execution needs adjustment
- `Cargo.toml` if new test file organization requires changes
- Documentation referencing smoke tests

## Value

Medium. Benefits:
- **Review efficiency**: Packet includes only modified test file(s), not all 121 KB
- **Token savings**: 50-100k per review round when combined with UPG-0042
- **Discoverability**: `cargo test --test smoke_dashboard` targets one area
- **Maintainability**: Easier to find and update tests for a specific tool
- **Merge safety**: Parallel changes to different tools won't conflict

Trade-offs:
- Initial refactor cost (splitting, verifying coverage)
- Slightly more test boilerplate (multiple files, helper imports)
- More test files to track (but better organization)

## Risk

Deciding hastily risks:
- Test coverage loss during split (must verify 119 → 119)
- Breaking test dependencies on shared state
- Confusion about where new tests should go
- Over-fragmentation (too many tiny files)

Do not implement without:
1. Clear grouping strategy (by tool, feature, or type)
2. Verification that all 119 tests pass after split
3. Documentation on where new tests should be added
4. Review efficiency measurement (confirm packet size reduction)

## Guardrail

The split must:
- Preserve all 119 existing tests (verify by name and count)
- Maintain 100% pass rate
- Be reversible (git history preserves original structure)
- Not change test behavior or fixture data
- **Preserve test names where practical**, so review history and failure diagnostics remain traceable after the split

If coverage drops or tests break, revert and re-plan.

## Related

- **UPG-0042**: Reviewer packet efficiency (primary motivation)
- **UPG-0027**: Lean review runner (established test patterns)

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| (none yet) | — | — | PROPOSED |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
