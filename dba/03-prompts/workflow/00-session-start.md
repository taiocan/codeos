---
component_question: How should a DBA working session be initialized and oriented from repository state?
out_of_scope: Performing workflow stages, changing approved artifacts, making approval decisions, and implementation.
---

# DBA Session Start

## Purpose

Orient the session from current repository state and select the next applicable workflow. Do not
perform stage work from this prompt.

## Inputs / Prerequisites

1. Read `.codeos/toolkit/dba-system.md`, its active configuration, and the selected doctrine. Read another
   selected component only when it applies to the requested work.
2. Read the project `CLAUDE.md` and, when it exists, the approved Solution Charter at
   `.codeos/00-project/charter.md`. If `.codeos/00-project/terminology.md` exists, read it as the
   canonical glossary for shared project-specific meanings. Its absence is valid.
3. Run:

   ```bash
   git branch --show-current
   git rev-parse --short HEAD
   git status --short
   ```

4. Identify the human's target feature or structural task. For a named feature, resolve matching
   Intent, Contract, Event Schema, and Architecture Scope types through the Downstream Project
   Layout Contract, then inspect those that actually exist together with relevant implementation
   and tests. A partially drafted Specification Package is normal: an Intent may exist before its
   Contract or Event Schema. Report only genuine identity or authority conflicts.

## Task

Classify the requested work and load one workflow:

- No approved Solution Charter yet, or a change to solution purpose, outcomes, scope, or a System
  Constraint → `00-charter.md`
- A post-acceptance Operational Observation → classify it and route to the earliest governed
  authority whose truth must change: Charter revision, the affected Specification Package, a new
  Intent, the applicable architecture policy, or `09-refine.md` for a conformance defect. Record it
  in the Learning Register only when it is material, unresolved, and not yet routable. An
  observation is evidence and never changes approved behavior by itself.
- New feature with a clear boundary → `01-intent.md`
- A problem requiring decomposition into several features → optional `00b-feature-brief.md`
- Existing feature work → the applicable Stage 1–9 prompt
- Existing code without DBA inputs → `00c-onboarding.md`
- Optional domain exploration before feature boundaries or Intents → `00a-solution-discovery.md`
- Optional whole-solution exploration before consequential commitments →
  `00-full-solution-concept.md`
- Project-level architecture decision → `03b-architecture-synthesis.md`
- Behavior-neutral, architecture-insignificant maintenance → the project's normal engineering
  process outside DBA

If an unresolved behavioral finding affects the target work, route it to the affected stage or
`09-refine.md`. If the work would establish or change a project-level boundary, apply the selected
architecture-synthesis policy.

## Output / Next Action

Report only:

- active configuration and doctrine;
- branch, commit, and whether the tree is clean or dirty;
- whether an approved Solution Charter exists;
- target feature/task and the relevant artifacts that currently exist;
- selected workflow and expected output;
- any concrete blocker or authority conflict.

Then stop and wait for the human to begin the selected work.
This prompt creates no durable project artifact.
