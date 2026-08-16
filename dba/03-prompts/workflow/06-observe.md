---
component_question: How should representative runtime execution produce trustworthy evidence?
out_of_scope: Changing code or specifications, reconciliation judgments, replay verification, and final acceptance.
---

# Stage 6: Runtime Evidence

## Purpose

Execute representative scenarios when permitted and preserve truthful evidence of what occurred.

Everything this stage produces is development evidence: it proves the candidate implementation
before acceptance. A fact arising from real system use after acceptance is an Operational
Observation and does not belong to this stage.

## Inputs / Prerequisites

Load the approved Contract and, when applicable, Event Schema types through the Downstream Project
Layout Contract. Use them to determine observation mode, controlled actions, and any minimum
environment.

## Task

- Run representative happy and failure scenarios only when the environment and authorization allow.
- In `events` mode, collect the bounded, sanitized event evidence produced by the implementation;
  check event authorization, required fields, and correlation chains.
- In `external-observation` mode, collect the Contract's declared observation artifact and check the
  observable outcomes it is meant to prove.
- Record any unobserved path and the concrete reason. Do not fabricate evidence or turn absence into
  a pass.
- Never edit implementation, schemas, fixtures, or prior runtime evidence in this stage. If required
  instrumentation is absent, route the gap back to Stage 4 or the Specification Package as
  appropriate.

Actions designated as controlled require explicit human authorization. Preserve existing evidence
according to its owning artifact's rules; use isolated new evidence when a clean run is required.

## Output / Next Action

Report scenarios executed, evidence collected, failures, skipped/blocked paths, and environment
limits inline. Runtime evidence remains at its Contract-owned location, with
`events/runtime_events.jsonl` as the shared default; this workflow creates no separate durable
report. Hand available evidence and explicit gaps to `07-reconcile.md`.
