# Downstream Consumer Compatibility Report — `dba-system.md` references

Per `changes/UPG-0065__CHG-20260808-002__downstream-consumer-compatibility-sweep.md`'s Change
Intent and Acceptance Criteria. Every `dba-system.md` occurrence across `prompts/`, `scripts/`,
`templates/`, `patterns/` gets exactly one row. Read-only analysis — no consumer file, `dba-system.md`,
or `dba/*/v1.md` is edited by this report.

**`kind` values:**
- **STRUCTURAL-POINTER** — cites a specific `dba-system.md` section/sub-topic by name. `detail`
  names the `dba/` component file(s) that content now lives in (per
  `changes/UPG-0065__CHG-20260808-001__v1-component-decomposition.md`'s six Source Traceability
  tables), and whether the citation still matches the source verbatim.
- **WHOLE-FILE-LOAD** — instructs reading/loading `dba-system.md` in full, no section dependency.
- **GENERIC-MENTION** — refers to `dba-system.md` as a concept/path only, no content dependency.

---

## Part 1 — Reference Table

| # | file | line | kind | cited text (verbatim, truncated) | detail |
|---|---|---|---|---|---|
| 1 | patterns/controlled-plain-english.md | 10 | STRUCTURAL-POINTER (section) | "Controlled Plain English Writing Discipline" | Exact match to the `##` heading. **Near-split, not clean — see Finding E.** 7 of 8 non-`RETIRE` rows land in `dba/policies/controlled-plain-english/v1.md` (incl. `CPE-1`), 1 (`CPE-3b`) in `dba/tools/reviewer/v1.md`; this row cites the section broadly and does not depend on `CPE-3b`. |
| 2 | patterns/controlled-plain-english.md | 16 | STRUCTURAL-POINTER (sub-part) | "`dba-system.md`'s call-site map" | Sub-element of the CPE section (its "Call-site map" table). Maps to `dba/policies/controlled-plain-english/v1.md` (CPE-3a). |
| 3 | patterns/controlled-plain-english.md | 79 | GENERIC-MENTION | "throughout `dba-system.md`/`CLAUDE.md`/`pipeline-reviewer.md`" | Lists the file among several where an authority principle already applies; no section cited. |
| 4 | patterns/controlled-plain-english.md | 164 | GENERIC-MENTION | "Never reconstruct these rules from `dba-system.md`..." | Names the file as a non-fallback source; no structural dependency. |
| 5 | patterns/rust-project-structure.md | 16 | STRUCTURAL-POINTER (section) | "Implementation Profile" | Exact match. Maps cleanly to `dba/policies/implementation-profile/v1.md`. |
| 6 | patterns/rust-project-structure.md | 18 | STRUCTURAL-POINTER (section) | "Multi-Feature Architecture Synthesis Gate" | Exact match. Broad section pointer — see Part 2 note on this section's near-split (architecture-synthesis policy, plus `ARCH-GATE-14`/reviewer tool contract for the reviewer-coverage sub-paragraph). |
| 7 | patterns/rust-project-structure.md | 308 | STRUCTURAL-POINTER (section) | "Contract-to-Implementation Failure Boundary" | Exact match. Maps cleanly to `dba/doctrine/v1.md` (`FAILURE-BOUNDARY-1..5`, all `KEEP-IN-CORE`). |
| 8 | patterns/rust-project-structure.md | 368 | STRUCTURAL-POINTER (section) | "Multi-Feature Architecture Synthesis Gate" | Exact match. Same near-split section as row 6. |
| 9 | prompts/00a-solution-discovery.md | 208 | STRUCTURAL-POINTER (section) | "Default Advisory Review" section | Exact match. **This section is a genuine split** — see Part 2. Maps to `dba/doctrine/v1.md` (`REVIEW-6`) + `dba/policies/review/v1.md` (most rows) + `dba/tools/reviewer/v1.md` (`REVIEWER-TOOL-1/2`). |
| 10 | prompts/00c-onboarding.md | 20 | STRUCTURAL-POINTER (section) | "Implementation Profile" | Exact match. Maps cleanly to `dba/policies/implementation-profile/v1.md`. |
| 11 | prompts/00-session-start.md | 7 | WHOLE-FILE-LOAD | "Claude will read .codeos/dba-system.md" | Conditionally compatible — see Part 2 note on manifest cascade. |
| 12 | prompts/00-session-start.md | 14 | WHOLE-FILE-LOAD | "Read `.codeos/dba-system.md` now... state the 3 non-negotiable rules" | Conditionally compatible (load instruction); **also carries an unrelated pre-existing defect** — see Part 2 note (dba-system.md has 6 Non-Negotiable Rules, not 3). |
| 13 | prompts/00-session-start.md | 62 | STRUCTURAL-POINTER (section) | "Multi-Feature Architecture Synthesis Gate" | Exact match. Same near-split section as row 6. |
| 14 | prompts/00-session-start.md | 72 | STRUCTURAL-POINTER (section) | "Implementation Profile" | Exact match. Maps cleanly to `dba/policies/implementation-profile/v1.md`. |
| 15 | prompts/03b-architecture-synthesis.md | 9 | STRUCTURAL-POINTER (section) | "Multi-Feature Architecture Synthesis Gate" | Exact match. Same near-split section as row 6. |
| 16 | prompts/03b-architecture-synthesis.md | 55 | GENERIC-MENTION | "(recommended, not required — see `dba-system.md`)" | No section cited. |
| 17 | prompts/03b-architecture-synthesis.md | 66 | STRUCTURAL-POINTER (section) | "Multi-Feature Architecture Synthesis Gate section" | Exact match. Same near-split section as row 6. |
| 18 | prompts/03b-architecture-synthesis.md | 179 | STRUCTURAL-POINTER (section) | "Default Advisory Review" | Exact match. Same split section as row 9. |
| 19 | prompts/04-implement.md | 34 | STRUCTURAL-POINTER (sub-part) | "Multi-Feature Architecture Synthesis Gate" → "Verifying a `baseline_version` or `logical_design_version` reference." | **Verbatim match on the core identifying clause** — matches `ARCH-GATE-13`'s bolded lead-in exactly on "Verifying a `baseline_version` or `logical_design_version` reference"; the clarifying parenthetical "(live Stage 4 eligibility)" is not quoted here, which this report does not treat as a defect (see the granularity rule stated for rows 19/40/45 vs. the genuine drift at row 36). Maps cleanly to `dba/policies/architecture-synthesis/v1.md`. |
| 20 | prompts/04-implement.md | 59 | STRUCTURAL-POINTER (sub-part) | "Implementation Profile" → "Profile–Architecture Baseline consistency" | Exact match to `IMPL-PROFILE-7`'s bolded lead-in. Maps cleanly to `dba/policies/implementation-profile/v1.md`. |
| 21 | prompts/04-implement.md | 112 | STRUCTURAL-POINTER (section) | "Contract-to-Implementation Failure Boundary" | Exact match. Maps cleanly to `dba/doctrine/v1.md`. |
| 22 | prompts/04-implement.md | 167 | STRUCTURAL-POINTER (section) | "Contract-to-Implementation Failure Boundary" | Exact match. Maps cleanly to `dba/doctrine/v1.md`. |
| 23 | prompts/05-tests.md | 53 | STRUCTURAL-POINTER (section) | "Contract-to-Implementation Failure Boundary" | Exact match. Maps cleanly to `dba/doctrine/v1.md`. |
| 24 | prompts/10-arch-refine.md | 20 | STRUCTURAL-POINTER (section) | "Multi-Feature Architecture Synthesis Gate" | Exact match. Same near-split section as row 6. |
| 25 | prompts/codeos-self-dev.md | 7 | GENERIC-MENTION | "not by the downstream 9-stage DBA doctrine in `dba-system.md`" | Self-dev-governance context, confirmed (per Step 1's preliminary flag) — contrasts self-dev with downstream doctrine; no structural dependency. |
| 26 | prompts/codeos-self-dev.md | 130 | GENERIC-MENTION | "the generated project `CLAUDE.md` still loads `.codeos/dba-system.md`" | Self-dev-governance context, confirmed — this is guidance for *future* self-dev changes to write downstream-compatibility ACs; not itself a dependency. |
| 27 | prompts/codeos-self-dev.md | 269 | GENERIC-MENTION | `git diff -- dba-system.md scripts/codeos-review.sh` | Self-dev-governance context, confirmed — shell command example, path literal only. |
| 28 | prompts/codeos-self-dev.md | 323 | GENERIC-MENTION | "self-dev only change turns out to touch `dba-system.md`" | Self-dev-governance context, confirmed — no structural dependency. |
| 29 | prompts/pipeline-reviewer.md | 8 | STRUCTURAL-POINTER (section) | "Default Advisory Review" section | Exact match. Same split section as row 9. |
| 30 | prompts/verify-only.md | 99 | STRUCTURAL-POINTER (sub-part) | "Verification round-trip" | Exact match to `REVIEW-7`'s bolded lead-in. Maps cleanly to `dba/policies/review/v1.md`. |
| 31 | scripts/dba-init.sh | 194 | WHOLE-FILE-LOAD | "Tell Claude: 'Read .codeos/dba-system.md'" | Conditionally compatible — see Part 2 note on manifest cascade. |
| 32 | scripts/dba-init.sh | 215 | STRUCTURAL-POINTER (section) | "Implementation Profile" | Exact match. Maps cleanly to `dba/policies/implementation-profile/v1.md`. |
| 33 | scripts/dba-init.sh | 219 | STRUCTURAL-POINTER (section) | "Controlled Plain English Writing Discipline" | Exact match. **Near-split, not clean** — same as row 1; this row cites the section broadly and does not depend on `CPE-3b` (`dba/tools/reviewer/v1.md`). |
| 34 | scripts/tests/codeos-implement-tests.sh | 402 | GENERIC-MENTION | `"doctrine/dba-system.md:outside-stage-area"` | Confirmed (per Step 1's preliminary flag) — a path-traversal-attack test literal, not a real reference to the file at all. |
| 35 | templates/architecture-baseline.md | 16 | STRUCTURAL-POINTER (section) | "Multi-Feature Architecture Synthesis Gate" | Exact match. Same near-split section as row 6. |
| 36 | templates/architecture-baseline.md | 33 | STRUCTURAL-POINTER (sub-part) | "Verifying a `baseline_version` reference" | **Pre-existing citation drift — fails the verbatim-core-clause test.** Unlike rows 19/40/45, this citation drops "or `logical_design_version`" from the core identifying clause itself, not just the parenthetical — `ARCH-GATE-13`'s actual bolded lead-in is "Verifying a `baseline_version` *or `logical_design_version`* reference (live Stage 4 eligibility)". Not misleading in context (this template is baseline-only), but not verbatim by this report's own rule. Maps to `dba/policies/architecture-synthesis/v1.md` (`ARCH-GATE-13`). |
| 37 | templates/architecture-baseline.md | 43 | STRUCTURAL-POINTER (sub-part) | "Cohort and baseline versioning" | **Pre-existing citation drift, more significant** — actual bolded lead-in is "**Cohort, baseline, and logical design versioning.**" This citation drops "logical design" entirely. Pre-existing, unrelated to `v1` decomposition. Maps to `dba/policies/architecture-synthesis/v1.md` (`ARCH-GATE-10`). |
| 38 | templates/codeos-change.md | 72 | GENERIC-MENTION | "generated project CLAUDE.md still loads .codeos/dba-system.md" | Self-dev-governance context, confirmed — this is the template's own *example* acceptance-criterion row, not a real project's dependency. |
| 39 | templates/cohort-logical-design.md | 22 | STRUCTURAL-POINTER (section) | "Multi-Feature Architecture Synthesis Gate" | Exact match. Same near-split section as row 6. |
| 40 | templates/cohort-logical-design.md | 40 | STRUCTURAL-POINTER (sub-part) | "Verifying a `baseline_version` or `logical_design_version` reference" | **Verbatim match on the core identifying clause** — same rule as row 19; the parenthetical is not quoted but the full core clause matches `ARCH-GATE-13`'s bolded lead-in exactly. Maps to `dba/policies/architecture-synthesis/v1.md`. |
| 41 | templates/cohort-logical-design.md | 51 | STRUCTURAL-POINTER (sub-part) | "Cohort, baseline, and logical design versioning" | Exact match to `ARCH-GATE-10`'s bolded lead-in. Maps to `dba/policies/architecture-synthesis/v1.md`. |
| 42 | templates/feature-registry.yaml | 42 | STRUCTURAL-POINTER (section) | "Multi-Feature Architecture Synthesis Gate" | Exact match. Same near-split section as row 6. |
| 43 | templates/feature-registry.yaml | 52 | STRUCTURAL-POINTER (section) | "Multi-Feature Architecture Synthesis Gate" | Exact match. Same near-split section as row 6. |
| 44 | templates/feature-registry.yaml | 91 | STRUCTURAL-POINTER (section) | "Multi-Feature Architecture Synthesis Gate" | Exact match. Same near-split section as row 6. |
| 45 | templates/feature-registry.yaml | 109 | STRUCTURAL-POINTER (sub-part) | "Verifying a `baseline_version` or `logical_design_version` reference" | **Verbatim match on the core identifying clause** — same rule as row 19; full core clause matches `ARCH-GATE-13`'s bolded lead-in exactly (this citation also omits the parenthetical, consistent with rows 19/40). Maps to `dba/policies/architecture-synthesis/v1.md`. |
| 46 | templates/implementation-profile.yaml | 4 | GENERIC-MENTION | "Independent of the Architecture Synthesis Gate (see dba-system.md)" | No section name cited (just the bare file). |
| 47 | templates/project-CLAUDE.md | 9 | WHOLE-FILE-LOAD | "read `.codeos/dba-system.md` before doing any work" | Conditionally compatible — see Part 2 note. **This is the scaffolded root instruction every new project gets from `dba-init.sh`; the single highest-leverage WHOLE-FILE-LOAD reference in this report.** |
| 48 | templates/project-CLAUDE.md | 11 | WHOLE-FILE-LOAD | "Read `.codeos/dba-system.md` — authoritative DBA doctrine; read it first, in full" | Same as row 47 — this is the numbered-step form of the same instruction, in the same file. |
| 49 | templates/review-package.md | 133 | STRUCTURAL-POINTER (section) | "Controlled Plain English Writing Discipline" | Exact match. **Near-split, not clean** — same as row 1; this row cites the section broadly and does not depend on `CPE-3b` (`dba/tools/reviewer/v1.md`). |

**Row count**: 49, matching the per-`(file, line)` grep sweep exactly (AC1).

---

## Part 2 — Findings

### Finding A — "Default Advisory Review" is a genuine split section (rows 9, 18, 29)

Three prompt files cite `dba-system.md`'s "Default Advisory Review" section as a whole, expecting
a single coherent destination. Per `CHG-20260808-001`'s delta table, that section's 13 rows split
across **three** `dba/` files: `REVIEW-6` (doctrine invariant) → `dba/doctrine/v1.md`; the bulk of
the mechanics (`REVIEW-1` through `REVIEW-5d`, `REVIEW-7`, `REVIEW-8`) → `dba/policies/review/v1.md`;
the invocation syntax (`REVIEWER-TOOL-1`, `REVIEWER-TOOL-2`) → `dba/tools/reviewer/v1.md`. A future
activation cannot resolve these three pointers with a single replacement path — each needs either a
specific sub-pointer (matching the pattern rows 19/20/30/36/37/40/41/45 already use for other
sections) or a decision that broad section-level citations are no longer supported post-activation.
**This is evidence for the next Phase A sub-step (proving `DBA-1` semantically equivalent), not a
defect this change fixes** — no consumer file is edited here.

### Finding B — "Multi-Feature Architecture Synthesis Gate" is a near-split (rows 6, 8, 13, 15, 17, 19, 24, 35, 39, 42, 43, 44)

Twelve rows literally cite this section's name (confirmed by re-reading each row's own quoted
text, corrected per Step 3 review R3, which found the previous count didn't match its own row
list): eleven cite it broadly with no sub-part (rows 6, 8, 13, 15, 17, 24, 35, 39, 42, 43, 44), and
one (row 19) cites it together with a specific sub-part ("Multi-Feature Architecture Synthesis
Gate" → "Verifying a `baseline_version`..."). The section's content is overwhelmingly
`architecture-synthesis policy` (`ARCH-GATE-1` through `13`, `15`, `STAGE-TABLE-3`), with one
exception: `ARCH-GATE-14` ("Reviewer coverage") maps to `reviewer tool contract`. None of these 12
rows depends on `ARCH-GATE-14`'s content — the 11 broad citations don't need that specific
sub-part, and row 19's sub-part (`ARCH-GATE-13`) is a different one. Recorded as a near-split for
completeness, not as a blocking compatibility gap the way Finding A is.

**Separately — not part of the 12-row count above, since their own quoted text never names the
parent section**: rows 36, 40, 41, and 45 each cite an `ARCH-GATE-10`/`ARCH-GATE-13` sub-part
phrase directly, with no "Multi-Feature Architecture Synthesis Gate" text on their own matched
line (the section name appears only in a *different*, separately-counted row nearby in the same
file — e.g. row 39 for `templates/cohort-logical-design.md`). Their `dba/` mapping still lands in
`architecture-synthesis policy`, the same file this section's broad citations map to, but they are
not themselves section-name citations and are not counted in this Finding's 12.

### Finding C — WHOLE-FILE-LOAD compatibility is conditional on a cascade mechanism that doesn't exist yet (rows 11, 12, 31, 47, 48)

Five references (most importantly `templates/project-CLAUDE.md`'s two — the text every new
project's `CLAUDE.md` is scaffolded with) instruct reading `dba-system.md` in full, with no
section-specific dependency. These remain **correct as written** only if, whenever `dba-system.md`
becomes the thin active-configuration manifest the brief sketches, reading it "in full" still
means reading everything it names — i.e. the manifest text itself must instruct cascading into
every `dba/` component the active `DBA-N` names. The brief's manifest sketch already states this
("All components named in the active configuration are jointly authoritative and must be loaded
when applicable") but the actual manifest text, and the mechanism that enforces the cascade, do not
exist yet — this change does not design or build them. Recorded as a dependency for the future
activation sub-step, not a defect.

### Finding D — Two pre-existing citation-drift defects, unrelated to `v1` decomposition (rows 12, 36, 37)

- **Row 12** (`prompts/00-session-start.md:14`): asks the reader to "state the 3 non-negotiable
  rules," but `dba-system.md`'s "The Non-Negotiable Rules" section has stated **6** rules since
  before this sweep began (confirmed: `grep -c "^[0-9]\. \*\*" dba-system.md` → 6). This is a
  pre-existing prompt-doctrine mismatch, not caused by this UPG's work.
- **Row 36** (`templates/architecture-baseline.md:33`) and **Row 37** (`:43`): both cite
  paraphrased forms of `ARCH-GATE-13`'s and `ARCH-GATE-10`'s bolded lead-ins respectively, each
  dropping a clause from the real wording. Minor; not misleading given each template's own
  narrower context, but not verbatim citations either.

None of the three is caused by, or a precondition for, the `v1` component decomposition — each is
a citation-accuracy defect against the *current* `dba-system.md` monolith, found only because this
sweep read every citation against its target. Filed as **OUT-OF-SCOPE BACKLOG** (see Reconciliation)
— this change does not edit `prompts/00-session-start.md` or `templates/architecture-baseline.md`.

### Finding E — "Controlled Plain English Writing Discipline" is a near-split (rows 1, 33, 49)

Three rows literally cite this section's name, all broadly, with no sub-part combined in the same
citation (rows 1, 33, 49 — applying the same literal-text standard Finding B's correction
established). Per `CHG-20260808-001`'s delta table, 7 of the section's 8 non-`RETIRE` rows land in
`dba/policies/controlled-plain-english/v1.md`; the eighth (`CPE-3b`, the wrapper-injection
mechanics for `codeos-reviewer-task.md`) lands in `dba/tools/reviewer/v1.md`. None of these 3 rows
depends on `CPE-3b`'s content. Recorded as a near-split for completeness, not a blocking gap.

**Separately — not part of the 3-row count above, for the same reason rows 36/40/41/45 are
separate from Finding B's count**: row 2 cites the `CPE-3a` sub-part ("Call-site map") directly,
with no "Controlled Plain English Writing Discipline" text on its own matched line. Its `dba/`
mapping still lands in `controlled-plain-english policy`, the same file this section's broad
citations map to, but it is not itself a section-name citation.
