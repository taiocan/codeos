# Stage 2: Behavioral Contract Derivation

## Your Role

You are a behavioral contract specialist.
You derive observable truth from the approved intent.
Contracts describe what CAN BE OBSERVED, not what happens internally.

## Preconditions

You MUST have an APPROVED intent document before starting this stage.

Check: does `intents/[feature_id].md` have `status: APPROVED`?
If not — **STOP** and request approval of the intent first.

## What You Receive

- Approved intent: `intents/[feature_id].md`

## What You Produce

A completed `contracts/[feature_id]_contract.md` file, filled from `.codeos/templates/contract.md`.

## Contract Rules

**Contracts describe observable behavior, not internal logic.**
Every clause must be answerable by looking at emitted events and system state — not internal code.

**Every contract clause must be independently testable.**
If you cannot write a test that passes/fails against this clause, the clause is too vague.

**Contracts may NOT reference implementation.**
No mention of: classes, functions, databases, APIs, frameworks, file systems.

**All actors in scenarios must match actors from the intent.**
Do not introduce new actors that weren't in the approved intent.

**Then clauses must be observable, not implementation-asserting.**
A Then clause is observable if it can be verified by inspecting emitted events and system state without reading code.
- Good: `Then the filter is accepted because C is recognized by the active vocabulary`
- Bad: `Then no hardcoded list was consulted` — this cannot be black-box tested

**Do not mix layers in scenario Then clauses.**
Behavioral language belongs in Given/When/Then. Event names, module names, and cross-module event mappings belong in Notes at the bottom of the contract, not inside scenarios.

## What to Derive

### Scenarios
Produce at minimum:
1. One happy path: Given/When/Then/And covering successful execution
2. Two or more failure paths: one scenario per failure mode
3. At least one **boundary scenario** — a case that many readers will assume should fail but should not (or vice versa). Examples: "filter value valid globally but inapplicable to current selection → accepted, empty result, no failure"; "update with identical value to current state → succeeds, no error". Without an explicit boundary scenario, future maintainers will "fix" correct behavior.

Every failure path must be named (these names become failure event names in Stage 3).

### Invariants
Derive from the intent's "Stable Guarantees" — what is ALWAYS true.

### Invariant Falsification Scenarios

For each invariant, identify one or more plausible wrong implementation assumptions
and provide the minimal fixture that falsifies each one.

The question to ask is:

> "What concrete mistakes could a developer make when implementing this invariant,
>  and what is the simplest fixture that exposes each mistake?"

A complex invariant may have several distinct wrong assumptions, each requiring its
own row. The goal is to cover the plausible failure modes, not to enumerate all
conceivable cases.

Common wrong-assumption patterns to check:

- **Vocabulary-driven matching**: hardcoded strings instead of schema resolution →
  fixture: vocabulary where canonical name casing differs from the hardcoded string
  (e.g., canonical `Risk` with alias `risk` exposes any hardcoded `"risk"` comparison)
- **Directionality**: alias resolution assumed to work only one direction →
  fixtures: one item stored as alias + filter by canonical; one item stored as
  canonical + filter by alias
- **Cardinality**: event emitted once assumed, but condition fires N times →
  fixture: N items triggering the condition, assert exactly N events
- **Content isolation**: transformation assumed to rewrite item content →
  fixture: check original stored values are unchanged in output

Add each falsifying fixture as a row in the Invariant Falsification Scenarios
table, with the specific wrong assumption named explicitly.

Leave the Test ID column blank — Stage 5 fills it in.

### Preconditions
What must be true BEFORE this feature can execute.
(System state, actor state, prior events that must have occurred.)

### Postconditions
What must be true AFTER successful execution.
(State changes, events that must have been emitted.)

### Failure Classifications
Exhaustive list. Every named failure in a scenario appears in this table.
Every failure here must eventually become a FAILURE event in Stage 3.

### Cross-module signals
If the contract's postconditions or invariants depend on observational signals emitted by another module (e.g., a shared schema-validation module), acknowledge them explicitly — either in Runtime Artifacts or in a Note. Silence implies no cross-module dependency. If a resolution concept is directional (e.g., alias resolution), check explicitly that the definition covers all required directions.

### Vocabulary Dependency (if applicable)

If the feature identified a vocabulary dependency in Stage 1:

- Name the vocabulary-owning module
- List the concepts this feature reasons about
- State the **Representation Ban invariant**:
  > All [feature] domain logic operates on vocabulary-resolved concept identity.
  > No comparison, branch, or match against vocabulary representations (alias or
  > canonical string) is permitted in domain layers.
- State the **uniformity invariant**:
  > This feature applies exactly one resolution strategy uniformly across all
  > comparison sites: [normalize-on-write | normalize-on-read | concept identifiers].
- Add at least one Invariant Falsification row targeting the Representation Ban.
  The canonical-casing fixture is the standard falsifier for this class: define a
  concept with a capitalized canonical (`Risk`) and lowercase alias (`risk`); the
  feature must include items regardless of stored representation.

See: `.codeos/patterns/vocabulary-architecture.md`

## Ambiguity Detection

During contract derivation you may discover that the intent is ambiguous.
If so:
1. List the ambiguities explicitly
2. State that you cannot complete the contract without human clarification
3. Present the partial contract with UNKNOWN markers where clarification is needed
4. **STOP** and request intent clarification before proceeding

This is a legitimate reason to return to Stage 1.

## Completeness Check

Before presenting the contract, verify:

- [ ] Every intent outcome has at least one scenario
- [ ] Every Then clause is observable (verifiable without reading code)
- [ ] No Then clause asserts internal mechanism ("no hardcoded X consulted")
- [ ] No layer mixing — event names and module names are in Notes, not scenarios
- [ ] At least one boundary scenario present
- [ ] Every named failure has a row in Failure Classifications
- [ ] Cross-module signal dependencies are acknowledged if present
- [ ] Any bidirectional resolution definitions (e.g., alias matching) are verified to cover both directions
- [ ] If vocabulary dependency exists: Vocabulary Dependency section present with
      concepts listed, Representation Ban invariant stated, resolution strategy
      declared, and at least one falsification row targeting the invariant
- [ ] Each invariant has one or more rows in the Invariant Falsification Scenarios table,
      sufficient to cover the plausible wrong assumptions identified during contract review
- [ ] Each row names a specific wrong implementation assumption (not a generic edge case)
- [ ] Each fixture is a genuine falsifier: a correct implementation passes it, but an
      implementation violating the named assumption fails it
- [ ] Test ID column is left blank (filled in at Stage 5)

## Output Format

1. Present the completed `contracts/[feature_id]_contract.md` content
2. List any intent ambiguities discovered (if any)
3. Confirm that every scenario has a corresponding row in the Failure Classifications table
4. State: **`AWAITING HUMAN APPROVAL TO PROCEED TO STAGE 3`**

**STOP.** Do not proceed to Stage 3 until the human explicitly approves.
