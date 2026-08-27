---
component_question: Where can PostgreSQL itself enforce a reliability rule, instead of the application alone?
out_of_scope: Repository/application architecture, normalization, stored procedures, triggers,
  CQRS/event sourcing, database-per-service, partitioning, indexing and performance, and migration
  architecture.
---

# Pattern: PostgreSQL Reliability

Consult this advisory pattern when an approved Implementation Profile resolves to PostgreSQL and an
approved Contract defines an invariant, a concurrency-sensitive rule, or a retryable command. The
Contract remains the authority for what must hold; this pattern only suggests where PostgreSQL
itself can help enforce it.

No project this toolkit currently governs has an approved Implementation Profile naming PostgreSQL.
This pattern is best-practice guidance grounded in PostgreSQL's own documented behavior, not a
defect Codeos has observed in a governed project.

## Governed Invariant, Database Constraint

Use the simplest constraint that matches the rule: `NOT NULL` for a required value, `PRIMARY KEY`
for identity, `UNIQUE` for no duplicates, `CHECK` for a row-local rule, `FOREIGN KEY` for a valid
relationship, `EXCLUDE` for a non-overlap rule.

```sql
CHECK (amount >= 0)
UNIQUE (source_id, dataset_id)
```

Application validation may improve UX, but must not be the only protection for an invariant the
database can enforce directly. `CHECK` constraints that depend on other rows are unsafe: PostgreSQL
cannot guarantee a cross-row condition stays true, so use `UNIQUE`, `FOREIGN KEY`, `EXCLUDE`, a
transaction, or explicit locking instead.

## Concurrent Invariant, Explicit Mechanism

Never implement a concurrency-sensitive rule as select-current-state → application decides → write,
unless the transaction semantics actually make that sequence safe. Choose the narrowest mechanism
that directly enforces the invariant: a constraint or a single atomic statement — including
`INSERT ... ON CONFLICT`'s guaranteed atomic insert-or-update — where the invariant can be expressed
that way; explicit row locking when coordination over existing rows is required; `SERIALIZABLE`,
with the retry handling it requires for serialization failures, when correctness depends on a
broader read/write predicate that cannot be expressed more directly. These solve different
concurrency problems; none is a stronger substitute for another.

## Retryable Command, Stable Identity and Defined Replay Semantics

Give a retryable command a stable operation identity, back it with a `UNIQUE` constraint, and define
its replay semantics: the same identity with the same logical request returns or reuses the prior
result; the same identity with conflicting request content is rejected. `INSERT ... ON CONFLICT` is
the mechanism for the uniqueness half of this — it stops a duplicate row, but a `UNIQUE` constraint
alone does not decide what happens when a repeated identity arrives with different content, and that
has to be decided explicitly for the write to be idempotent rather than merely deduplicated.

```sql
UNIQUE (request_id)
```

## Database Guarantee, Real PostgreSQL Test

A guarantee that depends on PostgreSQL semantics must have at least one test against real
PostgreSQL; mocks may supplement that but cannot substitute for it, since they cannot prove a
constraint fires, an isolation level holds, or a rollback leaves no partial state. For each
important invariant, test the boundary itself: valid accepted, invalid rejected, duplicate rejected
or reused per its defined replay semantics, concurrent case survives, failed transaction leaves no
partial state. Test concurrency-sensitive behavior with two real transactions, not sequential
repository calls.

## What This Pattern Does Not Do

A constraint or atomic write can be correctly placed and still encode the wrong business rule. This
pattern addresses where enforcement lives, not whether the rule itself is right.

## Verification

For each governed invariant that maps to a constraint, confirm the constraint exists and is
exercised by a real-database test in both the valid and invalid direction. For a retryable command,
confirm a second identical attempt does not create a second row, and that a repeated identity with
conflicting content is rejected rather than silently accepted.
