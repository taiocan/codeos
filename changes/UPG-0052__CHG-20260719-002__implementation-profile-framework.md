# Self-Development Change: UPG-0052__CHG-20260719-002 — implementation-profile-framework

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit
itself (prompts, templates, docs, patterns, scripts).

This is NOT a downstream DBA feature. It has no behavioral contract, no event schema,
and no replay. Trivial changes do not get a record.

Workflow: prompts/codeos-self-dev.md (4-step loop)
Each step requires explicit human approval; Codex review cadence is governed by the assigned review profile (see prompts/codeos-self-dev.md Step 0a).
The live status row lives in status/self-development.md, not here.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0052
primary_feature_id: UPG-0052
change_id: CHG-20260719-002
slug: implementation-profile-framework
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0052
related_features: [UPG-0051]
review_series: RVS__UPG-0052__CHG-20260719-002__S4
review_profile: PROFILE-4
review_state: ACCEPTED
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

`dba-system.md` is correctly language-neutral through Stage 3, but has no mechanism for a project
to state and enforce a default implementation language before Stage 4, and
`patterns/rust-project-structure.md` is confirmed orphaned — zero references from `dba-system.md`
or any `prompts/*.md` file, including `04-implement.md`. Without an explicit profile mechanism,
"Rust first" is either re-argued ad hoc per feature/crate or silently assumed, and the orphaned
Rust pattern file is never surfaced at the point implementation decisions are actually made. This
was analyzed in `backlog/UPG-0052-implementation-profile-framework.md`, which settled the
conceptual boundaries this change carries into doctrine text — in particular, that this framework
must have **no hard dependency on `UPG-0051`** (the Architecture Synthesis Gate, completed as
`CHG-20260719-001`): it must work for single-feature and loosely-coupled projects that never
declare a core cohort.

**What changes:**

1. `dba-system.md`:
   - New **"Implementation Profile"** section (placed after "Multi-Feature Architecture Synthesis
     Gate," since both are project-level, pre/at-Stage-4 concerns, and before "What You Do at Each
     Stage"), stating:
     - The lifecycle: `proposed` → `approved` → `superseded`, exactly one non-binding pre-approval
       state.
     - **Immutability with an explicit transition path**: once `approved`, a profile version is
       never edited in place, and the current path (`architecture/implementation-profile.yaml`)
       always holds only the current *approved* (or first-ever *proposed*) version — never a
       `proposed` replacement. A material change (language, applicability, exceptions) is drafted
       at `architecture/proposals/implementation-profile-v<N>.yaml` (`status: proposed`,
       `supersedes` pointing at the version it would replace) while the current approved version
       remains binding, unaffected, at the current path. On explicit human approval: (1) the old
       current file moves to `architecture/history/implementation-profile-v<old-N>.yaml` and its
       `status` becomes `superseded`; (2) the proposal is promoted to
       `architecture/implementation-profile.yaml` with `status: approved`. Steps happen in that
       order so the binding version is never ambiguous — the old version stays authoritative until
       the moment its replacement actually takes its place. A first-ever profile (no predecessor)
       may sit `proposed` directly at the current path, since there's no existing binding version
       it could be confused with. This is the same current-vs-historical distinction `UPG-0051`
       established for the Architecture Baseline (see `AJ-015`): **only the current approved
       version is binding for new Stage 4 entry; historical (and proposed-but-unapproved) versions
       are never binding.**
     - **Explicit human authority**: a profile becomes `approved` only following an explicit human
       approval decision. Setting `status: approved` plus `approved_by`/`approved_at` *records*
       that decision — it does not *constitute* it. An agent must never move a profile to
       `approved` merely because the fields are writable.
     - The binding-profile/advisory-pattern distinction (approved profile is binding within its
       resolved scope; the referenced technology pattern's recommendations stay advisory).
     - The structural separation between the language-neutral framework and the rust-first
       supplied profile.
     - Codeos's default-policy statement, phrased honestly about what exists today: a human
       creates the profile from the template; automatic scaffolding by `dba-init.sh` is
       `UPG-0053`, not yet built — this change must not claim `dba-init.sh` already does this.
     - **Profile–Architecture Baseline consistency** (new, addressing a real gap the original
       draft left open): when both an approved profile and an approved Architecture Baseline
       apply to a feature, and the baseline's authoritative decisions specify a different language
       for that feature than the profile (with no matching recorded exception), this is an
       **unreconciled contradiction**. Stage 4 must treat it as **ineligible**, not silently prefer
       either artifact — the human must add a profile exception or approve a reconciling profile
       version before the feature can proceed.
   - Artifact Classification table: new row for the Implementation Profile (Optional; governs
     Stage 4 language/pattern consultation only once `approved`).
   - Templates table: new row for `.codeos/templates/implementation-profile.yaml`.
   - File Layout: clarify that `architecture/` holds independently-optional artifacts (the
     Implementation Profile never requires a cohort; the Architecture Baseline still does — this
     corrects `UPG-0051`'s File Layout comment, which described the whole directory as
     cohort-gated, now that a second, cohort-independent artifact lives there too), and add
     `implementation-profile.yaml`, `proposals/implementation-profile-v[N].yaml` (a pending,
     not-yet-approved replacement — never binding), and
     `history/implementation-profile-v[N].yaml` (superseded, also never binding) to the tree.
   - **No Truth Authority change** — unlike the Architecture Baseline, the profile does not claim
     authority over behavior or structure broadly; it is a Stage 4 consultation requirement, not a
     new truth-resolution concern. (The profile–baseline consistency rule above is an internal
     check between two project-level structural artifacts, not a change to the global Truth
     Authority list.)
   - **No new Stage ID and no new session prompt** — deliberately simpler than `UPG-0051`'s
     Architecture Synthesis Gate, which needed a multi-step synthesis pipeline; a single
     project-wide language preference does not.
2. `templates/implementation-profile.yaml` (**new file**) — the profile artifact template, with
   **resolvable selectors** (not free text) so Stage 4 can actually determine coverage:
   ```yaml
   profile_id: implementation-profile   # stable across all versions of this profile
   profile_version: 1                   # increments each new version
   status: proposed | approved | superseded
   primary_language: rust
   applies_to:
     scope: all | feature_ids | cohort_ids
     feature_ids: []    # populated only when scope: feature_ids
     cohort_ids: []     # populated only when scope: cohort_ids (requires UPG-0051's cohort declared)
   exceptions:
     - scope: feature_id | cohort_id
       id: ""           # the specific F-#### or cohort_id this exception covers
       language: ""
       rationale: ""    # required, non-empty
   approved_by: ""
   approved_at: ""
   supersedes: null      # profile_version of the prior approved version this replaces, or null
   ```
   `scope: all` and `feature_ids`/`cohort_ids` are deliberately the only selector kinds for this
   change — a baseline-defined-component selector was considered (per Step 1 review discussion)
   but would require `UPG-0051`'s baseline template to define addressable component IDs, which it
   does not today; that is a possible future extension, not built here.

   **Exception determinism** (new): exceptions are considered only for a feature already found
   within `applies_to`'s resolved scope. When a feature matches both a feature-level exception and
   a cohort-level exception, **the feature-level exception wins** (more specific). Multiple
   matching exceptions at the *same* specificity that disagree make the profile **invalid for that
   feature** — Stage 4 treats it as ineligible, the same as an unreconciled profile-baseline
   contradiction, rather than picking one arbitrarily. Data-integrity constraint: unused selector
   fields must be empty — a `scope: feature_ids` entry (profile-level or exception-level) must have
   an empty `cohort_ids`, and vice versa; a populated-but-unused field is itself an invalid profile.
3. `patterns/rust-project-structure.md`:
   - Add the missing cross-reference: this pattern is consulted by Stage 4 when an approved
     Implementation Profile names `rust`, and optionally by Architecture Synthesis when
     `UPG-0051` also applies.
   - Add a short "Recommended toolchain/lint baseline" section (`rust-toolchain.toml`, workspace
     lint policy, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
     `cargo test --workspace`) — explicitly labeled as a **recommendation**, not mandatory project
     configuration every Rust project must adopt identically; exact MSRV, edition, and lint
     strictness stay project-specific decisions, optionally recorded in an Architecture Baseline.
4. `prompts/04-implement.md` — add an **Implementation Profile consultation** check, immediately
   after `UPG-0051`'s existing cohort eligibility check:
   - Absent, or `status: proposed` at the current path — no profile is binding; proceed with no
     language requirement. (A pending `proposals/` replacement is never consulted here either.)
   - `status: approved` — verify `profile_version` matches the current approved version at
     `architecture/implementation-profile.yaml` (not a `proposals/` or `history/` file — same
     current-only rule as the Architecture Baseline's `baseline_version` check); resolve whether
     this feature is in scope via `applies_to.scope` (`all` / `feature_ids` membership /
     `cohort_ids` membership against this feature's `architecture_cohort`); if in scope, check for
     a matching exception, resolving feature-level-over-cohort-level per the determinism rule
     above (same-specificity conflicts → ineligible); if in scope with no exception,
     `primary_language` is binding and its applicable Codeos pattern (if one exists) is consulted,
     advisory-only; if not in scope, no requirement applies.
   - **Profile–Baseline consistency check**: if this feature also has an approved Architecture
     Baseline whose authoritative decisions specify a conflicting language with no matching
     exception, **STOP** — ineligible, unreconciled contradiction (see `dba-system.md`).
   - **Provenance recording** (new): when an approved profile applies to this feature, the Stage 4
     output must record `profile_id`, `profile_version`, the resolved language, and any matched
     exception (or explicitly note "no profile" / "profile proposed, non-binding" when neither
     applies) — parallel to `UPG-0051`'s requirement that implementations record which baseline
     version governed them. This is added to `04-implement.md`'s existing Review Package output
     format, not a new artifact.
5. `prompts/00-session-start.md` — add step **3e** (after `UPG-0051`'s existing 3d cohort-state
   step): surface the Implementation Profile's status (absent / `proposed` / `approved`) at
   session start.
6. `prompts/00c-onboarding.md` — add a short "Implementation Profile Awareness" note: Session
   Type D (existing-codebase onboarding) must not presumptively propose or impose rust-first for
   an existing non-Rust codebase; it may derive a `proposed` profile consistent with the observed
   dominant language, or ask the human, but never auto-`approve`.

**Semantic wiring verification** (the backlog brief explicitly called for this to be
"not grep-only"): the five scenarios below become named Step 2 acceptance criteria, each verified
at Step 4 by walking the actual prompt text against the scenario — not merely checking that a
filename is mentioned somewhere. There is no executable test harness for this: `prompts/*.md`
files are natural-language instructions for an LLM agent, not executable code (unlike, say,
`tools/reviewer/src/`, which has real Rust tests) — so "not grep-only" is satisfied by scenario
read-through, the same method `UPG-0051` used and had accepted.
1. Approved rust-first profile in scope for a feature → the Rust pattern is actually surfaced.
2. Approved non-Rust exception for a feature → the Rust pattern is not treated as mandatory for it.
3. No approved profile (absent or `proposed`) → no hidden Rust enforcement.
4. An approved Architecture Baseline may specialize the generic pattern where explicitly
   justified — not silently overridden, not silently overriding.
5. Onboarding an existing non-Rust codebase never silently proposes rust-first.

**Scope boundary — what stays the same:**

- No change to `UPG-0051`'s Architecture Synthesis Gate content, cohort schema, or eligibility
  check logic — this change only *adds* a second, independent check to `04-implement.md`
  immediately after it, and touches File Layout's `architecture/` comment for accuracy (both
  edits are additive, not a behavior change to the completed `CHG-20260719-001` work).
- No new Stage ID, no new session prompt, no Truth Authority change (see above — deliberately
  simpler than `UPG-0051`).
- No claim that `dba-init.sh` already scaffolds a profile — that is `UPG-0053`'s job, not yet
  built; this change's doctrine text states the intended policy without overclaiming current
  script behavior.
- No failure-boundary/error-classification content — that is `UPG-0054`, out of scope here.
- No baseline-defined-component selector for `applies_to`/`exceptions` — only `all`,
  `feature_ids`, `cohort_ids` (see item 2 above); a future extension, not built here.
- No downstream project's actual `.codeos/` symlink or generated files are touched — this changes
  the toolkit source; downstream projects pick it up when they resync `.codeos/`.
- **Reviewer coverage — corrected**: this change introduces **no new review identifier**.
  `prompts/04-implement.md` changes remain covered by the existing Stage ID `4`.
  `prompts/00c-onboarding.md` changes remain covered by the existing `onboarding` Stage ID —
  onboarding is *not* an unreviewable prompt, it already has a Stage ID in `dba-system.md`'s
  table. `prompts/00-session-start.md` has no Stage ID at all (true for session-start only, not
  onboarding). No Review Waiver note is needed the way `UPG-0051` needed one, since no new stage
  id is introduced here.

**Class:** downstream-doctrine
**Scope axis:** downstream doctrine only
**Backlog item:** backlog/UPG-0052-implementation-profile-framework.md

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | No Truth Authority change: `dba-system.md:19-28`'s list is untouched by this change. | `git diff -- dba-system.md` shows zero lines changed within that section. |
| 2 | Profile lifecycle is exactly `proposed \| approved \| superseded` — no additional ad-hoc states. | Read-through of `templates/implementation-profile.yaml` and the new `dba-system.md` section confirms only these three values are documented. |
| 3 | Immutability + transition path: the current path (`architecture/implementation-profile.yaml`) never holds a `proposed` replacement while an approved current version exists; a pending replacement is drafted under `architecture/proposals/`; approval archives the old current to `history/` *before* promoting the proposal to current. | `dba-system.md`'s new section and `templates/implementation-profile.yaml`'s comments state this exact three-directory model and ordering; `prompts/04-implement.md`'s check explicitly excludes `proposals/`/`history/` files from live eligibility. |
| 4 | Explicit human authority: doctrine states the `status: approved` field-edit *records* a prior human decision, not that it *constitutes* one. | Grep the new `dba-system.md` section for this exact framing (not "approval is just editing a field"). |
| 5 | Binding profile / advisory pattern kept sharply separate. | `prompts/04-implement.md`'s check requires profile/exception coverage for Stage 4 eligibility; the referenced Codeos pattern is consulted but described as advisory in both `dba-system.md` and `patterns/rust-project-structure.md`. |
| 6 | Resolvable selectors only: `applies_to.scope` is one of `all \| feature_ids \| cohort_ids` — no free-text scope labels anywhere in the template or doctrine. | Read-through of `templates/implementation-profile.yaml`; grep for any leftover free-text scope example (e.g. `core-domain`) — none should remain as a literal selector value. |
| 7 | Exception determinism: feature-level exception overrides cohort-level; same-specificity conflicting exceptions make the profile invalid for that feature (ineligible, not arbitrary); unused selector fields (`feature_ids`/`cohort_ids`) must be empty when not the active scope kind. | `prompts/04-implement.md`'s check states the specificity order and the same-specificity-conflict outcome explicitly; `templates/implementation-profile.yaml`'s comments state the empty-unused-field constraint. |
| 8 | Profile–Baseline consistency: an unreconciled language contradiction between an approved profile and an approved Architecture Baseline makes Stage 4 **ineligible** — neither artifact silently wins. | `dba-system.md`'s new section states this rule; `prompts/04-implement.md`'s check implements it as an explicit STOP condition distinct from (and after) the ordinary profile/exception resolution. |
| 9 | Provenance recording: when an approved profile applies, Stage 4's output records `profile_id`, `profile_version`, the resolved language, and any matched exception (or explicitly notes "no profile"/"proposed, non-binding"). | `prompts/04-implement.md`'s Review Package output format includes this as an explicit field, not left implicit. |
| 10 | No hard dependency on `UPG-0051`: the profile framework's doctrine, template, and Stage 4 check function correctly described for a project with no declared cohort at all. | Read-through confirms every `UPG-0051`-referencing sentence is phrased as "if a cohort also exists" / "when present," never as a precondition for the profile mechanism itself. |
| 11 | No overclaiming of `dba-init.sh`'s current behavior: doctrine states the intended policy without asserting `dba-init.sh` already scaffolds a profile. | Grep `dba-system.md`'s new section for any present-tense claim about `dba-init.sh` scaffolding; must read as future/`UPG-0053`-scoped, not current behavior. |
| 12 | Toolchain/lint baseline in `patterns/rust-project-structure.md` is explicitly labeled a recommendation, not mandatory uniform project configuration. | Grep the new "Recommended toolchain/lint baseline" section for the word "recommend" / equivalent hedging language, and confirm MSRV/edition/lint-strictness are stated as project-specific. |
| 13 | Reviewer coverage correctly stated: no new Stage ID introduced; `04-implement.md` changes covered by existing Stage ID `4`; `00c-onboarding.md` changes covered by the existing `onboarding` Stage ID (not "no Stage ID at all"); `00-session-start.md` has no Stage ID (true only for session-start). | Read the Scope boundary section's corrected wording; cross-check against `dba-system.md`'s "What You Do at Each Stage" table showing `onboarding` already listed with a Stage ID. |
| 14 | Semantic wiring — 5 named scenarios each independently verifiable by prompt-text walkthrough (not grep-only): (a) approved rust-first profile in scope → Rust pattern surfaced; (b) approved non-Rust exception → Rust pattern not mandatory for that feature; (c) no approved profile → no hidden Rust enforcement; (d) an approved Architecture Baseline may specialize the generic pattern where justified, without silent override in either direction; (e) onboarding an existing non-Rust codebase never silently proposes rust-first. | Step 4 walks `prompts/04-implement.md`, `prompts/00-session-start.md`, and `prompts/00c-onboarding.md`'s actual text against each named scenario and records PASS/FAIL with the specific text cited — not merely confirming a filename appears. |
| 15 | No baseline-defined-component selector built (explicitly out of scope, correctly excluded, not silently half-implemented). | Read-through confirms `templates/implementation-profile.yaml` has no component-selector scaffold or dangling reference to one. |
| 16 | **Downstream-compatibility** (required for `downstream-doctrine` class): generated project `CLAUDE.md` still loads `.codeos/dba-system.md`; every existing prompt/template filename referenced in `dba-system.md`'s tables still exists (no rename, only additions); every new filename this change introduces (`implementation-profile.yaml` and its `proposals/`/`history/` variants, the new `dba-system.md` section, the new `patterns/rust-project-structure.md` section) is referenced from every location Step 1 named. | Grep sweep across `dba-system.md`, `prompts/*.md`, `templates/*`, `patterns/*` — the same method used for `UPG-0051`, which caught real orphaning risk. |
| 17 | No collision with `UPG-0051`'s already-completed additions: the new Implementation Profile check in `prompts/04-implement.md` is purely additive after the existing cohort eligibility check, and the new step 3e in `prompts/00-session-start.md` is additive after the existing step 3d — neither replaces, reorders, or contradicts the other. | Read-through of both files confirms the `UPG-0051` content is byte-unchanged except for the new content appended after it. |

---

## Implementation Notes

<!-- Summary only — the git diff is the source of truth. -->

All 6 files from Step 1's "What changes" list were edited/created as planned. No scope creep; no
additional files touched.

**Key decisions made during implementation** (each maps to an acceptance criterion):
- The new `dba-system.md` "Implementation Profile" section placed immediately after "Multi-Feature
  Architecture Synthesis Gate," before "What You Do at Each Stage" — mirrors `UPG-0051`'s
  placement reasoning (project-level, pre/at-Stage-4 concern, not one of the 9 stages).
- `templates/implementation-profile.yaml` implements the exact schema from Step 1: resolvable
  `applies_to.scope` (`all|feature_ids|cohort_ids`), same-model exceptions with required
  `rationale`, `profile_id`/`profile_version`/`supersedes` for the immutability + transition path.
- `prompts/04-implement.md`'s new check is appended strictly after `UPG-0051`'s existing cohort
  eligibility check (AC 17) — neither reorders nor rewords the other's content.
- `prompts/00-session-start.md`'s new step is `3e`, appended after the existing `3d`, with the
  "After completing 3a–3d" summary line updated to "3a–3e" (a small but necessary consistency fix
  discovered during implementation — otherwise the closing line would silently under-count the
  steps it summarizes).
- `prompts/00c-onboarding.md`'s awareness note is placed right after the role/scope intro, before
  "When This Prompt Applies," so it's read before any module work begins.
- `patterns/rust-project-structure.md` gained both the missing cross-reference (top of file) and
  the toolchain/lint baseline section (placed before "When NOT to Apply This Pattern"), each
  explicitly hedged as advisory/recommended per AC 12.

**Nothing was deferred or discovered out-of-scope during implementation.**

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | No Truth Authority change | PASS | `git diff -- dba-system.md` shows the only removed lines are in the File Layout tree restructuring (item 1's planned edit) — zero lines touched in the Truth Authority section. |
| 2 | Lifecycle exactly `proposed \| approved \| superseded` | PASS | `dba-system.md:267`: "`proposed → approved → superseded`"; `templates/implementation-profile.yaml:55`: `status: proposed # proposed \| approved \| superseded`. |
| 3 | Immutability + `proposals/` transition path | PASS | `dba-system.md:278`, File Layout tree (`architecture/proposals/`), `templates/implementation-profile.yaml` header comments, and `prompts/04-implement.md:36,39` all state the same three-location model and exclude `proposals/`/`history/` from live eligibility. |
| 4 | Explicit human authority framing | PASS | `dba-system.md:269`: "**records** that decision... it does not **constitute** it." |
| 5 | Binding profile / advisory pattern separation | PASS | `dba-system.md`'s "Binding profile, advisory pattern" paragraph; `prompts/04-implement.md`'s check requires profile/exception coverage, consults the pattern advisory-only. |
| 6 | Resolvable selectors only — no free text | PASS | `templates/implementation-profile.yaml`'s `applies_to.scope` is `all \| feature_ids \| cohort_ids`; the only "core-domain"-style mention in `dba-system.md:300` is the negative example explaining *why* free text was rejected. |
| 7 | Exception determinism + empty-unused-field constraint | PASS | `dba-system.md:308-309` states feature-level-wins and same-specificity-conflict-invalidates; `templates/implementation-profile.yaml`'s header states the empty-unused-field rule (its own stated verification method — template documentation, not a live 04-implement.md re-check). |
| 8 | Profile–Baseline consistency | PASS | `dba-system.md:315` and `prompts/04-implement.md:53-56` both state the unreconciled-contradiction-is-ineligible rule. |
| 9 | Provenance recording | PASS | `prompts/04-implement.md:162-163`'s Review Package output field; `dba-system.md:320` states the same requirement. |
| 10 | No hard dependency on `UPG-0051` | PASS | `dba-system.md:263-264`: "independent, optional mechanism... no dependency on the Multi-Feature Architecture Synthesis Gate above." |
| 11 | No `dba-init.sh` overclaim | PASS | `dba-system.md:327-328`: "tracked separately and not yet built... not something `dba-init.sh` already does" — correctly future-scoped, not a present-tense claim. |
| 12 | Toolchain/lint baseline is advisory, not mandatory | PASS | `patterns/rust-project-structure.md:308,320`: "a **recommendation**, not mandatory project configuration"; MSRV/edition/lint strictness stated as project-specific. |
| 13 | Reviewer coverage correctly stated (onboarding has an existing Stage ID) | PASS | `dba-system.md:350` (stage table) lists `onboarding` with Stage ID `onboarding`, confirming the Scope boundary's corrected wording is accurate. |
| 14 | 5 semantic wiring scenarios | PASS (all 5) | Walked `prompts/04-implement.md:34-56` and `prompts/00c-onboarding.md`'s new note against each named scenario: (a) rust-first in scope → pattern consulted (line 50-52); (b) non-Rust exception → binding language becomes the exception's language, not rust (line 46-49); (c) absent/proposed → no requirement (line 35-37); (d) baseline may specialize the pattern, pattern never overrides baseline (`dba-system.md`'s binding/advisory paragraph, combined with `UPG-0051`'s existing human-approval requirement for baseline content — any baseline choice is inherently explicit, not silent); (e) onboarding derives a `proposed` profile from the *observed* dominant language, never auto-`approve`s (`00c-onboarding.md`'s new note). |
| 15 | No baseline-component selector built | PASS | `templates/implementation-profile.yaml:51`: explicitly noted as a future extension, not built; no scaffolding present. |
| 16 | Downstream-compatibility | PASS | Grep sweep, full output embedded below this table (not summarized) — every prompt and template filename referenced in `dba-system.md`'s two tables resolves to an existing file; `templates/project-CLAUDE.md` still references `.codeos/dba-system.md`. |
| 17 | No collision with `UPG-0051`'s content | PASS | `prompts/04-implement.md`'s cohort eligibility check (line 19) is unchanged; the new Implementation Profile check is appended strictly after it. `prompts/00-session-start.md`'s step 3d is unchanged; 3e is appended after it, with the closing summary line correctly updated from "3a–3d" to "3a–3e." |

All 17 criteria PASS.

**AC16 downstream-compatibility sweep — full output:**

```
$ grep -oP '\.codeos/prompts/\K[a-zA-Z0-9_-]+\.md' dba-system.md | sort -u | while read f; do
    test -f "prompts/$f" && echo "OK: $f" || echo "MISSING: $f"; done
OK: 00a-solution-discovery.md
OK: 00b-feature-brief.md
OK: 00c-onboarding.md
OK: 00-session-end.md
OK: 00-session-start.md
OK: 01-intent.md
OK: 02-contract.md
OK: 03b-architecture-synthesis.md
OK: 03-event-schema.md
OK: 04-implement.md
OK: 05-tests.md
OK: 06-observe.md
OK: 07-reconcile.md
OK: 08-replay.md
OK: 09-refine.md
OK: 10-arch-refine.md
OK: pipeline-reviewer.md

$ grep -oP '\.codeos/templates/\K[a-zA-Z0-9_.-]+' dba-system.md | sort -u | while read f; do
    test -f "templates/$f" && echo "OK: $f" || echo "MISSING: $f"; done
OK: architecture-baseline.md
OK: arch-refinement.md
OK: codebase-digest.md
OK: contract.md
OK: conventions.md
OK: event-schema.md
OK: feature-brief.md
OK: feature-spec.md
OK: handoff.md
OK: implementation-profile.yaml
OK: intent.md
OK: refinement.md
OK: review-file.md
OK: review-package.md

$ grep -n "dba-system.md" templates/project-CLAUDE.md
9:**At the start of every Claude Code session — read `.codeos/dba-system.md` before doing any
11:1. Read `.codeos/dba-system.md` — authoritative DBA doctrine; read it first, in full
```

Zero `MISSING:` lines across both sweeps.

**Consistency sweep (grep):**

No stale references, orphaned links, or stage-table↔prompt-file drift found. Every new filename
this change introduces is referenced from every location Step 1 named; every pre-existing
stage-table/template-table entry (including `UPG-0051`'s additions) still resolves.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| Step 1 R1: scope-drift false positive from uncommitted `UPG-0051` diff | IN-SCOPE BLOCKER (root cause, not this change's content) | Fixed — committed `UPG-0051` as two commits matching repo convention, confirmed with human first |
| Step 1 R3 (1st human round): free-text selectors, missing immutability/history, no profile-baseline consistency rule | IN-SCOPE BLOCKER (×3, bundled review) | Fixed — resolvable selectors, immutability + history added |
| Step 1 R4 (2nd human round): no valid location for a pending proposed replacement, non-deterministic exception resolution, no provenance recording | IN-SCOPE BLOCKER (×3, bundled review) | Fixed — `proposals/` directory, exception specificity rule, provenance field added |
| Step 3 R1: `status/self-development.md` bookkeeping lagged the packet's step | SELF-REFERENCE / REVIEW-BOOKKEEPING | No action needed — resolved naturally once the dashboard row was updated post-review, as it always is |
| Step 4 R1: AC16 marked PASS with a prose summary, not the actual grep sweep output — not independently verifiable from the packet | IN-SCOPE BLOCKER | Fixed — full sweep output embedded verbatim in the Reconciliation section below the acceptance table; R2 confirmed NO OBJECTION with evidence grade upgraded A |

---
