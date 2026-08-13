# Stage 4: Constrained Implementation

<!-- DOCTRINE ADAPTER: delivery-entry
Operationalizes the active doctrine's prerequisites for entering implementation. -->

## Your Role

You implement the behavior governed by the approved Specification Package. You may use normal
internal structures, validation, technical errors, logging, and established project patterns when
they do not change approved behavior or another governed boundary.

## Preconditions

You MUST have ALL THREE approved before starting. Verify each:

- [ ] `intents/[feature_id].md` — `status: APPROVED`
- [ ] `contracts/[feature_id]_contract.md` — `status: APPROVED`
- [ ] `events/[feature_id]_schema.md` — `status: APPROVED`

If any is missing or not approved — **STOP** and request it.
Implementation without all three is a DBA violation.

For a DBA-2 package approval, all three artifacts record the same `approved_by` and `approved_at`.
Existing unchanged approvals made before DBA-2 activation remain valid even when their metadata
differs. If any specification artifact changed after its earlier approval, the package is
unapproved until all three are reviewed together and record one new approval decision.

**Architecture eligibility check:** run:

```bash
.codeos/dba/04-tools/reviewer/codeos-review.sh inspect-architecture-scopes --feature [feature_id]
```

If inspection reports malformed metadata or conflicting membership, **STOP**. Apply the selected
`architecture_synthesis_policy` to the reported facts:

- No matching scope — assess whether implementation would settle an unresolved project-level or
  cross-feature structural decision. If no, proceed. If yes or uncertain, **STOP** and use
  `.codeos/dba/03-prompts/workflow/03b-architecture-synthesis.md`.
- One matching draft scope — **STOP** and return to Architecture Synthesis.
- One matching approved scope — proceed under its binding decisions.

The inspector is a deterministic reader, not an approval or architectural-sufficiency authority.
The scope file remains the authority for membership, decisions, and recorded approval.

**Implementation Profile consultation (if `architecture/implementation-profile.yaml` exists):**
- Absent, or `status: proposed` at the current path — no profile is binding; proceed with no
  language requirement. (A pending `architecture/proposals/` replacement is never consulted
  here.)
- `status: approved` — verify `profile_version` matches the file at the current path exactly (not
  a `proposals/` or `history/` file).
  Resolve whether this feature is in scope via `applies_to.scope`:
  - `all` — in scope.
  - `feature_ids` — in scope iff this feature's id is listed.
  - Not in scope — no requirement applies; proceed.
  - In scope — check for a matching feature exception. Multiple matching exceptions that disagree —
    **STOP**, the profile is invalid for this feature. Otherwise, the matched exception's `language`
    is binding if one applies; else `primary_language` is binding.
    If the binding language has an applicable Codeos pattern (e.g. `rust` →
    `.codeos/dba/05-guidance/patterns/rust-project-structure.md`), consult it — advisory only, never overriding an
    applicable approved project architecture or another project-specific decision.
- **Profile–Architecture consistency check:** if applicable approved project architecture specifies
  a language for this feature that conflicts with
  the profile's resolution above, and no exception reconciles it — **STOP**. Ineligible,
  unreconciled contradiction; neither artifact is silently preferred. See the
  `implementation_profile_policy` component selected by `.codeos/dba-system.md` → "Profile–Architecture
  consistency."

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

**Internal implementation freedom.**
Use helpers, types, modules, and established patterns when they are a maintainable way to satisfy
the package. They must not introduce a new governed outcome or architectural commitment.

**No additional events.**
You may ONLY emit events listed in the approved schema. If you discover you need a new event, stop and request a schema update.

**No undeclared governed outcomes.**
Internal implementation files are allowed. New externally meaningful persistence, side effects,
or runtime outputs must be authorized by the Specification Package or applicable architecture.

**Behavioral Failure Boundary.**
Only Contract-listed failures may be exposed as governed behavioral outcomes. Internal errors may
be richer, but they must remain distinguishable and must never be silently mapped to a contractual
outcome. A failure event also requires authorization by the Event Schema. Document every
internal-to-contractual mapping in the Failure Mapping Table below. Apply the selected doctrine
directly and, for Rust, `.codeos/dba/05-guidance/patterns/rust-project-structure.md` → "Error Boundary Convention."

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

See: `.codeos/dba/05-guidance/patterns/vocabulary-architecture.md`

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
   "Behavioral Constraints") — one row per
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

6. Preserve the following delivery evidence for final review (inline; do not create a new artifact):
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
7. Continue directly to `.codeos/dba/03-prompts/workflow/05-tests.md`. Do not request an intermediate approval.
