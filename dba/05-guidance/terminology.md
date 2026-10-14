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
| **Actor class** | Who may execute a reported next action: `AUTO` (a deterministic CodeOS command may run the mechanical step without human initiation), `ASSISTED` (an agent or tool may perform bounded work), or `HUMAN` (human judgment or authority is required). It grants no approval, acceptance, or decision authority; a reviewer's judgment stays advisory and F8 and F9 stay human. `AUTO` does not imply that CodeOS can run the action now; only an action that also carries a command has a known complete invocation. |
| **Alias** | An alternative input form accepted for compatibility, migration, or user preference and resolved before domain logic runs. |
| **Architecture Scope** | The single artifact for one project-level architectural scope. Its filename identifies the scope; it owns feature membership, structural decisions, and recorded human approval and cannot invent or alter behavior. |
| **Architecture Impact Report** | A deterministic, non-authoritative comparison of accepted and proposed architecture that identifies changed feature-scoped architecture identities, affected features, and unresolved ambiguity before human approval. |
| **Artifact Governance** | Whether an artifact type carries approval-bearing frontmatter and full process weight, as configured project-wide in `codeos.yaml`. Contrast with Decision Governance, which it never suspends. |
| **Artifact Summary** | The Summary block required immediately after an artifact's frontmatter, previewing its key messages ahead of the full content. |
| **Attention class** | The presentation classification of an attention item in `codeos-workflow status`: `BLOCKER` (an unmet condition prevents progression, including an actionable prerequisite), `STALE` (a previously valid accepted state is no longer current), `UNRESOLVED` (a required human disposition does not yet exist), or `WARNING` (important information that does not block). It never replaces an underlying workflow state. |
| **Behavioral Contract** | Observable truths derived from Intent, normally expressed as Given/When/Then conditions. |
| **Behavioral Event** | A runtime event representing a verified outcome. |
| **Behavioral invariant** | A condition that must remain true in externally observable behavior, independent of internal implementation structure. |
| **Accepted Snapshot** | The immutable F9 identity of a feature's completed lifecycle, governed inputs, judged implementation, and applicable Candidate Bundle; later state never rewrites it. |
| **Canonical** | The single stable runtime identifier that names a Concept within a vocabulary. |
| **Checkpoint** | An ordered point in a governed workflow with a defined entry condition, required evidence, required human decisions, exit condition, and valid return routes. Its state is derived from canonical evidence as `PASS`, `BLOCKED`, or `WAITING`; a later checkpoint is never treated as satisfied while a required earlier one is unresolved. |
| **Candidate Assessment** | The one valid semantic assessment permitted for one immutable candidate identity, resulting in `BLOCKED` or `PASSED`; readiness and structurally invalid executions are not semantic assessments. |
| **Candidate Bundle** | The immutable content-addressed set of governed inputs, dependency-scoped implementation identities, verification records, and selected production/runtime evidence that F7, F8, and F9 assess as one candidate. |
| **Codeos Mechanic** | A fixed delivery, verification, or communication obligation named by the Codeos Mechanics policy; a project's configuration may display it but not disable it. |
| **DBA configuration** | An immutable, approved composition selecting compatible doctrine, policies, prompts, and tool contracts. Its identity and number do not imply a doctrine version. |
| **Legacy DBA-7 cutover compatibility baseline** | The DBA-7 composition applied by a DBA-8-or-later-capable toolkit to a schema-less project for compatibility; it is not evidence that the project historically adopted DBA-7. Changing such a project's toolkit mount is a controlled, human-authorized compatibility action. |
| **Concept** | The semantic identity used by domain logic, independent of its string or storage representation. |
| **Current Validity** | The derived fact that an Accepted Snapshot still applies to the current governed meaning and its recorded product/dependency surface; it is distinct from historical acceptance. |
| **Concept Dependency Rule** | Domain logic compares resolved Concept identity rather than vocabulary representations. |
| **Concept leak** | A defect in which a vocabulary representation escapes the resolution boundary into domain logic. |
| **Consequential conformance repair** | A change that restores already-approved behavior where an incorrect repair could have consequential effects; it is not a new semantic decision. |
| **Correlation ID** | The UUID linking all runtime events from one feature execution chain. |
| **Decision Governance** | The doctrine-level human-control guarantees — consequential architecture approval, canonical terminology consistency, and the like — that remain active regardless of any artifact type's configured governance state. |
| **Decision receipt** | An append-only record that a governed-workflow decision occurred and which governed inputs it was bound to at the time. It is evidence of workflow progression only — never a governing artifact, never overriding the Charter, a Specification Package, an Architecture Scope, or any human-control boundary — and stops establishing its checkpoint once any bound input changes. Only the closed set of irreducible human or authorized-agent decisions gets one. |
| **Declarative Behavioral Architecture (DBA)** | The Codeos model, also called Intent-Driven System (IDS), that carries human intent through behavioral contracts, implementation, runtime evidence, reconciliation, replay, and targeted refinement. |
| **Development Evidence** | Evidence produced before acceptance to prove the candidate implementation. Contrast with Operational Observation. |
| **Early Development Preview** | An unverified, clearly labeled preview shown to the human after basic integration smoke and before full verification, for a direction and UX check only; feedback that would change approved behavior returns to the Specification Package rather than being applied silently. |
| **Engineering Health** | The read-only per-domain result `codeos-workflow status` derives from existing CodeOS authority, each with its basis. Every domain has a closed status set; an absence status (`UNOBSERVED`, `NOT_CONFIGURED`, `NOT_APPLICABLE`, `UNRESOLVED`) reports missing basis and never becomes `PASS`. It is not a score and carries no workflow authority. |
| **Event Spine** | The complete ordered set of events a feature is permitted to emit. |
| **External assessment** | Machine-generated findings supplied by a model Codeos did not invoke. Advisory evidence for human review; it never satisfies a required review round. Contrast with a review, which Codeos obtains itself. |
| **Evidence source** | The strongest direct basis for a reconciliation result: `runtime`, `test`, `static`, or `none`. It is not a confidence or quality score. |
| **Evidence-insufficient finding** | An F7 blocker stating that available evidence does not prove an approved claim while alleging no defect in implementation, approved meaning, architecture, safety, integrity, or another governed input. |
| **Evidence Repair Loop** | The F7 lifecycle in which named evidence-insufficient findings may be reassessed against materially new canonical evidence while implementation and governed inputs remain unchanged, without consuming ordinary review rounds. |
| **Reconciliation status** | One of `ALIGNED`, `GAP`, `MISMATCH`, or `MISSING`. The row note explains the specific issue; status names do not encode gap subtypes. |
| **External Event** | A runtime event representing a side effect on a system outside the governed boundary. |
| **Failure Event** | A runtime event representing a classified error condition. |
| **Feature ID** | The stable `F-####` identity assigned when an Intent is created. It is never reused; partial Specification Packages remain valid identities. |
| **Feature Impact Accounting** | A feature's recorded changed-or-unchanged, with-reason account of its effect on each Platform Baseline tier, replacing any requirement to touch every tier regardless of need. |
| **Full coverage** | `FULL_COVERAGE`: every input in a review's required input set is inspectable in the packet. Material outside the required input set does not lower coverage. |
| **Governed workflow** | One of the three named workflows enclosing the nine-stage lifecycle — Solution Bootstrap, Feature Development, Operation & Learning — each an ordered set of checkpoints whose progression is mechanically verified against real evidence. A governed workflow adds no lifecycle stage, approval, or source of product authority. |
| **Implementation Profile** | An approved project-level statement of preferred implementation language and scope that implementation must consult. |
| **Intent** | Why a feature exists, expressed as actor and desired outcome without implementation details. |
| **Learning Register** | An optional queue of material, unresolved post-acceptance observations that could change governed engineering truth. It is evidence, never authority, and is not an operational log. |
| **Module Design Note** | Optional descriptive documentation of how an implementation module currently works; its normative rules are defined by the doctrine's Structural Ownership section. |
| **NORMAL** | The default Codeos self-development classification for work that does not reasonably risk an approved consequential boundary. |
| **Normalize-on-read** | A vocabulary strategy that stores the original representation and resolves it at every comparison site. |
| **Normalize-on-write** | A vocabulary strategy that resolves input at ingestion and stores Concept identity. |
| **Observational Event** | A runtime event representing a raw observed fact. |
| **Observed Surface** | The identity of what a feature's browser verification observes of the running product, in deterministic provider mode. |
| **Operational Observation** | A fact arising after acceptance, from real system use, that may affect governed engineering truth. It is evidence and cannot change approved behavior by itself. |
| **Post-Acceptance Dependency Revalidation** | The single bounded operation that determines whether a historically accepted feature still satisfies unchanged approved meaning after a mechanically later dependency or integration change invalidates its implementation-bound evidence. It is not another acceptance or Stage-8 review cycle. |
| **Outcome** | A measurable result the solution must create, identified as `O-#` in the Solution Charter. An outcome is not a feature. |
| **Oversimplification Risk Note** | The standing caveat accompanying an Artifact Summary, noting that the summary can omit nuance the full artifact carries. |
| **Platform Baseline** | The default set of architectural tiers — PostgreSQL persistence, a Rust backend, a Svelte GUI, and Docker runtime — every solution resolves unless the Solution Charter records why a tier does not apply. |
| **Product Journey Frontier** | The furthest point on the Charter's primary product journey established by a valid acceptance receipt for a journey-advancing Specification Package. An approved but unaccepted package may propose, but never establish, a new frontier. |
| **Product Journey Transition** | One step between consecutive primary-journey frontiers. Cross-feature topology is defined by an approved Architecture Scope; runtime verification observes the transition without becoming its authority. |
| **Production-source evidence** | Canonical evidence produced by a project-configured verifier from an applicable real-world source, bound to the current reality-dependent claims, governed inputs, and implementation. Appending it does not select it for progression. |
| **Reality-dependent claim** | A governed user-visible claim whose usefulness or truth depends on facts outside the software's controlled state. Fixtures may prove its software mechanics but not its external-world validity, currency, accuracy, or readiness. |
| **Semantic revision** | The shared positive integer on an approved Intent, Contract, and Event Schema that changes exactly once for a governed semantic specification change and scopes an ordinary Stage 8 review cycle. Cosmetic, implementation, evidence, and configuration changes do not alter it. |
| **Journey Attempt** | One execution of the canonical product journey whose per-transition runtime observations are retained as verification evidence. A failed extension does not invalidate a successfully re-proven accepted prefix. |
| **Operational Readiness** | A time- and environment-bound observation that an external provider is currently usable, including credentials, quota, reachability, and a successful live request. It is distinct from deterministic behavioral acceptance through the real provider adapter. |
| **Protected invariant** | The condition identified before a PROTECTED semantic change that must remain true after implementation. |
| **PROTECTED semantic change** | An intentional Codeos change that could alter approved behavior or authority, project architecture, persistent compatibility, safety or integrity, or human-control boundaries. |
| **Quality Requirement** | A governed statement of how well the solution must behave, declared with a verification method. Feature-specific ones belong to the Behavioral Contract; cross-cutting ones are System Constraints. Architecture may respond to them but never originates them. |
| **Reconciliation Review** | A structural comparison of governed artifacts for gaps, mismatches, and missing evidence. |
| **Re-entry Rule** | Post-acceptance learning returns to the earliest governed authority whose truth must change. |
| **Reader Model** | The declared reader-oriented progression (Stable Topic, Known-to-New, Whole Before Parts, or Preview Then Traverse) a substantial narrative artifact states it uses, making its application inspectable rather than only instructed. |
| **Replay Verification** | Confirmation that a runtime event log conforms to its approved schema and contract sequence. |
| **Review re-entry** | A single exceptional Codex-backed review after the ordinary R1-R3 budget is exhausted, enabled and bounded by an explicit human decision without resetting or extending the ordinary round sequence. |
| **Review applicability** | The mechanically derived ability of a structurally valid reviewer conclusion to govern progression for the exact inputs it assessed; it is distinct from execution validity and semantic conclusion. |
| **Review Readiness Gate** | The single deterministic check every reviewer invocation must pass. Failure is `REVIEW_NOT_READY` and consumes no review capacity. |
| **Review Input Manifest** | The deterministic per-review record of the complete input identity graph. It is the single source for packet construction and review lineage validation. |
| **Reviewed-state snapshot** | The content-addressed identity of every input a review assessed, including uncommitted and untracked files. Later deltas compare against it, not against a Git commit. |
| **`REVIEW_NOT_READY`** | The result of a failed Review Readiness Gate. The reviewer is not invoked, no capacity is consumed, and no governance state changes. |
| **`REVIEW_EXECUTION_INVALID`** | The result of a reviewer response that fails structural validation. It is preserved, carries no semantic conclusion, consumes no capacity, and may be re-executed with identical inputs. |
| **Invalid Review Input Lineage** | A review-applicability state proving from immutable evidence that a mandatory governed input was contradictory before reviewer execution. The review remains immutable and consumed, but contributes no progression authority. |
| **Runtime Verification Profile** | Optional project configuration naming only the runtime locations that mechanical workflow checks cannot safely derive. It describes where an already-declared project topology is verified; it neither defines that topology nor changes checkpoint semantics. |
| **Representation Ban Rule** | Domain logic must not store, compare, branch on, or pattern-match vocabulary representations; it uses resolved Concept identity. |
| **Resolution** | Mapping an Alias or canonical representation to its Concept through the vocabulary owner's API. |
| **Shared Infrastructure Module** | A module used by at least two feature modules that provides only mechanical infrastructure and contains no domain logic. |
| **Solution Charter** | The single approved artifact defining a solution's problem, vision, primary supported decision, measurable outcomes, scope boundary, and System Constraints. It governs purpose; it does not define feature behavior. |
| **Solution Framing** | Optional, non-authoritative exploration of a solution's problem, vision, candidate outcomes, scope, and candidate constraints. It may identify open architecture concerns but cannot approve solution meaning or decide architecture. |
| **Specification Package** | The Intent, Contract, and Event Schema considered together by the specification-approval adapter. Decision semantics come only from the selected doctrine. |
| **System Constraint** | A Charter-owned obligation applying across features or solution-wide, each with a verification route. The broader category: cross-cutting quality requirements are one type, alongside regulatory, interoperability, deployment, and externally imposed technology constraints. |
| **Terminal Review Closure** | One human-authorized, repair-scoped semantic assessment that may establish closure of the finite unresolved conditions from a consumed terminal Stage 8 review. A structurally invalid execution is not that assessment. It is not another review round and cannot reopen unrelated product meaning or architecture. |
| **Assessability Correction** | A bounded evidence-representation correction used only for the single bridge retry of a structurally incomplete Terminal Review Closure. It cites an exact evidence deficiency exposed by the failed attempt and makes the same authorized conditions inspectable without changing implementation, governed meaning, architecture, decision scope, or semantic review capacity. |
| **Assessment Input Correction** | The single bounded operation that corrects objectively wrong or omitted inputs which already existed before a consumed lineage-replacement assessment, without changing implementation, governed meaning, architecture, F6 selection, review scope, or semantic review capacity. |
| **Targeted Refinement** | The smallest effective repair of an implementation that does not satisfy already-approved behavior. It never redefines intended behavior, requirements, or architecture. |
| **Verification record** | An append-only record that a mechanical verification actually executed and passed against a specific state, written by the workflow checker's `check` command and bound to the governed inputs and implementation state it ran against. It asserts only that the verification ran and passed — never that the behavior is adequate — and goes stale the moment any bound input changes. Distinct from a decision receipt, which records a human or process decision, not a mechanical result. |
| **Vertical Drift** | Accumulation of domain logic in shared infrastructure, bypassing feature-isolation boundaries. |
| **Vocabulary consumer** | A module that uses vocabulary-defined Concepts through the resolution API without interpreting representations itself. |
| **Vocabulary owner** | The single module that defines Concepts, accepts Aliases, validates the vocabulary, and exposes Resolution. |
