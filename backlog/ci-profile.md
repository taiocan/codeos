# Upgrade: ci-profile — CI Integration Profile

**Priority**: P3
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: workflow-profiles, stack-drift-detector

## Problem

Local evidence may not match CI.

## Upgrade

Define how Codeos evidence maps to CI checks.

## Scope

CI integration (optional).

## Proposed artifact(s)

Proposed CI checks:

```text
behavioral tests
replay tests
schema conformance tests
lint/typecheck
no unapproved event names
no raw runtime log leakage
stack manifest reconciliation if dependency/config changed
```

## Design notes

Maps existing Codeos evidence types (behavioral tests, replay, schema conformance) onto CI
gates so local and CI evidence converge.

## Value

Medium-high if you use CI heavily.

## Risk

CI maintenance overhead.

## Guardrail

Start with minimal checks.

## DBA-philosophy note

No rule touched. CI enforces existing evidence types (schema conformance, no unapproved
events) — reinforces, rather than alters, the behavioral chain.
