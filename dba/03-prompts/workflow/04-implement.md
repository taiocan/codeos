---
component_question: How should approved behavior and applicable architecture constrain implementation?
out_of_scope: Changing approved requirements, defining test evidence, runtime observation, and final acceptance.
---

# Stage 4: Constrained Implementation

<!-- DOCTRINE ADAPTER: delivery-entry -->

## Purpose

Implement the approved Specification Package under applicable approved architecture and
Implementation Profile constraints, without adding governed behavior.

## Inputs / Prerequisites

Read the selected doctrine, architecture and Implementation Profile policies. Load the three
approved Specification Package artifact types and applicable approved architecture from the
canonical locations defined by the Downstream Project Layout Contract. Verify the package records
one valid approval and has not materially changed since. Run the architecture inspector for the
feature and stop on invalid metadata, draft membership, conflicting membership, or an unresolved
project-level decision.

Resolve an approved Implementation Profile only as its policy specifies. Consult a language pattern
when applicable; project architecture and approved behavior remain authoritative.

## Task

- Implement every Contract requirement and no additional governed outcome.
- In `events` mode, emit only Event-Schema-authorized governed events and propagate correlation as
  specified. In `external-observation` mode, do not invent governed internal events; preserve the
  declared observation boundary.
- Keep internal technical errors distinguishable from Contract-approved behavioral failures and
  map only authorized failures to authorized events or signals.
- Use normal helpers, types, validation, logging, and established patterns when they preserve all
  governed boundaries.
- Identify affected structural chokepoints before editing and use verification proportional to
  their risk.
- Stop when implementation would require changing approved behavior, event semantics, architecture,
  safety, authorization, or integrity.

If an approved artifact explicitly deferred a material choice that implementation must settle,
record its source, resolution, implementation location, and whether it is final or interim. Silence
and ordinary implementation freedom are not deferrals.

## Applicable Checks

Map each Contract clause to code. In event mode, map each emitted governed event to its schema
definition. Record internal-to-behavioral failure mappings. Use the current Review Package template
for inline delivery evidence rather than restating its format here.

## Output / Next Action

Implement in the project's native source layout. Present the changed paths, implementation,
Contract satisfaction mapping, applicable event mapping, failure mapping, and any genuine deferral
trace. This stage creates no separate workflow artifact. Continue directly to `05-tests.md` without
an intermediate approval unless the human requests one.
