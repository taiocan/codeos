---
feature_id: UPG-0076
slug: svelte-gui-verification-support
title: Svelte GUI Verification Support
status: COMPLETE
priority: P1
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: Svelte GUI Verification Support

## Problem

Codeos has advisory guidance for Svelte state and component communication, but no shared way to
obtain DBA-quality evidence from a rendered browser interface. PlotSpot and EvidenceAtlas are
approaching Svelte GUI work, while DBA-4 now requires acceptance evidence to observe the boundary
named by the claim. Without a browser-verification pattern, projects may repeat the F-0008 failure
mode: prove a field exists in a model without proving it is visible, or submit a changed input
without proving the visible governed result changed.

## Upgrade

Provide a narrow advisory Svelte GUI verification pattern:

```text
pure TypeScript logic        -> Vitest
browser component behavior  -> Vitest Browser Mode with Playwright
bounded end-to-end journeys -> Playwright Test
visual or spatial claims     -> screenshot evidence only when the requirement needs it
```

The pattern demonstrates how to satisfy DBA-4 at a browser boundary; it does not restate or own
DBA-4 semantics. It covers rendered required fields, visible loading/failure/empty/unknown/success
states, changed visible results, coordinated list-and-map behavior, and proof that a browser
performance harness exercises the governed behavior before its measurements are accepted.

Correct `svelte-state-and-components.md` so consultation follows an approved project architecture
choice of Svelte rather than an Implementation Profile supposedly resolving to a framework. Keep
generic state ownership and component architecture outside this upgrade unless they are directly
needed to produce or verify user-visible behavior.

Include a minimal, currently verified configuration recipe and an optional user-workflow-map
template. The map may record the actor, supported decision, actions, visible responses, evidence
needs, handoffs, and failure paths. It remains non-authoritative, unapproved, non-gating, and
optional for simple single-screen behavior.

## Scope

**In scope:** browser/component/end-to-end test boundaries; rendered acceptance evidence; visible
states; semantic-before-performance examples; a disposable verification fixture; the optional
workflow-map template; correction of the existing Svelte pattern's consultation trigger.

**Out of scope:** general Svelte development guidance; routing; CSS; UI/component libraries;
directory layout; application-wide state architecture; deployment topology; a generated scaffold;
new Codeos runtime dependencies; new DBA semantics; changes to approved downstream architecture.

## Acceptance

- The guidance assigns pure logic, browser component behavior, end-to-end journeys, and genuinely
  visual/spatial claims to the stated proportional test boundaries.
- Examples distinguish model state from rendered evidence and changed input from changed visible
  governed output.
- Performance guidance demonstrates the governed browser behavior before accepting timing evidence.
- The recipe is exercised successfully in a disposable Svelte fixture without committing an app
  skeleton or Node dependency tree to Codeos.
- The workflow map is explicitly optional and non-authoritative and creates no approval, lifecycle,
  review, or traceability obligation.
- Existing Codeos checks pass, and no active doctrine, policy, reviewer contract, or downstream
  project architecture changes.

## Deferred Triggers

- Propose a reusable scaffold only after both projects independently establish substantially
  identical stable configuration.
- If an approved architecture establishes TypeScript/Svelte as a persistent implementation layer
  and this causes repeated feature-level exceptions or cannot be represented cleanly, propose the
  smallest Implementation Profile change that represents it directly.

## Value and Risk

**Value:** turns the immediate GUI work into direct, repeatable browser evidence without making
Codeos own application architecture.

**Risk:** the guidance could drift into a general frontend framework. Apply one filter to every
addition: if it does not help build or verify user-visible behavior under DBA, leave it
project-local.

## Outcome

Completed on 2026-08-30 as advisory guidance only. Codeos now provides the proportional Svelte GUI
verification pattern, an optional non-authoritative workflow-map template, architecture-selected
Stage 4 routing, and a Stage 5 browser-verification reference. The existing Svelte pattern no
longer treats framework selection as an Implementation Profile decision.

The current recipe was exercised once in a disposable SvelteKit fixture using Svelte 5.56.1,
Vitest Browser Mode 4.1.11, `vitest-browser-svelte` 2.1.1, and Playwright 1.60.0. Type checking and
the complete fixture suite passed. Two intentional regressions confirmed that the evidence fails
when currency is absent from rendered output and when a changed criterion leaves the visible result
unchanged. The fixture and its Node/browser dependencies were not added to Codeos.

Permanent verification remains lightweight: the guidance contract checks the inventory,
workflow references, semantic examples, authority boundary, and absence of a routine live-browser
dependency. The complete `dba/04-tools/tests/run.sh` suite passes. No doctrine, policy, reviewer
contract, runtime dependency, or downstream project architecture changed.
