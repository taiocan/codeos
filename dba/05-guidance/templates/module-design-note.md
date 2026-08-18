---
module: [module_slug]
verified_against_commit: [git SHA]
---

# Module Design Note: [module]

<!--
Optional descriptive documentation of how one implementation module currently works. Explanatory
only: it never governs behavior or structure, and it is stale rather than authoritative when it
disagrees with the code. Create one only when understanding the module from approved artifacts
alone would require reading substantial code. When saved, use `.codeos/03-design/<module-slug>.md`.

`verified_against_commit` records when the description was last checked against the code. A later
commit does not by itself invalidate the note; the doctrine's update trigger does.

Purpose and How it works are required. Omit any other section that adds nothing.
-->

## Purpose

[What responsibility this module owns, and what it explicitly does not own.]

## How it works

[The major processing flow in roughly five to ten steps, in engineering terms a reader can follow
without opening the source. One small diagram may replace or accompany the steps.]

## Main parts

[The few important internal parts and what each does, by logical name. Not an inventory of every
type or function.]

## Data and state

[What comes in, what state is read or written, and what comes out.]

## Design choices

[Only the choices that materially explain the implementation, especially consequential local ones.]

## Dependencies and boundaries

[What other modules this one calls, and what it must not bypass or own.]
