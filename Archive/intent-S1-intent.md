It can do both.

But the important thing is:

```text id="f1"
Intent should exist at every meaningful boundary of autonomy.
```

Not just once globally.

---

# Core Principle

Intent is not:

* a giant master vision statement
* a requirements document
* a business plan

Intent is simply:

```text id="f2"
why this thing exists
```

Anything that:

* can evolve independently
* can fail independently
* can be reasoned about independently

should probably have its own intent.

---

# Recommended Intent Hierarchy

Usually:

| Level                | Purpose                     |
| -------------------- | --------------------------- |
| Product Intent       | why the whole system exists |
| Domain/Module Intent | why a subsystem exists      |
| Feature Intent       | why a feature exists        |

This creates very clean reasoning boundaries.

---

# Example

---

# Product Intent

```md id="f3"
# Product Intent

Enable users to purchase products online reliably across devices.
```

Very broad.

---

# Cart Module Intent

```md id="f4"
# Cart Module Intent

Maintain temporary user purchase selections prior to checkout.
```

Now bounded.

---

# Payment Module Intent

```md id="f5"
# Payment Module Intent

Convert authorized purchase intent into confirmed financial transaction.
```

Different operational purpose.

---

# Feature Intent

```md id="f6"
# Feature Intent: AddItemToCart

Allow user to persistently associate an item with their active cart.
```

Very localized.

---

# Why Multiple Intent Levels Matter

Without modular intents:

* subsystems become semantically blurry
* feature boundaries degrade
* AI agents lose contextual understanding
* observability naming drifts

Intent provides:

* semantic compression
* architectural alignment
* local reasoning context

---

# Important Heuristic

If something can be:

* deployed separately
* observed separately
* owned separately
* evolved separately

it probably deserves its own intent.

---

# But Avoid Overfragmentation

Do NOT create intent documents for:

* every class
* every utility
* every endpoint

Intent should exist only for:

```text id="f7"
meaningful behavioral boundaries
```

---

# Good Boundaries

Usually:

* product
* domain/module
* major workflow
* feature

---

# Bad Boundaries

Usually:

* repository layer
* DTOs
* helper functions
* infrastructure utilities

Those are implementation details.

---

# Recommended Structure

```text id="f8"
project/

  intent.md

  modules/

    cart/
      intent.md
      observable-features.md
      features/

    payment/
      intent.md
      observable-features.md
      features/
```

This scales very naturally.

---

# Important Philosophical Point

Intent is NOT architecture.

Example:

* Cart module could move:

  * monolith
  * microservice
  * event-driven system

while intent remains stable.

That stability is extremely valuable.

---

# Most Important Insight

Intent acts like:

```text id="f9"
semantic gravity
```

It helps:

* humans
* AI agents
* observability systems
* naming conventions

converge toward coherent behavior.

---

# Final Recommendation

Yes:

* product can have intent
* modules can have intent
* features can have intent

Use intent wherever there is:

* independent behavioral purpose
* independent observability
* independent evolution

That is probably the cleanest heuristic.
