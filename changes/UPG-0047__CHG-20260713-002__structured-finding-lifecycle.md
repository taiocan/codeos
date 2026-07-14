# Self-Development Change: UPG-0047__CHG-20260713-002 — structured-finding-lifecycle

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0047
primary_feature_id: UPG-0047
change_id: CHG-20260713-002
slug: structured-finding-lifecycle
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0047
related_features: [UPG-0046, UPG-0045, UPG-0048, UPG-0001]
review_series: RVS__UPG-0047__CHG-20260713-002__S4
review_profile: PROFILE-3
review_state: REVIEWED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

## Change Intent

**Why (problem in the toolkit):**

`backlog/UPG-0047-structured-finding-lifecycle.md` leaves three open questions for the
implementer: finding record shape, who writes it, and resolution-tracking granularity. Before
proposing scope, I validated the parsing approach the brief leans toward against the **real**
corpus, not just the spec:

- `grep -c "^Finding:" reviews/codex/*.md` → 631 lines matching `^Finding:` across 302 files.
  **This raw grep count is superseded — see the Step 1 review's own non-blocking observation and
  the corrected figures in Implementation Notes below.** It counts every physical line starting
  with `Finding:` in the *whole file*, which includes the duplicate CLI-transcript echo each
  assessment file contains after its real answer (§ Implementation Notes). The corrected,
  real-region-only count established in Step 3 is **317 real `Finding:` lines across 305 files**.
  The qualitative conclusion this number supported — that the required output shape is regular
  enough to parse mechanically — held up, but the specific "631/zero gaps" figures were wrong and
  are corrected, not just re-asserted, below. Checking every `Classification:` value used in real
  output (excluding the packaged prompt template's own placeholder text, which every packet
  echoes back) → exactly the five canonical TRIAGE RULE labels, byte-for-byte. This supported
  Q2 resolved in favor of parsing existing output, not changing the prompt, per the brief's own
  preference — that conclusion is unaffected by the count correction.
- **`backlog/UPG-0001-feature-thread-traceability.md` already specifies the exact finding-id
  grammar**: `FND__REV__UPG-####__CHG-YYYYMMDD-NNN__S<N>__R<N>__NN` (2-digit sequence, example
  `FND__REV__UPG-0000__CHG-20260627-001__S4__R2__01`). Since `review_id` now exists mechanically
  (`UPG-0046`), `finding_id` reduces to `FND__<review_id>__<NN>` — inheriting `UPG-0046`'s own
  already-approved deviation (raw stage string, not `S<N>`) rather than introducing a new one.
- **Every change record already has a `fixes_findings: []` trace-header field** (see
  `templates/codeos-change.md`) that has sat unpopulated in every change record to date, because
  there was no mechanical `finding_id` to put in it. This directly resolves Q3 (resolution
  tracking) without inventing new mutable storage: a finding is "resolved" when a later,
  accepted change's `fixes_findings` list names its `finding_id` — self-reported by the acting
  agent at fix time (the brief's own cheaper, current-practice-matching option), zero new
  storage, and finally makes an existing dormant field meaningful.

**What changes:**

- `tools/reviewer/src/assessment.rs` (or a new `finding.rs` — decided at Step 3 based on which
  keeps `parse_review_output`'s existing scope cleanest) — new `parse_findings(raw_text: &str,
  review_id: &str) -> (Vec<Finding>, usize)` returning parsed findings plus a count of
  `Finding:` lines that did **not** match the full expected two-line shape (never silently
  dropped). Parsing is **keyword-anchored**, not naive `" / "` splitting: each field boundary is
  located by its literal prefix (`" / Severity: "`, `" / Classification: "`, `" / Why: "`,
  `" / Required action: "`), so a finding's own free-text summary containing a stray `/` cannot
  misalign the split — validated against the real corpus above, which does contain findings with
  `/` inside their prose (e.g. "R1/R2/R3", file paths).
- `Finding` struct (internal parse result — **not** identical to what gets serialized; see
  Step 2's "Guardrail 1 resolved" for the exact YAML subset): `finding_id`, `classification`
  (parsed verbatim — exactly the five TRIAGE RULE labels, unmodified vocabulary, per the brief's
  own guardrail), `severity`, `summary`, `required_action`, plus `evidence`/`why`/`scope_reason`
  parsed and held **only** for use in the malformed-block diagnostic (AC-9) and for byte-diffing
  the body against `raw.text` (AC-4) — never written to the assessment frontmatter. Only
  `finding_id`/`severity`/`classification`/`summary`/`acceptance_criterion`/`required_action`
  are serialized into YAML (Step 2's compact schema).
- `tools/reviewer/src/assessment.rs::write_assessment` — adds a `findings:` list (serialized from
  the compact struct subset above) and an `unparsed_findings_count: <N>` field to the **same**
  assessment YAML frontmatter `review_id` already lives in (additive, same file, no new artifact
  — matching `UPG-0046`'s approach and the existing `artifacts:`/`excluded_paths:` list-of-maps
  precedent already in that schema).
- `tools/reviewer/src/cmd/review.rs` — calls `parse_findings` once, alongside the existing
  `parse_review_output` call, passes the result into `write_assessment`.
- `docs/reviewer-artifact-schemas.md` — add `findings` (list) and `unparsed_findings_count`
  (integer) to the assessment YAML-frontmatter table (additive).
- `docs/reviewer-pipeline.md` §7 ("What a good review looks like") or a new small subsection —
  document that findings are now also machine-parsed into frontmatter, and that resolution is
  tracked via the existing `fixes_findings` trace-header field, not a mutable `status` anywhere.
- New unit tests (inside `assessment.rs`'s own `#[cfg(test)] mod tests`, for the same no-`[lib]`-
  target reason `UPG-0046` discovered) covering: single finding parses correctly; multiple
  findings in one assessment; the `SELF-REFERENCE / REVIEW-BOOKKEEPING` compound label parses as
  one classification, not two; a finding whose summary text contains a literal `/` still parses
  correctly (regression-guards the keyword-anchoring design); a malformed `Finding:` line
  increments `unparsed_findings_count` and does **not** abort — the assessment is still written
  and the review still completes; zero findings (e.g. a `NO OBJECTION` round with no findings
  raised) produces an empty `findings: []` list, not an error.

**Two design decisions, flagged for approval rather than assumed:**

1. **No `status`/`resolved_by` field is added to the finding record at all in this change.** The
   brief's illustrative sketch includes `status: open | resolved | waived | backlog | rejected`
   and `resolved_by: null`. I'm deliberately **not** implementing that field: an assessment file
   is a committed, durable, point-in-time snapshot (`docs/reviewer-pipeline.md` §4a) — at write
   time, no finding can yet be "resolved," so `status` would always read `open` in every
   committed file, encoding no real information and inviting someone to eventually mutate an
   already-committed file to update it (violating the append-only/durability guarantee). Instead,
   resolution is answered by a **query**, not a stored field: "is `finding_id` X named in any
   later, accepted change's `fixes_findings` list?" This uses infrastructure that already exists
   (the dormant trace-header field) rather than adding a new one that can't be safely kept
   truthful. No lookup/query tooling is built in this change — surfacing "open findings" is
   explicitly named as later, optional work in the backlog brief's own Related section.
2. **Findings live in the assessment file's frontmatter, not a new artifact.** Consistent with
   `UPG-0046`'s "additive to existing artifacts, no new file format" precedent. The brief's
   illustrative sketch shows a separate structure; this change treats that as the "smaller,
   legitimate outcome" the same way `UPG-0046` did for `ReviewRun`.

**Scope boundary — what stays the same:**

- The five-category TRIAGE RULE vocabulary and `prompts/codeos-reviewer-task.md`'s required
  output shape are **not** changed — parsing adapts to existing output, per the brief's explicit
  preference and out-of-scope note.
- No auto-resolution — nothing in this change infers a finding is fixed from a diff; resolution
  remains something an acting agent asserts, by naming the `finding_id` in `fixes_findings`.
- No dashboard, no new CLI subcommand, no query tool for "which findings are open" — out of
  scope per the brief, left for later, optional work.
- No change to `review`'s exit codes or fail-closed behavior for the review itself — a parsing
  gap is flagged (`unparsed_findings_count`), never fatal, mirroring `UNCLASSIFIED`'s existing
  posture for a malformed `LOG SUMMARY` line.
- No change to `scripts/codeos-review.sh` expected (confirmed, not assumed, at Step 4, per
  `UPG-0045`/`UPG-0046` precedent).
- No change to `CLAUDE.md` or `dba-system.md`.

**Class:** script-tooling
**Scope axis:** self-dev only
**Backlog item:** backlog/UPG-0047-structured-finding-lifecycle.md

---

## Acceptance Criteria

**Guardrail 1 resolved — compact frontmatter schema.** The human flagged that duplicating full
`Evidence`/`Why`/`Scope reason` prose into YAML risks bloated frontmatter. Resolution: the
structured entry carries only short, classification/triage-relevant fields; the full prose
**stays exactly where it already is** — in the assessment body, unstructured, unchanged. YAML
does **not** duplicate `Evidence:`, `Why:`, or `Scope reason:` at all (not even truncated — a
truncation heuristic is its own unresolved design question, avoided by simply not doing it):

```yaml
findings:
  - finding_id: FND__<review_id>__NN
    severity: High|Medium|Low
    classification: <exact TRIAGE RULE label, unmodified>
    summary: <the "Finding:" line's own text, verbatim>
    acceptance_criterion: AC-9        # best-effort: first "AC-\d+" match in summary, else omitted
    required_action: fix now|optional fix|backlog|reject
unparsed_findings_count: 0
```

**Guardrail 2 resolved — finding-id shape.** `FND__<review_id>__NN`, 2-digit zero-padded
sequence — matching `backlog/UPG-0001-feature-thread-traceability.md`'s own canonical example
(`FND__REV__UPG-0000__CHG-20260627-001__S4__R2__01`, 2-digit) exactly, not the 3-digit variant.
Deterministic: re-parsing the same assessment body must always assign the same `finding_id` to
the same finding (sequence = order of appearance in the raw text, stable because `raw.text` is
itself immutable once the assessment is written).

**Scope clarification carried into AC-6/AC-10 below:** this change does **not** build a
`fixes_findings` lookup/query tool (explicitly out of scope, per Step 1) — AC-6 verifies the
*invariant* that nothing in this change mutates an already-written assessment file, not that a
resolution-lookup feature exists. AC-10's test list accordingly covers `finding_id` determinism
and stability, not `fixes_findings` query semantics.

| # | Criterion | How it will be verified |
|---|---|---|
| AC-1 | Parser extracts all finding blocks matching the canonical reviewer format (`Finding:`/`Severity:`/`Classification:` line, then `Evidence:`/`Why:`/`Required action:` line). | Unit tests parse real historical assessment bodies (fixture strings copied from `reviews/codex/*.md`) and assert the expected count/content of extracted findings. |
| AC-2 | Only the five canonical classification labels are accepted verbatim: `IN-SCOPE BLOCKER`, `IN-SCOPE NON-BLOCKER`, `OUT-OF-SCOPE BACKLOG`, `REJECTED`, `SELF-REFERENCE / REVIEW-BOOKKEEPING`. | Unit test asserts each of the five parses to itself unmodified; a sixth, invented label is treated as a malformed/unparseable block (AC-9), not silently accepted. |
| AC-3 | Each parsed finding receives a deterministic `finding_id` derived from `review_id` + a stable ordinal. | Unit test: parsing the same fixture text twice yields identical `finding_id` values in the same order. |
| AC-4 | Structured findings are written into the assessment frontmatter **without changing the human-readable finding body** (the raw Codex text below the `---` closing marker is untouched). | Unit test: byte-diff the assessment body before/after this change's `write_assessment` — identical to `raw.text`, exactly as today. |
| AC-5 | No mutable `status`/`resolved_by` field is added to assessment files. | Code inspection: the `Finding` struct and the YAML-serialization code contain no `status`/`resolved_by` field; `docs/reviewer-artifact-schemas.md`'s updated table does not list one. |
| AC-6 | No code in this change ever re-opens an already-written assessment file for writing. Resolution is derived later, externally, via `fixes_findings` — not built or tested as a lookup feature in this change. | Code inspection: grep the diff for any `fs::write`/`OpenOptions::new().write` targeting an assessment path outside `write_assessment`'s single initial write call. |
| AC-7 | `prompts/codeos-reviewer-task.md`'s required output text is unchanged. | `git diff --stat -- prompts/codeos-reviewer-task.md` empty. |
| AC-8 | Existing review behavior and verdict parsing (`parse_review_output`, `LOG SUMMARY`/`EVIDENCE`/`HIGHEST-IMPACT UNCERTAINTY`) are unchanged. | Full test suite passes unchanged (171 baseline + new tests); `git diff` on `parse_review_output` itself is empty — findings parsing is a new, separate function. |
| AC-9 | A malformed/ambiguous `Finding:` line produces a clear diagnostic and is never silently dropped or silently counted as a complete structured finding — but never aborts the review. | Unit test: a hand-crafted malformed block (e.g. missing `Classification:`) increments `unparsed_findings_count` and triggers an `eprintln!` diagnostic naming the offending line; `write_assessment` still succeeds and the review still completes (matches `UNCLASSIFIED`'s existing non-fatal posture for a malformed `LOG SUMMARY`). |
| AC-10 | Tests cover: one finding, multiple findings in one assessment, all five classification labels (including the compound `SELF-REFERENCE / REVIEW-BOOKKEEPING` as one label, not two), zero findings (`NO OBJECTION` round → `findings: []`), a malformed block, a finding whose summary text itself contains a literal `/` (regression-guards keyword-anchored parsing over naive delimiter-splitting), and `finding_id` determinism across repeated parses. | Enumerated as unit tests in `assessment.rs`'s test module; each scenario above maps to exactly one test. |
| AC-11 | The corpus-validation claim is established with directly-shown evidence, not just asserted (Step 1's original 631/302/"zero gaps" figures were later found to double-count a duplicate transcript echo — see Implementation Notes — so this AC now also covers re-establishing the *corrected* figures, not only re-running the original command). | Step 4 Reconcile pastes the actual `cargo test` output for the corpus-regression test into the Reconciliation table, addressing the Step 1 reviewer's own non-blocking observation about unverifiable corpus claims. |

**Class note:** `script-tooling` — AC-7/AC-8 are the I/O-behavior-unchanged contract; AC-9 is the
fail-safe (never-fatal, never-silent) contract this class requires.

---

## Implementation Notes

All edits landed in `tools/reviewer/src/assessment.rs` (kept in the existing file rather than a
new `finding.rs` — `Finding`/`parse_findings`/`to_yaml_entry` sit naturally alongside
`ParsedReview`/`parse_review_output`, sharing no new cross-module surface), plus the minimal
wiring in `cmd/review.rs` and the two docs.

- **`Finding` struct + `parse_findings`** — implemented exactly as Step 2 specified: a
  keyword-anchored parser (never naive `" / "` splitting), scanning only the region before the
  first line-anchored `LOG SUMMARY:` (avoiding both the packaged-prompt placeholder text and the
  observed duplicate transcript echo — verified by a dedicated regression test).
- **`to_yaml_entry`** — serializes exactly the Step 2 compact schema
  (`finding_id`/`severity`/`classification`/`summary`/`acceptance_criterion?`/`required_action`);
  `evidence`/`why`/`scope_reason` are parsed (held on the struct, used for nothing but the
  malformed-block diagnostic and are never written to YAML) — verified by a dedicated test
  asserting the serialized string does not contain any of the three excluded fields' content.
- **`write_assessment`/`cmd/review.rs`** — `findings:` and `unparsed_findings_count:` added
  immediately after `review_id:` in the same frontmatter block; the body (`raw.text`, pushed
  verbatim) is completely untouched — same call, same position, as before this change.

**Significant discovery during Step 3 (materially changed the implementation from what Step 1
described):** Step 1/2 described parsing the *single* combined-line `Evidence: X / Why: Y /
Required action: Z` shape the current prompt asks for. Building the corpus-regression test
(new, beyond what Step 2 required — added because AC-11 asked for corpus evidence and a
synthetic-fixture-only test suite cannot actually prove corpus coverage) immediately falsified
that assumption: running the single-shape parser against every real historical assessment file
left **112 of 317** real finding lines unparsed. Investigating a sample showed Codex does not
reliably follow the "combine onto one line" instruction — the fields often land on three
separate lines instead — and, critically, **this is not a resolved historical format version**:
the exact same separate-line shape appears in this repo's own `UPG-0045` Step 3 R1 assessment
from earlier in *this session*. Fixed by extending the parser to accept three real, permanently-
supported shapes (not one current + two legacy): combined-one-line, separate-line, and an
even-earlier combined-with-`Scope-reason` variant. This reduced unparsed findings 112 → 74 → 23
(of 317). The residual 23 trace to the project's earliest bootstrap sessions (including a
4-label triage era, before the current 5-category rule existed) and do not share one common
recurring shape — extending further would be unbounded scope creep for non-recurring value. The
fail-closed guardrail (flag, never silently drop) holds regardless: all 23 are counted in
`unparsed_findings_count`, none are lost or misrepresented.

**AC-11 evidence (re-established with direct output, per Step 1's evidence-C gap):**
```
$ cargo test --bin codeos-reviewer parse_findings_corpus_regression_check
test assessment::tests::parse_findings_corpus_regression_check ... ok
```
Precise figures at time of writing: 317 real `Finding:` lines across 305 assessment files;
294 parsed (92.7%); 23 unparsed (7.3%) — all individually `eprintln!`-flagged, all counted, none
silently dropped. The corpus test asserts an unparsed-rate ceiling (15%) rather than an exact
count, so it tolerates this known, documented residual while still catching a future regression
(a new *systematic*, recurring shape going unsupported).

**Test coverage (AC-10):** one finding; multiple findings with stable ordered ids; all five
canonical classifications including the compound `SELF-REFERENCE / REVIEW-BOOKKEEPING` label;
zero findings (`findings: []`); a malformed block (counted, not dropped); a finding whose summary
contains a literal `/` (regression-guards keyword anchoring — using a real string from this
session's own `UPG-0046` history); `finding_id` determinism across repeated parses; the
duplicate-transcript-echo non-double-counting case; the YAML-exclusion assertion; and the full
corpus regression check. 48 unit tests now pass (was 38), 181 total across the suite (was 171),
zero regressions.

No out-of-scope changes were introduced. The three-shape parser extension is a deeper
implementation of the *same* Step 1/2 goal ("parse existing reviewer output"), not new scope —
discovered by testing against reality rather than assumed from the spec.

**R1 fixes (Step 3 review):** two real bugs, both fixed structurally.

1. **AC-2 violation — no classification allow-list.** `parse_findings` captured
   `Classification:` with a generic `(.+?)` group and never validated it against the five
   canonical TRIAGE RULE labels — an invented sixth label would have been silently accepted as a
   valid finding, directly contradicting AC-2. Fixed: added a `CANONICAL_CLASSIFICATIONS`
   allow-list constant; a non-canonical value is now treated as malformed (counted in
   `unparsed_findings_count`, never silently accepted or dropped). New test
   `parse_findings_rejects_non_canonical_classification` proves this.
2. **Internal contradiction — stale corpus figures.** Step 1's "Why" section still stated the
   original `631 finding blocks / 302 files / zero gaps` claim after Implementation Notes had
   already superseded it with the corrected `317 / 305 / 23 unparsed` figures discovered while
   building the corpus-regression test. Fixed: Step 1 now explicitly marks the original figure as
   superseded and explains why (it double-counted the duplicate transcript echo), rather than
   leaving two silently conflicting numbers in the same artifact. AC-11 in Step 2 updated to
   match.

49 unit tests now pass (was 48), 182 total across the suite (was 181), zero regressions.

---

## Reconciliation

**Headline claim, stated precisely (not overclaimed):** the parser supports all canonical
finding blocks plus the three recurring real-world variants found via corpus regression testing.
Current corpus coverage is **294/317 findings parsed (92.7%)**; **23/317 remain documented,
unsupported historical long-tail cases** (individually flagged via `unparsed_findings_count`,
never silently dropped). This is not "all historical findings parse" — that claim would be false.

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| AC-1 | Parser extracts all finding blocks matching the canonical reviewer format | PASS | `parse_findings_single_finding`, `parse_findings_multiple_findings_get_stable_ordered_ids`; corpus regression confirms 294/317 real blocks extracted |
| AC-2 | Only the five canonical classification labels accepted | PASS (fixed at Step 3 R1) | `CANONICAL_CLASSIFICATIONS` allow-list; `parse_findings_accepts_all_five_canonical_classifications` + `parse_findings_rejects_non_canonical_classification` |
| AC-3 | Deterministic `finding_id` from `review_id` + stable ordinal | PASS | `parse_findings_deterministic_across_repeated_parses`; format `FND__<review_id>__NN`, 2-digit, matching `UPG-0001`'s canonical example |
| AC-4 | Structured findings written to frontmatter without changing the human-readable body | PASS | `git diff` on `assessment.rs` shows zero changes to the `content.push_str(&raw.text)` line (no diff hit at all — confirmed by direct grep); body is `raw.text` verbatim, exactly as before this change |
| AC-5 | No mutable `status`/`resolved_by` field | PASS | `Finding` struct and `to_yaml_entry()` contain no such field; `docs/reviewer-artifact-schemas.md`'s updated table doesn't list one |
| AC-6 | No re-opening of an already-written assessment for writing; resolution derived externally via `fixes_findings`, not built as a lookup feature here | PASS | Code inspection: `write_assessment` is called exactly once per review round (unchanged call site count); no new `fs::write`/`OpenOptions::write` targeting an assessment path added |
| AC-7 | `prompts/codeos-reviewer-task.md` unchanged | PASS | `git diff --stat` empty |
| AC-8 | Existing review behavior/verdict parsing unchanged | PASS | 182/182 tests pass; `parse_review_output` itself has zero diff |
| AC-9 | Malformed/ambiguous blocks produce a clear diagnostic, never silently dropped, never abort | PASS | Every rejection path (`Finding:` header mismatch, non-canonical classification, missing Evidence/Why/Required-action) has its own `eprintln!` + `unparsed_count += 1`; `parse_findings_malformed_block_counts_unparsed_never_drops_silently` |
| AC-10 | Test coverage per the stated list | PASS | 11 dedicated `parse_findings_*`/`finding_yaml_*` tests (see Implementation Notes), covering every named scenario plus the corpus regression check |
| AC-11 | Corpus-validation claim re-established with directly-shown evidence | PASS | `cargo test --bin codeos-reviewer parse_findings_corpus_regression_check` → `ok`; precise figures: 317 real `Finding:` lines / 305 files, 294 parsed, 23 unparsed (7.3%, within the test's 15% ceiling) — see Implementation Notes for the full breakdown and the Step 1 figure correction |

**Consistency sweep (grep):**
- `docs/reviewer-artifact-schemas.md` and `docs/reviewer-pipeline.md` §4g both describe the same
  compact schema and the same "no mutable status field" rule — no drift between the two.
- No stale references to the old (superseded) "631/302/zero gaps" figure remain outside the
  explanation of why it was wrong (Step 1's own text now marks it superseded in place).
- `git diff --stat -- scripts/codeos-review.sh CLAUDE.md dba-system.md
  tools/reviewer/Cargo.toml tools/reviewer/Cargo.lock` — all empty. No shim, doctrine, or
  dependency changes.
- No new file created outside the declared set; `git status --short` matches Step 1's "What
  changes" exactly (plus the usual Feature Thread/status bookkeeping already flagged as such
  since `UPG-0044`).

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| Step 2 R1: Step 1/Step 2 schema description contradiction (evidence/why/scope_reason serialization) | IN-SCOPE BLOCKER | Fixed — Step 1 text aligned with Step 2's resolved compact schema |
| Step 3 R1: AC-2 classification allow-list missing | IN-SCOPE BLOCKER | Fixed — `CANONICAL_CLASSIFICATIONS` allow-list added, with a dedicated rejection test |
| Step 3 R1: internal contradiction between Step 1's original and Implementation Notes' corrected corpus figures | IN-SCOPE BLOCKER | Fixed — Step 1 figure explicitly marked superseded with explanation, AC-11 updated to match |
| Mid-Step-3 discovery: single-shape parser assumption falsified by corpus testing (112/317 unparsed) | Resolved within scope, not a triaged "finding" — a legitimate deepening of the same Step 1/2 goal, not new capability | Extended to three real, recurring shapes; residual 23/317 documented as bounded historical long-tail, not chased further |

All findings across all four reviewed steps are resolved. No OUT-OF-SCOPE BACKLOG, REJECTED,
SELF-REFERENCE, or REVIEW-BOOKKEEPING findings arose in this change.

**Stack/dependency reconciliation:** Not applicable — `Cargo.toml`/`Cargo.lock` unchanged
(confirmed above); no watched-file reconciliation report required.

**Follow-up implication for `UPG-0047`'s own dependents:** `UPG-0047`'s own backlog Related
section named `UPG-0048` (event sourcing) as a future, non-required consumer of structured
findings — unaffected by this change either direction. No other feature currently depends on
this one completing.
