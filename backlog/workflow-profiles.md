# Upgrade: workflow-profiles — Branch / PR / CI Workflow Profiles

**Priority**: P1
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: branch-helper, feature-registry, ci-profile, reviewer-full-diff

## Problem

You currently commit and push at the end of Stage 9. That is simple and often fine. But as
features grow, you may want better diff review, CI feedback, stage-level history, and
reviewer-agent access to the full diff.

## Upgrade

Define optional workflow profiles rather than one mandatory PR policy.

## Scope

Branch / PR / CI discipline. Optional, profile-based.

## Proposed artifact(s)

`docs/workflow-profiles.md`

## Design notes

**Profile A — Simple local flow.** Use when: solo work; small feature; low risk.

```text
Run Stage 1–9 locally.
Commit and push after Stage 9.
Optional reviewer pass before final commit.
```

**Profile B — One branch per feature** (default recommended profile).

```text
Create feature/<feature_id>.
Commit after each approved stage.
Open draft PR after Stage 3 or Stage 5.
Run CI as soon as implementation/tests exist.
Merge only after Stage 8 or Stage 9.
```

Benefits: stage history preserved; reviewer can inspect full diff; CI detects breakage
earlier; feature remains bounded.

**Profile C — Split PRs for risky/large features.** Use when: feature is large; multiple
subsystems touched; security/compliance risk; high architectural uncertainty; reviewer needs
smaller diffs.

```text
PR 1: artifacts only        (intent, contract, event schema)
PR 2: implementation + tests (code, behavioral tests, telemetry tests)
PR 3: runtime/replay evidence (sanitized fixtures, replay tests, reconciliation/replay reports)
PR 4: refinement            (only if Stage 9 required)
```

Reviewer-agent use — the reviewer agent should have access to: full diff; changed files;
approved artifacts; stage reports; runtime/replay evidence; CI output if available. Reviewer
should check:

```markdown
Diff scope:
Unrelated files:
Approved artifacts changed after approval:
Implementation matches contract:
Events match schema:
Tests match contract/failure modes:
Runtime evidence present:
Replay evidence present:
Risk of hidden behavior:
CI status:
Recommended decision:
```

## Value

High when projects become more serious.

## Risk

Too much process for small features.

## Guardrail

Make this profile-based, not mandatory. Your current Stage-9 commit/push remains valid for
small work.

## DBA-philosophy note

No rule changed. Explicitly **optional/profile-based** — making any PR cadence mandatory would
add ceremony without behavioral benefit. The one-commit-per-stage option in Profile B/C is
what later enables checkpoint-based rollback.
