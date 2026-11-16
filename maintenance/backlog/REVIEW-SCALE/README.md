# REVIEW-SCALE — Deterministic Review Accounting

Planning context only. Git remains authoritative. Supersedes REVIEW-BUDGET.

## Problem

JamFree Stage 8 packets are 380–420 KB against a 450 KB budget raised from the 50 KB default.
Measured on F-0002 (380 KB): production evidence 122 KB, of which 109 KB is one route-coordinate
array (5,012 points); whole test-definition files 139 KB and whole-suite test evidence 45 KB, both
project-wide; governed documents 53 KB. Production evidence is already per feature. The packet
manifest records path, mode, bytes and sha per input, but not why an input was included, so growth
is found by hand. Wall time is noisy (the same packet took 154 s and 85 s); bytes are the
deterministic measure.

Principle: measure exactly, remove one demonstrated form of useless review content, restore a fixed
regression guard, and let `review-status` show the next real bottleneck.

| WO | Class | Title | Status |
|---|---|---|---|
| [WO-01](WO-01-review-accounting.md) | NORMAL | Deterministic review accounting | Accepted 2026-11-16 |
| [WO-02](WO-02-route-geometry-projection.md) | PROTECTED | Route-geometry projection | DBA-33 issued 2026-11-16; JamFree adoption and release next |
| [WO-03](WO-03-budget-guard.md) | NORMAL | Budget as a fixed guard | After WO-02 |

This table is the only place WO status is kept.

## Deferred

Chosen later only from what `review-status` demonstrates:

- **Claim-to-test file selection.** The project-wide test material (about 185 KB) grows with every
  feature. Slicing test files is not safe (Rust inline tests `use super::*`); the lever would be
  choosing files through a claim-to-test map (PROTECTED).
- **Delta reassessment.** Only if `review-status` shows reassessments are a material share. A
  BLOCKED review may stop at its first blocker, so its packet is not reusable coverage.
- **Parallel reviews, reasoning-effort tuning, Playwright run reuse, model-driven retrieval.**
  Throughput or verification concerns, not review scoping.
