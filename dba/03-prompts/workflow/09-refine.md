---
component_question: How should observed behavioral problems be corrected with the smallest effective refinement?
out_of_scope: Unobserved improvements, broad redesign, unrelated maintenance, and approval decisions.
---

# Stage 9: Targeted Refinement

## Purpose

Apply the smallest justified response to an observed behavioral problem or explicitly approved
evolution request.

## Inputs / Prerequisites

Read the relevant runtime evidence, reconciliation row, replay result, or explicit human evolution
decision plus the current approved artifacts.

## Task

1. Name the trigger and evidence.
2. Identify the supported root cause and affected governed boundary.
3. Propose the smallest effective change; do not prefer observability, behavior, reliability,
   performance, or structure by a fixed cost taxonomy when the actual minimal fix differs.
4. Route the change:
   - approved behavior or event meaning changes → revise and reapprove the Specification Package;
   - project-level architecture changes → revise the applicable architecture scope;
   - conformance-only repair → implement under existing authority with proportional verification;
   - behavior-neutral, architecture-insignificant maintenance → normal engineering outside DBA.
5. Re-run only affected delivery stages, always returning to reconciliation and final verification
   before acceptance.

A single safety, authorization, or integrity failure is sufficient evidence; recurrence is not
required. Do not improve unrelated code or disguise redesign as refinement.

## Output / Next Action

Fill the current refinement template only when a durable refinement record adds value; otherwise
use the governing artifact changes and Git as the record. Present trigger, evidence, root cause,
minimal change, affected authority, and verification route. Return the verified result to the
final-acceptance adapter in `08-replay.md`.
