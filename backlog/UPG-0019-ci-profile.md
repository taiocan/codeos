---
feature_id: UPG-0019
slug: ci-profile
title: CI Integration Profile
status: PROPOSED
priority: P3
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: ci-profile — CI Integration Profile

**Priority**: P3
**Status**: PROPOSED
**Type**: toolkit-upgrade
**Related**: workflow-profiles, stack-drift-detector

## Problem

Local evidence may not match CI.

## Upgrade

Define how Codeos evidence maps to CI checks.

## Scope

CI integration (optional).

## Proposed artifact(s)

Proposed CI checks:

```text
behavioral tests
replay tests
schema conformance tests
lint/typecheck
no unapproved event names
no raw runtime log leakage
stack manifest reconciliation if dependency/config changed
```

## Design notes

Maps existing Codeos evidence types (behavioral tests, replay, schema conformance) onto CI
gates so local and CI evidence converge.

## Value

Medium-high if you use CI heavily.

## Risk

CI maintenance overhead.

## Guardrail

Start with minimal checks.

## DBA-philosophy note

No rule touched. CI enforces existing evidence types (schema conformance, no unapproved
events) — reinforces, rather than alters, the behavioral chain.

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
