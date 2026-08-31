---
component_question: How should the solution's purpose, primary supported decision, outcomes, boundary, and cross-cutting constraints be established and approved?
out_of_scope: Feature behavior, architecture decisions, implementation, verification evidence, and non-authoritative exploration.
---

# Solution Charter

<!-- DOCTRINE ADAPTER: purpose-approval -->

## Purpose

Establish the one approved statement of why the solution exists, which primary decision it
supports, what success means, where its boundary lies, and which obligations apply across it. This
support workflow owns that approval boundary; a Charter must be approved before the first
Specification Package approval.

## Inputs / Prerequisites

Read the selected doctrine and `.codeos/toolkit/dba/05-guidance/templates/charter.md`. Use the
human's description, relevant Solution Framing, and explicit human decisions. Framing may propose
outcomes, scope, and constraints, but only their promotion into an approved Charter makes them
governed truth; do not synchronize framing material afterwards.

When revising an existing Charter, read the current artifact first and treat its approved content
as binding until the replacement is approved.

An existing approved Charter remains valid without a `Decision Supported` section. Do not revise
one solely to add it. A new or materially revised Charter uses the current template and includes
the section.

## Task

- State the problem from the affected people's situation, not from technology.
- Describe the vision so it is understandable without implementation knowledge.
- Name the primary actor and primary decision the solution supports in one concise statement. Keep
  it consistent with Problem, Vision, Outcomes, and Scope. It is an alignment lens, not an
  acceptance requirement; do not turn it into feature behavior.
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

When a Charter is approved over Intents that were approved before it existed, those Intents record
no `serves_outcomes` yet, and the doctrine requirement that an Intent records the outcomes it serves
is not yet satisfied. Propose each such Intent's `serves_outcomes` as part of constructing this
Charter. Proposing is not recording: leave those Intents' `status`, `approved_by`, and `approved_at`
untouched, and treat the mapping as confirmed only once the human approves this Charter.

An Intent that cannot be truthfully mapped to an approved outcome is reported as unresolved and
excluded from the proposed set; never assign a mapping to make the set complete. Such an
inconsistency is a Charter question — either this Charter lacks an outcome the feature serves, or
the feature lies outside the scope boundary — so, as with anything else unresolved above, keep
`approval: null` while an in-scope Intent cannot be mapped. The human resolves it by adding the
missing outcome or by explicitly placing the feature outside the boundary; once outside, it is not
in the mapping set and no longer blocks approval.

## Applicable Checks

For a new or materially revised Charter, confirm that `Decision Supported` names the primary actor
and primary decision, agrees with Problem, Vision, Outcomes, and Scope, and is not treated as an
acceptance requirement. Confirm that outcomes are measurable and identified, the boundary is
explicit, every System Constraint has a type and a verification route, every threshold carries its
context and rationale, and no feature behavior or architecture decision leaked into the artifact.
Apply the selected review policy at this decision boundary.

## Output / Next Action

Create or revise `.codeos/00-project/charter.md` using the current template, keep `approval: null`
while any of the above remains unresolved, present the complete artifact with the impact assessment
when revising, and state:

`AWAITING HUMAN APPROVAL OF THE SOLUTION CHARTER`

When Intents approved before this Charter exist, list the proposed `serves_outcomes` per Intent
alongside the artifact, so the human approving the Charter is approving those mappings rather than
receiving them as hidden detail.

After explicit approval, record `approval.by` and `approval.at` on the Charter. Where mappings were
proposed, record into each affected Intent exactly the `serves_outcomes` presented for approval —
do not re-infer or adjust them while recording — and change nothing else in those Intents,
including their existing `status`, `approved_by`, and `approved_at`. Then route feature work to
`01-intent.md`, recording the outcomes each Intent serves.
