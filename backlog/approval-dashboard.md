# Upgrade: approval-dashboard — Human Approval Dashboard

**Priority**: P3
**Status**: BACKLOG
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
