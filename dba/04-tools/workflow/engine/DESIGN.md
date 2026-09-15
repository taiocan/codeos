---
module: workflow-engine
verified_against_commit: pending DBA-10 pilot
---

# Module Design Note: workflow engine

<!--
Descriptive documentation of how `dba/04-tools/workflow/engine` currently works. Explanatory only:
the Workflow Governance policy and the three workflow contract policies selected by the active DBA
configuration are the authoritative external interface, and the code is the truth about actual
implementation. If this note disagrees with either, the note is stale and gets corrected.
-->

## Purpose

Derive each checkpoint's state for one workflow subject from real evidence, block a later checkpoint
while a required earlier one is unresolved, and report the exact missing condition with one next
action. The engine owns state derivation, the verification-record and decision-receipt stores,
staleness detection, and live execution of the mechanical verifications.

It does not own semantic judgment. It never decides whether a product direction is good, whether a
user interface is understandable, whether evidence is persuasive, whether a Contract requirement is
correct, or which operational route an observation belongs to. Those stay with humans, agents, and
the reviewer. It adds no lifecycle stage, approval, or source of product authority.

## How it works

State is derived from three evidence classes and nothing else:

1. **Canonical state** — approved artifacts and their approval metadata, `codeos.yaml` validity, the
   reviewer's own assessment records, and binding currency. Cheap deterministic predicates,
   re-evaluated live on every query. `codeos.yaml` validity shells out to the authoritative
   `project-config-contract.sh` when the toolkit is reachable and falls back to a structural check.
2. **Verification records** — `06-workflow/verifications.jsonl`. Passing mechanical verifications
   are persisted with bound inputs, command, and timestamp. Under Workflow Governance v2, failed
   product-journey attempts are also retained with Codeos-derived transition definitions and
   validated runtime observations so the first broken seam remains visible. Neither result judges
   adequacy; drift makes it stale.
3. **Decision receipts** — `06-workflow/decisions.jsonl`. The closed set of seven irreducible human
   or authorized-agent decisions (Initial Product Preview, Early Development Preview outcome,
   Reconciliation completion, Final Human UX Validation, Acceptance, Operation route, `no_action`
   closure). Append-only, invalidated by binding drift, never a governing artifact.

The command paths:

1. `main.rs` parses the CLI, `Project::discover` walks up from `--project` (default `.`) to the
   directory holding `.codeos/` (downstream) or `dba-system.md` + `dba/00-entry` (self-development),
   resolves the adopted Feature Workflow policy, Cargo's workspace/package evidence, and the
   optional Runtime Verification Profile, and
   `Subject::resolve` reads the Feature Impact Accounting table and GUI-visible-outcome marker from
   the Contract to fix `gui` / `persistence` applicability. Without a runtime profile, the shipped
   skeleton's existing service and endpoint defaults remain in effect.
2. `status` and `next` call `checker::evaluate`, which is strictly read-only. For a Feature subject
   with no prior acceptance receipt it first synthesizes a `bootstrap-entry` report by evaluating
   the Bootstrap subject and checking B1–B4. It then walks the contract in order: a non-applicable
   checkpoint is `n/a`; once any checkpoint is not `PASS`, every later one is `BLOCKED` behind it;
   otherwise `evaluate_checkpoint` resolves each `Requirement` against the three evidence classes.
   A checkpoint with unmet conditions is `WAITING` when the only thing missing is a human decision,
   `BLOCKED` otherwise.
3. `check` calls `run_checks`, which loops: `evaluate` read-only, find the first `BLOCKED` /
   `WAITING` checkpoint, stop if it is blocked behind an earlier checkpoint or waits on a
   non-mechanical condition, otherwise run each of its `LiveVerification` mechanics whose record is
   absent or stale. On a pass it appends a verification record bound to the current inputs; on a
   DBA-10 product-journey failure it also appends the failed attempt, then stops. It never writes a
   receipt.
4. `decide` computes the bindings the checker will later re-check for that checkpoint, adds the
   observation hash for an `operation_route` / `no_action_closure` receipt, and appends one receipt.
   The store rejects an out-of-set checkpoint and a `no_action_closure` with no rationale. Under
   Feature Workflow v2, acceptance first re-evaluates every preceding checkpoint so a competing
   acceptance cannot establish a branched product-journey frontier.

The three contracts as they flow end to end:

- **Bootstrap** — B1 Charter approval, B2 `codeos.yaml` validity, B3 the integrated `baseline`
  verification (`check` brings the Docker stack up, proves a clean migration and DB↔backend↔GUI
  reachability, runs the shipped tests and Playwright journey, tears down; the record binds to
  `codeos.yaml` only), B4 the Initial Product Preview receipt (bound to the Charter and
  `codeos.yaml`, never to working-tree state), B5 derived from B1–B4.
- **Feature v1** — F1 Specification Package approval, F2 Feature Impact Accounting, F3 the `smoke`
  verification, F4 the Early Development Preview receipt where the Contract has a GUI-visible
  outcome, F5 `behavior` + `repeatability` (plus F5d `data_integrity` when persistence changed and
  F5g `playwright` when GUI), F6 the Reconciliation completion receipt (`completed` regardless of
  honestly recorded gaps), F7 the reviewer's record bound to the current commit, F8 the Final Human
  UX Validation receipt, F9 the Acceptance receipt. Every feature verification binds to the
  Specification Package and implementation state, so revising the spec or the code re-blocks the
  earliest affected checkpoint and everything after it.
- **Feature v2** — retains v1 and adds Product Journey Contribution at F2 plus
  `product_journey` at F3. The journey target is the accepted frontier for `preserve` and the
  proposed new frontier for `advance`. The accepted frontier derives from the Charter and current
  acceptance receipts; journey evidence additionally binds to both.
- **DBA-10 journey evidence** — derives cross-feature topology from approved Architecture Scopes,
  gives that definition to the runner, validates runtime-only observations, and derives
  reachability per transition. `journey` reports accepted/candidate frontiers and the first
  non-passing seam without storing a Journey Status artifact.
- **Operation** — O1 an observation statement carried on the O2 receipt, O2 the route classification
  receipt (one of six routes; the tool never selects one), O3 derived, O4 route-appropriate
  resolution evidence (an acceptance recorded after the route decision for a build route; an
  explicit `no_action_closure` receipt bound to the same observation for `no_action`; a human-linked
  resolution for the Charter / architecture routes), O5 derived.

## Main parts

- `checker.rs` — `evaluate` (read-only state derivation, fail-closed ordering, the `bootstrap-entry`
  synthetic gate), `evaluate_checkpoint` (one arm per `Requirement`), `run_checks` (the `check`
  execution loop), `current_bindings` / `verification_bindings` (what each receipt and verification
  record is bound to), `run_mechanic` / `run_integrated_baseline` (live execution).
- `contract.rs` — the `Workflow`, `Applies`, and `Requirement` vocabularies and the three
  `*_contract()` functions. Plain data, not user-authored, not a DSL.
- `verification.rs` — `VerificationRecord`, `VerificationStore` (append / latest by
  checkpoint+verification), and `evaluate` → `Absent` / `Current` / `Failed` / `Stale`.
- `receipts.rs` — `Receipt`, `ReceiptStore` (append with closed-set and rationale enforcement), and
  `evaluate` with the same three-state result.
- `evidence.rs` — frontmatter parsing, approval predicates, `section_filled`, reviewer-record
  lookup, and the command runner (`run` / `run_env`).
- `hashing.rs` — `file_sha256`, `working_tree_state` (committed tree entries + tracked diff +
  untracked bytes for resolved implementation paths, so previews bind to uncommitted work without
  drifting on unrelated commits), `text_sha256`, and `binding_drift` shared by both stores.
- `project.rs` / `report.rs` / `main.rs` — path resolution, `status` / `next` rendering, CLI.

## Data and state

Two append-only JSONL files under `.codeos/06-workflow/` (or `maintenance/` in self-development).
Neither is authority: both are progression evidence that ceases to establish its checkpoint when a
bound input drifts. `implementation_state` is a content hash over the Cargo-resolved workspace and
package set, `web`, applicable migrations, and `docker-compose.yml`, not a commit SHA. Nothing else
is persisted — every canonical-state predicate is recomputed on each query.

## Design choices

- **No indirect reconstruction of a PASS.** A verification either has a current record or it does
  not; a later receipt never vouches for an earlier verification, and there is no mode in which
  `status` differs from `check` except that `check` may first execute and persist.
- **B3 and B4 bind away from feature state.** Bootstrap is a completed phase. Binding its evidence
  to working-tree implementation state would deadlock a solution's first feature — F1–F9 could never
  pass, so no acceptance receipt could ever satisfy the gate's own bypass. The live integration
  guarantee is carried forward by each feature's F3/F5/F5d/F5g instead.
- **Mechanizing progression raises no acceptance bar.** A checkpoint proves the evidence a decision
  needs exists and is current; it never demands a stronger result than the rest of DBA requires. A
  reconciliation that honestly records unresolved gaps is `completed`.
- **The route is never inferred.** O2 is the one irreducible classification; the tool reports that a
  route is missing or invalid and stops.

## Dependencies and boundaries

`anyhow`, `clap`, `serde` / `serde_json` / `serde_yaml`, `sha2`, `hex`, `chrono`; `tempfile` for
tests. It shells out to `git`, `docker compose`, `curl`, `cargo`, `npm`, and
`project-config-contract.sh`. It reads the reviewer's assessment records but never writes them, and
it treats every reviewer conclusion as advisory. It does not flip the active DBA configuration or
edit any governed artifact.
