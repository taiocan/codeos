````markdown id="7tq9vk"
# agent.md

## Purpose

`observable-features.md` defines the observable behavior map of the system.

It exists to answer:

```text
What observable transformations exist in this system?
```

It connects:
- intent
- capabilities
- feature specifications
- implementation
- observability

---

# Core Principles

Keep the feature map:
- behavioral
- observable
- runtime-oriented
- event-oriented
- implementation-independent

Avoid:
- architecture diagrams
- service decomposition
- database structure
- workflow engines
- specification hierarchy

---

# Feature Model

Every observable feature represents:

```text
Trigger -> Transformation -> Observable Outcome
```

Example:

```text
Validation request
  -> behavior evaluated
  -> ValidationCompleted
```

---

# Observable Feature Rules

Observable features must be:

| Property | Requirement |
|---|---|
| independently observable | yes |
| independently measurable | yes |
| independently traceable | yes |
| independently evolvable | yes |

If a feature cannot be independently observed,
it is probably:
- implementation detail,
- orchestration logic,
- or internal structure.

---

# Feature Map Structure

Use:

```md
# Observable Features

## Capability: <Capability>

| Feature | Trigger | Transformation | Observable Outcome |
|---|---|---|---|
```

Keep descriptions:
- short,
- behavioral,
- stable.

---

# Naming Rules

## Features

```text
VerbNoun
```

Examples:
- ExecuteValidation
- ApplyRepositoryChange
- RejectInvalidChange

---

## Observable Outcomes

Use stable event language:

```text
<Entity><Action><Outcome>
```

Examples:
- ValidationCompleted
- RepositoryChangeRejected
- ExecutionLimitExceeded

Observable outcomes define:
- runtime vocabulary,
- telemetry alignment,
- traceability language.

---

# Relationship Between Artifacts

| Artifact | Purpose |
|---|---|
| intent.md | why system exists |
| capabilities.md | behavioral abilities |
| observable-features.md | runtime behavior map |
| feature.md | detailed feature guidance |

---

# Module Structure

Organize the system by behavioral module.

```text
modules/

  <module>/

    intent.md
    capabilities.md
    observable-features.md

    features/

      <feature>/

        feature.md
        observable-contract.md
        refinement.md

        examples/
        tests/

    telemetry/
    src/
```

---

# Module Rules

A module represents:
- one behavioral domain,
- one capability grouping,
- one observable topology.

Modules should remain:
- behavior-centered,
- independently evolvable,
- independently observable.

Do not organize modules around:
- databases,
- frameworks,
- transport layers,
- infrastructure.

---

# Feature Directory Rules

Each feature directory represents:
- one observable transformation,
- one measurable outcome,
- one traceable execution boundary.

## feature.md

Defines:
- purpose
- trigger
- transformation
- expected outcomes
- failure conditions

## observable-contract.md

Defines:
- emitted events
- metrics
- logs
- correlation behavior

## refinement.md

Defines:
- behavioral refinement history
- validation discoveries
- operational constraints

---

# Examples And Tests

Examples should demonstrate:
- successful behavior
- rejected behavior
- edge conditions

Tests should validate:
- behavior
- observability
- runtime evidence

---

# Most Important Rule

Observable behavior is more stable than system structure.

Architecture may evolve:
- monolith → distributed
- sync → async
- SQL → event sourcing

But observable outcomes should remain stable.

Example:

```text
ValidationCompleted
```

should survive structural change.

---

# Goal

The observable feature map becomes:

```text
the stable operational language of the system
```

for:
- developers
- AI agents
- observability
- debugging
- refinement
- architecture evolution
````
