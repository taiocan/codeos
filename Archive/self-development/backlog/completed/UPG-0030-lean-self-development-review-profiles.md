---
feature_id: UPG-0030
slug: lean-self-development-review-profiles
title: Lean Self-Development Review Profiles
status: COMPLETE
priority: P1
depends_on:
  - UPG-0001
  - UPG-0029
related_features:
  - UPG-0028
supersedes: []
superseded_by: []
---

# Upgrade: lean-self-development-review-profiles — Lean Self-Development Review Profiles

**Priority**: P1
**Status**: COMPLETE
**Type**: self-dev-governance
**Related**: UPG-0001 (feature-thread-traceability), UPG-0028 (reviewer self-reference scoping),
UPG-0029 (review artifact durability)

> Filed during UPG-0029 (CHG-20260629-001) as the follow-up for the review-loop efficiency
> problem first observed there. UPG-0029 ran 6 Codex rounds on Step 3 alone before the human
> intervened. This upgrade fixes the process before the next change is implemented.

## Problem

The current self-dev loop provides no review-round budgets, no delta review mode, no local
pre-review checks, no claim-audit step, no review-profile assignment, and no named triage
category for the self-reference causal loop. Concretely:

1. **No round budgets.** Nothing in `prompts/codeos-self-dev.md` or `CLAUDE.md` says "after N
   rounds, escalate to human decision." UPG-0029 Step 3 ran 6 rounds; UPG-0001 Step 3 ran 8
   rounds; UPG-0001 Step 4 ran 7 rounds before a Codex cap forced the human to decide.

2. **No delta review mode.** R2+ reviews send the full context packet even when only one line
   changed. This wastes reviewer attention and token budget on unchanged material, and can
   cause the reviewer to re-flag already-resolved issues (or miss the tiny changed line).

3. **No local pre-review checks.** Deterministic failures — stale placeholders, false universal
   claims, mismatched trace-header fields, scope-boundary violations — could be caught by grep
   before invoking Codex, but currently aren't. UPG-0029 Step 3 R6 found a false "all" claim
   that a 30-second grep would have caught.

4. **No claim audit before review.** Universal quantifiers ("all", "every", "never", "always",
   "no X") in new or modified prose have been a recurring Codex-flagged false-claim source
   across UPG-0001 and UPG-0029. No guidance exists to audit these before sending.

5. **No review profile assignment.** All change classes that enter the 4-step loop run the
   same per-step review cadence with no round limit. (`backlog-only` and `trivial` are
   direct edits that never enter the loop. It is the looped classes — `documentation`,
   `template`, `prompt`, `script-tooling`, `downstream-doctrine`, `self-dev-governance` —
   that lack cadence differentiation and any ceiling.)

6. **No SELF-REFERENCE / REVIEW-BOOKKEEPING triage category.** The four existing categories
   (IN-SCOPE BLOCKER / IN-SCOPE NON-BLOCKER / OUT-OF-SCOPE BACKLOG / REJECTED) don't name
   the case where the reviewer flags that review records themselves are stale — a causal
   loop. The self-reference stop rule (two consecutive rounds find only bookkeeping churn →
   human decision) exists in `prompts/codeos-self-dev.md`, but no triage label matches it.

## Proposed solution

Add a **review profile** step (Step 0a) after triage that maps change class → review cadence.
Profiles are numbered 0–5 (lightest to heaviest). Update `prompts/codeos-self-dev.md` as the
primary operational home for all profile logic, local checks, claim audit, delta spec, and
budget table. Update `CLAUDE.md` minimally: add one cross-reference sentence after the triage
table, and update the §"Compulsory review" section to §"Review cadence and advisory verdict"
so it no longer states the blanket per-step review rule. Add supporting sections to
`docs/reviewer-pipeline.md`. Add a `review_profile` field to `templates/codeos-change.md`.

**Profiles:**

| Profile | Applies when | Cadence | Max rounds/step |
|---|---|---|---|
| PROFILE-0 | `trivial`; or `backlog-only` that stays a direct edit | No review, no loop | — |
| PROFILE-1 | `backlog-only` that escalates into the 4-step loop (changes accepted scope) | 1 review, at Reconcile only | 2 |
| PROFILE-2 | `documentation` (normative) | 1 review per step | 2 |
| PROFILE-3 | `template` / `prompt` / `script-tooling` | 1 review per step, R2+ delta | 3 |
| PROFILE-4 | `downstream-doctrine` | 1 review per step, R2+ delta | 3 |
| PROFILE-5 | `self-dev-governance` | 1 review per step, R2+ delta | 3 |

Human approval at each step transition is required at every profile. Profiles vary only
Codex review cadence and round budgets — not the human-gate safety invariant.

**Budget exceeded:** when the per-step round limit is reached, fix any remaining findings
inline and escalate to human decision. Do not run further Codex rounds automatically.

**Policy change declared:** UPG-0030 intentionally supersedes the current blanket
per-step advisory-review rule with profile-based review cadence. Human approval at each
step transition remains the sole gate at every profile; reviewer output remains advisory
and non-gatekeeping at every profile; high-risk profiles (PROFILE-3 through PROFILE-5)
retain per-step review; low-risk profiles (PROFILE-1, PROFILE-2) may limit Codex review
to Reconcile only or reduce the round budget — an intentional trade of review ceremony
for operational speed on simpler changes.

## Scope

**Self-dev only.** No changes to `dba-system.md` or `scripts/codeos-review.sh`.

## Files touched by CHG-20260629-001

See the Feature Thread below.

---

## Feature Thread

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260629-001 | changes/UPG-0030__CHG-20260629-001__lean-review-profiles.md | Add review profiles, delta mode, local checks, claim audit, round budgets, SELF-REFERENCE triage | COMPLETE |

### Reviews

| Review series | Step | Verdict summary |
|---|---|---|
| RVS__UPG-0030__CHG-20260629-001__S1 | 1-Intent | ACCEPTED (3 rounds; budget exhausted; human design decisions on bundling and CLAUDE.md scope) |
| RVS__UPG-0030__CHG-20260629-001__S2 | 2-Acceptance | ACCEPTED (3 rounds; budget exhausted; inline fixes) |
| RVS__UPG-0030__CHG-20260629-001__S3 | 3-Implement | ACCEPTED (3 rounds; budget exhausted; inline fixes incl. CLAUDE.md + docs/reviewer-pipeline.md §2 taxonomy) |
| RVS__UPG-0030__CHG-20260629-001__S4 | 4-Reconcile | ACCEPTED (3 rounds; budget exhausted; inline fixes incl. reviewer-pipeline.md scope and binding: field) |

### Findings

OUT-OF-SCOPE BACKLOG: UPG-0027 workspace change appeared in diff (pre-existing from prior session; restored; not a UPG-0030 finding). No findings filed as new UPG-#### items.

### Follow-up

(none yet)
