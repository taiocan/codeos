---
component_question: How does a solution get from an approved Charter to a runnable, integrated, human-previewed product baseline before its first feature enters the delivery cycle?
out_of_scope: Feature behavior and checkpoints, operational routing, architecture synthesis, and how the workflow checker evaluates a predicate.
---

# Solution Bootstrap

<!-- DOCTRINE ADAPTER: bootstrap-entry -->

## Purpose

Carry a solution from an approved Solution Charter to a runnable, integrated, minimally verified
product baseline that a human has previewed — before any feature narrows attention to one
capability. This support workflow owns the `bootstrap-entry` boundary: a solution's first
Specification Package MUST NOT enter the bounded delivery cycle until Solution Bootstrap is
complete. Later features do not repeat it.

Bootstrap does not design the future product. It builds no feature-specific schema, no complete
navigation, no component library, and no speculative infrastructure. It answers one question: can
Codeos now deliver the first real feature through the complete solution.

## Inputs / Prerequisites

Read the selected doctrine and the selected Workflow Governance and Solution Bootstrap policies.
The Bootstrap checkpoints B1–B5, their evidence, and their routes are owned by the Solution
Bootstrap policy; the common state-derivation and receipt mechanism is owned by Workflow
Governance.

Semantic authority is unchanged. B1 rests on the existing `purpose-approval` decision (the approved
Charter); B2 on the resolved Platform Baseline and a valid `.codeos/00-project/codeos.yaml`. This
workflow verifies that evidence is present and adds no approval of its own.

## Task

Use the workflow checker to derive Bootstrap state and to record the one human decision it needs:

```bash
codeos-workflow status --workflow bootstrap --subject solution
codeos-workflow check  --workflow bootstrap --subject solution
```

- `status` / `next` report each checkpoint as PASS, BLOCKED, or WAITING, read-only.
- `check` executes B3's integrated baseline against the real stack and, on a full pass, writes a
  mechanical verification record. It never records a decision.
- When the running minimal product has been presented to the human, record their direction call as
  the Initial Product Preview decision receipt in `.codeos/06-workflow/decisions.jsonl`:

  ```bash
  codeos-workflow decide --workflow bootstrap --subject solution \
    --checkpoint initial_product_preview --result direction_confirmed
  ```

- If the human identifies a solution-purpose or architecture problem, route it through the existing
  Charter or Architecture Synthesis revision mechanisms. Bootstrap does not resolve it.

## The Bootstrap Gate

While Bootstrap's checkpoints are not all PASS, a solution's first Feature Development workflow
cannot enter its first checkpoint. The workflow checker enforces this as the `bootstrap-entry`
adapter: `01-intent.md` and the delivery cycle for a solution's first feature proceed only once
`codeos-workflow status --workflow bootstrap --subject solution` reports every checkpoint PASS.
Once any feature has been accepted, Bootstrap has by definition already happened and the gate is
satisfied.
