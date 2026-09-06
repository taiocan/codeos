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

Read the selected doctrine, architecture and Implementation Profile policies, and the Codeos
Mechanics policy for the fixed delivery obligations below. Load the three
approved Specification Package artifact types and applicable approved architecture from the
canonical locations defined by the Downstream Project Layout Contract. Verify the package records
one valid approval and has not materially changed since. Run the architecture inspector for the
feature and stop on invalid metadata, draft membership, conflicting membership, or an unresolved
project-level decision.

Under the selected Workflow Governance policy, entry to the bounded delivery cycle is also gated on
the Feature Development checkpoints F1–F2 being mechanically verified as present and current —
`codeos-workflow status --workflow feature --subject <feature-id>` reports F1 and F2 as PASS. This
confirms evidence exists; it raises no acceptance bar.

Resolve an approved Implementation Profile only as its policy specifies. When approved architecture
selects a technology covered by an advisory pattern, consult that pattern. The profile governs
language and the architecture governs technology selection; patterns authorize neither and never
override approved behavior or architecture.

## Task

- Implement per the Contract's Feature Impact Accounting: land the change to every Platform
  Baseline tier the Contract marks changed, together, inside this delivery cycle (`vertical_slice`
  mechanic) — do not implement a GUI-visible outcome on the backend alone and defer its GUI tier to
  a later cycle.
- Once the first runnable implementation across the touched tiers passes basic integration smoke,
  and before the full verification loop in `05-tests.md` proceeds, surface a clearly labeled
  Development Preview to the human: *"Development Preview — direction and UX review only;
  verification is incomplete."* This is a direction check, not an approval gate.
  - Route the resulting feedback per doctrine: feedback that would change approved behavior returns
    to the affected Specification Package for revision and human re-approval before implementation
    continues. Feedback that concerns only implementation or UX choices within already-approved
    behavior stays here in implementation. Do not apply a behavior-changing correction directly
    without that return trip — the preview is a direction check, not a side channel for redefining
    approved meaning.
  - Record the human's direction call as the Early Development Preview decision receipt in
    `.codeos/06-workflow/decisions.jsonl`: `codeos-workflow decide --workflow feature --subject
    <feature-id> --checkpoint early_preview --result
    <direction_confirmed|behavior_revision_required|implementation_or_ux_refinement_required>`. The
    receipt records that the direction call happened and binds it to the current Specification
    Package and implementation state; it carries no product authority.
- Implement every Contract requirement and no additional governed outcome.
- In `events` mode, emit only Event-Schema-authorized governed events and propagate correlation as
  specified. In `external-observation` mode, do not invent governed internal events; preserve the
  declared observation boundary.
- Keep internal technical errors distinguishable from Contract-approved behavioral failures and
  map only authorized failures to authorized events or signals.
- Use normal helpers, types, validation, logging, and established patterns when they preserve all
  governed boundaries.
- Make feature-local design decisions inside approved architectural boundaries, including ones that
  would be costly to reverse; do not promote those choices into project-level architecture.
- Record a feature-local structural decision only when it is not cheap to undo, as
  `decision | rationale | affected scope`. Ordinary class, module, function, helper, validation, and
  error-style choices are never recorded, and a routine feature records nothing. A decision that
  establishes or constrains a shared boundary is not recorded here at all; it stops and returns to
  architecture synthesis.
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
Contract satisfaction mapping, applicable event mapping, failure mapping, any costly-to-reverse
feature-local structural decisions, and any genuine deferral trace. The inline decision record is
never written to a file. Separately, this stage MAY create or update a Module Design Note at
`.codeos/03-design/<module-slug>.md` when the doctrine's rule warrants one; the note is descriptive
and governs nothing. Surface the Development Preview once basic smoke passes, resolve any
behavior-changing feedback through the affected Specification Package first, then continue directly
to `05-tests.md` without an intermediate approval unless the human requests one.
