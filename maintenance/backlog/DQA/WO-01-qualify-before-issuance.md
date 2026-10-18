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

## Decisions (2026-10-18)

- **Closure is semantic only.** The closure holds the selected files, plus path-referenced
  normative files outside advisory patterns, templates, prompts and tool scripts, plus the glossary
  rows for terms the selected files use, each frozen per definition. Tool and runtime changes stay
  under the existing source and build identity. There is no execution-closure mechanism.
- **Replay observes the revalidation plan.** `status` cannot show the residuals: a DBA-27 overlay
  on JamFree `172282b` gives the same validity, reasons and surfaces as DBA-28. `check --revalidation
  --plan --json` serializes the plan that `check` itself executes. It is extracted once and executed
  by normal `check`, and it performs no writes, no verification processes and no provider calls.
  Its output is minimal: per checkpoint, the action (`reuse | rerun | blocked | decision_required`),
  the reused record identity, a stable reason code and the upstream subject. WO-03's
  environment-boundary command is not part of WO-01.
- The B3 `codeos_yaml` staleness that a configuration overlay causes is real adoption behavior. It
  stays in the goldens.

## Implementation steps

WO-01's single objective: no DBA configuration can be issued with unqualified or mutable
semantics. Each step has one invariant and a falsifier. There is no general fixture framework and
no WO-03 environment or provider behavior.

- **01A — Candidate issuance.** An unissued candidate has no authority. Issuance publishes only the
  exact human-approved `qualified_candidate_sha256`.
- **01B — Semantic immutability.** Only the normative semantic closure is frozen. Tools keep the
  existing source and build identity.
- **01C — Read-only revalidation plan.** `check --revalidation --plan --json` serializes the plan
  that `check --revalidation` executes, and does no work.
- **01D — Qualification replay.** Real JamFree snapshots detect broad drift. Exactly two generic
  lifecycle fixtures reproduce the residuals that history cannot: inheritance from a pre-bundle
  upstream, and a test-runner change under web-scoped evidence. Both are observed through `status`
  and `check --plan`.

| Step | Status |
|---|---|
| 01A | Pending |
| 01B | Implemented |
| 01C | Implemented |
| 01D | Pending |

## Protected invariant

Only a human-approved immutable DBA configuration governs a project. Issuance succeeds only for the
exact qualified semantic snapshot the human approved. Qualification never grants approval authority
to a test or replay project.

## Acceptance criteria

1. Either residual found in the first DBA-27 replay prevents issuance. Revalidation-plan replay
   cases cover the F-0002 inheritance and the `71adabb` test-runner configuration: a
   DBA-27-equivalent candidate fails qualification and a DBA-28-equivalent candidate passes.
2. No project adopts a numbered configuration merely to qualify it.
3. JamFree-specific behavior cannot enter universal policy through replay.
4. Existing supported configurations resolve unchanged.
5. Editing a draft closure file after qualification makes issuance refuse.
6. Mutating an indirectly referenced dependency of an issued configuration fails the integrity
   check.
7. `--plan` makes zero writes, launches no verification process, and makes zero provider calls.
   Normal `check` consumes the same plan value, and a test proves the plan and the execution cannot
   diverge.
