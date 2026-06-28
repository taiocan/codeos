---
feature_id: UPG-0021
slug: stage-report-generator
title: Stage Report Generator
status: PROPOSED
priority: P3
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: stage-report-generator — Stage Report Generator

**Priority**: P3
**Status**: PROPOSED
**Type**: toolkit-upgrade
**Related**: stage-4-6-reports

## Problem

Stage reports can become manual burden.

## Upgrade

Generate reports from git diff, test output, and runtime files where possible.

## Scope

Stage 4–6 report generation.

## Proposed artifact(s)

Generated Stage 4–6 report skeleton, filled automatically where possible.

## Design notes

Inputs:

```text
git diff
git status
test output
runtime_events.jsonl
approved artifact paths
```

Output: Stage 4–6 report skeleton, filled automatically where possible.

## Value

Medium. Useful after report templates prove valuable.

## Risk

Automation hides errors.

## Guardrail

Generated report must say what was inferred vs human/model-written.

## DBA-philosophy note

No rule touched, but automation must **label inferred vs authored** content so generated
evidence is never mistaken for verified human/model judgment.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
