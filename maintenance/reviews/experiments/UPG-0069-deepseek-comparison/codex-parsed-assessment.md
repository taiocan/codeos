---
review_id: EXT__UPG-EXTPILOT2__selfdev-step-1__A2
findings:
    - finding_id: FND__EXT__UPG-EXTPILOT2__selfdev-step-1__A2__01
      severity: High
      classification: IN-SCOPE BLOCKER
      summary: "Packet metadata is not cryptographically bound to the exported packet bytes"
      required_action: fix now
    - finding_id: FND__EXT__UPG-EXTPILOT2__selfdev-step-1__A2__02
      severity: High
      classification: IN-SCOPE BLOCKER
      summary: "Untracked-file discovery fails open"
      required_action: fix now
    - finding_id: FND__EXT__UPG-EXTPILOT2__selfdev-step-1__A2__03
      severity: Medium
      classification: IN-SCOPE BLOCKER
      summary: "The documented external-assessment command cannot run as shown"
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
  reviewed_packet: packets/20260821T052822Z-UPG-EXTPILOT2-stage-selfdev-step-1-360b838.packet.txt
  reviewed_packet_sha256: 2a5ed7d4d74dddd95222f2327e5245e54b7d1247eacba6f7882924e7ae26894f
  source: external
  reviewer: "external assessment (gpt-5.6-sol-via-codex-exec, as supplied — not invoked or verified by Codeos)"
  reported_concern: DO NOT ADVANCE
  effective_concern: DO NOT ADVANCE
  evidence: B
  parse_status: OK
  assessment_status: COMPLETE
  highest_impact_uncertainty: "No explicit acceptance criteria or “What changes” list was supplied, so scope was inferred from UPG-EXTPILOT2 and the artifacts’ own interface and integrity claims."
  counts_as_review_round: false
---

Acceptance criteria: none are explicitly supplied, and this stage has no expected-output template. Against the artifacts’ stated guarantees:

- External assessments are recorded without invoking Codex: supported by `cmd/review.rs` and the behavioral test source.
- External assessments use separate `EXT__…__A<N>` sequencing and do not consume review rounds: supported by `log.rs` and test source.
- Unreadable or empty assessments fail before reviewer-record writes: supported by `cmd/review.rs` and test source.
- Malformed or truncated assessments become incomplete and receive an effective `DO NOT ADVANCE`: supported by `assessment.rs` and test source.
- Exact, validated packet binding: not satisfied; see Finding 1.
- Complete visibility of untracked files: not satisfied fail-closed; see Finding 2.
- Usable documented external-assessment interface: not satisfied; see Finding 3.

Strong claims are generally reflected in implementation, but the packet contains no executed test output. The claims that the sidecar checks make the imported packet trustworthy and that every untracked file is shown or recorded as excluded are contradicted directly by the implementation.

Finding: Packet metadata is not cryptographically bound to the exported packet bytes / Severity: High / Classification: IN-SCOPE BLOCKER  
Evidence: `dba/04-tools/reviewer/engine/src/cmd/review.rs:384-433`; `dba/04-tools/reviewer/contract/v4.md:43-44,56-58` / Why: `load_exported_packet` independently reads the packet and sidecar but verifies no packet hash or length. Altered or even empty packet bytes can therefore be combined with a sidecar claiming valid, non-empty coverage and then recorded as trusted reviewed evidence. / Required action: fix now  
Scope reason: Exact packet/evidence binding and empty-packet rejection are explicit guarantees of this change.

Finding: Untracked-file discovery fails open / Severity: High / Classification: IN-SCOPE BLOCKER  
Evidence: `dba/04-tools/reviewer/engine/src/packet.rs:775-799,357-419` / Why: `git_untracked_files` ignores both process-launch errors and non-successful Git exit status by returning an empty list. The packet can consequently report full coverage while silently omitting untracked files, contradicting the stated integrity purpose. / Required action: fix now  
Scope reason: Preventing invisible untracked implementation files while claiming full coverage is an explicit capability introduced by this artifact.

Finding: The documented external-assessment command cannot run as shown / Severity: Medium / Classification: IN-SCOPE BLOCKER  
Evidence: `dba/04-tools/reviewer/codeos-review-deepseek.sh:7-11`; `dba/04-tools/reviewer/contract/v4.md:53-58`; `dba/04-tools/reviewer/engine/src/main.rs:61-68` / Why: Both user-facing examples omit `--packet`, while the CLI makes it mandatory whenever `--assessment` is supplied. Following the documented workflow therefore fails argument validation. / Required action: fix now  
Scope reason: The external-assessment interface and its transport adapter are the central deliverables of this change.

In-scope blockers: yes—the packet-binding guarantee can be bypassed, coverage can fail open, and the advertised invocation is unusable as documented.

PR decision: DO NOT ADVANCE  
Scope drift warning: yes — `sds-dba.md` and `software-development-structure.md` concern workflow restructuring rather than external-review transport; treat that content as OUT-OF-SCOPE BACKLOG and separate it from this artifact.

LOG SUMMARY: DO NOT ADVANCE — imported evidence is not integrity-bound to its sidecar, allowing altered or empty packet bytes to be recorded as validated evidence
EVIDENCE: B
HIGHEST-IMPACT UNCERTAINTY: No explicit acceptance criteria or “What changes” list was supplied, so scope was inferred from UPG-EXTPILOT2 and the artifacts’ own interface and integrity claims.