---
component_question: How should a solution be framed before any of its proposed meaning becomes governed truth?
out_of_scope: Approving solution meaning, decomposing features, deciding architecture, producing specification artifacts, and implementation.
---

# Solution Framing

## Purpose

Frame the solution's problem, vision, candidate outcomes, scope, and candidate constraints before
governing them. This support workflow is optional, non-authoritative planning; it is not a stage or
approval gate.

## Inputs / Prerequisites

Use the human's description, existing evidence, and relevant approved authority. Clearly
distinguish existing binding decisions from proposals. Do not reopen approved decisions merely
because framing takes a whole-solution view.

## Task

1. **Problem** — describe what is wrong, difficult, risky, or missing and who is affected, without
   starting from technology.
2. **Vision** — describe the better future in terms understandable without implementation detail.
3. **Candidate outcomes** — propose measurable improvements, not features. Do not assign governed
   outcome identities.
4. **Scope** — propose what belongs inside and outside the solution boundary.
5. **Candidate constraints** — identify possible cross-cutting obligations and the evidence or
   source that could justify them. Unsupported thresholds remain hypotheses.

Framing may identify an architecture-relevant concern, its reason, and the status
`OPEN — resolve during Architecture Synthesis`. It MUST NOT resolve that concern into components,
responsibilities, flows, interfaces, technologies, runtime placement, data ownership, or
implementation structure.

Label proposed outcomes and constraints as `CANDIDATE`. Record explicit non-decisions and
out-of-scope discoveries only when they have durable value. Do not write a Charter, Intent,
Contract, Event Schema, Architecture Scope, or implementation artifact.

## Output / Next Action

Present concise framing beginning with:

> This document is non-authoritative planning material. It proposes solution framing but does not
> approve outcomes, scope, constraints, features, architecture, behavior, or implementation.
> Approved DBA artifacts prevail.

The result is inline and has no required path. Save it at
`.codeos/00-discovery/<topic-slug>.md` only when the human explicitly requests persistence and the
reasoning has durable value. Proposed outcomes, scope, and constraints become governed truth only
when promoted into and approved in the Solution Charter. Open architecture concerns remain
non-authoritative until Architecture Synthesis decides them. Review promoted claims at the next
applicable review point under the selected review policy.
