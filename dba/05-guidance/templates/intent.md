---
artifact_type: intent
feature_id: F-####
status: DRAFT
serves_outcomes: [O-1]
approved_by:
approved_at:
derived_contracts: .codeos/01-specification/contracts/<feature-id>_contract.md
reader_model:  # stable-topic | known-to-new | whole-before-parts | preview-then-traverse
---

# Intent: [feature_id] — [short name]

## Summary
- [key message]
- [key message]

Oversimplification risk: this summary can omit nuance the full artifact carries. Read the
relevant section directly before relying on the summary alone for a consequential decision.

<!--
PURPOSE OF THIS FILE:
Defines why this feature exists and what meaningful outcomes it enables.
This is NOT a requirements document, feature list, or architecture plan.
Intent must remain stable even if implementation changes significantly.

METADATA — recording behavior is owned by the specification-approval doctrine adapter.
`serves_outcomes` names the approved Solution Charter outcomes this feature contributes to. It is
how a later outcome change identifies the features it may affect, so keep it accurate and non-empty.
Changes to Charter scope, boundary, or System Constraints are NOT discoverable this way and require
an explicit impact assessment.

RULES:
- State outcomes, not mechanisms
- Use "Actor can [outcome]" form for every statement
- No implementation details (no APIs, databases, frameworks, file formats)
- No feature decomposition or workflow steps
- No observability mechanics (no events, logs, metrics)
- Guarantees must be enforceable and testable
- Fits on one screen — if it expands into architecture, it is no longer intent
-->

[SystemOrModule] exists to let [Actor] [achieve meaningful outcome].

Specifically:
- [Actor] can [outcome-oriented ability]
- [Actor] can [outcome-oriented ability]
- [Actor] can [outcome-oriented ability]

## Stable Guarantees

<!--
These are invariants — what is always true regardless of inputs.
Each must be testable. Remove aspirational language.

Good: "Changes apply atomically"
Bad: "System is highly reliable"
-->

- [invariant — enforceable, testable]
- [invariant — enforceable, testable]

## Scope Boundary

<!--
Explicit exclusions prevent scope creep during contract derivation.
List what this feature does NOT do.
-->

This feature does NOT:
- [explicit exclusion]
- [explicit exclusion]
