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

Known unresolved evidence gaps follow the existing refinement route before final acceptance. Do
not create a separate eligibility status or treat mechanical packet readiness as a judgment that
the evidence proves the approved behavior.

Under the selected Workflow Governance policy, final acceptance is also gated on the Feature
Development checkpoints F5–F8 being mechanically verified as present and current —
`codeos-workflow status --workflow feature --subject <feature-id>` reports each applicable F5, F5d,
F5g, F6, F7, and F8 as PASS. This confirms the required evidence exists and is bound to the current
Specification Package and implementation; final acceptance remains the human decision below, with
its criteria unchanged.

## Task

- In `events` mode, verify schema conformance, governed event sequence, correlation chains, and
  deterministic payload content. A chain may contain one event when that event validly represents
  the complete governed outcome.
- Compare repeat runs using governed sequence and deterministic payload semantics. Ignore generated
  IDs, timestamps, and other nondeterministic envelope fields unless the Contract governs them.
- In `external-observation` mode, rerun the declared verification and compare the governed
  observable outcomes; do not require an event log or event replay.
- Replay a quality requirement or System Constraint only where replay is its declared verification
  method. Otherwise carry forward the evidence its method produced, or report it as an outstanding
  gap.
- Report nondeterminism, missing fixtures, and environment limitations. Runtime evidence never
  amends approved requirements.

## Applicable Checks

Use the selected review policy at this final decision boundary. Build the final inline Review
Package from the current template and available evidence. The reviewer examines whether acceptance
evidence observes the boundary named by each claim, whether governed performance evidence exercises
the required behavior, and whether reconciliation stays within its cited observations.

## Output / Next Action

Present the verification result, remaining gaps, and advisory review or waiver, then state:

`AWAITING FINAL HUMAN ACCEPTANCE`

The review package and verification result remain inline; this workflow creates no separate durable
report. The human accepts the feature or requests targeted refinement through `09-refine.md`. On
acceptance, record it as the Acceptance decision receipt: `codeos-workflow decide --workflow
feature --subject <feature-id> --checkpoint acceptance --result accepted`. The receipt records that
the human's acceptance decision occurred and binds it to the Specification Package, the reviewer
record, and the reconciliation and final-validation receipts; it is not an `ACCEPTED` status and
carries no product authority.

Acceptance ends the delivery cycle and opens the post-acceptance path. From that point, a fact
arising from real system use is an Operational Observation: classify it at session start and return
it to the earliest governed authority whose truth must change. Evidence produced before acceptance
remains development evidence and is not an Operational Observation.
