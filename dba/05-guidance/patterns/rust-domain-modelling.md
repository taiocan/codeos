---
component_question: When does a simple Rust type directly prevent a governed invalid state?
out_of_scope: Boundaries and adapters, governed events, failure signals, crate layout and verification commands, and general Rust design-pattern advice.
---

# Pattern: Rust Domain Modelling

Consult this advisory pattern when an approved Implementation Profile resolves to Rust and an
approved Contract or Event Schema defines a closed set of values, a constrained value, or an
ordering rule. The Contract and Event Schema remain the authority for what must hold; this pattern
only suggests where a type can help implement a governed constraint structurally.

> When a simple Rust type can directly prevent a governed invalid state, prefer that representation
> over an unconstrained primitive.

Where no simple type prevents the state, validate at runtime and emit the governed failure signal.
Neither choice changes what the approved artifacts require.

## Closed Set

When an approved artifact enumerates allowed values — a schema's `allowed values` list, or a
contract's set of distinguishable outcomes — an `enum` with one variant per value keeps the set
closed and makes an unhandled addition a compile error. A `String` field leaves the set open, so a
value outside the approved set becomes a runtime concern rather than an impossible one.

## Constrained Value

When an approved artifact constrains a value's form rather than only its name — `uuid-v4` in the
required base fields is the recurring case — a newtype whose constructor is the only way to build it
prevents an arbitrary string from reaching the field. Distinct identifiers that share a primitive
type also stop being interchangeable by accident.

## Ordering Invariant

When a contract states that one step must precede another, consider whether the API can make the
wrong order unavailable — a method that consumes `self` and returns the next type, or a constructor
that requires the earlier result as an argument. This is worth doing for a small, stable lifecycle.
It is **not** a default: for many states, or a lifecycle still changing, an `enum` state machine or
simply a narrower public API is simpler and equally sufficient.

## What This Pattern Does Not Do

A type prevents a value from being *misrepresented*; it does not force code to *consult* it. A
field can be correctly typed and still be ignored by the logic that should have used it, and no
modelling choice here detects that. Reviews and tests remain responsible for use.

## Verification

Confirm each closed set in the approved artifacts appears as a closed type or is validated with its
governed failure signal, and that no approved constraint was weakened to fit a type. Preserve
behavior with proportional tests.
