# WO-01 — Root-Cause Table of P3 Invalidations

NORMAL · analysis only · Charter: Problem, Measure

## Goal

Classify every invalidation from the evidence already collected, so that WO-02 designs only what
the data justifies. No TomTom calls and no CodeOS changes.

## Cases

1. **F-0005 page change** (`e9c9518`): F-0001 to F-0004 stale; four F4 receipts stale.
2. **rustls patch** (`874728d`): four accepted features stale.
3. **F-0002 test-only repair** (`b713734`): F-0002 production evidence regenerated.
4. **Inherited production evidence:** F-0003 and F-0004 rerun the F-0001 and F-0002 journeys.
5. **Evidence that cannot be refreshed:** F-0003's production evidence was current but insufficient,
   and it does not bind its verifier, so the improved verifier (`3818ff6`) could not replace it
   (STAB WO-04).

## Method

Write one row per invalidated feature and evidence item, with these columns:

- the identity that caused it (an input file, `implementation_state` or `product_state`);
- whether the judged surface or dependency actually changed;
- **necessary** or **incidental**;
- the narrowest binding that would have been correct;
- the cost incurred (TomTom calls, assessments, human decisions).

Sources: JamFree `verifications.jsonl`, `decisions.jsonl`, the candidate bundles, Git, and the STAB
WO-04 findings.

## Acceptance criteria

The human accepts the table. Each incidental row names the goal (1 to 4) that would remove it.

## Results

**Accepted (2026-10-16).** The human accepts the classification and root-cause table. The cost
column is indicative only: because of the accounting discrepancy (finding 7), it is not
authoritative quantitative evidence.

**Table (2026-10-16).** The sources are JamFree's `verifications.jsonl`, `decisions.jsonl`, the
candidate bundles, and Git up to `d30545c`.

**Cost unit.** A run is one production-evidence record, a journey of about 27 TomTom calls (STAB
WO-04). The evidence files do not record per-run call counts. The accounted day totals stay as
recorded:
- 2026-10-14: 216 calls, which matches 8 runs;
- 2026-10-15: 162 measured calls plus an unknown incident amount.

**Rule behind each row.** Accepted currency compares every non-test file in the feature's Candidate
Bundle (`candidate::acceptance_comparisons`). Production evidence and F6 bind the whole
`implementation_state`, test files included.

| # | Case → feature, evidence | Identity that caused it | Judged surface changed? | Class | Narrowest correct binding | Cost (goal) |
|---|---|---|---|---|---|---|
| 1a | `e9c9518` → F-0001–F-0004 accepted currency | `web/src/routes/+page.svelte` in every bundle | Yes. The page each journey crosses changed (2 lines) | Necessary (re-verify) | Unchanged: the page is a real shared input | Fixture journeys only |
| 1b | `e9c9518` → F-0001–F-0004 F4 receipts | Legacy `product_state` | Yes. The Observed Surface changed: F-0001 `992e9f97` → `e322ccca`, F-0002 `1469e1ce` → `bd685607`, because the window renders on the shared page | Necessary under the whole-frontier Observed Surface | Unchanged unless the surface becomes feature-scoped (open question for WO-02) | 4 human F4 decisions, in journey order |
| 1c | `e9c9518` → F-0001–F-0004 production evidence | `implementation_state` | No. The window is client-only and sends no request (`departure-window.e2e.ts`) | Incidental | The provider path the claims observe | F-0001 1 run, F-0002 1 run (plus case 3 below), F-0003 and F-0004 1 run each (case 4) (goal 2) |
| 1d | `e9c9518` → four revalidation assessments | Revalidation follows stale acceptance | Page yes, claims no | Necessary: the page change merits review, and 2 of the 5 assessments found real gaps | Unchanged | 5 assessments: F-0001 and F-0004 NO OBJECTION; F-0002 BLOCKED, then NO OBJECTION; F-0003 BLOCKED |
| 2a | rustls `874728d` → F-0001, F-0002 production evidence | `implementation_state` (`Cargo.lock`) | Yes. rustls carries their TomTom HTTPS calls | **Necessary — must remain** | A declared runtime dependency of the provider path | F-0001 1 run, F-0002 1 run |
| 2b | rustls `874728d` → F-0003, F-0004 production evidence | `implementation_state` | Only through inherited claims | Incidental | The upstream F-0001 and F-0002 evidence (goal 4) | 2 runs |
| 2c | Runs made just before `874728d` (2026-10-14 15:04Z) | — | — | Ordering, not binding (STAB WO-04 question A) | — | 3 runs spent, then invalidated |
| 3a | Test-only `b713734` → F-0002 production evidence | `implementation_state` includes `page.e2e.ts` | No | Incidental | Non-test implementation, or the provider path | 1 run (goal 2) |
| 3b | Test-only `2ac0fe0` → F-0002 production evidence | Same | No | Incidental | Same | 1 run (goal 2) |
| 3c | Test-only `71adabb` → F-0005 production evidence and F6 receipt | Same; F6 also binds `implementation_state` | No | Incidental | Same, for evidence and F6 | 1 run and 1 human F6 decision (goal 2) |
| 4 | F-0003, F-0004 production evidence, at every revalidation | Each claim is evidenced separately | Inherited claims (`outbound-summary`, `return-summary`) restate upstream provider data | Incidental | The authoritative upstream evidence | 2 runs per cycle (2b, 1c) (goal 4) |
| 5 | F-0003 production evidence, current but insufficient (`3818ff6`) | The verifier `scripts/verify-production-claims.mjs` is not bound | Yes. The verifier gained a rendered-summary observation | **Missing invalidation**, the opposite of incidental | The evidence should also bind the verifier that produced it | F-0003 stays STALE, with no governed refresh |
| 6 | Latent: F-0001's revalidated bundle `8bf4b720` includes F-0005's `DepartureWindow.svelte` | `derive_inputs` takes every file changed since Intent approval | — | Incidental, not yet incurred | The feature's own and declared-dependency files (goal 1) | The next window change would stale F-0001 |

**Findings.**
1. **Goal 2 (feature-scoped evidence bindings) removes the most cost.** It covers rows 1c, 3a, 3b
   and 3c: 6 production-evidence runs and 1 human decision.
2. **Goal 4 (inherited evidence) removes 2 runs per revalidation cycle** (rows 2b and 4).
3. **Goal 1 (input scope) has no incurred cost in P3**, only the latent row 6. The page is a real
   shared input (row 1a).
4. **Goal 3 (dependency propagation) is needed for row 2a.** A narrower provider-path binding must
   still see `Cargo.lock` and rustls.
5. **Row 1b does not support the Charter's target** "F-0001 UX stale only if its Observed Surface
   changed". F-0001's surface did change, because the Observed Surface is the whole frontier page.
   Making F4 incidental here would need a feature-scoped Observed Surface. That is a semantic
   change to F4, for WO-02 to decide or drop.
6. **Row 5 asks for a broader binding.** Production evidence should also bind its verifier.
7. **Accounting discrepancy, unresolved.** There are 7 production-evidence records for 2026-10-15,
   but the 162 measured calls equal 6 runs of 27. The accounting is left unchanged.
