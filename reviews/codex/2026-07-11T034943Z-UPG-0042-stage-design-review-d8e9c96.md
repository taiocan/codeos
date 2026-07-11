---
reviewed:
  feature: UPG-0042
  stage: design-review
  branch: main
  base_commit: (no base pin)
  review_commit: d8e9c96c8ef6c030d19259ae2a061f926f1ed07b
  artifacts:
    - path: docs/superpowers/specs/2026-07-11-reviewer-packet-efficiency-design.md
      sha256: 64b07c44639444f48d226e5e962cf4dc353919154a80100462acaef768a1c3c3
      visibility: shown
  diff_hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
  coverage_state: FULL_COVERAGE
  workspace_dirty: true
  redaction_count: 0
  secret_redaction: false
  excluded_paths: []
  reviewed_packet: packets/20260711T034943Z-UPG-0042-stage-design-review-d8e9c96.packet.txt
  reviewed_packet_sha256: 512af7b1acf4050df53d276b2e673c149b83e4fad7211916787d60e365f052f8
  reviewer: "codex (session 019f4f43-6d55-72f1-9cf2-6a2871adbc2d)"
  codex_concern: NO OBJECTION
  effective_concern: NO OBJECTION
  evidence: A
  reasoning_effort: high
  reconnect_count: 0
  elapsed_ms: 28806
---

1. Acceptance criteria: For this design-review stage, the artifact defines the implementation acceptance criteria and supporting plan; it does not claim they are already completed.
   - Criteria 1, 2, 7, 8: specified in Component 1 and Implementation Constraints; no implementation/run evidence, but not claimed complete.
   - Criteria 3, 4: supported at design level by the documentation/help-text sections that expose `--mode delta`, `--sha-only`, and label SHA-only as evidence-reducing.
   - Criterion 5: supported by a five-test testing strategy; no pass output, but not claimed complete.
   - Criterion 6: supported by a manual verification plan; no completion evidence, but not claimed complete.
2. Claim support: No blocker. The prior approval-status issue is fixed: status now says “Design complete, awaiting implementation approval.” Strong v1 scope claims such as “No automatic mode selection,” “No persistent state,” “No exit code changes,” and “No schema changes” are consistently repeated in scope, constraints, and out-of-scope sections.
3. Scope drift: No. The diff is empty, and the only reviewed artifact is the UPG-0042 design file.
4. In-scope blockers: None found.
5. Finding classification: No IN-SCOPE BLOCKER findings.

Non-blocking observation: The design relies on current-state claims that delta mode and SHA-only mode already exist, but the packet does not include code/help output proving that. That is a verification gap for implementation review, not a blocker for this design artifact.

PR decision: ADVANCE  
Scope drift warning: no — no out-of-scope diff or behavior change is shown.

LOG SUMMARY: NO OBJECTION — The design artifact is internally scoped, human-gated, and no in-scope blocker remains.
EVIDENCE: A
HIGHEST-IMPACT UNCERTAINTY: Whether the claimed existing delta and SHA-only modes behave exactly as described, since their implementation is not included in this packet.
OpenAI Codex v0.142.5
--------
workdir: /home/rimo/projects/Codeos
model: gpt-5.5
provider: openai
approval: never
sandbox: read-only
reasoning effort: high
reasoning summaries: none
session id: 019f4f43-6d55-72f1-9cf2-6a2871adbc2d
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
  generated: 2026-07-11T03:49:14Z
  task_prompt: /home/rimo/projects/Codeos/prompts/codeos-reviewer-task.md (4959 bytes)
  review_content_bytes: 12394
  estimated_review_tokens: ~3098
  budget_status: OK
  packet_mode: full
  delta_base: none
  items:
    - path: docs/superpowers/specs/2026-07-11-reviewer-packet-efficiency-design.md
      mode: full_file
      bytes: 12394
      sha256: 64b07c44639444f48d226e5e962cf4dc353919154a80100462acaef768a1c3c3
    - path: (diff)
      mode: full_file
      bytes: 0

REVIEW CONTEXT
  Feature:                UPG-0042
  Stage:                  design-review
  Branch:                 main
  Base commit:            (no base pin)
  Review commit:          d8e9c96c8ef6c030d19259ae2a061f926f1ed07b (+ uncommitted workspace changes)
  Current approved stage: n/a (non-numeric stage)
  Evidence coverage:      FULL_COVERAGE
  Workspace dirty:        yes (uncommitted changes at review time)

DBA RULES RELEVANT TO THIS STAGE
  - Human approval is required for every stage transition; you are advisory only.
  - Memory is not truth — assess only what is provided, pinned to the review commit.
  - Implementation must trace to approved artifacts; no behavior beyond intent+contract+schema.
  - No events outside the approved event schema; no hidden behavior.

STAGE-SPECIFIC CHECKS
  - (no stage-specific checklist for stage design-review)

EXPECTED STAGE OUTPUT
  (no expected-output template for stage)

ARTIFACTS TO REVIEW
  --- docs/superpowers/specs/2026-07-11-reviewer-packet-efficiency-design.md (sha256: 64b07c44639444f48d226e5e962cf4dc353919154a80100462acaef768a1c3c3, visibility: shown) ---
    # Reviewer Packet Efficiency — Design v1
    
    **Feature:** UPG-0042  
    **Date:** 2026-07-11  
    **Status:** Design complete, awaiting implementation approval
    
    ## Problem
    
    Reviewer packets repeatedly include large stable files in full, even when only small portions changed. During UPG-0041, every review round included the full 121 KB `smoke.rs` file, causing:
    
    - Packet bloat: 192-202 KB packets vs. 50 KB budget (4× overage)
    - Token waste: ~50-60k tokens per round × 8 rounds
    - Review noise: reviewer processes full stable content instead of targeted evidence
    - Context pressure: large packets consume space for deeper analysis
    - Human inspectability: 200KB packet files are hard to inspect
    
    The issue is not cache efficiency but **packet evidence design**: large stable files are included as full review evidence when targeted diffs, test lists, or hash-pinned references would suffice.
    
    ## Current State
    
    **Already implemented:**
    - Delta mode (`--mode delta --base <ref>`) — compares artifacts to base commit
    - SHA-only mode (`--sha-only <path>`) — includes only hash, not content
    - Size limiting (256KB) — files over this are auto-excluded
    - Multiple visibility modes: `shown`, `delta_diff`, `path_sha_only`, `oversize_omitted`
    
    **The gap:**
    These features exist but **aren't being used**. During UPG-0041, all 8 review rounds passed artifacts without these flags, so smoke.rs was included in full every time. The features are:
    - Undocumented (not in help text or docs)
    - Opaque (warnings don't explain what to do)
    - Undiscoverable (users don't know they exist)
    
    ## Design Decision: Explicit Mode Selection (Option A)
    
    **Chosen approach:** Explicit mode selection with enhanced warnings and documentation. **No automatic evidence downgrades in v1.**
    
    **Rationale:**
    - Preserves explicit human control (Codeos philosophy)
    - Advisory guidance, not autonomous decisions
    - If automation comes later, it should be opt-in (`--auto-evidence`), never default
    
    ## v1 Scope: Enhanced Warnings + Documentation (Approach 2)
    
    **What v1 includes:**
    1. Enhanced oversized-packet warnings with actionable suggestions
    2. Documentation of existing evidence modes
    3. Improved help text with examples
    
    **What v1 excludes:**
    - No automatic mode selection
    - No persistent state (review-commit tracking deferred to follow-up)
    - No new packet format or schema changes
    - No exit code changes
    
    ---
    
    ## Component 1: Enhanced Warning Output
    
    ### Format
    
    When packet size exceeds 50 KB budget, emit to **stderr**:
    
    ```
    warning: packet is 202 KB (4× over 50 KB budget)
      largest inputs:
        tools/reviewer/tests/smoke.rs: 121 KB (60%)
        tools/reviewer/src/packet.rs: 43 KB (21%)
      suggest for R2+:
        codeos-reviewer review UPG-0041 selfdev-step-3 --mode delta --base <last-review-commit> <artifacts>
      optional:
        use --sha-only <path> only for large unchanged context files that are not the primary artifact under review; this reduces review evidence
    ```
    
    ### Implementation Details
    
    **Location:** `tools/reviewer/src/packet.rs`, emitted after packet build completes
    
    **Trigger:** Total packet size > 50 KB (current `CODEOS_PACKET_BUDGET_BYTES`)
    
    **Largest inputs logic:**
    - Show top 3 files by size (or all if < 3 files)
    - Use full artifact-relative paths (not bare filenames) to avoid collision
    - Calculate percentage as: `(file_bytes / review_content_bytes) × 100`
    - Use same denominator (`review_content_bytes`) as budget check
    
    **Output:**
    - Emit to **stderr only** — warning must not change packet hash or content
    - No exit code change — oversized is advisory, not fatal
    - Deterministic format — always shows size, contributors, suggestion
    - No conditional logic based on round number (user decides if R2+)
    
    **Placeholders:**
    - `<last-review-commit>` — user must fill in (no auto-tracking in v1)
    
    ### Constraints
    
    1. Warning text is **not embedded in packet content** and does not affect packet hashing
    2. Warning does not change exit code
    3. Percentages use `review_content_bytes` (same as budget check)
    4. Full artifact paths shown to avoid filename collisions
    5. Top contributors capped at 3 files
    
    ---
    
    ## Component 2: Documentation — Evidence Modes
    
    Add new section to `docs/reviewer-pipeline.md`:
    
    ### §X. Evidence Modes
    
    The reviewer supports three evidence modes to control packet size and review focus. These modes affect what evidence is included in the review packet; they do not change the reviewer's advisory role or the human approval gate.
    
    #### Full Mode — default
    
    Includes full artifact content where allowed by packet size and redaction rules.
    
    **Use when:**
    - Running Round 1 of a review
    - Reviewing the primary artifact under active change
    - The reviewer needs full context to assess the artifact
    
    **Command:**
    ```bash
    codeos-reviewer review <feature> <stage> <artifact-paths>
    ```
    
    #### Delta Mode
    
    Includes only changes since a base commit. Unchanged artifacts are represented by path and hash only.
    
    **Use when:**
    - Running Round 2 or later after fixing reviewer findings
    - The packet exceeds the size budget and most artifacts are unchanged
    - The review should focus on what changed since the previous round
    
    **Command:**
    ```bash
    codeos-reviewer review <feature> <stage> --mode delta --base <commit-sha> <artifact-paths>
    ```
    
    **Guardrail:** Delta mode requires artifact paths to be tracked by git. Untracked files cannot be compared to the base commit and will error.
    
    #### SHA-Only Mode
    
    Includes only the file path and hash, not file content. **This reduces packet size but also reduces review evidence.**
    
    **Use only for:**
    - Large unchanged context files
    - Files needed for packet completeness but not for substantive review
    - Files that are not the primary artifact under review
    
    **Command:**
    ```bash
    codeos-reviewer review <feature> <stage> --sha-only <context-file> <other-artifacts>
    ```
    
    **Guardrail:** Do not use SHA-only for files whose changed behavior, wording, or structure the reviewer must assess. Changed behavior must remain reviewable as full content or diff.
    
    #### Combining Modes
    
    Delta mode and SHA-only can be combined. When both apply, SHA-only paths are included as path/hash references rather than full content or diff.
    
    ```bash
    codeos-reviewer review UPG-0042 selfdev-step-3 \
      --mode delta --base abc123 \
      --sha-only docs/large-reference.md \
      changes/UPG-0042__CHG-*.md src/packet.rs
    ```
    
    ---
    
    ## Component 3: Help Text Updates
    
    Enhance `codeos-reviewer review --help` in `tools/reviewer/src/main.rs`:
    
    ```rust
    /// Build packet, invoke reviewer, save assessment, append log
    Review {
        feature: String,
        stage: String,
    
        /// Artifact paths and options:
        ///   <paths>               Files to include in packet
        ///   --mode delta          Include only diff since base; requires --base and tracked files
        ///   --base <commit-sha>   Base commit for delta mode
        ///   --sha-only <path>     Include path/hash only; reduces review evidence
        ///   --print-packet        Print packet to stdout instead of invoking reviewer
        ///   --fresh               Force fresh session
        ///
        /// Examples:
        ///   # Round 1: full review
        ///   codeos-reviewer review UPG-0042 selfdev-step-1 \
        ///     changes/UPG-0042__CHG-*.md src/packet.rs
        ///
        ///   # Round 2+: delta review after fixes
        ///   codeos-reviewer review UPG-0042 selfdev-step-1 \
        ///     --mode delta --base abc123 \
        ///     changes/UPG-0042__CHG-*.md src/packet.rs
        ///
        ///   # Large unchanged context file; reduces evidence for that path
        ///   codeos-reviewer review UPG-0042 selfdev-step-3 \
        ///     --sha-only tests/smoke.rs \
        ///     changes/UPG-0042__CHG-*.md src/packet.rs
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        rest: Vec<String>,
    }
    ```
    
    **Key points:**
    - Explicitly states delta requires `--base` and tracked files
    - Labels `--sha-only` as "reduces review evidence"
    - Uses specific globs (`UPG-0042__CHG-*.md`) not broad wildcards
    - Shows typical patterns: R1 full, R2 delta, sha-only for context
    
    ---
    
    ## Testing Strategy
    
    ### Smoke Tests (5 new tests)
    
    ```rust
    #[test]
    fn smoke_review_oversized_packet_warning() {
        // Verify warning appears on stderr when packet > 50KB
        // Assert warning contains:
        //   - "packet is X KB (Nx over 50 KB budget)"
        //   - "largest inputs:" section with ≤3 files
        //   - "suggest for R2+:" section
        //   - "optional:" section with evidence-reducing note
        // Check stable content, not exact byte counts
    }
    
    #[test]
    fn smoke_review_warning_goes_to_stderr_not_packet() {
        // Verify warning goes to stderr, not packet content
        // Run review with oversized artifact
        // Assert warning text present in stderr
        // Assert warning text absent from packet file/stdout
    }
    
    #[test]
    fn smoke_review_delta_mode_tracked_files_only() {
        // Verify delta mode errors on untracked files
        // Create untracked artifact
        // Run review --mode delta --base HEAD <artifact>
        // Assert error mentions "untracked" and suggests staging/commit
    }
    
    #[test]
    fn smoke_review_sha_only_reduces_packet_size() {
        // Verify --sha-only excludes content from packet
        // Create large artifact
        // Run review with and without --sha-only
        // Assert --sha-only packet significantly smaller
        // Assert --sha-only shows "path_sha_only" visibility in manifest
    }
    
    #[test]
    fn smoke_review_help_mentions_evidence_modes() {
        // Verify help text regression prevention
        // Run codeos-reviewer review --help
        // Assert help contains:
        //   - "--mode delta"
        //   - "requires --base"
        //   - "tracked files"
        //   - "--sha-only"
        //   - "reduces review evidence"
        //   - "--print-packet"
    }
    ```
    
    ### Manual Verification
    
    - Run against real UPG change with oversized packet (e.g., UPG-0041 Step 3)
    - Confirm warning shows actual file contributors
    - Confirm suggested delta command works when executed
    - Verify help text is readable and examples work
    
    ---
    
    ## Files to Change
    
    | File | Change |
    |------|--------|
    | `tools/reviewer/src/packet.rs` | Add enhanced warning output after build (stderr) |
    | `tools/reviewer/src/main.rs` | Update `Review` command help text with examples |
    | `docs/reviewer-pipeline.md` | Add §X Evidence Modes section |
    | `tools/reviewer/tests/smoke.rs` | Add 5 new smoke tests |
    
    ---
    
    ## Out of Scope for v1
    
    **Deferred to future work:**
    - Review-commit tracking (Approach 3) — file as follow-up UPG
    - Automatic evidence mode selection
    - Per-file change detection in warnings
    - Test-file-specific summary modes (test counts, etc.)
    - `--auto-evidence` opt-in mode
    - No schema or packet-format change beyond diagnostic warning output
    
    ---
    
    ## Success Criteria
    
    *Implementation must satisfy all criteria below:*
    
    1. Oversized packets (>50KB) show actionable warnings with top contributors
    2. Warning emitted to stderr only and absent from packet content
    3. Users can discover and use existing `--mode delta` and `--sha-only` flags
    4. Documentation clearly labels `--sha-only` as evidence-reducing
    5. All 5 smoke tests pass
    6. Manual verification against real oversized UPG packet succeeds
    7. No exit code change for oversized packets
    8. No packet hash changes from warning addition
    
    ---
    
    ## Implementation Constraints
    
    1. **No new persistent state** — v1 remains stateless, no `.codeos-review/` tracking
    2. **No automatic downgrades** — all mode selection remains explicit
    3. **No exit code changes** — oversized packet is advisory, not fatal
    4. **Backward compatible** — existing review commands work unchanged
    5. **Warning to stderr only** — warning text not embedded in packet content and does not affect packet hashing
    6. **No schema changes** — no packet format or structure changes beyond diagnostic output
    
    ---
    
    ## Design Principles
    
    This design follows Codeos core principles:
    
    - **Explicit over automatic** — users choose evidence mode, tool advises
    - **Advisory, not gatekeeping** — warnings guide, don't block
    - **Human approval primacy** — no autonomous evidence reduction
    - **Evidence integrity** — SHA-only clearly labeled as evidence-reducing
    - **Safe-by-default** — changed files remain reviewable as content or diff
    
    If automation is added later (Approach 3 or auto-evidence mode), it should be:
    - Opt-in via explicit flag (`--auto-evidence`)
    - Never the default behavior
    - Still subject to guardrails (changed artifacts never auto-downgraded)

DIFF TO REVIEW (base->review, secret/size filtered)

codex
1. Acceptance criteria: For this design-review stage, the artifact defines the implementation acceptance criteria and supporting plan; it does not claim they are already completed.
   - Criteria 1, 2, 7, 8: specified in Component 1 and Implementation Constraints; no implementation/run evidence, but not claimed complete.
   - Criteria 3, 4: supported at design level by the documentation/help-text sections that expose `--mode delta`, `--sha-only`, and label SHA-only as evidence-reducing.
   - Criterion 5: supported by a five-test testing strategy; no pass output, but not claimed complete.
   - Criterion 6: supported by a manual verification plan; no completion evidence, but not claimed complete.
2. Claim support: No blocker. The prior approval-status issue is fixed: status now says “Design complete, awaiting implementation approval.” Strong v1 scope claims such as “No automatic mode selection,” “No persistent state,” “No exit code changes,” and “No schema changes” are consistently repeated in scope, constraints, and out-of-scope sections.
3. Scope drift: No. The diff is empty, and the only reviewed artifact is the UPG-0042 design file.
4. In-scope blockers: None found.
5. Finding classification: No IN-SCOPE BLOCKER findings.

Non-blocking observation: The design relies on current-state claims that delta mode and SHA-only mode already exist, but the packet does not include code/help output proving that. That is a verification gap for implementation review, not a blocker for this design artifact.

PR decision: ADVANCE  
Scope drift warning: no — no out-of-scope diff or behavior change is shown.

LOG SUMMARY: NO OBJECTION — The design artifact is internally scoped, human-gated, and no in-scope blocker remains.
EVIDENCE: A
HIGHEST-IMPACT UNCERTAINTY: Whether the claimed existing delta and SHA-only modes behave exactly as described, since their implementation is not included in this packet.
tokens used
37,705
