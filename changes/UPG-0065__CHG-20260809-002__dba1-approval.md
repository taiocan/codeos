# Self-Development Change: UPG-0065__CHG-20260809-002 — dba1-approval

<!--
PURPOSE: Fifth change under UPG-0065 (Modular DBA Configuration Architecture). Phase A's fifth
sub-step (see backlog/UPG-0065's "Migration approach"): "explicit human approval of DBA-1."
Invariant 1(c) — the human decision that `DBA-1` (all six v1 components, proven
configuration-equivalent to dba-system.md in CHG-20260809-001) is approved as the migration
baseline. This is NOT activation (1(d)) — dba-system.md stays the sole file downstream projects
load, unchanged in content and role, until a separate, later change performs that atomic switch.
Workflow: prompts/codeos-self-dev.md (4-step loop).
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0065
primary_feature_id: UPG-0065
change_id: CHG-20260809-002
slug: dba1-approval
state: COMPLETE          # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 4-Reconcile   # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0065
related_features: []
review_series: S4         # S1/S2/S3/S4 all human APPROVED. S4: R1-R3 DO NOT ADVANCE fixed inline, budget exhausted at R3
review_profile: PROFILE-4   # downstream-doctrine (Step 0a)
review_state: ACCEPTED   # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round; resets per step)
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round. Reference the stable review SERIES (review_series) + review_state; exact rounds live
only in reviews/review-log.md and reviews/codex/*. -->

---

## Change Intent

**Why (problem in the toolkit):**

*(As of this change's Step 1, before Reconcile — the problem this change was opened to solve.
See the Reconciliation section below for the resolved, current state: `DBA-1` is now approved.)*
`CHG-20260809-001` proved `DBA-1` (`doctrine: v1`, `review_policy: v1`,
`architecture_synthesis_policy: v1`, `implementation_profile_policy: v1`,
`controlled_plain_english_policy: v1`, `reviewer_tool_contract: v1`) configuration-equivalent to
`dba-system.md` @ `77599e9` — Invariant 1(a)-(b). `dba/configurations/DBA-1.yaml` still read
`status: candidate`. Invariant 1 is explicit that "filing this brief does not pre-approve `DBA-1`"
and that approving this *exact pinned combination* is a separate, deliberate decision from how the
live system was approved incrementally across many unrelated change records — proving equivalence
is not the same act as approving the result, and the brief's own Migration Approach lists them as
two distinct sequential sub-steps. Nothing performed that decision or recorded it, until this
change did.

**What changes:**

- `changes/UPG-0065__CHG-20260809-002__dba1-approval.md` (this file) — the change record. The
  human's own explicit "Approved" at this change's Step 4 gate is the Invariant-1(c) approval act
  itself — this change doesn't produce a second, separate approval signal beyond that gate.
- `dba/configurations/DBA-1.yaml` — at Step 3/Reconcile, once (and only once) the equivalence
  evidence is re-confirmed still current, `status: candidate` is prepared to flip to
  `status: approved` with an `approved_at` date — but the flip is written into the file only after
  the human's Step 4 "Approved," never before, and never implied by an earlier step's approval of
  this change record's own drafting.
- `backlog/UPG-0065-modular-dba-configuration-architecture.md`, `status/self-development.md`,
  `status/roadmap.md` — Feature Thread / dashboard updated as this change progresses.

**Scope boundary — what stays the same:**

- `dba-system.md` is **not edited**. It remains the sole file downstream projects load, unchanged
  in content and role. No project's `.codeos` symlink target changes. This is the hard boundary
  separating this change (1(c), approval) from activation (1(d)), which stays a separate, later,
  explicitly-scoped change per Invariant 2 ("configuration activation is atomic" — there is no
  partially-migrated state, so approval must not silently double as activation).
- No `dba/*/v1.md` file's content is edited. The 12 citation-precision/wording defects
  `CHG-20260809-001`'s AC3 found and the human explicitly waived as non-blocking for `DBA-1`
  remain open and out of scope here too — this change concerns the approval decision, not a
  reopening of that already-settled waiver. They stay tracked as a follow-up hygiene item (see
  that change's Reconciliation), not folded into this one.
- No file under `prompts/`, `scripts/`, `templates/`, or `patterns/` is edited — none of the
  downstream-consumer compatibility findings (A, B, C, E from `CHG-20260808-002`) are addressed
  here. They are activation-time (1(d)) concerns, not approval-time (1(c)) ones, per
  `CHG-20260809-001`'s own AC7 reasoning, which this change inherits rather than re-litigates.
- No `DBA-2`, `dba-system-lean.md` decomposition, or any Phase B work.
- No new component version (`v2` of anything) is drafted.
- If the equivalence evidence turns out to have drifted since `CHG-20260809-001` (e.g. an
  unrelated concurrent change touched `dba-system.md`, `dba-system-lean.md`, or `dba/*/v1.md`),
  this change does not silently re-run the full equivalence proof — it stops and surfaces the
  drift to the human, since re-proving equivalence is `CHG-20260809-001`'s job, not this one's.

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** backlog/UPG-0065-modular-dba-configuration-architecture.md

---

## Acceptance Criteria

**`DBA-1.yaml` post-approval schema (binding on Step 3/Reconcile).** On approval, exactly three
fields are added to the existing candidate schema — `status: approved` (replacing
`status: candidate`), `approved_at: [ISO date]`, `approved_via: UPG-0065__CHG-20260809-002` — and
no other field changes. `doctrine`, `review_policy`, `architecture_synthesis_policy`,
`implementation_profile_policy`, `controlled_plain_english_policy`, `reviewer_tool_contract`,
`constructed_at`, and `equivalence_report` stay exactly as `CHG-20260809-001` wrote them.

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | **Equivalence evidence currency — the precondition for approving anything.** Nothing that `CHG-20260809-001`'s equivalence proof depends on has drifted since that change completed: `dba-system.md`, `dba-system-lean.md`, all 6 `dba/*/v1.md` files, and `changes/UPG-0065__CHG-20260807-001__delta-table.md` are byte-identical to their state at commit `1b8f984` (the commit that closed `CHG-20260809-001`). If any has changed, this AC fails outright — re-proving equivalence is `CHG-20260809-001`'s job, not this change's, and this change does not silently re-derive or assume continued equivalence. | `git diff 1b8f984 -- dba-system.md dba-system-lean.md dba/doctrine/ dba/policies/ dba/tools/ changes/UPG-0065__CHG-20260807-001__delta-table.md` — expect empty. Any non-empty result is reported to the human as a blocking discovery, not silently worked around. |
| 2 | **The approval act is traceable to the human's own Step 4 decision — checkable from durable, pinned evidence, not from ephemeral working-tree history.** The verifiable claim is narrower than "never edited before Step 4," which no post-commit check can prove from a repository that only records this change's final committed state: `dba/configurations/DBA-1.yaml`'s `approved_via` field names this exact change, and a matching Step 4 `APPROVE_STAGE` decision exists in the review log — the two artifacts cross-reference each other, which is what "this approval is properly recorded through this change's own gate" actually reduces to as a checkable fact. (Process discipline — editing `status` only at Reconcile, after the Step 4 decision is logged — will be followed, and this change's own Implementation Notes will record that it was, but that discipline is not itself the acceptance criterion, since it isn't independently verifiable after the fact.) | At Reconcile, verify both durable, post-commit facts hold together in the final state: (a) `dba/configurations/DBA-1.yaml`'s `approved_via` field names this exact change, `UPG-0065__CHG-20260809-002`; (b) `reviews/review-log.md` contains an `APPROVE_STAGE` decision entry for `UPG-0065__CHG-20260809-002 selfdev-step-4`. |
| 3 | **No activation act, anywhere in scope.** `dba-system.md` is not edited — same content, same role, same sole-file-downstream-projects-load status as before this change. No file under `prompts/`, `scripts/`, `templates/`, or `patterns/` is touched. | `git diff -- dba-system.md prompts/ scripts/ templates/ patterns/` — expect empty (tracked). `git status --porcelain --untracked-files=all -- prompts/ scripts/ templates/ patterns/` — expect empty (untracked). |
| 4 | **Scope boundary held — the 12 waived hygiene defects and the compatibility-sweep findings stay untouched.** No `dba/*/v1.md` file's content changes (the 9 anchor corrections, 1 wording fix, and 2 editorial-parenthetical removals that `CHG-20260809-001` recorded as human-waived remain exactly as they were). No `DBA-2` or Phase B artifact is created. | `git diff -- dba/doctrine/ dba/policies/ dba/tools/` — expect empty. `test ! -f dba/configurations/DBA-2.yaml` and no new file under `dba/*/v2.md` paths. |
| 5 | **`DBA-1.yaml` stays well-formed post-approval.** Matches the binding schema above exactly — the six component-version fields, `constructed_at`, and `equivalence_report` are byte-identical to `CHG-20260809-001`'s version; only `status`, `approved_at`, and `approved_via` differ. | Diff the pre-edit and post-edit `dba/configurations/DBA-1.yaml` directly; confirm only the three named fields changed. |
| 6 | **Cross-reference consistency.** The change record, the brief's Feature Thread, `status/self-development.md`, `backlog/features.md`, and `status/roadmap.md` agree `DBA-1` is approved once this change completes, comparing only the fields each surface actually records. | Grep sweep for `UPG-0065` / `CHG-20260809-002` / `DBA-1` across all five files at Reconcile; no stale pre-approval claims (AJ-020/AJ-025 class). |

---

## Implementation Notes

**AC1 (equivalence evidence currency) re-verified fresh at Step 3.** `git diff 1b8f984 --
dba-system.md dba-system-lean.md dba/doctrine/ dba/policies/ dba/tools/
changes/UPG-0065__CHG-20260807-001__delta-table.md` is empty — nothing `CHG-20260809-001`'s
equivalence proof depends on has drifted since that change's completing commit. `DBA-1.yaml`
confirmed still `status: candidate` — this Step has not touched it.

**This step deliberately makes no edit to `dba/configurations/DBA-1.yaml`.** Per AC2, the `status`
flip to `approved` happens only at Reconcile (Step 4), after the human's own Step 4 "Approved" is
recorded — this Step 3 produces no artifact change at all beyond this Implementation Notes entry
and the tracking-surface updates below. This is an intentional departure from this feature's other
changes, where Step 3 is where the substantive artifact work happens: here, the "artifact" this
change produces *is* the record of the human's approval decision, so writing it before that
decision exists would defeat the point of having a dedicated approval gate.

**No activation act.** `git diff -- dba-system.md prompts/ scripts/ templates/ patterns/` empty;
`git status --porcelain --untracked-files=all -- prompts/ scripts/ templates/ patterns/` empty.

**Scope boundary held.** `git diff -- dba/doctrine/ dba/policies/ dba/tools/` empty (the 12
human-waived hygiene defects from `CHG-20260809-001` remain untouched); no `DBA-2.yaml` or `v2`
component file exists.

---

## Reconciliation

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Equivalence evidence currency | **PASS** | `git diff 1b8f984 -- dba-system.md dba-system-lean.md dba/doctrine/ dba/policies/ dba/tools/ changes/UPG-0065__CHG-20260807-001__delta-table.md` empty, re-run fresh at Reconcile. No drift since `CHG-20260809-001` completed. |
| 2 | Approval act traceable via durable, post-commit co-presence | **PASS** | `dba/configurations/DBA-1.yaml`'s `approved_via: UPG-0065__CHG-20260809-002` names this exact change. `reviews/review-log.md` contains the matching entry, quoted verbatim here so this AC is checkable from this change record alone (that file has a pre-existing, already-tracked precheck false-positive in its own header legend, unrelated to this change and out of scope to fix here — tracked to `UPG-0031` per that file's own line 1023 history — so it is not passed whole into review packets):<br><br>`## 2026-08-09T13:13:43Z HUMAN DECISION — UPG-0065__CHG-20260809-002 — Stage selfdev-step-4`<br>`Commit at decision: 1b8f984fea02d2cbf14d053a283aac57d7bd99a8`<br>`Decision: APPROVE_STAGE`<br>`Reason/next: Human explicitly approved DBA-1 (doctrine: v1, review_policy: v1, architecture_synthesis_policy: v1, implementation_profile_policy: v1, controlled_plain_english_policy: v1, reviewer_tool_contract: v1) as the migration baseline per Invariant 1(c). This is the substantive approval act itself, not just a routine step-gate approval. Activation (Invariant 1(d)) remains separate and not authorized by this decision; dba-system.md is unchanged.`<br><br>Both facts present, cross-referencing each other. |
| 3 | No activation act | **PASS** | `git diff -- dba-system.md prompts/ scripts/ templates/ patterns/` empty; `git status --porcelain --untracked-files=all -- prompts/ scripts/ templates/ patterns/` empty. |
| 4 | Scope boundary held | **PASS** | `git diff -- dba/doctrine/ dba/policies/ dba/tools/` empty (12 waived hygiene defects untouched); no `dba/configurations/DBA-2.yaml` or any `v2` component file exists. |
| 5 | `DBA-1.yaml` stays well-formed post-approval | **PASS** | `git diff -- dba/configurations/DBA-1.yaml` shows exactly the three named data fields change (`status: candidate → approved`, `+approved_at`, `+approved_via`); the six component-version fields, `constructed_at`, and `equivalence_report` are byte-identical to `CHG-20260809-001`'s version. The header **comment** (documentation, not a YAML data field) was also updated — it previously stated "This file's existence is NOT an approval act," which became false the moment approval happened; leaving a known-false statement on disk would itself violate Truth Authority. AC5's own binding-schema text scopes its byte-identical claim to the eight named data fields, not file-wide text, so this is not a criterion violation — noted explicitly rather than left for a reviewer to ask about. |
| 6 | Cross-reference consistency | **PASS** | Neither `backlog/features.md` nor `status/roadmap.md` tracks step-level or approval-level state — both are feature-level only, quoted verbatim here so this AC is checkable from this change record alone:<br><br>`backlog/features.md:98`: `\| UPG-0065 \| [UPG-0065-modular-dba-configuration-architecture.md](UPG-0065-modular-dba-configuration-architecture.md) \| Modular DBA Configuration Architecture \| P1 \| IN_PROGRESS \|`<br>`status/roadmap.md:127`: `\| UPG-0065 \| Modular DBA Configuration Architecture \| P1 \| — \| CHG-20260807-001, CHG-20260808-001, CHG-20260808-002, CHG-20260809-001, CHG-20260809-002 \| IN_PROGRESS \|`<br><br>Both correctly say `UPG-0065` is `IN_PROGRESS` (true — Phase A's activation sub-step remains) and neither makes any claim about `DBA-1`'s approval state, so neither requires editing and neither can be stale on a fact it doesn't track. `status/self-development.md`, the brief's status line, and this change record's own trace header all state `DBA-1` approved, not activated — consistent with each other, verified by this Reconciliation's own writing. |

---

### The approval decision

Invariant 1(c) is satisfied. **`DBA-1`** — `doctrine: v1`, `review_policy: v1`,
`architecture_synthesis_policy: v1`, `implementation_profile_policy: v1`,
`controlled_plain_english_policy: v1`, `reviewer_tool_contract: v1` — is **approved as the
migration baseline**, on the strength of `CHG-20260809-001`'s equivalence proof (163/175 rows
exact-verbatim match against `dba-system.md` @ `77599e9`; 12 rows with citation-precision defects
explicitly waived as immaterial in that change) and this change's fresh re-confirmation of no
drift. Recorded in `dba/configurations/DBA-1.yaml` (`status: approved`, `approved_at:
2026-08-09`, `approved_via: UPG-0065__CHG-20260809-002`) and in `reviews/review-log.md`'s Step 4
human decision entry.

**Activation (Invariant 1(d)) is explicitly not authorized by this decision.** `dba-system.md` is
unchanged — still the sole file downstream projects load, still in its current monolithic-doctrine
role. Switching it to a thin manifest naming `DBA-1` active is a separate, later, explicitly-scoped
change, one that also needs to resolve or consciously accept the compatibility sweep's Findings A,
B, C, and E first (`CHG-20260808-002`) — none of which this change touches.
