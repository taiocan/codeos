# Upgrade: config-discovery — Configuration Discovery and Configuration Schema Track

**Priority**: P2
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: solution-discovery-00b, stack-manifest

## Problem

Configuration requirements often appear across multiple features. If they are discovered too
late, implementation may hardcode behavior or introduce hidden environment assumptions.

## Upgrade

Add configuration discovery to expanded 00b and formalize config schema only when needed.

## Scope

Pre-Stage-1 discovery (inside expanded 00b); later formalization through Stages 1–3 / 10.

## Proposed artifact(s)

Proposed 00b section (see also `solution-discovery-00b`).

## Design notes

```markdown
# Candidate Configuration Requirements

Config name:
Purpose:
Feature(s) likely affected:
Default:
Required/optional:
Secret/non-secret:
Environment-specific:
Runtime-changeable:
Validation needed:
Possible failure mode:
Possible event impact:
```

Later formalization — if configuration becomes real, route it through:
- Stage 1–3 if config changes observable behavior;
- Stage 10 / ADR if config is structural or infrastructure-level;
- readiness checklist if config docs/examples need update.

## Value

High for multi-feature systems.

## Risk

Premature config architecture.

## Guardrail

00b config output is hypothesis only. Approved feature artifacts or ADRs decide.

## DBA-philosophy note

Touches **artifact authority**: config hypotheses are non-authoritative until routed through
approved Stages 1–3 or an ADR. Must not let environment assumptions enter implementation
untraced.
