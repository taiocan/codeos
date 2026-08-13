---
component_question: How should recorded evidence verify deterministic approved behavior before final acceptance?
out_of_scope: Defining new behavior, implementing fixes, changing event schemas, and making the acceptance decision.
---

# Stage 8: Replay Verification and Final Acceptance

<!-- DOCTRINE ADAPTER: final-acceptance -->

## Purpose

Verify repeatable conformance to the approved Specification Package and present the delivery result
for final human acceptance.

## Inputs / Prerequisites

Read the Stage 7 reconciliation and resolve or explicitly carry every non-`ALIGNED` item. Read the
Contract's observation mode and applicable replay or external-observation tests.

## Task

- In `events` mode, verify schema conformance, governed event sequence, correlation chains, and
  deterministic payload content. A chain may contain one event when that event validly represents
  the complete governed outcome.
- Compare repeat runs using governed sequence and deterministic payload semantics. Ignore generated
  IDs, timestamps, and other nondeterministic envelope fields unless the Contract governs them.
- In `external-observation` mode, rerun the declared verification and compare the governed
  observable outcomes; do not require an event log or event replay.
- Report nondeterminism, missing fixtures, and environment limitations. Runtime evidence never
  amends approved requirements.

## Applicable Checks

Use the selected review policy at this final decision boundary. Build the final inline Review
Package from the current template and available evidence.

## Output / Next Action

Present the verification result, remaining gaps, and advisory review or waiver, then state:

`AWAITING FINAL HUMAN ACCEPTANCE`

The human accepts the feature or requests targeted refinement through `09-refine.md`.
