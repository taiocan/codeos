---
feature_id: UPG-0072
slug: gemini-frontier-qualification
title: Gemini 3.7 Flash Qualification Across Three Codeos Roles
status: DECIDED — NO / NO / YES for gemini-3.7-flash at reasoning_effort high (2026-08-23); adoption since rejected and provider switch removed by UPG-0073
priority: P2
depends_on: []
related_features: [UPG-0060, UPG-0064, UPG-0066, UPG-0069, UPG-0071]
supersedes: []
superseded_by: []
---

# Upgrade: Gemini 3.7 Flash qualification

## Question

UPG-0060/0064/0066/0069/0071 asked whether a non-Claude model can carry three Codeos roles —
reviewer fallback, Stage-4 implementation, Stage-5 test generation. Every arm was DeepSeek, and
every verdict was NO. UPG-0060 closed the line on one condition: *reopening requires a new, specific
hypothesis with its own measurement — not another harness round.*

The hypothesis, stated narrowly:

> `gemini-3.7-flash` at `reasoning_effort: high`, given the same frozen task evidence and the same
> acceptance bars, clears at least one Codeos role that the tested DeepSeek configurations did not.

The permitted readings of the result are exactly these:

```text
0 of 3 roles clear   -> gemini-3.7-flash @ high is not viable for these roles
>=1 role clears      -> at least one DeepSeek negative does not generalize to
                        gemini-3.7-flash @ high
```

This run changes both the model and the provider API implementation at once, so nothing further may
be inferred — not "provider-specific", not "delegation-specific", not anything about tier. Only the
reviewer packet is proven byte-identical, by hash; Stage 4 and Stage 5 reuse the same artifacts,
roles, source commits, and bars, which makes the comparison controlled but not byte-identical at the
request envelope.

Three roles are decided **independently**. `NO / YES / YES` is a valid and useful outcome.

## Frozen controls — fixed 2026-08-23 before the first billed call

| | Value |
|---|---|
| Model | `gemini-3.7-flash`, `reasoning_effort: high`, OpenAI-compatible endpoint |
| Ladder | **none.** One model arm only. A negative result closes the question; no less advanced Gemini model is tested |
| Output bounds | default 32768, then **one** explicit 65536 retry after `length`; every attempt counts |
| Model repair iterations | **none.** One bounded mechanical compile repair is permitted under the Stage-5 repair rule below; no second model call and no semantic repair |
| Reviewer packet | `maintenance/reviews/experiments/UPG-0069-deepseek-comparison/canonical-packet.txt`, sha256 `2a5ed7d4…6894f`, byte-for-byte. **Not regenerated** |
| Codex arm | not re-run; the recorded arm is reused as comparator |
| Stage-4 cases | EA-0010 `policy_registry`, EA-0003 `corpus_construction` (EvidenceAtlas `46e0a8b`) |
| Stage-4 case 3 | **BLOCKED**, unchanged — no approved package carries a Stage 1-3 deferral still open at Stage 4 entry, and no easier case substitutes |
| Stage-5 features | F-0001 `source_inventory`, F-0004 `source_fitness`, F-0006 `regional_availability` (PlotSpot `c3b8215`) |
| Stage-5 mutations | UPG-0071's nine, verbatim; an inert mutation is reported, not scored |
| Attribution | to the tested configuration `gemini-3.7-flash` @ `high` — never to provider or tier |
| Pooling | Gemini figures are **never** pooled with `deepseek-chat`, V4-Flash, V4-Pro, or Codex. No DeepSeek or Codex evidence file is modified |

### Why `high` and not a higher setting

`reasoning_effort: "max"` is rejected by this API with `400 Invalid reasoning_effort: max. Valid
values are: high, low, medium, minimal, none`. `high` is the ceiling, so the frozen configuration is
the strongest this model offers.

## Verified API facts — probed 2026-08-23, before any arm

Measured, not assumed. Each one drives a design decision:

| Fact | Consequence |
|---|---|
| The OpenAI-compatible endpoint accepts `Authorization: Bearer` and returns `.model`, `.choices[0].finish_reason`, `.choices[0].message.content`, `.usage` | The existing curl/jq transport shape is reusable unchanged |
| `thinking:{type:"enabled"}` → **HTTP 400**, `Unknown name "thinking"` | The request body must be provider-conditional; env vars alone cannot reach this |
| `reasoning_effort:"max"` → **HTTP 400** | Frozen effort is `high` |
| `usage` carries only `prompt_tokens`, `completion_tokens`, `total_tokens` — no `reasoning_tokens`, no cache fields | Report those three as returned. The residual `total − prompt − completion` is recorded as **unclassified**, never as reasoning |
| `completion_tokens` **excludes** that residual, where DeepSeek's `completion_tokens` includes its reported reasoning | Final-content tokens = `completion_tokens` directly. UPG-0071's `completion − reasoning` formula does not apply here and is not carried over |
| `max_tokens` **includes** thinking: a 256-token bound returned `finish_reason: length` with 23 characters of content | The 32768 → one 65536 retry rule transfers, and V4-Flash's zero-output failure mode is reachable here too |

## Measurement

Per attempt: returned model identity, prompt / completion / total tokens as reported,
`unclassified_tokens_derived`, **final-content tokens** (`completion_tokens`), finish reason, wall
time.

Claude cost is measured from actual Anthropic session usage with a narrow boundary:

```text
Claude supervision cost =
  Claude usage from the first inspection of the Gemini candidate
  through the final usable/unusable judgement.

Excluded: experiment setup, API execution and waiting, bookkeeping,
          cross-experiment synthesis, unrelated repository work.
```

Engineering quality, usability without rewrite, Claude supervision cost, Gemini token cost, and wall
time are reported separately and never collapsed into one score. Where no credible direct-Claude
baseline exists, `Claude-token savings: UNKNOWN`; nothing is inferred from a counterfactual.

## Per-role acceptance

Unchanged from the briefs that own them, so the bars were not set by anyone who had seen a Gemini
result.

| Role | Owning brief | Bar |
|---|---|---|
| Reviewer fallback | UPG-0069 | The seven frozen measures: true findings, false positives, missed important defects, protocol compliance, completion, cost, decision usefulness |
| Stage-4 implementation | UPG-0064 | Usable without rewrite, on the two runnable frozen cases; Codex advisory review of each candidate |
| Stage-5 test generation | UPG-0066 | ≥2 of 3 candidates usable without rewrite |

### Stage-5 rules frozen up front

UPG-0071 had to append these after the fact when its verdict rested on a compile error, which does
not distinguish "almost usable but mechanically wrong" from "semantically wrong". Freezing them
before the run means one pass yields a real verdict.

**Repair boundary.** Mechanical repair may make a candidate compile against interfaces already
present in source. It must not change expected values, assertions, control flow, scenario coverage,
or any behavior a helper represents. Adding a missing required field is mechanical only if its value
is irrelevant to the behavior under test; if choosing the value requires interpreting an approved
artifact, that is **semantic** repair and the feature FAILS. Committed direct-path suites are not
opened during repair — repair uses module source and the compiler's own errors only.

**Specification-drift rule.** A candidate test that fails against the real implementation is **not**
a candidate failure when the failed assertion is directly required by an approved artifact and the
implementation is independently confirmed nonconformant. A test supported by the approved
specification is not wrong merely because a nonconformant implementation fails it. Applies
symmetrically to every feature:

```text
candidate disagrees with implementation
  -> is the candidate supported by approved artifacts?
       NO  -> candidate defect
       YES -> implementation defect (not a candidate failure); record the drift
```

Confirming nonconformance means checking the approved package and its history. UPG-0071's F-0001
candidate found live specification drift in PlotSpot that the committed direct-path suite
structurally cannot see; that outcome must remain reachable here.

## Harness — `CODEOS_LLM_PROVIDER` is experiment support, not architecture

Reaching this API needs more than configuration: the request body differs (`thinking` is rejected)
and the usage shape differs. A `CODEOS_LLM_PROVIDER` switch, defaulting to `deepseek`, is added to
`dba/04-tools/implementer/codeos-implement.sh` and `dba/04-tools/reviewer/codeos-review-deepseek.sh`.
The DeepSeek request body and `tokens.txt` line stay byte-identical, so the existing suites run
unmodified and are themselves the isolation proof.

**The switch carries no presumption of permanence.** Its disposition is a separate decision taken
after the verdict:

```text
any role passes -> retention/adoption is its own decision, on its own evidence;
                   passing an arm does not by itself make the Gemini path permanent
all roles fail  -> preserve the evidence (this brief, the curated experiment files,
                   the commits), then remove the Gemini branch, its env vars, and its tests
```

Removal is the default outcome on a negative result, not a later argument. Nothing here advertises
the switch as a supported extension point, and no downstream documentation, template, or
configuration references it. The reviewer engine is untouched: `engine/src/provider/` is empty
because UPG-0032's multi-provider trait was deliberately removed, and nothing here reinstates it.

## Stop rules

- One model. If `gemini-3.7-flash` fails a role, no lesser Gemini model is tested for it.
- Two attempts per call, maximum: 32768, then one 65536 retry after `length`.
- No model repair iteration and no packet change. If a result turns on something the frozen protocol
  forbids, record the open question — it is a new experiment needing its own human decision, not an
  extension of this one.

## Results — ran 2026-08-23

Every attempt across all three arms returned `model: gemini-3.7-flash` at `reasoning_effort: high`;
no attempt silently fell back to another model. The canonical packet hash matched before and after
Arm 1, and both downstream repositories ended at their frozen commits with the temporary activation
files removed.

### Arm 1 — reviewer (UPG-0069 DeepSeek arm re-run; Codex not re-run)

Same `canonical-packet.txt`, hash verified before and after. `stop` on the **first** attempt at the
default 32768 bound: 42,279 prompt, 1,069 final content, 51,219 total, 25 seconds.

| Measure | Flash | V4-Pro | **Gemini 3.7 Flash** | Codex |
|---|---|---|---|---|
| True findings | 1 | 2 | **1** | 3 |
| False positives | 0 | 0 | **0** | 0 |
| Missed important defects | both integrity | both integrity | **both integrity** | none |
| Protocol compliance | FAILED | FAILED | **PASSED** | PASSED |
| Completion | 2 attempts | 1 attempt | **1 attempt** | 1 attempt |
| Cost | 149,975, ~12 min | 69,042, ~8 min | **51,219, 25 s** | 57,958, ~3 min |
| Decision usefulness | not self-sufficient | not self-sufficient | **self-sufficient** | self-sufficient |

Operationally this is the strongest non-Codex arm recorded, and the **first** to satisfy the output
protocol: `parse_status: OK`, `assessment_status: COMPLETE`, one finding recorded, zero unparsed.
Both DeepSeek arms failed there. It is also the cheapest and fastest arm of the four, Codex included.

The finding is real: the reviewed header documents the import command without the `--packet` flag
clap requires, verified at packet line 365.

It is nonetheless **unsafe as a review record**, for the same reason V4-Pro was. It missed both
packet-integrity defects the Codex arm found and **listed both among its satisfied and supported
claims** — asserting that the exported bytes are the reviewed bytes, and that untracked-file
discovery downgrades coverage. Those are exactly the two defects still live at HEAD as UPG-0070. A
reviewer that misses a defect is weak; one that certifies the broken property is worse.

Its scope-drift note about `sds-dba.md` and `software-development-structure.md` is factually true of
the reviewed snapshot — the sidecar records both as untracked budget contributors — so it is recorded
as a snapshot observation, not counted as a false positive.

**Reviewer fallback: NO.** Better than every prior non-Codex arm on completion, cost, and protocol,
and still unusable as a review record.

### Arm 2 — Stage 4 (UPG-0064 frozen cases)

Both cases: `stop` on the **first** attempt at the default bound, then a clean build first try in an
isolated worktree, one worktree per candidate. All five role labels were present in each preserved
packet.

| Case | Attempt 1 | Wall | Build | Codex review |
|---|---|---|---|---|
| EA-0010 | 48,205 total, 6,315 final content | 61s | clean | `DO NOT ADVANCE`, evidence A, 3 IN-SCOPE BLOCKERS (2 High, 1 Medium) |
| EA-0003 | 50,463 total, 6,105 final content | 59s | clean | `DO NOT ADVANCE`, evidence A, 3 IN-SCOPE BLOCKERS (2 High, 1 Medium) |

Speed is the headline: one attempt each in about a minute, against DeepSeek's two attempts and
12-22 minutes. Registering the new crate in the workspace members list was needed for both, applied
identically, and is build setup rather than candidate content.

**The EA-0010 headline defect reproduces for the third model in a row.** Codex: "Lookup accepts only
a target decision point and filters solely by exact target equality. It ignores every version's
`scope` and `applicability`, so a non-applicable sole match is labeled `Authoritative`." That is the
same applicability-blind lookup both DeepSeek configurations produced, confirmed by direct reading of
`lookup_applicable_rule`. Three models, three providers, same defect: this is a systematic reasoning
failure on invariant-dense specification, not provider noise.

EA-0003 is the same shape. Its candidate does supply an `execute_corpus_construction` orchestrator
returning start and completion together — better than the DeepSeek arms on that point — but
`evaluate_construction_completion` remains public and emits a completion event with no reference to
whether a start occurred, so the ordering invariant is still non-structural.

Neither candidate is usable without rewrite; both fail on the invariant-dense core rather than on
mechanics. **Stage-4 implementation delegation: NO.**

### Arm 3 — Stage 5 (UPG-0066 frozen features)

F-0001 completed at the default bound; F-0004 and F-0006 hit `length` and completed on the single
permitted 65536 retry. Recorded in the three dimensions the protocol fixes, not collapsed:

#### Executability — after the predefined mechanical adaptation allowance

| Feature | Raw compile errors | Repair classes | Result |
|---|---|---|---|
| F-0001 | 0 | none | compiles as delivered, 19 tests |
| F-0004 | 112 | 4 | compiles after mechanical repair, 20 tests |
| F-0006 | ~90 | 4 | compiles after mechanical repair, 24 tests |

The repair burden is recorded because its **shape** matters more than its size: 112 error instances
collapse to a handful of repeated interface misunderstandings, not diverse semantic failures.
The classes were: `EventMeta.timestamp` → `timestamp_ms`; direct private-field reads → the existing
`event_type()` / `source_module()` / `correlation_id()` / `payload()` accessors; a pure forwarding
`Store` impl for `&mut InMemoryStore` in the test file; and borrow-structure adjustments (E0499,
E0502) that change sequencing only.

No production code was changed and no accessor was added. For F-0006's two identity assertions,
`RuntimeEvent` exposes no `event_id` / `timestamp` accessor, but it already derives `Serialize` with
no renames and `serde_json` is already a declared dependency, so the serialized form is an existing
public observation path and rewriting those assertions to inspect it is mechanical. Had that path not
existed, the assertions would have counted as candidate defects.

#### Baseline correctness — does the suite accept the real implementation

All three: **63 tests, 0 failures** (19 / 20 / 24).

This must be read with one qualifier that cuts against Gemini. UPG-0071's V4-Pro F-0001 suite
*failed* baseline because it enforced the currently approved Event Schema against an implementation
that predates it — live specification drift, still present at `c3b8215`: the approved schema requires
`publisher_claims` as `array<object>` with `claim` and `discovery_evidence_refs`, while the emitted
`OfficialCandidateRecorded` carries `Vec<String>`. **Gemini's F-0001 suite never asserts the event
payload's `publisher_claims` shape at all.** Its clean baseline is therefore partly leniency rather
than fidelity, and on this specific point V4-Pro was more faithful to the approved specification.
The committed direct-path suite shares the same blind spot.

#### Defect discrimination — the nine frozen mutations

| Mutation | Feature | Gemini candidate | Committed direct suite |
|---|---|---|---|
| M1 accept empty `responsible_organization_evidence_refs` | F-0001 | KILLED | KILLED |
| M2 drop the `candidate_dataset_evidence_refs` guard | F-0001 | **KILLED** | **SURVIVES** |
| M3 country-scoped view returns every country | F-0001 | not applicable | not applicable |
| M4 report LIMITED as ACCEPT | F-0004 | KILLED | KILLED |
| M5 supported decision without its sparse-data threshold | F-0004 | KILLED | KILLED |
| M6 overwrite prior fields on revision | F-0004 | KILLED | KILLED |
| M7 `available` without ACCEPT/LIMITED fitness support | F-0006 | KILLED | KILLED |
| M8 skip the upstream approval check | F-0006 | KILLED | KILLED |
| M9 normalize away freshness / measurement-basis differences | F-0006 | KILLED | KILLED |

**8 of 8 applicable mutations rejected.** M3 stays inapplicable for the same reason UPG-0071 recorded:
the country filter lives in `list_official_candidates`, a trait method implemented by the
caller-supplied store, so there is no module-level injection point. It is a defect in the mutation,
not in either suite.

M2 reproduces UPG-0071's most durable finding independently: a contract-valid mutation that the
committed direct suite **survives** and the delegated suite **kills**. Mutation validity came from the
approved Contract, never from whether the direct suite died.

**Stage-5 test generation: YES** — 3 of 3 usable under the frozen protocol, against UPG-0066's
>=2 of 3 bar, with 8 of 8 applicable mutations killed.

### The uuid-v4 result — recorded separately from any role verdict

V4-Pro failed F-0004 and F-0006 semantically in UPG-0071 for one reason: their approved Event Schemas
state `"event_id": "uuid-v4"` and `"correlation_id": "uuid-v4"`, and the suites used readable ids
(`"event-corr-happy"`). Supplying conforming values would have been Claude supplying contract
conformance the delegate got wrong, so both features failed the repair boundary.

**Gemini produced uuid-v4-conforming ids in both suites** — `00000000-0000-4000-8000-{seq}` and
`11111111-1111-4111-8111-{nonce}`, correct version and variant nibbles. This is a real model-level
capability difference on an event-identity rule, and it is recorded here independently of the Stage-5
verdict so that a role-level outcome cannot erase it.

## Decision

```text
                         gemini-3.7-flash @ high
Reviewer fallback        NO
Stage-4 implementation   NO
Stage-5 test generation  YES
```

All three attributed to the tested configuration, never to provider or tier, and never pooled with
`deepseek-chat`, V4-Flash, V4-Pro, or Codex figures.

Against the permitted readings fixed before the run: **>=1 role clears, so at least one DeepSeek
negative does not generalize to `gemini-3.7-flash` @ `high`.** Nothing wider may be inferred — the run
varied both the model and the provider API, so this is not evidence about providers, tiers, or
delegation in general.

Two results are worth separating from the verdicts. The EA-0010 reproduction across three models is
the strongest evidence this line has produced that Stage-4 failure is systematic rather than
provider-specific. And the reviewer arm shows that operational quality and substantive safety are
independent: this is simultaneously the best-behaved and one of the least trustworthy reviewer arms
recorded.

**Consequences.** The required-reviewer policy stands unchanged — Codex remains the only accepted
reviewer, and `dba/02-policies/review/v2.md` needs no edit. The delegated implementer stays
`status: disabled`, and no default configuration moves. A Stage-5 YES is evidence that the role is
reachable, not authority to enable anything.

### Disposition of `CODEOS_LLM_PROVIDER`

One role passed, so the negative branch of the disposition rule does not fire and the Gemini path is
**not** removed. Per the rule fixed before the run, retention and adoption are a separate decision on
their own evidence: passing an arm does not by itself make the Gemini path permanent, and nothing
here promotes it to a supported extension point.

> **Disposition resolved (UPG-0073, 2026-08-23).** The separate adoption decision this brief
> deferred has been taken and is negative: on three features selected by rule rather than already
> exercised, Gemini produced 1 usable suite of 3 against direct Claude's 3 of 3, and Claude usage was
> not materially lower per usable result. The `CODEOS_LLM_PROVIDER` Gemini branch, its environment
> variables, and its tests have been removed, and the alternative-model delegation line is closed.
> This brief's own results stand as written and are not revised.
>
> One finding here is qualified rather than overturned: the uuid-v4 result below was real but not
> reliable. UPG-0073 saw the same configuration produce conforming ids for one feature and
> non-conforming ids for two others within a single run.

**The open question this run did not settle**, recorded rather than pursued: whether a Stage-5 suite
can be obtained that both passes baseline and enforces the drifted `publisher_claims` schema. Gemini
passed by not testing it; V4-Pro enforced it and failed. Answering that needs a new hypothesis and
its own human decision.
