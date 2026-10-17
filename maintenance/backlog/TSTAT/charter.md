# TSTAT — Compact Terminal Status Charter

- **Status:** IMPLEMENTED 2026-10-17 (requested by the human); awaiting acceptance
- **Class:** NORMAL; presentation only
- **Input:** `codeos-workflow status` on JamFree DBA-28

Planning context only. This Charter changes no workflow state or authority.

## Problem

The terminal report repeats full sentences, hides the project condition below long health details,
and renders row-shaped data as paragraphs. The user must scan the same stale reason repeatedly.

## Goal

Show project condition, exceptions, and next actions as compact terminal tables.

## Invariants

- `status --json`, derivation, exit codes, ordering, and authority remain unchanged.
- No state, reason, evidence, or executable next action is lost.
- Labels use canonical terminology; compact text does not weaken meaning.
- Plain text remains readable without colour or Unicode-width assumptions.

## Target structure

```text
CODEOS — JamFree | DBA-28 | UNQUALIFIED

SUMMARY
BOOTSTRAP  5/5   FEATURES  2 VALID · 4 STALE   ATTENTION  4

HEALTH
DOMAIN              STATE       BASIS
Product             CURRENT     Charter approved
Verification        STALE       F-0002–F-0005 · REVALIDATION_REQUIRED

FEATURES
ID       NAME                         STATE      POINT                         ACCEPTED
F-0002   Return departure decision    STALE      POST_ACCEPTANCE_REVALIDATION  2026-10-08

ATTENTION
SUBJECT  CLASS   CHANGED                   NEXT
F-0002   STALE   implementation_state      AUTO · revalidation
```

## Success criteria

1. Summary and exceptions appear before supporting detail.
2. Repeated causes are grouped or referenced once.
3. Routine rows use fragments, not explanatory sentences.
4. Full evidence and commands remain available in the same report.
5. Existing JSON consumers observe no change.

Work orders: [README](README.md).
