---
component_question: How should a DBA working session be initialized and oriented from repository state?
out_of_scope: Performing workflow stages, changing approved artifacts, making approval decisions, and implementation.
---

# DBA Session Start

## Purpose

Orient the session from current repository state and select the next applicable workflow. Do not
perform stage work from this prompt.

## Inputs

1. Read `.codeos/dba-system.md`, its active configuration, and the selected doctrine. Read another
   selected component only when it applies to the requested work.
2. Read the project `CLAUDE.md` and, if present, `docs/codebase-digest.md`.
3. Run:

   ```bash
   git branch --show-current
   git rev-parse --short HEAD
   git status --short
   ```

4. Identify the human's target feature or structural task. If `features/registry.yaml` contains a
   matching entry, read that entry and its referenced artifacts. Treat artifacts on disk as
   authoritative; report a target-specific disagreement without silently resolving it. Drafting
   artifacts ahead of the recorded stage is not by itself an error.

## Task

Classify the requested work and load one workflow:

- New feature discovery → `00b-feature-brief.md`
- Existing feature work → the applicable Stage 1–9 prompt
- Existing code without DBA inputs → `00c-onboarding.md`
- Optional domain exploration before a feature brief → `00a-solution-discovery.md`
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
- target feature/task and its current authoritative state;
- selected workflow and expected output;
- any concrete blocker or authority conflict.

Then stop and wait for the human to begin the selected work.
