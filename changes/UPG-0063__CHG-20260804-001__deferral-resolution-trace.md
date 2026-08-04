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
state: COMPLETE         # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 4-Reconcile  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0063
related_features: [UPG-0062, UPG-0051, UPG-0058]
review_series: RVS__UPG-0063__CHG-20260804-001__S4   # S1, S2, S3 ACCEPTED
review_profile: PROFILE-4   # touches prompts/04-implement.md — downstream doctrine (Step 0a)
review_state: ACCEPTED  # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round)
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

*Amended at the Step 1 gate (human, 2026-08-04): safeguard 1 adds a narrow `tools/reviewer/` change.
Step 1's original list did not include it; the amendment is recorded rather than applied silently.*

- `prompts/04-implement.md` — **modified, if the hypothesis holds.** Add a short
  **Deferral → Resolution** subsection to the existing Stage 4 output format, with the five fields
  below, populated only when a material explicit deferral was resolved. Extending the existing output
  is strongly preferred over adding anything new.
- `tools/reviewer/src/packet.rs` — **modified.** One line: `stage_checks("4")` gains the reviewer's
  active question (safeguard 1). No other match arm, and no engine, provider, packet-architecture,
  config, or CLI change.
- `tools/reviewer/tests/` — **modified.** Coverage for that checklist content.
- `templates/` — **only if** Step 3 shows the prompt alone cannot carry it. Not assumed.
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
- **`tools/reviewer/` changes are confined to the stage-4 checklist string and its test** (see
  safeguard 1). No provider, packet-architecture, config, or CLI change; no other stage's checklist or
  expected-output string is touched.
- No change to `scripts/` or any delegation tooling. UPG-0060 and UPG-0062 are closed and this is not
  a route back to either.
- **The PlotSpot defect found during Q0 is out of scope** — filed as
  `PlotSpot/refinements/F-0001-known-access-form-canonicalization.md`, PlotSpot's to triage. This
  change must not become a bug-fixing change.

**Class:** downstream-doctrine + script-tooling (modifies `prompts/04-implement.md`, which downstream
projects load, plus a one-line stage-4 reviewer checklist)
**Scope axis:** downstream doctrine only — the reviewer change serves the downstream Stage 4 gate and
introduces no self-dev behavior
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

**Resolved at the gate — see "Step 1 gate" below. Retained for provenance.** The unresolved
consequence, and the honest open question that was put to the gate: a semantic definition is
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

## Step 1 gate — decisions carried into Step 2

**The weaker enforcement model is ACCEPTED** (human, 2026-08-04). This feature governs *traceability of
an explicit deferral resolution*, not runtime correctness, and requiring mechanical detection would
make the mechanism worse than the problem: phrase matching is bypassable, mandatory tags everywhere
add doctrine weight, and automatic enumeration of semantic deferrals is unrealistic.

**The rule to be stated at Stage 4:**

> When Stage 4 resolves a material question that an approved upstream artifact explicitly deferred,
> the implementation author must record a Deferral→Resolution trace. The Stage 4 review checks this
> obligation as part of artifact reconciliation. **Absence is a traceability defect, not an automatic
> implementation failure.**

**Two safeguards, both in scope:**

1. **The reviewer asks actively**, rather than relying on the author's memory — the Stage 4 checklist
   carries the question directly.
2. **No automatic deferral discovery.** If later evidence shows omissions are common, that justifies a
   separate improvement — not scope growth here.

**Scope consequence:** safeguard 1 means `tools/reviewer/` is now in scope, narrowly — `stage_checks`
for stage `"4"` is a single string (`tools/reviewer/src/packet.rs:688`), which is exactly where a
Stage 4 reviewer check belongs. This is a one-line checklist extension plus test coverage, not an
engine change.

---

## Acceptance Criteria

<!-- downstream-doctrine + a narrow reviewer checklist line. Verification is by reading the changed
text, by the reviewer engine's own tests, and by a retrofit sanity check against Q0's real cases. -->

### Group 1 — the Stage 4 obligation

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | **The obligation is stated in `prompts/04-implement.md`** in the terms agreed at the gate, including that absence is a **traceability defect, not an automatic implementation failure**. | Read the added text against the rule quoted above. |
| 2 | **A deferral is defined semantically**, and the definition explicitly excludes (a) **silence** — an artifact that never mentions a question has deferred nothing — and (b) **implementation freedom** — behavior settled, technique open. Both exclusions are present, not implied. | Read; both exclusions appear with their reasons. |
| 3 | **No phrase list is normative.** If example phrasings appear at all, they are marked as illustration or discovery assistance and never as the trigger for the obligation. | Grep the added text for any list presented as definitional; confirm none. |
| 4 | **Exactly the five fields**: source artifact + deferral; chosen resolution; where implemented; final or interim; expected superseder when interim. No sixth field is added. | Read the format. |
| 5 | **Materiality gates entry.** Only deferrals whose resolution determines invariant placement, component responsibility, state/data integrity, or future architectural freedom. | Read the stated gate. |
| 6 | **Empty is the normal case and costs nothing** — no empty table, no "none" line, no ceremony when nothing was deferred. | Read the instruction for the empty case. |

### Group 2 — the reviewer safeguard

| # | Criterion | How it will be verified |
|---|---|---|
| 7 | **The Stage 4 reviewer checklist asks the question directly** — in substance: *did implementation resolve any question an approved artifact explicitly deferred, and is each material resolution recorded?* | Read `stage_checks("4")`; generate a stage-4 packet and confirm the question appears under STAGE-SPECIFIC CHECKS. |
| 8 | **Only stage 4 changes.** No other stage's checklist or expected-output string is altered. | `git diff` of `packet.rs`; every other match arm byte-unchanged. |
| 9 | **Reviewer test coverage exists** for the stage-4 checklist content, and the full existing suite still passes. | `cargo test` in `tools/reviewer/`; count compared against the pre-change baseline. |

### Group 3 — what must NOT happen

| # | Criterion | How it will be verified |
|---|---|---|
| 10 | **No automatic deferral discovery.** Nothing scans, greps, or enumerates artifacts for deferrals — not in the prompt, not in the engine. | Read both changed files; no scanning logic, no phrase-matching. |
| 11 | **No new stage, no new gate, no standalone artifact.** The existing Stage 4 human gate reviews this. `dba-system.md`'s stage table, Non-Negotiable Rules, and stage count are unchanged. | `git diff --stat`; read `dba-system.md`. |
| 12 | **The trace never becomes a second architecture authority.** It is stated as subordinate, and a conflict with an approved artifact is **reconciled** — never silently resolved in the trace's favour, and never by the trace overriding or reinterpreting the artifact. | Read the added text. |
| 13 | **`tools/reviewer/` changes are confined to the stage-4 checklist string and its test.** No provider, packet-architecture, config, or CLI change. | `git diff --stat` over `tools/reviewer/`. |
| 14 | **No other downstream stage prompt changes.** `prompts/01-intent.md`, `02-contract.md`, `03-*.md`, `05-tests.md`, `06-observe.md`, `07`–`10` byte-unchanged. | `git diff --stat`. |

### Group 4 — downstream compatibility and fitness

| # | Criterion | How it will be verified |
|---|---|---|
| 15 | **Downstream compatibility holds.** A generated project still loads `.codeos/dba-system.md`; stage names, prompt filenames, and cross-references move together; no reference is orphaned. | `grep` cross-reference sweep; `scripts/dba-init.sh` scratch run. |
| 16 | **Retrofit sanity check — the five fields actually fit the real cases.** Write the trace for Q0's three confirmed instances (PlotSpot validation-ordering; PlotSpot vocabulary-ownership; EA-0001 validator seam) and confirm each is expressible without inventing a field, and that the interim/superseder fields do real work on the vocabulary case. | The three retrofitted records appear in the change's evidence; any field that proves unusable or missing is reported rather than silently accommodated. |
| 17 | **The retrofit is retrospective examples only — never historical backfill.** (Human guardrail, 2026-08-04.) The three traces live in this change's evidence as clearly-marked test fixtures. **No downstream artifact is modified by the retrofit**: no PlotSpot or EvidenceAtlas contract, schema, intent, or module is touched, and no historical governance state is rewritten as though these traces existed when those implementations were approved. Backfilling real downstream artifacts, if ever wanted, is a separate decision. *(One file was added to PlotSpot this session — `refinements/F-0001-known-access-form-canonicalization.md` — at explicit human instruction and as a separate action from this change: it is a new advisory refinement candidate, not a modification of any approved artifact, and is not part of this change's diff.)* | `git status` in each downstream project shows no artifact modification attributable to this change; the fixtures file carries the retrospective marking on its face. |

**Explicitly not in scope for these criteria:** whether the obligation is *complied with* in practice.
That needs downstream features to pass through Stage 4 after this ships, and is the evidence that
would later justify stronger machinery — per the gate decision, stronger machinery is not added
pre-emptively.

---

## Implementation Notes

<!-- Factual reporting. The git diff is the source of truth. -->

**Doctrine — `prompts/04-implement.md`.** New item 5 in the Output Format, between the Failure Mapping
Table and the Review Package; the closing state item renumbers 6 → 7. It carries the semantic
definition, both exclusions with their reasons, the materiality gate, the five-column table, the
subordination-and-reconciliation rule, and the traceability-defect-not-implementation-failure
statement. Opens with *"only if"* and instructs omitting the section entirely when nothing was
deferred — no empty table, no "none".

Placement is deliberate: the existing Review Package already has a **"Key architectural decisions"**
field, whose examples (*internal data structure, error propagation strategy*) are exactly the
`ORDINARY IMPLEMENTATION CHOICE` category. The trace is a different thing — keyed to an upstream
deferral, carrying interim/superseder — so it sits as its own item and the prompt explicitly routes
technique choices back to the existing field, rather than the two silently competing.

**Reviewer — `tools/reviewer/src/packet.rs`.** One added line inside `stage_checks("4")`. No other
match arm touched; no engine, provider, packet-architecture, config, or CLI change.

**Tests.** Two added in `packet.rs`: one asserts the stage-4 checklist asks the question and keeps it
advisory and semantic; one asserts no other stage carries it. **184 tests pass** (182 before).

**A stale-binary gap the unit tests could not catch.** After the source change, a generated stage-4
packet still showed only the old checklist line — `scripts/codeos-review.sh` runs the prebuilt
`tools/reviewer/target/release/codeos-reviewer`, so source changes are invisible until rebuilt. Tests
passed against source the deployed binary did not contain. Rebuilt with
`cargo build --release --manifest-path tools/reviewer/Cargo.toml`, then re-verified against a freshly
generated packet, which now shows both lines. **AC-7 is satisfied by the packet, not by the unit
test** — the unit test alone would have been false assurance.

**Retrofit (AC-16/17).** `changes/UPG-0063__CHG-20260804-001__retrofit-fixtures.md`, carrying a
retrospective-examples banner. All three Q0 cases are expressible in five fields with nothing
invented. `Final / Interim` + `Expected Superseder` are decisive on the PlotSpot vocabulary case
(interim by the contract's own wording, with nothing today recording it), useful on EA-0001, and
correctly inert on validation-ordering.

Two limitations **reported rather than accommodated**: `Expected Superseder` is only as precise as the
deferral it points at (EA-0001's contract says rows are pending without naming what settles them —
a property of the artifact, not a schema defect, and a sixth field would not fix it); and
`Where Implemented` line numbers drift, so the durable part is `file:function`.

**Downstream compatibility (AC-15).** `dba-init.sh` scratch run in a fresh git repo: `.codeos`
symlink resolves, `.codeos/dba-system.md` reachable, and the generated project's
`.codeos/prompts/04-implement.md` carries the new section.

**Scope (AC-11/13/14).** Changed: `prompts/04-implement.md` (+45/-2), `tools/reviewer/src/packet.rs`
(+35/-1), this record, the fixtures file. `dba-system.md` byte-unchanged; no other stage prompt
changed; no `scripts/` or delegation-tooling change.

**Assumptions:** the Stage 4 author reads the approved artifacts closely enough to notice an explicit
deferral. That is the enforcement model accepted at the Step 1 gate, not an oversight.

---

## Reconciliation

<!-- Layer D1: advisory verdict, evidence separated from inference. -->

**All 17 acceptance criteria PASS. Accepted by the human 2026-08-04**, after S4 R2 returned NO
OBJECTION (evidence A, zero findings) — in that order. An earlier draft of this section set
`COMPLETE`/`ACCEPTED` before either the review or the acceptance had happened; it was reverted and is
recorded in the triage table below, because claiming acceptance ahead of the gate undercuts the
guarantee the gate exists to provide. Raw verification output is embedded in
`changes/UPG-0063__CHG-20260804-001__retrofit-fixtures.md` §"Raw verification output" rather than
summarised here.

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Obligation stated, absence is a traceability defect not an implementation failure | PASS | `04-implement.md:215-218` — "that is a traceability defect — the Stage 4 review will ask — not an automatic implementation failure" |
| 2 | Semantic definition; both exclusions present with reasons | PASS | `:182-195` — **Silence** ("otherwise you would owe a record of everything the artifacts failed to say") and **Implementation freedom** ("choosing a data structure … resolves no deferral") |
| 3 | No normative phrase list | PASS | Grep for `not prescribed` / `MANUAL-PENDING` in the added text returns **nothing** — no phrase list is present at all, normative or otherwise. The instruction is "judge this by meaning, never by matching particular phrases" |
| 4 | Exactly five fields | PASS | `:200` — the table header carries five columns and no sixth |
| 5 | Materiality gates entry | PASS | `:196-198` — the invariant / responsibility / state-model / data-integrity / future-freedom test |
| 6 | Empty is normal, no ceremony | PASS | `:179-180` — "omit this section entirely. Do not write an empty table and do not write \"none\"" |
| 7 | Stage 4 checklist asks the question | PASS | **Verified against a generated packet, not the source** — `reviews/codex/packets/20260804T100605Z-UPG-0063-ac7-recheck-stage-4-*.packet.txt` shows both checklist lines under STAGE-SPECIFIC CHECKS |
| 8 | Only stage 4 changed | PASS | `git diff` of `packet.rs` touches no other match arm |
| 9 | Reviewer tests exist; suite passes | PASS | 2 tests added; **184 pass**, up from 182 |
| 10 | No automatic deferral discovery | PASS | Diff of both changed files contains no grep, scan, regex, enumeration, or search logic |
| 11 | No new stage, gate, or standalone artifact | PASS | `dba-system.md` byte-unchanged; stage table, Non-Negotiable Rules and stage count untouched |
| 12 | Trace subordinate; conflict reconciled | PASS | `:208-213` — "subordinate to the approved artifacts … that conflict must be **reconciled** … implementation may not legitimately continue until the artifact is amended through its own governance path" |
| 13 | `tools/reviewer/` diff confined | PASS | Only `packet.rs` (+35/-1): the stage-4 string and its two tests. No provider, packet-architecture, config, or CLI change |
| 14 | No other downstream stage prompt changed | PASS | `git diff --stat prompts/` lists `04-implement.md` alone |
| 15 | Downstream compatibility | PASS | `dba-init.sh` scratch run: `.codeos` symlink resolves, `dba-system.md` reachable, generated `04-implement.md` carries the new section |
| 16 | Five fields fit the real cases | PASS | Three retrofit fixtures, nothing invented; two limitations reported rather than accommodated |
| 17 | Retrospective examples, not historical backfill | PASS | No PlotSpot or EvidenceAtlas contract/schema/intent/module touched; pre-existing dirt in both predates this change; the one PlotSpot file added is a separate human-instructed refinement candidate, disclosed in the criterion itself |

**Stale-reference sweep.** Clean. The renamed brief
(`UPG-0063-feature-implementation-design-layer.md` → `UPG-0063-deferral-resolution-trace.md`) is
referenced only inside `reviews/codex/*` — historical assessments and packets recording what was true
when they ran. Those are immutable by design and are **not** stale references to repair
(SELF-REFERENCE / REVIEW-BOOKKEEPING). No template, `dba-system.md` section, or `README.md` entry pins
the Stage 4 output item numbering, so renumbering 6 → 7 orphaned nothing.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| S1 R1: Q0's grep-enumerability claim contradicted the semantic definition | IN-SCOPE BLOCKER | Fixed — claim withdrawn by appended correction, original left standing |
| S2 R1: scope contradiction (`tools/reviewer/` in the gate section, excluded in the boundary); dashboard/record disagreed on S1 approval | IN-SCOPE BLOCKER | Fixed — scope, class and axis amended at the gate and recorded as an amendment |
| S3 R1: dashboard Loop step stale | IN-SCOPE BLOCKER | Fixed. **AJ-020 recurrence** — the journaled rule is specifically that the row is updated before the review runs |
| S3 R1: AC-9/15/17 asserted rather than evidenced | IN-SCOPE NON-BLOCKER | Fixed — raw output embedded (AJ-016) |
| Stale release binary hiding a correct source change | IN-SCOPE BLOCKER (self-found) | Fixed by rebuild; AC-7 re-verified against a generated packet. Not reviewer-raised — the packet evidence would have looked fine to a reader trusting the unit tests |
| AC-17 originally claimed "nothing is written into PlotSpot" | IN-SCOPE BLOCKER (self-found) | Fixed before review — the human-instructed refinement candidate made it literally false |
| Recurring AJ-016 / AJ-020 bookkeeping failures | OUT-OF-SCOPE BACKLOG | **UPG-0061.** Not addressed here, per the human's instruction. The signal worth carrying: the rules already exist and still fail to change behavior at the required moment — a knowledge-application problem, not a knowledge gap |

**Honest assessment.** The mechanism is small and the boundary held: conditional output, ordinary
choices routed elsewhere, one advisory reviewer line, no scanning machinery, no new gate. What it is
*not* yet is proven in use — no downstream feature has passed through Stage 4 under this obligation.
Whether authors notice deferrals, and whether the reviewer's question surfaces omissions, is evidence
that can only come later, and is what would justify anything stronger. Per the gate decision, nothing
stronger is added pre-emptively.
