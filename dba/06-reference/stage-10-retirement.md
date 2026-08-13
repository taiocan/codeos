---
component_question: How should legacy Stage 10 registry state be classified after formal Stage 10 retirement?
out_of_scope: Current DBA lifecycle semantics, automatic downstream migration, historical record deletion, and implementation procedure.
---

# Stage 10 Retirement Migration

Formal Stage 10 is no longer part of current Codeos workflows. Existing registry values and
architectural-refinement records remain historical input and must not be deleted automatically.

Classify each active legacy item once:

- If it changes observable behavior, move the work to the affected feature's Specification Package
  or targeted-refinement path.
- If it establishes or changes a project-level architectural boundary for governed features, create
  or reopen the applicable architecture scope with `approval: null`.
- Otherwise remove it from active DBA workflow state and track it through the project's normal
  engineering process.

Current registries author `current_stage: 0–9 | null` and do not create
`architectural_refinements`. Tools may continue reading legacy `current_stage: 10` and ignored
`architectural_refinements` sections during the compatibility period.
