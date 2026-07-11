# Self-Development Change: UPG-0042__CHG-20260711-001 — Reviewer Packet Efficiency

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit
itself (prompts, templates, docs, patterns, scripts).

This is NOT a downstream DBA feature. It has no behavioral contract, no event schema,
and no replay. Trivial changes do not get a record.

Workflow: prompts/codeos-self-dev.md (4-step loop)
Each step requires explicit human approval; Codex review cadence is governed by the assigned review profile (see prompts/codeos-self-dev.md Step 0a).
The live status row lives in status/self-development.md, not here.

FILENAME CONVENTION (Feature Thread model — see backlog/UPG-0001-feature-thread-traceability.md):
  changes/UPG-0042__CHG-20260711-001__reviewer-packet-efficiency.md
  - UPG-0042 = the PRIMARY feature this change implements (visible grouping).
  - CHG-20260711-001 = the unique change id (execution).
  - slug describes the concrete change, not the whole roadmap.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0042
primary_feature_id: UPG-0042
change_id: CHG-20260711-001
slug: reviewer-packet-efficiency
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0042
related_features: []
review_series: RVS__UPG-0042__CHG-20260711-001__S4
review_profile: PROFILE-3
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

Reviewer packets repeatedly include large stable files in full, even when only small portions changed. During UPG-0041, every review round included the full 121 KB `smoke.rs` file, causing:

- Packet bloat: 192-202 KB packets vs. 50 KB budget (4× overage)
- Token waste: ~50-60k tokens per round × 8 rounds = 400k+ tokens
- Review noise: reviewer processes full stable content instead of targeted evidence
- Context pressure: large packets consume space for deeper analysis
- Human inspectability: 200KB packet files are hard to inspect

The issue is **packet evidence design**: delta mode (`--mode delta`) and SHA-only mode (`--sha-only`) already exist in the reviewer but are **undiscovered** — not in help text, not in docs, warnings don't explain what to do. Users don't know these modes exist, so large files are included in full across all rounds.

This change makes the existing evidence modes **discoverable and actionable** through enhanced warnings and documentation.

**What changes:**

Files to be modified:
- `tools/reviewer/src/packet.rs` — add enhanced oversized-packet warning to stderr
- `tools/reviewer/src/main.rs` — update `Review` command help text with evidence mode examples
- `docs/reviewer-pipeline.md` — add new §X Evidence Modes section (full / delta / sha-only)
- `tools/reviewer/tests/smoke.rs` — add 5 new smoke tests for warning output and evidence modes

Files to be created:
- (none — all changes are modifications to existing files)

**Scope boundary — what stays the same:**

What will NOT change:
- No automatic evidence mode selection (explicit mode selection only)
- No persistent state (no `.codeos-review/` tracking or review-commit history)
- No new packet format or schema changes (warnings go to stderr, not packet content)
- No exit code changes (oversized packets remain advisory, not fatal)
- No changes to the reviewer's advisory role or human approval gates
- No changes to downstream DBA doctrine (`dba-system.md`)
- No changes to the self-development workflow itself (`CLAUDE.md`, `prompts/codeos-self-dev.md`)

**Class:** script-tooling  
**Scope axis:** self-dev only  
**Backlog item:** backlog/UPG-0042-reviewer-packet-efficiency.md

---

## Acceptance Criteria

<!-- The consistency contracts this change must satisfy. Each must be checkable in Reconcile. -->

| # | Criterion | How it will be verified |
|---|---|---|
| AC-1 | Existing delta and sha-only modes remain semantically unchanged (this CHG only improves discoverability, not behavior) | Grep packet.rs/main.rs for changes to PacketBuildOptions, delta_mode logic, sha_only_paths logic; verify changes are warning/help-text only, not mode semantics |
| AC-2 | Oversized packets (>50KB) emit actionable warning to stderr with top 3 file contributors and delta suggestion | Manual test: create oversized packet, verify warning appears on stderr with file list and suggested command |
| AC-3 | Warning text does not appear in packet content | Manual test: verify warning text absent from packet stdout/file |
| AC-4 | Warning does not change exit code (oversized remains advisory) | Manual test: create oversized packet, verify exit code 0 (or existing non-zero if other error) |
| AC-5 | Help text (`codeos-reviewer review --help`) documents delta mode, sha-only, and includes examples | Run `codeos-reviewer review --help`, grep for `--mode delta`, `--sha-only`, `reduces review evidence`, example commands |
| AC-6 | Documentation (`docs/reviewer-pipeline.md`) includes Evidence Modes section explaining full/delta/sha-only with use-when guidance | Read reviewer-pipeline.md, verify §X Evidence Modes section exists with all three modes documented |
| AC-7 | Documentation clearly labels sha-only as "reduces review evidence" | Grep reviewer-pipeline.md and help text for "reduces review evidence" or equivalent warning |
| AC-8 | All 5 new smoke tests pass: oversized warning, warning-to-stderr-not-packet, delta-tracked-files-only, sha-only-reduces-size, help-mentions-modes | Run `cargo test --test smoke` and verify 5 new tests pass (grep test output for new test names) |
| AC-9 | No changes to packet format, schema, or manifest structure beyond warning output | Diff review of packet.rs changes; verify PacketManifest struct, visibility enum, and packet serialization unchanged |
| AC-10 | Warning non-interference: oversized-packet warning emitted to stderr only, absent from packet content, does not alter packet manifest structure, artifact visibility, or review evidence content | Manual test: inspect packet content/stdout for warning text absence; verify packet schema/manifest format unchanged via diff review |

<!-- For downstream-doctrine or both: include downstream-compatibility criteria. -->
<!-- For script-tooling: include I/O behavior, exit-code / fail-closed cases, idempotency. -->

---

## Implementation Notes

<!-- Filled during Step 3. Summary only — the git diff is the source of truth.
Note decisions, discoveries, and anything deferred (and re-triaged as its own change). -->

**Files modified:**
1. `tools/reviewer/src/packet.rs` (enhanced warning output)
   - Added `file_contributors` Vec to track (path, bytes) during packet build
   - Replaced simple warning with enhanced format showing: overage multiple, budget in KB, top 3 contributors with percentages, delta suggestion, sha-only note
   - Warning emitted to stderr only after budget check

2. `tools/reviewer/src/main.rs` (help text documentation)
   - Enhanced `Review` command doc comment with detailed option descriptions
   - Added 3 usage examples: R1 full review, R2+ delta review, sha-only for context files
   - Labeled `--sha-only` as "reduces review evidence"

3. `docs/reviewer-pipeline.md` (Evidence Modes documentation)
   - Added new § 14. Evidence Modes section before Appendix A
   - Documented Full Mode (default), Delta Mode, SHA-Only Mode with use-when guidance and guardrails
   - Included combining modes example

4. `tools/reviewer/tests/smoke.rs` (5 new smoke tests)
   - `smoke_review_oversized_packet_warning`: verifies enhanced warning format on stderr
   - `smoke_review_warning_goes_to_stderr_not_packet`: confirms warning excluded from packet content
   - `smoke_review_delta_mode_tracked_files_only`: verifies delta mode fails on untracked files
   - `smoke_review_sha_only_reduces_packet_size`: confirms sha-only excludes content (uses Cargo.toml to avoid working-tree diff issues)
   - `smoke_review_help_mentions_evidence_modes`: regression test for help text content

**All tests pass:** 124 total (119 existing + 5 new)

**Cross-references:** All documentation references updated in the same change.

**No scope creep:** All changes confined to approved scope (warnings, docs, tests). No changes to delta/sha-only semantics (AC-1), no automatic mode selection, no exit code changes, no packet schema changes.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| AC-1 | Delta/sha-only modes unchanged (discoverability only) | PASS | `git diff` shows no semantic changes to PacketBuildOptions, delta_mode, or sha_only_paths logic; only file_contributors tracking and warning output added |
| AC-2 | Oversized packets emit actionable warning with top contributors | PASS | Manual test: 155 KB packet shows "warning: packet is 155 KB (4× over 48 KB budget)" with top 3 files (smoke.rs 80%, README.md 9%, diff 9%), delta suggestion, and sha-only note |
| AC-3 | Warning text not in packet content | PASS | Manual test: warning "packet is 139 KB..." appears in stderr only; exact warning text NOT in stdout packet (only the eprintln code in diff) |
| AC-4 | Warning doesn't change exit code | PASS | Manual test: oversized packet exits 0 (advisory, not fatal) |
| AC-5 | Help text documents evidence modes with examples | PASS | `codeos-reviewer review --help` shows --mode delta, --sha-only, --base, and 3 examples (R1 full, R2+ delta, sha-only context) |
| AC-6 | Documentation includes Evidence Modes section | PASS | `docs/reviewer-pipeline.md` § 14 documents Full Mode, Delta Mode, SHA-Only Mode with use-when guidance and guardrails |
| AC-7 | Documentation labels sha-only as evidence-reducing | PASS | "reduces review evidence" appears in both reviewer-pipeline.md and main.rs help text |
| AC-8 | All 5 new smoke tests pass | PASS | `cargo test --test smoke`: 124 passed (119 + 5 new: oversized_warning, stderr_not_packet, delta_tracked_only, sha_only_reduces_size, help_mentions_modes) |
| AC-9 | No packet format/schema/manifest changes | PASS | PacketManifestEntry and ReviewPacket structs unchanged; only warning logic and file tracking added |
| AC-10 | Warning non-interference | PASS | Manual test confirms warning to stderr only, absent from packet stdout, no schema/manifest/evidence structure changes |

**Consistency sweep (grep):**
- ✓ No orphaned references to removed/renamed items
- ✓ Cross-references between help text and docs consistent
- ✓ No unresolved TODO/FIXME in implementation code
- ✓ Profile PROFILE-3 correctly set in change record
- ✓ All touched files exist and cross-reference correctly

**Findings scope-triage:**

No reviewer findings to triage (Step 4 review pending).

---
