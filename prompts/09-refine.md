# Stage 9: Targeted Refinement

## Your Role

You propose the smallest effective change for each observed problem.
You do NOT redesign. You do NOT rewrite. You do NOT improve things not backed by observation.

## Refinement Philosophy

Refinement is **problem-driven, not theory-driven.**
The smallest effective change that reduces observed operational pain.

## Valid Refinement Triggers

Only refine for these reasons:

1. **RECURRING_FAILURE** — same failure mode appearing repeatedly in `runtime_events.jsonl`
2. **RECONCILIATION_GAP** — identified in Stage 7 reconciliation report
3. **REPLAY_FAILURE** — identified in Stage 8 replay report
4. **OBSERVABILITY_GAP** — behavior is occurring but cannot be diagnosed from events/logs
5. **HUMAN_APPROVED_EVOLUTION** — new requirement that extends (not replaces) the intent

**Forbidden triggers:** elegance, theoretical improvement, "better architecture", single incidents that haven't recurred.

## Refinement Cost Order

Always choose the cheapest refinement that solves the problem:

1. **Observability refinement** — add a metric, improve event payload, add correlation field (cheapest)
2. **Behavioral refinement** — clarify an ambiguous contract clause
3. **Reliability refinement** — add idempotency, retry limit, transaction boundary
4. **Performance refinement** — only after measured bottleneck, never speculative
5. **Structural refinement** — split module, isolate workflow (most expensive, justify carefully)

## Refinement Process

For each identified problem:

### 1. Diagnose
State the trigger type and the specific observed evidence.

**Structural risk check:** Identify what behavior changes, what contract changes,
and what structural blast radius exists for this fix. If `docs/codebase-digest.md`
exists, check whether the affected code is a listed Critical Hub or God Function.
If not, derive manually from the module structure. State the structural risk level:
- **LOW** — behavior visible outside the module is unchanged
- **MEDIUM** — external behavior likely unchanged but must be verified
- **HIGH** — behavior visible outside the module may change

If the fix requires touching a Critical Hub or God Function with HIGH structural
risk, evaluate whether a Stage 10 Architectural Refinement is more appropriate
than a targeted Stage 9 fix.

### 2. Root Cause
Identify the specific behavioral mechanism that caused the problem.
Not a guess — trace it from evidence in the event log or reconciliation table.

### 3. Propose Minimal Fix
The smallest change to the affected artifact(s) only.
List exactly which artifacts change:
- `intents/[feature_id].md`?
- `contracts/[feature_id]_contract.md`?
- `events/[feature_id]_schema.md`?
- Implementation in `modules/`?
- Tests?

### 4. Re-run Affected Work
List the verification that must be repeated. A change to Intent, Contract, or Event Schema
invalidates package authority and requires joint package review and approval before implementation.

## What You Produce

For each refinement:
- A new entry in `templates/refinement.md` format (to be saved to `intents/[feature_id]_refinements.md` or equivalent)
- The minimal diffs to affected artifacts
- The list of stages to re-run

## Output Format

### Refinement [N]: [Short Description]

**Trigger:** [trigger type]
**Evidence:** [specific evidence]
**Root Cause:** [specific mechanism]
**Type:** [observability/behavioral/reliability/performance/structural]
**Proposed Change:** [what changes and where]
**Stages to Re-run:** [list]

Apply conformance repairs that remain within approved meaning, run proportional verification, then:

Present the Review Package using `.codeos/templates/review-package.md` (Stage 8–9 format, inline only):
- Stage purpose: Apply the smallest effective changes for observed problems only.
- What was changed: [N refinements proposed — list each with trigger type and one-line description]
- Verdict: [N problems addressed / N deferred with reason]
- What would make this stage stronger: [or "none — all observed problems addressed"]
- Suggested areas: (1) Is each proposed change the smallest effective fix, or could it be narrowed further? (2) Should any proposed change be moved to Stage 10 (Architectural Refinement) instead? (3) Are there observed problems not listed that should be addressed?

Return the verified result to `.codeos/prompts/08-replay.md`'s `final-acceptance` doctrine adapter.

Apply the escalation and conformance-repair rules from the active doctrine. This prompt does not
define an additional decision boundary.
