---
component_question: How should tests prove approved observable behavior and observation semantics?
out_of_scope: Feature implementation, runtime evidence collection, reconciliation, replay execution, and approval.
---

# Stage 5: Behavioral and Observation Tests

## Purpose

Test approved observable behavior without coupling tests to private implementation.

## Inputs / Prerequisites

Load the approved Contract and Event Schema types through the Downstream Project Layout Contract,
then read the Stage 4 implementation. The Contract owns observation mode, scenarios, failures,
invariants, and any minimum environment requirement.

Read the selected Codeos Mechanics policy. Its `validation` mechanics are fixed, not
project-configurable: `smoke`, `behavior`, and `repeatability` apply to every feature; `playwright`
and `human_ux` apply whenever the Contract has a GUI-visible outcome; and where the policy defines
`data_integrity`, it applies whenever the feature persists or changes data. When approved
architecture selects Svelte for a browser interface — the Platform Baseline default — consult
`.codeos/toolkit/dba/05-guidance/patterns/svelte-gui-verification.md`. It supplies proportional
verification technique; the Codeos Mechanics policy, not this pattern, is what makes its
application mandatory rather than optional.

Under the selected Workflow Governance policy, verification is not complete while the Early
Development Preview (F4) is unresolved for a GUI-visible feature. Each applicable `validation`
mechanic's evidence is the mechanical verification record `codeos-workflow check --workflow feature
--subject <feature-id>` writes on a pass, bound to the Specification Package and implementation
state and stale the moment either drifts. This records that the verification ran and passed; it
makes no judgment that the behavior is adequate, and it changes none of the test requirements
below.

## Task

- Add behavioral tests covering every applicable Contract scenario, governed failure, and invariant
  falsifier. Test Contract notes only when they state observable requirements.
- Write automated tests for every Contract quality requirement whose declared verification method is
  a test, exercising it under the stated workload and operating context. For a requirement verified
  by measurement, analysis, inspection, review, or operational evidence, record the method and what
  it must show instead of fabricating a test that does not prove it.
- Do the same for any Charter System Constraint this feature's implementation can affect, using the
  conformance route the Contract recorded.
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
- Confirm basic DB↔backend↔GUI integration smoke actually runs for any tier the Feature Impact
  Accounting table marked changed (`smoke` mechanic), before deeper verification.
- For a Contract with a GUI-visible outcome, add at least one bounded, integrated Playwright journey
  demonstrating the critical user-visible outcome against the real Postgres, real Rust backend, and
  real Svelte app — no tier mocked (`playwright` mechanic). Record the separate human UX validation
  (`human_ux` mechanic) as evidence distinct from behavioral correctness; it is not satisfied by a
  passing Playwright assertion alone.

Replay comparisons ignore generated identifiers, timestamps, and other nondeterministic envelope
fields unless the Contract governs them. Compare governed sequence and deterministic payload
semantics.

## Applicable Checks

Build a Contract coverage mapping with no required behavior or quality requirement silently
omitted. Confirm each test would fail under the wrong behavior it claims to detect. When a test is
presented as acceptance evidence, confirm that it observes the boundary named by the acceptance
claim rather than only an internal proxy. This adds no separate record for ordinary tests. Use the
Review Package template for inline delivery evidence.

## Output / Next Action

Add tests in the project's native test layout. Present changed paths, results, coverage mapping,
and explicitly uncovered runtime behavior. This stage creates no separate workflow artifact. Hand
the evidence to `06-observe.md`.
