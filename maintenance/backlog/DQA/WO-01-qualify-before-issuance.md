# WO-01 — Qualify Configurations Before Issuance

PROTECTED · Charter criteria 1 · semantics approved 2026-10-18

## Goal

Keep a configuration non-governing until its full release evidence is accepted.

## Approved semantics

- **Selection identity.** A candidate is `maintenance/candidates/<name>.yaml`, written with the
  same keys as a DBA-N file. It lives outside `dba/00-entry`, so no project can resolve it, and it
  grants no authority.
- **Qualified semantic identity.** `qualified_candidate_sha256` is the sha256 of four things:
  - the candidate's selections;
  - the bytes of every file in its normative closure;
  - the CodeOS source digest (as computed by `build-qualified-release.sh`);
  - the replay-case definitions and goldens.

  Human approval binds this digest. Issuance recomputes it and refuses on any mismatch, which
  requires a new qualification and a new approval. A rerun of the tests never replaces the
  approved digest.
- **Qualification.** Qualification runs in a disposable toolkit copy, where the candidate is
  materialized as the next DBA-N. It runs the full suite and the compatibility matrix, then
  replays every replay case, each in a disposable clone of a pinned consumer commit. The real
  toolkit never holds an unissued numbered configuration. Qualification refuses a consumer's live
  working tree.
- **Normative closure.** Every normative semantic dependency reachable from an issued
  configuration is immutable under it. The closure contains:
  - the selected files;
  - every toolkit file they reference by path, followed transitively.

  Its hashes are recorded at issuance. Configurations issued before WO-01 get forward-only
  baselines; past edits are not rewritten.
- **Genericity.** Consumer facts exist only in compatibility and replay fixtures. No normative file
  or tool code branches on consumer identity. A consumer-name grep over doctrine, policies and
  prompts is a hygiene guard, not proof of genericity.

## Open decision (before any closure hash is frozen)

Selected policies reference mutable shared files: `dba/05-guidance/terminology.md`,
`reader-oriented-output.md`, review prompts, patterns and templates. The existing model therefore
does not give closure. The first step inventories the exact closure and recommends, for each file,
one of two treatments:
- normative, so it needs versioned successors;
- guidance, excluded by rule.

The human decides.

## Protected invariant

Only a human-approved immutable DBA configuration governs a project. Issuance succeeds only for the
exact qualified semantic snapshot the human approved. Qualification never grants approval authority
to a test or replay project.

## Acceptance criteria

1. Either residual found in the first DBA-27 replay prevents issuance. Two replay cases cover them:
   - JamFree `f94f3b6`, the F-0002 inheritance;
   - JamFree `71adabb`, the test-runner configuration.

   A DBA-27-equivalent candidate fails qualification and a DBA-28-equivalent candidate passes.
2. No project adopts a numbered configuration merely to qualify it.
3. JamFree-specific behavior cannot enter universal policy through replay.
4. Existing supported configurations resolve unchanged.
5. Editing a draft closure file after qualification makes issuance refuse.
6. Mutating an indirectly referenced dependency of an issued configuration fails the integrity
   check.
