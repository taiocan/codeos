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
**Type**: downstream-doctrine + self-dev-governance + script-tooling (two CHGs: CHG-A downstream
doctrine — pattern + prompt wiring; CHG-B self-development adoption + automatic status delivery in
`scripts/codeos-review.sh` for both the downstream and self-development branches of that shared
wrapper — `tools/reviewer` itself is not touched)

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

**Status note (added after CHG-A's Step 3 review):** `CHG-A` establishes the pattern and consumer
wiring, including a status line `codeos-reviewer-task.md` recognizes when present among reviewed
artifacts. Discovered during `CHG-A`'s implementation: `tools/reviewer` embeds whatever artifact
paths it is given and has no code path to read a config file on an invoker's behalf, so within
`CHG-A` alone that line must be included by hand. **This is accepted as `CHG-A`'s scope, not as
this discipline's final operating model.** `CHG-B` gives `scripts/codeos-review.sh` (the bash
wrapper) the job of resolving the status automatically — for both the downstream and
self-development branches of that one shared script — before invoking the reviewer, and of
establishing that wrapper as the supported entry point so a direct `codeos-reviewer` invocation
cannot silently bypass it. `tools/reviewer` itself is not touched by `CHG-B`. **`UPG-0057` is not
considered complete until `CHG-B` reaches `COMPLETE`.**

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the change
> records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260726-003 | `changes/UPG-0057__CHG-20260726-003__controlled-plain-english-writing-discipline.md` | CHG-A: pattern + downstream doctrine + prompt wiring | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| RVS__UPG-0057__CHG-20260726-003__S1 | CHG-20260726-003 | 1-Intent | R1 | NO OBJECTION |
| RVS__UPG-0057__CHG-20260726-003__S2 | CHG-20260726-003 | 2-Acceptance | R1→R2 | R1 DO NOT ADVANCE (AC3/AC4/AC18 depended on external plan content not in the packet; AC18's "cannot silently reintroduce" overstated an advisory generation discipline as enforcement) → fixed → R2 NO OBJECTION |
| RVS__UPG-0057__CHG-20260726-003__S3 | CHG-20260726-003 | 3-Implement | R1→R3 | R1 DO NOT ADVANCE (`codeos-reviewer-task.md` falsely claimed as a pattern consumer able to enforce the pattern-unavailable check; File Layout "none by default" contradicted `dba-init.sh`'s own scaffolding; "Thirteen files" undercounted the actual 18) → fixed → R2 NO OBJECTION → revised post-R2 per human direction (scaffolded default changed `disabled`→`enabled`) → R3 DO NOT ADVANCE (Implementation Notes' own file-list sentence still said `status: disabled` for `dba-init.sh`, contradicting AC13's own "no remaining stale claim" text) → fixed. **PROFILE-4's 3-round/step budget is now exhausted for Step 3** — fix applied inline per CLAUDE.md's budget-exceeded rule; no further automatic round run; escalated to human decision |
| RVS__UPG-0057__CHG-20260726-003__S4 | CHG-20260726-003 | 4-Reconcile | R1→R2 | R1 DO NOT ADVANCE (Post-R1-fixes historical note still asserted `dba-init.sh` scaffolds `status: disabled` without qualifying it as historical, contradicting AC13/Reconciliation's "no remaining claim") → fixed (note marked as historical record, Reconciliation's AC13 row explains the expected grep hit) → R2 NO OBJECTION — `findings: []`, `unparsed_findings_count: 0` |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|
| (Step 2 R1) AC3/AC4/AC18 depended on external plan content not included in the review packet | RVS__…__S2 | IN-SCOPE BLOCKER | Fixed — made self-contained (exact rule sentence, full 15-item list, explicit rule enumeration inlined) |
| (Step 2 R1) AC18's "cannot silently reintroduce" phrased an advisory generation discipline as an enforcement guarantee | RVS__…__S2 | IN-SCOPE BLOCKER | Fixed — reworded to check that the doctrine text states each rule and refinement prompts carry no carve-out, not a provable behavioral guarantee |
| (Step 3 R1) `codeos-reviewer-task.md` claimed as a pattern consumer subject to the enabled-but-pattern-unavailable rule, but it never reads the pattern or any status file | RVS__…__S3 | IN-SCOPE BLOCKER | Fixed — "Consulted by" and the rule's own text now explicitly carve it out |
| (Step 3 R1) File Layout said the CPE status file is "none by default," contradicting `dba-init.sh`'s own scaffolding at `status: disabled` | RVS__…__S3 | IN-SCOPE BLOCKER | Fixed — File Layout now states it's scaffolded by default, opt-in via a separate human action |
| (Step 3 R1) "Thirteen files touched" undercounted the actual 18 files named in the same sentence | RVS__…__S3 | IN-SCOPE BLOCKER | Fixed — corrected to eighteen |
| (Post-R2, human direction) Scaffolded CPE default changed from `status: disabled` to `status: enabled` across `dba-init.sh`, `dba-system.md`, and this change record | — (human direction, not a Codex finding) | IN-SCOPE BLOCKER | Fixed — all documentation updated consistently |
| (Step 3 R3) Implementation Notes' own file-list sentence still said `dba-init.sh` scaffolds `status: disabled`, contradicting AC13's "no remaining stale claim" text | RVS__…__S3 | IN-SCOPE BLOCKER | Fixed inline — budget exhausted, escalated to human decision |
| (Human direction, post-Step-3) Manual status-line inclusion within CHG-A alone should not be accepted as this discipline's final operating model | — (human architectural direction, not a Codex finding) | IN-SCOPE BLOCKER | Resolved by scoping CHG-A honestly (pattern+wiring only) and deferring automatic delivery to a required CHG-B (`scripts/codeos-review.sh`, no Rust changes); UPG-0057 marked incomplete until CHG-B lands |
| (Step 4 R1) Post-R1-fixes historical note still asserted `dba-init.sh` scaffolds `status: disabled` without qualifying it as historical, contradicting AC13/Reconciliation's "no remaining claim" | RVS__…__S4 | IN-SCOPE BLOCKER | Fixed — note now explicitly marked as a historical record of Step 3's earlier state, not a current-default claim |

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
| — | — | — |
