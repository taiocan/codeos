---
feature_id: UPG-0072
slug: gemini-frontier-qualification
title: Gemini 3.7 Flash Qualification Across Three Codeos Roles
status: PROPOSED — controls frozen 2026-08-23, not yet run
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

## Results

Not yet run.
