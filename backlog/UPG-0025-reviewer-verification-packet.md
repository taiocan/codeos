---
feature_id: UPG-0025
slug: reviewer-verification-packet
title: Verification Packet for Reviewer Agent
status: PROPOSED
priority: P3
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: reviewer-verification-packet — Verification Packet for Reviewer Agent

**Priority**: P3
**Status**: PROPOSED
**Type**: toolkit-upgrade
**Related**: reviewer-decision-brief, verify-only-mode

## Problem

Reviewer may need independent verification but should not edit files.

## Upgrade

Reviewer can request a verification-only packet.

## Scope

Reviewer agent + verification-only mode interaction.

## Proposed artifact(s)

Proposed flow:

```text
Reviewer identifies uncertainty.
Reviewer requests verification packet.
Claude runs read-only verification.
Verification result returns to reviewer.
Reviewer updates decision brief.
Human decides.
```

## Design notes

Builds on `verify-only-mode`: the verification is strictly read-only and its result becomes
additional evidence for the reviewer's brief.

## Value

Medium. Useful when reviewer confidence is low.

## Risk

Adds a round-trip; reviewer may over-request verification.

## Guardrail

Verification is read-only; reviewer never edits; human still decides.

## DBA-philosophy note

No rule touched. Keeps the reviewer **read-only** even when it needs fresh evidence — preserves
the artifact-production / artifact-assessment separation.

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
