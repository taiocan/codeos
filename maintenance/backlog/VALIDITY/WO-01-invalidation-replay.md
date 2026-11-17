# WO-01 — Historical Invalidation Replay

NORMAL · analysis only · approved by the human 2026-11-17

## Goal

Measure which invalidations in the window were semantically unnecessary, so WO-02 designs only what
the data justifies. No TomTom calls, no JamFree record changes.

## Work

Read-only replay of JamFree 2026-11-13 → 11-17 (`.codeos/06-workflow/decisions.jsonl`,
`verifications.jsonl`, `candidates/`, the review log). For every re-run or re-decision of F4, F6,
F7, F8, F9 and Stage 8:

1. Classify it by the **actual inputs and bindings the old rule used**, and which of them changed
   (for example `delivered_state`, a record identity in a Candidate Bundle, `observed_surface`,
   `contract`, the DBA configuration).
2. Separately, state whether each changed input was semantically relevant to that decision.

The replay does not judge history with the new design. WO-02 turns its finding into the identity
model.

## Acceptance criteria

1. A per-checkpoint table names the responsible binding and the relevance verdict for every case.
2. Counts reproduce from one committed read-only command.
3. Every invalidation is classified, or carries a stated reason why it cannot be.

## Results (accepted 2026-11-17)

Command: `python3 -I maintenance/backlog/VALIDITY/wo01-replay.py <JamFree> 80fb5ec [--rows]`. It reads
only `git show 80fb5ec:<path>` and runs in about 10 s. The counts reproduce the analysis: F4 17, F6
38, Stage 8 36, F8 21, F9 18 (130 cases).

**Step 1, the old rule.** Almost every verification record in a changed Candidate Bundle changed
through `implementation_state`, the whole-project hash, and `execution_identity`. F6 binds the
whole-repository `delivered_state`. One case changed only record timestamps (F-0001, after the
2ae639e fix). JamFree entered the window on DBA-30 and adopted four DBA versions in it, not three (Git history of
`codeos.yaml`): DBA-31 (e5d710b, 11-13), DBA-32 (d713eaa, 11-15), DBA-33 (5a84dec, 11-16) and
DBA-34 (7a01ff4, 11-16). DBA-30 was adopted before the window. Four F6 cases show DBA-29 → 30
because they were the first re-decision after a decision recorded under DBA-29.

**Step 2, relevance.** Each case gets one verdict from these rules, applied in order:

- *not-relevant*: a re-run record with identical content; a spec hash change without a
  `semantic_revision` change; an `execution_identity`-only change, caused by CodeOS; a repeat with
  identical bindings; or only test, evidence, script or other features' spec files changed.
- *relevant*: the feature's own semantic revision changed, its judged `observed_surface` changed,
  or a product-source, governed, own-spec or declared-dependency file changed.
- *runtime-only*: only build or runtime-dependency files changed (`web/Dockerfile`, package locks).
  The product needs re-verification, but no product meaning changed.
- *per-DBA*: only the DBA configuration changed.

"Relevant" is an upper bound. Any product-source change counts for every feature, and any change to
a shared screen counts for every feature that judged it.

| Verdict | F4 | F6 | Stage 8 | F8 | F9 |
|---|---|---|---|---|---|
| relevant | 15 | 19 | 16 | 11 | 10 |
| runtime-only | 0 | 10 | 6 | 4 | 4 |
| not-relevant | 2 | 3 | 11 | 6 | 4 |
| per-DBA | 0 | 6 | 2 | 0 | 0 |
| unresolved | 0 | 0 | 1 | 0 | 0 |

The *not-relevant* cases split into these classes:

- spec hash changed without a semantic revision (YAML quoting and Reality Basis row corrections):
  2 F4, 2 F6, 4 Stage 8, 2 F8, 2 F9;
- only another feature's spec, or tests and evidence, changed: 4 Stage 8, 1 F8, 1 F9;
- `execution_identity` only (CodeOS defects 062f6b2 and 2ae639e): 2 Stage 8, 2 F8, 1 F9;
- record timestamp only: 1 Stage 8, 1 F8;
- repeat with identical bindings: 1 F6.

The *runtime-only* cases are two dependency changes, the SvelteKit security upgrade (34540a9) and the
npm removal (783efc9): 10 F6, 6 Stage 8, 4 F8, 4 F9.

The *per-DBA* cases, judged per DBA:

- DBA-30 → 31 changed nothing F6 judges (4 F6): not relevant.
- DBA-32 → 33 changed the Stage 8 packet. The 2 F6 cases are not relevant. The 2 Stage 8 cases are
  relevant to Stage 8, and both found gaps.

The one *unresolved* case is F-0010's Stage 8 at 2026-11-17T05:34. No project file changed; its
qualification record gained the dependency-addition bindings from CodeOS 027894b and 062f6b2. That
makes it CodeOS-induced, like the `execution_identity` class.

**Findings.**

1. **Human decisions:** 20 of 56 F4/F8/F9 decisions (36 %) followed no relevant change. That is 12
   not-relevant and 8 runtime-only. **Stage 8:** 17 of 36 reviews; 18 with the CodeOS-induced
   unresolved case and 20 with the per-DBA cases (47–56 %). **F6:** 13–19 of 38 (34–50 %). These
   measured upper bounds, about 35–55 %, are the authoritative baseline. The earlier 60–70 % figure
   is a superseded hypothesis.
2. **The largest single lever is the spec hash without a semantic revision:** 12 cases across all
   five checkpoints. A non-semantic edit to the Specification Package re-asks every gate.
3. **Dependency-only changes are the second lever:** 24 cases. WO-02 must decide whether a runtime
   change needs only re-verification (F3/F5 and qualification) or also new human and Stage 8
   judgments.
4. **CodeOS-induced invalidations:** 8 cases (`execution_identity`, timestamp, new binding keys).
   Content identity (WO-02) removes this class.
5. **Narrowing Stage 8 has a measured cost.** 5 of the 10 Stage 8 findings in the window (9 CHANGES ADVISED, 1 DO NOT
   ADVANCE) came from
   reviews whose trigger was not relevant to the feature:
   - DBA-33 packet: the architecture describes the previous refusal threshold; the return
     presentation lacks its authoritative comparison;
   - DBA-34: human readability evidence not inspectable; post-display endpoint preservation not
     verified;
   - F-0010's privacy prerequisite not established.

   Two came from a reviewer-contract change, which is relevant to Stage 8 by design (WO-04). The
   other three are latent gaps that earlier reviews missed. WO-02 must not remove those catches
   silently. It has to state that a review is skipped when the feature's dependency set is
   unchanged, and accept that latent gaps then surface only on a relevant change.
6. **Shared surface:** F-0009 and F-0010 judge the shared return screen, so 4 of the 8 F4 surface
   cases came from F-0001/F-0002 changes. Mechanically these count as relevant; a feature-scoped
   surface remains WO-02's open question, as in FAI WO-01 row 1b.
