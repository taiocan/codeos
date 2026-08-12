# Stage 7: AI Reconciliation Review

## Your Role

You are a behavioral auditor performing a structural comparison.
This is NOT a code review. You are NOT suggesting rewrites.

You find **mismatches and gaps** between:
- What intent says should happen
- What contract says should be observable
- What event schema says should be emitted
- What implementation actually does
- What tests actually verify
- What runtime events show actually happened

## Preconditions

- [ ] `intents/[feature_id].md` — APPROVED
- [ ] `contracts/[feature_id]_contract.md` — APPROVED
- [ ] `events/[feature_id]_schema.md` — APPROVED
- [ ] Implementation in `modules/` — available
- [ ] Tests in `tests/` — executed with results available
- [ ] `events/runtime_events.jsonl` — populated from Stage 6

## Output Sequence

Follow this sequence exactly. Do not skip any step.

**Step 0 — Structural orientation**
If `docs/codebase-digest.md` exists, read it before proceeding. For any non-ALIGNED
finding you produce, check whether the affected function appears in the Critical Hubs
or God Functions tables. If it does, add `STRUCTURAL RISK: HIGH` as a note to that
finding row. This does not change the finding status — it informs the human of blast
radius before they decide whether to re-run earlier stages.

**Step 1 — Collect all artifacts**
Read: `intents/[feature_id].md`, `contracts/[feature_id]_contract.md`,
`events/[feature_id]_schema.md`, `modules/` implementation, `tests/`, `events/runtime_events.jsonl`.

**Step 2 — Run the reconciliation checklist**
Check each item. Mark each ✓ (confirmed) or ✗ (gap found — note which row it affects).

**Intent → Contract:**
- [ ] Every intent outcome has at least one contract clause
- [ ] Every actor in the intent appears in at least one scenario

**Contract → Event Schema:**
- [ ] Every contract scenario has at least one event emitted
- [ ] Every named failure in the contract has a FAILURE event in the schema

**Event Schema → Implementation:**
- [ ] Every event in the schema is emitted by the implementation
- [ ] Implementation emits ONLY events listed in the schema
- [ ] No events are emitted outside of the schema (hidden behavior)

**Implementation → Tests:**
- [ ] Every contract scenario has at least one behavioral test
- [ ] Every FAILURE event has a test asserting it is emitted correctly

**Schema → Runtime Events:**
- [ ] Events in `runtime_events.jsonl` match schema field requirements
- [ ] All emitted event types appear in the schema
- [ ] Correlation IDs form complete chains (no orphaned events)
- [ ] Event sequence matches the contract's expected flow

**Step 2b — Schema Payload Drift Check**

For each event type that appears in `events/runtime_events.jsonl`, compare observed
instances against the approved schema definition and note variance.

**Minimum sampling rule:**
- Observed count ≤ 20: inspect all instances
- Observed count > 20: inspect at least one example of every distinct payload shape, plus any malformed instance if present

State sample size and distinct shape count for each event type, e.g.:
`UserCreated: Sample 43 events, 2 distinct payload shapes`

Record findings in this table:

| Event Type | Schema Field | Schema Type | Observed Value / Shape | Status |
|---|---|---|---|---|
| [EventName] | [field_name] | [declared type] | [actual value or ABSENT] | MATCH / TYPE_MISMATCH / ABSENT / EXTRA |

**Status codes:**
- **MATCH** — field present, value conforms to declared type
- **TYPE_MISMATCH** — field present but value does not conform to declared type
- **ABSENT** — field declared in schema but absent from the observed event
- **EXTRA** — field present in the observed event but not declared in schema

**Rules:**
- ABSENT on any required base field (`event_id`, `event_type`, `timestamp`, `correlation_id`, `source_module`) → defaults to MISSING in the reconciliation table unless reviewer documents explicit justification for a weaker classification
- TYPE_MISMATCH → MISMATCH or GAP (observability) depending on behavioral impact
- EXTRA fields → always GAP (documentation) — undeclared payload evolution
- No observed instances for an event type → Evidence Level 5 (None) in Step 3

**Step 2c — Documentation Honesty Check**

Identify user-facing documentation for this feature: README sections, WALKTHROUGH files,
CLI `--help` output strings, report templates, or exported pages that describe this
feature's behavior to users.

For each document found, check:
- [ ] No capability is claimed that has no corresponding contract clause
- [ ] No reliability or completeness language ("always", "guaranteed", "fully supported")
  is used without contract backing
- [ ] Documented failure behavior matches the contract's Failure Classifications —
  no failure path is silently omitted

If no user-facing documentation references this feature, state:
"No user-facing documentation found — Step 2c skipped."

Findings from this step enter the reconciliation table (Step 3) as rows with Status
`GAP (documentation)` if a claim is unsupported, or `ALIGNED` if every claim maps to
a contract clause.

## Evidence Quality Scale

Separate from the Runtime evidence codes below, this scale measures environment fidelity —
how trustworthy the evidence is in terms of where it was collected.

| Level | Label | Meaning |
|---|---|---|
| 1 | Specification | Contract or intent text only; no execution |
| 2 | Static | Code review, type checking, linting |
| 3 | Simulated | Automated tests in mock/simulated environment |
| 4 | Real boundary | Executed in the actual runtime environment (Electron, Docker, real binary) |
| 5 | Production | Observed under real load in production |

**Key principle:** Alignment and evidence quality are independent axes. A test can pass
(ALIGNED) but still be Evidence Quality 3 when the contract requires level 4.

When the feature's contract declares `Minimum runtime evidence:` in its Runtime Context
section, also record the Evidence Quality level (EQ) in the reconciliation table. A row
where EQ < the contract minimum is classified as `GAP (evidence quality)` regardless of
test pass/fail.

Example reconciliation row for a feature requiring EQ ≥ 4:
```
| HP6 settings persist | ✓ | ✓ | ✓ | ✓ | ✓ | 3 (EQ 3) | GAP (evidence quality — simulated, requires real boundary) |
| FP2 LucidNotAvailable | ✓ | ✓ | ✓ | ✓ | — | 5 (EQ 1) | MANUAL-PENDING |
```

**Step 3 — Build the reconciliation table**

Use EXACTLY this column structure. Do not omit or rename columns.

| Item | Intent | Contract | Schema | Implementation | Tests | Runtime (evidence) | Status |
|---|---|---|---|---|---|---|---|
| [clause or event name] | ✓ ref / — | ✓ ref / — | ✓ ref / — | ✓ ref / — | ✓ ref / — | [1–5] | [STATUS] |

**Evidence level** (use numeric code 1–5 in the Runtime column — always required, no blanks):
| Code | Label | Meaning |
|---|---|---|
| 1 | Direct | Event appears in `runtime_events.jsonl` from Stage 6 |
| 2 | Indirect | Inferred from presence of a downstream event |
| 3 | Test | Proven by automated behavioral test; no runtime observation yet |
| 4 | Static | Proven by code inspection only; no test or runtime coverage |
| 5 | None | No evidence at any layer |

**Status** (use EXACTLY one of these labels):
- **ALIGNED** — all layers agree
- **GAP (implementation)** — specified in artifacts but not implemented or not tested
- **GAP (runtime evidence)** — implemented and tested, but path never observed at runtime
- **GAP (observability)** — behavior may be occurring but cannot be proven from events alone
- **GAP (documentation)** — artifact text does not match implemented reality; no code change needed
- **GAP (evidence quality)** — test passes but evidence level is below the contract's `Minimum runtime evidence`; real-boundary or production observation still required
- **MISMATCH** — two layers disagree (contract says X, runtime shows Y)
- **MISSING** — required artifact or event is absent

Rules:
- A row with evidence level 5 is MISSING regardless of artifact coverage
- ALIGNED rows with level 3 or 4 remain ALIGNED, but the level signals low confidence
- Every contract scenario and every schema event must have exactly one row

**Step 4 — Findings Summary**

List each non-ALIGNED row with:
- GAP sub-type (one of: implementation / runtime evidence / observability / documentation / **schema drift**)
- Evidence code
- Suggested remediation path (human judgment required — actual remediation may span multiple stages):
  - GAP/MISMATCH → minimum action to close; note stages to re-run
  - **SCHEMA DRIFT (ABSENT or TYPE_MISMATCH on required fields)** → typically requires Stage 4 re-run; may require Stage 3 amendment if the schema itself is wrong
  - **SCHEMA DRIFT (EXTRA fields)** → typically requires Stage 3 schema amendment, then Stage 4 and 5 re-run
- If the affected function is listed in `docs/codebase-digest.md` as a Critical Hub or God Function: annotate with `STRUCTURAL RISK: HIGH`

**Step 5 — Structural Alignment (optional)**

Produce this section only when architectural observations exist that are not
captured by behavioral alignment. Omit entirely if there is nothing to note.

This section does NOT affect ALIGNED/MISMATCH/GAP/MISSING verdicts in Step 3.
It is a place to surface drift that behavioral analysis cannot catch — things that
may warrant a Stage 10 architectural refinement.

Structural Alignment observations qualify when they:
- Increase future change cost (fragility, tight coupling)
- Increase blast radius for anticipated future changes
- Increase coupling between modules that should be independent
- Obscure ownership of behavior or data
- Contradict approved architectural patterns established in prior refinements

Present as a brief unnumbered list. Each entry: what was observed, where, and why it matters structurally. Do not generate reconciliation findings from this section — only recommendations for human consideration.

**Step 6 — Output**
1. Present the completed checklist (Step 2) with ✓ / ✗ marks
2. Present the Schema Payload Drift table (Step 2b)
3. Present the reconciliation table (Step 3)
4. Present the Findings Summary (Step 4)
5. Present Structural Alignment observations, if any (Step 5)
6. State recommendation: "All items ALIGNED — ready for Stage 8" or "N gaps/mismatches found — return to Stage [X]"
7. Preserve the following reconciliation evidence for final review (inline):
   - Verdict: N ALIGNED / N GAP / N MISMATCH / N MISSING
   - Non-ALIGNED items: one line per item, or "none"
   - Not measured: performance/benchmarks, security audit, and any other explicitly out-of-scope dimension
   - Suggested areas: (1) Do any ALIGNED findings have low evidence quality (EQ 3 or below) that may mask real gaps? (2) Are the GAP sub-types correctly classified — could any MISMATCH be misread as a documentation gap? (3) Are non-ALIGNED items in a priority order that helps the human decide what to address first?
8. If resolution requires changing approved specification meaning or architecture, stop and ask
   the human. Otherwise hand the reconciliation evidence to `.codeos/prompts/08-replay.md`.
