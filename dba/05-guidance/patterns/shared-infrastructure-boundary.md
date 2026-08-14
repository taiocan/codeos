---
component_question: How can shared infrastructure remain free of feature-specific domain logic?
out_of_scope: Feature behavior, mandatory module layouts, architecture approval, and technology choice.
---

# Pattern: Shared Infrastructure Boundary

Use this pattern when at least two feature modules depend on the same infrastructure module.

## Boundary Test

Before adding shared code, ask:

> Would infrastructure with no knowledge of the domain vocabulary need this?

If yes, the addition is likely mechanical infrastructure: event/log adapters, generic transport or
storage mechanics, IDs, configuration plumbing, or domain-neutral DTOs. If no, keep the business
rule, status derivation, vocabulary query, aggregation, or feature-specific result in its owning
feature or vocabulary module.

An exception is acceptable only when a concrete duplication or dependency problem justifies it.
Record that architectural decision in an applicable Architecture Scope when it establishes or
changes a project-level responsibility boundary; otherwise the code and Git diff are sufficient.

## Verification

Inspect dependencies into the shared module and confirm its public API contains no feature-specific
decision or vocabulary interpretation. Preserve feature behavior with proportional tests.
