# STAB — CodeOS Delivery Stabilization Charter

- **Status:** APPROVED — Primoz Gorjup, 2026-10-10
- **Scope:** CodeOS-wide; pilot project JamFree
- **Baseline:** ECP-2 to ECP-5 complete; STAGE8-EVIDENCE implemented; JamFree F-0001 candidate
  `03818120` awaiting its Stage 8 assessment

Planning context only. Git and the governing policies stay authoritative.

## Problem

CodeOS keeps changing while it delivers. JamFree has 4 accepted features, and none is currently
valid. The four features generated 76 additional decision receipts (25 of them F-0001
`early_preview`) and 32 assessments that did not move work forward. There have been 25 DBA
configurations since 2026-08-10, and gate conformance defects were still being found on 2026-10-10.
An internal one-field change to `provider.rs` made a human preview decision stale. Delivery also
stalls on one quota-limited reviewer.

## Success criterion

**CodeOS is stable when normal features can move from specification to acceptance without CodeOS
changes during delivery.** Productivity target: reduce unnecessary invalidation and repeated human
decisions before automating execution.

A **qualifying normal feature** is an ordinary product feature or change that completes F1→F9 with
no CodeOS capability work and no recovery mechanism. Product fixes are allowed. A CodeOS
conformance repair made during its delivery disqualifies it.

**Target:** three qualifying features in total, at least two of them after P2. The historical 1/3
is JamFree F-0003.

## Phases

1. **P1 — Finish F-0001 with the system as it is.** Run the single Stage 8 assessment. Classify a
   BLOCKED finding before changing anything:
   - a JamFree defect: fix JamFree;
   - evidence that current policy already requires: CodeOS conformance repair;
   - a new reviewer expectation: never silently made policy.

   Record which human decisions went stale and why.
2. **P2 — Scope-aware human-decision invalidation.** A receipt goes stale only when the surface it
   judged changes: invalidation follows dependency, not broad implementation identity. This is
   PROTECTED. Its work order states the semantics, invariants, and verification, and gets human
   approval before implementation.
3. **P3 — Freeze and prove.** No CodeOS capability work. Only defects that block correct
   progression or violate existing authority are repaired. A usability inconvenience, an extra
   manual step, or a desirable optimization is not a defect. Record it for P4.
4. **P4 — Decide.** Review the ECP history, then choose the next structural improvement.

## Measures per feature

| Measure | Role |
|---|---|
| CodeOS capability or conformance changes during delivery | Required: 0 |
| Feature completes F1→F9 | Required |
| Candidate Bundles | Diagnostic target: 1–2 |
| Valid Stage 8 assessments | Diagnostic |
| Repeated human receipts | Diagnostic |
| `NOT_READY` / execution-invalid | Diagnostic |

The sources are Git (CodeOS commits between the feature's first F1 receipt and its F9 receipt) and
the ECP history. No new store is created.

## Deferred, with entry conditions

- **Batched human attention:** only after P2.
- **Reviewer throughput:** route a candidate to the reviewer only when its packet is mechanically
  proven complete. Never weaken reviewer independence.
- **WO-07 automatic execution:** only if P3 shows repeated AUTO→AUTO cycles.

## Constraints

- No new DBA configuration solely to carry a conformance or stabilization repair. Prospective
  semantics are versioned through the owning policy or contract.
- No recovery mechanisms.
- The reviewer stays advisory and independent.
- No project-specific rules in CodeOS.

Work orders are added per phase.
