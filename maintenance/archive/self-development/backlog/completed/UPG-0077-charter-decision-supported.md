---
feature_id: UPG-0077
slug: charter-decision-supported
title: Charter Decision Supported Statement
status: COMPLETE
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: Charter Decision Supported Statement

## Problem

A Solution Charter defines the problem, vision, outcomes, and boundary, but can still leave the
primary decision the solution supports implicit. That ambiguity can weaken GUI prioritization and
make technically valid features harder to assess for contribution to the intended business or user
outcome. Unlike the browser-verification gap, this has not yet produced repeated expensive failure,
so it is a P2 proposal and must be approved separately before implementation.

## Upgrade

Add one Solution Charter interface element after Vision:

```markdown
## Decision Supported

[One concise statement naming the primary actor and primary decision supported by the solution.]
```

The statement must agree with Problem, Vision, Outcomes, and Scope. It is an alignment lens, not an
acceptance requirement. It creates no identifier, feature mapping, traceability obligation,
lifecycle state, or decision boundary. Secondary decisions may remain elsewhere; this statement
identifies the primary one.

Update Doctrine only if the existing normative semantics are insufficient to require or define the
new Charter statement. If no Doctrine change is necessary, keep this an interface, authoring, and
review-rubric change. If normative semantics must change, treat implementation as PROTECTED, version
the affected component, and activate the change atomically through a new DBA configuration after
human approval.

## Scope

**In scope:** the Charter template, Solution Charter authoring support, and the Charter review
rubric; the minimum normative change proven necessary during implementation.

**Out of scope:** another Charter artifact; decision trees; per-feature mappings; another acceptance
criterion; a new lifecycle state or gate; external-model purpose assessment; automatic component
classification; migration of approved Charters; unrelated review-policy or reviewer-tool changes.

## Acceptance

- The Charter interface asks for one concise statement naming the primary actor and primary
  decision, without imposing sentence-count formatting governance.
- Authoring and review guidance checks consistency with Problem, Vision, Outcomes, and Scope while
  stating that the field is not an acceptance requirement.
- A new or materially revised Charter uses the interface; existing approved Charters remain valid
  without migration.
- No feature mapping, identifier, traceability surface, lifecycle state, or decision boundary is
  introduced.
- Doctrine remains unchanged unless a documented ownership analysis shows existing semantics are
  insufficient; any resulting normative change follows PROTECTED approval and component versioning.
- Existing configuration/layout and relevant reviewer tests pass.

## Value and Risk

**Value:** gives product, UX, and architecture decisions one concise alignment lens without adding
another layer of requirements.

**Risk:** the statement could become duplicate purpose prose or an informal acceptance gate. Keep it
primary, concise, and explicitly non-acceptance-bearing.

## Outcome

Completed on 2026-08-31 as a protected Charter-interface change. New or materially revised
Charters now name the primary actor and primary decision supported after Vision. Existing approved
Charters remain valid without migration, and the template, authoring workflow, and reviewer
checklist all state or preserve the boundary that this is an alignment lens rather than an
acceptance requirement or feature behavior.

The canonical Solution Charter definition was updated for consistency. Lightweight guidance checks
protect the template, applicability, compatibility, and non-acceptance wording; the existing
reviewer unit test protects the semantic review prompt. Both were positive-controlled by removing
their required content and observing the expected failure before restoration. The complete
`dba/04-tools/tests/run.sh` suite passes.

Doctrine, DBA-4 configuration, review policy, lifecycle, identifiers, traceability, and existing
approved downstream Charters were not changed.
