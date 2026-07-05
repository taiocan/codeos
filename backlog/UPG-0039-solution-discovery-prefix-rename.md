---
feature_id: UPG-0039
slug: solution-discovery-prefix-rename
title: Resolve the 00b Prompt-Filename Collision (Discovery -> 00a)
status: PROPOSED
priority: P3
depends_on: [UPG-0007]
related_features: [UPG-0037]
supersedes: []
superseded_by: []
---

# Upgrade: solution-discovery-prefix-rename — Resolve the 00b Prompt-Filename Collision (Discovery -> 00a)

**Priority**: P3
**Status**: PROPOSED
**Type**: downstream-doctrine

## Problem

`prompts/00b-feature-brief.md` and `prompts/00b-solution-discovery.md` both use the `00b`
prefix, even though they are distinct, sequential pre-Stage-1 steps — Solution Discovery
(Session Type E) is upstream of Feature Brief (Session Type A), per UPG-0007's own design.
This collision predates UPG-0037 and was explicitly left untouched there (UPG-0037's scope
boundary stated "no prompt file renames" — it added a separate `discovery`/`brief`/
`onboarding`/`1`-`10` identifier vocabulary for reviewer invocation, but did not touch
filenames). Discovered via direct question during UPG-0037's close-out review.

## Upgrade

Rename `prompts/00b-solution-discovery.md` to `prompts/00a-solution-discovery.md`, since it
precedes Feature Brief in the actual workflow. Update the small number of live files that
reference the old path.

## Scope

The prompt filename itself, plus every **live** (non-historical) file that names it.
Historical/append-only records (completed change records, review-log entries, frozen Codex
assessments) are explicitly **not** touched — they correctly describe the filename that
existed at the time.

## Proposed artifact(s)

- `prompts/00a-solution-discovery.md` (renamed from `00b-solution-discovery.md`; no internal
  content change needed — the file doesn't self-reference its own filename).
- Updated `dba-system.md`: the Stage ID table's File column (`dba-system.md:153`).
- Updated `prompts/00-session-start.md`: Session Type E's "Prompt to load" reference
  (`prompts/00-session-start.md:114`).

## Design notes

**Files confirmed to need updating** (grepped directly, live references only):
1. `dba-system.md:153` — `| Solution Discovery ... | discovery | .codeos/prompts/00b-solution-discovery.md |`
2. `prompts/00-session-start.md:114` — `Prompt to load: .codeos/prompts/00b-solution-discovery.md`

**Open Step 1 decision, not resolved by this brief**: whether to also update
`backlog/UPG-0007-solution-discovery-00b.md:37`'s one internal path mention (in its own
"Proposed artifact(s)" section). Leaning toward yes, since that backlog brief's own content
should stay accurate — but its *filename* (`UPG-0007-solution-discovery-00b.md`) should
**not** be renamed regardless, since backlog filenames are stable historical identifiers
referenced by dozens of other files (feature ID is the permanent identity, not the slug).

**Explicitly not touched** (historical, must not be rewritten):
- `changes/UPG-0007__CHG-20260630-005__solution-discovery-00b.md` (and its filename/slug)
- `backlog/UPG-0008-config-discovery.md`'s completed Feature Thread row
- All `reviews/codex/*.md` frozen assessments and `reviews/review-log.md` entries
- UPG-0022's and UPG-0037's own completed change records

**Compatibility**: checked `/home/rimo/projects/FundFlow` (the real downstream adopter) for
any hardcoded reference to the old path — none found, so no downstream-project-side edit is
needed. `.codeos` being a symlink means the rename is live immediately; this is the same
compatibility posture as UPG-0036/UPG-0037.

## Value

Low-medium. Purely a naming-consistency fix — removes a real but cosmetic collision. No
behavior change.

## Risk

Very low. A single file rename plus two live-reference updates. Main risk is an incomplete
grep sweep leaving a stale reference somewhere.

## Guardrail

No behavior change — this is a rename plus reference updates only. Historical/append-only
records are never rewritten.

## DBA-philosophy note

Touches `dba-system.md` and a `.codeos/prompts/` file downstream projects load — classified
`downstream-doctrine`, same rigor tier as UPG-0007 and UPG-0037.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
