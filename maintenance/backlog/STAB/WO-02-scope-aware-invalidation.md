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

### Part A brief (2026-10-14, awaiting decision)

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
6. **Versioning.** The semantics are versioned in Feature Workflow (v10), not in a new DBA
   configuration.

**Verification.**
- The three Part B tests.
- A determinism test: the same tree observed twice gives the same identity.
- Replay of the WO-01 F4 and F8 rows. Run the F-0001 journey in fixture mode before and after each
  trigger commit; there are no TomTom calls. The only F4 and F8 receipts that become stale should
  be those whose observation changed.

**Open decisions for you.**

1. **What counts as the observation.** Accessibility snapshot text is stable, but it misses
   pure-layout changes such as `fdbe263`. Screenshots at the C-2 viewports catch layout, but pixel
   determinism is a risk, and that risk fails toward stale. Recommendation: the accessibility
   snapshot plus the computed layout boxes of the journey's assertions.
2. **Coverage.** The binding is only as wide as the journey. The human may have judged something
   the journey does not visit. Recommendation: accept this, because F5g already makes the journey
   the feature's browser proof.
3. **How a project selects Feature Workflow v10** without a new DBA configuration, as the
   Charter's constraint requires. This needs checking against how configurations pin policy
   versions today.
4. **Terminology.** "Observed surface" is a new governed term. Proposed definition: *the identity
   of what a feature's browser verification observes of the running product, in deterministic
   provider mode*.

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
