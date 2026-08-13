---
component_question: How should a feature problem and scope be elicited into a concise Feature Brief?
out_of_scope: Formal Intent, Contract, Event Schema, architecture approval, and implementation.
---

# Feature Brief

## Purpose

Help the human produce one plain-language Feature Brief for Stage 1. The brief captures the problem,
actor, desired outcome, unresolved questions, and suspected dependencies without formalizing DBA
requirements.

## Inputs

Use the human's filled template, description, or initial idea. Read
`.codeos/dba/05-guidance/templates/feature-brief.md`; it owns the artifact shape and readiness
check. Do not repeat questions the supplied material already answers.

## Task

1. Establish whether this is a new feature or a refinement, the problem, human actor, desired
   outcome, rough boundary, and suspected dependencies.
2. For a refinement, identify the parent feature and observed or explicitly approved trigger.
3. Ask gaps in one batch when the feature is clear; use short conversational rounds only when its
   problem or scope remains ambiguous.
4. Record genuine open questions. Do not manufacture one solely to complete the template.
5. Assign identifiers according to the current conventions and registry. New features receive the
   next unused `F-####`; refinements reuse their parent identifier.
6. Fill the template using only human-provided information. Mark unresolved content as unknown
   rather than inventing it, then apply the template's readiness check.

Do not write Intent guarantees, Contract scenarios, events, or implementation design. Rough scope
notes are discovery input, not governed Scope Boundary language.

## Output / Next Action

Present the completed brief ready to save at the template's recommended path, list unresolved gaps,
and ask the human to confirm its accuracy. After confirmation, route it to `01-intent.md`; do not
perform Stage 1 unless requested.
