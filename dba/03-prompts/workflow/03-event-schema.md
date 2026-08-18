---
component_question: How should Contract outcomes be represented as governed events and completed as a Specification Package?
out_of_scope: Feature implementation, runtime evidence, reviewer mechanics, and project-level architecture decisions.
---

# Stage 3: Event Schema and Specification Approval

<!-- DOCTRINE ADAPTER: specification-approval -->

## Purpose

Complete the Event Schema and present Intent, Contract, and Event Schema as one mutually consistent
Specification Package for explicit human approval.

## Inputs / Prerequisites

Read the current Intent and Contract from the canonical locations defined by the Downstream Project
Layout Contract, the selected doctrine and review policy, and
`.codeos/toolkit/dba/05-guidance/templates/event-schema.md`. All three specification artifacts
remain `DRAFT` until one package approval.

## Task

Apply the Contract's observation mode:

- `events`: define only governed events traceable to Contract outcomes or failures, with required
  fields, payload semantics, flow, and coverage.
- `external-observation`: retain the Event Schema as the third package artifact, state that the
  feature defines no governed internal events, name the Contract's observation artifact, and map
  Contract outcomes to that evidence. Do not invent placeholder events.

In event mode, avoid uncontracted ordering, implementation lifecycle language, and processing steps
in event flow. Separate events emitted by this feature from exact cross-module events it relies on.
Any new event, payload meaning, or ordering guarantee must trace to the Contract; otherwise revise
the Contract or remove it.

## Applicable Checks

Verify mutual consistency across all three artifacts: Intent owns meaning, Contract owns observable
behavior and observation mode, and Event Schema authorizes only the governed events required by the
Contract. Apply the selected review policy at this decision boundary.

## Output / Next Action

Create `.codeos/01-specification/event-schemas/<feature-id>_schema.md`, then present the complete
Specification Package and a concise review package using the current template. Run or waive
advisory review as the selected review policy permits, then state:

`AWAITING HUMAN APPROVAL OF THE SPECIFICATION PACKAGE`

Do not implement before explicit approval. Record the one approval consistently on all three
artifacts. Before Stage 4, apply the selected architecture policy to determine whether an unresolved
project-level decision requires architecture synthesis.
