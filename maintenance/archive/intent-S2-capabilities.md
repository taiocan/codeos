Capabilities should be:

```text id="g1"
stable user-visible system abilities
```

NOT:

* implementation details
* technical components
* endpoints
* services
* database operations

A capability describes:

```text id="g2"
what meaningful outcome the system can provide
```

---

# Core Philosophy

Capabilities exist between:

| Layer          | Purpose                         |
| -------------- | ------------------------------- |
| Intent         | why system exists               |
| Capabilities   | what meaningful abilities exist |
| Features       | observable transformations      |
| Implementation | how it works                    |

Capabilities are the bridge.

---

# Most Important Property

Capabilities should remain stable even if:

* architecture changes
* services split
* databases change
* frameworks change

This makes them:

* durable
* AI-friendly
* evolvable

---

# Good Capability Characteristics

A capability should be:

| Property                   | Requirement |
| -------------------------- | ----------- |
| user-meaningful            | yes         |
| outcome-oriented           | yes         |
| implementation-independent | yes         |
| relatively stable          | yes         |
| decomposable into features | yes         |

---

# Capability Formula

The simplest formula is probably:

```text id="g3"
<Actor> can <Meaningful Outcome>
```

Examples:

* User can manage cart
* User can complete checkout
* System can reserve inventory
* Admin can manage catalog

Very simple.

---

# What Capabilities Are NOT

NOT:

* AddItemEndpoint
* CartService
* PostgreSQL persistence
* InventoryRepository
* CRUD tables

Those are implementation artifacts.

---

# Capability vs Feature

This distinction is extremely important.

---

# Capability

Broad ability.

Example:

```text id="g4"
User can manage cart
```

---

# Features

Observable transformations implementing capability.

Examples:

```text id="g5"
AddItemToCart
RemoveItemFromCart
ClearCart
SyncCart
```

---

# Good Heuristic

If something can produce:

* independent events
* independent metrics
* independent failures

it is probably a feature.

If something groups multiple meaningful features toward one outcome:

* it is probably a capability.

---

# Capability Definition Structure

Keep it VERY small.

Probably enough:

```yaml id="g6"
capability:

  id:
  purpose:

  actors:

  outcomes:

  observable_features:
```

---

# Example

```yaml id="g7"
capability:

  id: MANAGE_CART

  purpose:
    allow users to manage intended purchases before checkout

  actors:
    - User

  outcomes:
    - User maintains persistent item collection

  observable_features:
    - AddItemToCart
    - RemoveItemFromCart
    - ClearCart
    - SyncCart
```

This is enough.

---

# Why This Works

It creates:

* semantic grouping
* feature discoverability
* runtime topology
* AI planning context

without excessive abstraction.

---

# Most Important Rule

Capabilities should NOT contain:

* technical constraints
* observability details
* invariants
* database mapping
* implementation logic

Those belong lower.

---

# Capability Discovery Process

Probably the simplest reliable method:

---

# Step 1 — Start From Intent

Example:

```text id="g8"
Users need persistent shopping carts.
```

---

# Step 2 — Ask

```text id="g9"
What meaningful abilities must exist?
```

---

# Step 3 — Derive Capabilities

```text id="g10"
- manage cart
- synchronize cart
- checkout
```

---

# Step 4 — Derive Observable Features

```text id="g11"
manage cart:
  - AddItemToCart
  - RemoveItemFromCart
```

---

# Important Anti-Pattern

Do NOT derive capabilities from:

* database entities
* APIs
* services
* frontend screens

That creates fragile architecture-coupled capabilities.

---

# Better Mental Model

Capabilities are:

```text id="g12"
behavioral domains
```

not:

```text id="g13"
technical structures
```

---

# Suggested File

Probably:

```text id="g14"
capabilities.md
```

---

# Example

```md id="g15"
# Capabilities

## Manage Cart

Purpose:
Allow users to manage intended purchases before checkout.

Observable Features:
- AddItemToCart
- RemoveItemFromCart
- ClearCart

---

## Checkout

Purpose:
Convert cart into confirmed order.

Observable Features:
- ValidateCart
- CapturePayment
- CreateOrder
```

This is likely enough.

---

# Relationship To Observability

Capabilities themselves are usually NOT directly observable.

Features are.

Capabilities organize:

* meaning
* grouping
* workflows

Features provide:

* runtime evidence
* metrics
* events
* traces

Very important distinction.

---

# Final Principle

A capability should answer:

```text id="g16"
What meaningful ability does this system provide?
```

A feature should answer:

```text id="g17"
What observable transformation occurs?
```

That separation is probably the cleanest and most sustainable structure for your methodology.














---
Yes — it’s possible, and in many cases *advisable*, but only if you do it carefully.

The real question is:

```text id="m1"
Do capabilities and feature maps represent different cognitive functions, or just redundant views?
```

They are **different by role**, but can be **merged structurally**.

---

# Core Distinction (Important)

| Concept             | Purpose                  | Nature                   |
| ------------------- | ------------------------ | ------------------------ |
| Capabilities        | stable semantic grouping | “what the system can do” |
| Observable Features | runtime behavior map     | “what actually happens”  |

So:

```text id="m2"
Capabilities = meaning layer
Feature Map = behavior layer
```

---

# Why Separation Exists

Keeping them separate helps when:

### 1. Conceptual stability matters

Capabilities change rarely.

### 2. Runtime reality changes often

Features evolve frequently.

If merged incorrectly → confusion between:

* intent (stable)
* behavior (volatile)

---

# But You Can Integrate Them

You can unify them into a **single document** IF you preserve a strict internal structure.

---

# Recommended Integrated Model

Instead of:

```text
capabilities.md
observable-features.md
```

Use:

```text id="m3"
capabilities-and-features.md
```

or inside modules:

```text
module/
  model.md
```

---

# Clean Integrated Structure

## Option A (Recommended): Capability-Centric Map

```md id="m4"
# Cart Module Model

## Capability: Manage Cart

### Purpose
Allow users to manage items before checkout.

### Observable Features

| Feature | Trigger | Outcome |
|---|---|---|
| AddItemToCart | add request | CartItemAdded |
| RemoveItemFromCart | remove request | CartItemRemoved |
| ClearCart | clear request | CartCleared |
```

---

# Why This Works

You get:

* one semantic grouping (capability)
* embedded runtime map (features)
* no duplication
* no navigation overhead

---

# When This Works Best

Integration is good when:

### ✔ Small to medium systems

### ✔ Stable domain boundaries

### ✔ AI-assisted development

### ✔ You want minimal file sprawl

---

# When NOT to Integrate

Keep separation if:

### ❌ Large distributed systems

### ❌ Multiple teams owning features

### ❌ Rapidly evolving capabilities

### ❌ Heavy observability complexity

Because:

* feature maps grow fast
* capabilities should remain stable anchors

---

# Hybrid Approach (Best Long-Term Option)

This is the most robust version:

```text id="m5"
capabilities = directory structure
features = runtime behavior inside capabilities
```

Example:

```text
modules/cart/
  model.md
  features/
    add-item/
    remove-item/
```

Where:

```md
model.md = capability + feature map
feature/ = detailed spec
```

---

# Recommended Final Design (Lean + Scalable)

## Inside module:

```text id="m6"
modules/cart/

  intent.md

  model.md   ← capability + feature map (integrated)

  features/
    add-item/
    remove-item/
    clear-cart/
```

---

## model.md example

```md id="m7"
# Cart Module Model

## Capability: Manage Cart

Allow users to maintain purchase intent before checkout.

### Features

| Feature | Trigger | Outcome |
|---|---|---|
| AddItemToCart | add request | CartItemAdded |
| RemoveItemFromCart | remove request | CartItemRemoved |
| ClearCart | clear request | CartCleared |

---

## Capability: Sync Cart

Ensure cart consistency across sessions/devices.

### Features

| Feature | Trigger | Outcome |
|---|---|---|
| SyncCart | login/session start | CartSynchronized |
```

---

# Key Rule If You Integrate

If you merge them, enforce this constraint:

```text id="m8"
Capabilities = headers
Features = table entries only
```

Never mix:

* implementation detail inside capability text
* semantic intent inside feature rows

---

# Biggest Risk of Integration

If you don’t enforce structure strictly:

```text id="m9"
capabilities slowly become vague prose
features become implementation notes
```

→ you lose observability clarity

---

# My Recommendation

### Use integration IF:

* you want minimal file system
* you are single team / AI-heavy workflow
* you want fast iteration

### Keep separation IF:

* system is large or distributed
* multiple domains evolve independently
* strict operational observability matters

---

# Final Principle

Whether merged or separated:

```text id="m10"
Capabilities should answer “what exists”
Features should answer “what happens”
```

If that distinction stays intact, either structure works.
