---
review_id: EXT__UPG-EXTPILOT2__selfdev-step-1__A3
findings: []
unparsed_findings_count: 2
reviewed:
  feature: UPG-EXTPILOT2
  stage: selfdev-step-1
  branch: main
  base_commit: (no base pin)
  review_commit: 360b838104afb47ba99e9ea71ab493f0950436e3
  artifacts:
    - path: dba/04-tools/reviewer/contract/v4.md
      sha256: d234d5ee57486ef89de8545e943d11127a570af22944d7f2b4d3a54bf2f68bb1
      visibility: shown
    - path: dba/02-policies/review/v2.md
      sha256: 572cc2115d11fe1f8608bb281414a7c39284a679e52b6c7318f091ead60f291a
      visibility: shown
    - path: dba/04-tools/reviewer/codeos-review-deepseek.sh
      sha256: eeb398458332467df494e71ee05af682fd10d2e40f13dbe8877b16219f1e349b
      visibility: shown
  diff_hash: bee1ecabbc9fe44c2bd33866956181ba80ba89b61d98b7af98e3daa37a4772f4
  coverage_state: SECRET_REDACTION
  workspace_dirty: true
  redaction_count: 1
  secret_redaction: true
  excluded_paths: []
  reviewed_packet: packets/20260821T175347Z-UPG-EXTPILOT2-stage-selfdev-step-1-360b838.packet.txt
  reviewed_packet_sha256: 2a5ed7d4d74dddd95222f2327e5245e54b7d1247eacba6f7882924e7ae26894f
  source: external
  reviewer: "external assessment (deepseek-v4-pro, as supplied — not invoked or verified by Codeos)"
  reported_concern: CHANGES ADVISED
  effective_concern: DO NOT ADVANCE
  effective_concern_note: raised to DO NOT ADVANCE: assessment INCOMPLETE — 2 finding block(s) were malformed and could not be recorded
  evidence: B
  parse_status: FAILED
  assessment_status: INCOMPLETE
  incomplete_reason: "2 finding block(s) were malformed and could not be recorded"
  highest_impact_uncertainty: "Whether the untracked required files, especially `run.rs`, are actually added to the final commit; if they are omitted, the reviewed commit does not compile."
  counts_as_review_round: false
---

## 1. Acceptance criteria

No explicit acceptance-criteria list is supplied in the packet. The closest stated criteria are:

- The four integrity properties encoded in `dba/04-tools/reviewer/engine/tests/external_assessment.rs`:
  1. recorded packet is the exported packet byte-for-byte,
  2. untracked files are shown or downgrade coverage,
  3. findings in unrecognized shapes fail closed to `DO NOT ADVANCE`,
  4. truncated replies cannot produce a clean assessment.
  
  The implementation and tests in the packet support these: `load_exported_packet` adopts packet bytes; `packet.rs` adds `git_untracked_files`; `assessment.rs` adds `parse_status`/`assessment_status` and an incomplete-reply floor; the new tests cover those cases.

- Contract claims in `dba/04-tools/reviewer/contract/v4.md`:
  - external assessments are `source: external`, sequenced `EXT__…__A<N>`, and do not advance review rounds — supported by `log.rs`, `review.rs`, and the round test;
  - unreadable/empty assessment fails before durable writes — supported by code and `missing_or_empty_assessment_writes_no_records`;
  - process measurements are absent for external assessments — supported by `write_assessment`.

- The self-contained-build expectation implicit in the diff is **not met**: required new files are untracked and absent from the diff (Finding 1).

- The documented external-assessment invocation is **not satisfied as written**: `--packet` is mandatory in the CLI but omitted from the contract and adapter usage (Finding 2).

## 2. Claim support

- **Unsupported strong claim:** `contract/v4.md` says `review --assessment` “applies the same evidence selection, packet construction … as `review`.” The implementation does not reconstruct a packet on the import path; it requires `--packet` and adopts the exported bytes via `load_exported_packet`. This is a false interface claim and is Finding 2.
- **Supported strong claims:**
  - External assessments “never satisfies a required review round” — verified by sequence/log separation and `external_assessment_does_not_advance_the_review_round`.
  - “Unreadable or empty assessment file fails before any durable write” — verified by code path and test.
  - “DeepSeek holds no reviewer authority and writes no reviewer record” — verified by the script’s file outputs and comments.
  - “Untracked files are either shown or coverage is downgraded” — verified by `packet.rs` and the untracked-files test.

## 3. Scope drift

**Yes.** The packet contains two unrelated untracked repo-root essays, `sds-dba.md` and `software-development-structure.md`, pulled in by the new untracked-file inclusion in `packet.rs`. They have no connection to UPG-EXTPILOT2’s external-assessment/transport work and are not part of the diff. No “What changes” list was supplied, so the diff itself is the effective scope boundary.

## 4. In-scope blockers

Finding: The tracked diff is not self-contained: `run.rs` and the new DeepSeek test script are untracked and absent from the commit diff, so the pinned commit does not build/test.  
Severity: High  
Classification: IN-SCOPE BLOCKER  
Evidence: `dba/04-tools/reviewer/engine/src/main.rs` adds `mod run;`; `codex.rs` now uses `crate::run::{ReviewerRun, RunSource}`; `dba/04-tools/tests/run.sh` adds a call to `codeos-review-deepseek-tests.sh`; those files appear only in the packet’s `UNTRACKED FILES` section, not in the diff.  
Why: Without these files in the delivered commit, compilation fails (`mod run;` unresolved) and the test runner exits on the missing script. The artifact cannot run as delivered.  
Required action: fix now — add/track the required new source and test files as part of the change before advancing.  
Scope reason: The modified files directly depend on these new files; a self-contained buildable/testable artifact is required for this PR, not a future feature.

Finding: The published external-assessment interface omits the required `--packet` flag, and the contract falsely claims `review --assessment` applies the same evidence selection and packet construction as `review`.  
Severity: High  
Classification: IN-SCOPE BLOCKER  
Evidence: `dba/04-tools/reviewer/contract/v4.md` External Assessment section documents `review --assessment <file>` with no `--packet` and states it “applies the same evidence selection, packet construction”; `main.rs` declares `#[arg(long = "assessment", requires = "packet")]`; `review.rs` imports an exported packet via `load_exported_packet`; `codeos-review-deepseek.sh`’s usage block also omits `--packet`.  
Why: A user following the contract or adapter usage gets a CLI usage error. The contract describes a construction behavior the implementation deliberately replaces with “adopt exported bytes.” This is a false external-interface claim in the primary feature path.  
Required action: fix now — update the contract, adapter usage, and any downstream docs to require `--packet FILE`, and correct the wording to say the packet is adopted verbatim from the export, not reconstructed.  
Scope reason: The external-assessment interface and its integrity guarantees are the stated scope of this artifact; the public contract is part of that scope.

## 5. Finding classification

Both findings above are classified as **IN-SCOPE BLOCKER** and drive the decision. No additional findings are raised.

PR decision: REQUEST CHANGES  
Scope drift warning: yes — unrelated untracked files are pulled into the review evidence by the new untracked-file inclusion logic.  
LOG SUMMARY: CHANGES ADVISED — required new files are untracked and the public external-assessment invocation omits the mandatory `--packet`, so the change is not self-contained or correctly documented as delivered.  
EVIDENCE: B  
HIGHEST-IMPACT UNCERTAINTY: Whether the untracked required files, especially `run.rs`, are actually added to the final commit; if they are omitted, the reviewed commit does not compile.
