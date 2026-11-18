# WO-04 — DBA Impact-Aware Adoption

PROTECTED · after WO-02, reusing its dependency model

## Goal

Adopting a DBA re-runs only what its semantic change affects.

## Direction

- Compare the old and new configurations' semantic closures, and mechanically derive which bindings
  and checkpoints change. No manual "invalidates" declarations.
- Accepted features that are unaffected stay valid.
- Operating rule, not a mechanism: projects adopt DBA versions between feature cycles.

## Verification

Replay JamFree's DBA-32 → 33 → 34 adoptions, and show which re-runs were necessary.

## Observed case: JamFree DBA-34 → DBA-35 (2026-11-18)

JamFree adopted DBA-35 in c3b2e56. Read-only `status` afterwards showed:

- **v14 held.** F-0001, F-0002, F-0009 and F-0010 are VALID, F1–F9 complete, and no F6, F8 or F9
  decision is asked.
- **Still invalidated by the `codeos.yaml` bytes alone:**
  - B3, with B4 behind it (another human re-confirmation);
  - the four features' `project_qualification` records and the deployment qualification;
  - the Health and Platform Assurance states derived from them.

DBA-35 changed neither the platform baseline nor the qualification semantics. These are the
bindings WO-04 must derive from semantic impact: `checker.rs` binds `codeos_yaml` for B3, B4
(`initial_product_preview`), F6 and `project_qualification`.

**Human direction (2026-11-18).** JamFree pays the DBA-35 refresh once now (B3, B4,
qualifications), with no TomTom calls; this case is WO-04's canonical example. WO-04's target:

> A DBA or configuration change invalidates Bootstrap or qualification only when the changed
> configuration semantics intersect that checkpoint's actual dependency set. A raw change to the
> `codeos.yaml` bytes is not sufficient by itself.

This applies to B3, B4, project and deployment qualification, and the Health and Platform Assurance
states derived from them. **Boundary:** B4 stays valid only because its own dependency identity is
unchanged; CodeOS never special-cases DBA adoption, or a product that "looks the same".

**Refresh result (JamFree, 2026-11-18).** B3 and all five qualifications passed in about 53 s, with
no provider calls. Still stale afterwards, from the same cause:

- **Health:** the F9-checkpoint `project_qualification` records bind `codeos_yaml`. Only a feature
  `check` writes them; `qualify-project` writes the candidate-checkpoint records.
- **Platform Assurance:** the deterministic F3/F5/F5g records show "execution_identity changed",
  because `evidence_set::common_inputs` includes the `codeos.yaml` bytes. Features stay VALID:
  Current Validity compares the bundle's inputs, not record currency.

A deterministic feature `check` clears both, at about 15–20 min for four features. That is the
measured cost WO-04 removes.

**Deterministic checks (JamFree 28fca60).** F-0001 took 269 s, F-0002 26 s, F-0009 3 s, F-0010 3 s;
no provider calls. Neither stale class cleared:

- Every feature `check` now stops at "F6 WAITING: 'codeos_yaml' changed". Clearing it would mean
  the full F6 → candidate → Stage 8 → F8/F9 cascade for an unchanged product; not run, by
  decision.
- Health and Platform Assurance stay STALE. They are read-only reports, not gates, and all four
  features stay VALID and COMPLETE. They are recorded as WO-04 debt.

**Conformance miss in DBA-35 (found here).** Feature Workflow v14 keeps `codeos.yaml` in F6's
dependency set. The human's annex table for F6 does not include it. The WO-02 annex note claimed
the F6 rule conformed; that was inaccurate. DBA-35 is frozen, so WO-04 owns the repair, in the next
DBA. WO-04's binding targets, each to be replaced by derived configuration impact:

- F6 `codeos_yaml`;
- `codeos_yaml` in execution identities (`evidence_set::common_inputs`);
- `codeos_yaml` in `project_qualification`, both the candidate and F9 checkpoints;
- `codeos_yaml` in B3 and B4.

## Part A — Decision brief (2026-11-18, for approval)

**Finding: a closure diff is not a reliable impact input.** A configuration's semantic closure
lists only its directly selected files. Feature Workflow v14 incorporates v13 by reference, but v13
is not in DBA-35's closure, so terms used only by v13 seem to vanish (`Validation claim`,
`Feature ID`, …). Impact derived from raw closure diffs would be noisy in both directions.

**Finding: `codeos.yaml` mixes three things.** It holds the DBA selection, project sections
(`platform`, `runtime_verification`, `reality_verification`, `artifacts`) and the fixed `mechanics`
display. Every consumer below hashes all of it.

**Rule W4.** `codeos.yaml` is never a dependency as a whole. Each consumer binds only:

1. the canonical parsed values of the project sections it reads; and
2. the identity of the governing contract it applies: a selected policy or tool contract,
   identified by its immutable version. Issued versions never change, and a new version is the
   only way semantics change. This is also the reviewer contract's semantic identity that R5 left
   to WO-04.

| Consumer | Project sections | Governing contract | Today |
|---|---|---|---|
| B3 integrated baseline | `platform`, `runtime_verification` | workflow-bootstrap policy, Codeos Mechanics policy | `codeos.yaml` bytes |
| B4 platform direction | `platform` (plus the Charter, unchanged) | workflow-bootstrap policy | `codeos.yaml` bytes + Charter |
| Execution identity of shared verifications | `platform`, `runtime_verification`, `reality_verification` | Codeos Mechanics policy (already bound) | `codeos.yaml` bytes + mechanics |
| `project_qualification` (candidate and F9) | none; the project script and the Charter are bound already | the qualification protocol's schema version, owned by the tool (`schema: 1`) | `codeos.yaml` bytes |
| F6 reconciliation | none (annex conformance) | — | `codeos.yaml` bytes |
| Stage 8 / Candidate Bundle | — | reviewer tool contract + review policy versions (R5, already in DBA-35) | — |

**Predicted for DBA-34 → DBA-35:** only Review, Feature Workflow and Governance changed. B3, B4,
execution identities, qualification and F6 stay current. Stage 8 re-runs only for a new candidate
under the new review policy. None of the refresh work measured above would be needed.

**Invariant.** A consumer goes stale when a section it reads or the contract version it applies
changes, and never for any other edit to `codeos.yaml`. History is never rewritten. Receipts and
records written earlier keep their `codeos_yaml` binding and are evaluated as recorded (DBA-35
pattern). Lands as a new DBA (Feature Workflow v15 with the F6 repair, plus Bootstrap and
qualification changes as needed).

**Verification.** Replay DBA-32 → 33 → 34 → 35 on JamFree history: which B3, B4, qualification and
verification re-runs would remain. The DBA-34 → 35 adoption must need none. Fixtures: a
`platform` change stales B3 and B4; a comment or DBA-selection change stales nothing; a Codeos
Mechanics version change stales execution identities.

**Questions for the human:**

1. Approve Rule W4 with version-granular contract identity, instead of closure diffs or declared
   per-version impact (both rejected above)?
2. Qualification protocol identity: is the tool-owned schema version acceptable as its governing
   contract? The alternative is binding the Feature Workflow version, which owns the protocol text;
   that would re-qualify on every Feature Workflow change, including DBA-35's.
3. U5 (advisory as-of identity): `cargo audit` has a local advisory database with an identity;
   `npm audit` queries the registry online and has none. Binding the advisory identity for
   currency would therefore need network access on every check. The proposal is to record it in
   the qualification result as provenance only (Boundary 2: as-of identity exists, and no rule
   depends on it yet), and to keep any freshness rule for later evidence. Fold it into WO-04, or
   keep it separate?
