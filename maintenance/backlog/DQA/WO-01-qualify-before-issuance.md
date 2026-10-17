# WO-01 — Qualify Configurations Before Issuance

PROTECTED · Charter criteria 1

## Goal

Keep a configuration non-governing until its full release evidence is accepted.

## Work

- Add a candidate-configuration identity that grants no downstream authority.
- Run workflow, reviewer, configuration, compatibility, and pinned downstream replay against it.
- Issue `DBA-N` only after the replay acceptance criteria receive human acceptance.
- Keep every issued configuration and policy byte-immutable.

## Protected invariant

Only a human-approved immutable DBA configuration governs a project; qualification never grants
approval authority to a test or replay project.

## Acceptance criteria

1. Either residual found in the first DBA-27 replay prevents issuance.
2. No project adopts a numbered configuration merely to qualify it.
3. JamFree-specific behavior cannot enter universal policy through replay.
4. Existing supported configurations resolve unchanged.
