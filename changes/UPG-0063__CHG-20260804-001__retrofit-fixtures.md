> ## ⚠ RETROSPECTIVE EXAMPLES — NOT HISTORICAL BACKFILL
>
> These three traces are **test fixtures**, written now, against implementations approved months ago.
> They exist to validate the five-field schema against real cases (AC-16) and for nothing else.
>
> They are **not** part of any downstream project's governance record. They were not present when
> those implementations were approved, and nothing here should be read as though they were. **No
> PlotSpot or EvidenceAtlas artifact is modified by this change** (AC-17). If backfilling real
> downstream artifacts is ever wanted, that is a separate decision taken on its own merits.

# UPG-0063 — Retrofit fixtures: the five-field trace against Q0's real cases

Source cases: `changes/UPG-0063__Q0-classification-evidence.md` §1a. Purpose: does the schema fit
without inventing a sixth field, and do `Final / Interim` and `Expected Superseder` do real work?

---

## Fixture 1 — PlotSpot F-0001, validation ordering

| Source Artifact + Deferral | Chosen Resolution | Where Implemented | Final / Interim | Expected Superseder |
|---|---|---|---|---|
| `events/F-0001_schema.md` — "Validation ordering is not prescribed." Order of the four failure checks, and behaviour when several apply at once, are left open | Sequential checks in the order selected-country → responsible-organization → candidate-identity → publisher-claim-evidence, returning on the **first** failure. A multiply-invalid input therefore surfaces exactly one signal, determined by this order | `modules/source_inventory/src/lib.rs:155-193` (`record_official_candidate`) | FINAL | — |

**Fits.** `FINAL` is right: the schema grants the freedom permanently, so nothing upstream is expected
to supersede it. Recording it still matters — a maintainer reordering these checks would silently
change which signal callers observe.

## Fixture 2 — PlotSpot F-0001/2/3, vocabulary ownership

| Source Artifact + Deferral | Chosen Resolution | Where Implemented | Final / Interim | Expected Superseder |
|---|---|---|---|---|
| `contracts/F-0001_contract.md:111` (and `F-0002:108`, `F-0003:106`) — "canonical ownership is unresolved until Architecture Synthesis." Who owns the canonical vocabulary, and where resolution lives, is left open | A hardcoded literal map local to the feature module, lowercase-matching eight known values and passing anything unrecognised through unchanged. Applied to `access_limitations` and `lifecycle_limitations` | `modules/source_inventory/src/lib.rs:382-396` (`canonicalize_representations`), called at `:208-209` | **INTERIM** | Architecture Synthesis's resolution of canonical vocabulary ownership for the source-intelligence cohort — the contracts name it explicitly |

**This is the fixture that justifies the two extra columns.** The resolution is interim by the
contract's own wording, and nothing in PlotSpot records that today. Written down, it becomes a
retirable obligation instead of a permanent-looking local map.

It also demonstrates the review value claimed for the mechanism: stating *"applied to
`access_limitations` and `lifecycle_limitations`"* immediately raises "why not `known_access_forms`,
which the contract also lists as a governed concept?" — the apparent defect filed separately at
`PlotSpot/refinements/F-0001-known-access-form-canonicalization.md`. Nothing currently asks that.

## Fixture 3 — EA-0001, validator semantics

| Source Artifact + Deferral | Chosen Resolution | Where Implemented | Final / Interim | Expected Superseder |
|---|---|---|---|---|
| `contracts/EA-0001-research_brief_contract.md:148,149,151` — three falsification rows marked "MANUAL-PENDING: validator semantics — orchestration only". Whether a submission is well-formed, and what "independently answerable" means, are left to a validator this contract does not specify | A `ResearchContractValidator` trait: the module implements orchestration (event shape, correlation propagation, lineage, combined-failure handling) fully and takes validator semantics as an injected dependency, deliberately not committing to a judgment mechanism | `modules/research_brief/src/lib.rs:65-74` (trait), consumed in `evaluate_submission` at `:165` | **INTERIM** | A decision on validator semantics — the contract marks these rows MANUAL-PENDING, implying an intended future resolution |

**Fits, with one honest wrinkle.** The expected superseder is less crisp than fixture 2's: the contract
says the rows are pending but does not name what will settle them. The field still earns its place —
"something is expected to settle this, and it has not" is more useful than silence — but it shows the
field can only be as precise as the deferral it points at.

---

## Assessment against AC-16

| Question | Answer |
|---|---|
| Expressible without inventing a sixth field? | **Yes**, all three |
| Do `Final / Interim` + `Expected Superseder` do real work? | **Yes** — decisively on fixture 2, usefully on fixture 3, correctly inert on fixture 1 |
| Any field unusable or missing? | **None.** One limitation reported, not accommodated: `Expected Superseder` is only as precise as the deferral (fixture 3). Not a schema defect — a property of the underlying artifact — and adding a field would not fix it |
| Is `Where Implemented` stable? | Line numbers drift. `file:function` is the durable part; the trace should be read as pointing at a function, not a line |

---

## Raw verification output

Embedded rather than summarised — a claim of `184 tests pass` in prose is not the same evidence
as the run that produced it (AJ-016).

### AC-9 — `cargo test` in `tools/reviewer/`
```
test packet::tests::stage_4_deferral_question_is_scoped_to_stage_4_only ... ok
test packet::tests::stage_4_checklist_asks_for_the_deferral_resolution_trace ... ok
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.85s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### AC-7 — STAGE-SPECIFIC CHECKS from a freshly generated stage-4 packet (post-rebuild)
```
STAGE-SPECIFIC CHECKS
  - code traces to approved contract/schema only; no unapproved events; no hidden behavior; no unrelated files; report complete.
  - did implementation resolve any question an approved artifact EXPLICITLY deferred? if so, is each material resolution recorded in a Deferral -> Resolution Trace (source deferral, resolution, where implemented, final/interim, expected superseder)? judge deferral by meaning, not by phrase; a missing record is a traceability finding, not an implementation failure.

```

### AC-15 — `dba-init.sh` scratch run
```
$ ls -l /tmp/dbainit-check/.codeos
lrwxrwxrwx 1 rimo rimo 26 Aug  4 12:06 /tmp/dbainit-check/.codeos -> /home/rimo/projects/Codeos
$ test -f .codeos/dba-system.md && grep -c 'Deferral → Resolution Trace' .codeos/prompts/04-implement.md
1
```

### AC-17 — downstream projects carry no artifact modification from this change
```
$ git -C PlotSpot status --porcelain -- modules/ contracts/ events/ intents/   # all predate this change
 M contracts/F-0003_contract.md
 M events/runtime_events.jsonl
?? modules/dataset_profile/
$ git -C EvidenceAtlas status --porcelain -- modules/ contracts/ events/ intents/   # all predate this change
 M contracts/EA-0001-research_brief_contract.md
 M contracts/EA-0002-research_planning_contract.md
 M contracts/EA-0003-corpus_construction_contract.md
 M contracts/EA-0007-investigation_review_contract.md
$ git -C PlotSpot status --porcelain -- refinements/   # the one file added, at human instruction, separate from this change
?? refinements/F-0001-known-access-form-canonicalization.md
```
