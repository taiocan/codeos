# UPG-0063 Q0 — Does a material NEW DESIGN decision exist in shipped DBA code?

<!--
Gates UPG-0063 entirely. If Q0 finds nothing material, the feature closes: no demonstrated
governance problem. Recording cost is deliberately NOT revisited here — first establish that
something needs recording.

Runs against EA-0001 (correcting the original mis-analysis) plus three PlotSpot modules (an
independent DBA project, previously unexamined).
-->

## 0. Method and decision rule — PRECOMMITTED before any classification

**Declared 2026-08-04, before reading PlotSpot's contracts or reclassifying EA-0001.**

### The error being corrected

UPG-0062 conflated *"does the contract prescribe the code structure or name?"* with *"does the
approved artifact already determine the semantic rule?"* Those are very different questions. Only the
second one matters. A contract that states a rule has governed it, whatever vocabulary the code then
uses to realise it.

### Classes

| Class | Meaning |
|---|---|
| `SOURCE-DETERMINED` | Approved artifacts already determine the rule; the code merely realises it |
| `ORDINARY IMPLEMENTATION CHOICE` | Artifacts leave freedom, but the choice carries no governance significance |
| `MATERIAL NEW DESIGN` | Artifacts leave multiple valid possibilities, **and** the selected mechanism materially determines invariant placement, component responsibility, state/data integrity, or future architectural freedom |

### The two-part test — both must be YES for `MATERIAL NEW DESIGN`

1. **Could another materially different mechanism satisfy every approved artifact without requiring an
   artifact revision?**
2. **Would choosing between those mechanisms matter enough that a future maintainer or reviewer should
   know it was deliberate?**

`NO` at (1) → `SOURCE-DETERMINED`. `YES` then `NO` → `ORDINARY IMPLEMENTATION CHOICE`.

### Explicitly not counted

Newtypes, resolver objects, helper seams, predicates, trait injection, and particular Rust structures
are **not** counted merely because they are absent from a contract. They may be nothing more than
implementation techniques for an already-governed semantic rule. Absence of the technique's *name*
from an artifact is not evidence of anything — that was the original error.

### Decision rule — fixed in advance

| Q0 outcome | Consequence |
|---|---|
| No `MATERIAL NEW DESIGN` across EA-0001 + PlotSpot | **Close UPG-0063.** No demonstrated governance problem |
| One isolated case | Record it, but **probably still close or downgrade** the feature |
| Repeated material cases across independent features/projects | **Proceed to Step 1**, with real evidence for the problem and some indication of its shape |

No renegotiation of these after the results are in.

---

## 1. Findings

**Sample:** EA-0001 (EvidenceAtlas) + PlotSpot F-0001/F-0002/F-0003 (`source_inventory`,
`dataset_profile`, `source_snapshot`) — two independent DBA projects with different authors' artifact
styles.

### 1a. The decisive discovery — the artifacts declare their own silence

The corrected method changed the finding's *shape*, not just its size. The strongest cases are not ones
where I inferred an artifact was silent. They are ones where **the approved artifact explicitly states
that it is leaving a question open**, and the implementation had to resolve it anyway in order to
exist.

That removes the method error entirely: there is no judgement call about whether the artifact
determines the rule, because the artifact says it does not.

| # | Explicit deferral (quoted from the approved artifact) | Where | How the implementation resolved it | Class |
|---|---|---|---|---|
| 1 | *"Validation ordering is not prescribed."* | `events/F-0001_schema.md:178`, `F-0002_schema.md:200`, `F-0003_schema.md:201` | `record_official_candidate` checks country → responsibility → identity → claim-evidence and returns on the **first** failure. For a multiply-invalid input, which failure signal the caller observes is fixed by this order | **MATERIAL NEW DESIGN** |
| 2 | *"canonical ownership is unresolved until Architecture Synthesis."* | `contracts/F-0001_contract.md:111`, `F-0002:108`, `F-0003:106` | A hardcoded literal map inside the feature module (`canonicalize_representations`), lowercase-matching eight known values and passing anything else through unchanged | **MATERIAL NEW DESIGN** |
| 3 | *"MANUAL-PENDING: validator semantics — orchestration only"* | `contracts/EA-0001-…_contract.md:148,149,151` | A `ResearchContractValidator` trait seam; orchestration fully implemented, validator semantics left to an injected implementation | **MATERIAL NEW DESIGN** |
| 4 | *"No evaluation order between `NoApprovedPlan` and `StalePlanVersion` is prescribed."* | `events/EA-0003-…_schema.md:215` | Not yet implemented — recorded as a pending instance of the same pattern | (pending) |

Each of rows 1-3 passes both halves of the precommitted test:

- **Could another materially different mechanism satisfy every approved artifact without an artifact
  revision?** Yes, trivially — the artifact says the question is not prescribed. Any order, or
  reporting all failures; a resolver seam, a Policy-Registry lookup, or a local map; validation in the
  module or injected.
- **Would the choice matter enough that a maintainer or reviewer should know it was deliberate?** Yes.
  Row 1 fixes observable failure behaviour a consumer may come to depend on. Row 2 places vocabulary
  ownership somewhere the contract says is *unresolved*, so it must move when Architecture Synthesis
  resolves it — and nothing records that it is interim. Row 3 determines where validation authority
  lives.

### 1b. What did NOT survive the test

Correctly excluded under the precommitted rules — these are implementation technique for an
already-governed rule, not governance-significant decisions:

- EA-0001's `is_locked` conjunction and version-binding — **`SOURCE-DETERMINED`**, stated at
  `contracts/EA-0001-…_contract.md:134` with a falsification row at 157. (This is the UPG-0062
  retraction, confirmed.)
- Newtypes, `non_empty`, `collect_discovery_evidence_refs`, the JSONL exporter, trait injection as
  such — **`ORDINARY IMPLEMENTATION CHOICE`**. Absent from contracts, but absence of a technique's
  name is not evidence of anything.
- Transaction boundary (insert + append event in one commit) — **`SOURCE-DETERMINED`**;
  `architecture/core-baseline.md:102` covers the transaction boundary.

### 1c. Verdict against the precommitted decision rule

**Repeated material cases across independent features and projects** — three confirmed instances
spanning two projects and four features, plus a fourth pending instance. Per §0's fixed rule, that is
the **"proceed to Step 1"** branch.

### 1d. The reframing this forces on UPG-0063

The brief's current problem statement is **wrong and should be rewritten before Step 1.** It says the
artifacts "do not determine by what mechanism a feature-local invariant is enforced." They largely
*do* — that was the retracted claim.

The real gap is narrower, better evidenced, and more tractable:

> **Approved artifacts explicitly defer specific questions. Implementations must resolve those
> deferrals in order to exist. Nothing records how they were resolved, or that the resolution is
> interim.**

This matters for the remedy's shape and cost. The record does not need to survey every decision in a
feature — it keys to the artifacts' own explicit deferrals, which are **enumerable from the artifacts
themselves** (`grep` for "not prescribed", "unresolved", "MANUAL-PENDING", "left open"). That is a far
leaner mechanism than anything previously contemplated, and it is checkable: a deferral with no
recorded resolution is a detectable omission.

---

## 2. Incidental finding — an apparent contract violation in shipped PlotSpot code

Not part of Q0, surfaced while examining row 2. Reported because it is a live correctness question,
and flagged as *apparent* because I have not executed it.

F-0001's Vocabulary Dependency lists **"known access form"** among the concepts operated on, and its
Display invariant requires the displayed name to be the canonical representation *"regardless of the
representation used when the concept was recorded."*

In `modules/source_inventory/src/lib.rs`, `canonicalize_representations` is applied to
`access_limitations` (line 208) and `lifecycle_limitations` (209) but **not** to `known_access_forms`
(200, passed through unchanged). The contract's own Happy Path (line 80) shows access-form values
—`restricted`, `paid`, `non-machine-readable`— drawn from the same vocabulary the canonicalizer
normalises. So `"Restricted"` and `"restricted"` recorded as a known access form appear to produce
different displayed outcomes, which the Display invariant forbids.

Worth noting for UPG-0063's argument: this is exactly the class of defect a recorded deferral
resolution would expose. Writing down *"vocabulary canonicalization: local hardcoded map, applied to
limitations"* invites the immediate question "why not to access forms?" — which nothing currently asks.

