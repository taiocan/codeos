# Self-Development Change: UPG-0060__CHG-20260803-001 — implementer-harness-correction

<!--
PURPOSE: Per-change source of truth for the harness correction that the CHG-B gate measurement
identified as a prerequisite to any further delegation comparison (condition 0 in the feature brief;
AJ-022 same-day amendment). Self-development toolkit change (prompt + script-tooling); no downstream
behavioral contract, event schema, or replay. Workflow: prompts/codeos-self-dev.md (4-step loop).
The live status row lives in status/self-development.md, not here.

This change corrects the delegation harness. It does NOT re-open CHG-B, does not enable the mechanism
anywhere, and does not by itself decide whether delegation is adopted — the re-test that follows it is
a separate measurement, judged on the three axes named in the feature brief.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0060
primary_feature_id: UPG-0060
change_id: CHG-20260803-001
slug: implementer-harness-correction
state: IN_PROGRESS      # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 1-Intent  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0060
related_features: [UPG-0056, UPG-0057]
review_series: RVS__UPG-0060__CHG-20260803-001__S1
review_profile: PROFILE-3   # prompt + script-tooling, same as CHG-A (Step 0a)
review_state: IN_REVIEW # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round)
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: CHG-20260802-001
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round. Reference the stable review SERIES (review_series) + review_state; exact rounds live
only in reviews/review-log.md and reviews/codex/*. -->

---

## Change Intent

**Why (problem in the toolkit):**

The CHG-B gate measurement (`changes/UPG-0060__CHG-B-GATE__realistic-feature-evidence.md`) ran a real
downstream feature through delegated Stage 4 and returned a negative result. Re-reading the harness
afterwards showed part of that result was caused by CHG-A's own packet, not by the delegate:

| Observed in the measurement | Harness cause |
|---|---|
| Candidate arrived with no build manifest and could not be built | `prompts/codeos-implementer-task.md` forbids it — *"Never emit a path that is not a source or test file"* and *"Add no … files … not traced to the approved artifacts."* `Cargo.toml` appears 0 times in the 105,510-byte packet. The **script already permits** `modules/<name>/Cargo.toml` under its stage-area allowlist; only the prompt blocks it |
| Module named against the project's convention | No layout exemplar was in the packet — the only `modules/` string in it comes from the prompt itself |
| The candidate delegated every contract invariant to its caller | The prompt says *"Add no behavior, no files, no abstractions … not traced to the approved artifacts,"* which reads as an instruction to omit the invariant-carrying structure the contract actually requires |
| Two `E0599` compile errors reached the human | Single shot, JSON-escaped source, no compiler feedback — all harness choices, all known to degrade generated code |

Until these are fixed, no comparison between delegate models is interpretable: a re-test that changes
the model while keeping the packet cannot separate model capability from harness handicap. That is why
the feature brief makes this **condition 0**, gating conditions 1-3.

**What changes:**

- `prompts/codeos-implementer-task.md` — **modified.**
  - Permit non-source files the target language requires in order to build (build manifest, module
    config), scoped to files required for the candidate to build, still inside the stage area.
  - Replace the blanket *"add no abstractions"* with *"no unnecessary capability; implement the
    abstractions the contract's explicit invariants require"* — the current wording pushes against
    exactly the structure a rigorous contract needs, while the intended constraint (do not invent
    capabilities the artifacts never asked for) is preserved.
  - Replace the single-JSON-object output contract with a delimited plain-text file protocol, so
    source is emitted as source rather than JSON string escapes.
  - Describe the layout exemplar section so the model knows the packet's exemplar is a convention to
    follow, not an artifact to implement.
- `scripts/codeos-implement.sh` — **modified.**
  - Accept and label a repository-layout exemplar distinctly from approved artifacts, so it is never
    mistaken for something to implement.
  - Parse the new delimited output protocol in place of `jq`-extracted JSON, keeping every existing
    safety property: stage-area allowlist, absolute-path and `..` rejection, `CANDIDATE_BLOCKED.md`
    escape hatch, key never in argv/body/packet/response/candidate, timestamped non-overwriting
    staging, full audit set, token instrumentation.
  - Accept a repair input (previous candidate + build/test output) so one bounded repair iteration can
    be run — see the open question below for what the tool itself is permitted to execute.
  - Documented exit codes extended for the new failure modes, in the existing header-table style.
- `changes/UPG-0060__CHG-20260803-001__implementer-harness-correction.md` — **new**, this record.
- Lifecycle bookkeeping: `status/self-development.md` (new IN_PROGRESS row), `status/roadmap.md`,
  `backlog/UPG-0060-deepseek-delegated-implementation.md` (Feature Thread row for this change, plus
  correcting two stale sentences that still asserted the feature would be abandoned on a negative
  pilot — the 2026-08-03 decision held it at CHG-A instead).

**Uncommitted prior work in the same working tree (not part of this change).** The workspace also
carries the completed, accepted UPG-0060 CHG-A work and the CHG-B gate record — `scripts/codeos-implement.sh`,
`prompts/codeos-implementer-task.md`, `config/delegated-implementation.yaml`,
`changes/UPG-0060__CHG-20260802-001__*`, `changes/UPG-0060__CHG-B-GATE__*`,
`reviews/architecture-journal.md` (AJ-022), `reviews/review-log.md`, and the UPG-0060 status row in
`backlog/features.md`. Those belong to CHG-20260802-001 and the gate decision, both already accepted;
they appear in this change's reviewed diff only because they are not yet committed. This is the
AJ-017 pattern (completed-but-uncommitted self-dev work reads as scope drift on the next change's
review). `backlog/features.md` in particular is **not** modified by this change.

**Scope boundary — what stays the same:**

- **No re-opening of CHG-B.** No change to `dba-system.md`, any downstream stage prompt, or
  `scripts/dba-init.sh`. The feature stays PILOTED (negative) until a re-test clears all three axes.
- **The mechanism stays off by default everywhere.** `config/delegated-implementation.yaml` stays
  `status: disabled`; no downstream status file is scaffolded.
- **Every existing safety property is preserved, not renegotiated** — candidate staging only, never
  `modules/`/`tests/` in the real tree, never a commit, no key leakage, fail-closed preconditions,
  **and the tool continues to execute nothing on the local machine.** The repair iteration is
  therefore scoped to **option B below**: the tool accepts build output as *input*; it does not run a
  build. Options A and C are recorded as alternatives the human may substitute at this gate — if A is
  chosen, this bullet no longer holds and the scope boundary must be rewritten before Step 2.
- No change to `tools/reviewer/`; no DeepSeek provider added there.
- No new mandatory human-approval gate, no new Stage ID, no Non-Negotiable Rule change.
- This change makes no claim about whether delegation is net-positive. It only makes the next
  measurement interpretable.

**Class:** prompt + script-tooling
**Scope axis:** self-dev only
**Backlog item:** `backlog/UPG-0060-deepseek-delegated-implementation.md` (re-test condition 0)

---

## Design position taken, and the alternative the human may substitute

**This change takes option B: the tool accepts build output as input and never runs a build itself.**
That position is what the scope boundary above asserts, and it is what Step 2's acceptance criteria
will be written against. The alternatives are recorded here so the human can substitute one at this
gate; A in particular would require rewriting the scope boundary first, because it gives up a safety
property this change otherwise claims to preserve.

The repair iteration needs compiler output. Today the tool executes nothing local — it makes one
network call and writes files. Running `cargo build` (or any project build) on a downstream repository
means executing arbitrary project code: build scripts, proc macros, test harnesses. That is a
categorical change to what this tool is, which is why it is surfaced at the gate rather than decided
inside the implementation.

| Option | Effect |
|---|---|
| **A. Tool runs the build** | Fully automatic repair loop. Gives up the "executes nothing local" property and inherits arbitrary-code-execution risk from any downstream repo it is pointed at |
| **B. Tool accepts build output as input** *(recommended)* | The human or Claude runs the build and re-invokes with the errors. Preserves "executes nothing local," delivers the same feedback signal, costs a small amount of Claude token budget per iteration |
| **C. No repair iteration in this change** | Smallest change; leaves re-test condition 0 only partly satisfied, so the re-test still cannot show whether repair converges |

I recommend **B**. The feedback signal is what the measurement needs, and it does not cost the tool its
read-only-locally character — which is the same property that makes the Codex reviewer safe to point at
any repository. The token cost of running a build and re-invoking is small relative to the generation
it is meant to protect.

---

## Acceptance Criteria

<!-- Step 2. Not yet written — this record is at Step 1. -->

*(pending Step 2)*

---

## Implementation Notes

*(pending Step 3)*

---

## Reconciliation

*(pending Step 4)*
