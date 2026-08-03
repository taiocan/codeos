# UPG-0060 — CHG-B Gate Evidence: realistic-feature net-token measurement

<!--
PURPOSE: Decision evidence for the CHG-B go/no-go on UPG-0060, not a change record. CHG-A
(changes/UPG-0060__CHG-20260802-001__deepseek-implement-tool.md) is COMPLETE and accepted; its own
pilot was a toy feature that proved the mechanism runs end-to-end but produced no measurable saving.
The human decision at CHG-A acceptance (2026-08-03) made CHG-B contingent on this second measurement:
one realistic downstream feature through delegated implementation, total DeepSeek + Claude
reconciliation/review cost against the Claude-only path. Proceed with CHG-B only if materially
net-positive.

No toolkit behavior changes here. Nothing was promoted or committed in either repository.
-->

```yaml
feature_id: UPG-0060
gate: CHG-B go/no-go
date: 2026-08-03
target: EvidenceAtlas EA-0003 corpus_construction, Stage 4 (Implementation)
tool: scripts/codeos-implement.sh (CHG-A, shipped disabled)
delegate_model: deepseek-chat, temperature 0
verdict: NOT NET-POSITIVE — do not proceed with CHG-B as designed
decision: HOLD AT CHG-A, feature PILOTED (negative) — human, 2026-08-03 (option 1 of §5)
journaled: AJ-022
```

---

## 1. Why EA-0003

The measurement needed a feature that is realistic rather than illustrative. EA-0003 qualifies on
every axis CHG-A's toy counter did not:

| Property | EA-0003 |
|---|---|
| Stage 2 contract | APPROVED (R7, 2026-08-01) — 320 lines, 22,472 bytes |
| Stage 3 event schema | APPROVED (R6, 2026-08-01) — 263 lines, 16,286 bytes |
| Contract scenarios | 4 scenarios + 3 boundary + 2 falsification, 9 invariants, 11 invariant-falsification rows |
| Architecture context | cohort member; approved baseline, cohort logical design, and Implementation Profile (Rust) all apply |
| Position | `current_stage: 3` — Stage 4 is genuinely the next step, not a contrived exercise |

The full realistic packet was sent: intent, contract, event schema, `architecture/core-baseline.md`,
`architecture/cohort-logical-design.md`, `architecture/implementation-profile.yaml` — 101,166 bytes,
the same material a Claude-only Stage 4 would read.

The mechanism was enabled in EvidenceAtlas only for the duration of the run, by a runner with an
`EXIT` trap that removes the activation file (`.codeos-state/run-ea0003-delegated.sh`). EvidenceAtlas
has no `architecture/delegated-implementation.yaml` afterwards, and nothing was committed there.

---

## 2. Arm A — delegated (measured)

```
candidate staged: .codeos-state/deepseek-candidates/EA-0003-stage-4/20260803T034517Z.Utrvld/candidate
  tokens: prompt=23824 completion=4613 total=28437   model=deepseek-chat
  wall time: 29s   exit code: 0
```

One candidate file, `modules/ea0003_corpus_construction/src/lib.rs`, 466 lines / 16,779 bytes.
The tool behaved correctly throughout: stage-area allowlist held, no key in any artifact, audit set
complete, activation restored.

### 2a. The candidate does not compile

Delivered as-is it cannot be built at all — no `Cargo.toml` accompanied it, and the model returned
exactly one file. Supplying a manifest by hand (mirroring EA-0001's) still fails:

```
error[E0599]: the method `insert` exists for struct `HashMap<(RequirementType, String), bool>`,
              but its trait bounds were not satisfied
              `RequirementType: Hash` ... is required by `(RequirementType, String): Hash`
error[E0599]: the method `get_mut` exists for struct `HashMap<...>`, ...
error: could not compile `ea0003_corpus_construction` (lib) due to 2 previous errors
```

`RequirementType` is used as a `HashMap` key without `#[derive(Hash)]`. Cheap to fix; recorded
because it is objective and because it means the draft was never executed by its author.

### 2b. Eight confirmed contract/schema violations

After the minimum repair (add the `Hash` derive), the candidate compiles and the approved contract's
own scenarios were run against it. Each test below asserts the **observed** behavior and passes,
which is what confirms the violation. Suite: `/tmp/ea0003-candidate-check/tests/scenarios.rs`,
10 tests, all passing.

| # | Approved requirement | Observed behavior | Source |
|---|---|---|---|
| V1 | Schema: `scope_fully_examined` is `true`/`false` for an unresolved result with empty `traceable_source_ids`; "`null` never means examined-but-unknown" | Caller's `Option<bool>` passed through with no derivation and no validation; emits `null` | schema §`CorpusConstructionCompleted` |
| V2 | Falsification `DuplicateDoesNotInflateCoverage` — a mirrored/syndicated copy must not count as independent coverage | No deduplication logic exists; `duplicate_of` is a passthrough field; a mirror yields 2 traceable sources | contract §Falsification |
| V3 | Concept Dependency + Representation Ban + Display invariants — "Weak" and "weak" must produce identical outcomes and a canonical display value | `quality_characteristic` is a raw `String` passthrough; the two representations produce different reports | contract §Vocabulary Dependency |
| V4 | Schema: `unresolved_importance` "present only when `status` is `unresolved`" | Serialized unconditionally; accepted and emitted on a `satisfied` result | schema §`requirement_results` |
| V5 | Schema: a source's `requirement_type` is exactly `research_question` or `required_perspective` — "a source is never mapped to a stopping criterion" | `MappedRequirement` reuses the full enum; `stopping_criterion` is representable and is emitted | schema §`mapped_requirements` |
| V6 | Boundary `StoppedWithoutSatisfyingCriteria` — the report must distinguish a non-criteria stop | `StoppingBasis::Other("stopping_criteria_met")` is accepted and erases the distinction | contract §Boundary |
| V7 | Base field `timestamp` | `now_millis()` returns the hardcoded literal `1710000000000` with a comment saying a real implementation would use `SystemTime` | schema §Required Base Fields |
| V8 | Invariant: a started record always exists before a completed record for the same attempt; each attempt has its own distinct identity | Not enforced — `complete_construction` builds a Completed event for an attempt that never started, with any identity the caller invents. Its own doc comment claims it validates "that the corpus version matches the started one"; no such check exists | contract §Invariants |

What the candidate did get right (2 confirming tests): both failure paths (`NoApprovedPlan`,
`StalePlanVersion`) and their events; `predecessor_version_id` omitted rather than nulled on a first
attempt; and the "accounts for every requirement, never a subset" validation, which does work.

### 2c. The shape of the failure, not just its size

V1–V3 and V8 share one cause. The candidate implements the Execution Report as a **serializer**: the
caller computes `requirement_results`, decides `retained_in_corpus`, supplies `traceable_source_ids`
and `scope_fully_examined`, and the module writes them to JSON. Every invariant the contract exists
to protect is thereby delegated to an unspecified caller. That is not a defect that can be patched at
eight points — the invariants have no place to live in that design.

Notably, the model's own `notes.txt` flagged its weakest spot unprompted:

> "Reviewer should check that the event payloads match the schema exactly, especially the
> `scope_fully_examined` null handling and the `requirement_type` values."

That is V1 and V5. The draft is accompanied by an accurate warning that it is wrong.

---

## 3. Arm B — Claude-only (measured)

Same feature, same approved artifacts, written directly:
`.codeos-state/claude-candidates/EA-0003-stage-4/candidate/modules/corpus_construction/` —
661 lines / 26,374 bytes plus a `Cargo.toml`.

- `cargo check` — **clean on the first attempt**, no repair cycle.
- 10 scenario tests derived from the approved contract — **10/10 pass**, covering both failure paths,
  omitted-not-null predecessor, `DuplicateDoesNotInflateCoverage`, `WeakButRequiredMaterialRetained`
  with unusable material excluded, representation-substitution invariance, `scope_fully_examined`
  never null for an unresolved no-source result and correctly split `true`/`false` across
  `NoMaterialFoundForRequirement` vs `StoppedWithoutSatisfyingCriteria`, "never a subset", the
  mapped-requirement restriction, and the reserved stopping-basis value.

The design differences that carry the invariants: coverage credited per *distinct underlying source*
so duplicates cannot inflate it; a `ClassificationResolver` seam so representations never reach
domain logic and only canonical values are displayed; `scope_fully_examined` **derived** rather than
accepted; a `StartedConstruction` handle that only `begin_construction` can mint, making
started-before-completed unrepresentable otherwise; `CoverageTarget` as a narrower type than
`RequirementType`, making V5 impossible by construction; and injected identity/clock rather than a
stubbed timestamp.

These are not stylistic preferences. Each one is a specific line of the approved contract or schema.

---

## 4. Token accounting

Measured exactly: the DeepSeek counts, and the byte size of every artifact on both sides. Claude-side
token figures are derived from those measured byte counts (~3.7 bytes/token for code and prose), and
are labeled as derived. No Claude-side meter was read.

| Cost component | Arm A (delegated) | Arm B (Claude-only) |
|---|---|---|
| DeepSeek prompt + completion | **28,437** (measured) | 0 |
| Claude: read intent + contract + schema + architecture context | ~13K in (derived) | ~13K in (derived) |
| Claude: read the 466-line candidate | ~4.4K in (derived) | — |
| Claude: compile, diagnose, run scenario suite against the draft | ~1K in (derived) | — |
| Claude: produce the implementation | ~7K out (derived) — the rework is a rewrite, see §2c | ~7K out (derived) |
| Advisory Codex review + human gate | identical in both arms | identical in both arms |

**Net: Arm A costs Arm B plus ~5.4K Claude input tokens plus 28,437 DeepSeek tokens, and saves zero
Claude output tokens.** The saving mechanism — Claude not writing the code — never engaged, because
the draft could not be kept.

Money is not the constraint being measured: 28,437 DeepSeek tokens costs a fraction of a cent. The
constraint is Claude's budget, and on that axis the delegated arm is strictly worse than not using it.

---

## 5. Verdict

**Not materially net-positive. Do not proceed with CHG-B as designed.**

The result is not a near miss to be tuned away, and it is not incidental to this one feature. DBA's
Stage 2 and Stage 3 artifacts exist to pin down exactly the subtleties a cheaper model drops —
falsification scenarios, invariant tables, and vocabulary invariants are, by construction, the parts
of a specification that a fluent-but-shallow draft will satisfy in appearance and violate in fact.
The more rigorously a downstream project applies the methodology, the less delegable its Stage 4
becomes. On a feature with a weak contract the draft would likely have survived — and that is exactly
the case where delegation matters least.

**Limits of this measurement, stated plainly.** One feature, one model (`deepseek-chat`), one
temperature-0 run. Not tested: `deepseek-reasoner`, a repair loop that feeds compiler and test output
back to the delegate, or Stage 5 test authoring, which is more mechanical given an approved contract
and whose failures are loud rather than silent. Any of these could change the answer; none of them is
CHG-B as currently scoped.

**Correction (2026-08-03, after the verdict was first recorded): the harness handicapped the
delegate.** Re-reading `prompts/codeos-implementer-task.md` against the packet shows part of §2's
finding is attributable to CHG-A's own prompt, not to the model:

| Reported as a model defect | Actually |
|---|---|
| No `Cargo.toml` accompanied the candidate | The prompt forbids it — *"Never emit a path that is not a source or test file"* + *"Add no … files … not traced to the approved artifacts."* `Cargo.toml` appears 0 times in the 105,510-byte packet |
| Module named `ea0003_corpus_construction` rather than the project's convention | The only `modules/` string in the whole packet comes from the prompt itself. No layout exemplar was supplied — the naming was a guess with nothing to guess from |
| The serializer design (§2c, called the root cause) | Plausibly induced in part: the prompt says *"Add no behavior, no files, no abstractions … not traced to the approved artifacts"*, which pushes a literal reader away from exactly the invariant-carrying structure whose absence §2c identifies |
| — | Output was additionally constrained to JSON-escaped source in a single shot with no compiler feedback; both are known to degrade generated code, and both are harness choices |

**What survives the correction and remains model-attributable:** the missing `derive(Hash)` on a
`HashMap` key (basic type-system error); a doc comment asserting a validation the function does not
perform; a knowingly-stubbed timestamp; a `#[cfg(test)]` module shipped against the prompt's explicit
*"Do not write tests in a Stage 4 candidate"*; and V1, whose governing schema sentence — *"`null` never
means 'examined but the outcome is unknown'"* — was present in the packet **twice** and still ignored.
Seven of the eight violations landed on invariant-dense contract text that *was* supplied.

**What this does and does not change.** The §5 decision stands: no doctrine, feature held at CHG-A.
The structural claim — verification cost does not compress the way generation cost does — is
independent of both harness and delegate. The stronger claim, that a cheap model *cannot* satisfy a
rigorous contract, is **not established by this run**. Accordingly the re-test conditions in the
feature brief were re-ordered: harness correction is now **condition 0, a prerequisite** gating any
model comparison, and any re-test is judged on three separately-reported axes (contract adherence,
technical correctness, net cost) with net cost decisive. See AJ-022's same-day amendment.

**Grader independence.** The same author wrote the comparator implementation, the violation suite, and
this document. Each violation is objective against quoted contract/schema text, but the count and the
framing are not independent of the grader.

**What this costs to leave as-is: nothing.** CHG-A shipped the tool off by default with no downstream
footprint — no `dba-system.md` text, no prompt text, no `dba-init.sh` scaffolding. Not doing CHG-B
leaves every downstream project exactly as it is today.

**Options for the human, in the order recommended:**

1. **Hold UPG-0060 at CHG-A, feature status PILOTED (negative).** Keep the tool, do not wire doctrine.
   Record a named re-test condition — a stronger delegate model, or a repair loop — so the question
   can be reopened on evidence rather than reopened on impulse. No downstream change, no cost.
2. **Re-scope CHG-B to Stage 5 only.** Test authoring from an approved contract is the more mechanical
   half, and a wrong test fails visibly instead of shipping a silent invariant violation. This would
   need its own measurement before any doctrine text.
3. **Abandon UPG-0060 outright**, per the abandonment path the feature brief already reserves.

Recommendation: option 1. It preserves the (already paid-for) mechanism, commits no doctrine, and
keeps the question answerable later without re-litigating it now.

### 5a. Decision (human, 2026-08-03)

**Option 1 taken.** UPG-0060 is held at CHG-A with feature status **PILOTED (negative)**; CHG-B will
not be done. The tool stays exactly as CHG-A shipped it — off by default, no `dba-system.md` text, no
prompt text, no `dba-init.sh` scaffolding — so no downstream project changes in any way. The three
named re-test conditions (a materially stronger delegate model; a bounded compiler/test-feedback
repair loop; a Stage-5-only re-scope) are recorded in the feature brief, each requiring its own gate
measurement before any doctrine text. The generalizable finding is journaled as **AJ-022** — *a
rigorous specification is a poor delegation target: Stage 4 delegability falls as contract rigor
rises*. Decision recorded in `reviews/review-log.md`.

---

## 6. Reproduction

| Artifact | Path |
|---|---|
| Runner (temporary enable + `EXIT`-trap restore) | `EvidenceAtlas/.codeos-state/run-ea0003-delegated.sh` |
| DeepSeek packet / request / response / tokens / sidecars | `EvidenceAtlas/.codeos-state/deepseek-candidates/EA-0003-stage-4/20260803T034517Z.Utrvld/` |
| DeepSeek candidate | `…/candidate/modules/ea0003_corpus_construction/src/lib.rs` |
| Invocation log | `EvidenceAtlas/.codeos-state/deepseek-candidates/implement-log.md` |
| Claude comparator candidate | `EvidenceAtlas/.codeos-state/claude-candidates/EA-0003-stage-4/candidate/modules/corpus_construction/` |
| Violation suite vs. DeepSeek candidate + its output + the as-delivered compile failure | `…/20260803T034517Z.Utrvld/verification/` (`scenarios.rs`, `scenario-suite-output.txt`, `cargo-check-as-delivered.txt`, `Cargo.toml.supplied-by-hand`) |
| Scenario suite vs. Claude candidate + its output | `…/claude-candidates/EA-0003-stage-4/verification/` (`smoke.rs`, `scenario-suite-output.txt`) |

Neither candidate was promoted into `modules/`; nothing was committed in either repository;
EvidenceAtlas's tracked files are unchanged by this measurement.

### 6a. Verbatim output

Compile of the DeepSeek candidate as delivered, with a hand-supplied `Cargo.toml` and no other edit
(`cargo-check-as-delivered.txt`):

```
error[E0599]: the method `insert` exists for struct `HashMap<(RequirementType, std::string::String), bool>`, but its trait bounds were not satisfied
error[E0599]: the method `get_mut` exists for struct `HashMap<(RequirementType, std::string::String), bool>`, but its trait bounds were not satisfied
    |                                       ^^^^^^^ method cannot be called due to unsatisfied trait bounds
error: could not compile `ea0003_corpus_construction` (lib) due to 2 previous errors
```

Approved-contract scenarios against the DeepSeek candidate, after adding only the missing `Hash`
derive. Every `violates_*` test asserts the observed non-conforming behavior, so passing **confirms**
the violation:

```
running 10 tests
test conforms_rejects_missing_requirement_result ... ok
test conforms_failure_paths_and_started_event ... ok
test violates_duplicate_does_not_inflate_coverage ... ok
test violates_scope_fully_examined_never_null ... ok
test violates_started_before_completed_invariant ... ok
test violates_source_never_mapped_to_stopping_criterion ... ok
test violates_representation_substitution_invariance ... ok
test violates_unresolved_importance_presence_rule ... ok
test violates_timestamp_is_a_hardcoded_stub ... ok
test violates_stopping_basis_distinction ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The same scenarios against the Claude candidate, where each test asserts **conforming** behavior:

```
running 10 tests
test mapping_to_unplanned_requirement_is_rejected ... ok
test no_approved_plan_rejected ... ok
test report_accounts_for_every_requirement ... ok
test reserved_stopping_basis_rejected ... ok
test duplicate_does_not_inflate_coverage ... ok
test stale_plan_rejected ... ok
test started_omits_predecessor_on_first_attempt ... ok
test weak_retained_unusable_excluded ... ok
test representation_substitution_is_outcome_invariant ... ok
test scope_fully_examined_is_never_null_for_unresolved_no_source ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
