---
component_question: How should observable behavior be derived from Intent into a Behavioral Contract?
out_of_scope: Intent ownership, event-schema design, internal architecture, implementation, and approval mechanics.
---

# Stage 2: Behavioral Contract Derivation

## Purpose

Derive independently testable observable behavior from the current Intent draft without specifying
internal implementation.

## Inputs / Prerequisites

Read the current Intent from its canonical location, the approved Solution Charter, and
`.codeos/toolkit/dba/05-guidance/templates/contract.md`; the template owns the Contract shape. Keep
both artifacts in `DRAFT` and revise Intent if Contract work exposes a product gap.

## Task

- Cover every Intent outcome with enough Given/When/Then scenarios to define the required happy,
  failure, and boundary behavior. Do not manufacture failure counts or scenario types that do not
  apply.
- Make every Then clause observable from allowed outputs, state, or the Contract's declared
  observation artifact. Keep event and module names out of scenarios unless they are themselves
  approved user-visible behavior.
- Derive invariants and minimal falsifying fixtures for plausible wrong implementations. Add more
  than one fixture only when distinct plausible mistakes require it.
- Define preconditions, postconditions, runtime artifacts, and the exhaustive set of governed
  behavioral failures. A failure may be observed through an approved event or another Contract
  signal; event authorization belongs to Stage 3.
- Declare `observation_mode` in Runtime Context:
  - `events` when governed internal events provide runtime evidence;
  - `external-observation` only when outcomes are proven through a named external observation
    artifact and no governed internal events are required.
- Record a quality requirement here only when it applies specifically to this feature and states
  something observable or verifiable. A cross-cutting obligation belongs to the Charter's System
  Constraints; route it there instead of restating it.
- Give every quality requirement a verification method. Use automated tests where the requirement is
  testable; otherwise name measurement, analysis, inspection, review, or operational evidence. A
  requirement with no method is not admissible.
- State workload, operating context, and rationale for every threshold. A deliberately new target
  needs no prior evidence; an unexplained number is not a requirement.
- Keep the requirement separate from its architectural consequence. State what must be true, not the
  mechanism that would achieve it.
- Record how this feature demonstrates conformance with any Charter System Constraint its
  implementation can affect. Do not copy the constraint text.
- Identify exact cross-module signals only when behavior depends on them.
- Apply the vocabulary pattern only when Stage 1 identified a vocabulary dependency; keep
  resolution strategy out of the Contract.

If an unresolved Intent decision prevents a sound Contract, present the partial draft with an
actionable unknown and return to Stage 1.

## Applicable Checks

Confirm that every Intent outcome has observable coverage, every governed failure is defined,
invariants have genuine falsifiers, observation mode is explicit, every quality requirement has a
verification method and every threshold its context and rationale, and no internal mechanism or
unapproved event requirement appears.

## Output / Next Action

Create `.codeos/01-specification/contracts/<feature-id>_contract.md` as `DRAFT`, present it, list
unresolved Intent questions, and hand the current Intent and Contract to `03-event-schema.md`.
