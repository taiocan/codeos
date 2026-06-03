# Pattern: Vocabulary-Centric Architecture

## When This Pattern Applies

Use this pattern when a system contains types, statuses, relationships, or domain
concepts that are defined by a configurable schema or vocabulary — and where multiple
modules must reason about those concepts consistently.

Common contexts:
- Configurable type systems (project schemas, ERP item types, workflow states)
- Permission/role systems with aliases or hierarchy
- I18n: display keys vs. internal identifiers
- Event type registries with versioned type names
- Migration compatibility layers (old type name → new canonical)
- Configurable taxonomies and tagging systems

If none of these apply to your feature, this pattern is not required.

---

## Core Definitions

**Concept**: The semantic identity the system reasons about — independent of any string
representation. Concepts are the unit of domain logic. Two items are the same kind if
they share the same concept, regardless of what string was used to express them.

**Canonical**: The single stable runtime identifier chosen by the vocabulary to name a
concept. Unique and authoritative within the system. Not "just another representation":
there must be exactly one canonical per concept at runtime. If two canonicals can
identify the same concept, the vocabulary is broken. Implementations may represent
canonicals as strings, integers, typed enumerations, or UUIDs — the constraint is
uniqueness and stability, not form.

**Alias**: An alternative input form accepted for compatibility, migration, or user
preference. Aliases are translated to their concept before any domain logic runs. An
alias must never appear in business logic below the resolution boundary.

**Resolution**: Mapping any input form (alias or canonical string) to its concept.
Resolution is owned by one module — the vocabulary owner — and exposed as an API. All
other modules call this API; they never perform resolution themselves.

**Vocabulary owner**: The module that defines concepts, accepts aliases, exposes the
resolution API, and validates the vocabulary. One owner per vocabulary.

**Vocabulary consumer**: Any module that operates on vocabulary-defined concepts by
calling the resolution API. Consumers never inspect aliases, hardcode canonical strings,
or perform their own resolution.

---

## The Two Rules

### Concept Dependency Rule

> Business logic depends on vocabulary-defined concepts. The vocabulary resolution
> layer maps aliases and canonical names to concepts before any domain comparison
> occurs. Comparisons occur on resolved concept identity, not on representations.

### Representation Ban Rule (hard constraint)

> Domain layers must not store, compare, branch on, or pattern-match vocabulary
> representations (canonical strings or aliases). Only concept identity — resolved
> by the vocabulary module — is valid in domain logic.

These rules work together. The Concept Dependency Rule states the goal; the
Representation Ban Rule makes the constraint concrete and detectable.

---

## Architecture

```
User input / stored data
         │  (may be alias or canonical string)
         ▼
  Vocabulary Resolution   ← owned by exactly one module
         │  (output: concept identity)
         ▼
      Concept
         │
    ┌────┼────┐
    ▼    ▼    ▼
 Reports  Search  Analytics  (all domain layers: concept identity only)
```

Only the resolution layer knows aliases exist. Everything downstream reasons about
concepts. This means:

- A new alias for an existing concept requires one change: in the vocabulary
- No domain module needs to be updated when a new alias is added
- Domain logic is insulated from vocabulary evolution

**Vocabulary ownership rule**: Exactly one module owns each vocabulary. All other
modules are consumers. If two modules both "know" about aliases, ownership is unclear
and the pattern is broken.

---

## One Comparison Mechanism

There is one valid comparison mechanism in domain logic: concept equality.

How that equality is computed depends on the resolution strategy chosen — but domain
logic never sees the strategy. All of the following are valid implementation strategies
for concept equality:

- **Resolve-on-use**: `resolve(item_type) == resolve("risk")`
- **Normalize-on-write**: `item.canonical_type == risk_canonical` (stored at ingestion)
- **Concept identifiers**: `item.concept_id == risk_concept_id`

The domain layer sees one thing: concept identity. It does not know which strategy the
vocabulary module chose. This is the key abstraction.

**Uniformity invariant**: A single feature must choose exactly one resolution strategy
and apply it uniformly across all comparison sites. Mixing strategies within a feature
(e.g., normalize-on-write in one function, resolve-on-use in another) creates
partial-implementation bugs that are harder to detect than the original leak.

---

## Normalization Strategies

### Normalize on read (resolve-on-use)

Store the original representation; resolve at each comparison site.

```
Input: {type: "hazard"}                (stored as-is)
Query: resolve("hazard") == resolve("risk")   → true (both → Risk concept)
```

**Pros**: flexible vocabulary evolution; aliases can change without data migration.

**Cons**: every comparison site is a potential concept leak. If resolution is skipped
at even one site, that site becomes a representation-dependent bug.

**R8 failure mode**: resolution existed and was available; one comparison site called
`resolve(item_type) == Some("risk")` instead of
`resolve(item_type) == resolve(schema, "risk")`. The resolution call on the right side
was missing. All items whose concept was `Risk` (canonical) failed to match.

### Normalize on write

Resolve at ingestion; store concept identity.

```
Input:  {type: "hazard"}
Stored: {type: "Risk"}                 (canonical at ingestion)
Query:  item.type == risk_canonical    → simple string equality
```

**Pros**: simpler domain logic; no per-query resolution overhead.

**Cons**: vocabulary migrations require data backfill; the ingestion layer must not
skip resolution (or items will be stored with raw aliases).

### Concept identifiers

Resolve at ingestion; store a stable non-string identity.

```
Input:  {type: "hazard"}
Stored: {concept_id: 42}              (stable integer/UUID)
Query:  item.concept_id == risk_id    → integer equality
```

**Pros**: most resilient — canonical string changes don't affect stored data;
canonical can be renamed without backfill.

**Cons**: requires a type-safe concept registry; somewhat more complex ingestion.

---

## The Concept Leak

A **concept leak** is a bug where a vocabulary representation (alias string, assumed
canonical form, or canonical casing) escapes the resolution boundary and appears in
domain logic, bypassing the vocabulary module.

### Recognition patterns

These are all concept leaks:

```
# Alias in domain logic
item_type == "risk"

# Assumed canonical form (wrong casing assumption)
canonical == Some("risk")        # but canonical is "Risk"

# Normalization in domain layer (bypasses vocabulary)
type.to_lowercase() == "risk"

# Hardcoded representation in a filter
items.filter(|i| i.item_type == "stakeholder")
```

All four share the same failure: domain logic is inspecting a representation rather
than operating on resolved concept identity.

### Detection

Ask of every type/status/concept comparison in domain logic:
> "Does this comparison involve a string literal that represents a vocabulary concept?"

If yes, it is a concept leak. Replace it with a vocabulary-resolved equivalent using
the chosen resolution strategy for this feature.

### The architectural question

"Why is a consumer inspecting aliases?"

This question, applied during code review or reconciliation, identifies concept leaks
before they reach production.

---

## Contract and Testing Implications

When a feature has a vocabulary dependency, its contract should include:

1. **Representation Ban invariant** — explicitly stated, not just implied
2. **Uniformity invariant** — resolution strategy declared and applied uniformly
3. **Invariant Falsification row** — at minimum, the canonical-casing fixture:
   define a concept with capitalized canonical (`Risk`) and lowercase alias (`risk`);
   the feature must include items stored as either representation

The canonical-casing fixture is the standard falsifier for concept leaks: it proves
the implementation uses resolution rather than hardcoded string comparison.

---

## Example: R8 Concept Leak

### System
- Vocabulary owner: `project_schema` module
- Vocabulary consumer: `report_export` module
- Concepts: Task, Risk, Stakeholder, Milestone, Issue

### The leak
`report_export` used normalize-on-read but missed the resolution on the target side:

```rust
// Wrong — concept leak:
resolve_type(schema, &i.item_type) == Some("risk")
//                                         ^^^^^^ hardcoded representation
```

The left side correctly resolved the item's stored type to its concept. The right side
compared against the string `"risk"` — an alias in the user's vocabulary, not the
canonical `"Risk"`. Items of type `Risk` resolved to `Some("Risk")`, which did not
equal `Some("risk")`.

### The fix

```rust
// Correct — full concept equality:
let target = resolve_type(schema, "risk");   // resolves "risk" alias → "Risk" concept
resolve_type(schema, &i.item_type) == target
```

Both sides go through the same resolution API. The comparison is concept-to-concept,
not representation-to-representation.

### The lesson
The normalize-on-read strategy requires resolution at every comparison site — including
the target. A single site that skips resolution is a concept leak.

---

## Reference

- `.codeos/terminology.md` — vocabulary architecture term definitions
- `.codeos/prompts/02-contract.md` — Vocabulary Dependency section
- `.codeos/prompts/04-implement.md` — Representation Ban implementation constraint
- `.codeos/templates/contract.md` — Vocabulary Dependency contract section template
