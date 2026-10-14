# WO-04 — Decide the Next Structural Improvement

P4 · NORMAL · decision only · after WO-03 · Charter: Phases P4, Deferred

## Goal

Recommend one next structural improvement from the evidence. Nothing is implemented in this WO.

## Inputs

- The ECP process history.
- The WO-03 results and inconvenience log.
- The Charter's deferred items, each checked against its entry condition: batched human attention,
  reviewer throughput, and WO-07 automatic execution.
- The revalidation fan-out finding below.

## Finding: revalidation fan-out (2026-10-14)

**Hypothesis.** The gates now judge correctly. The identities and evidence ownership they work
over are too broad, so revalidation spreads to more features than the change touches, at
unnecessary cost. This probably limits throughput more than WO-07 automation does.

**Case.** On JamFree, two advisories against rustls forced a `Cargo.lock` patch upgrade (`874728d`).
That made all four accepted features stale. Each one then needs the full revalidation: fixture
journeys, production-source evidence and a Stage 8 assessment. 81 TomTom calls had already been
spent on evidence the upgrade invalidated. The F4 and F8 receipts stayed current (WO-02).
- F-0001 and F-0002 rightly need re-observation, because rustls carries their TomTom HTTPS
  calls.
- F-0003 and F-0004 declare their reality claims "inherited unchanged" from F-0001 and F-0002,
  creating "no observation of their own". Even so, each reruns the identical 27-call journey.

Completing these revalidations shows only that the current rules can recover the features. It
does not show that four full recovery paths were necessary.

**Questions for P4.**
- **A. Verification ordering.** Can every cheap check that can invalidate a run, including
  pre-candidate qualification, happen before scarce or expensive evidence is generated? Today
  `check --revalidation` produces production evidence before qualification runs.
- **B. Dependency-scoped invalidation.** Can a feature's inputs represent what it actually depends
  on? Today `candidate::derive_inputs` takes every file changed since the Intent was approved, so
  F-0001's bundle contains F-0004's `ReturnSummary.svelte`. An acceptance with no bundle compares
  the whole `implementation_state`, test files included.
- **C. Shared or inherited evidence.** Can a downstream feature bind the authoritative upstream
  reality evidence instead of reproducing it?

## Finding: one policy, two disagreeing implementations (2026-10-14)

The policy defined revalidation semantics, but no end-to-end run had ever covered a
reality-dependent feature. The workflow's candidate preparation and the reviewer's readiness
check each encoded their own assumption about F6, and the two were incompatible. Feature
Workflow v8 says revalidation creates no F6 receipt. Yet the revalidation candidate selected
F6's historical production evidence, and readiness required F6 to be bound to the current
implementation. So no revalidation of a reality-dependent feature could ever be assessed.

The repair restored conformance and added a test that crosses the workflow–reviewer boundary
(`reviewer/engine/tests/revalidation_readiness.rs`). The general weakness is lifecycle semantics
duplicated across components, with no end-to-end conformance test. Here the policy was correct
and its independent implementations disagreed.

**Question D for P4.** Which lifecycle rules does more than one component encode, and which of
those have no cross-boundary test?

## Finding: escaped verification defect, and a test-only change that costs provider evidence (2026-10-14)

**Escaped defect.** F-0002 was accepted on 2026-10-08 with incomplete browser evidence. Its
Contract C-5 requires browser checks for every distinct return-failure outcome, but four of the five
were checked only through the API. That acceptance's review did not catch the gap. The
revalidation Stage 8 assessment did, and the gap was repaired with a test-only change (JamFree
`b713734`). Use this case when judging whether Stage 8 is becoming more effective.

**Cost.** The repair changed no product code. It still moved `implementation_state`, because test
files under `web/` are implementation paths. So F-0002 needed a fresh 27-call production-source
run. This is another instance of question B: production evidence binds an identity broader than
what it observed.

**Recovery-cycle snapshot (ended 2026-10-14).** 4 of 4 JamFree features VALID. 216 TomTom calls
that day. Two CodeOS conformance repairs (`885172b`, `8ed6d67`). One escaped product verification
defect. One test-only change that forced provider-evidence regeneration. P3 is judged against a
fresh baseline, not this history.

## Acceptance criteria

The human accepts or rejects one recommendation. An accepted recommendation becomes its own
charter or WO.

## Results
