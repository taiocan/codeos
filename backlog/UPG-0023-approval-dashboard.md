---
feature_id: UPG-0023
slug: approval-dashboard
title: Human Approval Dashboard
status: COMPLETE
priority: P3
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: approval-dashboard — Human Approval Dashboard

**Priority**: P3
**Status**: COMPLETE
**Type**: toolkit-upgrade
**Related**: feature-registry, reviewer-decision-brief, release-evidence-package

## Problem

Multiple features and reviewer briefs can be hard to navigate.

## Upgrade

Create a simple generated dashboard.

## Scope

Cross-feature human-facing overview.

## Proposed artifact(s)

`reviews/approval-dashboard.md`

## Design notes

```markdown
Active features:
Current stage:
Reviewer recommendation:
Open blockers:
Next human decision:
Risk:
```

## Value

Medium later.

## Risk

Dashboard staleness.

## Guardrail

Generated from registry and reviewer briefs.

## DBA-philosophy note

No rule touched. **Generated** from the registry + reviewer briefs (not authored) — staleness
guard required; it is a navigation aid, not a decision record.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260704-001 | `changes/UPG-0023__CHG-20260704-001__approval-dashboard.md` | New Rust subcommand: `generate-approval-dashboard` | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| REV__UPG-0023__CHG-20260704-001__S1__R1 | CHG-20260704-001 | 1-Intent | R1 | NO OBJECTION |
| REV__UPG-0023__CHG-20260704-001__S2__R1 | CHG-20260704-001 | 2-Acceptance | R1 | NO OBJECTION |
| REV__UPG-0023__CHG-20260704-001__S3__R1 | CHG-20260704-001 | 3-Implement | R1 | NO OBJECTION (non-blocking: Cargo.lock not listed) |
| REV__UPG-0023__CHG-20260704-001__S3__R2 | CHG-20260704-001 | 3-Implement | R2 | NO OBJECTION |
| REV__UPG-0023__CHG-20260704-001__S4__R1 | CHG-20260704-001 | 4-Reconcile | R1 | NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
| UPG-0040 | `config::tests::toml_overrides_default` races on a global env var under parallel `cargo test`; passes deterministically single-threaded. `config.rs` untouched by this change — pre-existing, out of scope | Step 4 verification |
| UPG-0041 | `generate-approval-dashboard` works against the canonical `templates/feature-registry.yaml` schema (as scoped in Step 1), but fails to parse FundFlow's actual, drifted registry (missing `slug`, different `status` vocabulary, no `current_stage`/`blockers` fields). Not an in-scope defect — Step 1 never promised FundFlow-registry compatibility — but a real gap worth reconciling deliberately | Post-Step-4 demonstration against a real downstream registry |
