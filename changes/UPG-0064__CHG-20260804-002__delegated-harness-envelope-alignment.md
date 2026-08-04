# Self-Development Change: UPG-0064__CHG-20260804-002 — delegated-harness-envelope-alignment

<!--
PURPOSE: CHG-A of UPG-0064. Makes the governed Stage-4 envelope (Architecture Baseline, Cohort
Logical Design, Implementation Profile, and UPG-0063's deferral rule) visible and binding to the
delegated implementer. Harness alignment only — no pilot, no measurement, no adoption claim. The
three-case pilot is CHG-B and does not begin until this is accepted.
Workflow: prompts/codeos-self-dev.md (4-step loop).
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0064
primary_feature_id: UPG-0064
change_id: CHG-20260804-002
slug: delegated-harness-envelope-alignment
state: IN_PROGRESS      # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 1-Intent  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0064
related_features: [UPG-0051, UPG-0052, UPG-0063, UPG-0060, UPG-0062]
review_series: RVS__UPG-0064__CHG-20260804-002__S1
review_profile: PROFILE-3   # prompt + script-tooling, self-dev only (Step 0a)
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

UPG-0051, UPG-0052 and UPG-0063 built a governed envelope around Stage 4. The delegated
implementation path never received it. Measured against `prompts/codeos-implementer-task.md` — the
only prompt the delegate sees — "Architecture Baseline", "Implementation Profile", "Cohort" and
"deferral/deferred" each occur **zero** times. Its output contract asks for three loose sections and
has no notion of a deferral trace. `scripts/codeos-implement.sh` labels every input identically as
`--- APPROVED ARTIFACT: <path> ---`, so a behavioral contract and an Architecture Baseline are
indistinguishable to the model.

This is an **integration defect in the delegation harness**, not another architecture-design problem.
The envelope is complete; the delegated execution path was never updated to carry it. UPG-0062 planned
the prompt rewrite that would have fixed part of it and closed on cost before Step 3.

**Why it must be fixed before the pilot.** The Stage-4 reviewer checklist now asks for the deferral
trace. A pilot today would hand the delegate a feature containing an explicit deferral, never tell it
deferrals exist or that resolving one incurs an obligation, then measure whether it recorded one — a
harness defect reported as a model defect. That is precisely UPG-0060's documented error, and
repeating it against our own written correction would be worse than making it the first time.

**What changes:**

- `scripts/codeos-implement.sh` — **modified.** Artifacts are labelled by **authority role** instead
  of the flat `APPROVED ARTIFACT`: behavioral contract, event schema, architecture baseline, cohort
  logical design, implementation profile, layout exemplar. The label states how each input binds. Role
  is supplied by the caller, not inferred from filenames — the tool must not guess an artifact's
  authority from a path.
- `prompts/codeos-implementer-task.md` — **modified.** States each role's authority: contract =
  behavior to satisfy; event schema = events to emit correctly; baseline and cohort logical design =
  binding architectural constraints, **not** behavior to invent; implementation profile = binding
  implementation constraint; exemplar = context, not authority. Adds UPG-0063's rule semantically —
  report a resolution only when an approved artifact **explicitly deferred** a material decision,
  using the five fields; exclude ordinary technique choices and matters merely unspecified rather than
  deferred; no phrase list is normative. Adds a `deferral_resolution` output section.
- `scripts/tests/codeos-implement-tests.sh` — **modified.** Coverage for role labelling and the new
  output section, including its absence when nothing was deferred.
- `changes/UPG-0064__CHG-20260804-002__delegated-harness-envelope-alignment.md` — **new**, this record.
- Lifecycle bookkeeping: `backlog/features.md`, `status/self-development.md`, `status/roadmap.md`.

**Scope boundary — what stays the same:**

- **No pilot, no measurement, no adoption claim.** CHG-A ships harness alignment only. CHG-B is the
  three-case pilot and does not begin until this is accepted.
- **The delegate produces a candidate, never the authoritative Stage-4 report.** It returns code and
  evidence; Codeos/Claude assembles the Review Package. Making the delegate emit the canonical
  artifact would change the experiment from "can it satisfy an approved envelope?" to "can it also
  operate Codeos's governance protocol?" — different questions, different failure modes.
- **Stage 5 delegation is out**, and does not inherit any Stage-4 result. A model can implement a
  contract reasonably while writing tests that confirm its own interpretation rather than falsify the
  contract. Separate experiment, only on positive Stage-4 evidence.
- **The mechanism stays off by default** — `config/delegated-implementation.yaml` remains
  `status: disabled`; no downstream status file is scaffolded.
- Every existing safety property is preserved: candidate staging only, never `modules/`/`tests/` in
  the real tree, never a commit, no key leakage, fail-closed preconditions, and **the tool executes no
  build, test, or project-supplied command** (UPG-0060's Option B boundary).
- **No downstream doctrine change.** `dba-system.md`, `prompts/04-implement.md`, and every other stage
  prompt are untouched — UPG-0063's Stage-4 output format is *consumed* here, never redefined.
- No change to `tools/reviewer/`. No new stage, gate, Stage ID, or Non-Negotiable Rule change.
- **UPG-0060's conclusion is not revisited.** That the delegate cannot *derive* an architecture from a
  contract stands. This asks whether it can operate inside one that is supplied and labelled.

**Class:** prompt + script-tooling
**Scope axis:** self-dev only
**Backlog item:** `backlog/UPG-0064-delegated-stage4-envelope-alignment.md`

---

## Open question for the gate

**How does the tool know an artifact's role?**

Roles cannot be inferred from filenames — `architecture/core-baseline.md` is conventional, not
guaranteed, and a downstream project may name things differently. Guessing would put the tool in the
business of classifying authority, which is exactly what it must not do.

| Option | |
|---|---|
| **A. Explicit per-artifact flags** *(recommended)* — `--contract PATH`, `--event-schema PATH`, `--architecture PATH`, `--cohort-design PATH`, `--profile PATH`, existing `--exemplar PATH` | The caller declares authority; the tool transports it. Verbose at the call site, unambiguous, and no inference |
| **B. Infer from path conventions** | Concise, but the tool would be deciding what is architecturally binding — wrong actor, and silently wrong on any project that names files differently |
| **C. A manifest file** | Another artifact to keep in sync; disproportionate for six inputs |

I recommend **A**, and note it makes the existing bare positional `<artifact-path>` form ambiguous —
Step 2 must decide whether unlabelled positionals stay supported (as "contract-or-schema, unspecified")
or become an error. My inclination is to keep them working and label them
`--- APPROVED ARTIFACT (role unspecified) ---`, so the caller sees the degradation rather than the
model silently receiving a mislabelled authority.

---

## Acceptance Criteria

*(pending Step 2)*

---

## Implementation Notes

*(pending Step 3)*

---

## Reconciliation

*(pending Step 4)*
