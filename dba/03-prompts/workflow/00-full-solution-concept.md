---
component_question: How should an optional whole-solution direction be explored before consequential commitments become binding?
out_of_scope: Approving behavior, architecture, language, or verification evidence; creating a mandatory stage; and detailed implementation design.
---

# Full-Solution Concept

## Purpose

Develop a coherent whole-solution recommendation when feature-level artifacts do not make the
overall product, quality priorities, major solution shape, or consequential technology tradeoffs
easy to understand. This workflow is optional and non-authoritative. It creates no approval gate.

## Inputs / Prerequisites

Use relevant discovery, existing approved authority, known constraints, and direct human decisions.
Clearly distinguish existing binding decisions from recommendations. Do not reopen approved
decisions merely because the concept has a broader horizon.

## Task

1. **Frame** — establish the product problem, primary actors, target experience, critical
   interactions, boundaries, and three to five ranked architecture drivers. Keep the target
   product experience distinct from the order in which capabilities will be delivered.
2. **Explore** — sketch the logical responsibilities and consequential runtime or data flows that
   support the target experience; compare only consequential options; record the current
   recommendation, quality expectations, confidence, risks, assumptions, open decisions,
   validation needs, technology direction, and delivery evolution.
3. **Promote** — identify justified commitments and route them to their existing authoritative
   owners. Do not make them binding in the concept.

Use the smallest set of views needed to explain system context, major solution structure, and any
consequential runtime or data flow. Do not require a fixed number or notation.

Name logical responsibilities independently of technologies. Use conventional architecture
terminology when it makes the solution easier to recognize, but do not force a layered structure
or another architecture style where it would misrepresent the solution. Map consequential
technologies to logical responsibilities separately from the logical view.

For an interaction that materially drives architecture, state the actor or stimulus, relevant
operating context, expected response, and a candidate measure or validation need. Keep an
unsupported threshold visibly hypothetical until representative workload and operating conditions
are known. Do not promote mechanisms such as caching, clustering, tiles, queues, service splits,
or additional stores without evidence that the simpler direction is insufficient.

For consequential choices only, distinguish:

- `EXISTING BINDING` — already governed by approved authority;
- `RECOMMENDED` — the current preferred direction;
- `EXPERIMENT` — requires a bounded evaluation before adoption;
- `OPEN` — intentionally unresolved.

For each important validation need, state what must eventually be proven, why it matters, how it
might be proven, when it must be proven, and where it belongs if promoted:

- an observable-behavior constraint belongs in the Contract;
- a structural or quality-realization constraint belongs in the Architecture Scope;
- verification of an existing binding requirement belongs to tests, runtime evidence,
  reconciliation, or replay during normal delivery.

A concept validation need remains a non-authoritative question until promoted. Do not require
executable tests or verification evidence to complete the concept.

## Output / Next Action

Present the concept inline unless the human requests durable context. When saved, use the current
template at `.codeos/toolkit/dba/05-guidance/templates/full-solution-concept.md` and write exactly
`.codeos/00-discovery/solution-concept.md`.

The concept has no status, approval, version, synchronization check, or mandatory update cadence.
Reassess it only when new evidence, a new product area, or an architectural contradiction
materially changes the whole-solution recommendation.

Promote justified commitments through their existing owners:

- behavior → Specification Package;
- shared structure, data, runtime, or API → Architecture Scope;
- language constraint → Implementation Profile.

Then stop.
