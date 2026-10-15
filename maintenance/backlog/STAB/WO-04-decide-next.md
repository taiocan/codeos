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
- Candidates the human has already proposed: the [FAI charter](../FAI/charter.md) and the
  [SPERF charter](../SPERF/charter.md) (2026-10-15).

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

**Journey order from history (2026-10-15).** Two implementations rebuilt the accepted product
journey from the order of acceptance receipts. A legitimate re-acceptance of F-0001 put its
receipt after F-0002's, the valid chain became unreadable, and every new feature failed at F2.
`f5a8f9a` repaired this. The duplication let the defect spread. The deeper cause was using
append-order history to reconstruct semantic journey topology. When P4 chooses a structural
improvement, ask which other semantic structures are rebuilt from log order.

**Preserve feature behind a stale frontier (2026-10-15).** F-0005 adds no journey step, so it
targets the current accepted frontier. Its implementation changed `+page.svelte`, which staled
F-0001 to F-0004. The accepted frontier then fell back to `entry`, and F-0005's F3 journey ran only
`@frontier:entry`. An F4 or F8 receipt would have bound an Observed Surface without the window,
silently and without failing. JamFree avoids this by revalidating the four upstream features
(about 108 TomTom calls and four assessments) before F-0005's governed journey and F4. The
immediate cause is understood. The right architectural answer is not yet known, and the cost of
the workaround is evidence that this needs deeper analysis in P4.
The same change also staled the F4 receipts of F-0001 to F-0004.
Those receipts predate Feature Workflow v10, so they still bind the project-wide `product_state`
(F-0003 binds `implementation_state`). Revalidation then needs four new F4 decisions, taken one
feature at a time in journey order. A preserve feature's F3 journey must also be rerun after its
upstream frontier is restored, or its new receipt binds the `entry` surface. Status does mark that
journey stale once the frontier moves; it was F-0001's restoration that marked F-0005's.

**A related projection limitation (2026-10-15).** After revalidation, a current passing project
qualification can exist at the candidate stage. Even so, the ECP engineering-health rows (Project
quality, Security, Supply chain, Deployment) stay STALE, because they show only the latest
qualification recorded at an ordinary acceptance (ECP WO-01). Revalidation records none (Feature
Workflow v8). On JamFree, DBA-26 adoption staled F-0001's acceptance-time record, while nine
candidate-stage qualifications passed against the current configuration. This is conservative and
never a false PASS, but it hides current qualification. The first normal P3 acceptance should clear
it. Not repaired during the P3 freeze.

## Finding: escaped verification defect, and a test-only change that costs provider evidence (2026-10-14)

**Escaped defect.** F-0002 was accepted on 2026-10-08 with incomplete browser evidence. Its
Contract C-5 requires browser checks for every distinct return-failure outcome, but four of the five
were checked only through the API. That acceptance's review did not catch the gap. The
revalidation Stage 8 assessment did, and the gap was repaired with a test-only change (JamFree
`b713734`). Use this case when judging whether Stage 8 is becoming more effective.
A second gap surfaced on 2026-10-15. F-0002's contract requires a persistence inspection after a
completed round trip, and no such evidence ever existed. The revalidation assessment of candidate
`abd82325` was BLOCKED for this reason. The guarantee itself holds: the database has no domain
tables.

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
