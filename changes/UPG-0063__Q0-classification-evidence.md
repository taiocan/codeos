# UPG-0063 Q0 — Does a material NEW DESIGN decision exist in shipped DBA code?

<!--
Gates UPG-0063 entirely. If Q0 finds nothing material, the feature closes: no demonstrated
governance problem. Recording cost is deliberately NOT revisited here — first establish that
something needs recording.

Runs against EA-0001 (correcting the original mis-analysis) plus three PlotSpot modules (an
independent DBA project, previously unexamined).
-->

## 0. Method and decision rule — PRECOMMITTED before any classification

**Declared 2026-08-04, before reading PlotSpot's contracts or reclassifying EA-0001.**

### The error being corrected

UPG-0062 conflated *"does the contract prescribe the code structure or name?"* with *"does the
approved artifact already determine the semantic rule?"* Those are very different questions. Only the
second one matters. A contract that states a rule has governed it, whatever vocabulary the code then
uses to realise it.

### Classes

| Class | Meaning |
|---|---|
| `SOURCE-DETERMINED` | Approved artifacts already determine the rule; the code merely realises it |
| `ORDINARY IMPLEMENTATION CHOICE` | Artifacts leave freedom, but the choice carries no governance significance |
| `MATERIAL NEW DESIGN` | Artifacts leave multiple valid possibilities, **and** the selected mechanism materially determines invariant placement, component responsibility, state/data integrity, or future architectural freedom |

### The two-part test — both must be YES for `MATERIAL NEW DESIGN`

1. **Could another materially different mechanism satisfy every approved artifact without requiring an
   artifact revision?**
2. **Would choosing between those mechanisms matter enough that a future maintainer or reviewer should
   know it was deliberate?**

`NO` at (1) → `SOURCE-DETERMINED`. `YES` then `NO` → `ORDINARY IMPLEMENTATION CHOICE`.

### Explicitly not counted

Newtypes, resolver objects, helper seams, predicates, trait injection, and particular Rust structures
are **not** counted merely because they are absent from a contract. They may be nothing more than
implementation techniques for an already-governed semantic rule. Absence of the technique's *name*
from an artifact is not evidence of anything — that was the original error.

### Decision rule — fixed in advance

| Q0 outcome | Consequence |
|---|---|
| No `MATERIAL NEW DESIGN` across EA-0001 + PlotSpot | **Close UPG-0063.** No demonstrated governance problem |
| One isolated case | Record it, but **probably still close or downgrade** the feature |
| Repeated material cases across independent features/projects | **Proceed to Step 1**, with real evidence for the problem and some indication of its shape |

No renegotiation of these after the results are in.

---

## 1. Findings

*(pending — analysis follows)*
