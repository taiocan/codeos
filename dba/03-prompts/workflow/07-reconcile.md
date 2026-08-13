---
component_question: How should approved artifacts, implementation, tests, and runtime evidence be reconciled?
out_of_scope: General code review, redesign, implementing repairs, changing requirements, and approval decisions.
---

# Stage 7: Reconciliation

## Purpose

Compare the approved Specification Package with implementation, tests, and available runtime
evidence. Report disagreement without rewriting requirements from observed behavior.

## Inputs / Prerequisites

Read the approved Intent, Contract, and Event Schema; implementation; executed test results; Stage 6
evidence; and applicable architecture. Use the Contract's observation mode to select event or
external evidence.

## Task

Create one row for every Contract requirement and every governed event when event mode applies.
Trace each row across the layers that apply and use only:

- Status: `ALIGNED`, `GAP`, `MISMATCH`, or `MISSING`
- Evidence: `runtime`, `test`, `static`, or `none`
- Note: the concrete agreement, gap, disagreement, or missing item

Meanings:

- `ALIGNED`: applicable layers agree.
- `GAP`: a required layer or strength of evidence is incomplete, including absent runtime
  observation.
- `MISMATCH`: two applicable layers disagree.
- `MISSING`: a required artifact, Contract item, or Event-Schema item is absent.

Do not encode gap causes as additional statuses. State implementation, test, runtime,
observability, or environment-quality concerns in the Note. When the Contract declares a minimum
environment, record the observed environment plainly and use `GAP` if it is insufficient; do not
create a general evidence score.

In event mode, compare observed event types and payload shapes with the approved Event Schema. In
external-observation mode, compare the declared observation artifact with the Contract. Absence of
runtime evidence is a `GAP` with `evidence: none`, not `MISSING`, when all required artifacts exist.

## Applicable Checks

Confirm Intent-to-Contract coverage, Contract-to-Event-Schema consistency where applicable,
implementation conformance, test coverage, and runtime evidence. Structural observations may be
noted separately when they affect future change cost, but they do not alter behavioral statuses.

Use this exact table shape:

| Item | Intent | Contract | Event Schema | Implementation | Tests | Runtime / Observation | Status | Evidence | Note |
|---|---|---|---|---|---|---|---|---|---|

## Output / Next Action

Present the table and summarize non-`ALIGNED` rows with the minimum route to resolve each. If a fix
requires changed approved meaning or project-level architecture, stop for human governance.
Otherwise hand the evidence to `08-replay.md`.
