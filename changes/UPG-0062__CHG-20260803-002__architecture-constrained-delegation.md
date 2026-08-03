# Self-Development Change: UPG-0062__CHG-20260803-002 — architecture-constrained-delegation

<!--
PURPOSE: Per-change source of truth for the first change of UPG-0062 — establish the
architecture-constrained delegated-implementation mechanism as a Rust engine behind a thin shim, and
run the comparative pilot. Self-development toolkit change (script-tooling + Rust engine); no
downstream doctrine change is proposed. Workflow: prompts/codeos-self-dev.md (4-step loop).
The live status row lives in status/self-development.md, not here.

UPG-0060 is closed and is NOT reopened. Its evidence is prior evidence for a different hypothesis.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0062
primary_feature_id: UPG-0062
change_id: CHG-20260803-002
slug: architecture-constrained-delegation
state: DRAFT            # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 1-Intent  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0062
related_features: [UPG-0051, UPG-0058, UPG-0052, UPG-0060, UPG-0032]
review_series: RVS__UPG-0062__CHG-20260803-002__S1
review_profile: PROFILE-3   # script-tooling + Rust engine (Step 0a)
review_state: DRAFT     # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round)
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round. Reference the stable review SERIES (review_series) + review_state; exact rounds live
only in reviews/review-log.md and reviews/codex/*. -->

---

## Change Intent

**Why (problem in the toolkit):**

Stage 4 generation cannot currently be moved off Claude's budget. UPG-0060 established that a cheap
model cannot *derive* an implementation from a behavioral contract — but its final run showed the
same model executing an explicit architectural specification correctly, fixing 6 of 6 contract
violations in one iteration. The delegation boundary that failed was architectural derivation; the
boundary that worked was constrained execution.

This change tests the second boundary properly, and builds the mechanism in the shape Codeos has
already established for out-of-band tooling: a Rust engine behind a thin Bash shim.

Full problem statement, UPG-0060 evidence table, hypothesis, delegation boundary, measurement plan,
success and abandonment criteria, and the critical assessment of the Architecture Baseline are in
`backlog/UPG-0062-architecture-constrained-delegated-implementation.md`. That brief is the
feature-level statement; this record governs execution.

**What changes:**

- `tools/implementer/` — **new Rust crate.** Owns configuration loading and validation, packet
  construction, architecture and stage-artifact resolution, allowed-path enforcement, request
  construction, HTTP interaction, response parsing and validation, candidate staging, exit-code
  semantics, temp-file lifecycle, deterministic error handling, and audit outputs. Follows the
  `tools/reviewer/` pattern as a reference for *shape*, not by copying behavior.
- `scripts/codeos-implement.sh` — **reduced to a thin shim**, in the style of
  `scripts/codeos-review.sh` (106 lines): locate the binary, minimal bootstrap, forward arguments,
  invoke. The current 393-line Bash implementation's behavioral contract — exit codes, fail-closed
  ordering, staging layout, the nonce-delimited output protocol — is carried over deliberately as
  reusable evidence; its implementation complexity is not preserved to minimize a diff.
- `prompts/codeos-implementer-task.md` — **rewritten** from "constrained satisfier" to
  **architecture-constrained implementer**: implement the approved design, do not redesign it, do not
  move invariants to callers when the architecture assigns them elsewhere, do not introduce
  alternative abstractions, do not remove required ones to simplify, emit supporting files the build
  requires, follow layout exemplars, and return an explicit inability-to-implement result rather than
  inventing a design. It must not inherit `prompts/04-implement.md`'s "No additional abstractions"
  wording (see the brief's discovered-issue note).
- Invariant Allocation mechanism — the per-feature invariant→mechanism view identified in the brief
  as the smallest missing information. **Its form is deliberately undecided at Step 1** (persisted
  artifact vs. transient packet section) and is a Step 2 question, because the choice depends on the
  architectural question below.
- `changes/UPG-0062__CHG-20260803-002__architecture-constrained-delegation.md` — **new**, this record.
- Lifecycle bookkeeping: `backlog/features.md` (new UPG-0062 row), `status/self-development.md`
  (new IN_PROGRESS row), `status/roadmap.md`.
- Pilot evidence: the two-arm comparison on EvidenceAtlas EA-0004, recorded in a durable evidence
  file with the four measurement groups reported separately.

**Scope boundary — what stays the same:**

- **UPG-0060 is not reopened, modified, or reinterpreted.** Its records stay as they are.
- **No downstream doctrine change.** `dba-system.md`, `prompts/04-implement.md`, `prompts/05-tests.md`,
  `prompts/06-observe.md`, and `scripts/dba-init.sh` are untouched. Notably `04-implement.md` keeps
  its "No additional abstractions" wording despite the concern recorded in the brief — changing it is
  downstream-doctrine and belongs to its own change.
- **No changes to `tools/reviewer/`**, no shared orchestration framework, no generalized HTTP
  infrastructure, no CLI consolidation.
- The mechanism stays **off by default** — `config/delegated-implementation.yaml` remains
  `status: disabled`; no downstream status file is scaffolded.
- Every existing safety property is preserved: candidate staging only, never `modules/`/`tests/` in
  the real tree, never a commit, no key leakage, fail-closed preconditions, and **the tool executes
  no build, test, or project-supplied command** (UPG-0060's Option B boundary carries over).
- No new human-approval gate, no new Stage ID, no Non-Negotiable Rule change, no second architecture
  authority.
- Stage 5 delegation is out. Architectural synthesis, Stages 1-3, and human decisions are never
  delegated.

**Class:** script-tooling (+ new Rust engine)
**Scope axis:** self-dev only
**Backlog item:** `backlog/UPG-0062-architecture-constrained-delegated-implementation.md`

---

## Open question for the gate

**Does producing the Invariant Allocation cost materially less than writing the implementation?**

This is the load-bearing assumption. If deriving the allocation requires the same close contract
reading as implementing the feature, the hypothesis collapses into UPG-0060's finding and this feature
should be abandoned at Step 2 rather than piloted — cheaply, before any Rust is written.

Evidence for: UPG-0060's run-3 feedback was ~2.5 KB and yielded 610 lines (~25 KB) of correct Rust,
roughly 10:1. Evidence against: that feedback was written with a failed candidate in hand, which is
strictly easier than writing an allocation from approved artifacts alone.

**Proposed resolution, for approval:** make this a **falsifiable Step 2 gate, executed before Step 3**.
Draft EA-0004's Invariant Allocation from approved artifacts only, measure its cost, and compare
against the measured cost of Arm B's implementation of the same feature. If the allocation approaches
implementation cost, stop at Step 2 and close the feature. This spends one bounded piece of work to
test the premise before committing to a Rust engine — the sequencing error UPG-0060 made in reverse.

A second, smaller question for the same gate: **should the Invariant Allocation persist as a file?**
A transient packet section cannot drift from the approved artifacts but leaves the pilot less
auditable; a persisted file is auditable but is one more thing that can go stale. My inclination is
transient-plus-archived-in-the-audit-set — the packet already preserves what was sent — but this is a
genuine design choice and is deferred to Step 2.

---

## Acceptance Criteria

*(pending Step 2)*

---

## Implementation Notes

*(pending Step 3)*

---

## Reconciliation

*(pending Step 4)*
