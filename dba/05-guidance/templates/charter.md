---
approval: null
---

# Solution Charter: [solution name]

<!--
The one authoritative statement of why this solution exists, which primary decision it supports,
what success means, where its boundary lies, and which obligations apply across it. Canonical path:
.codeos/00-project/charter.md

Governance: front matter owns approval; Git owns history. There is no status or version field.
A material change returns `approval` to null before the replacement decisions are used.

  approval:
    by: Human name
    at: 2026-08-16

This artifact does not define feature behavior. Feature outcomes, guarantees, and scope belong to
Intents; observable behavior belongs to Contracts; structure belongs to Architecture Scopes.
Delete any section that has no content rather than filling it with "none".
-->

## Problem

[What is wrong, difficult, expensive, slow, risky, or missing today, and for whom? Describe the
affected people, the current situation, the pain or opportunity, known causes, and consequences.
Do not start from technology.]

## Vision

[What the affected people's world looks like once the problem is solved well. Understandable
without implementation knowledge. This is the solution concept, not architecture.]

## Decision Supported

[One concise statement naming the primary actor and primary decision supported by the solution.
This is an alignment lens, not an acceptance requirement or feature behavior.]

## Outcomes

<!--
What measurable result must the solution create? An outcome is not a feature.
  Feature: dynamic map filtering.
  Outcome: users find relevant plots quickly.
Each outcome keeps a stable O-# identity. Intents record the outcomes they serve, which is how a
later outcome change identifies the features it may affect. Never reuse a retired identity.
-->

| ID | Outcome | How success is measured |
|---|---|---|
| O-1 | [what must become true for people using the solution] | [the measure or observation that shows it] |

## Scope and Boundary

```text
IN SCOPE
[capability that belongs to this solution]

OUT OF SCOPE
[capability deliberately excluded]
```

[Add durable constraints that limit the solution's choices — existing systems, regulation, expected
scale, required technologies, available skills — when they are not already owned by source,
configuration, approved DBA artifacts, or applicable architecture.]

## System Constraints

<!--
Obligations that apply across features or to the whole solution. Admission test: it applies across
features or solution-wide, AND it is a product or system obligation rather than an implementation
choice. A constraint specific to one feature belongs in that feature's Contract instead.

System Constraint is the broader category. Cross-cutting quality requirements (performance,
availability, security, usability, maintainability, reliability, scalability) are one type.
Regulatory obligations, mandated interoperability, deployment restrictions, and externally imposed
technology constraints are others.

Every constraint needs a verification route: how conformance will be shown, and by whom. An
affected Specification Package demonstrates conformance when its implementation can affect the
constraint. A threshold must state its workload, operating context, and rationale; a deliberately
new target needs no prior evidence, but an unexplained number is not a requirement.
-->

| ID | Constraint | Type | Verification route |
|---|---|---|---|
| C-1 | [the obligation, with workload, operating context, and rationale when it states a threshold] | [quality \| regulatory \| interoperability \| deployment \| technology] | [how conformance is shown and where] |

<!--
Changing a constraint, the scope, or the boundary requires an explicit impact assessment of
affected artifacts — recorded served outcomes do not identify that impact. Existing approvals are
not automatically invalidated, but affected artifacts are reassessed before their next
implementation or acceptance.
-->
