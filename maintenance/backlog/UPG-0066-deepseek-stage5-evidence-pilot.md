---
feature_id: UPG-0066
slug: deepseek-stage5-evidence-pilot
title: DeepSeek Stage-5 Evidence Pilot
status: IN_PROGRESS
priority: P2
depends_on: [UPG-0060]
related_features: [UPG-0064]
supersedes: []
superseded_by: []
---

# Upgrade: DeepSeek Stage-5 evidence pilot

## Decision

Test one narrow hypothesis before adding any new delegation role or workflow integration:

> DeepSeek produces net-useful Stage 5 candidates when failure is cheap and independent verification
> costs less than primary-model test generation.

The durable selection rule is task-shaped, not model- or stage-shaped. This feature does not establish
permanent DBA-to-model roles.

## Current Change

Make the existing disabled experiment compatible with the current DeepSeek V4 API, account for
cache, reasoning, and termination usage, and make Stage 5 instructions applicability-driven. Preserve
the existing role-labelled packet, staging boundary, manual promotion, and single-shot invocation.

No downstream DBA doctrine, model router, benchmark framework, experiment registry, automatic cost
calculator, automatic retry, Stage 7 integration, or automatic promotion is added.

## Stage 5 Pilot

Use three realistic downstream features with approved Specification Packages and independently
authored Stage 4 implementations. For each feature, produce two isolated results from the same
approved package, implementation, and repository state:

1. Direct Stage 5, without access to the DeepSeek candidate or its corrections.
2. A DeepSeek candidate followed by independent verification/correction, without access to the
   direct-path tests or findings.

The Contract and Event Schema own expected behavior. Implementation is supporting context only for
public interfaces and mechanical integration. Generate tests for all applicable approved behavior;
do not manufacture event or replay requirements for external-observation features.

Record only:

- correctness — missed requirements, incorrect tests, private coupling, unapproved assumptions, and
  applicable falsifier or mutation evidence;
- usefulness — usable candidate content, local correction, or rewrite;
- supervision — retries, corrections, primary-model effort, and human effort;
- cost — actual DeepSeek and supervising-model usage and cost.

The pilot is positive only when there is no material correctness loss, at least two candidates are
usable without rewrite, and aggregate delivery cost or effort is materially lower than the direct
path. A 30% supervising-token reduction is a target, not a gate. Mixed or ambiguous evidence stops
progression.

## Termination and Follow-up

`finish_reason: stop` is required for a valid candidate. After `length`, the operator may make one
explicit 65536-token retry; after `insufficient_system_resource`, one identical retry. Every attempt
counts toward cost. Repeated truncation, repeated resource failure, or `content_filter` makes that
case unusable.

Stage 7 remains a one-shot, read-only experiment contingent on an explicit positive Stage 5 decision.
UPG-0064's bounded Stage 4 pilot remains deferred pending later evidence and a separate human
decision.

The three-feature pilot requires eligible downstream repositories and therefore remains pending; no
such Specification Packages are present under the current local projects workspace.

## Compatibility Evidence

On 2026-08-14, a real `deepseek-v4-flash` Stage 5 smoke request against a synthetic
external-observation feature completed with `finish_reason: stop`, staged one candidate test, and
that test passed. Usage accounting reported 2,805 prompt, 7,246 completion, 6,715 reasoning, 0
cache-hit, and 2,805 cache-miss tokens. This establishes API and protocol compatibility only; the
synthetic run is not Stage 5 quality or net-benefit evidence.
