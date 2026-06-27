# Upgrade: stack-drift-detector — Stack / Config Drift Detector

**Priority**: P3
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: stack-manifest, readiness-checklist, ci-profile

## Problem

Dependencies or config can change without manifest updates.

## Upgrade

Add a simple detector.

## Scope

Dependency/config drift detection at readiness/release.

## Proposed artifact(s)

A detector rule integrated with the readiness checklist.

## Design notes

Rule:

```text
If dependency/config files changed and readiness checklist does not include stack
reconciliation, block release.
```

## Value

Medium. Useful once stack manifest exists.

## Risk

False positives blocking release on benign config changes.

## Guardrail

Depends on `stack-manifest` existing; keep the watched-file set explicit.

## DBA-philosophy note

No behavioral rule touched. Operational release gate; depends on `stack-manifest`. Trigger-based,
not memory-based.
