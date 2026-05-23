````markdown
# agent.md

## Purpose

`features.md` defines observable transformations derived from `capabilities.md`.

Features describe:
- observable behavior,
- state transformation,
- validation boundaries,
- emitted runtime evidence.

Features exist between:
- capabilities (`behavioral domains`)
- implementation (`how behavior is achieved`)

---

# Core Principles

- Prefer observability over premature formalization.
- Keep feature definitions minimal.
- Add rigor only after operational failures.
- Runtime evidence is more trusted than prose guarantees.
- All features must emit structured events.
- All features must support correlation IDs.

---

# Feature Rules

A feature:
- represents one observable transformation,
- produces one meaningful outcome,
- emits operational evidence,
- remains implementation-independent.

A feature does not describe:
- APIs,
- services,
- databases,
- infrastructure,
- workflows,
- runtime topology.

If a definition spans multiple outcomes,
it is probably:
- multiple features,
- or a capability.

---

# Feature Formula

```text
<Feature> transforms <Input> into <Observable Outcome>
```

Examples:
- ExecuteValidation transforms behavior definition into validation result
- ApplyRepositoryChange transforms proposed modification into validated repository state

---

# Required Feature Structure

Every feature definition must include:

- id
- purpose
- inputs
- outcome
- transformation
- observability
- errors

---

# Naming Conventions

## Feature IDs

```text
VerbNoun
```

Examples:
- CreateBehaviorIntent
- ExecuteValidation
- ApplyRepositoryChange

---

## Events

```text
<Entity><Action><Outcome>
```

Examples:
- BehaviorIntentCreated
- ValidationExecutionCompleted
- RepositoryChangeRejected

Events must be:
- structured,
- machine-readable,
- timestamped,
- correlated.

---

## Metrics

```text
<feature>_<measurement>
```

Examples:
- execute_validation_duration
- validation_pass_rate
- apply_repository_change_failures

---

## Errors

```text
snake_case_failure_reason
```

Examples:
- validation_timeout
- invalid_behavior_definition
- execution_limit_exceeded

Errors should describe:
- observable failure,
- not implementation detail.

---

# Observability Requirements

Every feature must define:

- events
- logs
- metrics
- correlation_id

## Correlation IDs

Every execution must:
- accept correlation_id,
- propagate correlation_id,
- emit correlation_id in all evidence.

## Events

Events represent:
- state transitions,
- validation outcomes,
- acceptance/rejection decisions,
- execution boundaries.

## Logs

Logs provide:
- execution traceability,
- failure context,
- operational diagnostics.

## Metrics

Metrics should measure:
- success/failure rates,
- execution duration,
- retry frequency,
- validation outcomes.

---

# Optional Modules

Only add:
- reliability
- security
- performance
- consistency

when justified by real operational needs.

---

# Example

```yaml
feature:

  id: ExecuteValidation

  purpose:
    evaluate expected behavior against executable validation

  inputs:
    - behavior_definition
    - validation_definition
    - correlation_id

  outcome:
    validation result is produced

  transformation:
    executable validation evaluates intended behavior

  observability:

    correlation_id:
      required: true

    events:
      - ValidationExecutionStarted
      - ValidationExecutionCompleted
      - ValidationExecutionFailed

    logs:
      - validation execution context
      - validation failure details

    metrics:
      - execute_validation_duration
      - validation_pass_rate

  errors:
    - validation_timeout
    - invalid_validation_definition
```
````
