# WO-04 — Real-Project Replay and Value Decision

NORMAL · after WO-03

## Goal

Keep Platform Assurance only if it improves real project diagnosis.

## Work

- Replay JamFree and EvidenceAtlas with the new JSON and terminal view.
- Compare each result with current status and manual evidence inspection.
- Verify that JamFree does not imply persisted-domain assurance merely because PostgreSQL runs.
- Verify that EvidenceAtlas distinguishes database proof from backend event emission.

## Acceptance criteria

1. Every result matches its authoritative evidence and Feature Impact Accounting scope.
2. The view exposes a material distinction or missing proof from one command.
3. Stale Development Evidence is not described as a live outage.
4. If criterion 2 fails, remove the projection and close PLAT without WO-05 or WO-06.

## Result (2026-10-20)

Accepted — retain Platform Assurance.

JamFree reports backend `PASS`, persistence `NOT_APPLICABLE`, and GUI plus integrated product
`STALE`. That distinction is hidden by its project-level Verification `PASS`; the stale records
identify changed execution or charter bindings, not outages.

EvidenceAtlas reports all areas `UNOBSERVED`: 17 features lack complete Feature Impact Accounting,
while EA-0018 lacks exact current Candidate Bundle identities. This correctly refuses to infer
database, backend, or GUI assurance from broad project status. The replay exposed actionable proof
gaps without adding state or monitoring infrastructure.
