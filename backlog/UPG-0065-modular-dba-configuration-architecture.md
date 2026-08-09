---
feature_id: UPG-0065
slug: modular-dba-configuration-architecture
title: Modular DBA Configuration Architecture
status: IN_PROGRESS
priority: P1
depends_on: []
related_features: [UPG-0001, UPG-0051, UPG-0052, UPG-0056, UPG-0057, UPG-0058, UPG-0059]
supersedes: []
superseded_by: []
---

# Upgrade: modular-dba-configuration-architecture — Modular DBA Configuration Architecture

**Priority**: P1
**Status**: IN_PROGRESS — CHG-20260807-001 (Normative Delta Inventory) **COMPLETE**, accepted
2026-08-08: 203-row disposition of every normative rule in `dba-system.md`. Phase A's second
sub-step, CHG-20260808-001 (v1 Component Decomposition), is also **COMPLETE**, accepted
2026-08-08: 6 candidate `v1` component files under `dba/` (175 rows), transcribed from pinned
`dba-system.md` @ `77599e9` source text. **Post-acceptance hygiene fix (2026-08-08, human-authorized,
no new review round):** `CHG-20260807-001`'s delta table had 7 `source_anchor` line ranges
corrected in place per `AJ-024` — disposition/target_owner/rationale unchanged for all 7; see the
delta table's own correction note. Phase A's third sub-step, CHG-20260808-002 (Downstream
Consumer Compatibility Sweep), is also **COMPLETE**, accepted 2026-08-09: a 49-row catalog
(`changes/UPG-0065__CHG-20260808-002__compatibility-report.md`) of every `dba-system.md`
reference in `prompts/`/`scripts/`/`templates/`/`patterns/`, classifying compatibility with the
eventual thin-manifest activation. Phase A's fourth sub-step — CHG-20260809-001 (`DBA-1`
Equivalence Proof) — Step 1 **ACCEPTED** (human approved 2026-08-09); Step 2 (8 acceptance
criteria) **ACCEPTED** (human approved 2026-08-09, R1-R2 DO NOT ADVANCE fixed, R3 NO OBJECTION).
**COMPLETE**, accepted 2026-08-09: built `dba/configurations/DBA-1.yaml` (candidate) and
`changes/UPG-0065__CHG-20260809-001__equivalence-report.md`, proving `DBA-1` (all six components at
`v1`) configuration-equivalent to `dba-system.md` @ `77599e9` — Invariant 1(a)-(b). All 8 ACs PASS,
independently re-verified at Reconcile. AC3's full 175-row content-fidelity re-verification found
163 clean and 12 citation-precision/wording defects in `dba/*/v1.md` (none changing disposition,
ownership, or meaning) — collided with this change's own read-only scope boundary on those files;
**explicitly waived by human decision** as non-blocking for `DBA-1`, tracked as a follow-up hygiene
item (no dedicated `UPG-####` filed). Step 3's own review caught and fixed a real self-contradiction
(AC3's original "not waived" wording forbade the outcome the artifact recorded) plus the resulting
governance-state staleness. **`DBA-1` is now proven equivalent but remains `status: candidate` — not
approved (Invariant 1(c)), not activated (1(d)).** Approval (1(c)) is now in progress as its own
change, `CHG-20260809-002`; activation (1(d)) remains separate, later, and not yet started. Phase
A's fifth sub-step, `CHG-20260809-002` (`DBA-1` Approval), Step 1 **ACCEPTED** (human approved
2026-08-09). Step 2 (6 ACs) drafted, binding a narrow post-approval `DBA-1.yaml` schema (only
`status`/`approved_at`/`approved_via` change) and an equivalence-currency precondition: AC1
requires `git diff 1b8f984 -- dba-system.md dba-system-lean.md dba/doctrine/ dba/policies/
dba/tools/ changes/UPG-0065__CHG-20260807-001__delta-table.md` to be empty at Reconcile; run as a
pre-review self-check before Step 2's own review, the command returned empty (no drift) — the
formal, binding verification still happens fresh at Reconcile per AC1's own text, not substituted
by this self-check. R1-R3 progressively fixed AC2's sequencing claim: it originally relied on
unreconstructable working-tree history, then (after narrowing to durable co-presence evidence)
still asserted an unprovable "never during Steps 1-3" timing claim, then a false present-tense
reference to Implementation Notes (still an empty Step-3 placeholder) — each fixed inline.
**PROFILE-4's 3-round budget exhausted at R3; fixed inline, no automatic R4. Human approved
2026-08-09 — proceeding to Step 3.** Step 3 re-verified AC1 fresh (no drift since
`CHG-20260809-001`) and deliberately made no edit to `DBA-1.yaml` — the approval flip happens only
at Reconcile, after the Step 4 human decision itself is logged. R1 DO NOT ADVANCE (dashboard's
Loop-step column still read `2-Acceptance` after Step 3 began) — fixed. **COMPLETE**, accepted 2026-08-09. **`DBA-1` is explicitly approved as the migration baseline per
Invariant 1(c).** `dba/configurations/DBA-1.yaml` is now `status: approved`, `approved_via:
UPG-0065__CHG-20260809-002`. All 6 ACs PASS at Reconcile — AC2 and AC6's evidence quoted verbatim
inline in the change record after `--sha-only` packet inclusion proved insufficient twice.
Activation (1(d)) remains a separate, later, **now-started** change. Human decision (2026-08-09):
activation uses the **minimal-manifest design** — `dba-system.md` becomes a short pointer to the
active `DBA-N`, not a file that keeps its current section structure as pointers. Split into two
changes for safety (applying Invariant 2's "no partially-migrated state" spirit to the migration
*process*, not just the end state): `CHG-20260809-003` (`consumer-citation-migration`, Step 1
drafted, original scope before the Step 2 correction below) migrated all 34 `STRUCTURAL-POINTER`
citations + 5 `WHOLE-FILE-LOAD` instructions across 16 consumer files to the correct `dba/*/v1.md`
locations, while `dba-system.md` stays completely unchanged; the atomic `dba-system.md` swap itself
is a separate, later, not-yet-filed change, safe only once this one is COMPLETE. R1-R3
progressively fixed: unsupported "safe/reversible" claims (reworded to two checkable properties —
plain-revert reversibility, and a direct check that none of the (then) 34+5 retargeted citations
point at content compromised by `CHG-20260809-001`'s 12 waived defects); stale trace-header review
metadata after R1 ran; the top-level `state` field left at
`DRAFT` and a Self-Reference Boundary violation (embedding exact round numbers in the trace
header) introduced by the prior fix. **PROFILE-4's 3-round budget exhausted at R3; fixed inline,
no automatic R4. Human verdict: APPROVED — proceed to Step 2** (2026-08-09). **Scope correction
before Step 2 (self-caught):** the 5 WHOLE-FILE-LOAD rows do not need editing — "read
dba-system.md" stays valid regardless of file content; Finding C's cascade requirement is solved
by the later manifest's own text, not consumer-side edits. `templates/project-CLAUDE.md` drops out
of this change entirely. Scope is now exactly the 34 STRUCTURAL-POINTER rows across 16 files.
Step 2 written — 5 ACs (simplified from an initial 8-AC draft by merging two redundant checks
into AC1's own per-file line-level check). R1 DO NOT ADVANCE (High: AC1's grep pattern also
matched the compatibility report's own legend line; its diff-scope check was repo-wide, falsely
excluding this change's own record/tracking files; High: AC2's preface incorrectly required
multi-file citations for Findings B/E, which the report itself calls near-splits that don't
depend on the minority file — only Finding A is a genuine split; Medium: AC4's untouched-file
claim used `git diff` only, missing untracked files) — all fixed. R2 DO NOT ADVANCE (High:
dashboard row still read `1-Intent` and described the pre-correction scope; High: "What changes"
still said "genuine splits — Findings A, B, E," contradicting the already-corrected AC preface a
few lines below in the same artifact) — fixed. R3 DO NOT ADVANCE (High: change record's own trace
header still `null`/`DRAFT` after R1/R2 ran) — fixed. **PROFILE-4's 3-round budget exhausted at
R3; fixed inline, no automatic R4. Human approved 2026-08-09 — proceeding to Step 3.** Step 3: all
34 citations retargeted across the 16 files. AC1-AC4 self-verified PASS (34 rows, exactly 16 files
touched, citation-only edits, `dba-system.md`/`dba/*/v1.md` untouched). R1 DO NOT ADVANCE (High:
ACs marked PASS with the compatibility report only `--sha-only`; High: dashboard Loop-step column
stale) — fixed. R2 **NO OBJECTION**, 1 Low bookkeeping note, fixed anyway. **Human approved 2026-08-09 —
proceeding to Step 4.** Step 4 R1 DO NOT ADVANCE (High: AC1-AC3's Reconciliation evidence was
weaker than each AC's own stated verification method — fixed by re-running the real per-line/
per-citation/per-diff checks fresh at Reconcile, not re-citing Step 3; High: this Status line's own
older paragraph still stated the pre-correction "34+5" scope as current — fixed, marked as
historical). R2 **NO OBJECTION, 0 findings, evidence B**. All 5 ACs re-verified fresh at Reconcile,
all PASS. **Human approved 2026-08-09 — CHG-20260809-003 COMPLETE.** All 34 `STRUCTURAL-POINTER`
citations across 16 consumer files now point at `dba/*/v1.md`; `dba-system.md` remains unchanged.
First half of activation (Invariant 1(d)) done — the atomic `dba-system.md` swap remains a
separate, not-yet-filed change.
**Type**: downstream-doctrine

## Problem

`dba-system.md` (792 lines / 7,139 words) mixes several concerns at different authority levels in
one file — core DBA rules, the Architecture Synthesis Gate, Implementation Profile, review
mechanics, Controlled Plain English activation, artifact layout, tool invocation. Changing one
policy today means touching (and re-approving) the whole doctrine, and nothing distinguishes "this
is what DBA fundamentally guarantees" from "this is how a specific optional mechanism happens to
be wired up right now."

A concrete lean draft already exists on disk, `dba-system-lean.md` (257 lines), but it is
**ungoverned**: no backlog entry, no commit message discussing it, no dashboard row. It was swept
into commit `15daf7c` ("UPG-0064 Step 4 reconciliation") as incidental untracked content, unrelated
in subject to that change's actual scope (delegated Stage-4 envelope alignment). It currently sits
as real content with zero traceability.

A naive fix — "just split the file into smaller linked Markdown fragments" — introduces a new risk
it doesn't solve: **individually-valid components can be jointly contradictory.** For example, a
doctrine version that batches Stage 4–8 execution into one delivery cycle, paired with a review
policy that still assumes a human gate after every stage, are each internally coherent but
mutually inconsistent if combined. A file-split alone has no mechanism to prevent exactly that
combination from silently becoming "the doctrine."

## Upgrade

Replace "one monolithic doctrine file" with a **versioned configuration system**: independently
versioned components, assembled only through an explicitly human-approved, immutable
configuration. Three distinct concepts:

| Concept | Definition |
|---|---|
| **Component version** | An immutable version of one independently-evolving DBA element (e.g. `policies/review/v2.md`). Editing means writing `vN+1`; an approved version is never edited in place. |
| **Configuration version (`DBA-N`)** | An immutable, **explicitly human-approved** combination of specific component versions. |
| **Active configuration** | The single `DBA-N` currently named by `dba-system.md`. |

### Target layout

```text
dba/
├── doctrine/
│   ├── v1.md                       # candidate, decomposed from dba-system.md
│   └── v2.md                       # candidate, decomposed from dba-system-lean.md
├── policies/
│   ├── review/
│   │   ├── v1.md                   # dba-system.md's "Default Advisory Review"
│   │   └── v2.md                   # dba-system-lean.md's "Review Policy"
│   ├── architecture-synthesis/
│   │   ├── v1.md                   # dba-system.md's "Multi-Feature Architecture Synthesis Gate"
│   │   └── v2.md                   # dba-system-lean.md's "Multi-Feature Architecture Gate"
│   ├── implementation-profile/
│   │   └── v1.md                   # dba-system.md's "Implementation Profile" — no v2 candidate
│   │                                #   yet; dba-system-lean.md doesn't address this at all
│   └── controlled-plain-english/
│       └── v1.md                   # activation mechanics; full layered content stays in
│                                    #   patterns/controlled-plain-english.md, referenced
├── tools/
│   └── reviewer/
│       └── v1.md                   # codeos-review.sh invocation mechanics, CPE injection map
└── configurations/
    ├── DBA-1.yaml                  # candidate — NOT pre-approved (see Invariants)
    └── DBA-2.yaml                  # candidate — the lean combination, pending pilot
```

**Both `dba-system.md` and `dba-system-lean.md` are source documents to decompose**, each into
*several* components — never a 1:1 file→version mapping. `dba-system-lean.md` alone contains
doctrine, review-policy, architecture-policy, and writing-style content; decomposing it means
producing candidate `v2`s for several components, not one monolithic `doctrine/v2.md` that absorbs
everything.

**Recommended, not yet decided**: `dba-system.md` stays at its current path (repo root; downstream
`.codeos/dba-system.md`) so no already-onboarded project's symlink breaks, with its role changing
to the thin active-configuration manifest below. See the open question immediately following this
sketch — the alternative (a new `dba/dba.md` path) is not ruled out here.

```markdown
# Active DBA Configuration
active_configuration: dba/configurations/DBA-1.yaml

All components named in the active configuration are jointly authoritative and must be loaded
when applicable. Do not load a component version not named by the active configuration.
```

**Open question, deferred to the change that first creates manifest or component files** (not
resolved by the normative delta inventory, and not bound to any specific CHG's Step 1): whether to
keep `dba-system.md` at its current path as the manifest (recommended, backward-compatible) or
introduce a new `dba/dba.md` path instead.

**Component scope, deliberately not "version everything."** A split is justified only when a
component has independent authority, independent lifecycle, and a real reason to vary separately.
Current concrete set: doctrine, review policy, architecture-synthesis policy, implementation-
profile policy, controlled-plain-english policy, reviewer tool contract. `tools/implementer/` is
**not** included — no downstream doctrine content describes an implementer tool contract today
(the delegated-implementer work is self-dev tooling, `UPG-0060`/`UPG-0064`, unrelated to what
downstream projects load); inventing a placeholder would violate "artifacts on disk are
authoritative."

**`patterns/` vs. a new `policies/` directory — left open, with a criterion, not a default.** Move
material to the smallest existing home whose authority and loading semantics are unambiguous. Do
not invent a new directory taxonomy purely for aesthetics. Directory choice is deferred to the
change that first creates component files — not decided by the normative delta inventory, and not
bound to any specific CHG's Step 1.

### Two levels, kept separate: DBA configuration vs. project configuration

This matters concretely for Controlled Plain English and Implementation Profile, both already
optional, project-toggled mechanisms. `DBA-2` naming `controlled-plain-english-policy: v1` means
only: *this is the version of the rules that govern CPE, if this project uses CPE.* It does **not**
mean CPE is enabled. Whether a project actually uses an optional mechanism remains entirely a
project-local decision:

- **DBA configuration selects the governing policy *version***, applicable if and when a project
  uses that mechanism.
- **Project-local configuration** (e.g. `architecture/controlled-plain-english.yaml`,
  `architecture/implementation-profile.yaml`) **determines whether/how an optional mechanism
  applies to that project**, unchanged by which `DBA-N` is active.

Modular DBA configuration must not swallow project configuration.

### Architectural invariants

1. **No `DBA-N` carries `approved` status without an explicit human approval of that exact pinned
   combination — including `DBA-1`.** The current live system was approved incrementally, over
   many separate change records; that is not the same as approving "`DBA-1` — this exact
   combination — as a configuration object." `DBA-1` must be (a) constructed as the
   configuration-equivalent of today's live system, (b) proven semantically equivalent through the
   delta inventory and compatibility sweep, (c) explicitly approved as the migration baseline, and
   only then (d) activated. Filing this brief does **not** pre-approve `DBA-1`; both `DBA-1.yaml`
   and `DBA-2.yaml` above are illustrative candidates only.
2. **Configuration activation is atomic.** The active pointer names exactly one approved `DBA-N`.
   Components are never activated individually; there is no partially-migrated state.
3. **Published component versions are immutable.** A change is always a new `vN+1`. Old
   configurations remain readable, so a feature's provenance is reconstructible.
4. **A component split must earn its place** — independent authority, independent lifecycle, a
   real reason to vary separately.
5. **Configuration binding is explicit and per-feature.** Every feature must have exactly one
   explicitly recorded governing `DBA-N`. Changing the globally active configuration must never
   silently migrate the governing rules of work already in progress; moving in-flight work to a
   different `DBA-N` requires an explicit human decision and an impact assessment — mirroring how
   `UPG-0051`'s Architecture Baseline versioning already treats in-flight Stage 4 work under a
   superseded baseline version. This gives real provenance (`F-0017 → DBA-1`), not just "whatever
   happened to be globally active later." **Deliberately left open for the change that designs the
   binding mechanism** (not frozen by this brief, and not bound to any specific CHG's Step 1):
   exactly when a feature's binding is recorded — Stage 1 entry, Feature Brief
   acceptance, or some other precise point — and the single canonical location that binding lives.
   Avoid duplicating it across Intent, the feature registry, and the review log; every other
   surface derives or references the one canonical record, never restates it.

**Deliberately not included yet**: no cross-component dependency declarations (e.g.
`review/v3 requires doctrine >= v4`), no compatibility solver. A `DBA-N` configuration is already
the unit whose combined coherence is reviewed and human-approved — that's sufficient. If
combinations become numerous enough that this gets hard to reason about by hand, automate later;
building it now would be premature infrastructure.

### Known normative deltas identified so far — explicitly non-exhaustive

An initial line-by-line comparison already found more disagreement than a first pass surfaced.
This table is evidence the comparison must be done systematically, not a claim that it's complete:

| Area | `dba-system.md` (candidate v1) | `dba-system-lean.md` (candidate v2) |
|---|---|---|
| Authority | Runtime behavior can override intent *text* when more specific | Approved artifacts are authoritative for required behavior; runtime shows only what the system *does* |
| Stage 4 internal abstractions | "NEVER add abstractions... beyond" what artifacts specify | Normal internal abstractions permitted, provided observable behavior is unchanged |
| Stage 6 execution | Human runs the implementation | Agent may run representative scenarios when the environment permits |
| Stage 9 refinement | Human approves each refinement individually | A correction within already-approved behavior may proceed without a new product decision |
| Architecture governance | Same two artifacts (Core Architecture Baseline + Cohort Logical Design), produced through elaborate registry/version/history/wave-gate mechanics | Same two artifacts, produced through a condensed 4-step draft→approve sequence with no registry/history/versioning machinery |
| Review persistence | Extensive mandatory logging (Decision Log row, conditional Rationale, conditional AJ entry, always) | Save a review only when its decision changes behavior, architecture, or an accepted risk |
| Stage transitions (Non-Negotiable Rule #1) | Every stage transition requires explicit human approval | Event Schema approval authorizes Stages 4–8 as one uninterrupted delivery cycle |
| Independent review | Default at every reviewable gate | Conditional — only on named trigger conditions |

**The mandatory Step-1 deliverable, before drafting any component file**: a complete disposition
of *every* current normative rule in `dba-system.md`, each classified exactly one of
`KEEP-IN-CORE` (meaning preserved, stays in doctrine; `target_owner: doctrine`) / `MOVE` (meaning
preserved, relocates to a named non-doctrine component, or `target_owner: UNRESOLVED` when none
clearly fits — ownership must be resolved before component drafting begins) / `RETIRE` (redundant,
duplicated, or already-superseded rules only — any semantic removal is `INTENTIONAL-BEHAVIOR-
CHANGE` instead) / `INTENTIONAL-BEHAVIOR-CHANGE` (meaning changes, **regardless of whether the
rule's component also changes** — it still names a `target_owner` for the new form, and requires
an explicit human decision, never bundled into "adopt `DBA-2`"; intentional deletion with no
successor is `target_owner: NONE` / `proposed_rule: REMOVED`, distinct from `RETIRE`, which only
covers zero-semantic-loss removal). Meaning and location are two independent axes: a rule can
change meaning *and* move components at once, and that combination is always
`INTENTIONAL-BEHAVIOR-CHANGE`, never `MOVE`. The table above seeds that inventory; it does not
substitute for it. The inventory is allowed to challenge this brief's own proposed component
decomposition —
it is not forced to validate it.

### Migration approach — two decoupled phases, deliberately not confounded

**Phase A — did modularization preserve current DBA?**
Normative delta inventory (full disposition table) → decompose `dba-system.md` into candidate v1
components → compatibility sweep against prompts/scripts/templates assuming old semantics → prove
`DBA-1` semantically equivalent to the live monolith → explicit human approval of `DBA-1` →
activate the modular architecture with `DBA-1` as the active configuration.

**Phase B — is lean DBA actually better?** (only once Phase A is complete)
Decompose `dba-system-lean.md` into candidate v2 components → one real downstream pilot comparing
`DBA-2` against the now-active `DBA-1` → atomic activation of `DBA-2` only on explicit approval.

If `DBA-1` cannot reproduce the current system exactly, fix the modular architecture before judging
lean DBA at all — the two questions must not be confounded into one measurement.

This is the recommended shape for the eventual loop. Nothing in this filing runs either phase.

## Scope

**In scope for this filing**: this brief and its illustrative candidate mapping;
`backlog/features.md` and `status/roadmap.md` index rows.

**Out of scope for this filing**: creating the `dba/` tree; writing any `configurations/*.yaml`
for real; moving any content out of `dba-system.md`; approving `DBA-1` or `DBA-2`; starting the
4-step self-development loop; deciding the manifest-path or `patterns/`-vs-`policies/` open
questions.

## Value

Gives the orphaned `dba-system-lean.md` draft a governance home instead of leaving it as untracked
dead weight in git history. More importantly, reframes the actual opportunity correctly: not
"shrink one file" but "let DBA policies evolve independently, under an explicit, reviewable,
approved combination" — which directly serves Codeos's own stated goal of reducing review load and
keeping evidence integrity, applied to the doctrine that governs every downstream project.

## Risk

Filing this brief is low risk — no code, no doctrine edit, no consumer wired up. The risk lives
entirely in the eventual implementation, which is exactly why this brief exists: to make the
combination-validity risk, the two-level (DBA-config vs. project-config) risk, and the in-flight-
feature-binding risk explicit architectural invariants *before* any component file is drafted,
rather than discovering them mid-migration.

## Guardrail

- This filing must not edit `dba-system.md`, `dba-system-lean.md`, or create any file under a new
  `dba/` directory.
- Whenever the loop does start, it runs at `downstream-doctrine` scope, PROFILE-4 review cadence,
  per `CLAUDE.md`.
- Invariant 1 (no pre-approval, including `DBA-1`), Invariant 2 (atomic activation only), and
  Invariant 5 (explicit per-feature configuration binding) are hard guardrails an implementing
  agent must not relax or treat as aspirational.
- Do not build a cross-component dependency solver as part of this UPG (see "Deliberately not
  included yet" above).

## Related

- **UPG-0001** — Feature Thread traceability already established the immutable-version,
  explicit-pin discipline this proposal extends from review rounds to doctrine itself.
- **UPG-0051** — Architecture Baseline versioning is the direct precedent for Invariant 5's
  in-flight-work handling (a membership/version change doesn't retroactively invalidate
  already-approved Stage 4 work under an earlier version).
- **UPG-0052**, **UPG-0058**, **UPG-0059** — Implementation Profile, Cohort Logical Design, and
  Wave-Gated Batch Review are exactly the sections that would become
  `policies/implementation-profile` and `policies/architecture-synthesis` components.
- **UPG-0056**, **UPG-0057** — the Optional Mechanism Status Convention and Controlled Plain
  English already established the pattern of splitting optional content into `patterns/`; this
  proposal generalizes and formalizes that pattern into versioned, explicitly-approved components.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260807-001 | changes/UPG-0065__CHG-20260807-001__normative-delta-inventory.md | Complete per-rule disposition of dba-system.md before any component drafting (Phase A, first sub-step) | COMPLETE |
| CHG-20260808-001 | changes/UPG-0065__CHG-20260808-001__v1-component-decomposition.md | Decompose dba-system.md into 6 candidate v1 component files using the accepted delta table (Phase A, second sub-step) | COMPLETE |
| CHG-20260808-002 | changes/UPG-0065__CHG-20260808-002__downstream-consumer-compatibility-sweep.md | Catalog every prompts/scripts/templates/patterns reference to dba-system.md; classify compatibility with the future thin-manifest activation (Phase A, third sub-step) | COMPLETE |
| CHG-20260809-001 | changes/UPG-0065__CHG-20260809-001__dba1-equivalence-proof.md | Construct candidate DBA-1.yaml; assemble delta-inventory + v1-decomposition evidence into a formal semantic-equivalence proof against dba-system.md — not approval, not activation (Phase A, fourth sub-step) | COMPLETE |
| CHG-20260809-002 | changes/UPG-0065__CHG-20260809-002__dba1-approval.md | Explicit human approval of DBA-1 as the migration baseline (Invariant 1(c)) — not activation, which stays a separate later change (Phase A, fifth sub-step) | COMPLETE |
| CHG-20260809-003 | changes/UPG-0065__CHG-20260809-003__consumer-citation-migration.md | Migrate all 34 STRUCTURAL-POINTER citations across 16 consumer files to the correct dba/*/v1.md locations, while dba-system.md stays unchanged — first half of activation (Phase A, sixth sub-step, part 1); the 5 WHOLE-FILE-LOAD rows need no edit (see change record's Scope boundary) | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| RVS__UPG-0065__CHG-20260807-001__S1 | CHG-20260807-001 | 1-Intent | Codex R1→R3, then 2 rounds of human gate review | Codex R1-R3 DO NOT ADVANCE (MOVE-row location vs. deferred packaging questions; "Step-1 decision" wording falsely bound to this CHG; manifest-path self-contradiction) — all fixed, PROFILE-4 budget exhausted at R3. 1st human review: MOVE definition presupposed the six-component decomposition before the inventory tested it (contra Invariant 4); inventory universe was implicitly anchored to the lean-draft comparison; `RETIRE` undefined against semantic change — all fixed. 2nd human review: `MOVE`'s early and late definitions still contradicted each other; "normative rule" granularity was undefined; `RETIRE` referenced "non-normative" material outside the inventory's own universe — all fixed. **Human verdict: APPROVED — proceed to Step 2** |
| RVS__UPG-0065__CHG-20260807-001__S2 | CHG-20260807-001 | 2-Acceptance | R1→R6 | Codex R1-R3 DO NOT ADVANCE (taxonomy overlap; weak verification methods; count-vs-1:1 mapping; unpinned commit reference) — all fixed, PROFILE-4's 3-round budget exhausted at R3. 1st human gate review: required a mandatory delta-row schema for checkable traceability, bounded completeness to section granularity, replaced the "and"/"or" test with a semantic independence test, hardened AC-8 — all fixed; **human-authorized confirmatory R4: NO OBJECTION, 0 findings, evidence A.** 2nd human gate review: found the four dispositions conflated the meaning-change and component-location axes, making a rule that both changes meaning and relocates unrepresentable — fixed by adding a `target_owner` field to every non-`RETIRE` row and an explicit precedence (`RETIRE` → `INTENTIONAL-BEHAVIOR-CHANGE` → `KEEP-IN-CORE` → `MOVE`, meaning-change always wins); also tightened AC8 to this change's own diff and `source_anchor` to a pinned line range plus excerpt. Codex R5 DO NOT ADVANCE (the new schema still couldn't represent intentional deletion — no successor form, no target location, and `RETIRE` only covers zero-semantic-loss removal; AC5 checked `target_owner` was non-empty but not that its value was valid) — fixed: added `proposed_rule: REMOVED` / `target_owner: NONE` as a paired special case, and AC5 now enforces the full domain with Invariant-4 rigor matching AC3. Codex R6 DO NOT ADVANCE (AC8's `git diff` against a commit cannot see untracked files, so an untracked forbidden artifact could exist and still pass; one Low SELF-REFERENCE/REVIEW-BOOKKEEPING note on stale round-count comments, optional) — fixed: AC8 now also checks `git status --porcelain --untracked-files=all`, with human attribution required for any forbidden-path hit; bookkeeping updated to R6 in this same pass. |
| RVS__UPG-0065__CHG-20260807-001__S3 | CHG-20260807-001 | 3-Implement | R1→R3 | Produced the 184-row delta table. R1 DO NOT ADVANCE (missing `source_section` field; several anchors lacked quoted excerpts; `STEP6-EVENTS` retired as a duplicate of `NEVER-DO-8` when it actually stated an independent emission-destination fact `NEVER-DO-8` never covers; `ARTIFACT-CLASS-2` silently dropped the Feature Brief classification the same way; `ARTIFACT-CLASS-3`/`CPE-2`/`FILE-LAYOUT-1` bundled independently-changeable facts; `INTENTIONAL-BEHAVIOR-CHANGE` rows lacked the literal per-row `requires_human_decision: yes` marker) — fixed, table grew 115→131. R2 DO NOT ADVANCE (two of the R1 fixes still bundled facts under one *shared* disposition — a shared disposition doesn't license bundling; `STAGE-TABLE-4` presented two candidate resolutions instead of one `proposed_rule`; a stale cross-reference in `NEVER-DO-8` still said `STEP6-EVENTS` was retired after the R1 fix reversed that) — fixed, table grew 131→147. R3 DO NOT ADVANCE (`FILE-LAYOUT-2g`/`6`/`9` and — the largest single defect — `STAGE-TABLE-1`, which had bundled all 31 Stage-to-file and Artifact-to-template mappings into one row since the very first draft, plus `REVIEW-LOG-1` bundling four independently-approvable logging facts under one disposition, plus a File Layout section-coverage count that directly contradicted its own row listing) — fixed inline, table grew 147→184. **PROFILE-4's 3-round budget exhausted at R3; no automatic R4.** All three rounds found the same underlying gap — inconsistent application of the independent-changeability test during drafting — named explicitly in Implementation Notes rather than treated as three unrelated issues. **Human verdict: APPROVED — proceed to Step 4.** |
| RVS__UPG-0065__CHG-20260807-001__S4 | CHG-20260807-001 | 4-Reconcile | R1→R3 | Wrote the Reconciliation section: 9 ACs verified, consistency sweep, findings scope-triage. Self-caught before review: brief/dashboard staleness (Step 3 approval hadn't propagated) — fixed. R1 DO NOT ADVANCE (`ARCH-GATE-3`/`OPT-MECH-2`/`IMPL-PROFILE-9` each bundled facts about different subjects; stale File Layout coverage count) — fixed, plus a self-initiated scan found `IMPL-PROFILE-1`/`4` before Codex named them; table grew 184→193. R2 DO NOT ADVANCE (`IMPL-PROFILE-3`/`FILE-LAYOUT-5`/`HUMAN-NAV-2` bundled facts; `OPT-MECH-2c` not a normative rule at all, removed; another stale count) — fixed, 193→198. Per explicit human instruction, further self-initiated audits paused from here — fixing only what named review findings require. R3 DO NOT ADVANCE (`REVIEW-5`/`ARCH-GATE-7`/`IMPL-PROFILE-6` bundled facts; and — more serious — 6 `RETIRE` rows, `ARTIFACT-CLASS-3`/`4`/`5` and `FILE-LAYOUT-2a`/`2c`/`2d`, claimed duplication of content their named target rows never actually stated, the same real content-loss class as `STEP6-EVENTS`; Reconciliation's AC1 claim overstated its own verification method) — fixed, 198→203 (95/61/28/19). **PROFILE-4's 3-round budget exhausted at R3; fixed inline, no automatic R4. Human verdict: APPROVED — CHG-20260807-001 COMPLETE (2026-08-08).** |
| RVS__UPG-0065__CHG-20260808-001__S1 | CHG-20260808-001 | 1-Intent | R1→R3 | R1 DO NOT ADVANCE (High: `current_rule` is sometimes a condensed paraphrase — e.g. `REVIEWER-TOOL-1`/`ARCH-GATE-6` — copying it verbatim would drop real normative detail while claiming semantic equivalence; Medium: manifest-path decision deferred past the point the brief itself sets, "the change that first creates component files") — fixed: content source changed to the full pinned `dba-system.md` text at each row's `source_anchor`, `current_rule` used only as an index; manifest path decided now (keeps current `dba-system.md` path), not executed. R2 DO NOT ADVANCE (Medium: the manifest-path fix left a stale Scope-boundary bullet still saying the question "stays open — not decided here," self-contradicting the new decision) — fixed: stale bullet corrected to state the path is decided but not executed. R3 **NO OBJECTION, 0 findings, evidence B**. **Human verdict: APPROVED — proceed to Step 2** (2026-08-08). |
| RVS__UPG-0065__CHG-20260808-001__S2 | CHG-20260808-001 | 2-Acceptance | R1→R3 | R1 DO NOT ADVANCE (High: AC3/AC4's source-fidelity and no-lean-content checks were label/verbatim-only and could accept lost or paraphrased-lean semantics; Medium: AC8's "no file outside `dba/` references the new paths" was already false against this change record's own "What changes" list; Medium: AC2's `RETIRE` exclusion checked only `rule_id` labels, not source-text provenance) — fixed: AC3 made a hard mandatory-fix requirement (not flag-with-rationale); AC4 added a full-read meaning check alongside the verbatim grep; AC2 added a provenance-overlap check against `RETIRE` ranges; AC8 rescoped to operational/consumer surfaces only, explicitly excluding governance bookkeeping. R2 DO NOT ADVANCE (Medium: cross-reference inconsistency — the brief's own S1 Reviews-table row still said "Awaiting human gate decision" after the human had already approved Step 1; the change record's "What changes" list still named a stale `1-Intent` loop step) — fixed. R3 **NO OBJECTION, 0 findings, evidence B**. **Human verdict: APPROVED — proceed to Step 3** (2026-08-08). |
| RVS__UPG-0065__CHG-20260808-001__S3 | CHG-20260808-001 | 3-Implement | R1→R3 | Produced 6 candidate `v1` component files (175 rows) transcribed from `dba-system.md` @ `77599e9`'s pinned text. Self-check (before review) found the accepted delta table's own `source_anchor` ranges undercounted content in 7 places, most notably `ARCH-GATE-6` (missing 3 of 6 procedural steps) — corrected inline; not a re-opening of the ACCEPTED `CHG-20260807-001`. R1 DO NOT ADVANCE (High: `CPE-2c`/`CPE-2d` share source line 492 at a genuine sub-line split, making the Implementation Notes' "every RETIRE range is physically distinct" claim false for 1 of 28 rows; Medium: CPE thin-pointer AC6/prose said `patterns/controlled-plain-english.md` while the transcribed content correctly used `.codeos/patterns/controlled-plain-english.md`; Medium: `dba/doctrine/v1.md` placed `STAGE-TABLE-4` after `STAGE-TABLE-1r-ae`, violating the binding ascending-source-anchor schema) — all fixed. R2 DO NOT ADVANCE (High: fixed the R1 fact but not AC2's own criterion text, which still read as an absolute no-overlap rule contradicting the now-documented `CPE-2c`/`CPE-2d` exception) — fixed: AC2 reworded to state the real invariant (no `RETIRE`d text transcribed; a shared line number at a genuine sub-line split is not itself a violation). R3 **NO OBJECTION, 0 findings, evidence B**. **Human verdict: APPROVED — proceed to Step 4** (2026-08-08). |
| RVS__UPG-0065__CHG-20260808-001__S4 | CHG-20260808-001 | 4-Reconcile | R1→R2 | Wrote the Reconciliation section: all 10 ACs verified PASS with evidence, consistency sweep, findings scope-triage across all 4 steps. Self-caught before review: `status/self-development.md`'s Loop-step column and the brief's own Status line/S3 Reviews row still read as if Step 3 were unapproved — fixed before writing the table. R1 DO NOT ADVANCE (Medium, Codex: AC10 marked PASS without packet evidence for 2 of its 5 named surfaces — `backlog/features.md`/`status/roadmap.md` were never included in the review packet) — fixed: both included in full for R2, plus AC10's evidence reworded to cite exact line numbers and clarify neither file tracks step-level state. R2 **NO OBJECTION, 0 findings, evidence B**. Awaiting final human gate decision. |
| RVS__UPG-0065__CHG-20260808-002__S1 | CHG-20260808-002 | 1-Intent | R1→R2 | R1 DO NOT ADVANCE (High: the "21 files" scope count didn't reconcile against its own stated breakdown — 17 downstream + 2 self-dev-governance = 19, dropping `scripts/tests/codeos-implement-tests.sh` entirely; Medium: `status/roadmap.md` is in the packet diff but not listed in "What changes") — fixed: full 21-file breakdown corrected to 18 downstream + 2 self-dev-governance + 1 incidental test-fixture-string match (`scripts/tests/codeos-implement-tests.sh`, a path-traversal test literal, not a real reference); `status/roadmap.md` added to "What changes". R2 **NO OBJECTION**, 1 Low IN-SCOPE NON-BLOCKER (grep count not independently re-shown in packet — optional, not fixed), evidence A. **Human verdict: APPROVED — proceed to Step 2** (2026-08-08). |
| RVS__UPG-0065__CHG-20260808-002__S2 | CHG-20260808-002 | 2-Acceptance | R1→R3 | R1 DO NOT ADVANCE (High: AC1's verification diffed the raw report `file` column against the unique grep file list, which would falsely fail a correct report once any file legitimately has 2+ rows) — fixed by deduplicating both sides before diffing, which over-corrected. R2 DO NOT ADVANCE (High: the dedup fix let a file with 3 real occurrences pass with only 1 reported row, since row count per file was explicitly excluded from the criterion; High: AC5's guardrail checks used `git status`/`git diff`, which miss an already-committed `configurations/*.yaml` and miss a new untracked `dba/` file respectively) — fixed: AC1 rewritten to check per `(file, line)` pair via `grep -n`, not file-level presence; AC5 rewritten to a direct filesystem existence check for `configurations/` plus both `git diff` and `git status --untracked-files=all` for `dba/`. R3 DO NOT ADVANCE (High: the brief, dashboard, and this change record's own trace header disagreed on whether Step 2 review had run at all — self-inflicted staleness from not updating tracking surfaces after R1/R2) — fixed. **PROFILE-4's 3-round budget exhausted at R3; fixed inline, no automatic R4.** **Human verdict: APPROVED — proceed to Step 3** (2026-08-08). |
| RVS__UPG-0065__CHG-20260808-002__S3 | CHG-20260808-002 | 3-Implement | R1→R3 | Produced a 49-row compatibility report cataloging every `dba-system.md` reference across `prompts/`/`scripts/`/`templates/`/`patterns/`. R1 DO NOT ADVANCE (High: the `STRUCTURAL-POINTER` schema/AC2 only covered `##`-heading citations, but 9 of 34 such rows cite sub-part bolded lead-ins, not headings; High: AC3 was spot-checked on only 5 of 34 rows, not all) — fixed: schema/AC2 split into section-level vs. sub-part granularities, both independently verified; AC3 re-run for all 34 rows, recorded in full. R2 DO NOT ADVANCE (High: brief/dashboard/trace-header still said "review not yet run" after R1 — same staleness pattern as S2's R3; High: report rows 1/33/49 claimed the Controlled Plain English section "maps cleanly" while Implementation Notes correctly called it a near-split; Medium: AC2's verbatim rule was applied inconsistently — "matches closely" for one row, "exact match... but missing X" for another) — fixed: tracking surfaces synced; CPE rows corrected and Finding E added; one explicit core-clause verbatim rule applied uniformly across all 9 sub-part rows. R3 DO NOT ADVANCE (High: Finding B's row list — 12 stated — didn't reconcile with its own two sub-lists, which summed to 15 with row 35 mislabeled; root cause found: rows 8/15/24 do cite the full section name, but the report's own "cited text" column had truncated them with "…", producing miscounts by both Codex and by hand) — fixed: truncation removed, Finding B rewritten with a mechanically grep-verified 12-row list; the identical error was found and fixed proactively in Finding E before being flagged. **PROFILE-4's 3-round budget exhausted at R3; fixed inline, no automatic R4.** **Human verdict: APPROVED — proceed to Step 4** (2026-08-09). |
| RVS__UPG-0065__CHG-20260808-002__S4 | CHG-20260808-002 | 4-Reconcile | R1→R2 | Wrote the Reconciliation section: all 6 ACs independently re-verified at Reconcile (not re-trusting Step 3's own claims), consistency sweep, findings scope-triage across all 4 steps. Self-caught before review, ×2: dashboard Loop-step and brief Status line both still described Step 3 as current after human approval — fixed before writing the table. R1 DO NOT ADVANCE (High, Codex: AC6 marked PASS but `backlog/features.md`/`status/roadmap.md` were only `--sha-only` in the packet, not shown as content — the same finding CHG-20260808-001's own Step 4 R1 caught) — fixed: both included in full for R2. R2 **NO OBJECTION, 0 findings, evidence grade C** (lower than this feature's usual A/B — `compatibility-report.md` itself was `--sha-only` this round to fit budget, reducing direct visibility into AC1-AC5's primary evidence; the AC6 fix itself is fully evidenced). Awaiting final human gate decision. |
| RVS__UPG-0065__CHG-20260809-001__S1 | CHG-20260809-001 | 1-Intent | R1 | R1 **NO OBJECTION**, 0 findings, evidence B. **Human verdict: APPROVED — proceed to Step 2** (2026-08-09). |
| RVS__UPG-0065__CHG-20260809-001__S2 | CHG-20260809-001 | 2-Acceptance | R1→R3 | R1 DO NOT ADVANCE (High: AC2's `RETIRE`-exclusion check only covered labels + one previously-known sub-line case, not a full 28-row provenance sweep; High: Change Intent/AC7 claimed the compatibility sweep had "three findings" but it actually has five — Findings B and E were undercounted; Medium: AC6's no-activation check omitted the `dba/*/v1.md` read-only boundary) — fixed: AC2 now requires a full re-sweep matching `CHG-20260808-001`'s own method; finding count corrected everywhere with AC7 requiring a fresh grep-recount, not a remembered number; AC6 extended to cover `dba/`. R2 DO NOT ADVANCE (High: AC6's fix still used only `git diff`, which cannot see untracked new files, across the broader path set) — fixed: added `git status --untracked-files=all` across the same paths; also fixed a non-blocking self-reference-boundary note that had embedded a live round number in the artifact. R3 **NO OBJECTION, 0 findings, evidence A**. **Human verdict: APPROVED — proceed to Step 3** (2026-08-09). |
| RVS__UPG-0065__CHG-20260809-001__S3 | CHG-20260809-001 | 3-Implement | R1→R3 | Built `dba/configurations/DBA-1.yaml` (candidate) and the equivalence report. AC1/AC2/AC4/AC5/AC7 PASS clean; AC3's full 175-row re-verification against pinned `dba-system.md`@`77599e9` found 163 clean and 12 citation-precision/wording defects in `dba/*/v1.md` — presented to the human given this change's own read-only scope boundary on those files; **human explicitly waived all 12 as non-blocking for `DBA-1`**, tracked as a follow-up hygiene item, not fixed here. R1 DO NOT ADVANCE (High: AC3's own written text, "fixed before this AC can PASS, not waived," directly contradicted the artifact's recorded PASS-by-waiver outcome — a false-acceptance-claim risk) — fixed: AC3 reworded to state the actual invariant applied (fix, or an explicit human decision when fixing would cross the read-only boundary, recorded verbatim per row) — the same class of fix `CHG-20260808-001`'s own AC2 received at its Step 3 R2; non-blocking AC6 evidence-placement note also fixed. R2 DO NOT ADVANCE (High: trace header still read `review_series: null` after R1 had already run, contradicting the dashboard's own tracked state; Medium: Implementation Notes and the equivalence report still quoted AC3's *original* "not waived" wording as current after the reword, self-contradicting it) — fixed: trace header set to `S3`/`IN_REVIEW`; both narrative passages reworded to past tense, pointing at the current AC3 text. R3 **NO OBJECTION, 0 findings, evidence A**. **Human verdict: APPROVED — proceed to Step 4** (2026-08-09). |
| RVS__UPG-0065__CHG-20260809-001__S4 | CHG-20260809-001 | 4-Reconcile | R1 | Wrote the Reconciliation section: all 8 ACs independently re-verified at Reconcile (not re-trusting Step 3's own claims), consistency sweep, findings scope-triage. Self-caught before review: dashboard row and the brief's own S3 Reviews-table row still read "3-Implement"/"Awaiting human gate decision" after the human had already approved Step 3 — the same staleness class this feature catches at every Step 4 (AJ-020/AJ-025) — fixed before writing the table. R1 **NO OBJECTION, 0 findings, evidence B**. **Human verdict: APPROVED — CHG-20260809-001 COMPLETE** (2026-08-09). |
| RVS__UPG-0065__CHG-20260809-002__S1 | CHG-20260809-002 | 1-Intent | R1→R2 | R1 DO NOT ADVANCE (High: trace header said `state: DRAFT` while both tracking surfaces already showed this change active/`IN_PROGRESS`, a governance-state contradiction; Medium: the core claim this approval-only change depends on — `DBA-1.yaml` is still `status: candidate`, not already approved — was only `--sha-only` in the packet, not directly evidenced) — fixed: trace header set to `IN_REVIEW`; `DBA-1.yaml` shown in full for R2. R2 **NO OBJECTION, 0 findings**. **Human verdict: APPROVED — proceed to Step 2** (2026-08-09). |
| RVS__UPG-0065__CHG-20260809-002__S2 | CHG-20260809-002 | 2-Acceptance | R1→R3 | R1 DO NOT ADVANCE (High: brief's own status line said approval/activation were "not yet started" one sentence after describing this change's own Step 1 as ACCEPTED — fixed: reworded to reflect approval as in-progress, only activation not yet started; High: AC2's verification method depended on "working-tree state in this change's own history," not durable pinned evidence — fixed: reworded to check final-state co-presence of `DBA-1.yaml`'s `approved_via` field and the Step 4 log entry; Medium: brief's AC1 self-check claim lacked the actual command shown — fixed) — fixed. R2 DO NOT ADVANCE (High: the R1 fix still asserted an unprovable "never during Steps 1-3" timing claim while its own verification method only checks final-state co-presence — fixed: narrowed AC2's claim to what the evidence can actually show, moved the editing-discipline expectation to a non-criterion process note) — fixed. R3 DO NOT ADVANCE (Medium: that process note claimed the discipline "is followed as this change's own Implementation Notes describe" in present tense, but Implementation Notes is still an empty Step-3 placeholder — a false present-tense claim inside the AC contract) — fixed: reworded to future tense. **PROFILE-4's 3-round budget exhausted at R3; fixed inline, no automatic R4.** **Human verdict: APPROVED — proceed to Step 3** (2026-08-09). |
| RVS__UPG-0065__CHG-20260809-002__S3 | CHG-20260809-002 | 3-Implement | R1→R2 | Re-verified AC1 fresh (no drift since `CHG-20260809-001`'s completing commit) and deliberately made no edit to `DBA-1.yaml` — the approval flip is reserved for Reconcile, after the Step 4 human decision is logged. R1 DO NOT ADVANCE (High: `status/self-development.md`'s own Loop-step column still read `2-Acceptance` while both the row's own narrative and the change record already described Step 3 as underway) — fixed. R2 **NO OBJECTION, 0 findings**. **Human verdict: APPROVED — proceed to Step 4** (2026-08-09). |
| RVS__UPG-0065__CHG-20260809-002__S4 | CHG-20260809-002 | 4-Reconcile | R1→R3 | Human explicitly approved `DBA-1` as the migration baseline per Invariant 1(c); `dba/configurations/DBA-1.yaml` edited to `status: approved` with `approved_at`/`approved_via`. R1 DO NOT ADVANCE (High: dashboard's Loop-step column still read `3-Implement`; High: AC2 marked PASS without `reviews/review-log.md` shown in the packet; Medium: AC6 marked PASS without `backlog/features.md`/`status/roadmap.md` shown) — fixed inline. R2 DO NOT ADVANCE (High: change record's own trace header still read `review_series: null`/`review_state: DRAFT` after R1 had already run; High: `reviews/review-log.md` was mistakenly left `--sha-only` despite the R1 fix claiming it was included full; Medium: Change Intent's "Why" section stated the pre-approval problem in present tense after approval already happened) — fixed; discovered `reviews/review-log.md` carries a pre-existing precheck false-positive (`UPG-####` literal in its own header legend, already tracked to `UPG-0031`, unrelated to this change) that blocks passing the whole file to the reviewer — resolved by quoting the exact needed log entry verbatim inside AC2's own evidence cell. R3 DO NOT ADVANCE (High: AC6 still marked PASS without packet-visible evidence for `backlog/features.md`/`status/roadmap.md`) — fixed the same way, both files' relevant lines quoted verbatim in AC6's evidence cell. **PROFILE-4's 3-round budget exhausted at R3; fixed inline, no automatic R4.** Awaiting human gate decision. |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|
| (S1 R1) Required `MOVE`-row locations conflicted with the brief's own deferred directory/manifest questions | RVS__…__S1 | IN-SCOPE BLOCKER | Fixed — "authoritative location" defined as logical component identity, resolvable without the two open questions |
| (S1 R2) Brief's "Step-1 decision" wording bound the manifest/directory questions to this first CHG's own Step 1 | RVS__…__S1 | IN-SCOPE BLOCKER | Fixed — reworded to "deferred to the change that first creates manifest/component files," not tied to any specific CHG's Step 1 |
| (S1 R3) Brief asserted `dba-system.md`'s path as decided ("stays at its current path") while also calling it an open question | RVS__…__S1 | IN-SCOPE BLOCKER | Fixed — reworded as "Recommended, not yet decided" |
| (S1, human gate review) "Component identity already fixed by the brief's target layout" presupposed the six-component decomposition before the inventory tested it, contradicting Invariant 4 | — (human direction, not a Codex review finding) | IN-SCOPE BLOCKER | Fixed — `MOVE` now names a *candidate* owner; the inventory may find components should merge, not exist, split, or that a rule stays in the kernel |
| (S1, human gate review) Inventory completeness was implicitly anchored to the `dba-system.md`-vs-`dba-system-lean.md` seed comparison rather than every current normative rule | — (human direction, not a Codex review finding) | IN-SCOPE BLOCKER | Fixed — added an explicit "Inventory universe" rule: completeness is measured against current `dba-system.md`, the lean draft is comparison evidence only |
| (S1, human gate review) `RETIRE` had no definition separating it from a semantic behavior removal | — (human direction, not a Codex review finding) | IN-SCOPE NON-BLOCKER | Fixed — `RETIRE` narrowed to redundant/superseded/non-normative material only; any semantic removal is `INTENTIONAL-BEHAVIOR-CHANGE` |
| (S1, 2nd human gate review) The change record's early `MOVE` definition ("with its new authoritative location named") still contradicted the later `owner: UNRESOLVED` allowance | — (human direction, not a Codex review finding) | IN-SCOPE BLOCKER | Fixed — `MOVE` now uniformly defined as "candidate owner, or `owner: UNRESOLVED`" everywhere it appears, including this brief's own matching definition |
| (S1, 2nd human gate review) No definition existed for what counts as one "normative rule" — paragraph? sentence? each `must`? | — (human direction, not a Codex review finding) | IN-SCOPE BLOCKER | Fixed — added "Rule granularity": one row per independently meaningful obligation/prohibition/permission/authority rule/gate/lifecycle requirement, not per sentence or paragraph; to be pinned as a Step 2 acceptance criterion |
| (S1, 2nd human gate review) `RETIRE`'s definition named "non-normative" material, but the inventory universe is explicitly normative rules only | — (human direction, not a Codex review finding) | IN-SCOPE NON-BLOCKER | Fixed — `RETIRE` narrowed further to redundant/duplicated/superseded normative rules only; non-normative prose needs no disposition at all |
| (S2, human gate review) AC-1 required one-to-one traceability "by source location and meaning" with no mandatory row schema, leaving verification to informal recognition rather than checkable fields | — (human direction, not a Codex review finding) | IN-SCOPE BLOCKER | Fixed — added a mandatory delta-table row schema (`rule_id`/`source_section`/`source_anchor`/`current_rule`/`disposition`/`owner`/`rationale`, plus `proposed_rule`/`requires_human_decision` for behavior changes) |
| (S2, human gate review) AC-1's "non-normative prose explicitly noted as out of scope" could require accounting for every explanatory sentence in ~7,000 words | — (human direction, not a Codex review finding) | IN-SCOPE BLOCKER | Fixed — completeness now proven at section granularity (`NO NORMATIVE RULES` marker for rule-free sections), not per explanatory sentence |
| (S2, human gate review) AC-2's granularity check used an "and"/"or" keyword heuristic, which misclassifies atomic multi-clause gates (e.g. one gate requiring three approvals) as multiple rules | — (human direction, not a Codex review finding) | IN-SCOPE BLOCKER | Fixed — replaced with a semantic independence test: a row splits only if some part could change independently while the rest keeps the same meaning and disposition |
| (S2, human gate review) AC-8 required global `git status` to show only the named files, vulnerable to false failure from unrelated concurrent working-tree changes | — (human direction, not a Codex review finding) | IN-SCOPE NON-BLOCKER | Fixed — replaced with `test ! -d dba` plus a forbidden-path check against this change's own declared file list |
| (S2, human gate review) Trace header's `review_series` still pointed to `S1` after Step 2 acquired its own review series and rounds | — (human direction, not a Codex review finding) | IN-SCOPE NON-BLOCKER | Fixed — updated to `S2` |
| (S2, 2nd human gate review) The four dispositions conflated two independent axes (meaning-change vs. component-location), making a rule that both changes meaning and moves components unrepresentable — it couldn't be correctly coded as either `MOVE` (which requires meaning preserved) or `INTENTIONAL-BEHAVIOR-CHANGE` (which had no target-location field) | — (human direction, not a Codex review finding) | IN-SCOPE BLOCKER | Fixed — added `target_owner` field to the row schema (required for `KEEP-IN-CORE`/`MOVE`/`INTENTIONAL-BEHAVIOR-CHANGE`, blank for `RETIRE`); disposition now chosen by an explicit precedence (`RETIRE` → `INTENTIONAL-BEHAVIOR-CHANGE` → `KEEP-IN-CORE` → `MOVE`) where any meaning change takes precedence over `KEEP-IN-CORE`/`MOVE` regardless of whether location also changes |
| (S2, 2nd human gate review) AC-8's `test ! -d dba` asserted global non-existence — a legitimate unrelated concurrent change creating `dba/` would fail this CHG for something it didn't do | — (human direction, not a Codex review finding) | IN-SCOPE NON-BLOCKER | Fixed — replaced with `git diff 77599e9 --name-only` scoped to this change's own diff against its pinned baseline, not global repository state |
| (S2, 2nd human gate review) `source_anchor` allowed "excerpt or line range" as alternatives; a pinned line range plus excerpt together gives deterministic relocation | — (human direction, not a Codex review finding) | IN-SCOPE NON-BLOCKER | Fixed — `source_anchor` now requires both a line range pinned to commit `77599e9` and a short quoted excerpt |
| (S2 R5, Codex) `INTENTIONAL-BEHAVIOR-CHANGE` had no way to represent intentional deletion — a `proposed_rule`+`target_owner` were both mandatory, but a deleted obligation has neither a new form nor a new location, and `RETIRE` is reserved for zero-semantic-loss removal only | RVS__…__S2 | IN-SCOPE BLOCKER | Fixed — added `proposed_rule: REMOVED` / `target_owner: NONE` as a valid paired special case, distinct from both `RETIRE` and an ordinary `INTENTIONAL-BEHAVIOR-CHANGE` |
| (S2 R5, Codex) AC5 checked only that `target_owner` was non-empty, not that its value was actually in the valid domain, unlike AC3's stronger check for `MOVE` | RVS__…__S2 | IN-SCOPE BLOCKER | Fixed — AC5 now enforces the full domain (`doctrine`/five named components/justified-new-component/`UNRESOLVED`/`NONE`-with-`REMOVED`) with the same Invariant-4 rigor AC3 applies |
| (S2 R6, Codex) AC8's `git diff <commit>` cannot see untracked files, so an untracked forbidden artifact under `dba/`/`configurations/` could exist while AC8 still passed | RVS__…__S2 | IN-SCOPE BLOCKER | Fixed — AC8 now also runs `git status --porcelain --untracked-files=all` against the forbidden paths; any hit requires explicit human attribution at Reconcile, not silent dismissal |
| (S2 R6, Codex) Trace-header comment, Reviews-table summary, and Findings-table rounds had each frozen at a different round number (R3/R4/R5) as later rounds ran | RVS__…__S2 | SELF-REFERENCE / REVIEW-BOOKKEEPING | Fixed — all three brought current to R6 in this same pass; optional per Codex, done anyway since these updates were already in progress |

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
