# Self-Development Change: UPG-0057__CHG-20260726-003 — controlled-plain-english-writing-discipline

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0057
primary_feature_id: UPG-0057
change_id: CHG-20260726-003
slug: controlled-plain-english-writing-discipline
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0057
related_features: [UPG-0056]
review_series: RVS__UPG-0057__CHG-20260726-003__S4
review_profile: PROFILE-4
review_state: REVIEWED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round (which does not exist until after the packet is built). Reference the stable review
SERIES (review_series) + review_state; exact rounds live only in reviews/review-log.md and
reviews/codex/*. See prompts/codeos-self-dev.md → "Feature Thread & IDs" / "Self-Reference Boundary". -->


## Change Intent

**Why (problem in the toolkit):**

AI-generated prose across Codeos — both downstream DBA artifacts and Codeos's own self-development
briefs/change records — has no documented discipline distinguishing plain communication from
specification-grade precision, protecting literal/quoted content from paraphrase drift, or keeping
reviewer prose advisory and separate from specification writing. A detailed 15-section "Controlled
Plain English" writing guideline was supplied as the source, scoped by the human to cover both
downstream doctrine and self-development, with an explicit requirement for a real, human-controlled
enable/disable switch — now available via `UPG-0056` (Optional Mechanism Status Convention,
`COMPLETE`), which this change consumes rather than redefines.

This design was refined across eight adversarial planning-review rounds (see
`/home/rimo/.claude/plans/calude-consider-this-inputs-steady-pnueli.md`, approved) before this
change record was opened: a four-layer content split (A: plain communication, always advisory;
B: specification/planning precision, toggle-gated, a *generation discipline* not a review-compliance
regime; C1: existing literal-protection authority, always active; C2: new literal-protection rules,
toggle-gated; D1: reviewer/reporting integrity, always active; D2: plain review prose, toggle-gated);
an explicit call-site-to-configuration map so no shared reviewer template ever guesses which project's
status file applies; a fixed status-injection contract for `codeos-reviewer-task.md`; an
enabled-but-pattern-unavailable rule; a one-sentence non-retroactivity rule (no stamps, no
versioning); and a 15-section traceability matrix with corrected DBA ownership (observable behavior
and edge cases belong primarily to Stage 2 Contract, not Stage 1 Intent or Stage 5 Tests).

This is **CHG-A**: the downstream-doctrine half. A separate **CHG-B** (self-dev adoption) follows,
gated to start only after this CHG lands, since it references the pattern file this CHG creates.

**What changes:**

- `patterns/controlled-plain-english.md` (new) — the pattern: four layers (A/B/C1+C2/D1+D2),
  the reviewer model (no separate "CPE violation" category, no historical-compliance audit),
  non-retroactivity (one sentence), the softened Requirement-Set/Data-Interface-Spec mapping, the
  15-section adaptation matrix with corrected Stage 2 Contract ownership for observable
  behavior/edge cases, and the enabled-but-pattern-unavailable rule. Ordinary pattern skeleton — no
  version-declaration exception (that only existed under the superseded resolver-based design).
- `dba-system.md` — new short doctrine section: what the mechanism is, the exact one-line config
  grammar (per `UPG-0056`) and its downstream location (`architecture/controlled-plain-english.yaml`,
  project-local — not through the `.codeos` symlink; the pattern file itself is reached via
  `.codeos/patterns/controlled-plain-english.md`), the call-site map, that Layer A/C1/D1 are not new
  mandatory rules and B/C2/D2 are the only toggle-gated parts, the reviewer model, "no new Stage ID"
  statement. One new File Layout line.
- `scripts/dba-init.sh` — scaffolds `architecture/controlled-plain-english.yaml` containing exactly
  `status: enabled` (changed from `disabled` during Step 3, per explicit human direction — see
  Implementation Notes).
- `prompts/00-session-start.md` — new step, silent when the config file is absent (no new message
  for existing/legacy projects); reports enabled/disabled when present; stops on a malformed file.
- `prompts/01-intent.md`, `02-contract.md`, `03-event-schema.md`, `03b-architecture-synthesis.md` —
  check line (read the config per the four-outcome table from `UPG-0056`); Layer B applied when
  enabled (Contract carries the reassigned observable-behavior/edge-case ownership); Layer C1 always
  protects Event Schema identifiers; enabled-but-pattern-unavailable rule applied.
- `prompts/04-implement.md` — check line; Review Package free text uses factual reporting, not
  Layer B.
- `prompts/05-tests.md`, `06-observe.md` — check line; factual reporting profile (these verify/
  observe, they don't specify).
- `prompts/08-replay.md` — check line; factual reporting + Layer D1's evidence-vs-inference
  separation.
- `prompts/09-refine.md`, `10-arch-refine.md` — check line; Layer B applied when enabled.
- `templates/review-package.md` — one shared note in "Usage Notes for Stage Prompts" (reaches both
  the Stage 4-5 and Stage 7 format variants from one edit): D1 always applies to review prose, D2
  only when enabled.
- `prompts/07-reconcile.md`, `prompts/pipeline-reviewer.md` — check line reading
  `architecture/controlled-plain-english.yaml` directly; D1 always + D2 when enabled; findings use
  the reviewer model (no CPE-specific violation category, no historical audit). `pipeline-reviewer.md`'s
  "output ends with Observations" contract stays byte-identical.
- `prompts/codeos-reviewer-task.md` (shared reviewer infrastructure, consumed by both CHG-A and
  CHG-B — see Scope note below) — made configuration-neutral: it recognizes the fixed line
  `Controlled Plain English status for this review: enabled|disabled` wherever it appears among the
  reviewed artifacts; it never reads a config file itself. D1 always applies; D2 is driven by that
  value when present. The machine-parsed footer (`LOG SUMMARY`/`EVIDENCE`/`HIGHEST-IMPACT
  UNCERTAINTY`) stays byte-identical.

  **Interim state, explicitly not final (corrected after human review of this CHG's Step 3):** this
  CHG does not itself deliver *automatic* inclusion of that line — `tools/reviewer` embeds whatever
  artifact paths it is given and has no code path that reads a config file on an invoker's behalf.
  Within CHG-A alone, someone must manually pass a file containing that line as one of the reviewed
  artifacts, or D2 silently never activates. **This is accepted as CHG-A's scope (pattern and
  prompt-wiring only) — it is not accepted as this discipline's final operating model.** A separate
  change, **CHG-B**, gives `scripts/codeos-review.sh` (the bash wrapper — not `tools/reviewer`,
  which stays unchanged) the job of resolving the status automatically before invoking the reviewer,
  for both the downstream and self-development branches of that one shared script, and of
  establishing that wrapper as the supported entry point so a direct `codeos-reviewer` invocation
  cannot silently bypass it. **`UPG-0057` is not complete until `CHG-B` lands.**
- `backlog/UPG-0057-controlled-plain-english-writing-discipline.md` — already exists; updated as
  this CHG progresses.

**Lifecycle bookkeeping (standard, not substantive scope):** `status/self-development.md` gains this
CHG's row as Step 1 activates. `backlog/features.md`/`status/roadmap.md` already carry `UPG-0057`'s
row (registered earlier).

**Scope boundary — what stays the same:**

Anything not named above is in scope for no change. In particular, this change does **not**: alter
`UPG-0056`'s own convention beyond consuming it (no resolver, stamp, or versioning mechanism of any
kind); add any new Stage ID; add any new Non-Negotiable Rule; add any new mandatory human-approval
gate; touch `scripts/codeos-review.sh`, `CLAUDE.md`, `prompts/codeos-self-dev.md`, or
`config/writing-discipline.yaml` (all `CHG-B`'s responsibility, not started until this CHG lands);
touch `00a-solution-discovery.md`, `00b-feature-brief.md`, `00c-onboarding.md`, `00-session-end.md`,
`reviewer-automated.md`, or `verify-only.md` (deliberately deferred, smaller administrative/
automation prompts, a future follow-up).

**Cross-scope note:** `prompts/codeos-reviewer-task.md` is shared reviewer infrastructure used by
both downstream reviews and self-development reviews (via `scripts/codeos-review.sh`, edited in
`CHG-B`). It is edited here in `CHG-A` — one file's worth of configuration-neutral wiring doesn't
justify a third CHG — but its scope axis is stated honestly as **shared, consumed by both CHGs**,
not silently folded into "downstream doctrine only."

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** `backlog/UPG-0057-controlled-plain-english-writing-discipline.md`

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | `patterns/controlled-plain-english.md` states all four layers (A always advisory; B toggle-gated, a generation discipline not a review-compliance regime; C1 always active with cited existing authority, C2 toggle-gated as new rules; D1 always active, D2 toggle-gated) with no layer's toggle status ambiguous. | Read-through. |
| 2 | The reviewer model is stated explicitly: no separate "CPE violation" finding category; no historical-compliance audit; a cardinality/meaning change is reported as an ordinary Contract/Schema-authority finding, not a CPE-specific one. | Read-through of the pattern's Reviewer Model section. |
| 3 | The pattern file states non-retroactivity as this exact self-contained rule: "Enabling this mechanism affects only text generated or revised while it is enabled. It does not invalidate previously approved artifacts." No stamp field, version field, or per-artifact provenance mechanism of any kind exists in any file this CHG touches. | Read-through of the pattern file for this exact sentence; grep across all touched files for `stamp`, `provenance`, or a version field name finds none introduced as an actual mechanism. |
| 4 | The pattern file contains a complete 15-row adaptation matrix, one row per source concern below, each with a stated Codeos treatment and disposition (Retained / Reused / Reinforced / Adapted) — no row omitted: (1) identify the requested artifact type, (2) plain-but-exact language, (3) preserve important meaning, (4) do not invent missing information, (5) precise normative modal verbs, (6) explicit scope/quantity, (7) condition patterns, (8) observable behaviour definition — assigned to **Stage 2 Contract** as primary owner, not Stage 1 Intent, (9) separate requirements from design, (10) artifact-appropriate structure, (11) edge-case consideration — assigned to **Stage 2 Contract** as primary owner (Stage 5 Tests verifies, it does not decide), (12) measurable quality requirements, (13) verification-method statements, (14) a trailing "Precision Check" section — explicitly *not* implemented as one merged section, split across each stage's existing distinct disclosure fields instead, (15) honest uncertainty / no false completeness. | Read-through of the matrix table against this exact 15-item list; every row present, items 8 and 11 specifically assigned to Stage 2 Contract. |
| 5 | Every touched Stage/prompt file's check line reads its config file per `UPG-0056`'s exact four-outcome table (absent→disabled, exact `disabled`→disabled, exact `enabled`→enabled, anything else→configuration error) — no prompt re-implements or re-derives that grammar itself. | Read-through of each touched prompt's check line. |
| 6 | Enabled-but-pattern-unavailable rule applied everywhere the pattern is consulted: if `status: enabled` but `patterns/controlled-plain-english.md` (or its `.codeos/` symlink path downstream) is missing/unreadable, the prompt stops and reports a CPE pattern-access error rather than falling back to memory or proceeding as disabled. | Read-through of each check line for this exact three-way branch. |
| 7 | The call-site map is honored exactly: Stage 1-10 prompts and `pipeline-reviewer.md` read `architecture/controlled-plain-english.yaml` directly (project-local, not through `.codeos`); `codeos-reviewer-task.md` reads neither file itself and instead recognizes the fixed status line wherever it appears among the reviewed artifacts. This CHG's own text states plainly that automatic delivery of that line is **not yet built** — it is `CHG-B`'s job — and that manual inclusion within `CHG-A` alone is an accepted-scope interim state, not a claimed final behavior. | Read-through of each call site's check line against the map, and of the interim-state disclosure. |
| 8 | The status-line format `codeos-reviewer-task.md` recognizes is exact: `Controlled Plain English status for this review: enabled` or `...: disabled`; the line never touches the machine-parsed footer (`LOG SUMMARY`/`EVIDENCE`/`HIGHEST-IMPACT UNCERTAINTY`). No file in this CHG claims automatic packet-assembly delivers that line — the interim-state note names `CHG-B` as the change that will. | Read-through of `codeos-reviewer-task.md`'s wiring; confirm every mention of this line's delivery is honestly scoped as manual-within-CHG-A, automatic-via-CHG-B. |
| 9 | Layer B does not apply to Implementation Notes / implementation evidence (`04-implement.md`'s Review Package free text) or to Stages 5/6/8 — these use factual reporting instead. | Read-through confirming these four files explicitly state factual reporting, not Layer B. |
| 10 | `07-reconcile.md` and `pipeline-reviewer.md` apply D1 always, D2 only when enabled — never the reverse, never both gated. | Read-through. |
| 11 | `templates/review-package.md`'s one shared note reaches both the Stage 4-5 and Stage 7 format variants from a single edit. | Read-through confirming the note sits in the shared "Usage Notes for Stage Prompts" section, not duplicated per variant. |
| 12 | Session-start behavior is deterministic: absent → silent; `disabled` → reports disabled; `enabled` → reports enabled; malformed → stops and reports a configuration error. No "may optionally" language. | Read-through of `00-session-start.md`'s new step. |
| 13 | `scripts/dba-init.sh` scaffolds `architecture/controlled-plain-english.yaml` containing exactly `status: enabled` (changed from `disabled` by explicit human direction during Step 3) — nothing else — and this default is stated consistently everywhere it's documented (`dba-system.md`'s Activation paragraph and File Layout entry, this change record, `dba-init.sh`'s own closing next-steps text). | Read-through / live scratch run; grep confirms no remaining `status: disabled`-as-default claim for this file anywhere in this CHG's touched files. |
| 14 | No new Stage ID, no new Non-Negotiable Rule, no new mandatory human-approval gate. | `git diff` on the stage table and Non-Negotiable Rules sections of `dba-system.md` shows no change. |
| 15 | **Downstream-compatibility:** every new cross-reference this CHG introduces resolves to a real path; a downstream project's `.codeos` symlink still reaches `patterns/controlled-plain-english.md`; `dba-system.md`'s existing 9-stage substance and File Layout are otherwise untouched except the one new line. | Grep/`ls` confirming referenced paths exist once Step 3 lands; `git diff` shows only additions to `dba-system.md`. |
| 16 | **Protected interface fragments stay byte-identical:** `pipeline-reviewer.md`'s "output ends with the Observations section" contract and `codeos-reviewer-task.md`'s three machine-parsed footer lines are unchanged by this CHG. | `git diff` on those two files shows those specific fragments untouched, not merely "the file still works." |
| 17 | **Cross-reference integrity sweep** (required for `downstream-doctrine` class): no touched file retains stale references to the superseded resolver-based CPE design (activation modes, result codes, provenance stamps) discussed during planning. | Grep across all touched files for resolver/stamp/version vocabulary; none expected as an actual mechanism (negation prose describing what was deliberately *not* built is fine, per the same distinction established in `UPG-0056`). |
| 18 | **The pattern file's own text states each of the following rules explicitly** (this AC checks what the doctrine says, not a provable guarantee about future generation behavior, consistent with Layer B being an advisory generation discipline, not an enforcement mechanism): an unresolved quantity is recorded as `[TBD]`, never invented; Stage 1 Intent output excludes implementation/design detail; an exact quantifier or technical term is not to be loosened under Layer B/C1/C2; a false-completeness claim is not to be made; a recommendation is not to be phrased as a requirement; reviewer authority is described as advisory throughout Layer D1/D2; refinement prompts (`09-refine.md`/`10-arch-refine.md`) apply Layer B on every regeneration with no stated carve-out exempting a refinement pass from the same discipline applied at first generation. | Read-through of the pattern file confirming each of these seven rules is stated in its text, and that refinement prompts' check lines contain no such carve-out. |

---

## Implementation Notes

Eighteen files touched (1 new, 17 edited — recounted precisely, correcting an earlier miscount):
`patterns/controlled-plain-english.md` (new — four layers, reviewer model,
non-retroactivity, softened Requirement-Set mapping, 15-section adaptation matrix,
enabled-but-pattern-unavailable rule), `dba-system.md` (new "Controlled Plain English Writing
Discipline" section, call-site map, Artifact Classification row, File Layout lines), `scripts/dba-
init.sh` (new step 9, scaffolds `architecture/controlled-plain-english.yaml` at `status: enabled`,
renumbering old steps 9-12 to 10-13), and check-line wiring in `00-session-start.md` (new step 3f,
deterministic, silent on absence), `01-intent.md`, `02-contract.md`, `03-event-schema.md`,
`03b-architecture-synthesis.md`, `04-implement.md` (factual reporting, not Layer B), `05-tests.md`/
`06-observe.md` (factual reporting), `08-replay.md` (factual reporting + D1), `09-refine.md`/
`10-arch-refine.md` (Layer B, no refinement carve-out), `templates/review-package.md` (one shared
D1/D2 note reaching both format variants), `07-reconcile.md`/`pipeline-reviewer.md` (D1 always, D2
when enabled), and `codeos-reviewer-task.md` (configuration-neutral).

**Design correction found during implementation (disclosed, not silently folded in):** Steps 1-2's
"status-injection contract" language implied `codeos-reviewer-task.md` automatically *receives* an
injected status line from "whichever caller built its packet." Implementing this revealed that
`codeos-reviewer-task.md` is embedded into the review packet as static file content by
`tools/reviewer/src/packet.rs` (Rust) — there is no code path that dynamically reads a project's
config file and injects a line on the invoker's behalf, and adding one would require new Rust code,
directly contradicting this discipline's "no code" design (the same principle `UPG-0056` was
simplified down to). Resolved by redefining this as a **manual step**: whoever invokes
`codeos-reviewer review` includes the status line as one of the reviewed artifacts, exactly like
any other file — using the reviewer tool's *existing* "embed whatever paths you're given" capacity,
not a new automatic feature. `dba-system.md`, this change record (Change Intent + ACs 7-8), and
`codeos-reviewer-task.md` itself are all corrected to say this explicitly rather than imply
automation. No Rust code touched by this CHG.

No other out-of-scope items discovered.

**Post-R1-review fixes:** three real issues found and fixed: (1) the pattern's "Consulted by" line
claimed `codeos-reviewer-task.md` as a pattern consumer subject to the enabled-but-pattern-
unavailable rule, but that file never reads the pattern or any status file itself — it only reacts
to a manually-included status line — so it genuinely cannot perform that check; corrected the
"Consulted by" line and added an explicit carve-out to the rule's own text; (2) `dba-system.md`'s
File Layout said the concrete `controlled-plain-english.yaml` file is "none by default," directly
contradicting the same document's own "Activation" paragraph and `scripts/dba-init.sh`, both of
which — *at that point in Step 3, before the later enabled-default revision below* — scaffolded it
at `status: disabled` for every new project; fixed the File Layout entry to match. (This default
was subsequently changed to `status: enabled`; see "Post-R2-approval revision" below — this
sentence is preserved as an accurate historical record of the R1 fix, not a claim about the current
default.) (3) "Thirteen files touched" undercounted the actual 18 files named in the same
sentence — corrected.

**Post-R2-approval revision (explicit human direction):** after Step 3's R2 NO OBJECTION, the human
directed that the scaffolded default change from `status: disabled` to `status: enabled` —
downstream projects now get the discipline active immediately after `dba-init.sh` runs, and turning
it off is the explicit action, not turning it on. This does not change the mechanism itself (the
Optional Mechanism Status Convention, the four-outcome table, "missing still means disabled" as
that convention's own fallback) — only the one value `scripts/dba-init.sh` writes, and every place
that value is documented: `dba-system.md`'s "Activation" paragraph, its File Layout entry,
`dba-init.sh`'s own closing next-steps text, this AC, and this Change Intent. Re-review requested
for this content change before the human's final Step 3 approval.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1-6, 9-18 | (unchanged since Step 3 R2 NO OBJECTION) | PASS | See Step 3's review history; re-confirmed by direct read-through and grep during this Reconcile pass. |
| 7 | Call-site map honored; interim-state disclosed | PASS | `dba-system.md`'s call-site map row and this change record's "Interim state" paragraph both state plainly that automatic delivery is `CHG-B`'s job, not claimed here. |
| 8 | Status-line format exact; no automatic-delivery overclaim | PASS | `codeos-reviewer-task.md:53-61` recognizes the line without asserting how it arrives; this change record's AC8 and interim-state note are explicit that `CHG-A` alone does not deliver it automatically. |
| 13 | `dba-init.sh` scaffolds `status: enabled` consistently | PASS | `scripts/dba-init.sh:147-148`, `dba-system.md`'s Activation paragraph and File Layout entry, and this change record's file-list sentence all say `enabled`; the one remaining literal `status: disabled` text (in the Post-R1-fixes note) is explicitly marked as a historical record of what was true earlier in Step 3, not a claim about the current default — verified by reading that sentence's own qualifying clause, not by grep alone (a plain grep for the string would still find it; that is expected and correct). |

**Consistency sweep:** grep confirms no remaining claim that automatic status delivery is
"rejected" or unwanted (that framing existed only transiently between Step 3 R2 and this
Reconcile pass, and has been replaced throughout with the interim-state / `CHG-B` framing).
`dba-system.md`'s and `codeos-reviewer-task.md`'s existing text describing *recognition* of the
status line (not its delivery mechanism) required no changes — both were already neutral about
manual vs. automatic delivery. Full `cargo test --release --manifest-path
tools/reviewer/Cargo.toml` suite re-run: **182 tests, 0 failures** — expected, since `tools/reviewer`
is untouched by this CHG and stays untouched by the planned `CHG-B`.

**Findings scope-triage:** all findings raised across Steps 1-3 (8 total, listed in the backlog
Feature Thread) were IN-SCOPE BLOCKER and fixed inline within this same CHG. The one substantive
post-approval item — that manual status-line inclusion should not be accepted as this discipline's
final operating model — came from explicit human architectural direction (not a Codex finding) and
is resolved by scoping `CHG-A` honestly (pattern and prompt-wiring only) and explicitly deferring
automatic delivery to a required `CHG-B`, tracked as a **Follow-up Feature is not appropriate here
— `CHG-B` is a required completion dependency of this same `UPG-0057`, not an optional follow-up**
(see backlog brief's "Related" section).

**Completion statement:** `CHG-A` is accepted for its stated scope — the Controlled Plain English
pattern and its consumer wiring across 18 files. `UPG-0057` as a whole is **not** complete until
`CHG-B` (automatic status delivery in `scripts/codeos-review.sh`, covering both the downstream and
self-development branches of that shared wrapper, with `tools/reviewer` unchanged) reaches
`COMPLETE`.
