# Self-Development Change: UPG-0062__CHG-20260803-002 — architecture-constrained-delegation

<!--
PURPOSE: Per-change source of truth for the first change of UPG-0062 — a PREMISE TEST and a
GOVERNANCE QUESTION. It answers whether producing a per-feature implementation design is materially
cheaper than implementing directly (Q1), and whether Codeos needs a governed home for mechanism
decisions (Q2). NO CODE IS WRITTEN IN THIS CHANGE: the Rust engine, shim reduction, prompt rewrite,
and comparative pilot are deferred to a contingent CHG-C that does not open unless Q1 passes and Q2 is
settled. Self-development change (documentation/analysis); no downstream doctrine change.
Workflow: prompts/codeos-self-dev.md (4-step loop).
The live status row lives in status/self-development.md, not here.

UPG-0060 is closed and is NOT reopened. Its evidence is prior evidence for a different hypothesis.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0062
primary_feature_id: UPG-0062
change_id: CHG-20260803-002
slug: architecture-constrained-delegation
state: IN_PROGRESS      # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 2-Acceptance  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0062
related_features: [UPG-0051, UPG-0058, UPG-0052, UPG-0060, UPG-0032]
review_series: RVS__UPG-0062__CHG-20260803-002__S2   # S1 ACCEPTED (R1-R3 DO NOT ADVANCE → R4 NO OBJECTION)
review_profile: PROFILE-2   # documentation — no code in this change (Step 0a; re-assigned when scope narrowed at the Step 1 gate)
review_state: IN_REVIEW # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round)
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

Before any mechanism is built to exploit that, one assumption has to hold: that writing the
per-feature implementation design an external model would execute is materially cheaper than writing
the implementation. **This change tests that assumption and nothing else.** It writes no code. If the
assumption fails, UPG-0062 stops here having spent one bounded piece of analysis — the inverse of
UPG-0060's sequencing, which hardened a mechanism before measuring whether it was worth having.

Full problem statement, UPG-0060 evidence table, hypothesis, delegation boundary, measurement plan,
success and abandonment criteria, and the critical assessment of the Architecture Baseline are in
`backlog/UPG-0062-architecture-constrained-delegated-implementation.md`. That brief is the
feature-level statement; this record governs execution.

**What changes** *(amended at the Step 1 gate, human 2026-08-04 — see "Step 1 gate" below;
Step 1's original list moved wholesale to the contingent CHG-C)*:

- `changes/UPG-0062__CHG-20260803-002__…-evidence.md` — **new.** The premise-test record: EA-0004's
  Feature Implementation Design produced from approved artifacts only, its measured cost, Arm B's
  measured cost, the stop/continue verdict against a pre-stated threshold, and the Q2 governance
  analysis and recommendation.
- `changes/UPG-0062__CHG-20260803-002__architecture-constrained-delegation.md` — this record.
- Lifecycle bookkeeping: `backlog/features.md`, `status/self-development.md`, `status/roadmap.md`,
  and the feature brief's Feature Thread.

**Deferred to the contingent CHG-C, not done here:** `tools/implementer/` (the Rust crate),
the `scripts/codeos-implement.sh` shim reduction, the `prompts/codeos-implementer-task.md` rewrite to
an architecture-constrained implementer role, and the two-arm delegated pilot. Their content is
unchanged from Step 1; only their location moved.

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

**Class:** documentation (premise test + governance analysis; no code)
**Scope axis:** self-dev only
**Backlog item:** `backlog/UPG-0062-architecture-constrained-delegated-implementation.md`

---

## Open question for the gate — ANSWERED at the Step 1 gate

> **Resolved (human, 2026-08-04): approved as a premise test, with the artifact reclassified.** The
> proposal below was accepted and is now implemented by this change's narrowed scope and by Step 2's
> acceptance criteria. Retained for provenance; superseded by the "Step 1 gate" section.

**Does producing the Feature Implementation Design cost materially less than writing the implementation?**

This is the load-bearing assumption. If deriving the allocation requires the same close contract
reading as implementing the feature, the hypothesis collapses into UPG-0060's finding and this feature
should be abandoned at Step 2 rather than piloted — cheaply, before any Rust is written.

Evidence for: UPG-0060's run-3 feedback was ~2.5 KB and yielded 610 lines (~25 KB) of correct Rust,
roughly 10:1. Evidence against: that feedback was written with a failed candidate in hand, which is
strictly easier than writing an allocation from approved artifacts alone.

**Proposed resolution, for approval:** make this a **falsifiable Step 2 gate, executed before Step 3**.
Draft EA-0004's Feature Implementation Design from approved artifacts only, measure its cost, and compare
against the measured cost of Arm B's implementation of the same feature. If the allocation approaches
implementation cost, stop at Step 2 and close the feature. This spends one bounded piece of work to
test the premise before committing to a Rust engine — the sequencing error UPG-0060 made in reverse.

A second, smaller question for the same gate: **should the Feature Implementation Design persist as a file?**
A transient packet section cannot drift from the approved artifacts but leaves the pilot less
auditable; a persisted file is auditable but is one more thing that can go stale. My inclination is
transient-plus-archived-in-the-audit-set — the packet already preserves what was sent — but this is a
genuine design choice and is deferred to Step 2.

---

## Step 1 gate — human corrections carried into Step 2

Two corrections were made when Step 1 was approved (human, 2026-08-04). Both change what this change
does; neither is a wording tweak.

**1. The Feature Implementation Design is not a derived view.** Step 1 proposed it as one. That was
wrong. Where the approved architecture says *where* an invariant belongs but not *how* it is enforced,
the chosen mechanism is a **new design decision**. Calling it derived would have created a second
architecture layer that appeared to need no governance because of what it was called. Every row must
now be classified `SOURCE-DERIVED` (citing artifact + section) or `NEW DESIGN`, and `NEW DESIGN` rows
must never be attributed to the approved Baseline.

**2. Step 2 answers two questions, and this change narrows to them.** Step 1 declared `tools/implementer/`
(a new Rust crate), the shim reduction, the prompt rewrite, and the pilot all inside this change. The
human's sequencing instruction is: *if producing the design is comparable to direct implementation,
stop UPG-0062; if materially cheaper, continue and resolve its authority/lifecycle before building the
Rust delegation engine.*

**This change is therefore narrowed to the premise test and the governance question. No Rust is
written in it.** The engine, shim reduction, prompt rewrite, and comparative pilot move to a second,
contingent change (`CHG-C`) that does not begin unless Q1 passes and Q2 is settled. Narrowing is
surfaced here rather than done silently; it is the same two-change split that let UPG-0060 stop
cleanly instead of shipping doctrine for a saving that was never there, and it is the direct remedy
for the premature-hardening failure that feature diagnosed.

---

## Acceptance Criteria

<!-- Step 2. This change is a premise test and a governance decision. Criteria for the Rust engine,
shim, prompt, and pilot are deliberately NOT written here — they belong to the contingent CHG-C and
would be premature hardening if written before Q1 and Q2 are answered. -->

**Definitions.** The **Feature Implementation Design (FID)** is the per-feature artifact under test:
for each contract invariant and falsification scenario, the enforcing mechanism and its location.
**Arm B** is Claude implementing EA-0004 directly by the normal Stage 4 process. **Approved artifacts**
means EA-0004's intent, contract, event schema, plus the cohort Architecture Baseline, Cohort Logical
Design, and Implementation Profile.

### Group 1 — Q1: is the FID materially cheaper than implementing?

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | **The FID is produced from approved artifacts only.** No candidate implementation — delegated or Claude-written — exists or is consulted while it is written. This is the criterion that makes the test honest: UPG-0060's run-3 feedback was written with a failed candidate in hand, which is strictly easier and would measure the wrong thing. | Ordering is recorded: FID written and its content frozen (hash recorded) before Arm B begins. |
| 2 | **Every row is classified and every `SOURCE-DERIVED` row cites a real artifact section.** No row attributes a mechanism choice to the approved Baseline when the Baseline does not determine it. | Read every row; spot-check each cited section actually says what is claimed. |
| 3 | **The FID covers every invariant and falsification scenario in EA-0004's approved contract** — never a subset. | Count against the contract; each contract item appears. |
| 4 | **Cost of producing the FID is measured** — Claude input and output tokens (derived from byte counts, method stated), and its own size. | Recorded in the evidence file. |
| 5 | **Arm B is executed and measured** on the same feature: Claude implements EA-0004 directly, cost recorded on the same basis. | Recorded in the evidence file; the implementation exists and compiles. |
| 6 | **A stop/continue verdict is stated explicitly**, with the comparison, and with "materially cheaper" given a stated threshold rather than left to impression. A result inside the error bars of the estimation method counts as *not* materially cheaper. | Read the verdict; the threshold is stated before the numbers are compared. |

### Group 2 — Q2: does Codeos need a governed home for mechanism decisions?

| # | Criterion | How it will be verified |
|---|---|---|
| 7 | **The `NEW DESIGN` share of the FID is quantified**, not characterised. How much of EA-0004's implementation design is determined by approved artifacts, and how much is genuinely new. | Row counts by classification, plus a judgement of weight, both reported. |
| 8 | **The question "has Stage 4 been silently making these decisions all along?" is answered with evidence**, by examining at least one already-implemented feature (EA-0001) for mechanism decisions present in its code and absent from every approved artifact. | Named examples from `modules/research_brief/`, each traced to the absence of a governing artifact. |
| 9 | **A recommendation on governance is made**, with options and a preference: no artifact needed / an ungated record / a gated artifact — and if gated, where it sits relative to Stage 3 and Stage 4 and who approves it. | Read the recommendation; it names a preference and its cost. |
| 10 | **The recommendation does not create a second architecture authority.** Whatever is proposed, approved artifacts remain authoritative and the FID never overrides them. | Read against the brief's governance constraints. |

### Group 3 — gates and scope

| # | Criterion | How it will be verified |
|---|---|---|
| 11 | **If Q1 fails, this change closes at Step 2 and UPG-0062 stops.** No Rust is written, no engine scaffolded, no prompt rewritten. A negative result is a complete and acceptable outcome. | If triggered: the change record records the stop and the feature status changes; `git diff --stat` shows no `tools/implementer/`. |
| 12 | **If Q1 passes, Q2 is settled before CHG-C opens.** The engine is not built against an artifact whose authority and lifecycle are undecided. | CHG-C's Step 1 cites the settled governance answer as a precondition. |
| 13 | **No Rust, no prompt rewrite, no shim change in this change.** `tools/`, `scripts/`, and `prompts/` are untouched. | `git diff --stat`. |
| 14 | **No downstream doctrine change and no new gate.** `dba-system.md`, `prompts/04-implement.md`, `prompts/05-tests.md`, `scripts/dba-init.sh` byte-unchanged; `config/delegated-implementation.yaml` stays `status: disabled`; UPG-0060's records unmodified. | `git diff --stat`. |
| 15 | **EA-0004 stays uncontaminated for CHG-C's pilot, or the contamination is declared.** Arm B produces a Claude implementation of EA-0004 as the cost baseline — so if CHG-C later runs a delegated arm on EA-0004, the comparison is against a known baseline, not a blind one. Any effect this has on CHG-C's pilot design is stated now, not discovered later. | Read the statement in the evidence file. |

**Explicitly not in scope for these criteria:** whether the delegated model performs well against the
FID. That is CHG-C's question and cannot be answered here — by criterion 13 there is nothing to run it
with.

---

## Implementation Notes

*(pending Step 3)*

---

## Reconciliation

*(pending Step 4)*
