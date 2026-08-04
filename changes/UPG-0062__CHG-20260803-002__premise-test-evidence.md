# UPG-0062 CHG-20260803-002 — Premise Test Evidence

<!--
PURPOSE: The measured result of the Q1 premise test and the Q2 governance finding. Q1 determines
whether UPG-0062 proceeds to delegated tooling at all. Q2 is independent of Q1 and was answered
regardless, per the human's instruction to separate proving the gap from solving it.
-->

```yaml
change_id: CHG-20260803-002
date: 2026-08-04
q1_verdict: STOP — both precommitted thresholds missed
q2_verdict: GAP EXISTS (established in one shipped feature; prevalence NOT established)
```

---

## 1. Q1 — is producing the Feature Implementation Design materially cheaper than implementing?

### 1a. Ordering, which is what makes this honest

| Step | Artifact | Committed at |
|---|---|---|
| Threshold precommitted, before any measurement existed | change record §Step 3.0 | `7fb8c38` |
| FID written from approved artifacts only, frozen + hashed | `…__ea0004-fid-experimental.md`, sha256 `9bd4a6b8e77fae17f3012e27beb91e94f846944672002d4acf11adb0527b14d0` | `c879fe6` |
| Arm B implemented in an isolated workspace | `/tmp/armb-ea0004/` (never committed, AC-5/AC-13) | — |

The threshold commit precedes the FID commit, which precedes Arm B. No number was visible when the
bar was set.

### 1b. Measurement

Basis: byte counts at ~3.7 bytes/token, output weighted 5× input, both declared in §Step 3.0 before
measuring. Input is identical across arms — both read the same approved artifacts (89,911 bytes).

| | Bytes | ~Tokens |
|---|---|---|
| Common input (both arms) | 89,911 | 24,300 |
| **FID output** | 11,980 | 3,238 |
| **Arm B output** (`lib.rs` + `Cargo.toml`) | 19,361 | 5,233 |

| Ratio | Measured | Precommitted threshold | Result |
|---|---|---|---|
| FID output ÷ Arm B output | **0.619** | ≤ 0.400 | **FAIL** |
| FID weighted total ÷ Arm B weighted total | **0.802** | ≤ 0.500 | **FAIL** |

Arm B compiles clean and satisfies the contract it was written against, so it is a fair comparator
rather than a strawman. Verbatim, from `/tmp/armb-ea0004/` (`src/lib.rs` sha256 `f46a069c48ec9c0c1a34d1e0f4825c49c746f455fec98ddc09dc546a206913cd`):

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
```

The implementation itself is deliberately not committed (AC-5/AC-13 — this change is classified
no-code). That keeps the classification honest at the cost of the source not being in the review
packet; the hash and build output above are what can be carried without breaking the classification.

### 1c. Verdict — STOP

Both thresholds missed, and not narrowly: producing the design cost **62% of what implementing cost**.
Per §Step 3.0, a result between 0.50 and 1.0 is *cheaper but not materially cheaper* and counts as a
failure. **UPG-0062 does not proceed to delegated tooling.** No Rust engine is built, no shim reduced,
no prompt rewritten, no pilot run.

### 1d. Why the 10:1 hope did not survive contact — the predicted bias, confirmed

UPG-0060's repair feedback was ~2.5 KB and produced 610 lines of correct Rust, suggesting ~10:1
leverage. AC-1 flagged the reason to distrust that figure: **it was written with a failed candidate in
hand.** It only had to name six deltas against existing code. It never had to specify the other
90% of the design, because that part was already right.

Writing from approved artifacts alone has no such shortcut. Every mechanism must be stated, including
the ones a competent implementer would have got right unaided. The 10:1 ratio measured the cost of
*correcting* a design; the 0.62 ratio measures the cost of *specifying* one. Those are different
quantities, and only the second is what delegation would actually require.

This is the guardrail working as designed: AC-1 existed precisely to stop the encouraging-but-wrong
number from being measured, and the result it produced was the discouraging-but-right one.

### 1d-bis. A defect in the FID, recorded rather than repaired

Step 4 review found that FID row **A8** ("module directory `modules/evidence_extraction/`, crate name
matching") is classified `SOURCE-DERIVED` but cites *"existing repository convention"* and workspace
membership — not a section of an approved artifact. By AC-2's own rule that is a misclassification:
a layout convention inferred from a sibling module is closer to `NEW DESIGN`, or at minimum is not
artifact-cited. **AC-2 therefore does not fully pass.**

The FID is **not edited to fix it.** It was frozen and hashed before Arm B precisely so the ordering
AC-1 depends on is verifiable; silently editing it afterwards would destroy the integrity the whole
measurement rests on. The defect is recorded here instead.

Effect on the results: none material. A8 is one of eight *frame* rows, not one of the ten mechanism
allocations. Q1's cost ratios are unchanged (the row exists either way). Q2's finding is unchanged and
if anything slightly understated — reclassifying A8 would move one more row from `SOURCE-DERIVED` to
`NEW DESIGN`, widening the gap rather than narrowing it.

### 1e. What this does not claim

It does not show a cheaper model is incapable, or that a shorter FID could not exist. It shows that
*this* design, written to the completeness the contract demands, cost 62% of implementing directly —
and that the remaining 38% would have to absorb the delegated candidate's reconciliation, which
UPG-0060 measured as substantial. One feature, one author, one estimation method.

---

## 2. Q2 — does Codeos need a governed home for mechanism decisions?

Answered independently of Q1, per instruction. Q1's failure closes the delegation question; it does
not touch this one.

### 2a. Classification of EA-0004's design

Of the ten invariant→mechanism allocations in the frozen FID, **ten are `NEW DESIGN`.** Four carry a
`SOURCE-DERIVED` component (a field list, a two-state rule, an ownership assignment), but in every
case the *mechanism* — the thing that makes the invariant hold — was chosen, not read.

The eight `SOURCE-DERIVED` frame rows cover language, validation ownership, transactionality,
correlation propagation, event shapes, field lists, integration style, and module layout. Genuinely
useful, and none of it says how a single invariant is enforced.

Two questions surfaced that no approved artifact answers at all: where conflation-resistant
*rendering* belongs (FID B3), and who owns duplication classification (FID B10 — explicitly left open
by the contract itself).

### 2b. Existence check against a shipped feature — EA-0001

`modules/research_brief/src/lib.rs` contains, among others:

| Mechanism in shipped code | Named in approved contract? | Named in approved schema? |
|---|---|---|
| `ResearchContractValidator` trait — a resolver seam for validation | no | no |
| `is_locked(...)` — lockedness computed by combining an injected `LockDecisionReference` with a `ResearchBeganReference` | no | no |
| `evaluate_change` as the gate through which changes must pass | no | no |
| `LockDecisionReference` / `ResearchBeganReference` as distinct injected types | no | no |

The approved contract says "Locked" fifteen times — as a *state*. It never says lockedness is computed
by conjunction of two injected references, nor that a validator seam exists. Those are real
architectural decisions, they are load-bearing, and they live only in the code.

**Conclusion: the gap exists.** A real, human-approved, shipped Stage 4 implementation contains
material design decisions that no approved artifact determines or records. They were approved only
implicitly, by approving the code that embodied them.

### 2c. What is NOT claimed

**Existence only, not prevalence.** One feature establishes "this gap exists in at least one real
implementation". It cannot support "every feature", "Stage 4 always", or any frequency claim. If
prevalence matters, it needs 2–3 representative features examined in separate work — not by widening
this change.

### 2d. Q2 verdict, and the deliberate refusal to solve it here

A governed home for feature-level implementation-design decisions **is required** — the decisions are
real, load-bearing, and currently unrecorded.

**This change does not design it.** Per AC-9, proving the gap and solving it are separate. No
mechanism, gate, template, lifecycle, or artifact is proposed here. A separate feature (**UPG-0063**) is filed to
solve it. Its working hypothesis is deliberately lean — expose only load-bearing `NEW DESIGN`
decisions inside the *existing* Stage 4 workflow and gate, with no new stage, artifact, or gate — and
its cost basis is explicitly **not** this document's 62%: that figure measured deriving a complete
design up front, whereas Stage 4 makes these decisions anyway, so UPG-0063's true cost is only the
marginal effort of recording an already-necessary decision.

Per AC-12: any future delegated-tooling work is **blocked** until that capability exists, because a
delegation engine consuming an experimental non-authoritative artifact is precisely the ungoverned
second architecture authority these guardrails exist to prevent.

---

## 3. Consequences

| Item | Outcome |
|---|---|
| UPG-0062 delegated tooling (CHG-C) | **Not started, and now doubly blocked** — Q1 failed, and Q2 requires a governance capability that does not exist |
| The experimental FID | Stays non-authoritative. Governs nothing. Retained as evidence only |
| Arm B implementation | Stays in `/tmp/armb-ea0004/`, uncommitted. Evidence, not a deliverable |
| EA-0004 | Unchanged in EvidenceAtlas. No implementation promoted, no artifact added |
| The governance gap | Filed as a separate feature; not solved here |

**On EA-0004 contamination (AC-15):** a Claude implementation of EA-0004 now exists in scratch. Should
any future delegated pilot use EA-0004, that baseline exists and must be declared. Given Q1's result,
no such pilot is currently planned.

---

## 4. Honest summary

The delegation hypothesis this feature was built to test is **not supported**. The saving depends on
the specification being much cheaper than the implementation, and measured properly — from approved
artifacts, with no candidate in hand — it was not.

The feature's more durable output is the thing it found on the way: Codeos has a real gap between
logical architecture and code, and Claude has been bridging it silently inside Stage 4 implementations.
That was worth discovering, and it is worth fixing on its own terms rather than as a means to
delegation.
