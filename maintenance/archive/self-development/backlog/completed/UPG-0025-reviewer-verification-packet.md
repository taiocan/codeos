---
feature_id: UPG-0025
slug: reviewer-verification-packet
title: Verification Packet for Reviewer Agent
status: COMPLETE
priority: P3
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: reviewer-verification-packet — Verification Packet for Reviewer Agent

**Priority**: P3
**Status**: COMPLETE
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
| CHG-20260706-003 | `changes/UPG-0025__CHG-20260706-003__reviewer-verification-packet.md` | Document the verification round-trip connecting the reviewer's uncertainty line to `verify-only.md` | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| REV__UPG-0025__CHG-20260706-003__S1__R1 | CHG-20260706-003 | 1-Intent | R1 | NO OBJECTION |
| REV__UPG-0025__CHG-20260706-003__S2__R1 | CHG-20260706-003 | 2-Acceptance | R1 | NO OBJECTION |
| REV__UPG-0025__CHG-20260706-003__S3__R1 | CHG-20260706-003 | 3-Implement | R1 | NO OBJECTION |
| REV__UPG-0025__CHG-20260706-003__S4__R1 | CHG-20260706-003 | 4-Reconcile | R1 | NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
