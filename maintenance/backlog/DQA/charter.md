# DQA — Delivery Qualification and Assurance Charter

- **Status:** PROPOSED
- **Evidence:** DBA-24–28 and JamFree delivery, 2026-10-09 through 2026-10-18
- **Scope:** CodeOS release qualification, shared lifecycle mechanics, safe verification,
  system-scoped validation, and the existing STAB proof

Planning context only. This Charter grants no implementation or semantic approval.

## Problem

CodeOS now detects defects well, but normal JamFree delivery still exposes CodeOS defects and
repeats shared product validation at feature boundaries. F-0005 and F-0006 each required a
conformance repair, DBA-27 needed DBA-28 after its first real replay, and one production-mode check
started browser verification against the live provider. At one JamFree product state, six features
recorded identical behavior, repeatability, and Playwright evidence; F-0001 and F-0006 repeatedly
required separate F4 actions against an identical Observed Surface. STAB remains at 1 of 3
qualifying features.

## Goal

Make a supported DBA configuration safe to adopt, validate one immutable product state once where
the evidence directly supports several governed claims, preserve feature-level traceability and
human control, and keep CodeOS unchanged while ordinary features move from F1 to F9.

## Invariants

- Supported configurations and issued policies remain immutable.
- JamFree supplies replay evidence but never defines universal policy.
- Shared lifecycle semantics have one executable authority.
- Production verification cannot invoke fixture-only or browser work.
- Historical acceptance defines journey topology; Current Validity determines whether a claim may
  be consumed. Staleness does not remove an accepted journey step.
- Shared execution or observation never merges requirement ownership. Distinct feature claims may
  consume the same evidence.
- Claim meaning remains Contract authority. Validation code may declare coverage, never redefine
  the claim.
- Human observation may be shared; each feature, checkpoint, and claim outcome remains an explicit
  human judgment. No matching surface or agent action propagates a verdict.
- Candidate assessment and acceptance remain feature-atomic.
- Unknown claim coverage, scope, revision, environment, or attribution fails closed.
- No gate, approval, review round, or recovery path is added without demonstrated need.

## Success criteria

1. A configuration cannot be issued before generic qualification and accepted downstream replay.
2. Workflow and reviewer consumers cannot disagree on shared lifecycle derivations.
3. Production verification has isolated execution and reconciled provider-call accounting.
4. One compatible validation action executes at most once per bound implementation/product state
   and environment; every consuming feature identifies the exact claims and evidence it relies on.
5. Journey topology stays stable while stale or inherited claims remain visibly blocked.
6. One human observation session may record mixed, explicit outcomes for several features without
   implicit propagation or retroactive decisions.
7. JamFree replay removes duplicate execution and viewing without producing false currency.
8. STAB reaches 3 qualifying features, including 2 after the accepted shared-validation baseline.

Work orders: [README](README.md).
