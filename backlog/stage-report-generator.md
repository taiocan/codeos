# Upgrade: stage-report-generator — Stage Report Generator

**Priority**: P3
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: stage-4-6-reports

## Problem

Stage reports can become manual burden.

## Upgrade

Generate reports from git diff, test output, and runtime files where possible.

## Scope

Stage 4–6 report generation.

## Proposed artifact(s)

Generated Stage 4–6 report skeleton, filled automatically where possible.

## Design notes

Inputs:

```text
git diff
git status
test output
runtime_events.jsonl
approved artifact paths
```

Output: Stage 4–6 report skeleton, filled automatically where possible.

## Value

Medium. Useful after report templates prove valuable.

## Risk

Automation hides errors.

## Guardrail

Generated report must say what was inferred vs human/model-written.

## DBA-philosophy note

No rule touched, but automation must **label inferred vs authored** content so generated
evidence is never mistaken for verified human/model judgment.
