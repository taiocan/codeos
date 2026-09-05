---
component_question: What do recurring project-specific Codeos terms mean authoritatively?
out_of_scope: Ordinary technical vocabulary, workflow instructions, historical definitions, and downstream project glossaries.
---

# Codeos Terminology

This is the canonical lookup and definition source for recurring, project-specific Codeos terms.
Ordinary technical vocabulary does not belong here. Git preserves the history of every definition.

## Definitions

| Term | Definition |
|---|---|
| **Alias** | An alternative input form accepted for compatibility, migration, or user preference and resolved before domain logic runs. |
| **Architecture Scope** | The single artifact for one project-level architectural scope. Its filename identifies the scope; it owns feature membership, structural decisions, and recorded human approval and cannot invent or alter behavior. |
| **Artifact Governance** | Whether an artifact type carries approval-bearing frontmatter and full process weight, as configured project-wide in `codeos.yaml`. Contrast with Decision Governance, which it never suspends. |
| **Artifact Summary** | The Summary block required immediately after an artifact's frontmatter, previewing its key messages ahead of the full content. |
| **Behavioral Contract** | Observable truths derived from Intent, normally expressed as Given/When/Then conditions. |
| **Behavioral Event** | A runtime event representing a verified outcome. |
| **Behavioral invariant** | A condition that must remain true in externally observable behavior, independent of internal implementation structure. |
| **Canonical** | The single stable runtime identifier that names a Concept within a vocabulary. |
| **Codeos Mechanic** | A fixed delivery, verification, or communication obligation named by the Codeos Mechanics policy; a project's configuration may display it but not disable it. |
| **Concept** | The semantic identity used by domain logic, independent of its string or storage representation. |
| **Concept Dependency Rule** | Domain logic compares resolved Concept identity rather than vocabulary representations. |
| **Concept leak** | A defect in which a vocabulary representation escapes the resolution boundary into domain logic. |
| **Consequential conformance repair** | A change that restores already-approved behavior where an incorrect repair could have consequential effects; it is not a new semantic decision. |
| **Correlation ID** | The UUID linking all runtime events from one feature execution chain. |
| **Decision Governance** | The doctrine-level human-control guarantees — consequential architecture approval, canonical terminology consistency, and the like — that remain active regardless of any artifact type's configured governance state. |
| **Declarative Behavioral Architecture (DBA)** | The Codeos model, also called Intent-Driven System (IDS), that carries human intent through behavioral contracts, implementation, runtime evidence, reconciliation, replay, and targeted refinement. |
| **Development Evidence** | Evidence produced before acceptance to prove the candidate implementation. Contrast with Operational Observation. |
| **Early Development Preview** | An unverified, clearly labeled preview shown to the human after basic integration smoke and before full verification, for a direction and UX check only; feedback that would change approved behavior returns to the Specification Package rather than being applied silently. |
| **Event Spine** | The complete ordered set of events a feature is permitted to emit. |
| **External assessment** | Machine-generated findings supplied by a model Codeos did not invoke. Advisory evidence for human review; it never satisfies a required review round. Contrast with a review, which Codeos obtains itself. |
| **Evidence source** | The strongest direct basis for a reconciliation result: `runtime`, `test`, `static`, or `none`. It is not a confidence or quality score. |
| **Reconciliation status** | One of `ALIGNED`, `GAP`, `MISMATCH`, or `MISSING`. The row note explains the specific issue; status names do not encode gap subtypes. |
| **External Event** | A runtime event representing a side effect on a system outside the governed boundary. |
| **Failure Event** | A runtime event representing a classified error condition. |
| **Feature ID** | The stable `F-####` identity assigned when an Intent is created. It is never reused; partial Specification Packages remain valid identities. |
| **Feature Impact Accounting** | A feature's recorded changed-or-unchanged, with-reason account of its effect on each Platform Baseline tier, replacing any requirement to touch every tier regardless of need. |
| **Implementation Profile** | An approved project-level statement of preferred implementation language and scope that implementation must consult. |
| **Intent** | Why a feature exists, expressed as actor and desired outcome without implementation details. |
| **Learning Register** | An optional queue of material, unresolved post-acceptance observations that could change governed engineering truth. It is evidence, never authority, and is not an operational log. |
| **Module Design Note** | Optional descriptive documentation of how an implementation module currently works; its normative rules are defined by the doctrine's Structural Ownership section. |
| **NORMAL** | The default Codeos self-development classification for work that does not reasonably risk an approved consequential boundary. |
| **Normalize-on-read** | A vocabulary strategy that stores the original representation and resolves it at every comparison site. |
| **Normalize-on-write** | A vocabulary strategy that resolves input at ingestion and stores Concept identity. |
| **Observational Event** | A runtime event representing a raw observed fact. |
| **Operational Observation** | A fact arising after acceptance, from real system use, that may affect governed engineering truth. It is evidence and cannot change approved behavior by itself. |
| **Outcome** | A measurable result the solution must create, identified as `O-#` in the Solution Charter. An outcome is not a feature. |
| **Oversimplification Risk Note** | The standing caveat accompanying an Artifact Summary, noting that the summary can omit nuance the full artifact carries. |
| **Platform Baseline** | The default set of architectural tiers — PostgreSQL persistence, a Rust backend, a Svelte GUI, and Docker runtime — every solution resolves unless the Solution Charter records why a tier does not apply. |
| **Protected invariant** | The condition identified before a PROTECTED semantic change that must remain true after implementation. |
| **PROTECTED semantic change** | An intentional Codeos change that could alter approved behavior or authority, project architecture, persistent compatibility, safety or integrity, or human-control boundaries. |
| **Quality Requirement** | A governed statement of how well the solution must behave, declared with a verification method. Feature-specific ones belong to the Behavioral Contract; cross-cutting ones are System Constraints. Architecture may respond to them but never originates them. |
| **Reconciliation Review** | A structural comparison of governed artifacts for gaps, mismatches, and missing evidence. |
| **Re-entry Rule** | Post-acceptance learning returns to the earliest governed authority whose truth must change. |
| **Reader Model** | The declared reader-oriented progression (Stable Topic, Known-to-New, Whole Before Parts, or Preview Then Traverse) a substantial narrative artifact states it uses, making its application inspectable rather than only instructed. |
| **Replay Verification** | Confirmation that a runtime event log conforms to its approved schema and contract sequence. |
| **Representation Ban Rule** | Domain logic must not store, compare, branch on, or pattern-match vocabulary representations; it uses resolved Concept identity. |
| **Resolution** | Mapping an Alias or canonical representation to its Concept through the vocabulary owner's API. |
| **Shared Infrastructure Module** | A module used by at least two feature modules that provides only mechanical infrastructure and contains no domain logic. |
| **Solution Charter** | The single approved artifact defining a solution's problem, vision, primary supported decision, measurable outcomes, scope boundary, and System Constraints. It governs purpose; it does not define feature behavior. |
| **Solution Framing** | Optional, non-authoritative exploration of a solution's problem, vision, candidate outcomes, scope, and candidate constraints. It may identify open architecture concerns but cannot approve solution meaning or decide architecture. |
| **Specification Package** | The Intent, Contract, and Event Schema considered together by the specification-approval adapter. Decision semantics come only from the selected doctrine. |
| **System Constraint** | A Charter-owned obligation applying across features or solution-wide, each with a verification route. The broader category: cross-cutting quality requirements are one type, alongside regulatory, interoperability, deployment, and externally imposed technology constraints. |
| **Targeted Refinement** | The smallest effective repair of an implementation that does not satisfy already-approved behavior. It never redefines intended behavior, requirements, or architecture. |
| **Vertical Drift** | Accumulation of domain logic in shared infrastructure, bypassing feature-isolation boundaries. |
| **Vocabulary consumer** | A module that uses vocabulary-defined Concepts through the resolution API without interpreting representations itself. |
| **Vocabulary owner** | The single module that defines Concepts, accepts Aliases, validates the vocabulary, and exposes Resolution. |
