---
component_question: How should an existing codebase be observed and bootstrapped into minimum DBA artifacts?
out_of_scope: Inventing requirements, approving new behavior, changing the codebase, and completing delivery stages.
---

# Existing Codebase Onboarding

## Purpose

Turn observed existing behavior and explicit human intent into normal draft inputs for the
Specification Package workflow. Onboarding is an entry path, not a separate lifecycle.

## Inputs / Prerequisites

Read the current Intent template through `.codeos/toolkit/`. Select at most three related modules
with the human. For each, inspect relevant source, tests, and available runtime evidence.

## Task

1. Summarize observed inputs, outputs, and handled failures. Clearly distinguish observation from
   intended behavior.
2. Interview the human for the beneficiary, meaningful outcome, required guarantees, and scope.
   Never promote code accidents or legacy tests into intent without confirmation.
3. Classify the observed module:
   - a feature or part of a feature → produce normal draft Intent inputs;
   - shared infrastructure → record a short infrastructure note, not a feature.
4. Assign the next feature identifier by scanning existing Intent, Contract, Event Schema filenames
   and architecture-scope membership. Existing partial packages are valid; stop only on a genuine
   identity conflict.
5. Apply the current Intent prompt to the draft and name unresolved questions without inventing
   answers.

Do not change code, approve artifacts, create Contracts or Event Schemas, impose an Implementation
Profile, or register infrastructure as a feature.

## Output / Next Action

For each feature, create the draft Intent at
`.codeos/01-specification/intents/<feature-id>.md`, present it with unresolved questions, and hand
control to the ordinary Stage 1–3 Specification Package workflow.
