---
component_question: How should an existing codebase be observed and bootstrapped into minimum DBA artifacts?
out_of_scope: Inventing requirements, approving new behavior, changing the codebase, and completing delivery stages.
---

# Existing Codebase Onboarding

## Purpose

Turn observed existing behavior and explicit human intent into normal draft inputs for the
Specification Package workflow. Onboarding is an entry path, not a separate lifecycle.

## Inputs

Read the current Feature Brief, Intent, and registry templates. Select at most three related modules
with the human. For each, inspect relevant source, tests, and available runtime evidence.

## Task

1. Summarize observed inputs, outputs, and handled failures. Clearly distinguish observation from
   intended behavior.
2. Interview the human for the beneficiary, meaningful outcome, required guarantees, and scope.
   Never promote code accidents or legacy tests into intent without confirmation.
3. Classify the observed module:
   - a feature or part of a feature → produce normal draft Feature Brief and Intent inputs;
   - shared infrastructure → record a short infrastructure note, not a feature.
4. Assign the normal feature identifier and register the feature using the current registry
   template. The templates own all exact fields, statuses, paths, and values.
5. Apply the current Intent prompt to the draft and name unresolved questions without inventing
   answers.

Do not change code, approve artifacts, create Contracts or Event Schemas, impose an Implementation
Profile, or register infrastructure as a feature.

## Output / Next Action

For each feature, present the normal draft Feature Brief, draft Intent, conforming registry entry,
and unresolved questions. Hand control to the ordinary Stage 1–3 Specification Package workflow.
