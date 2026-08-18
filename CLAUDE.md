# Codeos Self-Development

## Purpose

This file governs development of the Codeos toolkit itself.

Downstream DBA behavior is governed by `dba-system.md` and its active components. Codeos toolkit
development does not run the downstream nine-stage DBA process.

Self-development governance protects consequential engineering decisions without governing
ordinary implementation work.

## Minimum Complexity

Minimum complexity is a primary constraint for Codeos design and communication.

Use the smallest solution, change, artifact, and response that satisfies the current need. Prefer
deletion, reuse, and one clear path over new mechanisms, variants, or future-proofing.

Do not obtain simplicity by changing exact meaning, names or literals, normative strength,
quantities, or unresolved decisions.

Remove a duplicated fact only when the duplicate is a second manually maintained authority. A check
that deliberately restates a list to detect unauthorized additions or silent removals is an
independent guard, and removing it reduces complexity only in appearance.

By default:

- lead with one recommendation, decision, or result;
- include only information needed for the current decision or action;
- use no more than three short sections unless required for correctness;
- do not enumerate hypothetical variants, future scope, edge cases, or alternative designs unless
  they materially affect the current decision;
- consolidate related review findings and omit routine PASS narration;
- omit optional artifact sections when they add no durable value.

Add complexity only to prevent a specific correctness, compatibility, safety, integrity, or
human-control failure. State that reason when it is not obvious.

When several approaches satisfy the requirement, choose the least complex one.

## Authority

Explicit human decisions are authoritative. Repository artifacts on disk are authoritative over
remembered or previously reported content. Unresolved conflicts MUST be surfaced rather than
silently resolved.

For downstream DBA rules, normative ownership follows this model:

```text
DOCTRINE  fundamental DBA guarantees
POLICY    conditional governance mechanisms
PROMPT    agent execution instructions
TOOL      deterministic enforcement or execution
```

A normative rule SHOULD have one authoritative owner. This file owns Codeos self-development
governance; it does not duplicate downstream DBA rules.

## Starting Work

Read this file and the artifacts relevant to the requested work.

Continue relevant work already in progress when it exists. Otherwise work from the new request.
Do not require a general repository-status review or human confirmation merely to begin work.

## Self-Development Process

### NORMAL

NORMAL is the default. Use it when the change does not reasonably risk changing an approved
consequential boundary.

```text
edit
-> inspect diff
-> run relevant checks
-> review if useful
-> commit when requested
```

No change record, fixed review round, reconciliation artifact, intermediate human gate, or routine
review log is required.

### PROTECTED Semantic Change

Use PROTECTED when an incorrect semantic change could alter:

```text
approved behavior or authority
project-level architecture
persistent compatibility
safety or integrity
human-control boundaries
```

A change to fundamental DBA doctrine is PROTECTED.

Before implementation, state what consequential semantics change, what must remain true, and how
the result will be verified. Name the affected component only when doing so adds useful precision.
The human MUST approve the consequential semantic change before implementation.

After implementation, verify the protected invariant and obtain human acceptance. Independent
review is used only when it materially reduces an identified consequential risk. Reviewer
conclusions remain advisory.

### Consequential Conformance Repair

A repair that only restores conformance with already-approved behavior is not a new semantic
decision. Cite the existing authority, make the repair, and use strong verification when an
incorrect repair could have consequential effects. Do not require a new behavioral approval merely
to restore existing authority.

## Scope and Escalation

Govern semantic changes, not file edits. One coherent change MAY affect several components and many
files without becoming several governed changes. Implementation-only work that preserves governed
semantics needs no component declaration.

NORMAL work becomes PROTECTED if implementation reveals that a consequential semantic boundary
must change. Private implementation choices that preserve approved boundaries do not cause
escalation.

## Evidence and Durable Memory

Git, diffs, tests, and targeted checks are the default evidence. Persist additional evidence only
when it is needed to explain or protect a consequential decision.

Do not persist routine review rounds, repeated PASS statements, implementation narration, or
information recoverable from Git unless it has durable engineering value. The architecture journal
is optional durable memory, not a required process artifact.

If operational state is stored, each fact MUST have one authoritative representation. Other views
SHOULD derive it mechanically rather than require manual synchronization.

## Minimum Traceability

Git is the authoritative history of completed Codeos changes.

Each commit MUST explain the engineering purpose of the change, not merely list modified files.

When work already has a backlog, issue, or decision identifier, reference it in the commit message.
Do not create an identifier solely to satisfy traceability.

For a PROTECTED semantic change, the accepted semantics MUST appear in the authoritative governing
artifact. The commit MUST make the relationship between that decision and its implementation clear.

Verification SHOULD be reproducible from committed tests or checks. When it is not self-evident,
summarize the verification in the commit message or pull-request description.

The active backlog tracks unfinished work that must survive the current task or session. It is not
required for every change and is not implementation authority.

Do not create separate change records, traceability ledgers, status copies, or journal entries when
the governing artifact and Git already provide sufficient traceability.

## Terminology

`dba/05-guidance/terminology.md` is the canonical glossary for recurring CodeOS-specific terms
whose ambiguity could affect behavior, architecture, authority, lifecycle, or responsibility.
Downstream projects use `.codeos/00-project/terminology.md` only when they have shared specialized
project meanings. Ordinary technical vocabulary does not require terminology governance.

Before introducing or materially relying on such a term, check the glossary. If the term is absent,
ambiguous, or used differently, notify the human and propose one definition within the current
interaction. After acceptance, add or update the definition as part of the same change; do not
create a separate terminology workflow or history.

When a change relies on a governed term, verification MUST confirm that its definition exists,
affected uses are consistent, and no conflicting meaning was introduced. If two governed concepts
would otherwise share a term, use distinct names.

## Boundaries

Do not change downstream DBA doctrine as an unintended side effect.

Do not let reviewer output become approval authority.

Do not create governance artifacts merely to prove that a process step occurred.

Do not duplicate authority or operational state across files.

Detailed review procedures, architecture procedures, writing rules, dependency checks, CLI
instructions, file layouts, and other operating mechanisms belong in their owning policy, prompt,
pattern, or tool when they are actually needed.
