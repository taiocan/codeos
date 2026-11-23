The proposed A0–A7 rename is cleaner than the current numbering, but I would **not implement it in this form**. It spends significant change effort on naming and then adds new enforcement such as “gap-free A0–A7,” without improving the actual DBA reasoning very much.

The better simplification is to reduce the **workflow model first**, and leave filenames alone unless renaming later becomes cheap.

## 1. The current support layer is more fragmented than necessary

The first three workflows are conceptually very close:

```text
00a-solution-discovery
        ↓
00-full-solution-concept
        ↓
00-charter
```

They all deal with the same solution-level questions:

> **Problem → Vision → Outcomes → Scope → Constraints**

The real distinction is not their subject matter. It is **authority**:

* discovery/concept = exploration;
* Charter = governed truth.

That distinction should remain.

I would therefore reduce these **three conceptual workflows to two**.

### A. Solution Framing — non-authoritative

One workflow answers all five questions:

```text
Problem
Vision
Outcomes
Scope
Constraints
```

Its purpose is:

> Understand and frame the solution before governing it.

It can replace both:

```text
00a-solution-discovery.md
00-full-solution-concept.md
```

The output can still be a `full-solution-concept.md` if that artifact is useful.

There is little value in having one workflow discover the problem and another independently synthesize Problem + Vision + Outcomes + Scope again.

### B. Solution Charter — authoritative

Keep this separate and deliberately small.

Its purpose becomes:

> Review the proposed solution framing and approve only the minimum solution-level authority needed by downstream work.

The Charter contains the governed subset:

```text
Problem
Vision
Outcomes
Scope / Boundary
System Constraints
```

The important difference becomes extremely clear:

```text
Solution Framing
    exploratory
    can hypothesize
    can change freely
        ↓
Solution Charter
    governed
    concise
    approved
        ↓
Feature Intents
```

This is simpler than three workflows **without weakening the authority boundary**.

---

# 2. I would not combine framing and Charter approval into one workflow

It is technically possible to create one large workflow:

```text
Discover → Frame → Approve
```

but I don't recommend it.

The same prompt would then be responsible for both:

> “Explore freely”

and:

> “Create governed truth.”

That makes the authority boundary harder to understand and easier for an agent to cross accidentally.

So I think **two parts is the sweet spot**:

> **Solution Framing + Solution Charter**

not three, and not one.

---

# 3. The support workflow set can therefore become much simpler

Conceptually, I would aim for this:

| Support workflow             | Purpose                                           |
| ---------------------------- | ------------------------------------------------- |
| **Solution Framing**         | Problem, Vision, Outcomes, Scope, Constraints     |
| **Solution Charter**         | Approve solution-level authority                  |
| **Feature Decomposition**    | Turn solution scope into candidate features       |
| **Existing-Codebase Intake** | Understand an existing system before applying DBA |
| **Architecture Synthesis**   | Resolve cross-feature structural decisions        |
| **Session Orientation**      | Determine what work should happen now             |
| **Session Handoff**          | Preserve state for the next session               |

That gives **7 support workflows instead of 8**, but more importantly the first part now has a much clearer model.

I would not force further consolidation. For example:

* onboarding solves a different entry condition;
* architecture synthesis has different authority and timing;
* session start/end have operational purposes rather than engineering reasoning purposes.

Combining those would save filenames but increase prompt branching.

---

# 4. I would also question whether `Feature Decomposition` should be positioned in a linear A sequence

Your current proposed sequence implies:

```text
A0 Problem Discovery
A1 Full Solution Concept
A2 Charter
A3 Feature Decomposition
A4 Existing Codebase Intake
A5 Architecture
A6 Session Orientation
A7 Session Handoff
```

But this is not really one sequence.

For example:

```text
Existing-Codebase Intake
```

can happen before Solution Framing.

And:

```text
Session Orientation
```

can happen before virtually anything.

Architecture Synthesis happens later, after relevant feature specifications exist.

So `A0–A7` creates a conceptual order that does not actually exist, even if documentation says it is “inventory order only.”

That is a smell.

If you have to explain:

> “The consecutive numbering looks like sequence, but it is not sequence”

then the namespace is not helping enough.

---

# 5. Therefore I would drop the “gap-free A namespace” requirement

This proposed invariant is unnecessary:

> support identifiers are gap-free from A0.

It governs cosmetics rather than engineering behavior.

Suppose later you remove one support workflow. Why should this:

```text
A0
A1
A3
A4
```

be invalid?

Nothing in software correctness depends on that.

Likewise, adding a workflow shouldn't require renaming everything after it merely to preserve sequence.

So I would **not create a mechanical contract around consecutive support numbering**.

Reserve mechanical checks for properties that protect real semantics:

```text
01–09 correspond exactly to governed Stage IDs
every referenced prompt exists
every doctrine adapter resolves to exactly one owner
support prompts cannot masquerade as Stage IDs
```

Those are valuable invariants.

“Gap-free A numbers” is not.

---

# 6. Consider avoiding numeric support IDs completely

If you eventually rename them, I think this is actually cleaner:

```text
support-solution-framing.md
support-solution-charter.md
support-feature-decomposition.md
support-codebase-intake.md
support-architecture-synthesis.md
support-session-start.md
support-session-end.md
```

versus:

```text
A0-
A1-
A2-
...
```

Why?

Because your most important discovered fact is:

> **Numbers have semantic meaning in DBA because 01–09 are Stage IDs.**

The safest solution is therefore not to create **another numeric-looking namespace**.

You get a very obvious distinction:

```text
01-intent.md
02-contract.md
03-event-schema.md
04-implement.md
...
09-refine.md

support-solution-framing.md
support-solution-charter.md
support-feature-decomposition.md
support-architecture-synthesis.md
...
```

Nobody has to learn what `A4` means.

The filename says what it is.

---

# 7. But I would still not rename them now

This is the most important practical recommendation.

You have already discovered that filenames are executable dependencies.

Therefore I would separate two questions:

### Semantic improvement

Worth doing:

```text
merge Discovery + Full-Solution-Concept workflow semantics
clarify Solution Framing → Charter
simplify responsibility boundaries
```

### Filename cleanup

Currently optional and coupled.

So do not make the semantic improvement dependent on:

```text
00a → A0
00 → A1
00-charter → A2
...
```

You could first simplify the actual workflow responsibilities while retaining existing filenames.

For example:

```text
00a-solution-discovery.md
```

could become the single **Solution Framing** workflow.

Then retire `00-full-solution-concept.md` if its workflow responsibility has been absorbed.

`00-charter.md` remains the governed step.

That produces actual simplification:

```text
BEFORE

Discovery
  ↓
Full-Solution Concept
  ↓
Charter


AFTER

Solution Framing
  ↓
Charter
```

That is much more valuable than changing eight prefixes.

---

# Recommended low-complexity target

I would aim for this conceptual model:

```text
                SOLUTION LEVEL

        Solution Framing
 Problem · Vision · Outcomes
   Scope · Constraints
        [non-authoritative]
               │
               ↓
        Solution Charter
       [governed authority]
               │
               ↓
      Feature Decomposition
               │
               ↓
        DBA Stages 1–3
               │
               ↓
     Architecture Synthesis
          [when needed]
               │
               ↓
        DBA Stages 4–9
```

With side-entry utilities:

```text
Existing-Codebase Intake ──→ appropriate entry point

Session Orientation ───────→ appropriate workflow/stage

Session Handoff ←────────── current work state
```

That is more truthful than forcing all support workflows into A0–A7.

---

## What I would change in the current proposal

I would reduce the rename initiative to these decisions:

**Keep**

> `01–09` are reserved governed Stage IDs.

**Add**

> Support workflows are not stages and MUST NOT use filenames that imply Stage identity.

**Simplify**

> Merge Problem Discovery and Full-Solution-Concept workflow responsibility into one **Solution Framing** workflow covering Problem, Vision, Outcomes, Scope, and Constraints.

**Keep separate**

> Solution Charter because it introduces governed authority.

**Remove**

> A0–A7 consecutive ordering as a governed invariant.

**Remove**

> gap-free support-prefix verification.

**Defer**

> actual filename renaming until filename/path coupling has an independent reason to be removed.

---

## Bottom line

I think you have uncovered a more important simplification than the original renaming task.

The best structure is **not**:

```text
8 better-named support workflows
```

It is closer to:

```text
2 solution-definition workflows
    Solution Framing
    Solution Charter

+ a few genuinely distinct support workflows
    Feature Decomposition
    Codebase Intake
    Architecture Synthesis
    Session Orientation
    Session Handoff

+ 9 governed DBA stages
```

And the key simplification is:

> **Combine Problem, Vision, Outcomes, Scope, and Constraints into one Solution Framing workflow, but keep Charter approval separate because exploration and authority are fundamentally different responsibilities.**

I would optimize for that semantic simplification first and treat namespace cleanup as secondary.
