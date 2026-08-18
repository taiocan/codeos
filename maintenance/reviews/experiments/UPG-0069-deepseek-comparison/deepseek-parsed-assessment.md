---
review_id: EXT__UPG-EXTPILOT2__selfdev-step-1__A1
findings: []
unparsed_findings_count: 1
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
  reviewed_packet: packets/20260818T202042Z-UPG-EXTPILOT2-stage-selfdev-step-1-360b838.packet.txt
  reviewed_packet_sha256: 2a5ed7d4d74dddd95222f2327e5245e54b7d1247eacba6f7882924e7ae26894f
  source: external
  reviewer: "external assessment (deepseek-v4-flash, as supplied — not invoked or verified by Codeos)"
  reported_concern: DO NOT ADVANCE
  effective_concern: DO NOT ADVANCE
  evidence: B
  parse_status: FAILED
  assessment_status: INCOMPLETE
  incomplete_reason: "1 declared finding(s) did not match the required finding format and were not recorded"
  highest_impact_uncertainty: "Whether an unseen wrapper layer injects --packet automatically; if so, this becomes a non-blocking documentation gap rather than a blocker."
  counts_as_review_round: false
---

Assessment pinned to review commit `360b838104afb47ba99e9ea71ab493f0950436e3`; earlier-session conclusions are not used.

## 1. Acceptance criteria

The packet provides no expected-output template, so I assess against the artifact’s own claims:

- **External-assessment path exists and is advisory** — satisfied in code, policy, contract, and tests: `review --assessment`, `ReviewerRun/RunSource`, `EXT__…__A<N>` IDs, and the policy statement that external assessments never satisfy a required review round.
- **External assessments do not advance the review round** — satisfied: `compute_review_round` counts only `REVIEW` headings; external entries use `EXTERNAL ASSESSMENT` headings and the test asserts the next Codex review is still `R1`.
- **One canonical packet construction path** — satisfied: `plan` and `review` share `prepare`; `emitted_packet_matches_the_packet_review_sends` verifies byte equality modulo the generated timestamp.
- **Fail-closed handling of malformed/truncated/unrecognized findings** — satisfied: `assessment_status: INCOMPLETE` forces `effective_concern: DO NOT ADVANCE`; tests cover truncated replies and unrecognized finding shapes.
- **Transport adapter holds no authority and writes no reviewer record** — satisfied: `codeos-review-deepseek.sh` writes only the raw reply, request/response files, and token accounting.
- **Untracked files are shown or downgrade coverage** — satisfied: packet includes `UNTRACKED FILES` content and the test confirms an unshowable untracked file downgrades coverage to `PARTIAL_COVERAGE`.

## 2. Claim support

Strong claims such as “one packet construction path”, “external assessment never satisfies a required review round”, and “recorded packet is the exported packet” are supported by shared code paths and behavioral tests.

One strong documentation claim is **not** supported: the external-assessment import command shown in `codeos-review-deepseek.sh` omits `--packet`, while the CLI requires `--packet` whenever `--assessment` is present. This is the in-scope blocker below.

## 3. Scope drift

No — the diff is coherent: external-assessment support, fail-closed parsing, exported-packet binding, and the corresponding policy/contract/terminology updates are all part of the same feature. The unrelated untracked workspace notes in the packet are not committed changes of this PR.

## 4. In-scope blockers

### Finding 1 / Severity: High / Classification: IN-SCOPE BLOCKER

**Evidence:** `dba/04-tools/reviewer/codeos-review-deepseek.sh` (header comment) documents the import step as:

```text
codeos-review.sh review <feature> <stage> --assessment ASSESSMENT \
    --reviewer-label deepseek-v4-flash <artifacts...>
```

…but `dba/04-tools/reviewer/engine/src/main.rs` defines `--assessment` with `requires = "packet"`, so the documented command exits with a usage error before doing any work. `dba/06-reference/downstream-upgrade.md` likewise says `review --assessment` records the assessment without mentioning the required `--packet`.

**Why:** The PR’s core external-assessment workflow cannot be run as documented. This creates a false claim in the artifact and prevents the documented flow from executing.

**Required action:** fix now — add `--packet PACKET` to the import command in the script comment, and make any contract/downstream usage text state that `--assessment` requires the exported packet.

**Scope reason:** This PR is explicitly about making the external-assessment path usable; the broken documented invocation is directly in scope.

## 5. Finding classification

The single finding above is classified **IN-SCOPE BLOCKER**.

No additional in-scope blockers were found.

---

PR decision: DO NOT ADVANCE  
Scope drift warning: no — the changes stay within the external-assessment/integrity scope of this PR.

LOG SUMMARY: DO NOT ADVANCE — documented external-assessment import command omits the required --packet flag  
EVIDENCE: B  
HIGHEST-IMPACT UNCERTAINTY: Whether an unseen wrapper layer injects --packet automatically; if so, this becomes a non-blocking documentation gap rather than a blocker.
