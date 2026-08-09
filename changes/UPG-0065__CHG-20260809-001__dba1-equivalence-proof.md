# Self-Development Change: UPG-0065__CHG-20260809-001 — dba1-equivalence-proof

<!--
PURPOSE: Fourth change under UPG-0065 (Modular DBA Configuration Architecture). Phase A's fourth
sub-step (see backlog/UPG-0065's "Migration approach"): "prove DBA-1 semantically equivalent to
the live monolith." Constructs the candidate `DBA-1.yaml` configuration (per Invariant 1(a)) and
assembles the already-gathered delta-inventory and v1-decomposition evidence into a formal
equivalence proof (Invariant 1(b)) — NOT approval (1(c)) and NOT activation (1(d)), both of which
stay explicitly out of scope here.
Workflow: prompts/codeos-self-dev.md (4-step loop).
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0065
primary_feature_id: UPG-0065
change_id: CHG-20260809-001
slug: dba1-equivalence-proof
state: COMPLETE          # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 4-Reconcile  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0065
related_features: []
review_series: S4         # S1/S2/S3/S4 all human APPROVED. S4: R1 NO OBJECTION, 0 findings, evidence B
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

`backlog/UPG-0065-modular-dba-configuration-architecture.md`'s Migration Approach names Phase A's
fourth sub-step, after the normative delta inventory (`CHG-20260807-001`, COMPLETE), the `v1`
component decomposition (`CHG-20260808-001`, COMPLETE), and the downstream consumer compatibility
sweep (`CHG-20260808-002`, COMPLETE): "prove `DBA-1` semantically equivalent to the live monolith."
Invariant 1 in the brief states this proof precedes any approval: "`DBA-1` must be (a) constructed
as the configuration-equivalent of today's live system, (b) proven semantically equivalent through
the delta inventory and compatibility sweep, (c) explicitly approved as the migration baseline, and
only then (d) activated. Filing this brief does not pre-approve `DBA-1`." This change performs (a)
and (b) only.

Most of the evidence this proof needs already exists, produced across the prior three changes: the
delta inventory's complete 203-row disposition, and the `v1` decomposition's own Reconciliation
(which already verified completeness, content fidelity, and `RETIRE`-row justification per file).
What does not yet exist is (i) the candidate `DBA-1.yaml` configuration object itself — today the
six `dba/*/v1.md` files exist but nothing names them as one pinned combination — and (ii) a single
assembled argument that ties the existing per-change evidence together into one equivalence claim
about the *whole* combination, rather than six separately-verified parts.

**What changes:**

- `changes/UPG-0065__CHG-20260809-001__dba1-equivalence-proof.md` (this file) — the change record.
- `changes/UPG-0065__CHG-20260809-001__equivalence-report.md` (new, created at Step 3) — the
  assembled equivalence argument, kept as a separate evidence file for the same reason the delta
  table and compatibility report were: it is the durable analytical deliverable, independently
  reviewable.
- `dba/configurations/DBA-1.yaml` (new, created at Step 3) — the candidate configuration naming
  all six component versions (`doctrine: v1`, `review policy: v1`, `architecture-synthesis policy:
  v1`, `implementation-profile policy: v1`, `controlled-plain-english policy: v1`, `reviewer tool
  contract: v1`), explicitly marked as a candidate, not approved — per Invariant 1, this file's
  existence is not itself an approval act.
- `backlog/UPG-0065-modular-dba-configuration-architecture.md`, `status/self-development.md`,
  `status/roadmap.md` — Feature Thread / dashboard / wave-plan updated as this change progresses.

**Scope boundary — what stays the same:**

- `dba-system.md` and `dba-system-lean.md` are not edited. `dba-system.md` remains the sole file
  downstream projects load, unchanged in content and role.
- No file under `prompts/`, `scripts/`, `templates/`, or `patterns/` is edited. The compatibility
  sweep's five findings (Finding A, the "Default Advisory Review" 3-way split; Finding B, the
  "Multi-Feature Architecture Synthesis Gate" near-split; Finding C, the manifest-cascade
  dependency for `WHOLE-FILE-LOAD` references; Finding D, the two pre-existing citation-drift
  defects; Finding E, the "Controlled Plain English Writing Discipline" near-split — corrected
  during Step 2 review, which found the original "three findings" claim here undercounted the
  report's actual Part 2, omitting Findings B and E) are **referenced** in the equivalence report as known
  pre-activation dependencies, not resolved here — they concern *consumer-side* compatibility, a
  precondition for activation (Invariant
  1(d)), not `DBA-1`'s own *content* equivalence to `dba-system.md` (Invariant 1(b)), which is a
  different, narrower claim this change scopes itself to.
- No `dba/*/v1.md` file's content is edited — the equivalence proof reads them, never writes to
  them.
- **`DBA-1` is not approved or activated.** `DBA-1.yaml` is created with an explicit
  non-approved status field (e.g. `status: candidate` or `status: proposed`); no human approval of
  this exact pinned combination happens in this change, per Invariant 1(c)-(d) — that is a
  separate, later gate this change does not reach.
- No `DBA-2`, `dba-system-lean.md` decomposition, or any Phase B work — per the explicit human
  decision this session to finish Phase A before starting Phase B.
- No new component version (`v2` of anything) is drafted.

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** backlog/UPG-0065-modular-dba-configuration-architecture.md

---

## Acceptance Criteria

**`DBA-1.yaml` schema (binding on Step 3).** `dba/configurations/DBA-1.yaml` names each of the
six components at the version it currently has (`doctrine: v1`, `review_policy: v1`,
`architecture_synthesis_policy: v1`, `implementation_profile_policy: v1`,
`controlled_plain_english_policy: v1`, `reviewer_tool_contract: v1`), plus `status: candidate`
(never `approved` — Invariant 1), a `constructed_at` date, and a `equivalence_report` field
pointing at this change's report file.

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | **Completeness, re-verified fresh, not cited from a prior change.** Every one of the 175 non-`RETIRE` rows in the accepted delta table is present in exactly one `dba/*/v1.md` file's Source Traceability table, matching its `target_owner`; no row missing, no row duplicated across files. | Re-run the per-`target_owner` `comm -23`/`comm -13` diff between the delta table's `rule_id` set and each of the 6 files' Source Traceability tables — the same method `CHG-20260808-001`'s own Reconciliation used, re-executed here as this change's own evidence, not assumed carried over. All six expected empty in both directions. |
| 2 | **`RETIRE` exclusion, re-verified fresh — by content provenance, not just by label.** None of the 28 `RETIRE` `rule_id`s, and none of their transcribed text, appears in any `dba/*/v1.md` file. | Grep all 6 files for each `RETIRE` `rule_id` (label check). Separately, for all 28 `RETIRE` rows (not just the one previously-known case), compare each row's pinned `source_anchor` line range against every included row's declared `source_anchor` for a line-number overlap; for every overlap found, read both rows' source text side by side and confirm the included row's transcribed text stops before the `RETIRE`d row's own clause begins — the same full sweep `CHG-20260808-001`'s Reconciliation performed, re-executed here as this change's own evidence, not cited as already settled. |
| 3 | **Content fidelity — full re-verification against the pinned source, not a sample.** Every one of the 175 rows' transcribed content is checked against `dba-system.md` @ commit `77599e9`'s text at its row's (corrected, where applicable) `source_anchor`. This is the load-bearing claim of the whole proof — a sampled or partial check does not satisfy it. A mismatch is a defect: it is fixed, unless fixing it would mean editing a `dba/*/v1.md` file's content, which this change's own scope boundary declares read-only — in which case it is not silently accepted, but presented explicitly to the human, naming every mismatch and its content-vs-citation nature, for an explicit decision (fix inline as a scope amendment, split into its own change, or waive as immaterial). Whatever the human decides is recorded verbatim against each affected row; a bare, undifferentiated PASS is never acceptable when any mismatch was found. *(Reworded during Step 3 — Codex's `selfdev-step-3` R1 found the original wording, "fixed before this AC can PASS, not waived," directly contradicted the artifact's own recorded outcome once 12 mismatches were found and the human explicitly chose to waive them; this states the actual invariant being applied here, the same class of fix `CHG-20260808-001`'s AC2 received at its own Step 3 R2.)* | For every row, extract its content block and its declared `source_anchor` range, and diff the block against `git show 77599e9:dba-system.md` for that range. Any row that doesn't match exactly (accounting only for the already-documented, human-visible formatting choices — e.g. code-fence wrapping for tree fragments, each file's own trailing divider before "## Source Traceability" — not content differences) is named individually with its mismatch type. Full per-row results, and the human's explicit disposition of every mismatch, are recorded in the equivalence report, not summarized away. |
| 4 | **`INTENTIONAL-BEHAVIOR-CHANGE` rows carry the current form only, re-confirmed.** None of the 19 `INTENTIONAL-BEHAVIOR-CHANGE` rows' `dba/` content matches its own Part-2 `proposed_rule` (by meaning, not just verbatim string) — `DBA-1` represents the system as it is today, not the lean proposal. | Re-read all 19 rows side-by-side (content block / pinned source / Part 2 `proposed_rule`) — the same check `CHG-20260808-001` performed, re-executed as this change's own evidence. Plus a fresh verbatim grep for distinctive `proposed_rule` phrases across all 6 files. |
| 5 | **`DBA-1.yaml` is well-formed and explicitly not approved.** The file matches the schema above; `status` is `candidate`, never `approved`; no other field in the file or this change's own prose asserts or implies approval. | Read the file against the schema. Grep this change's own artifacts for `approved` near `DBA-1`; any hit is read in context and confirmed non-assertive. |
| 6 | **No approval or activation act, anywhere in scope, including untracked additions.** `dba-system.md` remains the sole file downstream projects load, unchanged in content and role. No file outside `dba/configurations/DBA-1.yaml` is added or modified — tracked or untracked — including the six existing `dba/*/v1.md` files, which this change's own scope boundary requires stay read-only, not only the consumer directories. | `git diff 77599e9 -- dba-system.md dba-system-lean.md` → expect empty. `git diff -- prompts/ scripts/ templates/ patterns/ dba/doctrine/ dba/policies/ dba/tools/` → expect empty for tracked changes. Separately — `git diff` alone cannot see untracked files — `git status --porcelain --untracked-files=all -- prompts/ scripts/ templates/ patterns/ dba/` → the only new entry anywhere under `dba/` is `dba/configurations/DBA-1.yaml`; zero entries under `prompts/`, `scripts/`, `templates/`, `patterns/`. |
| 7 | **All of the compatibility sweep's findings are referenced, not silently dropped — the actual count, re-confirmed, not assumed.** The equivalence report explicitly lists every finding `CHG-20260808-002`'s report names in its own Part 2 — re-counted here, not carried forward from memory — and states why each is a consumer-side/activation-time concern that does not bear on `DBA-1`'s own content equivalence to `dba-system.md`. | Grep `changes/UPG-0065__CHG-20260808-002__compatibility-report.md` for `^### Finding` to get the authoritative count and list at Step 3, before writing this AC's evidence; confirm the equivalence report names every one found, not a remembered subset. |
| 8 | **Cross-reference consistency.** The change record, the brief's Feature Thread, `status/self-development.md`, `backlog/features.md`, and `status/roadmap.md` agree on this change's current step and state, comparing only the fields each surface actually records. | Grep sweep for `UPG-0065` / `CHG-20260809-001` across all five files at Reconcile; no stale step/state claims (AJ-020/AJ-025 class). |

---

## Implementation Notes

Built `dba/configurations/DBA-1.yaml` (all six components at `v1`, `status: candidate`,
`equivalence_report` pointing at the new report file) and
`changes/UPG-0065__CHG-20260809-001__equivalence-report.md`, assembling all 8 ACs' fresh evidence.

**AC1 (completeness) and AC2 (`RETIRE` exclusion):** re-run exactly as specified — per-`target_owner`
comparison of the delta table's 175 non-`RETIRE` rows against each of the 6 files' Source
Traceability tables (all six pairs empty both directions, no cross-file duplicates); full 28-row
`RETIRE` provenance sweep (one label mention, `CPE-2d` inside `CPE-2c`'s own boundary-note
explanation — not the retired text itself; one line-range overlap, the same already-documented
`CPE-2c`/`CPE-2d` sub-line split). Both PASS clean, no new findings.

**AC3 (content fidelity, full 175-row re-verification) — the load-bearing check, and where this
step found real defects.** Built a script-assisted diff: extracted every row's content block and
declared `source_anchor` from all 6 files, diffed each against `git show 77599e9:dba-system.md` at
its range (whitespace-normalized). 163 of 175 rows matched exactly or as a legitimate sub-line
clause split (the same pattern already established for rows like `REVIEW-5a-d` and
`TRUTH-AUTHORITY-5/6/7`). The remaining 12 did not: 9 rows whose declared `source_anchor` undercounts
or shifts by 1-4 lines relative to where the content actually sits (`FAILURE-BOUNDARY-5`,
`HUMAN-NAV-1`, `REVIEW-LOG-1b`, `REVIEW-LOG-1c`, `ARCH-GATE-3b`, `IMPL-PROFILE-4a/4b/4c`, `CPE-3a`),
1 row with an altered punctuation mark (`IMPL-PROFILE-8`: content has "," where the pinned source
has an em dash "—"), and 2 rows carrying an added editorial parenthetical not present in the pinned
source (`FILE-LAYOUT-5b`, `FILE-LAYOUT-5c`). Full per-row detail is in the equivalence report.

This directly collided with this change's own declared scope boundary — "No `dba/*/v1.md` file's
content is edited" — against AC3's *original* Step 2 wording (since reworded below), which had
required "any row that doesn't match is a defect, fixed before this AC can PASS, not waived." Per
Truth Authority (surface unresolved conflicts rather than silently resolving them), presented both
the 12 findings and the conflict to the human directly rather than picking a side unilaterally.
**Human decision (2026-08-09): waive all 12 as
non-blocking for `DBA-1`** — none change a row's disposition, `target_owner`, or normative meaning;
`dba/*/v1.md` stays untouched in this change; the 12 are recorded in the equivalence report and
tracked as a follow-up hygiene item, not fixed here. AC3 is recorded as **PASS by explicit human
waiver for the 12 named rows; 163 of 175 rows PASS by exact/verbatim match** — not an unqualified
PASS, consistent with how the earlier 7-anchor hygiene fix was itself surfaced rather than silently
folded in.

**AC4 (`INTENTIONAL-BEHAVIOR-CHANGE` rows, current form only):** all 19 rows read side-by-side
against their own Part 2 `proposed_rule` text — every one reflects the current system, not the lean
proposal. A fresh verbatim grep for 17 distinctive phrases drawn from the 19 `proposed_rule` texts,
across all 6 files, returned zero matches. PASS.

**AC5 (`DBA-1.yaml` well-formed, not approved):** file matches the binding schema; `status:
candidate` throughout; no `approved` assertion anywhere in this change's own artifacts near
`DBA-1`. PASS.

**AC6 (no approval/activation act):** `git diff -- dba-system.md dba-system-lean.md
prompts/ scripts/ templates/ patterns/ dba/doctrine/ dba/policies/ dba/tools/` empty;
`git status --porcelain --untracked-files=all -- prompts/ scripts/ templates/ patterns/ dba/`
shows only the new `dba/configurations/DBA-1.yaml`; `git diff 77599e9 -- dba-system.md
dba-system-lean.md` empty. PASS.

**AC7 (all compatibility-sweep findings referenced, fresh count):** `grep "^### Finding"` against
`CHG-20260808-002__compatibility-report.md` returns exactly 5 (A-E), matching the equivalence
report's own reference list, each with an explicit consumer-side/activation-time rationale
distinguishing it from this report's narrower content-equivalence claim. PASS.

**AC8 (cross-reference consistency):** verified at Reconcile below.

**Step 3 review R1** (Codex, `selfdev-step-3`): DO NOT ADVANCE, 1 High IN-SCOPE BLOCKER — AC3's
own written text ("fixed before this AC can PASS, not waived") directly contradicted the artifact's
recorded outcome (PASS by explicit human waiver for 12 rows), a false-acceptance-claim risk, not a
cosmetic wording gap. Fixed: reworded AC3 above to state the actual invariant now in effect (a
mismatch is fixed, or — when fixing would mean editing a file this change declares read-only —
named explicitly to the human for an explicit decision, recorded verbatim per row) — the same class
of correction `CHG-20260808-001`'s AC2 received at its own Step 3 R2, for the same reason: Step 3
implementation surfaced a real exception the Step 2 wording hadn't anticipated. One non-blocking
note in the same round: AC6 was stated as PASS with evidence in this change record but only
deferred to Step 4 in the equivalence report — fixed by giving AC6 its own evidence and PASS result
directly in the report, re-checked (not re-cited) at Reconcile.

---

## Reconciliation

Each acceptance criterion independently re-verified at Reconcile — not re-trusting Step 3's own
claims — the same discipline this feature's prior three changes applied at their own Step 4.

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Completeness | **PASS** | `dba/doctrine/`, `dba/policies/`, `dba/tools/reviewer/`, and `changes/UPG-0065__CHG-20260807-001__delta-table.md` are all unchanged (`git diff --stat` empty) since Step 3's per-`target_owner` comparison — that result still holds: all six pairs empty both directions, no cross-file duplicates. |
| 2 | `RETIRE` exclusion | **PASS** | Same unchanged-files basis as AC1; Step 3's full 28-row sweep result stands: one label mention inside `CPE-2c`'s own documented boundary note (not the retired text), one line-range overlap (the same already-documented `CPE-2c`/`CPE-2d` split). |
| 3 | Content fidelity | **PASS by explicit human waiver for 12 named rows; PASS by exact/verbatim match for the other 163.** Not an unqualified PASS — recorded this way deliberately, per Step 3 R1's finding that a bare PASS would misstate what happened. | `changes/UPG-0065__CHG-20260809-001__equivalence-report.md`'s AC3 section; the 12-row defect table; the human decision on 2026-08-09 to waive them as non-blocking (none change disposition, `target_owner`, or normative meaning); AC3's own criterion text (above) reworded during Step 3 review to state this as the actual invariant, not an exception to a still-absolute rule. |
| 4 | `INTENTIONAL-BEHAVIOR-CHANGE` rows, current form only | **PASS** | Same unchanged-files basis; Step 3's 19-row side-by-side read and 17-phrase verbatim grep (zero matches) still hold. |
| 5 | `DBA-1.yaml` well-formed, not approved | **PASS** | Re-read `dba/configurations/DBA-1.yaml` directly at Reconcile: all six components named at `v1`, `status: candidate`, `equivalence_report` field present and correct. Grepped this change's own artifacts for `approved` near `DBA-1`/`DBA-1.yaml`: only hits are the schema comment's `candidate \| approved` field-domain note and prose explicitly stating `DBA-1` is *not* approved — no assertive hit. |
| 6 | No approval or activation act | **PASS** | Re-run fresh at Reconcile: `git diff -- dba-system.md dba-system-lean.md prompts/ scripts/ templates/ patterns/ dba/doctrine/ dba/policies/ dba/tools/` empty; `git diff 77599e9 -- dba-system.md dba-system-lean.md` empty; `git status --porcelain --untracked-files=all -- prompts/ scripts/ templates/ patterns/ dba/` shows exactly one new entry, `dba/configurations/DBA-1.yaml`. |
| 7 | All compatibility-sweep findings referenced | **PASS** | `grep -c "^### Finding" changes/UPG-0065__CHG-20260808-002__compatibility-report.md` → 5, re-confirmed at Reconcile; `grep -n "^- \*\*Finding" changes/UPG-0065__CHG-20260809-001__equivalence-report.md` → A, B, C, D, E, all five present. |
| 8 | Cross-reference consistency | **PASS (after fixing staleness found at Reconcile itself)** | A fresh grep sweep for `UPG-0065`/`CHG-20260809-001` across the change record, the brief's Feature Thread, and `status/self-development.md` found the dashboard row and the brief's own S3 Reviews-table row still read as "3-Implement" / "Awaiting human gate decision" after the human had already approved Step 3 — the same class of staleness this feature's every prior change caught at this exact point (AJ-020/AJ-025). Fixed before writing this table, not after. `backlog/features.md` and `status/roadmap.md` re-checked: both are feature-level only (`IN_PROGRESS`, change-id list), track no step-level state, and required no edit. |

**Findings scope-triage (Step 3 review, carried here for the record):** the AC3 self-contradiction
(R1) and the governance-state staleness plus stale pre-reword quotes (R2) were both **IN-SCOPE
BLOCKER**, both fixed inline before R3. No **OUT-OF-SCOPE BACKLOG**, **REJECTED**, or
**SELF-REFERENCE**/**REVIEW-BOOKKEEPING** findings this step beyond what's already logged above.

**Follow-up tracked, not filed as a new `UPG-####`:** the 12 `dba/*/v1.md` citation-precision and
wording defects named in AC3 (9 anchor-range corrections, 1 em-dash-to-comma wording fix, 2 added
editorial parentheticals to remove or convert to a comment) remain open, human-waived-as-immaterial
for `DBA-1`'s purposes. They are small enough, and internal enough to the already-existing `dba/`
tree, that a dedicated feature brief would be disproportionate; the natural place to fix them is a
short, narrowly-scoped hygiene change whenever `dba/*/v1.md` is next legitimately opened for
editing (e.g. alongside drafting `v2` candidates in Phase B, or a dedicated small CHG if that's
sooner) — noted here so the equivalence report's own list isn't the only record of them.

This change does not reach Invariant 1(c) (explicit approval of `DBA-1` as the migration baseline)
or 1(d) (activation). Both remain separate, later, explicitly-scoped changes.

---
