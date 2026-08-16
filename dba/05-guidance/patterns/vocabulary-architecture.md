---
component_question: How should configurable vocabulary be owned and consumed across modules?
out_of_scope: Feature behavior, mandatory storage strategy, architecture approval, and display design.
---

# Pattern: Vocabulary Architecture

Use when configurable types, statuses, roles, relationships, or other concepts must mean the same
thing across modules. Shared downstream project meanings live in
`.codeos/00-project/terminology.md` when that optional glossary exists. CodeOS/DBA meanings remain
owned by `../terminology.md` in the toolkit.

## Rules

1. Exactly one vocabulary owner defines concepts, accepts representations, validates them, and
   exposes resolution.
2. Consumers make domain decisions using resolved concept identity, never aliases, casing, or
   canonical strings.
3. One feature uses one strategy consistently: resolve on use, normalize on write, or stable
   concept identifiers.

A representation appearing in domain comparison, branching, storage policy, or pattern matching is
a concept leak. Move interpretation back to the vocabulary boundary.

## Contract and Verification

When vocabulary affects behavior, the Contract states that equivalent representations produce the
same decision and includes a falsifying fixture whose alias and canonical form differ visibly, such
as `risk` and `Risk`. Implementation review checks every vocabulary comparison and tests prove that
the chosen strategy is applied uniformly.
