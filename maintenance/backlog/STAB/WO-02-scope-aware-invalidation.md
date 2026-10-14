# WO-02 — Scope-Aware Human-Decision Invalidation

P2 · PROTECTED · after WO-01 · Charter: Phases P2, Constraints

## Goal

A human receipt goes stale only when the surface it judged changes.

## Finding

`early_preview` binds `product_state`, which covers every non-test implementation file
(`hashing::product_state`). `reconciliation`, `final_ux_validation`, and `acceptance` bind the full
`implementation_state` (`checker::bindings_for_decision`). So an internal change that no user can
see, such as JamFree `0fd5954` in `provider.rs`, invalidates F4, F6, F8, and F9.

## Part A — Decision brief (before any code)

This part follows the CLAUDE.md PROTECTED statement and covers:

- **What each receipt judges and should bind.** For each of the four receipt checkpoints, the
  surface it judges and the binding that represents it.
- **How the surface is derived.** Mechanically, from canonical state, verification records, and
  receipts. No declared exemptions and no vouching.
- **What must remain true.** No receipt survives a change to what it judged. History is never
  rewritten. Semantics are versioned through the owning policy, not a new DBA configuration.
- **Verification and open decisions.**

The human approves or rejects Part A.

### Part A brief (2026-10-14)

**Existing authority.** Feature Workflow v8 already says that "product UX decisions are reused
when their feature-scoped product binding remains current". The binding used today,
`product_state`, covers every non-test file of the project, not one feature. So every change to any
feature makes the F4 and F8 receipts of every other feature stale. F-0001 needed 25
`early_preview` receipts. What "feature-scoped" means mechanically has never been decided; that is
the semantic change here.

**Judged surface and binding.**

| Receipt | What it judges | Binds today | Proposed |
|---|---|---|---|
| F4 `early_preview` | What a user can observe of the feature increment | Specification Package, `product_state` | Specification Package, `observed_surface` |
| F6 `reconciliation` | Whether the implementation and the selected production evidence conform to specification and architecture | Specification Package, `codeos_yaml`, `implementation_state`, `production_evidence` | Unchanged |
| F8 `final_ux_validation` | What a user can observe of the candidate | Specification Package, `codeos_yaml`, `candidate_bundle`, `product_state` | `product_state` replaced by `observed_surface` |
| F9 `acceptance` | The candidate as a whole | Specification Package, `codeos_yaml`, `candidate_bundle`, `implementation_state` | Unchanged |

Why F6 and F9 stay broad:
- F6 judges the implementation itself, so any implementation change can alter what it judged.
- F9 judges the whole candidate.
- After acceptance, v8 revalidation already restores current validity without new receipts.
- In WO-01, the only rows where the judged surface demonstrably changed were F6 and F9. The rows
  marked UNCERTAIN or NO EVIDENCE OF CHANGE are F4 and F8.

**Mechanical derivation.** `observed_surface` is the identity of what the feature's own browser
journey observes. A current passing journey verification records it, running in fixture provider
mode, and the receipt binds the identity that was current when the human decided. A receipt is
current when the latest current journey verification reproduces that identity.

When the implementation changes, the journey record goes stale first, so the checker projects an
AUTO re-run of the journey, and only then evaluates the receipt. An internal change that leaves the
observation identical keeps F4 and F8 current; this is the `0fd5954` case. A visible change makes
them stale. The evidence is a verification record (direct evidence). Nothing is declared exempt,
and no one vouches.

**What must remain true.**

1. **No false currency.** A receipt stays current only if its observed surface is reproduced.
   Missing or non-reproducible observation fails closed, as stale.
2. **History is never rewritten.** Receipts are immutable, and historical acceptance stands.
3. **Old receipts.** A receipt that bound `product_state` stays current while `product_state` is
   unchanged, the same compatibility rule as `implementation_state` covering `product_state`. That
   avoids a one-time mass invalidation.
4. **Candidate changes.** A Candidate Bundle change still invalidates F7 to F9.
5. **One evaluation for check and status**, using the required-binding rule from WO-01.
6. **Versioning.** The semantics are prospective: Feature Workflow v10, selected by DBA-26.
   Earlier configurations keep their meaning (decision 3).

**Verification.**
- The three Part B tests.
- A determinism test: the same tree observed twice gives the same identity.
- Replay of the WO-01 F4 and F8 rows. Run the F-0001 journey in fixture mode before and after each
  trigger commit; there are no TomTom calls. The only F4 and F8 receipts that become stale should
  be those whose observation changed.

**Decisions (human, 2026-10-14).** Part A approved as a whole with these decisions; DBA-26
is DBA-25 with only `workflow_feature_policy` changed to v10.

1. **Observation.** Normalized accessibility state plus normalized layout geometry of the
   elements the feature's governed browser journey actually observes or asserts, at the
   viewports that journey runs at. Viewports stay project-owned. Nothing beyond what the journey
   observes is inferred, and the observation must be deterministic in fixture mode.
2. **Coverage.** The binding is as wide as the feature's browser journey. A feature without a
   browser journey keeps binding `product_state`, so it fails closed as it does today.
3. **Versioning.** P2 is the Charter's planned semantic change, not a repair. It is introduced as
   Feature Workflow v10 and one new configuration, DBA-26; DBA-25's meaning is unchanged. Projects
   adopt DBA-26 explicitly. The immutable configurations pin `workflow-feature/v9.md`, so v10
   cannot reach a project any other way.
4. **Terminology.** *Observed surface*: the identity of what a feature's browser verification
   observes of the running product, in deterministic provider mode. It is added to
   `dba/05-guidance/terminology.md` in Part B.

## Part B — Implementation (after approval)

The approved semantics enter the owning policy first. Tests show that:

- an internal change keeps F4 current;
- a user-visible change makes F4 stale;
- a Candidate Bundle change still invalidates F7 to F9 as it does today.

## Acceptance criteria

1. Part A is approved and recorded in the owning policy.
2. The Part B tests pass.
3. Replaying the WO-01 table under the new rules produces only stale receipts whose judged surface
   changed.

## Results

### WO-01 replay (2026-10-14)

**Method.**
- Each historical JamFree product state was built in fixture mode, with no TomTom key in the stack
  environment.
- The state was matched to its F4 receipt by recomputing `product_state`.
- One fixed observation journey ran against every state. It performs F-0001's outbound
  submission and records the elements the current journey asserts. The same Playwright runner and
  browser were used throughout.
- Identities come from a verbatim copy of the engine's normalization. Every state gave identical
  identities in two runs at each viewport.
- The journey viewport is 1280×720, the only one JamFree's journey runs at. 390×844 is a
  diagnostic run.

| F4 receipt | Before → after | 1280×720 | 390×844 | What changed | WO-01 assessment |
|---|---|---|---|---|---|
| 10-07 19:05Z | `de6872f`* → `d3f06b1` | changed | changed | F-0003 outbound summary appears; result panels shift down | UNCERTAIN |
| 10-09 09:44Z | `d3f06b1` → `5c1fa66` | same | changed | Phone: result and chart panels narrowed to the viewport (`fdbe263`) | UNCERTAIN |
| 10-10 05:28Z | `5c1fa66` → `bca7f13` | same | same | — | NO EVIDENCE OF CHANGE |
| 10-10 16:17Z | `bca7f13` → `0fd5954` | same | same | — (`0fd5954` internal) | NO EVIDENCE OF CHANGE |

\* `de6872f` is inferred: the 10-05 receipt bound only `implementation_state`. The other four
states match their receipts' `product_state` exactly.

**Not replayable.** The F4 receipt of 10-05 and the F8 receipt of 10-14 compare against the 09-28
and 09-30 states. Those were never committed: no product commit exists between `ee3ce0d` (09-19)
and `de6872f`.

**Findings.**
1. No over-invalidation. Both NO EVIDENCE OF CHANGE pairs, including the motivating internal
   change `0fd5954`, keep the same surface at both viewports.
2. A known visible change escapes the journey as configured. The `fdbe263` phone-layout repair
   changes the 390×844 surface, but not the 1280×720 surface JamFree's journey observes. This is
   the agreed falsifier: the surface is too narrow at the journey's current viewports. The
   mechanism detects the change when the journey runs at the viewport where it is visible.

**Limitations.** One current runner and browser were used for every state. The observation
journey is the current one, applied unchanged to older products. The fixture forecasts depend on
the replay date, which was the same for all states.

**Decision (human, 2026-10-14): project-side viewport coverage.** Feature Workflow v10 is not
broadened into a rule tied to the Charter's viewports; viewports stay project-owned. When JamFree
adopts DBA-26, its governed F-0001 journey observes both 1280×720 and 390×844, because its own
Charter constraint C-2 makes the mobile width matter. The Observed Surface protects only the
behavior and viewports the governed journey exercises; CodeOS claims no coverage outside it. The
two rows that could not be replayed stay recorded as unobserved historical cases.
