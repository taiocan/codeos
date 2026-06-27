# Upgrade: release-evidence-package — Pre-Release Evidence Package

**Priority**: P3
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: readiness-checklist, reviewer-decision-brief, approval-dashboard

## Problem

Before release, artifacts may be scattered across stage files, reports, runtime logs, replay
results, and reviewer summaries.

## Upgrade

Create a release evidence package.

## Scope

Pre-release aggregation.

## Proposed artifact(s)

`reviews/release-evidence-[feature_id].md`

## Design notes

```markdown
Feature:
Branch/PR:
Approved artifacts:
Stage reports:
Reviewer briefs:
Reconciliation result:
Replay result:
Verification-only report:
Readiness checklist:
Known limitations:
Release decision:
```

## Value

Medium-high for serious releases.

## Risk

Duplicative with existing reviews.

## Guardrail

Generate from existing artifacts, do not manually rewrite.

## DBA-philosophy note

No rule touched. **Generated** from existing artifacts (not a new authored surface) — avoids
creating a second source of truth.
