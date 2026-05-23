# agent.md

## Purpose

`capabilities.md` defines the stable behavioral abilities implied by `intent.md`.

Capabilities describe:
- what meaningful abilities the system provides,
- which actors achieve which outcomes,
- which behavioral domains exist.

Capabilities are not:
- features,
- workflows,
- APIs,
- services,
- databases,
- infrastructure,
- or implementation structure.

They exist between:
- intent (`why`)
- features (`observable transformations`)
- implementation (`how`)

---

# Core principles

Keep capabilities:
- stable
- user-meaningful
- implementation-independent
- decomposable into features

Use:
- actor + outcome language
- simple wording
- behavioral semantics

Avoid:
- architecture
- endpoints
- storage models
- technical components
- observability mechanics
- implementation constraints

Capabilities should remain valid even if:
- frameworks change,
- databases change,
- services split,
- architecture changes.

---

# Capability formula

```text
<Actor> can <Meaningful Outcome>
```

Examples:
- User can manage cart
- User can complete checkout
- Admin can manage catalog

---

# Capability rules

## 1. Capabilities describe meaningful abilities

A capability answers:

```text
What meaningful ability does the system provide?
```

### Good
- User can manage purchases
- Admin can manage catalog

### Bad
- CartService persists items
- API validates requests

---

## 2. Capabilities are not features

Capabilities group related observable features.

### Capability
- User can manage cart

### Features
- AddItemToCart
- RemoveItemFromCart
- ClearCart

---

## 3. Do not derive capabilities from technical structure

Never derive capabilities from:
- APIs
- database tables
- services
- frontend screens
- infrastructure

### Bad
- InventoryRepository
- CheckoutEndpoint
- PostgreSQL persistence

---

## 4. Keep capabilities implementation-independent

Do not include:
- frameworks
- protocols
- storage details
- runtime topology
- observability systems

Capabilities describe behavior only.

---

## 5. Keep capabilities stable and broad

A capability should survive:
- feature expansion,
- refactoring,
- architecture rewrites,
- infrastructure changes.

If something represents a single observable transformation,
it is probably a feature.

---

# Capability discovery process

## Step 1 — Read intent

Identify:
- actors,
- outcomes,
- stable behavioral domains.

---

## Step 2 — Ask

```text
What meaningful abilities must exist?
```

---

## Step 3 — Derive capabilities

Example:

Intent:
```text
Users need persistent shopping carts.
```

Capabilities:
- User can manage cart
- User can synchronize cart
- User can complete checkout

---

## Step 4 — Derive observable features

```text
Manage cart:
- AddItemToCart
- RemoveItemFromCart
- ClearCart
```

---

# Suggested structure

```yaml
capability:

  id:
  purpose:

  actors:

  outcomes:

  observable_features:
```

---

# Example

```yaml
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
```

---

# Evaluation heuristic

A valid capability:
- describes a meaningful behavioral domain,
- groups related observable features,
- remains stable across implementation changes,
- avoids technical structure.

A capability does not describe:
- APIs,
- services,
- databases,
- infrastructure,
- workflows,
- or implementation logic.