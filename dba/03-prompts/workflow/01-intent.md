---
component_question: How should implementation-independent feature intent be captured and checked for ambiguity?
out_of_scope: Behavioral contracts, event schemas, architecture design, implementation, and approval mechanics.
---

# Stage 1: Intent Capture

## Purpose

Produce a compact, implementation-independent Intent describing why the feature exists, what its
human actor can achieve, what must remain true, and what is excluded.

## Inputs / Prerequisites

Read `.codeos/toolkit/dba/05-guidance/templates/intent.md`; it owns the artifact structure and core
writing rules. Read the approved Solution Charter at `.codeos/00-project/charter.md`; a feature
Intent requires one. Use an optional confirmed discovery brief when present, otherwise the human's
description. Resolve existing artifacts through the Downstream Project Layout Contract. When
`.codeos/00-project/terminology.md` exists, consult it before defining terms.

## Task

- Express purpose and outcomes from the human actor's perspective.
- Record in `serves_outcomes` the Charter outcomes this feature contributes to. If the feature
  serves no approved outcome or falls outside the Charter's scope boundary, stop and route the
  question to `support-solution-charter.md` rather than widening the solution silently.
- State stable guarantees as enforceable, testable invariants; they need not use actor/outcome
  grammar when that would obscure the invariant.
- Keep mechanisms, implementation technology, observability, workflow steps, and architecture out
  of Intent.
- Name only compatibility behavior this feature actually owns; avoid promising that all existing
  behavior remains unchanged.
- Reuse project-wide terms exactly as defined by the project glossary. Define only feature-local
  terms in the Intent, including legitimate narrower refinements that do not contradict the shared
  meaning.
- When a new specialized meaning must remain consistent across features, propose one project-wide
  definition. After the human confirms it, create or update
  `.codeos/00-project/terminology.md` from the project-terminology template and use that meaning in
  the Intent. Do not create a glossary for ordinary language or a feature-local term.
- Do not duplicate an existing approved Intent definition into the glossary merely for
  normalization. Promote it only during a later substantive revision of the affected package.
- When configurable domain vocabulary is involved, identify its owner or consumer and the concepts
  relied upon; consult the vocabulary pattern only when applicable.
- Surface unclear actors, vague outcomes, implicit constraints, excessive scope, and unresolved
  terms. Offer a reasoned proposal or bounded alternatives when possible; otherwise name the
  missing decision or evidence.
- For a new feature, allocate the next `F-####` after scanning IDs in Intent, Contract, Event
  Schema filenames and architecture-scope membership. Partial packages are normal. Stop only when
  incompatible artifacts claim the same identity; never require later-stage artifacts to exist.

## Applicable Checks

Apply the template rules and confirm that outcomes are actor-centered, guarantees are testable,
scope is explicit, `serves_outcomes` names existing Charter outcomes, and no mechanism or
observability detail leaked into the artifact. Revise
mechanical failures before presenting the draft; leave genuine product decisions visible for the
human.

## Output / Next Action

Create `.codeos/01-specification/intents/[feature_id].md` as `DRAFT` and present it followed by only
unresolved ambiguities. Keep it open to revision and hand it to `02-contract.md`.
