---
reviewed:
  feature: 0004-review-fixes
  stage: selfdev-step-4
  branch: feature/backlog-split-and-reviewer
  base_commit: 2563e37971f1f35c47ed34e420c0285807ac66be
  review_commit: 2563e37971f1f35c47ed34e420c0285807ac66be
  artifacts:
    - path: changes/0004-review-fixes.md
      sha256: 86d4eaceff71595c0b55c345857863266ae2fe4beae059176578f1184441fc19
      visibility: shown
    - path: status/self-development.md
      sha256: d57093722fe5dee878195c89812649b7c648de0be49d2535773286c871efe491
      visibility: shown
    - path: status/roadmap.md
      sha256: 33886fa466260869fd626382df5f960b88be432fbc81d8d2b65565d6b47b89d9
      visibility: shown
  diff_hash: 9254c9fead92a02923feac7eca5f27b58935462a48e8bf5a0063f56ca3f8baea
  coverage_state: FULL_COVERAGE
  workspace_dirty: true
  redaction_count: 0
  secret_redaction: false
  excluded_paths: []
  reviewed_packet: packets/2026-06-27T163909Z-0004-review-fixes-stage-selfdev-step-4-2563e37.packet.txt
  reviewed_packet_sha256: bcd0f5de9340c71167bf2823dc7f6566a11d429957e554df283b6b2ab726f2e2
  reviewer: "codex (session 019f09ee-1483-79a0-97a6-665d8861b4bf)"
  codex_concern: CHANGES ADVISED
  effective_concern: CHANGES ADVISED
  evidence: A
---

OpenAI Codex v0.114.0 (research preview)
--------
workdir: /home/arc/projects/claude/Codeos
model: gpt-5.4
provider: openai
approval: never
sandbox: read-only
reasoning effort: high
reasoning summaries: none
session id: 019f09ee-1483-79a0-97a6-665d8861b4bf
--------
user
Critically assess:

SCOPE CONTRACT
  Assess this artifact against the STATED SCOPE of this stage/PR — the Expected Stage
  Output below and what the artifacts actually claim — NOT against an ideal final system.
  A capability the artifacts do not claim to provide is OUT-OF-SCOPE, not a defect.
  The following are OUT-OF-SCOPE BACKLOG unless THIS artifact explicitly claims to provide
  them: formal approval-binding enforcement; rollback correctness; COMMIT_BOUND/
  WORKSPACE_BOUND enforcement; JSON Schema validation; CI validation; exact
  decision-integrity; per-feature decision ledgers; autonomous approval; enabled hooks.

TRIAGE RULE — classify EVERY finding as exactly one of:
  IN-SCOPE BLOCKER     breaks the stated goal; creates a FALSE CLAIM in this artifact;
                       weakens the advisory/read-only/human-gated guarantees; prevents
                       the work from running; or violates an explicit safety constraint.
  IN-SCOPE NON-BLOCKER improves it but is not required for this PR.
  OUT-OF-SCOPE BACKLOG valid, but belongs to a future feature / stronger guarantee.
  REJECTED             conflicts with the stated scope or Codeos philosophy.
  Base the PR decision ONLY on IN-SCOPE BLOCKER findings. An OUT-OF-SCOPE BACKLOG finding
  must NOT cause DO NOT ADVANCE unless this artifact FALSELY CLAIMS to solve it.

REVIEW CONTEXT
  Feature:                0004-review-fixes
  Stage:                  selfdev-step-4
  Branch:                 feature/backlog-split-and-reviewer
  Base commit:            2563e37971f1f35c47ed34e420c0285807ac66be
  Review commit:          2563e37971f1f35c47ed34e420c0285807ac66be (+ uncommitted workspace changes)
  Current approved stage: n/a (non-numeric stage)
  Evidence coverage:      FULL_COVERAGE
  Workspace dirty:        yes (uncommitted changes at review time)

DBA RULES RELEVANT TO THIS STAGE
  - Human approval is required for every stage transition; you are advisory only.
  - Memory is not truth — assess only what is provided, pinned to the review commit.
  - Implementation must trace to approved artifacts; no behavior beyond intent+contract+schema.
  - No events outside the approved event schema; no hidden behavior.

STAGE-SPECIFIC CHECKS
  - (no stage-specific checklist for stage selfdev-step-4)

EXPECTED STAGE OUTPUT
  (no expected-output template for stage selfdev-step-4)

ARTIFACTS TO REVIEW
  --- changes/0004-review-fixes.md (sha256: 86d4eaceff71595c0b55c345857863266ae2fe4beae059176578f1184441fc19, visibility: shown) ---
    # Self-Development Change: 0004-review-fixes
    
    ## Change Intent
    
    **Why (problem in the toolkit):**
    The retroactive doctrine-split series Codex review (run at HEAD `2563e37`, verdict
    **CHANGES ADVISED**, evidence A; `reviews/review-log.md`) raised two IN-SCOPE BLOCKERs and one
    NON-BLOCKER against the `0001`–`0003` work. They must be fixed before merge.
    
    **What changes:**
    - `status/self-development.md` — fix the dashboard header that still named `backlog/features.md`
      as "the roadmap" → name `status/roadmap.md` as the roadmap and `backlog/features.md` as the
      stable catalog (BLOCKER-1). Populate the `Review` column for `0001`–`0003`.
    - `status/roadmap.md` — soften over-absolute wording about every backlog item being a numbered
      self-dev change, acknowledging `reviewer-decision-brief` landed outside the sequence
      (NON-BLOCKER-3).
    - `changes/0001..0003-*.md` — update the "Codex review" lines to reference the series review
      and these fixes, so no record claims COMPLETE without a review record (BLOCKER-2).
    
    **Scope boundary — what stays the same:**
    - No doctrine, prompt, template, or script change. Pure bookkeeping/consistency fixes driven
      by the reviewer.
    
    **Class:** documentation / self-dev-governance
    **Scope axis:** self-dev only
    **Backlog item:** — (reviewer findings on 0001–0003)
    
    ---
    
    ## Acceptance Criteria
    
    | # | Criterion | How verified |
    |---|---|---|
    | 1 | No file calls `backlog/features.md` "the roadmap"; `status/roadmap.md` is named the roadmap | grep `the roadmap` across status/, read-through |
    | 2 | `Review` column populated for 0001–0003 in the dashboard | read-through |
    | 3 | No `changes/*` record claims COMPLETE while stating its review was "not run" | grep "not run" changes/ |
    | 4 | Roadmap wording no longer over-claims universal numbered sequence | read-through |
    
    ---
    
    ## Implementation Notes
    
    Header + Review-column edits in `status/self-development.md`; wording edit in
    `status/roadmap.md`; "Codex review" line updates in the three prior change records. The series
    review verdict (CHANGES ADVISED) and the human decision (REQUEST_CHANGES → fixes) are in
    `reviews/review-log.md`.
    
    ---
    
    ## Reconciliation
    
    | # | Criterion | Result | Evidence |
    |---|---|---|---|
    | 1 | Roadmap pointer correct | PASS | dashboard header + roadmap header agree (status/roadmap.md = roadmap; features.md = catalog) |
    | 2 | Review column populated | PASS | 0001–0003 show `CHANGES ADV →0004` |
    | 3 | No COMPLETE-without-review claim | PASS | `grep "not run" changes/` → none |
    | 4 | Wording tightened | PASS | roadmap intro acknowledges out-of-sequence pilot |
    
    **Codex review:** iterative. Round 1 (HEAD 2563e37) → **CHANGES ADVISED**: two IN-SCOPE
    blockers in this change itself — (a) premature `COMPLETE` / `NO OBJ` lifecycle state, and
    (b) roadmap prose still hard-coding next change IDs (`0004, 0005`). Both fixed in this change.
    This record stays `IN_REVIEW` and the dashboard row stays `IN_PROGRESS` until a re-review
    returns clean; only then is it marked COMPLETE (human decision). See `reviews/review-log.md`.
    
    ---
    
    <!-- METADATA -->
    status: IN_REVIEW
    change_id: 0004-review-fixes
    type: SELF_DEVELOPMENT
    class: documentation / self-dev-governance
    scope: self-dev only
    backlog_item: —
    step_completed: 4
    approved_by: human (in-session, 2026-06-27)
    approved_at: 2026-06-27

  --- status/self-development.md (sha256: d57093722fe5dee878195c89812649b7c648de0be49d2535773286c871efe491, visibility: shown) ---
    # Self-Development Status
    
    > Live dashboard for in-flight Codeos toolkit changes. **Mutable** — maintained by the
    > 4-step self-development loop (see repo-root `CLAUDE.md` and `prompts/codeos-self-dev.md`).
    > The stable rulebook is `CLAUDE.md`; per-change detail is in `changes/[change_id].md`;
    > the roadmap (wave plan) is `status/roadmap.md`; the stable backlog catalog is
    > `backlog/features.md`.
    >
    > Step 1 activates a row; each step updates **Loop step** and **Review**; Reconcile marks
    > **State: COMPLETE** once its compulsory review returns clean — otherwise the row stays
    > **IN_PROGRESS** with the latest advisory verdict until the findings are addressed.
    
    | Backlog item | Change ID | Class | Loop step | Review | State |
    |---|---|---|---|---|---|
    | — | 0001-claude-split | self-dev-governance + downstream-doctrine | 4-Reconcile | CHANGES ADV →0004 | COMPLETE |
    | backlog/doc-consistency-doctrine-rename | 0002-doc-consistency-rename | documentation | 4-Reconcile | CHANGES ADV →0004 | COMPLETE |
    | — (whole-backlog planning) | 0003-implementation-roadmap | backlog-only / self-dev planning | 4-Reconcile | CHANGES ADV →0004 | COMPLETE |
    | — (reviewer findings) | 0004-review-fixes | documentation / self-dev-governance | 4-Reconcile | CHANGES ADV (r1) | IN_PROGRESS |
    
    <!--
    Loop step : 1-Intent / 2-Acceptance / 3-Implement / 4-Reconcile / —
    Review    : NO OBJ / CHANGES ADV / DO NOT ADV / — (latest step's advisory verdict)
    State     : BACKLOG / IN_PROGRESS / BLOCKED / COMPLETE
    -->

  --- status/roadmap.md (sha256: 33886fa466260869fd626382df5f960b88be432fbc81d8d2b65565d6b47b89d9, visibility: shown) ---
    # Codeos Implementation Roadmap
    
    > **This is mutable planning state, not Codeos doctrine.** `backlog/features.md` remains the
    > stable backlog catalog (the briefs and their P0–P3 priorities). This file sequences that
    > catalog into dependency-aware waves and tracks current reality. **Each item still requires
    > its own self-dev change and explicit human approval before implementation** — appearing in a
    > wave here is *not* authorization. Live per-change status lives in
    > `status/self-development.md`; per-change detail in `changes/[id].md`.
    
    From here on, each backlog item is taken as its own self-dev change (the next available
    change ID) through the 4-step loop in `prompts/codeos-self-dev.md`. (Some items already
    landed outside this sequence — e.g. `reviewer-decision-brief` was piloted earlier; see
    Current State. Concrete change IDs are tracked in `status/self-development.md`, not here.)
    
    ---
    
    ## Current State (completed / piloted — not in a wave)
    
    | Item | State | Notes |
    |---|---|---|
    | `0001-claude-split` | DONE | Split downstream doctrine (`dba-system.md`) from the self-dev guide (`CLAUDE.md`). |
    | `0002-doc-consistency-rename` | DONE | Renamed doctrine-attribution refs in docs after the split. |
    | `reviewer-decision-brief` (#1) | PILOTED | Advisory Bash reviewer: `scripts/codeos-review.sh` + `docs/reviewer-pipeline.md` + `docs/reviewer-artifact-schemas.md`. Manual, read-only, non-gatekeeping; no Claude Code hooks wired. |
    
    (`reviewer-quality-scale` is **partially piloted** — the reviewer emits an evidence grade but
    the full scale is not realized; the remaining work is scheduled once in Wave 1, not here.)
    
    ---
    
    ## Waves (dependency-aware)
    
    Sequencing principles: value first (P0 → P3); respect hard dependencies; build on the existing
    reviewer pilot; preserve "advisory-not-autonomous" and "evidence-integrity"; take cheap
    pilot-adjacent wins early.
    
    ### Wave 1 — Transparency & state  *(P0, no deps, highest value / lowest risk)*
    | Item | Pri | Depends on | Notes |
    |---|---|---|---|
    | `stage-4-6-reports` | P0 | — | Structured Implementation/Test/Runtime reports. **Next pickup.** |
    | `current-verified-state` | P0 | — | Auto-generated session-start state snapshot. |
    | `reviewer-quality-scale` | P2 | reviewer pilot ✓ | **Partially piloted** — formalize the A–E evidence scale + reviewer consistency (pilot already emits a grade). |
    
    ### Wave 2 — Discovery & registry
    | Item | Pri | Depends on | Notes |
    |---|---|---|---|
    | `solution-discovery-00b` | P0 | — | Non-authoritative pre-Stage-1 topology discovery. |
    | `config-discovery` | P2 | solution-discovery-00b | Config surface inside expanded 00b. |
    | `feature-registry` | P2 | — | Feature ID ↔ branch ↔ stage ↔ PR binding; unlocks Wave 4–5 automation. |
    
    ### Wave 3 — Evidence discipline & gates  *(P1)*
    | Item | Pri | Depends on | Notes |
    |---|---|---|---|
    | `verify-only-mode` | P1 | — | Strict read-only verification (never edits evidence). |
    | `readiness-checklist` | P1 | — | Lightweight operational merge/release gate. |
    | `repair-before-next-feature` | P1 | — | Unresolved work blocks new behavioral features (human override allowed). |
    | `stage-4-activation-card` | P1 | — | Small activation card; references, never restates, approved artifacts. |
    
    ### Wave 4 — Reviewer hardening & delivery
    | Item | Pri | Depends on | Notes |
    |---|---|---|---|
    | `reviewer-full-diff` | P2 | reviewer-decision-brief ✓ | Reviewer inspects filtered full diff, not only artifacts. |
    | `reviewer-decision-integrity` | P1 | reviewer pilot ✓ | Bind approval to reviewed provenance (commit + diff_hash + workspace). |
    | `workflow-profiles` | P1 | — | Optional branch/PR/CI profiles (simple / one-branch / split-PR). |
    | `stack-manifest` | P2 | — | Two-layer stack record with diff-triggered reconciliation. |
    
    ### Wave 5 — Advanced automation & generators  *(mostly P3 + the big P1 rewrite)*
    | Item | Pri | Depends on | Notes |
    |---|---|---|---|
    | `reviewer-engine-v1` | P1 | reviewer pilot proven | Typed (Rust/Python) reviewer engine replacing the Bash pilot — only after the pilot is proven. |
    | `ci-profile` | P3 | stack-manifest | Map Codeos evidence types onto CI checks. |
    | `stack-drift-detector` | P3 | stack-manifest | Block release on unreconciled dependency/config drift. |
    | `stage-report-generator` | P3 | stage-4-6-reports | Auto-fill Stage 4–6 report skeletons. |
    | `00b-adr-generator` | P3 | solution-discovery-00b | Turn 00b architecture risks into routable ADR candidates. |
    | `approval-dashboard` | P3 | feature-registry, reviewer pilot | Generated cross-feature stage/review/blocker overview. |
    | `release-evidence-package` | P3 | — | Generated pre-release evidence bundle from existing artifacts. |
    | `reviewer-verification-packet` | P3 | verify-only-mode | Reviewer can request read-only verification when confidence is low. |
    | `branch-helper` | P2 | — | Optional branch-naming helper; doc-first. |
    
    ---
    
    ## Immediate next pickups
    
    1. `stage-4-6-reports` (Wave 1, P0, no deps) — foundational for later generators/dashboards.
    2. `current-verified-state` (Wave 1, P0, no deps).
    
    Each is taken as its own self-dev change (the next available change ID) via the 4-step loop,
    with human approval at every gate.
    
    ---
    
    ## Guardrails — "do NOT do yet" (from `backlog/features.md`)
    
    - No full OAP integration doctrine as a core Codeos change.
    - No mandatory PRs at every stage.
    - No large execution packet that duplicates intent / contract / event schema.
    - Stack manifest must never be manually-maintained-only.
    - Verification mode must never edit files.
    - 00b output is never an approved feature list or architecture.
    - **Do not change Codeos non-negotiable rules** until these smaller upgrades are piloted.

DIFF TO REVIEW (base->review, secret/size filtered)
diff --git a/changes/0001-claude-split.md b/changes/0001-claude-split.md
index 238c37f..0b99809 100644
--- a/changes/0001-claude-split.md
+++ b/changes/0001-claude-split.md
@@ -88,9 +88,13 @@ Loading-path refs clean. One documentation gap (descriptive doctrine attribution
 | Reviewer crashed on non-numeric stage | IN-SCOPE BLOCKER | fixed (guard in `build_packet`) |
 
 **Note on process:** This bootstrap change predates a runnable self-dev loop, so the
-compulsory per-step Codex reviews were not executed; the change was instead gated through the
-plan-mode review cycle (multiple human-relayed reviewer passes + explicit approval). Future
-non-trivial changes follow the per-step compulsory-review discipline.
+compulsory per-step Codex reviews were not executed at the time; the change was instead gated
+through the plan-mode review cycle (multiple human-relayed reviewer passes + explicit
+approval). A retroactive doctrine-split **series** Codex review was subsequently run at HEAD
+2563e37 (verdict **CHANGES ADVISED**, evidence A; see `reviews/review-log.md`); it flagged
+that this record's status-dashboard header still named `backlog/features.md` as "the roadmap"
+— addressed by change `0004-review-fixes`. Future non-trivial changes follow the per-step
+compulsory-review discipline.
 
 ---
 
diff --git a/changes/0002-doc-consistency-rename.md b/changes/0002-doc-consistency-rename.md
index e3ac0cc..c4d2037 100644
--- a/changes/0002-doc-consistency-rename.md
+++ b/changes/0002-doc-consistency-rename.md
@@ -69,9 +69,10 @@ naming-collision / governance references. Final scope is 25 edits across 2 files
 |---|---|---|
 | oap-codeos-integration.md / reviewer-pipeline.md CLAUDE.md refs | IN-SCOPE NON-BLOCKER → leave | Reverted; preserved as historical/governance refs ("rename only where it makes sense") |
 
-**Codex review:** not run — change executed under direct human supervision (human acting as
-in-session reviewer). `codex` CLI is available; automated review deferred by choice, not
-silently skipped. Run on request via `codeos-review.sh review 0002-doc-consistency-rename ...`.
+**Codex review:** covered by the retroactive doctrine-split series review (run at HEAD
+2563e37, verdict **CHANGES ADVISED**, evidence A; see `reviews/review-log.md`). The
+series-level findings (review-record compliance, status-dashboard roadmap pointer) are
+addressed by change `0004-review-fixes`; no finding was specific to this docs rename.
 
 ---
 
diff --git a/changes/0003-implementation-roadmap.md b/changes/0003-implementation-roadmap.md
index 5e4c22c..97093eb 100644
--- a/changes/0003-implementation-roadmap.md
+++ b/changes/0003-implementation-roadmap.md
@@ -54,9 +54,12 @@ its status note in Current State explicitly points to Wave 1 rather than re-list
 | 3 | Deps shown, acyclic | PASS | dependency column per wave |
 | 4 | Honest current state | PASS | reviewer-quality-scale = partially piloted |
 
-**Codex review:** not run — change executed under direct human supervision (human acting as
-in-session reviewer). `codex` CLI available; automated review deferred by choice, not silently
-skipped.
+**Codex review:** RUN (retroactively, as the doctrine-split series review at HEAD 2563e37).
+Verdict **CHANGES ADVISED** (evidence A, full coverage) — see
+`reviews/codex/2026-06-27T163034Z-dsplit-0003-stage-selfdev-step-4-2563e37.md` and
+`reviews/review-log.md`. Two IN-SCOPE BLOCKERs + one NON-BLOCKER raised; all addressed by
+change `0004-review-fixes` (status-dashboard roadmap pointer, review-record compliance,
+roadmap wording).
 
 ---
 
diff --git a/status/roadmap.md b/status/roadmap.md
index 4781f72..79d2e20 100644
--- a/status/roadmap.md
+++ b/status/roadmap.md
@@ -7,8 +7,10 @@
 > wave here is *not* authorization. Live per-change status lives in
 > `status/self-development.md`; per-change detail in `changes/[id].md`.
 
-Each backlog item is implemented as a self-dev change (`0004`, `0005`, …) through the 4-step
-loop in `prompts/codeos-self-dev.md`.
+From here on, each backlog item is taken as its own self-dev change (the next available
+change ID) through the 4-step loop in `prompts/codeos-self-dev.md`. (Some items already
+landed outside this sequence — e.g. `reviewer-decision-brief` was piloted earlier; see
+Current State. Concrete change IDs are tracked in `status/self-development.md`, not here.)
 
 ---
 
@@ -81,8 +83,8 @@ pilot-adjacent wins early.
 1. `stage-4-6-reports` (Wave 1, P0, no deps) — foundational for later generators/dashboards.
 2. `current-verified-state` (Wave 1, P0, no deps).
 
-Each is taken as its own self-dev change (`0004`, `0005`, …) via the 4-step loop, with human
-approval at every gate.
+Each is taken as its own self-dev change (the next available change ID) via the 4-step loop,
+with human approval at every gate.
 
 ---
 
diff --git a/status/self-development.md b/status/self-development.md
index f7d714c..06a124a 100644
--- a/status/self-development.md
+++ b/status/self-development.md
@@ -3,16 +3,19 @@
 > Live dashboard for in-flight Codeos toolkit changes. **Mutable** — maintained by the
 > 4-step self-development loop (see repo-root `CLAUDE.md` and `prompts/codeos-self-dev.md`).
 > The stable rulebook is `CLAUDE.md`; per-change detail is in `changes/[change_id].md`;
-> the roadmap is `backlog/features.md`.
+> the roadmap (wave plan) is `status/roadmap.md`; the stable backlog catalog is
+> `backlog/features.md`.
 >
 > Step 1 activates a row; each step updates **Loop step** and **Review**; Reconcile marks
-> **State: COMPLETE**.
+> **State: COMPLETE** once its compulsory review returns clean — otherwise the row stays
+> **IN_PROGRESS** with the latest advisory verdict until the findings are addressed.
 
 | Backlog item | Change ID | Class | Loop step | Review | State |
 |---|---|---|---|---|---|
-| — | 0001-claude-split | self-dev-governance + downstream-doctrine | 4-Reconcile | — | COMPLETE |
-| backlog/doc-consistency-doctrine-rename | 0002-doc-consistency-rename | documentation | 4-Reconcile | — | COMPLETE |
-| — (whole-backlog planning) | 0003-implementation-roadmap | backlog-only / self-dev planning | 4-Reconcile | — | COMPLETE |
+| — | 0001-claude-split | self-dev-governance + downstream-doctrine | 4-Reconcile | CHANGES ADV →0004 | COMPLETE |
+| backlog/doc-consistency-doctrine-rename | 0002-doc-consistency-rename | documentation | 4-Reconcile | CHANGES ADV →0004 | COMPLETE |
+| — (whole-backlog planning) | 0003-implementation-roadmap | backlog-only / self-dev planning | 4-Reconcile | CHANGES ADV →0004 | COMPLETE |
+| — (reviewer findings) | 0004-review-fixes | documentation / self-dev-governance | 4-Reconcile | CHANGES ADV (r1) | IN_PROGRESS |
 
 <!--
 Loop step : 1-Intent / 2-Acceptance / 3-Implement / 4-Reconcile / —

INSTRUCTIONS
  If this is a resumed session, ignore any earlier-session conclusions unless they are
  re-established by THIS packet; assess only the evidence above, pinned to this commit.
  Give your full critical assessment (operational, ranked by severity, with concrete
  better-designs). For EACH finding emit:
    Finding: / Severity: High|Medium|Low / Classification: <one of the TRIAGE RULE labels>
    Evidence: <file/line> / Why: <short> / Required action: fix now|optional fix|backlog|reject
    Scope reason: <why it does/does not belong to this PR's scope>
  Then emit:
    PR decision: ADVANCE | REQUEST CHANGES | DO NOT ADVANCE   (based ONLY on in-scope blockers)
    Scope drift warning: yes|no — <is anything pulling this PR beyond its stated scope?>
  Then on the LAST two lines emit exactly (map ADVANCE->NO OBJECTION,
  REQUEST CHANGES->CHANGES ADVISED, DO NOT ADVANCE->DO NOT ADVANCE):
    LOG SUMMARY: <NO OBJECTION | CHANGES ADVISED | DO NOT ADVANCE | UNCLASSIFIED> — <single most important point>
      (use UNCLASSIFIED if you genuinely cannot classify the artifact safely)
    EVIDENCE: <A|B|C|D|E>   (optional)

mcp startup: no servers
codex
I’ve narrowed this to whether the bookkeeping semantics are now internally consistent after the last fixes. I’m checking the current line-numbered files for one remaining point that affects the advance decision.
exec
/bin/bash -lc "nl -ba changes/0004-review-fixes.md | sed -n '1,220p'" in /home/arc/projects/claude/Codeos succeeded in 0ms:
     1	# Self-Development Change: 0004-review-fixes
     2	
     3	## Change Intent
     4	
     5	**Why (problem in the toolkit):**
     6	The retroactive doctrine-split series Codex review (run at HEAD `2563e37`, verdict
     7	**CHANGES ADVISED**, evidence A; `reviews/review-log.md`) raised two IN-SCOPE BLOCKERs and one
     8	NON-BLOCKER against the `0001`–`0003` work. They must be fixed before merge.
     9	
    10	**What changes:**
    11	- `status/self-development.md` — fix the dashboard header that still named `backlog/features.md`
    12	  as "the roadmap" → name `status/roadmap.md` as the roadmap and `backlog/features.md` as the
    13	  stable catalog (BLOCKER-1). Populate the `Review` column for `0001`–`0003`.
    14	- `status/roadmap.md` — soften over-absolute wording about every backlog item being a numbered
    15	  self-dev change, acknowledging `reviewer-decision-brief` landed outside the sequence
    16	  (NON-BLOCKER-3).
    17	- `changes/0001..0003-*.md` — update the "Codex review" lines to reference the series review
    18	  and these fixes, so no record claims COMPLETE without a review record (BLOCKER-2).
    19	
    20	**Scope boundary — what stays the same:**
exec
/bin/bash -lc "nl -ba status/self-development.md | sed -n '1,220p'" in /home/arc/projects/claude/Codeos succeeded in 0ms:
     1	# Self-Development Status
     2	
     3	> Live dashboard for in-flight Codeos toolkit changes. **Mutable** — maintained by the
     4	> 4-step self-development loop (see repo-root `CLAUDE.md` and `prompts/codeos-self-dev.md`).
     5	> The stable rulebook is `CLAUDE.md`; per-change detail is in `changes/[change_id].md`;
     6	> the roadmap (wave plan) is `status/roadmap.md`; the stable backlog catalog is
     7	> `backlog/features.md`.
     8	>
     9	> Step 1 activates a row; each step updates **Loop step** and **Review**; Reconcile marks
    10	> **State: COMPLETE** once its compulsory review returns clean — otherwise the row stays
    11	> **IN_PROGRESS** with the latest advisory verdict until the findings are addressed.
    12	
    13	| Backlog item | Change ID | Class | Loop step | Review | State |
    14	|---|---|---|---|---|---|
    15	| — | 0001-claude-split | self-dev-governance + downstream-doctrine | 4-Reconcile | CHANGES ADV →0004 | COMPLETE |
    16	| backlog/doc-consistency-doctrine-rename | 0002-doc-consistency-rename | documentation | 4-Reconcile | CHANGES ADV →0004 | COMPLETE |
    17	| — (whole-backlog planning) | 0003-implementation-roadmap | backlog-only / self-dev planning | 4-Reconcile | CHANGES ADV →0004 | COMPLETE |
    18	| — (reviewer findings) | 0004-review-fixes | documentation / self-dev-governance | 4-Reconcile | CHANGES ADV (r1) | IN_PROGRESS |
    19	
    20	<!--
exec
/bin/bash -lc "nl -ba status/roadmap.md | sed -n '1,220p'" in /home/arc/projects/claude/Codeos succeeded in 0ms:
     1	# Codeos Implementation Roadmap
     2	
     3	> **This is mutable planning state, not Codeos doctrine.** `backlog/features.md` remains the
     4	> stable backlog catalog (the briefs and their P0–P3 priorities). This file sequences that
     5	> catalog into dependency-aware waves and tracks current reality. **Each item still requires
     6	> its own self-dev change and explicit human approval before implementation** — appearing in a
     7	> wave here is *not* authorization. Live per-change status lives in
     8	> `status/self-development.md`; per-change detail in `changes/[id].md`.
     9	
    10	From here on, each backlog item is taken as its own self-dev change (the next available
    11	change ID) through the 4-step loop in `prompts/codeos-self-dev.md`. (Some items already
    12	landed outside this sequence — e.g. `reviewer-decision-brief` was piloted earlier; see
    13	Current State. Concrete change IDs are tracked in `status/self-development.md`, not here.)
    14	
    15	---
    16	
    17	## Current State (completed / piloted — not in a wave)
    18	
    19	| Item | State | Notes |
    20	|---|---|---|
codex
Finding: The dashboard now states a completion rule that its own completed rows do not satisfy.  
Severity: High  
Classification: IN-SCOPE BLOCKER  
Evidence: [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:9), [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:15), [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:16), [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:17)  
Why: The header says Reconcile marks `State: COMPLETE` only once the compulsory review returns clean, but rows `0001`–`0003` are marked `COMPLETE` while their latest recorded review is still `CHANGES ADV →0004`. That is an internal false claim in the bookkeeping surface this PR is explicitly repairing.  
Required action: fix now  
Scope reason: This belongs to this PR because the PR explicitly changes the dashboard header and review-column bookkeeping. A better design is to either relax the prose to match the actual model (`COMPLETE` can coexist with a later advisory finding tracked by a follow-up fix) or keep those rows non-complete until a clean re-review is actually recorded.

Finding: The `Review` column syntax is still undocumented even though this PR populates and relies on it.  
Severity: Low  
Classification: IN-SCOPE NON-BLOCKER  
Evidence: [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:15), [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:18), [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:21)  
Why: The legend documents only bare verdict values (`NO OBJ / CHANGES ADV / DO NOT ADV / —`), but the actual rows use extended forms like `CHANGES ADV →0004` and `CHANGES ADV (r1)`. The meaning is inferable, but the grammar is implicit.  
Required action: optional fix  
Scope reason: This is in scope because the PR’s stated work includes populating and clarifying the `Review` column, but it does not create a false claim or prevent the documentation fix from functioning.

PR decision: REQUEST CHANGES  
Scope drift warning: no — the remaining issue is still within the stated bookkeeping/consistency scope of this PR.

LOG SUMMARY: CHANGES ADVISED — the dashboard’s new completion rule still contradicts the completed `0001`–`0003` rows
EVIDENCE: A
tokens used
40,070
Finding: The dashboard now states a completion rule that its own completed rows do not satisfy.  
Severity: High  
Classification: IN-SCOPE BLOCKER  
Evidence: [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:9), [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:15), [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:16), [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:17)  
Why: The header says Reconcile marks `State: COMPLETE` only once the compulsory review returns clean, but rows `0001`–`0003` are marked `COMPLETE` while their latest recorded review is still `CHANGES ADV →0004`. That is an internal false claim in the bookkeeping surface this PR is explicitly repairing.  
Required action: fix now  
Scope reason: This belongs to this PR because the PR explicitly changes the dashboard header and review-column bookkeeping. A better design is to either relax the prose to match the actual model (`COMPLETE` can coexist with a later advisory finding tracked by a follow-up fix) or keep those rows non-complete until a clean re-review is actually recorded.

Finding: The `Review` column syntax is still undocumented even though this PR populates and relies on it.  
Severity: Low  
Classification: IN-SCOPE NON-BLOCKER  
Evidence: [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:15), [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:18), [status/self-development.md](/home/arc/projects/claude/Codeos/status/self-development.md:21)  
Why: The legend documents only bare verdict values (`NO OBJ / CHANGES ADV / DO NOT ADV / —`), but the actual rows use extended forms like `CHANGES ADV →0004` and `CHANGES ADV (r1)`. The meaning is inferable, but the grammar is implicit.  
Required action: optional fix  
Scope reason: This is in scope because the PR’s stated work includes populating and clarifying the `Review` column, but it does not create a false claim or prevent the documentation fix from functioning.

PR decision: REQUEST CHANGES  
Scope drift warning: no — the remaining issue is still within the stated bookkeeping/consistency scope of this PR.

LOG SUMMARY: CHANGES ADVISED — the dashboard’s new completion rule still contradicts the completed `0001`–`0003` rows
EVIDENCE: A
