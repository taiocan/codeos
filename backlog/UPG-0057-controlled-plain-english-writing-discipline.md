---
feature_id: UPG-0057
slug: controlled-plain-english-writing-discipline
title: Controlled Plain English Writing Discipline
status: PROPOSED
priority: P3
depends_on: [UPG-0056]
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: controlled-plain-english-writing-discipline — Controlled Plain English Writing Discipline

**Priority**: P3
**Status**: PROPOSED
**Type**: downstream-doctrine + self-dev-governance (two CHGs: CHG-A downstream doctrine, CHG-B
self-development adoption)

## Problem

AI-generated prose across Codeos (both downstream DBA artifacts and Codeos's own self-development
briefs/change records) has no documented discipline distinguishing plain communication from
specification-grade precision, protecting literal/quoted content, or keeping reviewer prose advisory
and separate from specification writing. A detailed 15-section "Controlled Plain English" writing
guideline was supplied as the source for this discipline, scoped by the human to cover both
downstream doctrine and self-development, with an explicit requirement for a real, human-controlled
enable/disable switch.

## Upgrade

A `patterns/controlled-plain-english.md` pattern with four content layers (A: plain communication,
always advisory; B: specification/planning precision, toggle-gated, a **generation discipline** not
a review-compliance regime; C1: existing literal-protection authority, always active; C2: new
literal-protection rules, toggle-gated; D1: reviewer/reporting integrity, always active; D2: plain
review prose, toggle-gated), consuming `UPG-0056`'s minimal one-line status-file convention (no
resolver, no stamps, no versioning — status is exactly `enabled` or `disabled`, missing means
disabled). A 15-section traceability matrix maps every source-guideline section to its Codeos
treatment, with corrected DBA ownership (observable behavior and edge cases belong primarily to
Stage 2 Contract, not Stage 1 Intent or Stage 5 Tests). Full design history — eight review rounds
converging on this lean shape — lives in
`/home/rimo/.claude/plans/calude-consider-this-inputs-steady-pnueli.md`.

**Definition of success**: with the downstream or self-dev status file set to `enabled`, Stage
1-10/self-dev prompts apply Layer B/C2 to specification-grade prose while Implementation Notes and
Stages 5/6/8 stay factual reporting; reviewers apply D1 always and D2 when enabled, using ordinary
review authority (no separate "CPE violation" category, no historical-compliance audit). With no
status file (or `disabled`), behavior is unchanged from today.

## Scope

**In scope**: the pattern file; `dba-system.md` and `CLAUDE.md` doctrine sections; the call-site map
and shared-reviewer status-injection contract; check-line wiring across the numbered Stage prompts,
`04-implement.md`/`05-tests.md`/`06-observe.md`/`08-replay.md` (factual-reporting profile),
`09-refine.md`/`10-arch-refine.md`, `07-reconcile.md`, `pipeline-reviewer.md`,
`codeos-reviewer-task.md` (made configuration-neutral), `scripts/dba-init.sh` and
`scripts/codeos-review.sh` (existing-script integration only), `prompts/codeos-self-dev.md`.

**Out of scope**: any change to `UPG-0056`'s own convention beyond consuming it; any resolver,
stamp, or versioning mechanism; any new Stage ID or mandatory approval gate; any Non-Negotiable Rule
change; `00a/00b/00c-*.md`, `00-session-end.md`, `reviewer-automated.md`, `verify-only.md` (deferred,
smaller follow-up).

## Value

Gives Codeos-generated prose a documented, human-toggleable discipline without inventing a
provenance/audit subsystem — meaning-loss is still caught (existing review authority), style
preference stays advisory, and legacy/unconfigured projects are unaffected.

## Risk

Low. No code. The main risk is prompt-wiring drift across ~15 touched files — mitigated by the
explicit call-site map and Reconcile's consistency sweep.

## Guardrail

- Controlled Plain English never creates a separate review-compliance category; findings stay under
  existing DBA review authority.
- The shared `codeos-reviewer-task.md` template never reads filesystem configuration itself — it
  only receives an injected status line from its caller.
- Non-retroactivity is a one-sentence doctrine rule, not a stamped/audited mechanism.

## Related

- Depends on `UPG-0056` (Optional Mechanism Status Convention), which must reach `COMPLETE` before
  this UPG's own Step 3 (Implement) can begin.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the change
> records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| — | (not yet opened) | — | — |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
| — | — | — |
