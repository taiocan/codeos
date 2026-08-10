# Stage 4: Constrained Implementation

## Your Role

You implement ONLY what is specified by the three approved artifacts.
You are not a creative designer at this stage. You are a **constrained satisfier**.

## Preconditions

You MUST have ALL THREE approved before starting. Verify each:

- [ ] `intents/[feature_id].md` — `status: APPROVED`
- [ ] `contracts/[feature_id]_contract.md` — `status: APPROVED`
- [ ] `events/[feature_id]_schema.md` — `status: APPROVED`

If any is missing or not approved — **STOP** and request it.
Implementation without all three is a DBA violation.

**Cohort eligibility check (if `features/registry.yaml` exists):** read this feature's registry
entry.
- If `architecture_cohort` is absent or `null` — no further check; proceed.
- If it names a cohort: find that cohort's `architecture_cohorts` entry. If `status` is not
  `approved` — including `declared`, `gate-in-progress`, or the compatibility state
  `baseline-approved`, all three of which block Stage 4 identically — or the entry (or either of
  its `baseline_version`/`logical_design_version`) is missing entirely — **STOP**. This feature is
  not eligible for Stage 4 yet; report the specific gap and point to
  `.codeos/prompts/03b-architecture-synthesis.md` rather than proceeding.
- If `approved`: verify **both** referenced versions — `baseline_version` equals
  `architecture/core-baseline.md`'s current `Baseline version` field exactly, **and**
  `logical_design_version` equals `architecture/cohort-logical-design.md`'s current `Logical design
  version` field exactly. A value matching only a file under `architecture/history/` is **stale,
  not valid** for either — treat it the same as a non-`approved` status and **STOP**; historical
  files are a provenance record for already-completed Stage 4 work, never a valid reference for
  entering Stage 4 now. See the `architecture_synthesis_policy` component selected by
  `.codeos/dba-system.md` →
  "Verifying a `baseline_version` or `logical_design_version` reference."

**Implementation Profile consultation (if `architecture/implementation-profile.yaml` exists):**
- Absent, or `status: proposed` at the current path — no profile is binding; proceed with no
  language requirement. (A pending `architecture/proposals/` replacement is never consulted
  here.)
- `status: approved` — verify `profile_version` matches the file at the current path exactly (not
  a `proposals/` or `history/` file — same current-only rule as the cohort baseline check above).
  Resolve whether this feature is in scope via `applies_to.scope`:
  - `all` — in scope.
  - `feature_ids` — in scope iff this feature's id is listed.
  - `cohort_ids` — in scope iff this feature's `architecture_cohort` (from `features/registry.yaml`)
    is listed.
  - Not in scope — no requirement applies; proceed.
  - In scope — check for a matching exception. A feature-level exception overrides a matching
    cohort-level exception (more specific). Multiple matching exceptions at the *same*
    specificity that disagree — **STOP**, the profile is invalid for this feature. Otherwise: the
    matched exception's `language` is binding if one applies; else `primary_language` is binding.
    If the binding language has an applicable Codeos pattern (e.g. `rust` →
    `.codeos/patterns/rust-project-structure.md`), consult it — advisory only, never overriding an
    approved Architecture Baseline or another project-specific decision.
- **Profile–Baseline consistency check:** if this feature also has an approved Architecture
  Baseline whose authoritative decisions specify a language for this feature that conflicts with
  the profile's resolution above, and no exception reconciles it — **STOP**. Ineligible,
  unreconciled contradiction; neither artifact is silently preferred. See the
  `implementation_profile_policy` component selected by `.codeos/dba-system.md` → "Profile–Architecture Baseline
  consistency."

**Controlled Plain English check (if `architecture/controlled-plain-english.yaml` exists):** read
its `status` per the Optional Mechanism Status Convention's four-outcome table
(`.codeos/templates/conventions.md`). Absent or `disabled` → proceed unaffected. `enabled` → read
`.codeos/patterns/controlled-plain-english.md`; if missing/unreadable, **STOP** and report a
pattern-access error; otherwise apply **factual reporting** (not Layer B) to this stage's Review
Package free text and Known Tensions disclosure — this is an implementation-evidence report, not a
specification (Layer C1 always applies regardless of the toggle). Malformed status file → **STOP**
and report a configuration error.

## What You Receive

- Approved intent: `intents/[feature_id].md`
- Approved contract: `contracts/[feature_id]_contract.md`
- Approved event schema: `events/[feature_id]_schema.md`

## What You Produce

Implementation code in `modules/`, satisfying all contract clauses and emitting all required events.

## Structural Orientation (before writing any code)

Identify any critical hubs, high-risk modules, or dependency chokepoints affected
by this change:

- If `docs/codebase-digest.md` exists: read it now and note which listed functions
  this implementation will touch.
- If no digest exists: derive manually — scan `modules/` and identify any function
  that appears to coordinate multiple downstream calls or is called from many sites.

This is a *thinking step*, not an artifact requirement. The goal is to know where
blast radius is concentrated before the first line of code is written.

## Implementation Constraints (non-negotiable)

**Every contract clause must be satisfied.**
Map each clause to specific code. If a clause cannot be satisfied without adding abstractions not in the contract, flag it — do NOT silently add them.

**Every event in the schema must be emitted at the correct point.**
The first thing you implement is correlation ID propagation and event emission infrastructure. Events are not optional.

**No additional abstractions.**
If the contract doesn't require it, don't build it. No "helper" classes, "utility" layers, or "service" abstractions beyond what's needed to satisfy the contracts.

**No additional events.**
You may ONLY emit events listed in the approved schema. If you discover you need a new event, stop and request a schema update.

**No undeclared runtime artifacts.**
You must not create or write to any file or directory other than `events/runtime_events.jsonl` unless the contract's Runtime Artifacts section explicitly names it. If state persistence is needed and not listed in the contract, stop and raise it for contract amendment — do NOT silently create files.

**No speculative error handling — and the Contract-to-Implementation Failure Boundary.**
Only handle failure modes explicitly listed in the contract's Failure Classifications. Other errors propagate as uncaught exceptions (or, in a language with richer error types, as the internal error type itself — never invented as a classified failure). Two boundaries stay distinct: the *behavioral boundary* (Failure Classifications the contract approves) and the *technical API boundary* (internal/storage/serialization/I/O errors, which may be as rich as the implementation needs). Two separate approvals gate an emitted failure event, not one blended condition: the classification must be named in the approved contract, and, independently, the specific event produced from it must be present in the approved event schema — the schema authorizes event types, not classification names; a contract-approved classification alone does not authorize emitting anything. Every mapping from an internal error to an approved classification is explicit and reviewable — document it in the Failure Mapping Table below, not left implicit in code. See the `doctrine` component selected by `.codeos/dba-system.md` → "Contract-to-Implementation Failure Boundary" and, for Rust, `.codeos/patterns/rust-project-structure.md` → "Error Boundary Convention."

**Implementation must be deterministic.**
No hidden randomness, no time-based branching not reflected in contracts.

**Correlation IDs propagate through all operations.**
This is the first thing you wire up. Every log line, every emitted event, carries the correlation ID from the feature invocation.

**If this feature consumes vocabulary: apply the Representation Ban.**

Domain logic in this feature must not store, compare, branch on, or pattern-match
vocabulary representations (aliases or canonical strings). Only concept identity —
resolved by the vocabulary module's API — is valid.

Before implementation begins:
1. Identify every site in domain logic that will operate on vocabulary-defined concepts
2. Choose one resolution strategy (normalize-on-write, normalize-on-read, or concept
   identifiers) and apply it uniformly — mixing strategies within one feature is a violation
3. If you find a string literal representing a vocabulary concept in domain logic,
   replace it with a vocabulary-resolved equivalent before proceeding

The wrong pattern in any strategy: comparing a type or concept against a hardcoded
string literal (`== "risk"`, `== Some("risk")`). The correct pattern: concept equality
via the vocabulary's resolution API applied uniformly on both sides of every comparison.

See: `.codeos/patterns/vocabulary-architecture.md`

## Structure

Place implementation in `modules/[feature_id]/` or follow the existing project module layout.

Emit events to `events/runtime_events.jsonl` as append-only JSONL. Each line is one complete JSON event object.

## Output Format

1. Present the implementation
2. Present a **Contract Satisfaction Table**:

| Contract Clause | Satisfied By | Line/Function | Structural Risk |
|---|---|---|---|
| [clause from contract] | [code location] | [file:line] | [LOW / MEDIUM / HIGH / —] |

Structural Risk levels (only populate when a Critical Hub or God Function is touched):
- **LOW** — renaming, extracting helpers, testability changes; behavior visible outside the module is unchanged
- **MEDIUM** — modifying internal logic; external behavior likely unchanged but must be verified
- **HIGH** — behavior visible outside the module may change (callers, emitted events, return values, error modes)
- **—** — no Critical Hub or God Function touched by this clause

3. Present an **Event Emission Table**:

| Event in Schema | Emitted At | Condition |
|---|---|---|
| [EventName] | [file:line] | [when] |

4. Present a **Failure Mapping Table** (see "No speculative error handling" above and
   the `doctrine` component selected by `.codeos/dba-system.md` →
   "Contract-to-Implementation Failure Boundary") — one row per
   approved Failure Classification that this feature's implementation actually maps to:

| Internal Error | Contract Failure Classification | Emitted Event | Mapping Site |
|---|---|---|---|
| [internal error variant/type] | [Failure Classification name from the contract] | [event name, must also be in the approved schema] | [file:line of the mapping] |

If no internal errors map to any approved classification for this feature, state "none" rather
than omitting the table.

5. Present a **Deferral → Resolution Trace** — *only if* this implementation resolved a material
   question that an approved upstream artifact **explicitly deferred**. Most features defer nothing;
   when that is the case, omit this section entirely. Do not write an empty table and do not write
   "none".

   **What counts as an explicit deferral.** A statement in an approved artifact that a specific design
   or behavioral question is deliberately left unresolved *by that artifact* — whatever wording it
   uses. The artifact names the question and says it does not settle it, often indicating where or
   when it would be. Judge this by meaning, never by matching particular phrases: an equivalent
   deferral written in different words counts exactly the same.

   Two things that are **not** deferrals:
   - **Silence.** An artifact that simply never mentions a question has not deferred it. Only an
     affirmative statement of non-resolution counts — otherwise you would owe a record of everything
     the artifacts failed to say.
   - **Implementation freedom.** An artifact that settles the *behavior* while leaving the *technique*
     open has deferred nothing. Choosing a data structure or an error-propagation style resolves no
     deferral; record those under "Key architectural decisions" in the Review Package as before.

   **Materiality.** Record a resolution only when changing it — while preserving the same public
   behavior — would materially affect an invariant, a component's responsibility, the state model,
   data integrity, or future architectural freedom.

| Source Artifact + Deferral | Chosen Resolution | Where Implemented | Final / Interim | Expected Superseder |
|---|---|---|---|---|
| [artifact + the question it left open] | [what you decided] | [file:line, function, or module] | [FINAL / INTERIM] | [for INTERIM: the upstream decision or artifact expected to replace this; otherwise —] |

   The last two columns are the point of the trace: an interim resolution that nothing records as
   interim is indistinguishable later from a settled decision, and the upstream work that should
   retire it is never triggered.

   **This trace is subordinate to the approved artifacts.** It records a choice made under an
   authority the artifact itself granted; it never overrides, reinterprets, or amends anything
   approved. If your resolution appears to conflict with an approved artifact, that conflict must be
   **reconciled** — raise it rather than recording a resolution that contradicts approved text, and
   expect that implementation may not legitimately continue until the artifact is amended through its
   own governance path.

   **If you omit this section and a material deferral was in fact resolved, that is a traceability
   defect** — the Stage 4 review will ask — **not an automatic implementation failure.** The
   implementation may still be correct; the record of how a deferred question got answered is what is
   missing.

6. Present the Review Package using `.codeos/templates/review-package.md` (Stage 4–5 format, inline only):
   - Artifact: `modules/[feature_id]/`
   - Stage purpose: Implement only what the three approved artifacts specify.
   - Files changed: [list all files created or modified]
   - Key architectural decisions: [choices not fully determined by the approved artifacts — e.g., internal data structure, error propagation strategy]
   - What is not covered yet: [explicit list of what stages 5–9 still need to verify]
   - Suggested areas: (1) Are there contract clauses technically satisfied but implemented in a surprising or fragile way? (2) Does the implementation introduce any behavior not traceable to the approved intent, contract, or schema? (3) What is the most likely Stage 7 gap or mismatch, given what was implemented?
   - Known tensions: from schema design decisions or contract boundary cases, or "none"
   - Implementation Profile applied: if an approved profile applied to this feature, state
     `profile_id`, `profile_version`, the resolved language, and any matched exception; otherwise
     state "no profile" or "profile proposed, non-binding."
7. State: **`AWAITING HUMAN APPROVAL TO PROCEED TO STAGE 5`**

**STOP.** Do not write tests until the human explicitly approves the implementation.
