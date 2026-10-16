# WO-04 — JamFree Replay

NORMAL · after WO-03 · Charter: Measure

## Goal

Show on real history that only necessary invalidations remain.

## Method

Replay each WO-01 case on JamFree in fixture mode, with no TomTom calls. For each one, compare the
stale features and receipts under the new rules with the WO-01 classification.

## Acceptance criteria

Incidental invalidations: zero. Necessary invalidations: all still present. The human accepts the
replay.

## Results

**Human decision (2026-10-16): not accepted as first replayed.** The two residuals below fail
the acceptance criterion and are repaired, not waived:
- **F-0002 inheritance** is a conformance defect against v11's "the upstream's current selected
  evidence". Inheritance now resolves the bundle behind the upstream's current validity: the
  accepted bundle while it is current, otherwise the passed revalidation bundle. That covers an
  acceptance made before bundles. v11 item 2 now says so.
- **`71adabb`** is a protected amendment to v11. Evidence Scope state and v11 F6 bind the new
  `delivered_state`, which also excludes test-runner configuration. The shared test-file rule, the
  F4/F8 `product_state` and Candidate Bundle roles are unchanged. The impact trace showed that
  `hashing::product_state` also feeds F4/F8 receipts from DBA-22 on, so changing it would have
  staled every existing JamFree F4/F8 receipt. The glossary gains **Delivered state**.

B4 was re-recorded `direction_confirmed` (JamFree `043536d`); Bootstrap is 5/5.

**Rerun after the repairs.** Every case was replayed again on JamFree:
- `71adabb` now changes no binding;
- F-0002 resolves to its revalidation bundle `df002430` and evidence `2a14128f`;
- all other rows are unchanged.

New tests:
- `a_test_runner_configuration_change_leaves_evidence_and_f6_current`. It also asserts that F4/F8
  `product_state` still changes and that `vite.config.ts` still stales.
- `an_inherited_claim_follows_the_upstream_revalidation_evidence`, for both a bundled and a
  pre-bundle acceptance. The pre-bundle case fails without the repair.

The workflow (166) and reviewer (279) suites and `dba/04-tools/tests/run.sh` pass. Awaiting human
acceptance.

**First replay (2026-10-16).** JamFree adopts DBA-27 in `f94f3b6`. No TomTom
calls were made.

**Adoption checks.**
- DBA-27 resolves, and the configuration contract passes.
- The scopes `backend: [backend]` and `web: [web]` resolve deterministically: two runs on the live
  tree and one on a fresh worktree give the same states.
- Against a mock backend, the verifier emits schema 2 with only declared `claim_scopes` and no
  `implementation_sha256`. It refuses to run when a scope it needs is undeclared, and its schema-1
  output is unchanged.
- The verifier imports only Node built-ins, `@playwright/test` and `devalue`. All claim logic is in
  the one hashed file, so the WO-03 guard does not trigger.

**Method.** A scratch program calls the workflow library and computes the v11 bindings
(`evidence_scope_state` for the backend and backend+web scopes, the F6 product state, and the
verifier hash). It runs on a worktree before and after each WO-01 commit, with the DBA-27 `codeos.yaml`
overlaid. Inherited claims are checked by resolving F-0003, F-0004 and F-0005 with an in-memory
inherited claim against the real records. Nothing is written.

| WO-01 row | Under v11 | Verdict |
|---|---|---|
| 1a, 1d accepted currency; 1b F4 | Rules unchanged | Necessary invalidations remain |
| 1c `e9c9518` page change | `backend` unchanged: F-0001, F-0002 and F-0004 evidence stays current. `backend+web` changes: F-0003's browser journey observed the page | Incidental removed for three features. F-0003's is now necessary, since its evidence observes `web` |
| 2a rustls `874728d` | `backend` changes | **Still stale, as required** |
| 2b rustls → F-0003, F-0004 | Their own evidence observes `backend` | Remains until their Contracts are revised to `inherited` (decision b) |
| 3a `b713734`, 3b `2ac0fe0` test-only | No scope or F6 change | Incidental removed |
| 3c `71adabb` test-only | `web` scope and F6 change | **Residual incidental:** `web/playwright.config.ts` is not a test path under the shared test-path rule |
| 4 inherited | From F-0001: current, with F-0001's selected evidence and no run of its own. From F-0003 (not VALID) and from a non-reality claim: fails closed. From F-0002: fails closed | Works for bundled upstreams. **Over-invalidation:** F-0002's 2026-10-08 acceptance has no Candidate Bundle, and inheritance does not follow its passed revalidation bundle, which keeps it VALID |
| 5 verifier `3818ff6` | Verifier hash changes | Evidence stale and regenerable, as required |
| 6 input breadth | Deferred | — |

**Adoption cost.** Bootstrap B3 binds `codeos.yaml` and was re-verified in fixture mode. B4, the
Initial Product Preview, is a human decision with the same binding and needs re-recording. The F9
project-qualification health rows for F-0001 and F-0005 are stale for the same reason, until the
next F9.

**Open for the human.** Acceptance asks for zero incidental invalidations. Two remain:
- row 3c, from the test-path rule, which F4 and bundle inputs share;
- the F-0002 inheritance gap.

Both are over-invalidation, not under-invalidation.
