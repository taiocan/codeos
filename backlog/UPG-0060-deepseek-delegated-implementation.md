---
feature_id: UPG-0060
slug: deepseek-delegated-implementation
title: DeepSeek-Delegated Implementation for Stages 4-5
status: PILOTED   # negative result — CHG-A COMPLETE and shipped off by default; CHG-B NOT DONE (2026-08-03)
priority: P2
depends_on: [UPG-0056]
related_features: [UPG-0032, UPG-0057]
supersedes: []
superseded_by: []
---

# Upgrade: deepseek-delegated-implementation — DeepSeek-Delegated Implementation for Stages 4-5

**Priority**: P2
**Status**: PILOTED (negative) — CHG-A COMPLETE and shipped off by default; **CHG-B will not be done**
**Type**: script-tooling + downstream-doctrine (two CHGs — CHG-A tool pilot, CHG-B doctrine wiring)

> **Outcome (2026-08-03).** The CHG-B gate measurement returned NOT NET-POSITIVE and the human decided
> to hold the feature here rather than wire doctrine or abandon outright. The tool ships as CHG-A left
> it: off by default, no downstream footprint. See the CHG-B Gate section below for the evidence, the
> verdict, and the named conditions under which this question may be reopened. The generalizable
> finding is journaled as **AJ-022**.

## Problem

Codeos already offloads one kind of AI work to a non-Claude model to keep it off Claude Code's token
budget: the advisory Codex reviewer (`scripts/codeos-review.sh` → `tools/reviewer/`). No equivalent
exists for the most token-expensive part of the downstream DBA loop — **Stage 4 (Implementation)**
and **Stage 5 (Tests)**, where the AI writes code and tests as a "constrained satisfier"
(`prompts/04-implement.md`, `prompts/05-tests.md`). Every downstream feature pays Claude's generation
tokens for that code and those tests. A cheaper model (`DEEPSEEK_API_KEY` is present in the
environment) could draft the Stage 4/5 candidate, leaving Claude's stronger reasoning for the
approval, reconciliation, and review that already guard correctness.

Two facts constrain any design:

- **Delegating implementation is not the same as delegating review.** Review is read-only and
  advisory — a weak review costs nothing because the human still decides at the gate. Implementation
  is the *primary artifact* that flows through the gates, so a weaker draft shifts cost onto
  reconciliation and review rather than removing it. Net token savings are plausible but **not proven
  in advance** — they must be measured before any doctrine commits to the mechanism.
- **The reviewer is a local-CLI, read-only integration, not an HTTP/API one.** It shells out to the
  `codex` CLI in a read-only sandbox; there is no HTTP client or API key anywhere in the toolkit. A
  DeepSeek-via-API tool would be the first HTTP/API-key path in the repo, and it *writes* rather than
  reads — so it cannot simply reuse the reviewer's read-only invariant.

## Upgrade

Introduce an **opt-in, off-by-default** tool that drafts Stage 4/5 candidates via the DeepSeek API,
staged for human promotion, with every existing human-approval gate and advisory review unchanged.
Delivered in two change records so the doctrine change is contingent on measured pilot evidence:

- **CHG-A (script-tooling, self-dev only) — build and pilot the tool.** A new
  `scripts/codeos-implement.sh` that mirrors the reviewer shim's shape (git-repo precondition,
  self-dev-vs-downstream context resolution by `pwd -P` symlink comparison, explicit exit codes,
  fail-closed preconditions). It reads a one-line activation status file per the Optional Mechanism
  Status Convention (UPG-0056) and refuses to run unless `status: enabled`. When enabled, it builds a
  packet from a new `prompts/codeos-implementer-task.md` (pinning DeepSeek to the constrained-satisfier
  role and the three approved artifacts), calls the DeepSeek API using `$DEEPSEEK_API_KEY`, and writes
  the candidate code/tests to a staging directory — never directly into `modules/` or `tests/`, never
  committed. It preserves the sent packet and raw response for audit and logs DeepSeek token usage.
  The change captures one pilot run with a token-and-quality comparison as evidence.

- **CHG-B (downstream-doctrine, both) — wire the mechanism into doctrine, only if CHG-A is
  net-positive.** Add an optional-mechanism section to `dba-system.md` documenting activation
  mechanics only (mirroring the Controlled Plain English section: no new Stage ID, no new
  Non-Negotiable Rule, no new mandatory human-approval gate); scaffold the downstream status file at
  `status: disabled` in `scripts/dba-init.sh`; add advisory notes to `prompts/04-implement.md` and
  `prompts/05-tests.md` that a delegated candidate MAY be produced but the same gate and advisory
  review apply. If the pilot is not net-positive, CHG-B is not done.

  > **Superseded by the 2026-08-03 decision.** This paragraph originally ended "…and this feature is
  > abandoned." The pilot was not net-positive and CHG-B was not done, but the human did **not**
  > abandon the feature: it is held at CHG-A as PILOTED (negative), because the gate measurement was
  > later found to be confounded by the harness itself (AJ-022 amendment). Abandonment was one of three
  > options offered and was not the one taken. The binding statement of this feature's state is the
  > **Outcome** note at the top of this file and the **CHG-B Gate** section below.

The tool stages a *candidate*; a human (or Claude) promotes it, and the existing Stage 4/5 human gate,
advisory Codex review, and Stage 7 reconciliation then apply exactly as today. The tool never
approves anything — Non-Negotiable Rule #1 (the human decides at the gate) is untouched.

## Scope

**In scope (CHG-A):** `scripts/codeos-implement.sh`, `prompts/codeos-implementer-task.md`, a self-dev
activation status file `config/delegated-implementation.yaml` (scaffolded at `status: disabled`), this
brief, the CHG-A change record, and lifecycle bookkeeping (`backlog/features.md`,
`status/self-development.md`, `status/roadmap.md`). One captured pilot run with token/quality evidence.

**In scope (CHG-B, contingent):** an optional-mechanism section in `dba-system.md`; downstream status
file scaffolding in `scripts/dba-init.sh` at `status: disabled`; advisory notes in
`prompts/04-implement.md` and `prompts/05-tests.md`; downstream-compatibility acceptance criteria and a
grep cross-reference sweep.

**Out of scope:** any change to the Rust reviewer engine (`tools/reviewer/`) — adding an HTTP DeepSeek
provider there is a possible future consolidation, decided when concrete, not here; any new mandatory
human-approval gate; any new Stage ID; any Non-Negotiable Rule change; automatic promotion of a
candidate into `modules/`/`tests/` or any auto-commit; Stage 6 (Observation is human-run runtime
execution — there is no implementation to delegate); enabling the mechanism by default anywhere.

## Value

Moves Stage 4/5 code and test generation off Claude Code's token budget when a project opts in, the
same lever the reviewer already applies to review work. The two-CHG split makes the saving *measured,
not assumed*: CHG-A proves (or disproves) net token benefit and gate-passing quality on a real run
before any downstream-doctrine text commits to the mechanism. Because the mechanism is off by default
and inert unless explicitly enabled, shipping CHG-A carries no behavioral change for any existing
project.

## Risk

- **First HTTP/API-key path in the toolkit.** The key must never reach a packet, log, response dump,
  or candidate file. Mitigated by reusing the reviewer's secret-redaction discipline and verifying by
  grep over the tool's outputs.
- **Writing automation, not read-only.** The tool writes generated code. Mitigated by candidate
  staging plus explicit human promotion: the tool never writes into `modules/`/`tests/` and never
  commits, preserving "artifacts on disk are authoritative" and the human gate.
- **Net-token benefit is unproven.** A weaker draft can add review/reconciliation rounds. Mitigated by
  making CHG-A measure it and by gating CHG-B on that evidence. *(Outcome: measured 2026-08-03 and
  found not net-positive, so CHG-B was not done. The feature was held at CHG-A rather than abandoned —
  the measurement was confounded by the harness, so the result does not support the stronger
  conclusion. See the CHG-B Gate section.)*
- **Autonomy prohibition.** `dba-system.md` forbids multi-step autonomous execution. Mitigated by
  framing the tool as single-shot, human-invoked generation of a staged candidate — not an autonomous
  agent.

## Guardrail

- The tool must refuse to run when its activation status file is absent, `status: disabled`, or
  malformed — fail-closed, exactly like the reviewer shim's preconditions.
- The tool must never write into `modules/` or `tests/`, and never commit. Staging directory only.
- `DEEPSEEK_API_KEY` must never appear in any preserved packet, log, response, or candidate.
- CHG-A must not touch `dba-system.md` or any downstream prompt. Doctrine wiring is CHG-B, and CHG-B
  proceeds only on positive CHG-A evidence.
- Do not add a DeepSeek provider to the Rust reviewer engine as part of this feature. That is a
  separate decision if a concrete need arises.

## Related

- **UPG-0056** (Optional Mechanism Status Convention) — the one-line on/off convention this tool's
  activation gate consumes; `depends_on`.
- **UPG-0057** (Controlled Plain English) — precedent for a two-CHG feature and for an optional
  mechanism documented as activation-mechanics-only in `dba-system.md` with no new gate.
- **UPG-0032** (Rust Reviewer Engine, multi-provider) — the reviewer's provider abstraction and the
  place a future consolidation could host a DeepSeek provider; explicitly out of scope here.
- Backlog thesis items 1 ("reduce human review load" / token load) and 2 ("make Stage 4–6 execution
  transparent") in `backlog/features.md`.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the change
> records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260802-001 | `changes/UPG-0060__CHG-20260802-001__deepseek-implement-tool.md` | Build and pilot the out-of-band DeepSeek Stage 4/5 implementer tool (CHG-A) | COMPLETE (accepted 2026-08-03) |
| CHG-20260803-001 | `changes/UPG-0060__CHG-20260803-001__implementer-harness-correction.md` | Correct the delegation harness — re-test **condition 0**: permit build manifests, supply a layout exemplar, stop instructing against required abstractions, emit plain source instead of JSON-escaped, allow one bounded repair iteration | IN_PROGRESS (Step 1) |
| (planned) CHG-B | — | Wire the mechanism into `dba-system.md` + prompts as an optional, off-by-default mechanism — was contingent on the CHG-B gate measurement below | **NOT DONE** — gate returned NOT NET-POSITIVE; human decision 2026-08-03. Re-openable only after condition 0 + a re-test clearing all three axes |

### CHG-B Gate — realistic-feature net-token measurement

CHG-A's own pilot was a toy feature: it proved the mechanism works end-to-end but produced no
meaningful token saving to measure. The human decision at CHG-A acceptance (2026-08-03) made CHG-B
contingent on a second, explicitly *non-toy* measurement: run one realistic downstream feature through
delegated implementation and compare **total DeepSeek + Claude reconciliation/review cost** against the
normal Claude-only path. Proceed with CHG-B only if the result is materially net-positive; otherwise
abandon the feature.

| Item | Value |
|---|---|
| Target feature | EvidenceAtlas `EA-0003` corpus_construction (Stage 3 approved; Stage 4 next) |
| Evidence file | `changes/UPG-0060__CHG-B-GATE__realistic-feature-evidence.md` |
| Delegate result | DeepSeek 28,437 tokens; candidate does not compile as delivered; 8 confirmed contract/schema violations after minimum repair |
| Claude-only comparator | Same feature, same artifacts: compiles clean first try, 10/10 approved-contract scenario tests pass |
| Verdict | **NOT NET-POSITIVE** — Arm A = Arm B + ~5.4K Claude input tokens + 28,437 DeepSeek tokens, saving zero Claude output tokens |
| Decision | **Hold at CHG-A, feature PILOTED (negative)** — human decision 2026-08-03 (option 1 of three offered; the alternatives were a Stage-5-only re-scope and outright abandonment). CHG-B will not be done. |
| Journaled as | **AJ-022** — a rigorous specification is a poor delegation target; Stage 4 delegability falls as contract rigor rises |

**Why hold rather than abandon.** CHG-A shipped the tool off by default with no downstream footprint —
no `dba-system.md` text, no prompt text, no `dba-init.sh` scaffolding — so holding costs nothing and
changes nothing for any existing project. Abandoning would discard a working, already-paid-for
mechanism whose negative result is attributable to the delegate's capability on invariant-dense
specifications, not to a defect in the tool.

**Re-test conditions — ordered, not a menu.** Re-reading the harness after the gate measurement showed
the packet handicapped the delegate (see AJ-022's same-day amendment). The measurement therefore
conflated a weak delegate with a harness that suppressed build manifests, withheld any layout
exemplar, instructed against the abstractions the invariants require, forced JSON-escaped source, and
allowed no repair iteration. **Condition 0 gates conditions 1-3: a re-test that changes the model
without first fixing the packet is uninterpretable.**

**0. Harness correction — PREREQUISITE.** A `prompt` + `script-tooling` change under this feature, run
through the 4-step self-development loop before any further comparison:
   - permit non-source files (build manifests, module config) when the target language requires them
     to build, while keeping the stage-area allowlist;
   - include one real repository-layout exemplar in the packet, so module naming is observed rather
     than guessed;
   - replace "add no abstractions" with "no unnecessary capability; implement the abstractions the
     contract's explicit invariants require" — the current wording pushes against invariant-carrying
     structure;
   - emit normal file output instead of JSON-escaped source;
   - allow one bounded compile-feedback repair iteration.

**1. A materially stronger delegate model** — `deepseek-reasoner` or a successor. Meaningful only after
condition 0, since the current result cannot separate model capability from harness handicap.

**2. A repair loop beyond the single iteration in condition 0** — feeding failing contract-derived
tests back for a bounded number of rounds. Would need to show that repair converges for less than the
cost of writing the code directly.

**3. A Stage-5-only re-scope** — delegating test authoring from an approved contract instead of
implementation. Tests are more mechanical given a contract, and a wrong test fails loudly rather than
shipping a silent invariant violation. A different feature shape; would warrant its own brief rather
than reviving CHG-B.

**Judging any re-test.** Score the three axes separately and report them separately — a gain on one
does not license adoption:

| Axis | Question |
|---|---|
| Contract adherence | How many approved contract/schema clauses does the candidate actually satisfy, tested? |
| Technical correctness | Does it compile and pass its own build, unaided? |
| Net cost | Does **total** Claude-token and human-review cost fall enough to matter? |

The third axis is decisive. Even a substantially improved candidate justifies adoption only if
downstream reconciliation cost falls materially — the delegated draft remains the primary artifact
through the gates, so verification cost does not compress the way generation cost does.

Absent condition 0 followed by a re-test that clears all three axes, the mechanism stays inert and this
feature stays closed.

### Reviews

| Review Series | Change ID | Step | Rounds | Accepted Verdict |
|---|---|---|---|---|
| RVS__UPG-0060__CHG-20260802-001__S1 | CHG-20260802-001 | 1-Intent | R1-R2 | R1 CHANGES ADVISED (unbacked roadmap scope claim) → R2 NO OBJECTION |
| RVS__UPG-0060__CHG-20260802-001__S2 | CHG-20260802-001 | 2-Acceptance | R1 | NO OBJECTION (accepted) |
| RVS__UPG-0060__CHG-20260802-001__S3 | CHG-20260802-001 | 3-Implement | R1-R4 | R1-R3 CHANGES ADVISED (all fixed inline); confirmatory R4 NO OBJECTION |
| RVS__UPG-0060__CHG-20260802-001__S4 | CHG-20260802-001 | 4-Reconcile | R1-R2 | R1 CHANGES ADVISED (AC-8 exit-code evidence) → R2 NO OBJECTION (evidence A) |

### Findings Tracked Inside This Feature

| Review Series | Classification(s) | Resolution |
|---|---|---|
| — | — | — |

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
| — | — | — |
