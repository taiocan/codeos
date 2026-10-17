# DQA — Delivery Qualification and Assurance Charter

- **Status:** PROPOSED
- **Evidence:** DBA-24–28 and JamFree delivery, 2026-10-09 through 2026-10-17
- **Scope:** CodeOS release qualification, shared lifecycle mechanics, safe verification, and the
  existing STAB proof

Planning context only. This Charter grants no implementation or semantic approval.

## Problem

CodeOS now detects defects well, but normal JamFree delivery still exposes CodeOS defects. F-0005
and F-0006 each required a conformance repair, DBA-27 needed DBA-28 after its first real replay,
and one production-mode check started browser verification against the live provider. STAB remains
at 1 of 3 qualifying features.

## Goal

Make a supported DBA configuration safe to adopt and keep CodeOS unchanged while ordinary features
move from F1 to F9.

## Invariants

- Supported configurations and issued policies remain immutable.
- JamFree supplies replay evidence but never defines universal policy.
- Shared lifecycle semantics have one executable authority.
- Production verification cannot invoke fixture-only or browser work.
- No gate, approval, review round, or recovery path is added without demonstrated need.

## Success criteria

1. A configuration cannot be issued before generic qualification and accepted downstream replay.
2. Workflow and reviewer consumers cannot disagree on shared lifecycle derivations.
3. Production verification has isolated execution and reconciled provider-call accounting.
4. STAB reaches 3 qualifying features, including 2 after the DBA-28 baseline.

Work orders: [README](README.md).
