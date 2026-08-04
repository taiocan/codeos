# Codeos DBA System: Lean Proposal

Status: alternative downstream doctrine for evaluation. This file does not replace
`dba-system.md`.

## Purpose

DBA turns a human-approved outcome into working, observable software. It uses a small
set of approved artifacts to keep behavior clear while leaving normal implementation
choices to the engineer or agent.

The goal is delivery with traceability. Process is useful only when it prevents a
material mistake.

## Working Rules

1. A human must approve Intent, Contract, and Event Schema. Approval of the Event Schema
   authorizes the agent to run Stages 4 through 8 as one delivery cycle.
2. The agent must ask before a production action, destructive operation, irreversible data
   change, or other action for which the environment requires human control.
3. Approved artifacts control externally visible behavior. Runtime evidence shows what
   the system does; it does not silently change what the system should do.
4. Implementation may use normal internal abstractions, technical errors, logs, and
   project patterns. These choices must not add or change observable feature behavior.
5. The system may emit only approved domain events. Diagnostic logs and metrics are not
   domain events, but they must not create hidden outcomes or expose protected data.
6. Do not invent a missing product decision. Ask the human when the missing decision can
   change behavior, safety, authorization, data integrity, or architecture.
7. Make the smallest change that satisfies the approved behavior. Do not add speculative
   capabilities.
8. Previously approved content remains approved until its meaning changes. A material
   change reopens only the changed stage and the dependent work affected by that change.

The human may request an extra checkpoint at any time. Otherwise, do not stop between
implementation, tests, observation, reconciliation, and replay.

An approval applies to the artifact or batch just presented. Plain statements such as
`approved` or `approve all Stage 2 drafts in this cohort` are sufficient. Do not require
ceremonial wording or duplicate approval records. After Stage 8, the human accepts the
feature or requests refinement.

## Authority

Use this order when sources conflict:

1. The human's current explicit decision.
2. Approved Intent, Contract, Event Schema, and applicable Architecture decisions.
3. Runtime evidence and tests for claims about current behavior.
4. Code and project conventions for internal details not fixed above.

Runtime evidence never overrides a safety, authorization, or data-integrity requirement.
If the conflict cannot be resolved without changing approved meaning, stop and present
the decision in one short question.

## Before Work

Read only the context needed for the current task:

- project instructions and current feature status;
- the current stage artifact and its direct approved inputs;
- applicable architecture decisions;
- the relevant code, tests, diff, and runtime evidence.

Do not load every project document, old review, or full history by default. Do not repeat
this doctrine in chat or in review packets.

Stop after each of Stages 1, 2, and 3 for approval. After Stage 3 approval, continue
through Stage 8 until the delivery result is ready, a material product decision is
blocked, or a controlled action requires the human.

## Optional Discovery

Solution Discovery may run before Stage 1 when feature boundaries or shared architecture
are unclear. It produces candidates, risks, and open questions. It is not an approved
behavior or architecture artifact.

A Feature Brief is also optional. Use one only when it helps decide whether a candidate
should enter Stage 1. Review discovery material only when a claim from it is carried into
an approved artifact.

For an existing codebase, inspect the current structure and behavior before Stage 1. Save
a codebase digest only when it will be reused; otherwise report the relevant findings in
the current work.

## The Nine Stages

### Stage 1: Intent

State the actor, desired outcome, stable guarantees, scope, exclusions, and unresolved
product decisions. Do not choose implementation details.

Output: `intents/[feature_id].md`.

### Stage 2: Behavioral Contract

Turn the approved Intent into observable scenarios. Cover the normal outcome, material
alternatives, boundary cases, contracted failures, and invariants. State only behavior
that can be tested through a public interface, state, or approved event.

Technical failures do not need product classifications unless the product must expose
them. Do not create cases only to fill a template.

Output: `contracts/[feature_id]_contract.md`.

### Stage 3: Event Schema

Define the domain events needed to observe or reconstruct the contracted behavior. Each
observable outcome must be provable by state, an event, or both. Several scenarios may
use the same event. A failure needs an event only when the approved behavior requires one.

Define event names, meaning, required fields, and valid ordering. Require correlation only
where two or more events must be linked.

Output: `events/[feature_id]_schema.md`.

### Stage 4: Implementation

Implement the approved behavior using the project's architecture and engineering
conventions. Internal helpers, validation, error types, and refactoring are allowed when
they are the simplest maintainable way to deliver the contract.

Run formatting, compilation, static checks, and existing tests. If implementation exposes
a missing behavior decision, return to the stage that owns that decision.

Output: working code. Do not create an implementation report unless evidence cannot be
understood from the code, diff, and command results.

### Stage 5: Tests

Test the public behavior, contracted failures, important boundaries, invariants, and event
conformance. Test internal units only when they contain meaningful logic; do not couple
behavioral tests to private structure.

Output: executable tests. Report only the commands run, results, and material gaps.

### Stage 6: Runtime Observation

Run representative scenarios and capture real runtime evidence. The agent may run them
when the environment permits. Never fabricate runtime evidence.

If the project uses `events/runtime_events.jsonl`, append new observations and never edit
old observations. A separate observation report is optional; create one only when the
evidence needs durable explanation.

### Stage 7: Reconciliation

Compare Intent, Contract, Event Schema, code, tests, and runtime evidence. Report only
gaps, mismatches, missing evidence, and the evidence that supports the conclusion. If all
layers align, a short conclusion is enough; do not produce a large table of aligned rows.

Route each problem to its earliest owning stage:

| Problem | Return to |
|---|---|
| Desired outcome or scope is wrong | Stage 1 |
| Observable rule or failure is wrong | Stage 2 |
| Event meaning or shape is wrong | Stage 3 |
| Code does not satisfy approved behavior | Stage 4 |
| Tests are wrong or incomplete | Stage 5 |
| Runtime evidence is missing | Stage 6 |

### Stage 8: Replay Verification

Replay or reprocess the captured event stream. Verify schema conformance, valid order,
and correlation where required. Verify deterministic results only when the Contract or
architecture requires determinism.

Output: the command and result, plus material failures. Do not restate the full event log.

### Stage 9: Targeted Refinement

For each observed problem, fix the earliest owning stage and only the affected downstream
work. Unchanged approved artifacts remain approved. Re-run the checks needed to prove the
fix; do not automatically replay the full workflow. A correction within approved behavior
may proceed without another product decision. A change to approved behavior must return to
Stage 1, 2, or 3 and receive approval there.

If no problem remains, record that no refinement is needed and close the feature.

## Multi-Feature Architecture Gate

Use this gate when two or more features could constrain each other's ownership,
dependency direction, persistence boundary, integration contract, shared infrastructure,
or deployment topology. Sharing a runtime or database alone does not trigger the gate.

The gate runs after the cohort's Stage 3 artifacts are approved and before any cohort
member enters Stage 4:

1. Check the cohort for material contradictions and unclear ownership.
2. Draft a Core Architecture Baseline for project-level decisions.
3. Draft a Cohort Logical Design for shared identity, interfaces, transactions, event
   ownership, persistence, and migration decisions that implementation needs now.
4. Present both drafts for one human approval.

The two documents must contain decisions, reasons, constraints, and open risks. They must
not repeat every feature artifact or invent behavior. The Logical Design must not repeat
the Baseline. Git history records superseded versions; duplicate history files and complex
registry states are not required.

Features in a cohort may be drafted and reviewed in same-stage batches. Approval may also
be given as one explicit batch decision, provided the human can identify every included
artifact.

## Review Policy

The acting agent performs a direct self-check before every gate. Independent review is
not required at every stage.

Use an independent review when at least one condition is true:

- the human asks for it;
- the Multi-Feature Architecture Gate is ready for approval;
- the change crosses a security, authorization, privacy, financial, migration, or
  irreversible data boundary;
- the acting agent has a material unresolved concern after its self-check;
- Stage 7 or Stage 8 finds a material mismatch.

A review receives only the changed artifact, its direct approved inputs, and the evidence
needed to assess the change. Prefer the diff. Do not send the full repository, unrelated
artifacts, unchanged review history, or this full doctrine.

Use one review pass by default. Use one focused retry after a material fix. The human then
decides whether to approve, revise, accept a known risk, or stop. A reviewer advises; it
does not add requirements or control the gate.

Save a review only when its decision changes behavior, architecture, or an accepted risk.
For ordinary corrections, the artifact diff and git history are enough.

## Lean Artifacts

Required feature artifacts are Intent, Contract, Event Schema, code, tests, runtime
evidence, reconciliation, and replay evidence. Architecture artifacts are required only
when the architecture gate applies. Everything else is optional.

Drafting targets are 500 words for Intent, 1,500 words for Contract, and 1,500 words for
Event Schema. These are not approval limits. If an artifact needs more space, first check
whether the feature should be split; otherwise state why the extra detail is necessary.

State each fact or decision once and link to it elsewhere. Do not maintain the same status,
decision, or explanation in several files. A multi-feature project may keep a small feature
index with only feature ID, last approved stage, status, and next action.

## Writing Rules

- Use short sentences and common words.
- Use `must` for a requirement, `should` for a recommendation, and `may` for permission.
- Preserve exact event names, field names, enum values, commands, and quoted requirements.
- State exact bounds when they matter.
- Use a decision table only when it is clearer than prose.
- Mark unknown values as unresolved. Do not guess.

## Structural-Only Changes

A change that cannot alter product behavior does not need the nine stages. State the
structural goal and constraints, get approval to implement, make the change, run relevant
checks, and reconcile the result. If the change can alter an observable outcome, use the
nine-stage workflow and return to the earliest affected stage.
