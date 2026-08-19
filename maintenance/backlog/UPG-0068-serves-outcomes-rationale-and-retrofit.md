---
feature_id: UPG-0068
slug: serves-outcomes-rationale-and-retrofit
title: No Retrofit Path for serves_outcomes on Pre-Charter Approved Intents
status: PROPOSED
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: serves-outcomes-rationale-and-retrofit

**Priority**: P2
**Status**: PROPOSED
**Type**: prompt

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
