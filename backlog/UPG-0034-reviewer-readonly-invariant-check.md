---
feature_id: UPG-0034
title: "Rust Reviewer: Read-Only Invariant Check (pre/post git status warning)"
priority: P3
status: PROPOSED
depends_on: UPG-0032
origin_change: CHG-20260702-001
---

# UPG-0034 — Rust Reviewer: Read-Only Invariant Check

## Problem

The Bash `codeos-review.sh` captured `git status --porcelain` immediately before and after
invoking Codex, then compared them. If the working tree changed during the review call,
it printed a warning on stderr: "WARNING: working tree changed during review — reviewer
should be read-only."

The Rust `codeos-reviewer` does not perform this check. The reviewer is supposed to be
read-only; a silent working-tree mutation during provider invocation would go unnoticed.

## What to add

After `prov.invoke()` returns in `cmd/review.rs`, capture `git status --porcelain` (with
the same exclusion pathspecs used elsewhere) and compare it to a snapshot taken before
invoke. If different, print a warning to stderr. No blocking — advisory only, same as Bash.

## Scope

Self-dev only. Touches `tools/reviewer/src/cmd/review.rs` only.

## Priority note

P3 — this is a warning, not enforcement. No review has ever triggered it in practice.
Useful as a safety net; low urgency.

## Feature Thread

| Change ID | State | Notes |
|---|---|---|
| — | PROPOSED | Origin: parity gap found during UPG-0032 / CHG-20260702-001 reconcile |
