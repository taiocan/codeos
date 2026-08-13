````markdown
# agent.md

## Purpose

`intent.md` defines the semantic purpose of a system.

It exists to support capability discovery by describing:
- what problem the system solves,
- which actors achieve which meaningful outcomes,
- which stable guarantees always hold.

Intent is not:
- a requirements document,
- a feature list,
- an architecture plan,
- an implementation specification,
- a workflow definition,
- or a delivery roadmap.

Intent should remain stable even if implementation changes significantly.

---

# Core principles

Keep the document:
- short
- stable
- implementation-independent

Use:
- simple language
- behavioral wording
- semantic clarity

Prefer:
- concise statements
- observable system purpose
- human-readable semantics

Remove:
- buzzwords
- enterprise language
- speculative design
- overformalization
- ideology
- marketing language
- subjective adjectives
- unmeasurable aspirations

Avoid:
- APIs
- architecture
- databases
- frameworks
- implementation details
- feature decomposition
- infrastructure
- observability mechanics

---

# Required structure

```markdown
# Intent

<System name> exists to let <actor(s)> <achieve meaningful outcome>.

Specifically:
- <Actor> can <outcome-oriented ability>
- <Actor> can <outcome-oriented ability>

## Stable guarantees

- <invariant, enforceable guarantee>
- <invariant, enforceable guarantee>
```

---

# Semantic rules

## 1. Describe outcomes, not mechanisms

State what becomes true, not how it happens.

### Good
- Developer can recover previous state

### Bad
- System stores snapshots

---

## 2. Use actor + outcome form

Each statement:
- starts with an actor,
- ends with an outcome.

Actors are external participants or the system as a whole,
not internal components.

### Good
- Developer can define expected behavior

### Bad
- API layer validates requests

---

## 3. Keep intent implementation-independent

Avoid implementation details:
- APIs
- databases
- frameworks
- protocols
- infrastructure
- file formats

### Forbidden
- REST endpoint
- SQL transaction
- JSON payload

---

## 4. Avoid observability mechanics

Describe measurable behavior,
not logs, metrics, events, or dashboards.

### Good
- Changes apply atomically

### Bad
- System emits events

---

## 5. Avoid feature decomposition

Do not enumerate:
- workflows
- scenarios
- tasks
- feature trees
- UI states

### Forbidden
- Feature: CreateProject
- Step 1: Upload file

---

## 6. Guarantees must be enforceable

Guarantees should be:
- testable
- measurable
- stable

### Good
- Constraints are enforced before modification

### Bad
- System is highly reliable

---

## 7. Keep intent compact

A valid `intent.md` contains:
- one purpose statement,
- a few actor-outcome statements,
- a few stable guarantees.

If it expands into architecture, workflows,
or implementation plans, it is no longer intent.

---

# Example

```markdown
# Intent

Codeos exists to let developers turn behavior descriptions
into working code without probabilistic drift.

Specifically:
- Developer can define behavior using plain language
- Developer can trigger executable validation from intent
- System can converge toward compliant implementation safely

## Stable guarantees

- Changes are applied atomically
- Constraints are enforced before modification
- Execution stops after bounded retries
```

---

# Evaluation heuristic

A valid `intent.md` answers:

- Who achieves what meaningful outcome?
- Which stable guarantees always hold?

It does not answer:

- Which technologies are used?
- Which components exist?
- Which APIs, files, or schemas exist?
- How observability works?
- How workflows execute?
- How features are decomposed?
- How the system is implemented?
````
