# DBA-1 Equivalence Report

Evidence file for `changes/UPG-0065__CHG-20260809-001__dba1-equivalence-proof.md`. Assembles the
delta inventory (`CHG-20260807-001`) and `v1` decomposition (`CHG-20260808-001`) evidence into one
argument: `dba/configurations/DBA-1.yaml` (`doctrine: v1`, `review_policy: v1`,
`architecture_synthesis_policy: v1`, `implementation_profile_policy: v1`,
`controlled_plain_english_policy: v1`, `reviewer_tool_contract: v1`) is the
configuration-equivalent of `dba-system.md` @ commit `77599e9` (Invariant 1(a)-(b)). This is a
content-equivalence claim only — not approval, not activation.

---

## AC1 — Completeness (fresh)

Delta table's 175 non-`RETIRE` rows, grouped by `target_owner`, compared against each
`dba/*/v1.md` file's own Source Traceability table:

| target_owner | delta count | file | file count | missing | extra |
|---|---|---|---|---|---|
| doctrine | 108 | `dba/doctrine/v1.md` | 108 | none | none |
| review policy | 20 | `dba/policies/review/v1.md` | 20 | none | none |
| architecture-synthesis policy | 18 | `dba/policies/architecture-synthesis/v1.md` | 18 | none | none |
| implementation-profile policy | 18 | `dba/policies/implementation-profile/v1.md` | 18 | none | none |
| controlled-plain-english policy | 7 | `dba/policies/controlled-plain-english/v1.md` | 7 | none | none |
| reviewer tool contract | 4 | `dba/tools/reviewer/v1.md` | 4 | none | none |

No `rule_id` appears in more than one file. **PASS.**

## AC2 — `RETIRE` exclusion (fresh, full 28-row sweep)

Label check: grepped all 6 files for each of the 28 `RETIRE` `rule_id`s. One hit — `CPE-2d` — is
a mention inside `dba/policies/controlled-plain-english/v1.md`'s own boundary note explaining the
`CPE-2c`/`CPE-2d` sub-line split; the retired text itself ("A missing file still means
disabled...") is not transcribed anywhere. No other label hits.

Provenance check: compared all 28 `RETIRE` source line-ranges against all 175 included rows'
source line-ranges for numeric overlap. One overlap: `CPE-2d` (L492-494) vs. `CPE-2c` (L490-492),
sharing line 492 — the same documented sub-line split (`CPE-2c`'s content stops at "...
controlled-plain-english.md`."; `CPE-2d`'s retired clause starts on the same line and is not
transcribed). No other overlaps. **PASS.**

## AC3 — Content fidelity (fresh, full 175-row re-verification)

Every row's content block was diffed against `git show 77599e9:dba-system.md` at its declared
`source_anchor` range (whitespace-normalized; code-fence wrapping and each file's own trailing
`---` divider before "## Source Traceability" excluded as already-documented formatting choices,
not content).

- **163 of 175 rows: exact or legitimate-partial match**, including all 21 established sub-line
  clause splits (e.g. `REVIEW-5a-d`, `TRUTH-AUTHORITY-5/6/7`, `CPE-2a-c`, `IMPL-PROFILE-1a-c`,
  `HUMAN-NAV-2a/2b`) and all 9 previously-corrected anchors from the `CHG-20260807-001` hygiene fix,
  re-confirmed exact here.
- **12 of 175 rows: citation-precision or minor wording defects found**, none of which change a
  row's meaning, disposition, or `target_owner`:

  | rule_id | file | defect |
  |---|---|---|
  | FAILURE-BOUNDARY-5 | doctrine | anchor `L455-457` should be `L455-458` — quote finishes one line later |
  | HUMAN-NAV-1 | doctrine | anchor `L703-712` should be `L703-713` — sentence finishes one line later |
  | REVIEW-LOG-1b | review policy | anchor `L735-738` should be `L735-737` — content stops at 737; line 738 belongs to `REVIEW-LOG-1c` |
  | REVIEW-LOG-1c | review policy | anchor `L739-741` should be `L738-741` — content actually starts at 738 |
  | ARCH-GATE-3b | architecture-synthesis policy | anchor `L183-184` should be `L183-185` — sentence finishes one line later |
  | IMPL-PROFILE-4a | implementation-profile policy | anchor `L377-379` should be `L376-378` — block is shifted one line early |
  | IMPL-PROFILE-4b | implementation-profile policy | anchor `L379-382` should be `L378-381` — same shift |
  | IMPL-PROFILE-4c | implementation-profile policy | anchor `L382-383` should be `L381-382` — same shift |
  | CPE-3a | controlled-plain-english policy | anchor `L500-501` should be `L496-501` — undercounts; content already includes the "Call-site map" lead-in |
  | IMPL-PROFILE-8 | implementation-profile policy | content wording: reads "...any matched exception**,** paralleling..." where the pinned source has an em dash: "...exception **—** paralleling..." |
  | FILE-LAYOUT-5b | doctrine | content contains an added parenthetical, "(This fact: session handoffs are **optional**.)", not present in the pinned source |
  | FILE-LAYOUT-5c | doctrine | content contains an added parenthetical, "(This fact: session handoffs are **not DBA artifacts**...)", not present in the pinned source |

  **Human decision (2026-08-09):** presented these 12 findings and the scope conflict — this
  change's own Change Intent scopes `dba/*/v1.md` as read-only, while AC3's *original* Step 2
  wording (since reworded — see the change record's AC3 for the current text and the reason for
  the reword) had required any mismatch "fixed before this AC can PASS, not waived." The human
  explicitly chose to **waive**
  these 12 as non-blocking for `DBA-1`: none alter a row's disposition, `target_owner`, or
  normative meaning; they are citation-precision and cosmetic defects, not semantic drift. Tracked
  for cleanup as a follow-up (see `backlog/UPG-0065-modular-dba-configuration-architecture.md`'s
  Feature Thread), not fixed in this change. Per Truth Authority rule 1 (explicit human correction
  overrides), this is recorded as **AC3: PASS by explicit human waiver for 12 named rows; 163 of
  175 rows PASS by exact/verbatim match** — not an unqualified PASS.

## AC4 — `INTENTIONAL-BEHAVIOR-CHANGE` rows carry current form only (fresh)

All 19 `INTENTIONAL-BEHAVIOR-CHANGE` rows (`MODE-1`, `TRUTH-AUTHORITY-2`, `NN-1`, `NN-3`, `NN-6`,
`REVIEW-1`, `REVIEW-3`, `STEP4-GATE`, `STEP5-GATE`, `STEP6-ACTIVITY`, `STEP7-ACTIVITY`,
`STEP7-GATE`, `STEP9-GATE`, `ARCH-GATE-5`, `ARCH-GATE-6`, `ARCH-GATE-10`, `STAGE-TABLE-4`,
`NEVER-DO-7`, `REVIEW-LOG-1b`) were read side-by-side against their own Part 2 `proposed_rule`
text. Every row's `dba/` content matches the *current* system (e.g. `NN-1` still requires
individual approval at each of 9 stages, not the proposed Stage-3-to-8 batch cycle; `MODE-1` still
reads the file fully, not the proposed task-scoped subset; `REVIEW-3` still uses the current
3-round budget, not the proposed one-pass-plus-one-retry model). A fresh verbatim grep for 17
distinctive phrases drawn from all 19 `proposed_rule` texts across all 6 `dba/*/v1.md` files
returned zero matches. **PASS.**

## AC5 — `DBA-1.yaml` well-formed, not approved

`dba/configurations/DBA-1.yaml` names all six components at `v1` and sets `status: candidate`
(never `approved`). No other field in this file or this change's own artifacts asserts or implies
approval. **PASS.**

## AC6 — No approval or activation act, including untracked additions

`git diff -- dba-system.md dba-system-lean.md prompts/ scripts/ templates/ patterns/
dba/doctrine/ dba/policies/ dba/tools/` is empty; `git diff 77599e9 -- dba-system.md
dba-system-lean.md` is empty; `git status --porcelain --untracked-files=all -- prompts/
scripts/ templates/ patterns/ dba/` shows exactly one new entry, `dba/configurations/DBA-1.yaml`.
**PASS.** Re-checked at Reconcile (Step 4) against the change record's final diff, not re-cited
from this Step 3 result alone.

## AC7 — All 5 compatibility-sweep findings referenced (fresh count)

`grep "^### Finding" changes/UPG-0065__CHG-20260808-002__compatibility-report.md` returns 5
findings, matching this report:

- **Finding A** — "Default Advisory Review" 3-way split (rows 9, 18, 29).
- **Finding B** — "Multi-Feature Architecture Synthesis Gate" near-split (12 rows).
- **Finding C** — 5 `WHOLE-FILE-LOAD` references conditionally compatible pending a manifest-cascade
  mechanism that doesn't exist yet.
- **Finding D** — 2 pre-existing citation-drift defects, unrelated to the `v1` decomposition.
- **Finding E** — "Controlled Plain English Writing Discipline" near-split (3 rows).

All five are **consumer-side / activation-time concerns** — they describe how downstream-facing
files (`prompts/`, `templates/`, etc.) currently cite `dba-system.md`'s section structure, and
whether those citations stay accurate once a modular configuration is *activated*. None bears on
whether `DBA-1`'s own content (the six `dba/*/v1.md` files) is equivalent to `dba-system.md`'s
current content — that is this report's AC1-AC4 claim, a narrower and different one. Findings
A-E remain open pre-activation dependencies, not resolved here.

## AC8 — Cross-reference consistency

Verified at Reconcile (Step 4) via a grep sweep across the change record, the brief's Feature
Thread, `status/self-development.md`, `backlog/features.md`, and `status/roadmap.md`.

---

## Conclusion

`DBA-1` (`doctrine: v1`, `review_policy: v1`, `architecture_synthesis_policy: v1`,
`implementation_profile_policy: v1`, `controlled_plain_english_policy: v1`,
`reviewer_tool_contract: v1`) is the configuration-equivalent of `dba-system.md` @ `77599e9`,
content-complete (AC1), free of retired content (AC2), content-faithful modulo 12 explicitly
human-waived citation/cosmetic defects (AC3), and free of lean-proposal contamination (AC4).
`DBA-1.yaml` is well-formed and unambiguously `status: candidate` (AC5). This satisfies Invariant
1(a)-(b). Invariant 1(c) (explicit human approval of this exact combination) and 1(d)
(activation) remain separate, later gates — not reached by this change.
