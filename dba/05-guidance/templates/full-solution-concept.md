# Full-Solution Concept: [project or product]

> This document is non-authoritative solution reasoning. It recommends a coherent whole-solution
> direction but does not approve behavior, architecture, language, or implementation. Approved
> DBA artifacts prevail.

## Product Problem and Target Experience

[Who is the solution for, what problem and outcome define the whole product, what should the target
experience feel like, and what is inside or outside the boundary? Do not organize the target
experience around delivery order.]

## Critical Interactions and Quality Expectations

[Include only interactions and quality expectations that materially drive the whole solution.
State relevant context and observable response. Treat unsupported measures as candidate validation
needs rather than commitments, and avoid prescribing mechanisms prematurely.]

## Architecture Drivers

| Priority | Driver | What it means for this solution |
|---|---|---|
| 1 | [quality driver] | [concrete decision lens] |

## Logical Solution Direction

[Use the smallest set of technology-neutral views needed to show logical responsibilities,
ownership, and consequential runtime or data flow. Use conventional architecture terminology when
helpful, but do not force a particular architecture style. Distinguish canonical state from
derived projections when that distinction matters.]

### Technology Direction and Delivery Evolution

[Map only consequential technologies to logical responsibilities. Separately show current binding
foundations, the next operable increment, and later target capabilities. Delete this subsection if
neither mapping nor delivery evolution adds durable value.]

## Consequential Choices

Use `EXISTING BINDING`, `RECOMMENDED`, `EXPERIMENT`, or `OPEN` only for choices where the distinction
materially helps the reader.

| Choice | Direction and rationale | Confidence, risk, or constraint | Validation need | Promotion owner |
|---|---|---|---|---|
| [consequential choice] | [current direction and why] | [what remains uncertain] | [what must eventually be proven] | [Specification Package, Architecture Scope, or Implementation Profile] |

## Validation, Risks, and Open Decisions

| Concern | What must eventually be proven and why? | Possible verification and timing | Owner if promoted |
|---|---|---|---|
| [important concern] | [claim and consequence] | [proportionate way and decision point] | [Contract, Architecture Scope, or normal delivery verification] |

- [Only consequential uncertainty that affects the whole-solution recommendation.]

Delete empty sections. Reassess this concept only when it becomes materially inadequate; minor
staleness is not a governance failure.
