---
component_question: How should the solution's purpose, outcomes, boundary, and cross-cutting constraints be established and approved?
out_of_scope: Feature behavior, architecture decisions, implementation, verification evidence, and non-authoritative exploration.
---

# Solution Charter

<!-- DOCTRINE ADAPTER: purpose-approval -->

## Purpose

Establish the one approved statement of why the solution exists, what success means, where its
boundary lies, and which obligations apply across it. This support workflow owns that approval
boundary; a Charter must be approved before the first Specification Package approval.

## Inputs / Prerequisites

Read the selected doctrine and `.codeos/toolkit/dba/05-guidance/templates/charter.md`. Use the
human's description, relevant Solution Framing, and explicit human decisions. Framing may propose
outcomes, scope, and constraints, but only their promotion into an approved Charter makes them
governed truth; do not synchronize framing material afterwards.

When revising an existing Charter, read the current artifact first and treat its approved content
as binding until the replacement is approved.

## Task

- State the problem from the affected people's situation, not from technology.
- Describe the vision so it is understandable without implementation knowledge.
- Express outcomes as measurable results rather than features, and give each a stable `O-#`
  identity. Never reuse a retired identity.
- Record the scope boundary explicitly, in and out.
- Record System Constraints that apply across features or solution-wide. Admit an entry only when it
  is a product or system obligation rather than an implementation choice; a constraint specific to
  one feature belongs in that feature's Contract.
- Treat System Constraint as the broader category. Cross-cutting quality requirements are one type;
  regulatory obligations, mandated interoperability, deployment restrictions, and externally imposed
  technology constraints are others.
- Give every System Constraint a verification route stating how conformance will be shown and where.
  A constraint with no route is not admissible.
- State workload, operating context, and rationale for every threshold. A deliberately new target
  needs no prior evidence; an unexplained number is not a requirement.
- Keep feature behavior, guarantees, architecture, and implementation out of the Charter.
- Surface unclear beneficiaries, unmeasurable outcomes, and boundary ambiguity rather than inventing
  answers.

When revising an approved Charter, first return `approval` to `null`, then assess impact:

- an outcome change identifies affected features through their recorded `serves_outcomes`;
- a scope, boundary, or System Constraint change requires an explicit impact assessment naming the
  approved artifacts that may be affected.

Existing approvals are not automatically invalidated. Report each affected artifact and the point at
which it must be reassessed — before its next implementation or acceptance.

## Applicable Checks

Confirm that outcomes are measurable and identified, the boundary is explicit, every System
Constraint has a type and a verification route, every threshold carries its context and rationale,
and no feature behavior or architecture decision leaked into the artifact. Apply the selected review
policy at this decision boundary.

## Output / Next Action

Create or revise `.codeos/00-project/charter.md` using the current template, keep `approval: null`
while any of the above remains unresolved, present the complete artifact with the impact assessment
when revising, and state:

`AWAITING HUMAN APPROVAL OF THE SOLUTION CHARTER`

After explicit approval, record `approval.by` and `approval.at`. Then route feature work to
`01-intent.md`, recording the outcomes each Intent serves.
