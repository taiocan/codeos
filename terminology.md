# Codeos Terminology

This is the canonical lookup and definition source for recurring, project-specific Codeos terms.
Ordinary technical vocabulary does not belong here. Git preserves the history of every definition.

## Definitions

| Term | Definition |
|---|---|
| **Alias** | An alternative input form accepted for compatibility, migration, or user preference and resolved before domain logic runs. |
| **Architecture Scope** | The single artifact for one project-level architectural scope. Its filename identifies the scope; it owns feature membership, structural decisions, and recorded human approval and cannot invent or alter behavior. |
| **Artifact status** | The lifecycle value of a governed artifact: `DRAFT`, `APPROVED`, `IN_PROGRESS`, or `COMPLETE`; onboarding Intent may also use `HYPOTHESIZED_INTENT`. |
| **Behavioral Contract** | Observable truths derived from Intent, normally expressed as Given/When/Then conditions. |
| **Behavioral Event** | A runtime event representing a verified outcome. |
| **Behavioral invariant** | A condition that must remain true in externally observable behavior, independent of internal implementation structure. |
| **Canonical** | The single stable runtime identifier that names a Concept within a vocabulary. |
| **Concept** | The semantic identity used by domain logic, independent of its string or storage representation. |
| **Concept Dependency Rule** | Domain logic compares resolved Concept identity rather than vocabulary representations. |
| **Concept leak** | A defect in which a vocabulary representation escapes the resolution boundary into domain logic. |
| **Consequential conformance repair** | A change that restores already-approved behavior where an incorrect repair could have consequential effects; it is not a new semantic decision. |
| **Correlation ID** | The UUID linking all runtime events from one feature execution chain. |
| **Declarative Behavioral Architecture (DBA)** | The Codeos model, also called Intent-Driven System (IDS), that carries human intent through behavioral contracts, implementation, runtime evidence, reconciliation, replay, and targeted refinement. |
| **Event Spine** | The complete ordered set of events a feature is permitted to emit. |
| **Evidence Quality** | Environment fidelity independent of semantic alignment: 1 Specification, 2 Static, 3 Simulated, 4 Real boundary, 5 Production. |
| **External Event** | A runtime event representing a side effect on a system outside the governed boundary. |
| **Failure Event** | A runtime event representing a classified error condition. |
| **HYPOTHESIZED_INTENT** | The status of an onboarding Intent draft that has not yet passed through the specification-approval adapter. |
| **Implementation Profile** | An approved project-level statement of preferred implementation language and scope that implementation must consult. |
| **Intent** | Why a feature exists, expressed as actor and desired outcome without implementation details. |
| **NORMAL** | The default Codeos self-development classification for work that does not reasonably risk an approved consequential boundary. |
| **Normalize-on-read** | A vocabulary strategy that stores the original representation and resolves it at every comparison site. |
| **Normalize-on-write** | A vocabulary strategy that resolves input at ingestion and stores Concept identity. |
| **Observational Event** | A runtime event representing a raw observed fact. |
| **Payload-drift status** | The comparison result for an observed payload field: `MATCH`, `TYPE_MISMATCH`, `ABSENT`, or `EXTRA`. |
| **Protected invariant** | The condition identified before a PROTECTED semantic change that must remain true after implementation. |
| **PROTECTED semantic change** | An intentional Codeos change that could alter approved behavior or authority, project architecture, persistent compatibility, safety or integrity, or human-control boundaries. |
| **Reconciliation Review** | A structural comparison of governed artifacts for gaps, mismatches, and missing evidence. |
| **Reconciliation status** | The alignment result for a reconciliation item: `ALIGNED`, `GAP`, `MISMATCH`, or `MISSING`. |
| **Replay Verification** | Confirmation that a runtime event log conforms to its approved schema and contract sequence. |
| **Representation Ban Rule** | Domain logic must not store, compare, branch on, or pattern-match vocabulary representations; it uses resolved Concept identity. |
| **Resolution** | Mapping an Alias or canonical representation to its Concept through the vocabulary owner's API. |
| **Shared Infrastructure Module** | A module used by at least two feature modules that provides only mechanical infrastructure and contains no domain logic. |
| **Specification Package** | The Intent, Contract, and Event Schema considered together by the specification-approval adapter. Decision semantics come only from the selected doctrine. |
| **Targeted Refinement** | The smallest effective change for a specific observed problem, not a rewrite. |
| **Vertical Drift** | Accumulation of domain logic in shared infrastructure, bypassing feature-isolation boundaries. |
| **Vocabulary consumer** | A module that uses vocabulary-defined Concepts through the resolution API without interpreting representations itself. |
| **Vocabulary owner** | The single module that defines Concepts, accepts Aliases, validates the vocabulary, and exposes Resolution. |
