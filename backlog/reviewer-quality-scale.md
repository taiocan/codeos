# Upgrade: reviewer-quality-scale — Reviewer Summary Quality Scale

**Priority**: P2
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: reviewer-decision-brief, reviewer-full-diff

## Problem

Reviewer outputs can be fluent but not useful.

## Upgrade

Require reviewer to label evidence quality.

## Scope

Reviewer agent output.

## Proposed artifact(s)

Evidence-quality scale embedded in reviewer output.

## Design notes

Proposed scale:

```text
A — Direct evidence from artifact/diff/test/runtime log
B — Strong inference from code and tests
C — Plausible but not directly proven
D — Speculative
E — Unknown / not reviewed
```

Every recommendation should include:

```markdown
Recommendation:
Evidence quality:
Most important uncertainty:
What human should inspect if time is limited:
```

> In the automated pipeline this is the optional `EVIDENCE: <A–E>` line. It is only "implemented"
> when the reviewer actually emits it; otherwise the log records `Evidence: not reported`.

## Value

High. Prevents reviewer from sounding more certain than the evidence supports.

## Risk

Reviewer ignores the scale or grades inconsistently.

## Guardrail

Concern level (what the reviewer thinks) and evidence quality (how well supported) are separate
axes — keep both.

## DBA-philosophy note

No rule touched. Improves reviewer honesty (separates conviction from evidence). Advisory only.
