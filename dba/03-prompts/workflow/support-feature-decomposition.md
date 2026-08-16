---
component_question: How should one problem be decomposed into candidate feature boundaries when needed?
out_of_scope: Simple feature intake, formal Intent, approval, architecture decisions, and implementation.
---

# Feature Decomposition

## Purpose

Optionally preserve decomposition reasoning when one problem needs separation into multiple features.
This is not a default workflow step or governed artifact.

## Inputs / Prerequisites

Read the approved Solution Charter and
`.codeos/toolkit/dba/05-guidance/templates/feature-decomposition.md`. Use the Charter's outcomes
and scope with the human's description. If no approved Charter exists, return to
`support-solution-charter.md`. If the problem already has one clear human outcome and boundary,
skip this prompt and proceed to Intent.

## Task

1. Identify the shared problem or opportunity and affected human actors within the approved
   Charter boundary.
2. Separate candidate features by meaningful outcome and execution boundary, not by code layer.
3. Record only dependencies or open decisions that materially affect that decomposition.
4. Do not allocate feature IDs; IDs belong to the resulting Intents.
5. Do not write guarantees, Contract scenarios, events, or implementation design.

## Output / Next Action

Present the candidate boundaries. Save the optional decomposition at
`.codeos/00-discovery/<topic-slug>.md` only when its reasoning has durable value. After
confirmation, route each accepted candidate independently to `01-intent.md`.
