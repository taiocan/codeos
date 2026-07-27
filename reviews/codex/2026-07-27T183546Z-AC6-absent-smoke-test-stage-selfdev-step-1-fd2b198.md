---
review_id: REV__AC6-absent-smoke-test__selfdev-step-1__R1
findings: []
unparsed_findings_count: 3
reviewed:
  feature: AC6-absent-smoke-test
  stage: selfdev-step-1
  branch: main
  base_commit: (no base pin)
  review_commit: fd2b1983a3bf21d54b19ba55f99f4da30f050609
  artifacts:
    - path: CLAUDE.md
      sha256: cf95a873a95d1bc16f6f6aaf5394c024846d51a75ac5adb14d77d7eae71f5e32
      visibility: shown
    - path: /tmp/codeos-cpe-status.wTywjn
      sha256: df37304546e025a68e31c8a379add367a5eca7354d3205bb2e74ac570295244d
      visibility: shown
  diff_hash: 406326db43c064d786beae91fd3238f5485fb703dcf01311d9268dbdf360c7a8
  coverage_state: FULL_COVERAGE
  workspace_dirty: true
  redaction_count: 0
  secret_redaction: false
  excluded_paths: []
  reviewed_packet: packets/20260727T183546Z-AC6-absent-smoke-test-stage-selfdev-step-1-fd2b198.packet.txt
  reviewed_packet_sha256: 64a27f2fcdf9c90c296588fd6f2b8f0c89cc3fdcf4080fd941cf6c2fcfb9e66f
  reviewer: "codex (session 019fa4d9-3c9e-7e62-8735-ff46a9518715)"
  codex_concern: DO NOT ADVANCE
  effective_concern: DO NOT ADVANCE
  evidence: A
  reasoning_effort: high
  reconnect_count: 0
  elapsed_ms: 218165
---

Acceptance criteria are not directly reviewable from this packet. The only AC-related evidence shown here is the bookkeeping claim that “all 13 ACs PASS” in [status/self-development.md](/home/rimo/projects/Codeos/status/self-development.md:82); the packet does not include the CHG-B acceptance-criteria/reconciliation artifact itself.

Finding: Automatic CPE status injection breaks documented delta-mode `review`/`plan`  
Severity: High  
Classification: IN-SCOPE BLOCKER  
Evidence: [scripts/codeos-review.sh](/home/rimo/projects/Codeos/scripts/codeos-review.sh:94), [scripts/codeos-review.sh](/home/rimo/projects/Codeos/scripts/codeos-review.sh:102), [docs/reviewer-pipeline.md](/home/rimo/projects/Codeos/docs/reviewer-pipeline.md:515), [docs/reviewer-pipeline.md](/home/rimo/projects/Codeos/docs/reviewer-pipeline.md:620), [docs/reviewer-pipeline.md](/home/rimo/projects/Codeos/docs/reviewer-pipeline.md:625), [backlog/UPG-0057-controlled-plain-english-writing-discipline.md](/home/rimo/projects/Codeos/backlog/UPG-0057-controlled-plain-english-writing-discipline.md:98)  
Why: The wrapper now always appends an untracked temp file to `review` and `plan`, but delta mode is documented to error on untracked artifacts. The packet also claims there were no Rust-engine changes, so there is no shown exemption path. This breaks a stated supported mode.  
Required action: fix now  
Scope reason: The PR’s stated scope is wrapper-level automatic status injection for `review` and `plan`; breaking a documented reviewer mode is directly in scope.

Finding: CHG-B / UPG-0057 state claims are internally contradictory  
Severity: Medium  
Classification: IN-SCOPE BLOCKER  
Evidence: [backlog/UPG-0057-controlled-plain-english-writing-discipline.md](/home/rimo/projects/Codeos/backlog/UPG-0057-controlled-plain-english-writing-discipline.md:90), [backlog/UPG-0057-controlled-plain-english-writing-discipline.md](/home/rimo/projects/Codeos/backlog/UPG-0057-controlled-plain-english-writing-discipline.md:112), [status/self-development.md](/home/rimo/projects/Codeos/status/self-development.md:82)  
Why: The backlog note says “With `CHG-B` accepted, `UPG-0057` is complete,” the change table still marks `CHG-20260727-001` as `DRAFT`, and the status dashboard says Step 4 is written but review is pending and the row is `IN_PROGRESS`. Those states cannot all be true at once.  
Required action: fix now  
Scope reason: This change explicitly updates self-development governance/bookkeeping, so truthful stage/state reporting is in scope.

Finding: The packet scope does not match its stated review stage  
Severity: Medium  
Classification: IN-SCOPE BLOCKER  
Evidence: [/tmp/codeos-cpe-status.wTywjn](/tmp/codeos-cpe-status.wTywjn:3), [CLAUDE.md](/home/rimo/projects/Codeos/CLAUDE.md:89), [CLAUDE.md](/home/rimo/projects/Codeos/CLAUDE.md:97), [CLAUDE.md](/home/rimo/projects/Codeos/CLAUDE.md:100), [scripts/codeos-review.sh](/home/rimo/projects/Codeos/scripts/codeos-review.sh:38), [scripts/codeos-review.sh](/home/rimo/projects/Codeos/scripts/codeos-review.sh:102), [status/self-development.md](/home/rimo/projects/Codeos/status/self-development.md:82)  
Why: The synthetic status artifact and review context say this is `selfdev-step-1`, which `CLAUDE.md` defines as Change Intent. But the reviewed diff includes shipped script behavior and Step 4 reconciliation bookkeeping. That is beyond the Step 1 boundary and prevents a clean stage-scoped assessment.  
Required action: fix now  
Scope reason: Stage identity and gate separation are explicit self-development rules; this belongs to the PR’s stated scope, not backlog.

PR decision: DO NOT ADVANCE  
Scope drift warning: yes — the packet is labeled `selfdev-step-1`, but it includes implementation in [scripts/codeos-review.sh](/home/rimo/projects/Codeos/scripts/codeos-review.sh:1) and Reconcile-era status/bookkeeping updates in [status/self-development.md](/home/rimo/projects/Codeos/status/self-development.md:82) and [backlog/UPG-0057-controlled-plain-english-writing-discipline.md](/home/rimo/projects/Codeos/backlog/UPG-0057-controlled-plain-english-writing-discipline.md:90).

LOG SUMMARY: DO NOT ADVANCE — automatic CPE injection appears to break the documented delta-mode review path
EVIDENCE: A
HIGHEST-IMPACT UNCERTAINTY: If the reviewer engine already special-cases the injected temp artifact during delta-mode packet building despite no engine changes shown here, the first blocker would weaken materially.
OpenAI Codex v0.145.0
--------
workdir: /home/rimo/projects/Codeos
model: gpt-5.4
provider: openai
approval: never
sandbox: read-only
reasoning effort: high
reasoning summaries: none
session id: 019fa4d9-3c9e-7e62-8735-ff46a9518715
--------
user
Reviewer task:

SCOPE CONTRACT
  Assess this artifact against the STATED SCOPE of this stage/PR — the Expected Stage
  Output below and what the artifacts actually claim — NOT against an ideal final system.
  A capability the artifacts do not claim to provide is OUT-OF-SCOPE, not a defect.
  The following are OUT-OF-SCOPE BACKLOG unless THIS artifact explicitly claims to provide
  them: formal approval-binding enforcement; rollback correctness; COMMIT_BOUND/
  WORKSPACE_BOUND enforcement; JSON Schema validation; CI validation; exact
  decision-integrity; per-feature decision ledgers; autonomous approval; enabled hooks.

YOUR TASK — answer these five questions using only the evidence in the packet:
  1. Acceptance criteria: Does the artifact satisfy each of its stated acceptance criteria?
     Cite evidence for each criterion (or note its absence).
  2. Claim support: Are universal or strong claims (all, every, never, always, no X) in the
     artifact supported by evidence in the packet? Any unverifiable strong claim is a candidate
     finding only if it affects acceptance, scope, safety, decision integrity, or the artifact's
     stated guarantees.
  3. Scope drift: Is there any change beyond the stated scope boundary? (Files not in the
     "What changes" list; behavior changes not in the intent; downstream doctrine modified
     rather than read.)
  4. In-scope blockers: Are there facts that, if left in, would make the artifact wrong,
     unsafe, or internally contradictory?
  5. Finding classification: Classify every finding you raise as exactly one of the five
     TRIAGE RULE categories below.

TRIAGE RULE — classify EVERY finding as exactly one of:
  IN-SCOPE BLOCKER         breaks the stated goal; creates a FALSE CLAIM in this artifact;
                           weakens the advisory/read-only/human-gated guarantees; prevents
                           the work from running; or violates an explicit safety constraint.
  IN-SCOPE NON-BLOCKER     improves it but is not required for this PR.
  OUT-OF-SCOPE BACKLOG     valid, but belongs to a future feature / stronger guarantee.
  REJECTED                 conflicts with the stated scope or Codeos philosophy.
  SELF-REFERENCE /         review records that are stale because of the previous round's
  REVIEW-BOOKKEEPING       own existence (causal loop); not a real artifact defect.
  Base the PR decision ONLY on IN-SCOPE BLOCKER findings. An OUT-OF-SCOPE BACKLOG finding
  must NOT cause DO NOT ADVANCE unless this artifact FALSELY CLAIMS to solve it.

WHAT NOT TO DO
  - Do not flag style or wording issues as blockers unless the wording creates a false claim,
    contradiction, parser breakage, or wrong governance instruction.
  - Do not re-review unchanged full context when the packet is in delta mode.
  - Do not treat local-only review history as a blocker unless the artifact falsely
    claims the review artifacts are committed/durable.

INSTRUCTIONS
  If this is a resumed session, ignore any earlier-session conclusions unless they are
  re-established by THIS packet; assess only the evidence above, pinned to this commit.
  Give a focused assessment of this artifact against the stated scope, acceptance criteria,
  and evidence in this packet. Rank findings by severity. Suggest a better design only when
  needed to explain a required fix for an IN-SCOPE BLOCKER.

  CONTROLLED PLAIN ENGLISH (if one of the artifacts under review states "Controlled Plain
  English status for this review: enabled" or "...: disabled" — this task template never reads
  any configuration file itself; whoever invokes the review includes that line as one of the
  reviewed artifacts, the same way any other file is included, when the relevant project has the
  discipline enabled): Layer D1 (advisory verdict, no invented requirements, evidence separated
  from inference — the TRIAGE RULE and "based ONLY on in-scope blockers" instructions above
  already are this) always applies regardless of that value. When the value is `enabled`,
  additionally write your prose in Layer D2 style — short sentences, common words, direct
  explanations. This affects wording only; it never changes the footer format below.

  Limit findings to the top 3 IN-SCOPE BLOCKERS. Additional non-blocking observations may be
  summarized in one short paragraph only if useful.

  For EACH finding emit:
    Finding: / Severity: High|Medium|Low / Classification: <one of the TRIAGE RULE labels>
    Evidence: <file/line> / Why: <short> / Required action: fix now|optional fix|backlog|reject
    Scope reason: <why it does/does not belong to this PR's scope>
  Then emit:
    PR decision: ADVANCE | REQUEST CHANGES | DO NOT ADVANCE   (based ONLY on in-scope blockers)
    Scope drift warning: yes|no — <is anything pulling this PR beyond its stated scope?>
  Then on the LAST three lines emit exactly (map ADVANCE->NO OBJECTION,
  REQUEST CHANGES->CHANGES ADVISED, DO NOT ADVANCE->DO NOT ADVANCE):
    LOG SUMMARY: <NO OBJECTION | CHANGES ADVISED | DO NOT ADVANCE | UNCLASSIFIED> — <single most important point>
      (use UNCLASSIFIED if you genuinely cannot classify the artifact safely)
    EVIDENCE: <A|B|C|D|E>
    HIGHEST-IMPACT UNCERTAINTY: <one sentence — what single thing, if wrong, most affects this assessment>

  Evidence grade — the grade describes what the assessment rests on, not reviewer confidence:
    A — Directly verified in the artifact, diff, or output shown in the packet
    B — Verified with multiple direct pieces of evidence, but coverage is not complete
    C — Partially verified, partially inferred from structure or context
    D — Mostly inferred from structure or indirect evidence
    E — Hypothesis or very limited basis — little to no direct evidence


PACKET MANIFEST
  generated: 2026-07-27T18:32:07Z
  task_prompt: /home/rimo/projects/Codeos/prompts/codeos-reviewer-task.md (5794 bytes)
  review_content_bytes: 53044
  estimated_review_tokens: ~13261
  budget_status: WARNING — 53044 bytes exceeds CODEOS_PACKET_BUDGET_BYTES=50000
  packet_mode: full
  delta_base: none
  items:
    - path: CLAUDE.md
      mode: full_file
      bytes: 14071
      sha256: cf95a873a95d1bc16f6f6aaf5394c024846d51a75ac5adb14d77d7eae71f5e32
    - path: /tmp/codeos-cpe-status.wTywjn
      mode: full_file
      bytes: 157
      sha256: df37304546e025a68e31c8a379add367a5eca7354d3205bb2e74ac570295244d
    - path: (diff)
      mode: full_file
      bytes: 38816

REVIEW CONTEXT
  Feature:                AC6-absent-smoke-test
  Stage:                  selfdev-step-1
  Branch:                 main
  Base commit:            (no base pin)
  Review commit:          fd2b1983a3bf21d54b19ba55f99f4da30f050609 (+ uncommitted workspace changes)
  Current approved stage: n/a (non-numeric stage)
  Evidence coverage:      FULL_COVERAGE
  Workspace dirty:        yes (uncommitted changes at review time)

DBA RULES RELEVANT TO THIS STAGE
  - Human approval is required for every stage transition; you are advisory only.
  - Memory is not truth — assess only what is provided, pinned to the review commit.
  - Implementation must trace to approved artifacts; no behavior beyond intent+contract+schema.
  - No events outside the approved event schema; no hidden behavior.

STAGE-SPECIFIC CHECKS
  - (no stage-specific checklist for stage selfdev-step-1)

EXPECTED STAGE OUTPUT
  (no expected-output template for stage)

ARTIFACTS TO REVIEW
  --- CLAUDE.md (sha256: cf95a873a95d1bc16f6f6aaf5394c024846d51a75ac5adb14d77d7eae71f5e32, visibility: shown) ---
    # Codeos Self-Development — Toolkit Operating Guide
    
    > **What this file is.** This file governs development of the **Codeos toolkit repository
    > itself** — its prompts, templates, docs, patterns, and scripts. It does **not** replace
    > the downstream DBA doctrine in [`dba-system.md`](dba-system.md), which is what
    > *downstream projects* load via `.codeos/dba-system.md`. Do not run the full 9-stage DBA
    > loop on the toolkit; use the lean self-development loop below.
    
    This is a **stable rulebook**. It contains no live workflow state. Current status lives in
    [`status/self-development.md`](status/self-development.md); per-change detail lives in
    `changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md`.
    
    All paths in this file are **repo-relative** (`prompts/`, `templates/`, `scripts/`, …).
    There is no `.codeos/` symlink inside this repo — that prefix is only for downstream projects.
    
    ---
    
    ## Mode Declaration
    
    You are developing the Codeos toolkit. Codeos *defines* Declarative Behavioral Architecture
    (DBA); it does not need to prove DBA to itself. Toolkit changes are prose (prompts,
    templates, docs, patterns) and bash tooling (the reviewer pipeline, `dba-init.sh`). There
    are no runtime events, no event schema, and no replay here — so the 9-stage loop does not
    apply. Self-development keeps DBA's **philosophy** — intent primacy, approve-the-WHAT-before-
    the-HOW, human-approved gates, advisory (never gatekeeping) review — in a leaner shape.
    
    ---
    
    ## At Session Start
    
    1. Read this file.
    2. Read [`status/self-development.md`](status/self-development.md) — the live Self-Development
       Status dashboard (what is in flight and at which step).
    3. Inspect the active `changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md` for any IN_PROGRESS row.
    4. Confirm the current state with the human, then **STOP** and ask what to work on.
    
    ---
    
    ## Truth Authority (inherited)
    
    1. **Explicit human correction** at any gate overrides everything else.
    2. **The artifacts on disk are authoritative** — never memory of them. Re-read before acting.
    3. Surface unresolved conflicts to the human rather than silently resolving them.
    
    The downstream doctrine in `dba-system.md` is the authority on DBA *substance*. When a
    self-development change touches it, see the `downstream-doctrine` rules below.
    
    ---
    
    ## Triage Front-Door
    
    Classify every change before doing anything. Class determines rigor.
    
    | Class | Path |
    |---|---|
    | `trivial` | Direct edit. No loop, no review, no change record. |
    | `backlog-only` | Direct edit, unless it changes accepted scope (then 4-step loop). |
    | `documentation` (normative) | 4-step loop. |
    | `template` / `prompt` / `script-tooling` | 4-step loop. |
    | `downstream-doctrine` | 4-step loop **+ downstream-compatibility acceptance criteria + grep cross-reference verification + reviewer scope-triage**. |
    | `self-dev-governance` | 4-step loop **+ scope-drift review** (changes to this file or the self-dev loop itself). |
    
    **`trivial` means non-semantic only:** typo, broken link, formatting, a wording
    clarification that does not change meaning, or a backlog note with no implementation.
    **Anything** that touches process, policy, behavior, script behavior, template meaning,
    prompt semantics, doctrine, stage names, approval rules, file layout, or generated-project
    layout is **non-trivial** and uses the 4-step loop. When unsure, treat it as non-trivial.
    
    **Scope axis** — every non-trivial change must declare its scope in Step 1:
    `self-dev only` | `downstream doctrine only` | `both`. This prevents accidental drift into
    the downstream master doctrine.
    
    After triage, assign a **review profile** (Step 0a of `prompts/codeos-self-dev.md`); the
    profile governs Codex review cadence and round limits.
    
    ---
    
    ## The 4-Step Self-Development Loop
    
    Anchor each non-trivial change to a `backlog/` item (create one if none exists). Use
    [`prompts/codeos-self-dev.md`](prompts/codeos-self-dev.md) for the detailed step prompt and
    [`templates/codeos-change.md`](templates/codeos-change.md) for the change record. One change
    record per non-trivial change: `changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md`.
    
    Each step: **produce output → run the Codex review if required by profile → STOP at the
    gate → human approves → next step.** (Profile governs review cadence — see Step 0a of
    `prompts/codeos-self-dev.md`.)
    
    1. **Change Intent** — Why (problem in the toolkit), what changes (named files), what stays
       the same (scope boundary), triage class, scope axis, originating backlog id. Start the
       change record; activate the row in `status/self-development.md`.
    2. **Acceptance Criteria** — The consistency contracts the change must hold.
       *Doctrine/downstream:* cross-reference integrity, downstream-compatibility (the generated
       project still loads `.codeos/dba-system.md`; stage tables, prompt filenames, and
       references move together), no internal contradiction. *Tooling:* I/O behavior, exit-code /
       fail-closed cases, idempotency.
    3. **Implement** — Edits constrained to the approved scope. Update **all** cross-references
       in the same change. No scope creep. If a change you discover is out of scope, stop and
       re-triage it as its own change.
    4. **Reconcile** — Verify each acceptance criterion. Sweep the toolkit for stale references,
       orphaned links, and stage-table↔prompt-file drift (grep). For tooling, do a smoke run.
       Apply reviewer **scope triage** (IN-SCOPE BLOCKER / IN-SCOPE NON-BLOCKER /
       OUT-OF-SCOPE BACKLOG / REJECTED / SELF-REFERENCE / REVIEW-BOOKKEEPING). Mark the row COMPLETE in `status/self-development.md`;
       log the decision (see Review Logging).
    
    ### Review cadence and advisory verdict
    
    Review cadence is governed by the **review profile** assigned in Step 0a of
    `prompts/codeos-self-dev.md`. High-risk profiles (PROFILE-3 through PROFILE-5) require
    a Codex review before each step gate. Lighter profiles (PROFILE-1, PROFILE-2) may
    limit Codex review to Reconcile only or reduce the round budget, as defined by the
    profile. Human approval at each step transition is required at every profile; reviewer
    output is advisory and non-gatekeeping at every profile.
    
    To run the reviewer:
    
    ```
    bash scripts/codeos-review.sh review UPG-####__CHG-YYYYMMDD-NNN selfdev-step-<N> changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md <touched-files>
    ```
    
    The verdict is **advisory** — NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE inform
    the human's decision but never auto-block. The reviewer is independent, read-only, and
    non-gatekeeping; the human decides at the gate.
    
    ### Gate discipline
    
    After each step output (and its review), STOP and state:
    `AWAITING HUMAN APPROVAL TO PROCEED TO STEP [N+1]`. Advance only on an explicit
    "APPROVED" / "approved" / "yes proceed" / equivalent. Anything else is a revision request.
    
    ---
    
    ## Stack / Dependency Reconciliation
    
    Evidence hygiene tied to watched files — not a second governance layer, not doctrine, and not
    an independent approval authority. `status/stack-manifest.md` records the current observed
    stack and dependency-policy status for Codeos self-development. Human approval at each gate
    remains the authority; the manifest and reconciliation reports are evidence, not authority.
    
    - Dependency/stack-file changes (watched files: `Cargo.toml`, `Cargo.lock` in this repo — see
      `status/stack-manifest.md` for the full list and which patterns actually apply here) must be
      declared in Step 1's "What changes," or explicitly re-triaged before implementation if
      discovered later.
    - If Step 1 declares a watched-file change, Step 2 must include a verification criterion for
      it.
    - Step 4 verifies that a `status/stack-reconciliation/<CHG-id>-stack-reconciliation-report.md`
      instance exists for the change, and runs `check-drift` against it.
    
    ---
    
    ## What You NEVER Do (self-development)
    
    - Treat a non-trivial change as trivial to skip the loop.
    - Advance a step without running the review required by your profile and getting explicit approval.
    - Change the downstream doctrine (`dba-system.md`) as a side effect — that requires an
      explicit `downstream-doctrine` (or `both`) scope declaration in Step 1.
    - Rewrite downstream 9-stage substance when only a path/location change is intended.
    - Put live status into this file — it belongs in `status/self-development.md`.
    - Turn `prompts/codeos-self-dev.md` into a second doctrine; keep it a practical step prompt.
    - Let the reviewer become an enforcement engine — its verdict is advisory, scope-triaged.
    
    ---
    
    ## Writing Discipline (Controlled Plain English)
    
    References, does not redefine, `patterns/controlled-plain-english.md`'s layers and reviewer model
    (see `dba-system.md` → "Controlled Plain English Writing Discipline" for the downstream side of
    this same mechanism). Self-development's own status file is `config/writing-discipline.yaml`
    (Codeos-repo-local, not project-local like the downstream file), read and injected automatically by
    `scripts/codeos-review.sh` per `UPG-0057` CHG-B — see `prompts/codeos-self-dev.md`'s Step 0b.
    
    **Layer A** (plain communication in ordinary chat, session updates, explanations) always applies,
    unconditionally, exactly as it already does everywhere else in this environment — it is not
    gated by this file's status.
    
    **Per-section rule table**, when `status: enabled`:
    
    | Change-record section | Layer applied |
    |---|---|
    | Change Intent / Acceptance Criteria / Implementation Plan | Layer B (specification precision) |
    | Implementation Notes | Factual reporting, not Layer B — this section reports what happened |
    | Review findings / Reconciliation | Layer D1 always; Layer D2 (plain review prose) when enabled |
    
    **Reviewer Model** (restated, not new authority): no separate "Controlled Plain English
    violation" finding category, and no historical-compliance audit — a reviewer reports meaning loss,
    authority mixing, or an unverifiable requirement using existing review authority, exactly as it
    does today. See the pattern's own "Reviewer Model" section for the full statement.
    
    **Placement note:** `config/writing-discipline.yaml` lives under `config/`, not `status/` — it is a
    human-set configuration toggle, not live mutable workflow state like the dashboard or roadmap.
    
    **Assumptions-subsection convention** (guidance only — no new formal field on
    `templates/codeos-change.md`): when Layer B applies and a material assumption exists that no
    existing change-record section already represents, an agent may add a plainly-labeled
    "Assumptions" subsection to Implementation Notes. Never rendered when empty; never parsed by any
    script or template validator.
    
    ---
    
    ## Self-Development File Layout
    
    ```
    Codeos/                          ← toolkit repo (this repo)
    ├── CLAUDE.md                    ← THIS FILE — stable self-development operating guide
    ├── dba-system.md                ← downstream DBA doctrine (loaded by downstream projects)
    ├── config/
    │   └── writing-discipline.yaml   ← Controlled Plain English status for self-development (see "Writing Discipline" above)
    ├── status/
    │   ├── self-development.md       ← live Self-Development Status dashboard (mutable; Feature ID + Change ID)
    │   ├── roadmap.md                ← dependency-aware wave plan, keyed by UPG-#### (mutable)
    │   ├── stack-manifest.md         ← live stack/dependency status (evidence, not authority; mutable)
    │   └── stack-reconciliation/     ← one *-stack-reconciliation-report.md per watched-file change
    ├── changes/
    │   └── UPG-####__CHG-YYYYMMDD-NNN__slug.md  ← per-change source of truth (one per non-trivial change)
    ├── backlog/
    │   ├── features.md               ← authoritative UPG-#### → file map (identity)
    │   └── UPG-####-slug.md          ← feature briefs w/ trace header + Feature Thread (feed Step 1)
    ├── prompts/                     ← stage + self-dev step prompts
    ├── templates/                   ← artifact templates
    ├── patterns/                    ← structural patterns
    ├── docs/                        ← toolkit documentation
    ├── scripts/                     ← dba-init.sh, codeos-review.sh
    └── reviews/
        ├── review-log.md             ← append-only review + decision log
        ├── architecture-journal.md   ← cross-cutting institutional memory (AJ-NNN)
        └── codex/                     ← reviewer assessments + packets
    ```
    
    **Identity & IDs (Feature Thread model).** Work is traced by stable **`UPG-####`** feature ids,
    per-execution **`CHG-YYYYMMDD-NNN`** change ids, **`REV__…__S<N>__R<N>`** review-round ids, and the
    stable **`RVS__…__S<N>`** review-series id. **Self-Reference Boundary:** reviewed artifacts carry
    `review_series` + `review_state`, never a live round — exact rounds live only in `reviews/`. The
    dashboard separates Feature ID from Change ID, and every backlog brief carries a
    `## Feature Thread` rollup. The full model is `backlog/UPG-0001-feature-thread-traceability.md`.
    In-scope review fixes stay inside the same `CHG-*`; only OUT-OF-SCOPE BACKLOG findings spawn a new
    `UPG-####` (see the Review-Fix Rule in `prompts/codeos-self-dev.md`).
    
    ---
    
    ## Review Logging
    
    When the human gives a reviewer's assessment and their decision, before any other work:
    show a brief (≈5-line) preview of what you will write, then write it.
    
    1. **One entry** to `reviews/review-log.md` (append-only) capturing the reviewer's core
       insight (close to verbatim) and the human decision separately.
    2. **One entry** to `reviews/architecture-journal.md` (`AJ-NNN`) only if the insight will
       still matter six months from now to someone who has forgotten this change. When unsure,
       journal only if future usefulness is clear.
    
    **Human overrides:** "do not log this review" / "journal this" / "do not journal this".
    
    **Fidelity:** preserve the insight verbatim; compress only context. Record conclusions and
    rationale, not conversation history. Log entries are append-only — supersede with a new
    entry, never rewrite.

  --- /tmp/codeos-cpe-status.wTywjn (sha256: df37304546e025a68e31c8a379add367a5eca7354d3205bb2e74ac570295244d, visibility: shown) ---
    Controlled Plain English status for this review: disabled
    Source: /home/rimo/projects/Codeos/config/writing-discipline.yaml
    Applicable scope: selfdev-step-1

DIFF TO REVIEW (base->review, secret/size filtered)
diff --git a/CLAUDE.md b/CLAUDE.md
index a3301a7..abe2383 100644
--- a/CLAUDE.md
+++ b/CLAUDE.md
@@ -161,12 +161,50 @@ remains the authority; the manifest and reconciliation reports are evidence, not
 
 ---
 
+## Writing Discipline (Controlled Plain English)
+
+References, does not redefine, `patterns/controlled-plain-english.md`'s layers and reviewer model
+(see `dba-system.md` → "Controlled Plain English Writing Discipline" for the downstream side of
+this same mechanism). Self-development's own status file is `config/writing-discipline.yaml`
+(Codeos-repo-local, not project-local like the downstream file), read and injected automatically by
+`scripts/codeos-review.sh` per `UPG-0057` CHG-B — see `prompts/codeos-self-dev.md`'s Step 0b.
+
+**Layer A** (plain communication in ordinary chat, session updates, explanations) always applies,
+unconditionally, exactly as it already does everywhere else in this environment — it is not
+gated by this file's status.
+
+**Per-section rule table**, when `status: enabled`:
+
+| Change-record section | Layer applied |
+|---|---|
+| Change Intent / Acceptance Criteria / Implementation Plan | Layer B (specification precision) |
+| Implementation Notes | Factual reporting, not Layer B — this section reports what happened |
+| Review findings / Reconciliation | Layer D1 always; Layer D2 (plain review prose) when enabled |
+
+**Reviewer Model** (restated, not new authority): no separate "Controlled Plain English
+violation" finding category, and no historical-compliance audit — a reviewer reports meaning loss,
+authority mixing, or an unverifiable requirement using existing review authority, exactly as it
+does today. See the pattern's own "Reviewer Model" section for the full statement.
+
+**Placement note:** `config/writing-discipline.yaml` lives under `config/`, not `status/` — it is a
+human-set configuration toggle, not live mutable workflow state like the dashboard or roadmap.
+
+**Assumptions-subsection convention** (guidance only — no new formal field on
+`templates/codeos-change.md`): when Layer B applies and a material assumption exists that no
+existing change-record section already represents, an agent may add a plainly-labeled
+"Assumptions" subsection to Implementation Notes. Never rendered when empty; never parsed by any
+script or template validator.
+
+---
+
 ## Self-Development File Layout
 
 ```
 Codeos/                          ← toolkit repo (this repo)
 ├── CLAUDE.md                    ← THIS FILE — stable self-development operating guide
 ├── dba-system.md                ← downstream DBA doctrine (loaded by downstream projects)
+├── config/
+│   └── writing-discipline.yaml   ← Controlled Plain English status for self-development (see "Writing Discipline" above)
 ├── status/
 │   ├── self-development.md       ← live Self-Development Status dashboard (mutable; Feature ID + Change ID)
 │   ├── roadmap.md                ← dependency-aware wave plan, keyed by UPG-#### (mutable)
diff --git a/backlog/UPG-0057-controlled-plain-english-writing-discipline.md b/backlog/UPG-0057-controlled-plain-english-writing-discipline.md
index d0b7d33..ff7e10f 100644
--- a/backlog/UPG-0057-controlled-plain-english-writing-discipline.md
+++ b/backlog/UPG-0057-controlled-plain-english-writing-discipline.md
@@ -87,17 +87,17 @@ explicit call-site map and Reconcile's consistency sweep.
 - Depends on `UPG-0056` (Optional Mechanism Status Convention), which must reach `COMPLETE` before
   this UPG's own Step 3 (Implement) can begin.
 
-**Status note (added after CHG-A's Step 3 review):** `CHG-A` establishes the pattern and consumer
-wiring, including a status line `codeos-reviewer-task.md` recognizes when present among reviewed
-artifacts. Discovered during `CHG-A`'s implementation: `tools/reviewer` embeds whatever artifact
-paths it is given and has no code path to read a config file on an invoker's behalf, so within
-`CHG-A` alone that line must be included by hand. **This is accepted as `CHG-A`'s scope, not as
-this discipline's final operating model.** `CHG-B` gives `scripts/codeos-review.sh` (the bash
-wrapper) the job of resolving the status automatically — for both the downstream and
-self-development branches of that one shared script — before invoking the reviewer, and of
-establishing that wrapper as the supported entry point so a direct `codeos-reviewer` invocation
-cannot silently bypass it. `tools/reviewer` itself is not touched by `CHG-B`. **`UPG-0057` is not
-considered complete until `CHG-B` reaches `COMPLETE`.**
+**Status note (updated after `CHG-B`'s Step 4 Reconciliation):** `CHG-A` established the
+Controlled Plain English pattern and consumer wiring. `CHG-B` completed automatic status delivery
+at the supported reviewer invocation boundary: `scripts/codeos-review.sh` (self-development) and
+`.codeos/scripts/codeos-review.sh` (downstream, the same shared script through the symlink) now
+resolve this project's status automatically and inject it before invoking the reviewer, for both
+the `review` and `plan` subcommands. Every currently-authoritative doctrine and prompt reference to
+running the reviewer names that wrapper, not the raw `codeos-reviewer` binary, which no longer
+serves as a supported alternative for Controlled Plain English purposes. **Manual inclusion is not
+part of the final operating model.** `tools/reviewer/src/*` was not touched by `CHG-B` — the
+injection is entirely a wrapper-level (bash) preprocessing step. **With `CHG-B` accepted, `UPG-0057`
+is complete.**
 
 ## Feature Thread
 
@@ -109,11 +109,15 @@ considered complete until `CHG-B` reaches `COMPLETE`.**
 | Change ID | File | Purpose | State |
 |---|---|---|---|
 | CHG-20260726-003 | `changes/UPG-0057__CHG-20260726-003__controlled-plain-english-writing-discipline.md` | CHG-A: pattern + downstream doctrine + prompt wiring | COMPLETE |
+| CHG-20260727-001 | `changes/UPG-0057__CHG-20260727-001__automatic-cpe-status-injection.md` | CHG-B: automatic status injection in `scripts/codeos-review.sh` (no Rust changes); wrapper established as sole supported entry point | DRAFT |
 
 ### Reviews
 
 | Review ID | Change ID | Step | Round | Verdict |
 |---|---|---|---|---|
+| RVS__UPG-0057__CHG-20260727-001__S1 | CHG-20260727-001 | 1-Intent | R1→R3 | R1 CHANGES ADVISED (Change Intent's "What changes" list omitted `status/self-development.md`'s own dashboard-bookkeeping edits; Acceptance Criteria was already fully drafted inside the Step 1 submission, blending Step 2 content into Step 1) → fixed (file added to "What changes"; AC table moved out of the Step 1 artifact to a Step 2 placeholder, draft held in scratchpad for reuse) → R2 CHANGES ADVISED (Step 2 placeholder claimed the "ten-item list quoted in Change Intent above," but only the five refinement conditions were actually quoted there — the ten-item list itself was missing) → fixed (ten-item list now quoted verbatim in condition 5 of Change Intent; placeholder wording corrected to match) → R3 NO OBJECTION |
+| RVS__UPG-0057__CHG-20260727-001__S2 | CHG-20260727-001 | 2-Acceptance | R1→R2 | R1 CHANGES ADVISED (AC3's verification only named a `--print-packet` run, silent on `plan`, though the criterion claims both subcommands; AC6's verification tested only `status: disabled`, silent on the absent-config case though the criterion claims both; AC8's grep set omitted `CLAUDE.md` despite this change's own self-dev-governance scope) → fixed (AC3/AC6 verification steps now name each case explicitly; AC8's grep set now includes `CLAUDE.md`) → R2 NO OBJECTION |
+| RVS__UPG-0057__CHG-20260727-001__S3 | CHG-20260727-001 | 3-Implement | R1→R3 | R1 CHANGES ADVISED (`docs/reviewer-pipeline.md` §10 still described the wrapper as a "static locator shim" with "no argument preprocessing," directly contradicting the implemented CPE injection logic in `scripts/codeos-review.sh`; a coverage-metadata secret-redaction flag also fired on a pre-existing, unrelated template field label — "Secret / non-secret:" in `prompts/00a-solution-discovery.md` — confirmed a false positive, no actual secret, no fix needed) → fixed (§10 rewritten to describe the locator-shim-plus-one-preprocessing-step architecture accurately) → R2 CHANGES ADVISED (the "Preview a plan before reviewing" prose still named the raw `codeos-reviewer plan` binary instead of the wrapper, missed in R1's fix pass) → fixed → R3 CHANGES ADVISED (no in-scope blocker; one IN-SCOPE NON-BLOCKER — a stray "§13 below" cross-reference for Controlled Plain English status, should say §12a; LOG SUMMARY driven only by the recurring benign `SECRET_REDACTION` false positive, not a real blocker) → fixed inline. **PROFILE-5's 3-round/step budget is now exhausted for Step 3** — fix applied per CLAUDE.md's budget-exceeded rule; no further automatic round; escalated to human decision |
 | RVS__UPG-0057__CHG-20260726-003__S1 | CHG-20260726-003 | 1-Intent | R1 | NO OBJECTION |
 | RVS__UPG-0057__CHG-20260726-003__S2 | CHG-20260726-003 | 2-Acceptance | R1→R2 | R1 DO NOT ADVANCE (AC3/AC4/AC18 depended on external plan content not in the packet; AC18's "cannot silently reintroduce" overstated an advisory generation discipline as enforcement) → fixed → R2 NO OBJECTION |
 | RVS__UPG-0057__CHG-20260726-003__S3 | CHG-20260726-003 | 3-Implement | R1→R3 | R1 DO NOT ADVANCE (`codeos-reviewer-task.md` falsely claimed as a pattern consumer able to enforce the pattern-unavailable check; File Layout "none by default" contradicted `dba-init.sh`'s own scaffolding; "Thirteen files" undercounted the actual 18) → fixed → R2 NO OBJECTION → revised post-R2 per human direction (scaffolded default changed `disabled`→`enabled`) → R3 DO NOT ADVANCE (Implementation Notes' own file-list sentence still said `status: disabled` for `dba-init.sh`, contradicting AC13's own "no remaining stale claim" text) → fixed. **PROFILE-4's 3-round/step budget is now exhausted for Step 3** — fix applied inline per CLAUDE.md's budget-exceeded rule; no further automatic round run; escalated to human decision |
diff --git a/dba-system.md b/dba-system.md
index 7c6f5d0..f0bf997 100644
--- a/dba-system.md
+++ b/dba-system.md
@@ -50,13 +50,22 @@ that Codeos's own toolkit development already holds itself to.
 
 **How to run it.** Before each gate's human-approval decision, run:
 ```
-codeos-reviewer review <feature_id> <stage>
+.codeos/scripts/codeos-review.sh review <feature_id> <stage>
 ```
-using the Stage ID from the table above (e.g. `codeos-reviewer review checkout-flow 2` before
-approving Stage 2's contract; `codeos-reviewer review checkout-flow brief` before confirming
-a Feature Brief). The reviewer is independent, read-only, and non-gatekeeping — its verdict
-(NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE) informs the human's decision but never
-auto-blocks. **The human decides at the gate; Non-Negotiable Rule #1 is unchanged.**
+using the Stage ID from the table above (e.g. `.codeos/scripts/codeos-review.sh review
+checkout-flow 2` before approving Stage 2's contract; `.codeos/scripts/codeos-review.sh review
+checkout-flow brief` before confirming a Feature Brief). The reviewer is independent, read-only,
+and non-gatekeeping — its verdict (NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE) informs the
+human's decision but never auto-blocks. **The human decides at the gate; Non-Negotiable Rule #1
+is unchanged.**
+
+**The wrapper is the supported entry point.** `.codeos/scripts/codeos-review.sh` (downstream) and
+`scripts/codeos-review.sh` (Codeos's own self-development) automatically resolve and inject this
+project's Controlled Plain English status (see "Controlled Plain English Writing Discipline"
+below) before invoking the reviewer. Invoking the compiled `codeos-reviewer` binary directly
+bypasses that injection — it still runs, but it is not a supported alternative for Controlled
+Plain English purposes, since the shared reviewer template never reads any config file itself
+(see the Call-site map below).
 
 **Round budget.** Round 1 runs before the gate. Rounds 2-3 are allowed for fixes or material
 deltas raised by the previous round. After 3 rounds, stop and require a human decision rather
@@ -283,8 +292,9 @@ ever accepts the current version of *both* artifacts; historical files matter fo
 the non-retroactive protection above, not for gating new work.
 
 **Reviewer coverage.** `codeos-reviewer` has a dedicated checklist for the `architecture-synthesis`
-stage id, covering all four pipeline steps — run `codeos-reviewer review <feature_id>
-architecture-synthesis` the same way as any other stage, per "Default Advisory Review" above. This
+stage id, covering all four pipeline steps — run `.codeos/scripts/codeos-review.sh review
+<feature_id> architecture-synthesis` the same way as any other stage, per "Default Advisory
+Review" above. This
 does not weaken Non-Negotiable Rule #1 — the human still explicitly approves both the baseline and
 the logical design.
 
@@ -453,7 +463,7 @@ are that convention's, unchanged.
 |---|---|
 | Stage 1-10 prompts (`.codeos/prompts/01-intent.md` … `10-arch-refine.md`) | `architecture/controlled-plain-english.yaml` |
 | `.codeos/prompts/pipeline-reviewer.md` | `architecture/controlled-plain-english.yaml` |
-| `.codeos/prompts/codeos-reviewer-task.md` (shared reviewer infrastructure) | **Reads neither file — configuration-neutral.** Whoever invokes the review (a human, or an agent under human instruction) includes a fixed status line — "Controlled Plain English status for this review: enabled/disabled" — as one of the reviewed artifacts, the same way any other file is passed to `codeos-reviewer review`. There is no automatic packet-assembly step that reads a config file on the invoker's behalf; `tools/reviewer` embeds whatever artifact paths it is given, unchanged. |
+| `.codeos/prompts/codeos-reviewer-task.md` (shared reviewer infrastructure) | **Reads neither file — configuration-neutral.** `.codeos/scripts/codeos-review.sh` resolves this project's status automatically and appends a synthetic status artifact — "Controlled Plain English status for this review: enabled/disabled" plus its config source and applicable stage — to the packet before invoking the reviewer, the same way any other file is passed to `codeos-reviewer review`. `codeos-reviewer-task.md` still never reads a config file itself; it only recognizes the already-resolved line among the reviewed artifacts. `tools/reviewer` is unchanged — it still embeds whatever artifact paths it is given; the wrapper is what supplies this one automatically now. |
 
 **What is and isn't toggle-gated.** Layer A (plain communication) and Layer C1/D1 (existing
 literal-protection and reviewer-integrity authority) are **not** new mandatory rules and are never
@@ -471,8 +481,8 @@ of its own.
 
 Use the corresponding prompt file from `.codeos/prompts/` for detailed instructions. The
 **Stage ID** column is the identifier vocabulary used both for documentation ordering and as
-the `<stage>` argument to `codeos-reviewer review <feature_id> <stage>` — see "Default
-Advisory Review" below.
+the `<stage>` argument to `.codeos/scripts/codeos-review.sh review <feature_id> <stage>` — see
+"Default Advisory Review" below.
 
 | Stage | Stage ID | File |
 |---|---|---|
diff --git a/docs/reviewer-pipeline.md b/docs/reviewer-pipeline.md
index 67ec4da..b8b7ca6 100644
--- a/docs/reviewer-pipeline.md
+++ b/docs/reviewer-pipeline.md
@@ -421,27 +421,31 @@ sessions feature-scoped · reviewed state pinned (base+review SHA, artifact hash
 output → UNCLASSIFIED/high-attention · secret/large-diff filtering present · no hooks active ·
 no core rules changed.
 
-## 10. Architecture: `codeos-review.sh` is a static locator shim
+## 10. Architecture: `codeos-review.sh` is a locator shim plus one preprocessing step
 
 ```bash
-# scripts/codeos-review.sh — final line; everything above it only locates the binary
-exec "${BINARY}" "$@"
+# scripts/codeos-review.sh — for review/plan, resolves + injects a CPE status artifact,
+# then invokes the binary as a subprocess (not exec, so its own cleanup trap can still
+# fire); every other subcommand still ends in the original: exec "${BINARY}" "$@"
 ```
 
-`codeos-review.sh` is a **~28-line static locator shim** (see `UPG-0038` for why it isn't
-shorter: a caller-git-repository precondition, plus script-relative binary-path resolution
-that works correctly through the `.codeos` symlink from a downstream project, plus a PATH
-fallback if the compiled binary isn't found at its expected location). It finds the compiled
-Rust binary (`tools/reviewer/target/release/codeos-reviewer`) and passes all arguments
-through verbatim (`"$@"`). Its conditionals are entirely about *locating the binary and
-validating preconditions* — it contains no argument preprocessing and no reviewer capability
-of its own.
-
-**Consequence for upgrades:** any reviewer capability change — new packet sections, new
-subcommand behavior, new flags, new decision-log fields — lives in the **Rust engine**
-(`tools/reviewer/src/`). Changing only the bash script cannot add or modify reviewer
-behavior. The bash script only needs to change if binary location, build instructions, or
-path-resolution semantics change.
+`codeos-review.sh` is a **locator shim** (see `UPG-0038` for why it isn't shorter: a
+caller-git-repository precondition, plus script-relative binary-path resolution that works
+correctly through the `.codeos` symlink from a downstream project, plus a PATH fallback if
+the compiled binary isn't found at its expected location) **plus, since `UPG-0057` CHG-B,
+one argument-preprocessing step**: for the `review` and `plan` subcommands only, it resolves
+this project's Controlled Plain English status and appends a synthetic status artifact to
+the argument list before invoking the binary (§12a). Every other subcommand (`decision`,
+`diagnose`, `stage-start`, `check-drift`, `generate-*`) is still passed through with no
+preprocessing at all, ending in the original `exec "${BINARY}" "$@"`.
+
+**Consequence for upgrades:** any reviewer *capability* change — new packet sections, new
+subcommand behavior, new flags, new decision-log fields, anything the Rust engine itself must
+parse or act on — still lives in the **Rust engine** (`tools/reviewer/src/`); the wrapper's one
+preprocessing step only ever appends an ordinary artifact path that the engine already knows how
+to embed, it does not teach the engine anything new. The bash script needs to change for: binary
+location, build instructions, path-resolution semantics, or the Controlled Plain English
+injection logic itself (§12a).
 
 ## 11. Usage
 
@@ -498,9 +502,43 @@ Reviewing a Feature Brief before confirming it:
 ```bash
 .codeos/scripts/codeos-review.sh review checkout-flow brief backlog/checkout-flow.md
 ```
-Direct binary invocation (`/path/to/Codeos/tools/reviewer/target/release/codeos-reviewer
-...`, where `/path/to/Codeos` is wherever `.codeos` resolves to — check with `readlink -f
-.codeos`) still works identically and remains a valid alternative.
+**`.codeos/scripts/codeos-review.sh` (or `scripts/codeos-review.sh` for Codeos's own
+self-development) is the supported entry point**, not a convenience wrapper among several equally
+valid options. Since `UPG-0057` CHG-B, it automatically resolves and injects this project's
+Controlled Plain English status (§12a below) before invoking the reviewer. Direct binary invocation
+(`/path/to/Codeos/tools/reviewer/target/release/codeos-reviewer ...`, where `/path/to/Codeos` is
+wherever `.codeos` resolves to — check with `readlink -f .codeos`) still runs identically, but
+skips that injection step entirely — it is **not** a supported alternative for Controlled Plain
+English purposes, since `codeos-reviewer-task.md` never reads any config file itself and depends
+on the wrapper to supply the resolved status line.
+
+## 12a. Controlled Plain English automatic status injection (`UPG-0057` CHG-B)
+
+Before invoking the reviewer for a `review` or `plan` subcommand (the two subcommands that build a
+packet), the wrapper:
+
+1. **Resolves context** — if the caller's own git root is the Codeos repo itself, this is a
+   self-development review and the wrapper reads `config/writing-discipline.yaml` (relative to the
+   Codeos repo root); otherwise it's a downstream review and the wrapper reads
+   `architecture/controlled-plain-english.yaml` relative to the *caller's* git root (not through
+   `.codeos`).
+2. **Resolves the four-outcome status** per `UPG-0056`'s Optional Mechanism Status Convention:
+   absent or exact `status: disabled` → `disabled`; exact `status: enabled` → `enabled`; anything
+   else → a configuration error.
+3. **On a valid status** (`enabled` or `disabled`), writes a synthetic, deterministic temp file
+   (recognizable name `codeos-cpe-status.*`, created with `mktemp`, cleaned up via a `trap` on both
+   the success and failure paths) containing the exact line `codeos-reviewer-task.md` recognizes,
+   plus its source config path and the stage argument, then appends that file's path to the
+   packet's artifact list — no operator-supplied status path is ever required.
+4. **On a malformed or contradictory status file**, stops with a clear error *before* invoking the
+   reviewer (exit code 7) — this is an invocation precondition failure, the same class as "binary
+   not found," never a reviewer finding.
+
+Ordinary style non-compliance in the reviewed prose itself is entirely unaffected: it remains an
+advisory reviewer finding under existing authority, never a packet-generation failure. A `disabled`
+(or absent/not-applicable) status is injected and reviewed normally — it never blocks anything.
+`tools/reviewer/src/*` is unchanged by this mechanism; the wrapper only appends an ordinary
+artifact path that the Rust engine already knows how to embed.
 
 **If reviewer tooling isn't built or configured** for a downstream project, see
 `dba-system.md`'s Review Waiver practice — record a plain reason in that feature's review
@@ -567,7 +605,7 @@ Includes full artifact content where allowed by packet size and redaction rules.
 
 **Command:**
 ```bash
-codeos-reviewer review <feature> <stage> <artifact-paths>
+.codeos/scripts/codeos-review.sh review <feature> <stage> <artifact-paths>
 ```
 
 ### Delta Mode
@@ -581,7 +619,7 @@ Includes only changes since a base commit. Unchanged artifacts are represented b
 
 **Command:**
 ```bash
-codeos-reviewer review <feature> <stage> --mode delta --base <commit-sha> <artifact-paths>
+.codeos/scripts/codeos-review.sh review <feature> <stage> --mode delta --base <commit-sha> <artifact-paths>
 ```
 
 **Guardrail:** Delta mode requires artifact paths to be tracked by git. Untracked files cannot be compared to the base commit and will error.
@@ -597,7 +635,7 @@ Includes only the file path and hash, not file content. **This reduces packet si
 
 **Command:**
 ```bash
-codeos-reviewer review <feature> <stage> --sha-only <context-file> <other-artifacts>
+.codeos/scripts/codeos-review.sh review <feature> <stage> --sha-only <context-file> <other-artifacts>
 ```
 
 **Guardrail:** Do not use SHA-only for files whose changed behavior, wording, or structure the reviewer must assess. Changed behavior must remain reviewable as full content or diff.
@@ -607,7 +645,7 @@ codeos-reviewer review <feature> <stage> --sha-only <context-file> <other-artifa
 Delta mode and SHA-only can be combined. When both apply, SHA-only paths are included as path/hash references rather than full content or diff.
 
 ```bash
-codeos-reviewer review UPG-0042 selfdev-step-3 \
+scripts/codeos-review.sh review UPG-0042 selfdev-step-3 \
   --mode delta --base abc123 \
   --sha-only docs/large-reference.md \
   changes/UPG-0042__CHG-*.md src/packet.rs
@@ -615,7 +653,7 @@ codeos-reviewer review UPG-0042 selfdev-step-3 \
 
 ### Preview a plan before reviewing
 
-`codeos-reviewer plan` accepts the exact same arguments as `review` (feature, stage,
+`codeos-review.sh plan` (like `review`, resolved through the wrapper — §12a) accepts the exact same arguments as `review` (feature, stage,
 artifacts, `--mode`/`--base`, `--sha-only`) and reports what a `review` call with those
 arguments would send — resolved artifacts with their mode and byte size, `review_content_bytes`
 vs. the packet budget, `estimated_review_tokens`, coverage state, and (when over budget) the
@@ -624,7 +662,7 @@ same `packet::build()` function `review`/`--print-packet` use, so it cannot desc
 `review` wouldn't actually build.
 
 ```bash
-codeos-reviewer plan UPG-0042 selfdev-step-1 changes/UPG-0042__CHG-*.md src/packet.rs
+scripts/codeos-review.sh plan UPG-0042 selfdev-step-1 changes/UPG-0042__CHG-*.md src/packet.rs
 ```
 
 `plan` never resolves or invokes a provider and never writes to `reviews/` or any other tracked
diff --git a/patterns/controlled-plain-english.md b/patterns/controlled-plain-english.md
index 18bc282..b695128 100644
--- a/patterns/controlled-plain-english.md
+++ b/patterns/controlled-plain-english.md
@@ -15,8 +15,9 @@ layers below apply to it and applies the enabled-but-pattern-unavailable rule (b
 `codeos-reviewer-task.md` is **not** a consumer of this file — it stays configuration-neutral (see
 "Reviewer Model" and `dba-system.md`'s call-site map): it never reads this pattern or any status
 file itself, and therefore never performs the enabled-but-pattern-unavailable check. It only reacts
-to a status line manually included by whoever invokes the review, using Layer D2's rule text, which
-is restated inline in `codeos-reviewer-task.md` itself rather than requiring it to read this file.
+to a status line automatically injected by `scripts/codeos-review.sh` (or `.codeos/scripts/
+codeos-review.sh` downstream) before the reviewer is invoked, using Layer D2's rule text, which is
+restated inline in `codeos-reviewer-task.md` itself rather than requiring it to read this file.
 This pattern's Layer B/C2/D2 rules are toggle-gated (see "Layers," below); Layer A/C1/D1 are always
 active regardless of the toggle, since they restate expectations that already exist elsewhere in
 this environment or in Codeos's own Non-Negotiable Rules — naming them here consolidates them, it
diff --git a/prompts/00a-solution-discovery.md b/prompts/00a-solution-discovery.md
index 9fc93d5..33e4eec 100644
--- a/prompts/00a-solution-discovery.md
+++ b/prompts/00a-solution-discovery.md
@@ -204,7 +204,7 @@ To continue with feature work:
    specification. Stage 1 (Intent Capture) is the authoritative entry point.
 
 **If this output is carried into a Feature Brief or a Stage 1 Intent**, that handoff gets
-the default advisory review (`codeos-reviewer review <feature_id> discovery`) or an explicit
+the default advisory review (`.codeos/scripts/codeos-review.sh review <feature_id> discovery`) or an explicit
 Review Waiver — see `dba-system.md`'s "Default Advisory Review" section. This session
 itself stays optional and non-gating either way; a Discovery session whose output nobody
 carries forward is never reviewed, because there is nothing yet to review.
diff --git a/prompts/03b-architecture-synthesis.md b/prompts/03b-architecture-synthesis.md
index 734b51e..30de51a 100644
--- a/prompts/03b-architecture-synthesis.md
+++ b/prompts/03b-architecture-synthesis.md
@@ -174,6 +174,7 @@ Output: confirmation of both approved artifacts + registry update +
 ## Reviewer Note
 
 `codeos-reviewer` has a dedicated checklist for the `architecture-synthesis` stage id, covering all
-four steps of this pipeline — run `codeos-reviewer review <feature_id> architecture-synthesis` for
-gate reviews at this stage, per "Default Advisory Review" in `dba-system.md`. This does not weaken
+four steps of this pipeline — run `.codeos/scripts/codeos-review.sh review <feature_id>
+architecture-synthesis` for gate reviews at this stage, per "Default Advisory Review" in
+`dba-system.md`. This does not weaken
 the requirement for explicit human approval at each step above.
diff --git a/prompts/codeos-self-dev.md b/prompts/codeos-self-dev.md
index a2fa67a..9b0e103 100644
--- a/prompts/codeos-self-dev.md
+++ b/prompts/codeos-self-dev.md
@@ -61,6 +61,26 @@ Record `review_profile: PROFILE-N` in the change record trace header.
 
 ---
 
+## Step 0b — Writing Discipline Check
+
+Read `config/writing-discipline.yaml` (Codeos-repo-local). Per the Optional Mechanism Status
+Convention's four-outcome table (`templates/conventions.md`): absent or exact `status: disabled` →
+disabled; exact `status: enabled` → enabled; anything else → stop and report a configuration error.
+
+When enabled, apply `CLAUDE.md`'s "Writing Discipline (Controlled Plain English)" per-section rule
+table to this change's own artifacts (Layer B for Change Intent / Acceptance Criteria /
+Implementation Plan; factual reporting for Implementation Notes; Layer D1 always + D2 when enabled
+for review findings and Reconciliation). No new change-record trace-header field is added for
+this — non-retroactivity is the one-sentence rule already stated in
+`patterns/controlled-plain-english.md`, nothing to stamp per change.
+
+`scripts/codeos-review.sh` reads this same file automatically and injects its resolved status into
+every review packet built for a self-development change — see `docs/reviewer-pipeline.md §12a`.
+This step is about applying the discipline while *writing* the artifact; the wrapper's injection is
+about what the *reviewer* is told, a separate but related mechanism.
+
+---
+
 ## The 4-Step Loop
 
 Each step requires explicit human approval before the next.
diff --git a/prompts/pipeline-reviewer.md b/prompts/pipeline-reviewer.md
index 7043c06..90cd033 100644
--- a/prompts/pipeline-reviewer.md
+++ b/prompts/pipeline-reviewer.md
@@ -4,7 +4,7 @@ Paste this at the start of a reviewer LLM session before providing a stage artif
 
 **This is an optional, supplementary second opinion** — an independent critical-assessor
 pass free to challenge the artifact, the feature, or DBA itself. It does not replace the
-default advisory review (`codeos-reviewer review <feature_id> <stage>`, structured and
+default advisory review (`.codeos/scripts/codeos-review.sh review <feature_id> <stage>`, structured and
 acceptance-criteria-bound) described in `dba-system.md`'s "Default Advisory Review" section.
 Use both when a stage warrants extra scrutiny; use this alone only when the default review
 tooling isn't available and a waiver has been recorded but a human still wants a second read.
diff --git a/scripts/codeos-review.sh b/scripts/codeos-review.sh
index 5bf00a7..44983a9 100755
--- a/scripts/codeos-review.sh
+++ b/scripts/codeos-review.sh
@@ -1,7 +1,14 @@
 #!/usr/bin/env bash
-# codeos-review.sh — thin shim delegating to the compiled Rust binary.
-# Subcommands: review / decision / diagnose / stage-start  (see: codeos-reviewer --help)
+# codeos-review.sh — thin shim delegating to the compiled Rust binary, with automatic
+# Controlled Plain English status injection for the two packet-building subcommands.
+# Subcommands: review / plan / decision / diagnose / stage-start / check-drift / generate-*
+# (see: codeos-reviewer --help)
 # To build: cargo build --release --manifest-path tools/reviewer/Cargo.toml
+#
+# Exit codes: 1 = not a git repo, 2 = binary not found, 7 = Controlled Plain English
+# status file is malformed (see "Controlled Plain English automatic status injection"
+# below). Codes 0/3/4/5/6 belong to the Rust binary itself (tools/reviewer/src/main.rs)
+# and are passed through unchanged.
 set -euo pipefail
 # Preserve the original precondition: the shim requires the CALLER to be inside some git
 # repository (a property of the project being reviewed) — unrelated to where the binary
@@ -25,4 +32,75 @@ if [[ ! -x "${BINARY}" ]]; then
     exit 2
   }
 fi
+
+SUBCOMMAND="${1:-}"
+
+# ── Controlled Plain English automatic status injection ────────────────────────────────
+# Only "review" and "plan" build a reviewer packet (see tools/reviewer/src/main.rs); every
+# other subcommand (decision, diagnose, stage-start, check-drift, generate-*) is passed
+# through unchanged below, exactly as before this change.
+if [[ "${SUBCOMMAND}" == "review" || "${SUBCOMMAND}" == "plan" ]]; then
+  # Context resolution: this one shared script is reached two ways — directly as
+  # scripts/codeos-review.sh from within this repo (a self-development review), or as
+  # .codeos/scripts/codeos-review.sh through the downstream symlink (a downstream
+  # project's review). The caller's own git root (already required to exist by the
+  # precondition above) tells them apart: it equals CODEOS_ROOT only in the former case.
+  CALLER_ROOT="$(cd "$(git rev-parse --show-toplevel)" && pwd -P)"
+  STAGE_ARG="${3:-}"
+
+  if [[ "${CALLER_ROOT}" == "${CODEOS_ROOT}" ]]; then
+    CPE_CONFIG="${CODEOS_ROOT}/config/writing-discipline.yaml"
+  else
+    CPE_CONFIG="${CALLER_ROOT}/architecture/controlled-plain-english.yaml"
+  fi
+
+  # Four-outcome resolution, per UPG-0056's Optional Mechanism Status Convention
+  # (templates/conventions.md): absent -> disabled; exact "status: disabled" -> disabled;
+  # exact "status: enabled" -> enabled; anything else -> configuration error. Leading/
+  # trailing blank lines are allowed and line endings are normalized (CRLF -> LF) before
+  # comparison; internal whitespace, case, tabs, and comments are NOT normalized and make
+  # the file invalid, exactly like every other consumer of this convention.
+  CPE_STATUS=""
+  if [[ ! -f "${CPE_CONFIG}" ]]; then
+    CPE_STATUS="disabled"
+  else
+    CPE_NONBLANK=()
+    while IFS= read -r _cpe_line || [[ -n "${_cpe_line}" ]]; do
+      [[ -n "${_cpe_line}" ]] && CPE_NONBLANK+=("${_cpe_line}")
+    done < <(tr -d '\r' < "${CPE_CONFIG}")
+
+    if [[ "${#CPE_NONBLANK[@]}" -eq 1 && "${CPE_NONBLANK[0]}" == "status: disabled" ]]; then
+      CPE_STATUS="disabled"
+    elif [[ "${#CPE_NONBLANK[@]}" -eq 1 && "${CPE_NONBLANK[0]}" == "status: enabled" ]]; then
+      CPE_STATUS="enabled"
+    else
+      # Malformed or contradictory configuration: fail BEFORE invoking the reviewer. This
+      # is an invocation precondition failure (same class as "binary not found" above),
+      # never a reviewer finding — ordinary style non-compliance in generated prose is
+      # always and only a reviewer finding, never a packet-generation failure.
+      echo "error: invalid Controlled Plain English status file: ${CPE_CONFIG}" >&2
+      echo "       must contain exactly one non-blank line: 'status: enabled' or 'status: disabled'" >&2
+      exit 7
+    fi
+  fi
+
+  # The generated artifact is explicitly synthetic: a recognizable filename
+  # (codeos-cpe-status.*), a deterministic three-line body, and safe temp-file handling
+  # (mktemp, quoted paths, trap cleanup covering both the success and failure paths).
+  # NOTE: this branch cannot end in `exec` (see below) — a bash EXIT trap never fires
+  # across exec, since exec replaces the process image instead of letting the shell
+  # return to run its traps. To still guarantee cleanup, the binary is invoked as an
+  # ordinary subprocess here and this script exits with its exact exit code afterward.
+  CPE_STATUS_FILE="$(mktemp "${TMPDIR:-/tmp}/codeos-cpe-status.XXXXXX")"
+  trap 'rm -f "${CPE_STATUS_FILE}"' EXIT
+  {
+    printf 'Controlled Plain English status for this review: %s\n' "${CPE_STATUS}"
+    printf 'Source: %s\n' "${CPE_CONFIG}"
+    printf 'Applicable scope: %s\n' "${STAGE_ARG}"
+  } > "${CPE_STATUS_FILE}"
+
+  "${BINARY}" "$@" "${CPE_STATUS_FILE}"
+  exit $?
+fi
+
 exec "${BINARY}" "$@"
diff --git a/status/self-development.md b/status/self-development.md
index 86fc334..b6d21b4 100644
--- a/status/self-development.md
+++ b/status/self-development.md
@@ -78,7 +78,8 @@
 | UPG-0055 | CHG-20260720-001 | script-tooling | self-dev only | 4-Reconcile | ACCEPTED (series RVS__…__S4; Step 1 R1 NO OBJECTION; Step 2 R1 NO OBJECTION; Step 3 R1 DO NOT ADVANCE → R2 DO NOT ADVANCE → R3 NO OBJECTION; Step 4 R1 DO NOT ADVANCE → R2 NO OBJECTION; all 9 ACs verified; 182 tests pass; 4 in-scope blockers found+fixed) | COMPLETE | — |
 | UPG-0056 | CHG-20260726-001 | downstream-doctrine | downstream doctrine only | 4-Reconcile | ACCEPTED (series RVS__…__S4; Step 1 R1-R3 DO NOT ADVANCE→NO OBJECTION (original design) then revised to lean convention, R4 DO NOT ADVANCE→APPROVE_STAGE; Step 2 R1 DO NOT ADVANCE→R2 NO OBJECTION; Step 3 R1-R2 DO NOT ADVANCE→R3 NO OBJECTION; Step 4 R1 DO NOT ADVANCE→R2 NO OBJECTION; all 12 ACs verified; no code shipped) | COMPLETE | UPG-0057 |
 | UPG-0058 | CHG-20260726-002 | downstream-doctrine | downstream doctrine only | 4-Reconcile | ACCEPTED (series RVS__…__S4; Step 1 R1→R2 NO OBJECTION; Step 2 R1 NO OBJECTION; Step 3 R1→R2 DO NOT ADVANCE→R3 NO OBJECTION; Step 4 R1→R2 DO NOT ADVANCE→R3 NO OBJECTION; all 16 ACs verified; 182 tests pass; 7 in-scope blockers found+fixed across the change) | COMPLETE | — |
-| UPG-0057 | CHG-20260726-003 | downstream-doctrine | downstream doctrine only | 4-Reconcile | CHG-A; Steps 1-3 ACCEPTED (Step 3 revised per human direction: default disabled→enabled, plus interim-state framing for status-line delivery); Step 4 R1 DO NOT ADVANCE (stale historical disabled-default note)→fixed→R2 NO OBJECTION — accepted for pattern+wiring scope only; awaiting human Step 4 gate decision; UPG-0057 as a whole not complete until CHG-B (bash-level automatic status injection) lands | IN_PROGRESS | — |
+| UPG-0057 | CHG-20260726-003 | downstream-doctrine | downstream doctrine only | 4-Reconcile | CHG-A ACCEPTED and committed (fd2b198) — pattern + downstream doctrine + prompt wiring; manual status-line inclusion accepted as CHG-A-scope only, not final operating model | COMPLETE | UPG-0057 (CHG-B) |
+| UPG-0057 | CHG-20260727-001 | script-tooling + downstream-doctrine + self-dev-governance | both | 4-Reconcile | CHG-B: automatic Controlled Plain English status injection implemented in `scripts/codeos-review.sh` (no Rust changes, 182 tests unchanged); wrapper established as supported entry point; Step 3 ACCEPTED by human decision (budget exhausted, no in-scope blocker, SECRET_REDACTION confirmed false positive recorded in Implementation Notes); Step 4 Reconciliation written, all 13 ACs PASS; review pending | IN_PROGRESS | — |
 
 <!--
 Identity   : Feature ID = UPG-#### (stable). Change ID = CHG-YYYYMMDD-NNN (one execution), or a

codex
I’m reading the packet as a stage review, not a code-change task. First pass is to pin the claimed scope and guarantees from the included artifacts, then I’ll check whether the diff stays inside that boundary and whether any stated guarantees are unsupported or contradicted.
exec
/bin/bash -lc "nl -ba docs/reviewer-pipeline.md | sed -n '421,670p'" in /home/rimo/projects/Codeos
 succeeded in 0ms:
   421	output → UNCLASSIFIED/high-attention · secret/large-diff filtering present · no hooks active ·
   422	no core rules changed.
   423	
   424	## 10. Architecture: `codeos-review.sh` is a locator shim plus one preprocessing step
   425	
   426	```bash
   427	# scripts/codeos-review.sh — for review/plan, resolves + injects a CPE status artifact,
   428	# then invokes the binary as a subprocess (not exec, so its own cleanup trap can still
   429	# fire); every other subcommand still ends in the original: exec "${BINARY}" "$@"
   430	```
   431	
   432	`codeos-review.sh` is a **locator shim** (see `UPG-0038` for why it isn't shorter: a
   433	caller-git-repository precondition, plus script-relative binary-path resolution that works
   434	correctly through the `.codeos` symlink from a downstream project, plus a PATH fallback if
   435	the compiled binary isn't found at its expected location) **plus, since `UPG-0057` CHG-B,
   436	one argument-preprocessing step**: for the `review` and `plan` subcommands only, it resolves
   437	this project's Controlled Plain English status and appends a synthetic status artifact to
   438	the argument list before invoking the binary (§12a). Every other subcommand (`decision`,
   439	`diagnose`, `stage-start`, `check-drift`, `generate-*`) is still passed through with no
   440	preprocessing at all, ending in the original `exec "${BINARY}" "$@"`.
   441	
   442	**Consequence for upgrades:** any reviewer *capability* change — new packet sections, new
   443	subcommand behavior, new flags, new decision-log fields, anything the Rust engine itself must
   444	parse or act on — still lives in the **Rust engine** (`tools/reviewer/src/`); the wrapper's one
   445	preprocessing step only ever appends an ordinary artifact path that the engine already knows how
   446	to embed, it does not teach the engine anything new. The bash script needs to change for: binary
   447	location, build instructions, path-resolution semantics, or the Controlled Plain English
   448	injection logic itself (§12a).
   449	
   450	## 11. Usage
   451	
   452	```bash
   453	# record the base commit for a stage (so review diffs base->review, not just HEAD)
   454	scripts/codeos-review.sh stage-start listing-ingestion 2
   455	
   456	# review an artifact (resumes the feature's Codex session; --fresh starts a new one)
   457	scripts/codeos-review.sh review listing-ingestion 2 contracts/listing-ingestion_contract.md
   458	
   459	# after the human decides, append the decision (never edits prior log entries)
   460	scripts/codeos-review.sh decision listing-ingestion 2 REQUEST_CHANGES "missing failure scenario"
   461	```
   462	
   463	**Local prechecks** run automatically before the packet is built and before any Codex
   464	invocation. They scan only the positional artifact paths passed to `review`. Two hard-fail
   465	checks exit non-zero immediately: (1) a literal unfilled template placeholder (`UPG-####` or
   466	`CHG-YYYYMMDD-NNN`) in any artifact, and (2) a line-anchored `latest_review:` field (a
   467	schema field superseded by UPG-0001). A warning is emitted to stderr (but Codex is still
   468	invoked) for unresolved draft markers: `TODO`, `FIXME`, `TBD`, `[to be filled]`. Pass
   469	`--guard-clean PATH` (repeatable) to assert that a specific file — e.g. `dba-system.md`
   470	during a `self-dev only` change — has no uncommitted changes; a non-existent path or a dirty
   471	path both exit non-zero before Codex. Pass `--skip-prechecks` to bypass all checks (emits a
   472	visible `warning: prechecks skipped` to stderr); useful for inspecting draft artifacts with
   473	`--print-packet`.
   474	
   475	---
   476	
   477	## 12. Downstream usage (DBA projects, not Codeos self-development)
   478	
   479	Everything above documents this pipeline from Codeos's own self-development perspective. A
   480	downstream project — one that ran `dba-init.sh` and loads `.codeos/dba-system.md` — uses the
   481	exact same `codeos-reviewer` binary and `review`/`decision`/`diagnose` subcommands, with two
   482	differences:
   483	
   484	1. **Stage identifiers are the downstream Stage IDs**, not `selfdev-step-N`: `discovery`,
   485	   `brief`, `onboarding`, `1` through `9`, and `10` — see `dba-system.md`'s "What You Do at
   486	   Each Stage" table for the full mapping and "Default Advisory Review" for when each is used.
   487	2. **Cadence is the flat rule in `dba-system.md`'s "Default Advisory Review" section** —
   488	   round 1 before the gate, rounds 2-3 for fixes/deltas, stop after 3 and escalate to a human.
   489	   This is a separate, uniform cadence from the review-round-budget table used for triaging
   490	   Codeos's own toolkit changes (§4d above) — that internal triage system never appears in
   491	   downstream-facing doctrine or prompts.
   492	
   493	**Invoking the shim from a downstream project.** `.codeos/scripts/codeos-review.sh` resolves
   494	its binary path from the script's own physical location (following the `.codeos` symlink
   495	through to Codeos), so it works correctly from within a downstream project (fixed by
   496	`UPG-0038`; previously it resolved via the *calling* project's git root instead, which broke
   497	under a symlinked invocation). E.g. reviewing a Stage 2 contract:
   498	```bash
   499	.codeos/scripts/codeos-review.sh review checkout-flow 2 contracts/checkout-flow_contract.md
   500	```
   501	Reviewing a Feature Brief before confirming it:
   502	```bash
   503	.codeos/scripts/codeos-review.sh review checkout-flow brief backlog/checkout-flow.md
   504	```
   505	**`.codeos/scripts/codeos-review.sh` (or `scripts/codeos-review.sh` for Codeos's own
   506	self-development) is the supported entry point**, not a convenience wrapper among several equally
   507	valid options. Since `UPG-0057` CHG-B, it automatically resolves and injects this project's
   508	Controlled Plain English status (§12a below) before invoking the reviewer. Direct binary invocation
   509	(`/path/to/Codeos/tools/reviewer/target/release/codeos-reviewer ...`, where `/path/to/Codeos` is
   510	wherever `.codeos` resolves to — check with `readlink -f .codeos`) still runs identically, but
   511	skips that injection step entirely — it is **not** a supported alternative for Controlled Plain
   512	English purposes, since `codeos-reviewer-task.md` never reads any config file itself and depends
   513	on the wrapper to supply the resolved status line.
   514	
   515	## 12a. Controlled Plain English automatic status injection (`UPG-0057` CHG-B)
   516	
   517	Before invoking the reviewer for a `review` or `plan` subcommand (the two subcommands that build a
   518	packet), the wrapper:
   519	
   520	1. **Resolves context** — if the caller's own git root is the Codeos repo itself, this is a
   521	   self-development review and the wrapper reads `config/writing-discipline.yaml` (relative to the
   522	   Codeos repo root); otherwise it's a downstream review and the wrapper reads
   523	   `architecture/controlled-plain-english.yaml` relative to the *caller's* git root (not through
   524	   `.codeos`).
   525	2. **Resolves the four-outcome status** per `UPG-0056`'s Optional Mechanism Status Convention:
   526	   absent or exact `status: disabled` → `disabled`; exact `status: enabled` → `enabled`; anything
   527	   else → a configuration error.
   528	3. **On a valid status** (`enabled` or `disabled`), writes a synthetic, deterministic temp file
   529	   (recognizable name `codeos-cpe-status.*`, created with `mktemp`, cleaned up via a `trap` on both
   530	   the success and failure paths) containing the exact line `codeos-reviewer-task.md` recognizes,
   531	   plus its source config path and the stage argument, then appends that file's path to the
   532	   packet's artifact list — no operator-supplied status path is ever required.
   533	4. **On a malformed or contradictory status file**, stops with a clear error *before* invoking the
   534	   reviewer (exit code 7) — this is an invocation precondition failure, the same class as "binary
   535	   not found," never a reviewer finding.
   536	
   537	Ordinary style non-compliance in the reviewed prose itself is entirely unaffected: it remains an
   538	advisory reviewer finding under existing authority, never a packet-generation failure. A `disabled`
   539	(or absent/not-applicable) status is injected and reviewed normally — it never blocks anything.
   540	`tools/reviewer/src/*` is unchanged by this mechanism; the wrapper only appends an ordinary
   541	artifact path that the Rust engine already knows how to embed.
   542	
   543	**If reviewer tooling isn't built or configured** for a downstream project, see
   544	`dba-system.md`'s Review Waiver practice — record a plain reason in that feature's review
   545	log and proceed; the human-approval gate (Non-Negotiable Rule #1) still applies regardless.
   546	
   547	---
   548	
   549	## 13. Verification round-trip
   550	
   551	Every review round already ends with a `HIGHEST-IMPACT UNCERTAINTY:` line (mandated by
   552	`prompts/codeos-reviewer-task.md`'s output format) — one sentence naming the single thing
   553	that, if wrong, most affects the assessment. Separately, `prompts/verify-only.md` implements
   554	a full read-only verification mode: a no-edit rule, before/after anti-blur `git status`/`git
   555	diff --exit-code` checks proving the tree wasn't mutated, and a structured Verification-Only
   556	Report.
   557	
   558	When that uncertainty names something mechanically checkable — a specific file, command, or
   559	repository state — the acting agent may run a `verify-only.md` pass targeting exactly that
   560	uncertainty, then feed the resulting report back as evidence for the next review round. This
   561	session used exactly this pattern more than once: UPG-0019's Step 3 rounds re-ran the review
   562	with `check_drift.rs` (then `main.rs`) shown directly after the prior round's uncertainty
   563	named an unverified claim about their behavior; UPG-0024's Step 2 rounds resolved two
   564	internal-contradiction findings the same way — show more, re-review, resolve.
   565	
   566	This is judgment, not automation: the acting agent decides whether an uncertainty is
   567	checkable and whether running verification is worth the round-trip. It is never mandatory,
   568	and a verification pass never substitutes for the human's decision at the gate — it only
   569	adds evidence to it. A verification pass does not itself count against the round-budget table
   570	in §4d; only the review round that follows it does.
   571	
   572	`dba-system.md`'s "Default Advisory Review" section carries the same practice, in the same
   573	terms, for downstream DBA projects — the two are kept in sync deliberately, not maintained
   574	independently.
   575	
   576	---
   577	
   578	## 14. Evidence Modes
   579	
   580	The reviewer supports three evidence modes to control packet size and review focus. These modes affect what evidence is included in the review packet; they do not change the reviewer's advisory role or the human approval gate.
   581	
   582	**At a glance:**
   583	
   584	| Mode | Best use | Main risk |
   585	|---|---|---|
   586	| `full` | Round 1 of a review; small/primary artifacts | Packet bloat on large stable files |
   587	| `delta` | Round 2+ after fixes | Wrong/stale `--base`; untracked artifacts error (§4b guardrail) |
   588	| `sha-only` | Large unchanged context files, not the primary artifact | Reduced review evidence — reviewer cannot inspect content |
   589	
   590	```mermaid
   591	flowchart LR
   592	    F[full] -->|best evidence,<br/>largest packets| R[Codex reviewer]
   593	    D[delta] -->|changed lines only,<br/>best for R2+| R
   594	    S[sha-only] -->|lowest evidence,<br/>stable context only| R
   595	```
   596	
   597	### Full Mode — default
   598	
   599	Includes full artifact content where allowed by packet size and redaction rules.
   600	
   601	**Use when:**
   602	- Running Round 1 of a review
   603	- Reviewing the primary artifact under active change
   604	- The reviewer needs full context to assess the artifact
   605	
   606	**Command:**
   607	```bash
   608	.codeos/scripts/codeos-review.sh review <feature> <stage> <artifact-paths>
   609	```
   610	
   611	### Delta Mode
   612	
   613	Includes only changes since a base commit. Unchanged artifacts are represented by path and hash only.
   614	
   615	**Use when:**
   616	- Running Round 2 or later after fixing reviewer findings
   617	- The packet exceeds the size budget and most artifacts are unchanged
   618	- The review should focus on what changed since the previous round
   619	
   620	**Command:**
   621	```bash
   622	.codeos/scripts/codeos-review.sh review <feature> <stage> --mode delta --base <commit-sha> <artifact-paths>
   623	```
   624	
   625	**Guardrail:** Delta mode requires artifact paths to be tracked by git. Untracked files cannot be compared to the base commit and will error.
   626	
   627	### SHA-Only Mode
   628	
   629	Includes only the file path and hash, not file content. **This reduces packet size but also reduces review evidence.**
   630	
   631	**Use only for:**
   632	- Large unchanged context files
   633	- Files needed for packet completeness but not for substantive review
   634	- Files that are not the primary artifact under review
   635	
   636	**Command:**
   637	```bash
   638	.codeos/scripts/codeos-review.sh review <feature> <stage> --sha-only <context-file> <other-artifacts>
   639	```
   640	
   641	**Guardrail:** Do not use SHA-only for files whose changed behavior, wording, or structure the reviewer must assess. Changed behavior must remain reviewable as full content or diff.
   642	
   643	### Combining Modes
   644	
   645	Delta mode and SHA-only can be combined. When both apply, SHA-only paths are included as path/hash references rather than full content or diff.
   646	
   647	```bash
   648	scripts/codeos-review.sh review UPG-0042 selfdev-step-3 \
   649	  --mode delta --base abc123 \
   650	  --sha-only docs/large-reference.md \
   651	  changes/UPG-0042__CHG-*.md src/packet.rs
   652	```
   653	
   654	### Preview a plan before reviewing
   655	
   656	`codeos-review.sh plan` (like `review`, resolved through the wrapper — §12a) accepts the exact same arguments as `review` (feature, stage,
   657	artifacts, `--mode`/`--base`, `--sha-only`) and reports what a `review` call with those
   658	arguments would send — resolved artifacts with their mode and byte size, `review_content_bytes`
   659	vs. the packet budget, `estimated_review_tokens`, coverage state, and (when over budget) the
   660	same size/contributor/delta-suggestion warning `review` itself prints. `plan` calls the exact
   661	same `packet::build()` function `review`/`--print-packet` use, so it cannot describe a packet
   662	`review` wouldn't actually build.
   663	
   664	```bash
   665	scripts/codeos-review.sh plan UPG-0042 selfdev-step-1 changes/UPG-0042__CHG-*.md src/packet.rs
   666	```
   667	
   668	`plan` never resolves or invokes a provider and never writes to `reviews/` or any other tracked
   669	file — it only builds and reports the packet plan. Unlike `--print-packet`, which prints the
   670	full packet text Codex would receive, `plan` prints a compact summary; use `--print-packet` when

exec
/bin/bash -lc "nl -ba CLAUDE.md | sed -n '150,260p'" in /home/rimo/projects/Codeos
exec
/bin/bash -lc "nl -ba dba-system.md | sed -n '45,75p;285,305p;448,490p'" in /home/rimo/projects/Codeos
 succeeded in 0ms:
    45	
    46	Advisory review runs by default at every reviewable gate across the whole workflow below —
    47	not only the numbered Stage 1-9 loop, but also Feature Brief, Onboarding, Solution Discovery
    48	(conditionally — see below), and Architectural Refinement. This mirrors the same default
    49	that Codeos's own toolkit development already holds itself to.
    50	
    51	**How to run it.** Before each gate's human-approval decision, run:
    52	```
    53	.codeos/scripts/codeos-review.sh review <feature_id> <stage>
    54	```
    55	using the Stage ID from the table above (e.g. `.codeos/scripts/codeos-review.sh review
    56	checkout-flow 2` before approving Stage 2's contract; `.codeos/scripts/codeos-review.sh review
    57	checkout-flow brief` before confirming a Feature Brief). The reviewer is independent, read-only,
    58	and non-gatekeeping — its verdict (NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE) informs the
    59	human's decision but never auto-blocks. **The human decides at the gate; Non-Negotiable Rule #1
    60	is unchanged.**
    61	
    62	**The wrapper is the supported entry point.** `.codeos/scripts/codeos-review.sh` (downstream) and
    63	`scripts/codeos-review.sh` (Codeos's own self-development) automatically resolve and inject this
    64	project's Controlled Plain English status (see "Controlled Plain English Writing Discipline"
    65	below) before invoking the reviewer. Invoking the compiled `codeos-reviewer` binary directly
    66	bypasses that injection — it still runs, but it is not a supported alternative for Controlled
    67	Plain English purposes, since the shared reviewer template never reads any config file itself
    68	(see the Call-site map below).
    69	
    70	**Round budget.** Round 1 runs before the gate. Rounds 2-3 are allowed for fixes or material
    71	deltas raised by the previous round. After 3 rounds, stop and require a human decision rather
    72	than continuing to iterate automatically.
    73	
    74	**Solution Discovery is reviewed conditionally, not unconditionally.** The Discovery session
    75	itself stays optional and non-authoritative — running it is never required, and its output is
   285	`architecture/cohort-logical-design.md`'s **current** version field exactly. A value that instead
   286	matches a file under `architecture/history/` is **stale, not valid** for either artifact — it
   287	blocks Stage 4 exactly as an unapproved status would (see `.codeos/prompts/04-implement.md`'s
   288	cohort eligibility check), until either the registry is updated to the current version (normally
   289	automatic, as part of approving that version) or a human resolves the discrepancy. This is
   290	deliberately stricter than "any version this feature was ever pinned to": the live check only
   291	ever accepts the current version of *both* artifacts; historical files matter for audit and for
   292	the non-retroactive protection above, not for gating new work.
   293	
   294	**Reviewer coverage.** `codeos-reviewer` has a dedicated checklist for the `architecture-synthesis`
   295	stage id, covering all four pipeline steps — run `.codeos/scripts/codeos-review.sh review
   296	<feature_id> architecture-synthesis` the same way as any other stage, per "Default Advisory
   297	Review" above. This
   298	does not weaken Non-Negotiable Rule #1 — the human still explicitly approves both the baseline and
   299	the logical design.
   300	
   301	**Naming.** This is the **Architecture Synthesis Gate**, producing the **Core Architecture
   302	Baseline** — deliberately not "Architecture Discovery." Solution Discovery
   303	(`.codeos/prompts/00a-solution-discovery.md`) is optional, non-gating, and pre-Stage-1; its
   304	output is never approved architecture. This gate is the opposite: conditional but, once
   305	triggered, mandatory, and it runs only after Stage 3 approval across the whole cohort, consuming
   448	non-retroactivity rule, and the 15-section adaptation matrix — lives in
   449	`.codeos/patterns/controlled-plain-english.md`. This section documents only the activation
   450	mechanics and where the mechanism applies.
   451	
   452	**Activation.** A downstream project's status file is `architecture/controlled-plain-english.yaml`
   453	(project-local — not reached through the `.codeos` symlink; `scripts/dba-init.sh` scaffolds it at
   454	`status: enabled` by default — a human sets it to `status: disabled` to turn the discipline off).
   455	The pattern file itself *is* reached through the symlink, at
   456	`.codeos/patterns/controlled-plain-english.md`. A missing file still means disabled (the Optional
   457	Mechanism Status Convention's own fallback, unchanged); the exact grammar and four-outcome table
   458	are that convention's, unchanged.
   459	
   460	**Call-site map** — every consumer is named explicitly; none guesses which file applies:
   461	
   462	| Call site | Configuration it reads |
   463	|---|---|
   464	| Stage 1-10 prompts (`.codeos/prompts/01-intent.md` … `10-arch-refine.md`) | `architecture/controlled-plain-english.yaml` |
   465	| `.codeos/prompts/pipeline-reviewer.md` | `architecture/controlled-plain-english.yaml` |
   466	| `.codeos/prompts/codeos-reviewer-task.md` (shared reviewer infrastructure) | **Reads neither file — configuration-neutral.** `.codeos/scripts/codeos-review.sh` resolves this project's status automatically and appends a synthetic status artifact — "Controlled Plain English status for this review: enabled/disabled" plus its config source and applicable stage — to the packet before invoking the reviewer, the same way any other file is passed to `codeos-reviewer review`. `codeos-reviewer-task.md` still never reads a config file itself; it only recognizes the already-resolved line among the reviewed artifacts. `tools/reviewer` is unchanged — it still embeds whatever artifact paths it is given; the wrapper is what supplies this one automatically now. |
   467	
   468	**What is and isn't toggle-gated.** Layer A (plain communication) and Layer C1/D1 (existing
   469	literal-protection and reviewer-integrity authority) are **not** new mandatory rules and are never
   470	toggle-gated — they restate expectations already binding elsewhere. Layer B, C2, and D2 are the
   471	only parts this switch actually gates. Disabling (or never configuring) this mechanism leaves
   472	generation and review behavior exactly as it is today.
   473	
   474	**No new Stage ID, no new Non-Negotiable Rule, no new mandatory human-approval gate.** This is a
   475	writing discipline consulted by existing stages and the existing reviewer, not a new stage or gate
   476	of its own.
   477	
   478	---
   479	
   480	## What You Do at Each Stage
   481	
   482	Use the corresponding prompt file from `.codeos/prompts/` for detailed instructions. The
   483	**Stage ID** column is the identifier vocabulary used both for documentation ordering and as
   484	the `<stage>` argument to `.codeos/scripts/codeos-review.sh review <feature_id> <stage>` — see
   485	"Default Advisory Review" below.
   486	
   487	| Stage | Stage ID | File |
   488	|---|---|---|
   489	| Session start | — | `.codeos/prompts/00-session-start.md` |
   490	| Session end (handoff) | — | `.codeos/prompts/00-session-end.md` |

 succeeded in 0ms:
   150	
   151	## What You NEVER Do (self-development)
   152	
   153	- Treat a non-trivial change as trivial to skip the loop.
   154	- Advance a step without running the review required by your profile and getting explicit approval.
   155	- Change the downstream doctrine (`dba-system.md`) as a side effect — that requires an
   156	  explicit `downstream-doctrine` (or `both`) scope declaration in Step 1.
   157	- Rewrite downstream 9-stage substance when only a path/location change is intended.
   158	- Put live status into this file — it belongs in `status/self-development.md`.
   159	- Turn `prompts/codeos-self-dev.md` into a second doctrine; keep it a practical step prompt.
   160	- Let the reviewer become an enforcement engine — its verdict is advisory, scope-triaged.
   161	
   162	---
   163	
   164	## Writing Discipline (Controlled Plain English)
   165	
   166	References, does not redefine, `patterns/controlled-plain-english.md`'s layers and reviewer model
   167	(see `dba-system.md` → "Controlled Plain English Writing Discipline" for the downstream side of
   168	this same mechanism). Self-development's own status file is `config/writing-discipline.yaml`
   169	(Codeos-repo-local, not project-local like the downstream file), read and injected automatically by
   170	`scripts/codeos-review.sh` per `UPG-0057` CHG-B — see `prompts/codeos-self-dev.md`'s Step 0b.
   171	
   172	**Layer A** (plain communication in ordinary chat, session updates, explanations) always applies,
   173	unconditionally, exactly as it already does everywhere else in this environment — it is not
   174	gated by this file's status.
   175	
   176	**Per-section rule table**, when `status: enabled`:
   177	
   178	| Change-record section | Layer applied |
   179	|---|---|
   180	| Change Intent / Acceptance Criteria / Implementation Plan | Layer B (specification precision) |
   181	| Implementation Notes | Factual reporting, not Layer B — this section reports what happened |
   182	| Review findings / Reconciliation | Layer D1 always; Layer D2 (plain review prose) when enabled |
   183	
   184	**Reviewer Model** (restated, not new authority): no separate "Controlled Plain English
   185	violation" finding category, and no historical-compliance audit — a reviewer reports meaning loss,
   186	authority mixing, or an unverifiable requirement using existing review authority, exactly as it
   187	does today. See the pattern's own "Reviewer Model" section for the full statement.
   188	
   189	**Placement note:** `config/writing-discipline.yaml` lives under `config/`, not `status/` — it is a
   190	human-set configuration toggle, not live mutable workflow state like the dashboard or roadmap.
   191	
   192	**Assumptions-subsection convention** (guidance only — no new formal field on
   193	`templates/codeos-change.md`): when Layer B applies and a material assumption exists that no
   194	existing change-record section already represents, an agent may add a plainly-labeled
   195	"Assumptions" subsection to Implementation Notes. Never rendered when empty; never parsed by any
   196	script or template validator.
   197	
   198	---
   199	
   200	## Self-Development File Layout
   201	
   202	```
   203	Codeos/                          ← toolkit repo (this repo)
   204	├── CLAUDE.md                    ← THIS FILE — stable self-development operating guide
   205	├── dba-system.md                ← downstream DBA doctrine (loaded by downstream projects)
   206	├── config/
   207	│   └── writing-discipline.yaml   ← Controlled Plain English status for self-development (see "Writing Discipline" above)
   208	├── status/
   209	│   ├── self-development.md       ← live Self-Development Status dashboard (mutable; Feature ID + Change ID)
   210	│   ├── roadmap.md                ← dependency-aware wave plan, keyed by UPG-#### (mutable)
   211	│   ├── stack-manifest.md         ← live stack/dependency status (evidence, not authority; mutable)
   212	│   └── stack-reconciliation/     ← one *-stack-reconciliation-report.md per watched-file change
   213	├── changes/
   214	│   └── UPG-####__CHG-YYYYMMDD-NNN__slug.md  ← per-change source of truth (one per non-trivial change)
   215	├── backlog/
   216	│   ├── features.md               ← authoritative UPG-#### → file map (identity)
   217	│   └── UPG-####-slug.md          ← feature briefs w/ trace header + Feature Thread (feed Step 1)
   218	├── prompts/                     ← stage + self-dev step prompts
   219	├── templates/                   ← artifact templates
   220	├── patterns/                    ← structural patterns
   221	├── docs/                        ← toolkit documentation
   222	├── scripts/                     ← dba-init.sh, codeos-review.sh
   223	└── reviews/
   224	    ├── review-log.md             ← append-only review + decision log
   225	    ├── architecture-journal.md   ← cross-cutting institutional memory (AJ-NNN)
   226	    └── codex/                     ← reviewer assessments + packets
   227	```
   228	
   229	**Identity & IDs (Feature Thread model).** Work is traced by stable **`UPG-####`** feature ids,
   230	per-execution **`CHG-YYYYMMDD-NNN`** change ids, **`REV__…__S<N>__R<N>`** review-round ids, and the
   231	stable **`RVS__…__S<N>`** review-series id. **Self-Reference Boundary:** reviewed artifacts carry
   232	`review_series` + `review_state`, never a live round — exact rounds live only in `reviews/`. The
   233	dashboard separates Feature ID from Change ID, and every backlog brief carries a
   234	`## Feature Thread` rollup. The full model is `backlog/UPG-0001-feature-thread-traceability.md`.
   235	In-scope review fixes stay inside the same `CHG-*`; only OUT-OF-SCOPE BACKLOG findings spawn a new
   236	`UPG-####` (see the Review-Fix Rule in `prompts/codeos-self-dev.md`).
   237	
   238	---
   239	
   240	## Review Logging
   241	
   242	When the human gives a reviewer's assessment and their decision, before any other work:
   243	show a brief (≈5-line) preview of what you will write, then write it.
   244	
   245	1. **One entry** to `reviews/review-log.md` (append-only) capturing the reviewer's core
   246	   insight (close to verbatim) and the human decision separately.
   247	2. **One entry** to `reviews/architecture-journal.md` (`AJ-NNN`) only if the insight will
   248	   still matter six months from now to someone who has forgotten this change. When unsure,
   249	   journal only if future usefulness is clear.
   250	
   251	**Human overrides:** "do not log this review" / "journal this" / "do not journal this".
   252	
   253	**Fidelity:** preserve the insight verbatim; compress only context. Record conclusions and
   254	rationale, not conversation history. Log entries are append-only — supersede with a new
   255	entry, never rewrite.

exec
/bin/bash -lc "nl -ba scripts/codeos-review.sh | sed -n '1,140p'" in /home/rimo/projects/Codeos
 succeeded in 0ms:
     1	#!/usr/bin/env bash
     2	# codeos-review.sh — thin shim delegating to the compiled Rust binary, with automatic
     3	# Controlled Plain English status injection for the two packet-building subcommands.
     4	# Subcommands: review / plan / decision / diagnose / stage-start / check-drift / generate-*
     5	# (see: codeos-reviewer --help)
     6	# To build: cargo build --release --manifest-path tools/reviewer/Cargo.toml
     7	#
     8	# Exit codes: 1 = not a git repo, 2 = binary not found, 7 = Controlled Plain English
     9	# status file is malformed (see "Controlled Plain English automatic status injection"
    10	# below). Codes 0/3/4/5/6 belong to the Rust binary itself (tools/reviewer/src/main.rs)
    11	# and are passed through unchanged.
    12	set -euo pipefail
    13	# Preserve the original precondition: the shim requires the CALLER to be inside some git
    14	# repository (a property of the project being reviewed) — unrelated to where the binary
    15	# itself lives, so this check is deliberately kept separate from binary-path resolution
    16	# below, running first, exactly as before this change.
    17	git rev-parse --show-toplevel >/dev/null 2>&1 || { echo "error: not inside a git repository" >&2; exit 1; }
    18	
    19	# Resolve the binary relative to this script's own physical location (following the
    20	# .codeos symlink when invoked from a downstream project), not the calling repo's git
    21	# root — git rev-parse --show-toplevel from within a downstream project resolves to that
    22	# project's own root, not through .codeos to Codeos, which is where the binary actually
    23	# lives. pwd -P (physical) is required, not plain pwd, to resolve through the symlink
    24	# rather than preserve its logical name.
    25	SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
    26	CODEOS_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd -P)"
    27	BINARY="${CODEOS_ROOT}/tools/reviewer/target/release/codeos-reviewer"
    28	if [[ ! -x "${BINARY}" ]]; then
    29	  command -v codeos-reviewer >/dev/null 2>&1 && BINARY="codeos-reviewer" || {
    30	    echo "error: binary not found at ${BINARY} and not on PATH" >&2
    31	    echo "       Build: cargo build --release --manifest-path ${CODEOS_ROOT}/tools/reviewer/Cargo.toml" >&2
    32	    exit 2
    33	  }
    34	fi
    35	
    36	SUBCOMMAND="${1:-}"
    37	
    38	# ── Controlled Plain English automatic status injection ────────────────────────────────
    39	# Only "review" and "plan" build a reviewer packet (see tools/reviewer/src/main.rs); every
    40	# other subcommand (decision, diagnose, stage-start, check-drift, generate-*) is passed
    41	# through unchanged below, exactly as before this change.
    42	if [[ "${SUBCOMMAND}" == "review" || "${SUBCOMMAND}" == "plan" ]]; then
    43	  # Context resolution: this one shared script is reached two ways — directly as
    44	  # scripts/codeos-review.sh from within this repo (a self-development review), or as
    45	  # .codeos/scripts/codeos-review.sh through the downstream symlink (a downstream
    46	  # project's review). The caller's own git root (already required to exist by the
    47	  # precondition above) tells them apart: it equals CODEOS_ROOT only in the former case.
    48	  CALLER_ROOT="$(cd "$(git rev-parse --show-toplevel)" && pwd -P)"
    49	  STAGE_ARG="${3:-}"
    50	
    51	  if [[ "${CALLER_ROOT}" == "${CODEOS_ROOT}" ]]; then
    52	    CPE_CONFIG="${CODEOS_ROOT}/config/writing-discipline.yaml"
    53	  else
    54	    CPE_CONFIG="${CALLER_ROOT}/architecture/controlled-plain-english.yaml"
    55	  fi
    56	
    57	  # Four-outcome resolution, per UPG-0056's Optional Mechanism Status Convention
    58	  # (templates/conventions.md): absent -> disabled; exact "status: disabled" -> disabled;
    59	  # exact "status: enabled" -> enabled; anything else -> configuration error. Leading/
    60	  # trailing blank lines are allowed and line endings are normalized (CRLF -> LF) before
    61	  # comparison; internal whitespace, case, tabs, and comments are NOT normalized and make
    62	  # the file invalid, exactly like every other consumer of this convention.
    63	  CPE_STATUS=""
    64	  if [[ ! -f "${CPE_CONFIG}" ]]; then
    65	    CPE_STATUS="disabled"
    66	  else
    67	    CPE_NONBLANK=()
    68	    while IFS= read -r _cpe_line || [[ -n "${_cpe_line}" ]]; do
    69	      [[ -n "${_cpe_line}" ]] && CPE_NONBLANK+=("${_cpe_line}")
    70	    done < <(tr -d '\r' < "${CPE_CONFIG}")
    71	
    72	    if [[ "${#CPE_NONBLANK[@]}" -eq 1 && "${CPE_NONBLANK[0]}" == "status: disabled" ]]; then
    73	      CPE_STATUS="disabled"
    74	    elif [[ "${#CPE_NONBLANK[@]}" -eq 1 && "${CPE_NONBLANK[0]}" == "status: enabled" ]]; then
    75	      CPE_STATUS="enabled"
    76	    else
    77	      # Malformed or contradictory configuration: fail BEFORE invoking the reviewer. This
    78	      # is an invocation precondition failure (same class as "binary not found" above),
    79	      # never a reviewer finding — ordinary style non-compliance in generated prose is
    80	      # always and only a reviewer finding, never a packet-generation failure.
    81	      echo "error: invalid Controlled Plain English status file: ${CPE_CONFIG}" >&2
    82	      echo "       must contain exactly one non-blank line: 'status: enabled' or 'status: disabled'" >&2
    83	      exit 7
    84	    fi
    85	  fi
    86	
    87	  # The generated artifact is explicitly synthetic: a recognizable filename
    88	  # (codeos-cpe-status.*), a deterministic three-line body, and safe temp-file handling
    89	  # (mktemp, quoted paths, trap cleanup covering both the success and failure paths).
    90	  # NOTE: this branch cannot end in `exec` (see below) — a bash EXIT trap never fires
    91	  # across exec, since exec replaces the process image instead of letting the shell
    92	  # return to run its traps. To still guarantee cleanup, the binary is invoked as an
    93	  # ordinary subprocess here and this script exits with its exact exit code afterward.
    94	  CPE_STATUS_FILE="$(mktemp "${TMPDIR:-/tmp}/codeos-cpe-status.XXXXXX")"
    95	  trap 'rm -f "${CPE_STATUS_FILE}"' EXIT
    96	  {
    97	    printf 'Controlled Plain English status for this review: %s\n' "${CPE_STATUS}"
    98	    printf 'Source: %s\n' "${CPE_CONFIG}"
    99	    printf 'Applicable scope: %s\n' "${STAGE_ARG}"
   100	  } > "${CPE_STATUS_FILE}"
   101	
   102	  "${BINARY}" "$@" "${CPE_STATUS_FILE}"
   103	  exit $?
   104	fi
   105	
   106	exec "${BINARY}" "$@"

codex
I have the main implementation and doctrine references pinned. Next I’m checking whether the change’s own bookkeeping claims line up with the code and whether the supported-entry-point claim is fully propagated across the reviewed docs.
exec
/bin/bash -lc "nl -ba prompts/codeos-self-dev.md | sed -n '55,95p'" in /home/rimo/projects/Codeos
 succeeded in 0ms:
    55	
    56	**Budget exceeded:** fix remaining findings inline and escalate to human decision. Do not
    57	run further Codex rounds automatically. See `docs/reviewer-pipeline.md §4d` for the full
    58	budget table and escalation procedure.
    59	
    60	Record `review_profile: PROFILE-N` in the change record trace header.
    61	
    62	---
    63	
    64	## Step 0b — Writing Discipline Check
    65	
    66	Read `config/writing-discipline.yaml` (Codeos-repo-local). Per the Optional Mechanism Status
    67	Convention's four-outcome table (`templates/conventions.md`): absent or exact `status: disabled` →
    68	disabled; exact `status: enabled` → enabled; anything else → stop and report a configuration error.
    69	
    70	When enabled, apply `CLAUDE.md`'s "Writing Discipline (Controlled Plain English)" per-section rule
    71	table to this change's own artifacts (Layer B for Change Intent / Acceptance Criteria /
    72	Implementation Plan; factual reporting for Implementation Notes; Layer D1 always + D2 when enabled
    73	for review findings and Reconciliation). No new change-record trace-header field is added for
    74	this — non-retroactivity is the one-sentence rule already stated in
    75	`patterns/controlled-plain-english.md`, nothing to stamp per change.
    76	
    77	`scripts/codeos-review.sh` reads this same file automatically and injects its resolved status into
    78	every review packet built for a self-development change — see `docs/reviewer-pipeline.md §12a`.
    79	This step is about applying the discipline while *writing* the artifact; the wrapper's injection is
    80	about what the *reviewer* is told, a separate but related mechanism.
    81	
    82	---
    83	
    84	## The 4-Step Loop
    85	
    86	Each step requires explicit human approval before the next.
    87	After each step output (**and its Codex review, if required by profile**), state:
    88	**`AWAITING HUMAN APPROVAL TO PROCEED TO STEP [N+1]`**
    89	
    90	For profiles that require a review at this step: run the review before the gate (see
    91	Reviewer Handling). Advance only on an explicit "APPROVED" / "approved" / "yes proceed"
    92	/ equivalent.
    93	
    94	---
    95	

exec
/bin/bash -lc "nl -ba prompts/00a-solution-discovery.md | sed -n '198,212p'; nl -ba prompts/03b-architecture-synthesis.md | sed -n '170,182p'; nl -ba prompts/pipeline-reviewer.md | sed -n '1,16p'" in /home/rimo/projects/Codeos
exec
/bin/bash -lc "nl -ba patterns/controlled-plain-english.md | sed -n '1,40p'" in /home/rimo/projects/Codeos
 succeeded in 0ms:
   198	
   199	To continue with feature work:
   200	1. Identify which candidate features to prioritize.
   201	2. Start a new session using **Session Type A** (Feature Brief) for each candidate you
   202	   want to develop into a DBA feature.
   203	3. Use this discovery output as background context for those briefs — not as a binding
   204	   specification. Stage 1 (Intent Capture) is the authoritative entry point.
   205	
   206	**If this output is carried into a Feature Brief or a Stage 1 Intent**, that handoff gets
   207	the default advisory review (`.codeos/scripts/codeos-review.sh review <feature_id> discovery`) or an explicit
   208	Review Waiver — see `dba-system.md`'s "Default Advisory Review" section. This session
   209	itself stays optional and non-gating either way; a Discovery session whose output nobody
   210	carries forward is never reviewed, because there is nothing yet to review.
   211	
   212	The standard DBA path remains: **Intent → Contract → Schema → Implement → Tests →
   170	**`AWAITING HUMAN APPROVAL — ARCHITECTURE SYNTHESIS APPROVED`**
   171	
   172	---
   173	
   174	## Reviewer Note
   175	
   176	`codeos-reviewer` has a dedicated checklist for the `architecture-synthesis` stage id, covering all
   177	four steps of this pipeline — run `.codeos/scripts/codeos-review.sh review <feature_id>
   178	architecture-synthesis` for gate reviews at this stage, per "Default Advisory Review" in
   179	`dba-system.md`. This does not weaken
   180	the requirement for explicit human approval at each step above.
     1	# Reviewer Activation Package — DBA / Codeos
     2	
     3	Paste this at the start of a reviewer LLM session before providing a stage artifact.
     4	
     5	**This is an optional, supplementary second opinion** — an independent critical-assessor
     6	pass free to challenge the artifact, the feature, or DBA itself. It does not replace the
     7	default advisory review (`.codeos/scripts/codeos-review.sh review <feature_id> <stage>`, structured and
     8	acceptance-criteria-bound) described in `dba-system.md`'s "Default Advisory Review" section.
     9	Use both when a stage warrants extra scrutiny; use this alone only when the default review
    10	tooling isn't available and a waiver has been recorded but a human still wants a second read.
    11	
    12	---
    13	
    14	## What Is DBA
    15	
    16	Declarative Behavioral Architecture (DBA) is a methodology for building software features

 succeeded in 0ms:
     1	# Pattern: Controlled Plain English
     2	
     3	## When This Pattern Applies
     4	
     5	This pattern documents a writing discipline for AI-generated prose in Codeos artifacts: plain
     6	language where precision doesn't require otherwise, exact modal verbs and quantifiers where it
     7	does, and protection for literal content (event names, field names, quoted normative text) from
     8	paraphrase drift. It applies wherever a downstream project or Codeos's own self-development has
     9	enabled it via the Optional Mechanism Status Convention (`.codeos/templates/conventions.md` →
    10	"Optional Mechanism Status Convention"; see `dba-system.md` → "Controlled Plain English Writing
    11	Discipline" for the activation mechanics and call-site map).
    12	
    13	**Consulted by:** every Stage 1-10 prompt and `pipeline-reviewer.md` — each check line names which
    14	layers below apply to it and applies the enabled-but-pattern-unavailable rule (below) directly.
    15	`codeos-reviewer-task.md` is **not** a consumer of this file — it stays configuration-neutral (see
    16	"Reviewer Model" and `dba-system.md`'s call-site map): it never reads this pattern or any status
    17	file itself, and therefore never performs the enabled-but-pattern-unavailable check. It only reacts
    18	to a status line automatically injected by `scripts/codeos-review.sh` (or `.codeos/scripts/
    19	codeos-review.sh` downstream) before the reviewer is invoked, using Layer D2's rule text, which is
    20	restated inline in `codeos-reviewer-task.md` itself rather than requiring it to read this file.
    21	This pattern's Layer B/C2/D2 rules are toggle-gated (see "Layers," below); Layer A/C1/D1 are always
    22	active regardless of the toggle, since they restate expectations that already exist elsewhere in
    23	this environment or in Codeos's own Non-Negotiable Rules — naming them here consolidates them, it
    24	does not create new authority.
    25	
    26	---
    27	
    28	## Layers
    29	
    30	### Layer A — Plain communication (always active, advisory)
    31	
    32	For ordinary chat, explanations, and session updates: short sentences, common words, conclusion
    33	first, preserve exact names and facts. Preserve exact technical terms (e.g. "idempotent" never
    34	loosens to "safe"); define an uncommon technical term in one plain sentence on first use, then use
    35	it consistently afterward.
    36	
    37	This restates tone expectations already present in this environment's own system instructions
    38	(terse, direct responses) — it is advisory guidance for general communication, never toggle-gated,
    39	and never a gate on artifact approval.
    40	

exec
/bin/bash -lc "nl -ba backlog/UPG-0057-controlled-plain-english-writing-discipline.md | sed -n '80,135p'; nl -ba status/self-development.md | sed -n '74,86p'" in /home/rimo/projects/Codeos
 succeeded in 0ms:
    80	  existing DBA review authority.
    81	- The shared `codeos-reviewer-task.md` template never reads filesystem configuration itself — it
    82	  only receives an injected status line from its caller.
    83	- Non-retroactivity is a one-sentence doctrine rule, not a stamped/audited mechanism.
    84	
    85	## Related
    86	
    87	- Depends on `UPG-0056` (Optional Mechanism Status Convention), which must reach `COMPLETE` before
    88	  this UPG's own Step 3 (Implement) can begin.
    89	
    90	**Status note (updated after `CHG-B`'s Step 4 Reconciliation):** `CHG-A` established the
    91	Controlled Plain English pattern and consumer wiring. `CHG-B` completed automatic status delivery
    92	at the supported reviewer invocation boundary: `scripts/codeos-review.sh` (self-development) and
    93	`.codeos/scripts/codeos-review.sh` (downstream, the same shared script through the symlink) now
    94	resolve this project's status automatically and inject it before invoking the reviewer, for both
    95	the `review` and `plan` subcommands. Every currently-authoritative doctrine and prompt reference to
    96	running the reviewer names that wrapper, not the raw `codeos-reviewer` binary, which no longer
    97	serves as a supported alternative for Controlled Plain English purposes. **Manual inclusion is not
    98	part of the final operating model.** `tools/reviewer/src/*` was not touched by `CHG-B` — the
    99	injection is entirely a wrapper-level (bash) preprocessing step. **With `CHG-B` accepted, `UPG-0057`
   100	is complete.**
   101	
   102	## Feature Thread
   103	
   104	> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the change
   105	> records and review files. May be maintained manually.
   106	
   107	### Changes
   108	
   109	| Change ID | File | Purpose | State |
   110	|---|---|---|---|
   111	| CHG-20260726-003 | `changes/UPG-0057__CHG-20260726-003__controlled-plain-english-writing-discipline.md` | CHG-A: pattern + downstream doctrine + prompt wiring | COMPLETE |
   112	| CHG-20260727-001 | `changes/UPG-0057__CHG-20260727-001__automatic-cpe-status-injection.md` | CHG-B: automatic status injection in `scripts/codeos-review.sh` (no Rust changes); wrapper established as sole supported entry point | DRAFT |
   113	
   114	### Reviews
   115	
   116	| Review ID | Change ID | Step | Round | Verdict |
   117	|---|---|---|---|---|
   118	| RVS__UPG-0057__CHG-20260727-001__S1 | CHG-20260727-001 | 1-Intent | R1→R3 | R1 CHANGES ADVISED (Change Intent's "What changes" list omitted `status/self-development.md`'s own dashboard-bookkeeping edits; Acceptance Criteria was already fully drafted inside the Step 1 submission, blending Step 2 content into Step 1) → fixed (file added to "What changes"; AC table moved out of the Step 1 artifact to a Step 2 placeholder, draft held in scratchpad for reuse) → R2 CHANGES ADVISED (Step 2 placeholder claimed the "ten-item list quoted in Change Intent above," but only the five refinement conditions were actually quoted there — the ten-item list itself was missing) → fixed (ten-item list now quoted verbatim in condition 5 of Change Intent; placeholder wording corrected to match) → R3 NO OBJECTION |
   119	| RVS__UPG-0057__CHG-20260727-001__S2 | CHG-20260727-001 | 2-Acceptance | R1→R2 | R1 CHANGES ADVISED (AC3's verification only named a `--print-packet` run, silent on `plan`, though the criterion claims both subcommands; AC6's verification tested only `status: disabled`, silent on the absent-config case though the criterion claims both; AC8's grep set omitted `CLAUDE.md` despite this change's own self-dev-governance scope) → fixed (AC3/AC6 verification steps now name each case explicitly; AC8's grep set now includes `CLAUDE.md`) → R2 NO OBJECTION |
   120	| RVS__UPG-0057__CHG-20260727-001__S3 | CHG-20260727-001 | 3-Implement | R1→R3 | R1 CHANGES ADVISED (`docs/reviewer-pipeline.md` §10 still described the wrapper as a "static locator shim" with "no argument preprocessing," directly contradicting the implemented CPE injection logic in `scripts/codeos-review.sh`; a coverage-metadata secret-redaction flag also fired on a pre-existing, unrelated template field label — "Secret / non-secret:" in `prompts/00a-solution-discovery.md` — confirmed a false positive, no actual secret, no fix needed) → fixed (§10 rewritten to describe the locator-shim-plus-one-preprocessing-step architecture accurately) → R2 CHANGES ADVISED (the "Preview a plan before reviewing" prose still named the raw `codeos-reviewer plan` binary instead of the wrapper, missed in R1's fix pass) → fixed → R3 CHANGES ADVISED (no in-scope blocker; one IN-SCOPE NON-BLOCKER — a stray "§13 below" cross-reference for Controlled Plain English status, should say §12a; LOG SUMMARY driven only by the recurring benign `SECRET_REDACTION` false positive, not a real blocker) → fixed inline. **PROFILE-5's 3-round/step budget is now exhausted for Step 3** — fix applied per CLAUDE.md's budget-exceeded rule; no further automatic round; escalated to human decision |
   121	| RVS__UPG-0057__CHG-20260726-003__S1 | CHG-20260726-003 | 1-Intent | R1 | NO OBJECTION |
   122	| RVS__UPG-0057__CHG-20260726-003__S2 | CHG-20260726-003 | 2-Acceptance | R1→R2 | R1 DO NOT ADVANCE (AC3/AC4/AC18 depended on external plan content not in the packet; AC18's "cannot silently reintroduce" overstated an advisory generation discipline as enforcement) → fixed → R2 NO OBJECTION |
   123	| RVS__UPG-0057__CHG-20260726-003__S3 | CHG-20260726-003 | 3-Implement | R1→R3 | R1 DO NOT ADVANCE (`codeos-reviewer-task.md` falsely claimed as a pattern consumer able to enforce the pattern-unavailable check; File Layout "none by default" contradicted `dba-init.sh`'s own scaffolding; "Thirteen files" undercounted the actual 18) → fixed → R2 NO OBJECTION → revised post-R2 per human direction (scaffolded default changed `disabled`→`enabled`) → R3 DO NOT ADVANCE (Implementation Notes' own file-list sentence still said `status: disabled` for `dba-init.sh`, contradicting AC13's own "no remaining stale claim" text) → fixed. **PROFILE-4's 3-round/step budget is now exhausted for Step 3** — fix applied inline per CLAUDE.md's budget-exceeded rule; no further automatic round run; escalated to human decision |
   124	| RVS__UPG-0057__CHG-20260726-003__S4 | CHG-20260726-003 | 4-Reconcile | R1→R2 | R1 DO NOT ADVANCE (Post-R1-fixes historical note still asserted `dba-init.sh` scaffolds `status: disabled` without qualifying it as historical, contradicting AC13/Reconciliation's "no remaining claim") → fixed (note marked as historical record, Reconciliation's AC13 row explains the expected grep hit) → R2 NO OBJECTION — `findings: []`, `unparsed_findings_count: 0` |
   125	
   126	### Findings Tracked Inside This Feature
   127	
   128	| Finding ID | Review ID | Classification | Resolution |
   129	|---|---|---|---|
   130	| (Step 2 R1) AC3/AC4/AC18 depended on external plan content not included in the review packet | RVS__…__S2 | IN-SCOPE BLOCKER | Fixed — made self-contained (exact rule sentence, full 15-item list, explicit rule enumeration inlined) |
   131	| (Step 2 R1) AC18's "cannot silently reintroduce" phrased an advisory generation discipline as an enforcement guarantee | RVS__…__S2 | IN-SCOPE BLOCKER | Fixed — reworded to check that the doctrine text states each rule and refinement prompts carry no carve-out, not a provable behavioral guarantee |
   132	| (Step 3 R1) `codeos-reviewer-task.md` claimed as a pattern consumer subject to the enabled-but-pattern-unavailable rule, but it never reads the pattern or any status file | RVS__…__S3 | IN-SCOPE BLOCKER | Fixed — "Consulted by" and the rule's own text now explicitly carve it out |
   133	| (Step 3 R1) File Layout said the CPE status file is "none by default," contradicting `dba-init.sh`'s own scaffolding at `status: disabled` | RVS__…__S3 | IN-SCOPE BLOCKER | Fixed — File Layout now states it's scaffolded by default, opt-in via a separate human action |
   134	| (Step 3 R1) "Thirteen files touched" undercounted the actual 18 files named in the same sentence | RVS__…__S3 | IN-SCOPE BLOCKER | Fixed — corrected to eighteen |
   135	| (Post-R2, human direction) Scaffolded CPE default changed from `status: disabled` to `status: enabled` across `dba-init.sh`, `dba-system.md`, and this change record | — (human direction, not a Codex finding) | IN-SCOPE BLOCKER | Fixed — all documentation updated consistently |
    74	| UPG-0051 | CHG-20260719-001 | downstream-doctrine | downstream doctrine only | 4-Reconcile | ACCEPTED (series RVS__…__S4; Step 1 R1 DO NOT ADVANCE → R2 NO OBJECTION; Step 2 R1 NO OBJECTION, R2 NO OBJECTION post CHANGES-ADVISED fixes; Step 3 R1 DO NOT ADVANCE → R2 NO OBJECTION; Step 4 R1 NO OBJECTION; all 15 ACs verified; 3 in-scope blockers found+fixed across the change) | COMPLETE | — |
    75	| UPG-0052 | CHG-20260719-002 | downstream-doctrine | downstream doctrine only | 4-Reconcile | ACCEPTED (series RVS__…__S4; Step 1 R1 DO NOT ADVANCE → R2 NO OBJECTION → 1st human CHANGES-ADVISED → R3 NO OBJECTION → 2nd human CHANGES-ADVISED → R4 NO OBJECTION (human-requested, beyond budget); Step 2 R1 NO OBJECTION; Step 3 R1 NO OBJECTION; Step 4 R1 DO NOT ADVANCE → R2 NO OBJECTION; all 17 ACs verified; 4 in-scope blockers found+fixed across the change) | COMPLETE | — |
    76	| UPG-0053 | CHG-20260719-003 | script-tooling | self-dev only | 4-Reconcile | ACCEPTED (series RVS__…__S4; Step 1 R1 NO OBJECTION; Step 2 R1 DO NOT ADVANCE → R2 NO OBJECTION; Step 3 R1 NO OBJECTION; Step 4 R1 NO OBJECTION; all 8 ACs verified via live scratch run) | COMPLETE | — |
    77	| UPG-0054 | CHG-20260719-004 | downstream-doctrine | downstream doctrine only | 4-Reconcile | ACCEPTED (series RVS__…__S4; Step 1 R1 NO OBJECTION; Step 2 R1 DO NOT ADVANCE → R2 NO OBJECTION; Step 3 R1 DO NOT ADVANCE → R2 DO NOT ADVANCE → R3 NO OBJECTION; Step 4 R1 NO OBJECTION; all 13 ACs verified; 3 in-scope blockers found+fixed) | COMPLETE | — |
    78	| UPG-0055 | CHG-20260720-001 | script-tooling | self-dev only | 4-Reconcile | ACCEPTED (series RVS__…__S4; Step 1 R1 NO OBJECTION; Step 2 R1 NO OBJECTION; Step 3 R1 DO NOT ADVANCE → R2 DO NOT ADVANCE → R3 NO OBJECTION; Step 4 R1 DO NOT ADVANCE → R2 NO OBJECTION; all 9 ACs verified; 182 tests pass; 4 in-scope blockers found+fixed) | COMPLETE | — |
    79	| UPG-0056 | CHG-20260726-001 | downstream-doctrine | downstream doctrine only | 4-Reconcile | ACCEPTED (series RVS__…__S4; Step 1 R1-R3 DO NOT ADVANCE→NO OBJECTION (original design) then revised to lean convention, R4 DO NOT ADVANCE→APPROVE_STAGE; Step 2 R1 DO NOT ADVANCE→R2 NO OBJECTION; Step 3 R1-R2 DO NOT ADVANCE→R3 NO OBJECTION; Step 4 R1 DO NOT ADVANCE→R2 NO OBJECTION; all 12 ACs verified; no code shipped) | COMPLETE | UPG-0057 |
    80	| UPG-0058 | CHG-20260726-002 | downstream-doctrine | downstream doctrine only | 4-Reconcile | ACCEPTED (series RVS__…__S4; Step 1 R1→R2 NO OBJECTION; Step 2 R1 NO OBJECTION; Step 3 R1→R2 DO NOT ADVANCE→R3 NO OBJECTION; Step 4 R1→R2 DO NOT ADVANCE→R3 NO OBJECTION; all 16 ACs verified; 182 tests pass; 7 in-scope blockers found+fixed across the change) | COMPLETE | — |
    81	| UPG-0057 | CHG-20260726-003 | downstream-doctrine | downstream doctrine only | 4-Reconcile | CHG-A ACCEPTED and committed (fd2b198) — pattern + downstream doctrine + prompt wiring; manual status-line inclusion accepted as CHG-A-scope only, not final operating model | COMPLETE | UPG-0057 (CHG-B) |
    82	| UPG-0057 | CHG-20260727-001 | script-tooling + downstream-doctrine + self-dev-governance | both | 4-Reconcile | CHG-B: automatic Controlled Plain English status injection implemented in `scripts/codeos-review.sh` (no Rust changes, 182 tests unchanged); wrapper established as supported entry point; Step 3 ACCEPTED by human decision (budget exhausted, no in-scope blocker, SECRET_REDACTION confirmed false positive recorded in Implementation Notes); Step 4 Reconciliation written, all 13 ACs PASS; review pending | IN_PROGRESS | — |
    83	
    84	<!--
    85	Identity   : Feature ID = UPG-#### (stable). Change ID = CHG-YYYYMMDD-NNN (one execution), or a
    86	             legacy change-stem (0001..0004) for historical/piloted work recorded truthfully.

codex
I’ve got two hard blockers already. I’m checking one last point around stage identity, because the packet is labeled `selfdev-step-1` while the diff also records Step 4 completion state, and that may itself be a scope-boundary failure rather than just bookkeeping noise.
exec
/bin/bash -lc "nl -ba CLAUDE.md | sed -n '85,125p'" in /home/rimo/projects/Codeos
 succeeded in 0ms:
    85	Each step: **produce output → run the Codex review if required by profile → STOP at the
    86	gate → human approves → next step.** (Profile governs review cadence — see Step 0a of
    87	`prompts/codeos-self-dev.md`.)
    88	
    89	1. **Change Intent** — Why (problem in the toolkit), what changes (named files), what stays
    90	   the same (scope boundary), triage class, scope axis, originating backlog id. Start the
    91	   change record; activate the row in `status/self-development.md`.
    92	2. **Acceptance Criteria** — The consistency contracts the change must hold.
    93	   *Doctrine/downstream:* cross-reference integrity, downstream-compatibility (the generated
    94	   project still loads `.codeos/dba-system.md`; stage tables, prompt filenames, and
    95	   references move together), no internal contradiction. *Tooling:* I/O behavior, exit-code /
    96	   fail-closed cases, idempotency.
    97	3. **Implement** — Edits constrained to the approved scope. Update **all** cross-references
    98	   in the same change. No scope creep. If a change you discover is out of scope, stop and
    99	   re-triage it as its own change.
   100	4. **Reconcile** — Verify each acceptance criterion. Sweep the toolkit for stale references,
   101	   orphaned links, and stage-table↔prompt-file drift (grep). For tooling, do a smoke run.
   102	   Apply reviewer **scope triage** (IN-SCOPE BLOCKER / IN-SCOPE NON-BLOCKER /
   103	   OUT-OF-SCOPE BACKLOG / REJECTED / SELF-REFERENCE / REVIEW-BOOKKEEPING). Mark the row COMPLETE in `status/self-development.md`;
   104	   log the decision (see Review Logging).
   105	
   106	### Review cadence and advisory verdict
   107	
   108	Review cadence is governed by the **review profile** assigned in Step 0a of
   109	`prompts/codeos-self-dev.md`. High-risk profiles (PROFILE-3 through PROFILE-5) require
   110	a Codex review before each step gate. Lighter profiles (PROFILE-1, PROFILE-2) may
   111	limit Codex review to Reconcile only or reduce the round budget, as defined by the
   112	profile. Human approval at each step transition is required at every profile; reviewer
   113	output is advisory and non-gatekeeping at every profile.
   114	
   115	To run the reviewer:
   116	
   117	```
   118	bash scripts/codeos-review.sh review UPG-####__CHG-YYYYMMDD-NNN selfdev-step-<N> changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md <touched-files>
   119	```
   120	
   121	The verdict is **advisory** — NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE inform
   122	the human's decision but never auto-block. The reviewer is independent, read-only, and
   123	non-gatekeeping; the human decides at the gate.
   124	
   125	### Gate discipline

codex
Acceptance criteria are not directly reviewable from this packet. The only AC-related evidence shown here is the bookkeeping claim that “all 13 ACs PASS” in [status/self-development.md](/home/rimo/projects/Codeos/status/self-development.md:82); the packet does not include the CHG-B acceptance-criteria/reconciliation artifact itself.

Finding: Automatic CPE status injection breaks documented delta-mode `review`/`plan`  
Severity: High  
Classification: IN-SCOPE BLOCKER  
Evidence: [scripts/codeos-review.sh](/home/rimo/projects/Codeos/scripts/codeos-review.sh:94), [scripts/codeos-review.sh](/home/rimo/projects/Codeos/scripts/codeos-review.sh:102), [docs/reviewer-pipeline.md](/home/rimo/projects/Codeos/docs/reviewer-pipeline.md:515), [docs/reviewer-pipeline.md](/home/rimo/projects/Codeos/docs/reviewer-pipeline.md:620), [docs/reviewer-pipeline.md](/home/rimo/projects/Codeos/docs/reviewer-pipeline.md:625), [backlog/UPG-0057-controlled-plain-english-writing-discipline.md](/home/rimo/projects/Codeos/backlog/UPG-0057-controlled-plain-english-writing-discipline.md:98)  
Why: The wrapper now always appends an untracked temp file to `review` and `plan`, but delta mode is documented to error on untracked artifacts. The packet also claims there were no Rust-engine changes, so there is no shown exemption path. This breaks a stated supported mode.  
Required action: fix now  
Scope reason: The PR’s stated scope is wrapper-level automatic status injection for `review` and `plan`; breaking a documented reviewer mode is directly in scope.

Finding: CHG-B / UPG-0057 state claims are internally contradictory  
Severity: Medium  
Classification: IN-SCOPE BLOCKER  
Evidence: [backlog/UPG-0057-controlled-plain-english-writing-discipline.md](/home/rimo/projects/Codeos/backlog/UPG-0057-controlled-plain-english-writing-discipline.md:90), [backlog/UPG-0057-controlled-plain-english-writing-discipline.md](/home/rimo/projects/Codeos/backlog/UPG-0057-controlled-plain-english-writing-discipline.md:112), [status/self-development.md](/home/rimo/projects/Codeos/status/self-development.md:82)  
Why: The backlog note says “With `CHG-B` accepted, `UPG-0057` is complete,” the change table still marks `CHG-20260727-001` as `DRAFT`, and the status dashboard says Step 4 is written but review is pending and the row is `IN_PROGRESS`. Those states cannot all be true at once.  
Required action: fix now  
Scope reason: This change explicitly updates self-development governance/bookkeeping, so truthful stage/state reporting is in scope.

Finding: The packet scope does not match its stated review stage  
Severity: Medium  
Classification: IN-SCOPE BLOCKER  
Evidence: [/tmp/codeos-cpe-status.wTywjn](/tmp/codeos-cpe-status.wTywjn:3), [CLAUDE.md](/home/rimo/projects/Codeos/CLAUDE.md:89), [CLAUDE.md](/home/rimo/projects/Codeos/CLAUDE.md:97), [CLAUDE.md](/home/rimo/projects/Codeos/CLAUDE.md:100), [scripts/codeos-review.sh](/home/rimo/projects/Codeos/scripts/codeos-review.sh:38), [scripts/codeos-review.sh](/home/rimo/projects/Codeos/scripts/codeos-review.sh:102), [status/self-development.md](/home/rimo/projects/Codeos/status/self-development.md:82)  
Why: The synthetic status artifact and review context say this is `selfdev-step-1`, which `CLAUDE.md` defines as Change Intent. But the reviewed diff includes shipped script behavior and Step 4 reconciliation bookkeeping. That is beyond the Step 1 boundary and prevents a clean stage-scoped assessment.  
Required action: fix now  
Scope reason: Stage identity and gate separation are explicit self-development rules; this belongs to the PR’s stated scope, not backlog.

PR decision: DO NOT ADVANCE  
Scope drift warning: yes — the packet is labeled `selfdev-step-1`, but it includes implementation in [scripts/codeos-review.sh](/home/rimo/projects/Codeos/scripts/codeos-review.sh:1) and Reconcile-era status/bookkeeping updates in [status/self-development.md](/home/rimo/projects/Codeos/status/self-development.md:82) and [backlog/UPG-0057-controlled-plain-english-writing-discipline.md](/home/rimo/projects/Codeos/backlog/UPG-0057-controlled-plain-english-writing-discipline.md:90).

LOG SUMMARY: DO NOT ADVANCE — automatic CPE injection appears to break the documented delta-mode review path
EVIDENCE: A
HIGHEST-IMPACT UNCERTAINTY: If the reviewer engine already special-cases the injected temp artifact during delta-mode packet building despite no engine changes shown here, the first blocker would weaken materially.
tokens used
56,187
