---
component_question: How should tests prove approved observable behavior and observation semantics?
out_of_scope: Feature implementation, runtime evidence collection, reconciliation, replay execution, and approval.
---

# Stage 5: Behavioral and Observation Tests

## Purpose

Test approved observable behavior without coupling tests to private implementation.

## Inputs / Prerequisites

Read the approved Contract and Event Schema plus the Stage 4 implementation. The Contract owns
observation mode, scenarios, failures, invariants, and any minimum environment requirement.

## Task

- Add behavioral tests covering every applicable Contract scenario, governed failure, and invariant
  falsifier. Test Contract notes only when they state observable requirements.
- Verify state changes and approved failure signals without asserting private methods, intermediate
  computations, or uncontracted ordering.
- Verify that internal technical errors do not masquerade as approved behavioral failures.
- In `events` mode, test authorized event types, required fields, relevant payload behavior,
  correlation integrity, and absence of unapproved governed events. Add replay coverage for governed
  sequence and deterministic payload content.
- In `external-observation` mode, test the Contract's declared observation artifact and do not create
  event or replay requirements that the package excludes.
- Isolate tests from machine-global configuration, including deliberate no-project-configuration
  cases. A no-project-config test must still neutralize unrelated global configuration unless the
  Contract explicitly makes that global state part of the scenario.
- When runtime boundaries cannot be exercised in tests, record the uncovered Contract behavior for
  Stage 6 instead of pretending it is covered.

Replay comparisons ignore generated identifiers, timestamps, and other nondeterministic envelope
fields unless the Contract governs them. Compare governed sequence and deterministic payload
semantics.

## Applicable Checks

Build a Contract coverage mapping with no required behavior silently omitted. Confirm each test
would fail under the wrong behavior it claims to detect. Use the Review Package template for inline
delivery evidence.

## Output / Next Action

Present the test changes, results, coverage mapping, and explicitly uncovered runtime behavior.
Hand the evidence to `06-observe.md`.
