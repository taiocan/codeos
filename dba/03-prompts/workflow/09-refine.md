---
component_question: How should an implementation that fails to satisfy already-approved behavior be corrected with the smallest effective repair?
out_of_scope: Redefining intended behavior, requirements, or architecture; unobserved improvements; broad redesign; unrelated maintenance; and approval decisions.
---

# Stage 9: Targeted Refinement

## Purpose

Correct an implementation that does not satisfy already-approved behavior, using the smallest
effective repair. This stage never redefines intended behavior.

## Inputs / Prerequisites

Read the relevant development evidence, reconciliation row, replay result, or classified Operational
Observation plus the current approved artifacts.

Confirm first that the trigger is a conformance defect. If the approved requirement, architecture, or
Charter is itself what must change, this stage does not apply: the doctrine's re-entry rule returns
the work to the earliest governed authority whose truth must change. Stage 9 is not the router and
never carries a requirement or architecture change under its own authority.

## Task

1. Name the trigger and evidence.
2. Identify the supported root cause and confirm the approved artifacts remain correct.
3. Propose the smallest effective repair; do not prefer observability, behavior, reliability,
   performance, or structure by a fixed cost taxonomy when the actual minimal fix differs.
4. Implement under existing authority with verification proportional to the risk. Behavior-neutral,
   architecture-insignificant maintenance remains normal engineering outside DBA.
5. Re-run only affected delivery stages, always returning to reconciliation and final verification
   before acceptance.

A single safety, authorization, or integrity failure is sufficient evidence; recurrence is not
required. Do not improve unrelated code or disguise redesign as refinement.

## Output / Next Action

Create `.codeos/04-refinement/<feature-id>-<slug>.md` from the optional refinement template only
when a durable refinement record adds value; otherwise use the governing artifact changes and Git
as the record and keep this result inline. Present trigger, evidence, root cause, minimal repair,
and verification route. Update an existing Module Design Note when the repair materially changes
what it documents, under the doctrine's update trigger. Return the verified result to the
final-acceptance adapter in `08-replay.md`.
