# Upgrade: reviewer-verification-packet — Verification Packet for Reviewer Agent

**Priority**: P3
**Status**: BACKLOG
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
