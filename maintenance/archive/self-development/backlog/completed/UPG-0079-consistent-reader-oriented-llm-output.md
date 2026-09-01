---
feature_id: UPG-0079
slug: consistent-reader-oriented-llm-output
title: Consistent Reader-Oriented LLM Output
status: COMPLETE
priority: P2
depends_on: []
related_features: [UPG-0057]
supersedes: []
superseded_by: []
---

# Upgrade: Consistent Reader-Oriented LLM Output

## Problem

Codeos AI instructions contained useful but duplicated opening advice and no canonical method for
reader-oriented progression or terminology use. Normal agents could be pointed at guidance without
being told to read it, while isolated reviewers and implementers could not see ambient project
instructions at all. This allowed technically correct prose to drift in terminology and structure
and made nominal guidance integration weaker than actual delivery.

## Upgrade

Add one advisory reader-oriented output component covering result-first openings, complex-output
previews, four proportional progression patterns, paragraph coherence, terminology behavior, and
semantic safety. Codeos terminology remains authoritative for Codeos/DBA concepts; canonical
project terminology remains authoritative for project-domain concepts. Missing recurring
project-specific terms use the existing terminology mechanism or are proposed explicitly when the
current workflow cannot authorize a change.

Normal-agent entry points explicitly instruct agents to read and apply the component. Isolated
reviewer and delegated-implementer requests receive the exact guidance, Codeos terminology, and
canonical project terminology when present. The canonical project glossary is optional when
absent; if it exists, it must be a safely readable regular file within the project repository or
the isolated invocation fails closed.

## Scope

**In scope:** one canonical guidance source; actionable normal-agent routing; explicit isolated
provider injection; terminology precedence and missing-term behavior; structural and integration
verification.

**Out of scope:** DBA semantics; a writing lifecycle, mode, score, linter, or approval gate;
style-based review findings; terminology metadata; provider-budget redesign; evidence clipping;
new serialized accounting fields; retrospective artifact rewriting.

## Acceptance

- Canonical terminology applies to all human-readable AI output; progression guidance applies when
  explanatory structure exists.
- The useful result-first and preview-before-detail behavior survives removal of duplicated prompt
  wording.
- Every normal-agent route gives an actionable read-and-apply instruction rather than a path-only
  reference.
- Reviewer and implementer inputs contain each automatically supplied communication source once,
  use only the canonical project-glossary location, and label terminology as context rather than
  evidence.
- A missing project glossary remains valid; an unsafe applicable glossary fails before provider
  invocation.
- Existing review-content budgeting, evidence coverage, clipping, provider token accounting,
  packet hashing, and serialized metadata remain unchanged.
- Retired writing-governance machinery does not return, and no prose-quality classifier or gate is
  introduced.

## Outcome

Completed on 2026-09-01. `reader-oriented-output.md` is the single active authority for Codeos AI
output structure and terminology use. Self-development and downstream normal-agent routes now say
to read and apply it. Reviewer packets and delegated implementation requests inject the exact
guidance and applicable canonical terminology, deduplicate sources resolving to the same file, and
fail before provider invocation when the canonical project glossary exists but is unsafe or cannot
be read.

Permanent checks cover the guidance inventory, opening behavior, four progression patterns,
terminology precedence, missing-term behavior, semantic and formal-syntax safety, actionable
normal-agent delivery, one-copy isolated delivery, canonical-path-only glossary discovery,
duplicate-source coalescing, fail-closed unsafe glossary handling, packet binding, and unchanged
review-content budgeting. The complete `dba/04-tools/tests/run.sh` suite passes.

No Doctrine, policy, active DBA configuration, reviewer contract, lifecycle, approval rule,
evidence-selection rule, budget semantic, clipping behavior, or serialized metadata changed.
UPG-0057 remains historical context only; none of its retired activation or writing-governance
machinery was restored.
