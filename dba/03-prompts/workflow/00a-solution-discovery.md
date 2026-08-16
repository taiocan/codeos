---
component_question: How should optional pre-feature discovery map an unfamiliar problem domain?
out_of_scope: Approving feature behavior, deciding architecture, producing specification artifacts, and implementation.
---

# Solution Discovery

## Purpose

Explore an unfamiliar domain before feature boundaries or Intents exist. This is optional,
non-authoritative planning; it is not a stage or approval gate.

## Inputs / Prerequisites

Agree with the human on the domain and which of these areas matter: problem framing, candidate
feature groups, critical actor interactions, quality hypotheses, shared vocabulary, event or
configuration hypotheses, architectural risks, and explicit non-decisions.

## Task

- Ask only questions needed for the selected areas.
- Label prospective features and configuration as `CANDIDATE`; label events and unsupported
  quality expectations as `HYPOTHESIZED`.
- Record out-of-scope discoveries as backlog candidates unless the human expands the session scope.
- Route later behavioral claims through normal Stage 1–3 work and project-level structural
  decisions through the selected architecture policy.
- Discovery may identify critical interactions and quality hypotheses. Keep detailed interaction
  design and solution-quality realization in a Full-Solution Concept or later approved artifacts.
- Do not write Intent, Contract, Event Schema, architecture, or implementation artifacts.

## Output / Next Action

Produce a concise planning document beginning with:

> This document is non-authoritative planning material. It does not approve features,
> architecture, contracts, schemas, events, or implementation. Approved DBA artifacts prevail.

Include only explored areas and an explicit non-decisions/backlog section. The result is inline and
has no required path. Save it at `.codeos/00-discovery/<topic-slug>.md` only when the human
explicitly requests persistence and the reasoning has durable value. If content is later
incorporated into governed artifacts, review occurs at the next applicable review point under the
selected review policy. When the problem is understood but the coherent whole-solution direction
remains unclear, route optional exploration to `00-full-solution-concept.md`.
