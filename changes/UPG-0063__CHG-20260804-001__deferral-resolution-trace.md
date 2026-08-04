# Self-Development Change: UPG-0063__CHG-20260804-001 — deferral-resolution-trace

<!--
PURPOSE: Per-change source of truth for the first change of UPG-0063 — determine whether a lightweight
Deferral -> Resolution trace can live inside the existing Stage 4 workflow, and if so, add it. The
working hypothesis is deliberately the leanest thing that could work; anything heavier must argue for
itself against evidence. Workflow: prompts/codeos-self-dev.md (4-step loop).
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0063
primary_feature_id: UPG-0063
change_id: CHG-20260804-001
slug: deferral-resolution-trace
state: DRAFT            # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 1-Intent  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0063
related_features: [UPG-0062, UPG-0051, UPG-0058]
review_series: RVS__UPG-0063__CHG-20260804-001__S1
review_profile: PROFILE-4   # touches prompts/04-implement.md — downstream doctrine (Step 0a)
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

Approved artifacts sometimes explicitly defer a design or behavioral question — naming it and stating
that the artifact does not settle it. Stage 4 must resolve some of those deferrals for an
implementation to exist. Codeos records nothing about how the deferral was resolved, whether the
resolution is interim, or whether an upstream artifact must later supersede it.

Q0 established this across two independent DBA projects
(`changes/UPG-0063__Q0-classification-evidence.md`, method precommitted at `1b0dbd1`). The sharpest
case: PlotSpot's F-0001/2/3 contracts state that vocabulary *"canonical ownership is unresolved until
Architecture Synthesis"*; all three implementations resolved it with a hardcoded local map; nothing
anywhere records that those maps are interim and must move when Synthesis lands.

**What this change does NOT rest on.** UPG-0062's claim that approved artifacts "do not determine the
mechanism" was **retracted** — it came from grepping contracts for the implementation's vocabulary and
reading absence of the code's names as absence of the rule. The artifacts determine a great deal. This
change rests only on the narrower, artifact-attested finding above. Full correction in
`changes/UPG-0060__CHG-20260803-002__premise-test-evidence.md` §5.

**What changes:**

Scope is contingent on Step 2's evidence and is stated here as the *hypothesis to be tested*, not as a
commitment:

- `prompts/04-implement.md` — **modified, if the hypothesis holds.** Add a short
  **Deferral → Resolution** subsection to the existing Stage 4 output format, with the five fields
  below, populated only when a material explicit deferral was resolved. Extending the existing output
  is strongly preferred over adding anything new.
- `templates/` — **only if** Step 2 shows the prompt alone cannot carry it. Not assumed.
- `changes/UPG-0063__CHG-20260804-001__deferral-resolution-trace.md` — **new**, this record.
- Lifecycle bookkeeping: `backlog/features.md`, `status/self-development.md`, `status/roadmap.md`.

The recorded fields: source artifact + deferral; chosen resolution; where implemented; **final or
interim**; and if interim, the expected superseder.

**Scope boundary — what stays the same:**

- **No new DBA stage, no new approval gate, no standalone design artifact.** The existing Stage 4
  human gate reviews this, or the hypothesis has failed and the change stops.
- **No survey of all implementation decisions**, no `SOURCE-DERIVED` inventory, nothing resembling
  UPG-0062's Feature Implementation Design. Only material deferrals actually resolved.
- **The trace never becomes a second architecture authority.** Approved artifacts stay authoritative;
  a conflict is reconciled through the existing governance path, never resolved by the trace.
- **No phrase list is normative.** A deferral is defined semantically (see below). Phrase search may
  assist discovery and may never define the obligation.
- No change to `dba-system.md`'s stage table, Non-Negotiable Rules, or any other stage prompt.
- No change to `tools/reviewer/`, `scripts/`, or any delegation tooling. UPG-0060 and UPG-0062 are
  closed and this is not a route back to either.
- **The PlotSpot defect found during Q0 is out of scope** — filed as
  `PlotSpot/refinements/F-0001-known-access-form-canonicalization.md`, PlotSpot's to triage. This
  change must not become a bug-fixing change.

**Class:** downstream-doctrine (modifies `prompts/04-implement.md`, which downstream projects load)
**Scope axis:** downstream doctrine only
**Backlog item:** `backlog/UPG-0063-deferral-resolution-trace.md`

---

## The definitional problem Step 2 must solve

This is the load-bearing question, and it is a definition problem rather than a mechanism problem.

**A deferral is:** a statement in an approved artifact that a specific design or behavioral question
is deliberately left unresolved *by that artifact*, whatever wording is used.

**It must be distinguished from two neighbours it superficially resembles:**

| Not a deferral | Why it matters |
|---|---|
| **Silence** — the artifact simply never mentions the question | If silence counted, the obligation would be unbounded: every feature would owe a record of everything its artifacts failed to say. A deferral is an *affirmative* statement of non-resolution |
| **Implementation freedom** — the artifact settles the behavior and leaves the technique open | Choosing a `BTreeSet` resolves nothing. This is what keeps the trace from degenerating into a design diary |

**Why a phrase list cannot be the definition.** Scanning for *"not prescribed"*, *"unresolved"*,
*"MANUAL-PENDING"* is how Q0 found its candidates and is genuinely useful. But if the phrase list were
normative, an author could write an equivalent deferral in different words and the obligation would
silently not attach — governance a synonym defeats is not governance. It would also produce false
positives on prose that merely contains the words. Phrase search is **discovery assistance only**.

**The unresolved consequence, and the honest open question for the gate:** a semantic definition is
correct but not mechanically checkable. A missing record cannot be detected by grep without
reintroducing the phrase-dependence the definition rejects. So the obligation likely rests on the
Stage 4 author identifying the deferral and the human gate catching omissions — which is weaker
enforcement than Codeos usually accepts, and Step 2 must decide whether that is good enough or whether
it sinks the lean hypothesis.

My inclination: accept the weaker enforcement. The deferrals are *in the approved artifacts the Stage
4 author is already required to read*, an unrecorded resolution is not a correctness failure but a
traceability one, and the alternative — a checkable phrase convention — buys enforceability by making
the mechanism bypassable and noisy. But this is a genuine trade and it is the human's call at the
gate.

---

## Acceptance Criteria

*(pending Step 2)*

---

## Implementation Notes

*(pending Step 3)*

---

## Reconciliation

*(pending Step 4)*
