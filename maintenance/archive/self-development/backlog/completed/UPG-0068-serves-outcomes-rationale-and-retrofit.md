---
feature_id: UPG-0068
slug: serves-outcomes-rationale-and-retrofit
title: No Retrofit Path for serves_outcomes on Pre-Charter Approved Intents
status: COMPLETE
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: serves-outcomes-rationale-and-retrofit

**Priority**: P2
**Status**: COMPLETE
**Type**: prompt

## Outcome (closed 2026-08-19)

Implemented in `dba/03-prompts/workflow/support-solution-charter.md`, with a one-sentence pointer in
`dba/03-prompts/workflow/01-intent.md`. No template, artifact, doctrine section, or glossary term was
added.

**The open question is answered: metadata recording, approval intact.** Doctrine v3 already requires
the mapping — "Each Charter outcome has a stable identity. An Intent records the outcomes it serves"
— so supplying it on a pre-Charter Intent restores conformance rather than deciding something new.
Approval in DBA-3 is semantic, not byte-level: it holds while the artifacts "continue to represent
the jointly approved specification", escalation triggers on changes to approved *behavior*, and
"Existing approvals are not automatically invalidated."

The confirmation model reuses the existing decision boundary rather than adding one. For an Intent
approved before its Charter, `serves_outcomes` is proposed during Charter construction, listed per
Intent alongside the artifact so the human is visibly approving the mappings, and recorded only
after Charter approval — copying exactly the values presented, with the Intents' `status`,
`approved_by`, and `approved_at` untouched.

An Intent that cannot be truthfully mapped is reported unresolved and excluded from the set rather
than given a mapping to complete it; `approval: null` holds while an in-scope Intent cannot be
mapped, and the human resolves it by adding the missing outcome or explicitly placing the feature
outside the scope boundary.

The rationale-field half was dropped during the 2026-08-19 rescope and remains dropped.

## Problem

Two linked gaps around `serves_outcomes`, both encountered in PlotSpot on 2026-08-16 while adopting
DBA-3.

**1. The mapping is recorded; the reasoning is not.** The intent template carries
`serves_outcomes: [O-1]` as a bare list. Nothing records *why* a feature was judged to serve a given
outcome. Doctrine relies on this mapping to identify features affected by an outcome change — but
when that change arrives, the reassessment has no basis to work from and restarts from scratch. The
judgement that F-0003 serves O-3 because it identifies when a measured finding cannot support a
conclusion is exactly the kind of reasoning a later reader needs and currently cannot get.

**2. There is no workflow for Intents approved before their Charter existed.**
`support-solution-charter.md` routes forward — "route feature work to `01-intent.md`, recording the
outcomes each Intent serves" — and `01-intent.md` covers an Intent being written now. Neither covers
approved Intents that predate the Charter. DBA-3 makes this universal: any project adopting it with
existing approved packages must retrofit `serves_outcomes` onto artifacts whose approval it must not
disturb.

PlotSpot retrofitted six approved Intents by hand. It worked, but nothing defined whether adding the
field is a material change, whether approval fields stay untouched, or where the reasoning goes.

## Upgrade

Define the retrofit path for approved Intents that predate their Charter.

Open for Step 1, not settled here:

- Whether retrofitting `serves_outcomes` onto an approved Intent is a material change requiring
  reapproval, or metadata recording that leaves approval intact. PlotSpot assumed the latter.

**Rescoped 2026-08-19 — the rationale field is not adopted.** Problem 1 below asked where the
reasoning behind a `serves_outcomes` judgement should live. The answer is nowhere: a rationale field
invites prose duplicating the Intent, which this brief's own Risk section names as its trap and
`CLAUDE.md` forbids as duplicated authority. The bare mapping is judged sufficient, which this brief
already allowed for — "in which case the retrofit gap is the whole feature." That is what remains.
Do not reopen this without new evidence that an outcome-change reassessment actually failed for want
of recorded reasoning.

## Scope

**In scope:** the routing text in `dba/03-prompts/workflow/support-solution-charter.md`, which
currently routes only forward after approval; and the recording instruction in
`dba/03-prompts/workflow/01-intent.md`, which currently covers only an Intent being written now.

**Out of scope:** `dba/05-guidance/templates/intent.md` — no new field is added, so the template is
untouched; the rationale field (see Upgrade); the impact-tracing mechanism itself, which works; any
new traceability ledger, index, or synchronized record surface; changing what approval means.

## Value

Makes outcome-change impact assessment usable rather than merely possible, and gives DBA-3 adopters
a defined path for existing approved packages instead of improvisation.

## Risk

Low, with one real trap: a rationale field invites prose that duplicates the Intent. If adopted it
must stay to one line per outcome, or it becomes a second place to maintain feature purpose.
