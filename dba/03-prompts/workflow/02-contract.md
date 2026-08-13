---
component_question: How should observable behavior be derived from Intent into a Behavioral Contract?
out_of_scope: Intent ownership, event-schema design, internal architecture, implementation, and approval mechanics.
---

# Stage 2: Behavioral Contract Derivation

## Purpose

Derive independently testable observable behavior from the current Intent draft without specifying
internal implementation.

## Inputs / Prerequisites

Read the current Intent and `.codeos/dba/05-guidance/templates/contract.md`; the template owns the
Contract shape. Keep both artifacts in `DRAFT` and revise Intent if Contract work exposes a product
gap.

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
- Identify exact cross-module signals only when behavior depends on them.
- Apply the vocabulary pattern only when Stage 1 identified a vocabulary dependency; keep
  resolution strategy out of the Contract.

If an unresolved Intent decision prevents a sound Contract, present the partial draft with an
actionable unknown and return to Stage 1.

## Applicable Checks

Confirm that every Intent outcome has observable coverage, every governed failure is defined,
invariants have genuine falsifiers, observation mode is explicit, and no internal mechanism or
unapproved event requirement appears.

## Output / Next Action

Present `contracts/[feature_id]_contract.md` as `DRAFT`, list unresolved Intent questions, and hand
the current Intent and Contract to `03-event-schema.md`.
