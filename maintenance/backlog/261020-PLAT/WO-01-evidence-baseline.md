# WO-01 — Evidence and Value Baseline

NORMAL · before WO-02

## Goal

Prove which Platform Assurance claims existing evidence can support and whether the projection has
material diagnostic value.

## Work

- Map Persistence, Backend, GUI, and Integrated product to authoritative evidence.
- Replay the mapping against JamFree and EvidenceAtlas.
- Record unsupported claims and the current manual steps needed to answer each question.
- Test the adversarial cases: backend reachable while GUI fails; services reachable while database
  behavior is unproved; stale Development Evidence mistaken for a live outage.

## Acceptance criteria

1. Every proposed positive state has an exact evidence basis and feature scope.
2. Missing or stale evidence cannot become `PASS`.
3. The analysis identifies at least one material distinction hidden by current status.
4. If it only restates Verification status or needs a new evidence store, PLAT stops.

## Results (2026-10-20)

Baseline: CodeOS `64ff78d`; JamFree `973d0e1`; EvidenceAtlas `c5345bd` plus its generated-file
changes.

| Area | JamFree | EvidenceAtlas |
|---|---|---|
| Persistence | Not applicable: F-0001–F-0009 declare it unchanged | EA-0018 `data_integrity` is stale |
| Backend | F-0001 has exact Candidate Bundle evidence; F-0002 has no exact accepted record identities | EA-0018 smoke and behavior are stale |
| GUI | Seven features have exact Candidate Bundle evidence; F-0002/F-0003 have legacy identity gaps | EA-0018 Playwright is stale |
| Integrated product | Seven features bind smoke and product journey; F-0002/F-0003 have legacy gaps | EA-0018 evidence is stale; DBA-8 does not require product journey |

Current reconstruction requires project status, Feature Impact Accounting, targeted status, and
verification or Candidate Bundle inspection. Smoke proves backend and web reachability only at
execution time; it proves neither database behavior nor current availability. Persistence requires
`data_integrity`. Stale Development Evidence means revalidation, not a live outage.

**Decision: qualified GO to WO-02.** The projection exposes distinctions hidden by project-level
Verification `PASS` without a new evidence store. WO-02 must derive required mechanics from the
active workflow contract, use exact Candidate Bundle record identities where present, report
legacy unbound or absent evidence as `UNOBSERVED`, and reserve `BLOCKED` for a current failed result.
