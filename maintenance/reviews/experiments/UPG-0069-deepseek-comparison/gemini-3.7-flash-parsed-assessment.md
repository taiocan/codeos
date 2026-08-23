---
review_id: EXT__UPG-EXTPILOT2__selfdev-step-1__A1
findings:
    - finding_id: FND__EXT__UPG-EXTPILOT2__selfdev-step-1__A1__01
      severity: Medium
      classification: IN-SCOPE BLOCKER
      summary: "Missing `--packet` argument in `codeos-review-deepseek.sh` usage snippet"
      required_action: fix now
unparsed_findings_count: 0
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
  reviewed_packet: packets/20260823T072023Z-UPG-EXTPILOT2-stage-selfdev-step-1-360b838.packet.txt
  reviewed_packet_sha256: 2a5ed7d4d74dddd95222f2327e5245e54b7d1247eacba6f7882924e7ae26894f
  source: external
  reviewer: "external assessment (gemini-3.7-flash, as supplied — not invoked or verified by Codeos)"
  reported_concern: CHANGES ADVISED
  effective_concern: CHANGES ADVISED
  evidence: B
  parse_status: OK
  assessment_status: COMPLETE
  highest_impact_uncertainty: "Whether external tooling integrations attempt to invoke review --assessment without passing --packet as illustrated in the script header."
  counts_as_review_round: false
---

### 1. Acceptance Criteria
* **External Assessment Interface & Contract Update (`dba/04-tools/reviewer/contract/v4.md`):** Satisfied. Defines `plan --emit-packet`, `review --assessment <file>`, metadata attribution (`--reviewer-label`), sequencing (`EXT__...__A<N>`), and explicit non-satisfaction of required review rounds.
* **Review Policy Alignment (`dba/02-policies/review/v2.md`):** Satisfied. Formally introduces the External Assessment section and Review Waiver progression path.
* **Byte-Identical Packet Export & Sidecar Metadata (`cmd/plan.rs`, `cmd/review.rs`, `packet.rs`):** Satisfied. `plan --emit-packet` exports canonical review packet content along with `.meta.json` sidecar; `review --assessment` verifies packet metadata matching before adopting bytes verbatim. Tested in `tests/external_assessment.rs::emitted_packet_matches_the_packet_review_sends`.
* **Fail-Closed Assessment Parsing & Validation (`assessment.rs`):** Satisfied. `count_declared_findings` checks parsed finding counts against declared classifications and escalates unparsed/unaccounted findings to `INCOMPLETE` (`DO NOT ADVANCE`). Tested in `tests/external_assessment.rs::findings_in_an_unrecognised_shape_fail_closed`.
* **Untracked File Visibility & Coverage Tracking (`packet.rs`):** Satisfied. `git_untracked_files` includes new untracked files or records exclusions downgrading coverage.
* **Transport Adapter & Test Suite (`codeos-review-deepseek.sh`, `codeos-review-deepseek-tests.sh`):** Satisfied. Pure transport implementation passing unit tests against stub server with strict argument, dependency, and termination checks.

---

### 2. Claim Support
* *"There is one packet construction path; the exported bytes are the reviewed bytes"* — Supported by `cmd/plan.rs` and `cmd/review.rs` sharing `review::prepare`, and direct verification in `tests/external_assessment.rs`.
* *"An external assessment... does not advance the review round"* — Supported by `log.rs` (`compute_review_round` counting only `REVIEW` headings) and verified in `tests/external_assessment.rs::external_assessment_does_not_advance_the_review_round`.
* *"An unreadable or empty assessment file fails before any durable write"* — Supported by pre-write validation in `cmd/review.rs` and verified in `tests/external_assessment.rs::missing_or_empty_assessment_writes_no_records`.

---

### 3. Scope Drift
* **Untracked conceptual design files (`sds-dba.md`, `software-development-structure.md`):** Untracked files in the workspace root contain architectural essays unrelated to the external assessment tooling scope.
* Core changes across contracts, policies, reviewer engine, and adapter tests remain strictly within the UPG-EXTPILOT2 scope boundary.

---

### 4. In-Scope Blockers
* The header documentation in `dba/04-tools/reviewer/codeos-review-deepseek.sh` (lines 8–11) provides a 3-step usage snippet that omits the mandatory `--packet <file>` argument on `review --assessment`. Because `main.rs` configures clap with `#[arg(long = "assessment", requires = "packet")]`, executing the documented example directly results in a CLI parsing failure.

---

### 5. Finding Classification

Finding: Missing `--packet` argument in `codeos-review-deepseek.sh` usage snippet / Severity: Medium / Classification: IN-SCOPE BLOCKER
Evidence: `dba/04-tools/reviewer/codeos-review-deepseek.sh:10` / Why: Header usage instructions omit `--packet PACKET`, which clap strictly requires when `--assessment` is provided, causing the documented example command to fail / Required action: fix now
Scope reason: Directly affects the usage instructions of the newly introduced transport script in this PR.

*Non-blocking observation:* Untracked files `sds-dba.md` and `software-development-structure.md` in repository root should be removed or moved to appropriate maintenance/concept directories to keep workspace clean (`IN-SCOPE NON-BLOCKER`).

---

PR decision: REQUEST CHANGES
Scope drift warning: yes — untracked conceptual documentation files (`sds-dba.md`, `software-development-structure.md`) present in workspace root.

LOG SUMMARY: CHANGES ADVISED — codeos-review-deepseek.sh usage instructions omit mandatory --packet flag required by reviewer CLI
EVIDENCE: B
HIGHEST-IMPACT UNCERTAINTY: Whether external tooling integrations attempt to invoke review --assessment without passing --packet as illustrated in the script header.
