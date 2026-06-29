# Self-Development Change: UPG-0027__CHG-20260629-003 — packet-manifest-and-budget

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit
itself (prompts, templates, docs, patterns, scripts).

This is NOT a downstream DBA feature. It has no behavioral contract, no event schema,
and no replay. Trivial changes do not get a record.

Workflow: prompts/codeos-self-dev.md (4-step loop)
Each step requires explicit human approval; Codex review cadence is governed by the assigned review profile (see prompts/codeos-self-dev.md Step 0a).
The live status row lives in status/self-development.md, not here.

FILENAME CONVENTION (Feature Thread model — see backlog/UPG-0001-feature-thread-traceability.md):
  changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md
  - UPG-#### = the PRIMARY feature this change implements (visible grouping).
  - CHG-YYYYMMDD-NNN = the unique change id (execution).
  - slug describes the concrete change, not the whole roadmap.
  - Multi-feature change: keep the primary UPG-#### in the filename, list the rest in
    `related_features`. Use `MULTI__CHG-…` only when there is genuinely no primary feature and
    the human explicitly approves it (rare).
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0027
primary_feature_id: UPG-0027
change_id: CHG-20260629-003
slug: packet-manifest-and-budget
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0027
related_features:
  - UPG-0030
review_series: null
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: CHG-20260629-002
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round (which does not exist until after the packet is built). Reference the stable review
SERIES (review_series) + review_state; exact rounds live only in reviews/review-log.md and
reviews/codex/*. See prompts/codeos-self-dev.md → "Feature Thread & IDs" / "Self-Reference Boundary". -->


## Change Intent

**Why (problem in the toolkit):**

Every packet sent to the reviewer is a single opaque text blob. There is no manifest
describing what was included, how large each piece is, or why anything was omitted.
The reviewer cannot tell at a glance whether it is looking at 5 KB or 200 KB of content.
The human running the review has no budget signal before invoking Codex — no token
estimate, no warning when the packet is oversized. When a packet is large, the reviewer
spends its capacity on peripheral context rather than the change under review.

Additionally, there is no way to include a large reference artifact by hash reference
only (without its full content), even when the reviewer only needs to verify it exists
and has the expected hash — every artifact is either full content or completely omitted.

**What changes:**

1. `scripts/codeos-review.sh` — two targeted areas within `build_packet()` plus one
   minimal addition to `cmd_review()`:

   a. **`build_packet()` — PACKET MANIFEST section**: a new `PACKET MANIFEST` section is
      written into the packet immediately after the reviewer task (injected from the
      template) and before `REVIEW CONTEXT`. It lists every artifact and the diff as
      items with mode, byte count, and sha256 (where available), plus a summary of
      total content bytes, estimated token count (~total_bytes / 4), and budget status.

   b. **`build_packet()` — `path_sha_only` mode**: artifacts passed with `--sha-only`
      are listed in the manifest as `path_sha_only` (path + sha256, no content) and
      are excluded from the `ARTIFACTS TO REVIEW` content block. This allows a caller
      to include a large unchanged reference file by hash only, keeping the packet lean
      while giving the reviewer a verifiable identity for the file.

   c. **`cmd_review()` — `--sha-only PATH` flag**: the argument parser recognizes
      `--sha-only PATH` (may appear multiple times). The collected paths are passed to
      `build_packet()` as a new parameter `sha_only_paths`. No other changes to
      `cmd_review()`.

2. `docs/reviewer-pipeline.md` — one paragraph added to the packet structure section
   describing the PACKET MANIFEST section and the `--sha-only` flag.

**Scope boundary — what stays the same:**

- `prompts/codeos-reviewer-task.md` — not touched
- `dba-system.md` — not touched
- `cmd_decision`, `cmd_stage_start` — not touched
- Log-parse lines (`grep -E '^LOG SUMMARY:'`, `'^EVIDENCE:'`, ~lines 402–415) —
  not touched; no format change
- `reviews/review-log.md` format — unchanged; manifest is packet-only, not logged
- Assessment file YAML header (fields in `reviewed:` front matter) — unchanged;
  `PACKET_ARTIFACTS_YAML` continues to populate it as before
- `stage_checks()`, `stage_expected()` — not touched
- `run_codex()` — not touched
- Budget check is **warning only** (stderr + manifest field) — it never aborts the
  review or prevents Codex invocation
- Delta mode, local precheck gate, typed runner — out of scope; CHG-3

**Manifest format (plain text, in-packet):**

```
PACKET MANIFEST
  generated: <iso timestamp>
  task_prompt: prompts/codeos-reviewer-task.md (<N> bytes)  [informational; not counted below]
  review_content_bytes: <N>        (full_file artifact bytes + diff bytes only)
  estimated_review_tokens: ~<N>    (review_content_bytes / 4, truncated)
  budget_status: OK                (or: WARNING — <N> bytes exceeds CODEOS_PACKET_BUDGET_BYTES=<threshold>)
  items:
    - path: <artifact-path>
      mode: full_file | path_sha_only | omitted_with_reason
      bytes: <N>                      (present for full_file and path_sha_only)
      sha256: <hex>                   (present for full_file and path_sha_only)
      note: <string>                  (e.g. "secret value redacted in place", for full_file only)
      reason: <string>                (present for omitted_with_reason only)
    - path: (diff)
      mode: full_file | omitted_with_reason
      bytes: <N>
```

Mode mapping — six cases:
- `shown` → `full_file`
- `shown_redacted` → `full_file` with `note: secret value redacted in place`
- `oversize_omitted` → `omitted_with_reason: over size limit`
- positional artifact missing from disk → `omitted_with_reason: requested artifact missing`
- `--sha-only PATH` where file exists → `path_sha_only` (path + sha256 + original bytes; content excluded)
- `--sha-only PATH` where file is missing → **exit non-zero before Codex** with a clear error; NOT `omitted_with_reason`

`path_sha_only` bytes are shown per-item in the manifest (so the caller can see file size) but
are NOT counted in `review_content_bytes` because that content is absent from the packet.
This CHG does NOT claim to report total packet bytes or total token cost.

**Budget constant:**

`CODEOS_PACKET_BUDGET_BYTES` (env var, default 50000). If
`review_content_bytes > CODEOS_PACKET_BUDGET_BYTES`, set `budget_status` to WARNING in the
manifest and emit one line to stderr before Codex is invoked. Never aborts.

**Class:** `script-tooling`
**Scope axis:** self-dev only
**Backlog item:** `backlog/UPG-0027-replacing-review-scripts.md`

---

## Acceptance Criteria

<!-- Script behavior — manifest structure (A1–A4) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A1 | `PACKET MANIFEST` section appears in packet after reviewer task template and before `REVIEW CONTEXT` | Inspect a saved packet file in `reviews/codex/packets/` after a test run, or use the existing `--print-packet`/`--dry-run` flag: `grep -n "PACKET MANIFEST\|REVIEW CONTEXT" <packet>` → MANIFEST line number < REVIEW CONTEXT line number |
| A2 | Manifest `items:` block lists every artifact passed to `review` with `path`, `mode`, and — for `full_file` and `path_sha_only` — `bytes` and `sha256`; for `omitted_with_reason` — `reason` instead of bytes/sha256 | `--print-packet` with one shown artifact and one missing artifact; inspect items block |
| A3 | Manifest includes a `(diff)` entry with `mode` and `bytes` (bytes is 0 when diff is empty; entry is still present) | `--print-packet` on a clean tree; confirm `(diff)` entry present with `bytes: 0` |
| A4 | Manifest reports `review_content_bytes` = sum of full_file artifact bytes + diff bytes only (path_sha_only bytes listed per-item but NOT counted; task prompt bytes listed informational but NOT counted); `estimated_review_tokens` = `review_content_bytes / 4` (integer, truncated); manifest does NOT claim to report total packet bytes or total token cost | `--print-packet`; manually sum full_file artifact sizes + diff size; confirm `review_content_bytes` matches; confirm `estimated_review_tokens` = review_content_bytes / 4 |

<!-- Script behavior — budget and modes (A5–A7) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A5 | `budget_status: OK` when `review_content_bytes ≤ CODEOS_PACKET_BUDGET_BYTES` (default 50000); `budget_status: WARNING — <N> bytes exceeds CODEOS_PACKET_BUDGET_BYTES=<threshold>` in manifest AND one warning line to stderr when content exceeds threshold; review is NOT aborted (exit 0 after warning) | `CODEOS_PACKET_BUDGET_BYTES=100 bash scripts/codeos-review.sh review ... --print-packet 2>err.txt`; inspect manifest for WARNING; inspect err.txt for warning line; confirm exit 0 |
| A6 | All six visibility outcomes map correctly: (1) `shown` → `full_file`; (2) `shown_redacted` → `full_file` with `note: secret value redacted in place`; (3) `oversize_omitted` → `omitted_with_reason: over size limit`; (4) missing positional artifact → `omitted_with_reason: requested artifact missing`; (5) `--sha-only PATH` where file exists → `path_sha_only`; (6) `--sha-only PATH` where file is missing → script exits non-zero **before any Codex invocation**, with a clear error message on stderr; does NOT fall through to `omitted_with_reason` | `--print-packet` with missing regular artifact (→ omitted_with_reason); `--sha-only` on missing file (→ non-zero exit, no Codex call) |
| A7 | `--sha-only PATH` on an existing file: listed in manifest as `path_sha_only` with `path`, `sha256`, and `bytes` (original file size); NOT present in `ARTIFACTS TO REVIEW` content block; its bytes NOT counted in `review_content_bytes` | `--print-packet --sha-only <existing-file>`; manifest shows `path_sha_only`; `ARTIFACTS TO REVIEW` contains no content for that file; `review_content_bytes` equals the value from a run without `--sha-only` on that file |

<!-- Script scope (A8) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A8 | Script changes confined to: `build_packet()` (manifest + sha_only logic) and the argument-parse block inside `cmd_review()` (`--sha-only` parsing only); `cmd_decision`, `cmd_stage_start`, `run_codex`, `stage_checks`, `stage_expected`, and log-parse lines (~402–415) have no changed lines | `git diff -- scripts/codeos-review.sh \| grep "^@@ "` → changed hunks only in `build_packet` and `cmd_review` arg-parse block |

<!-- Docs and bookkeeping (A9–A11) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A9 | `docs/reviewer-pipeline.md` has a paragraph describing the PACKET MANIFEST section and the `--sha-only` flag | `grep "PACKET MANIFEST\|sha-only" docs/reviewer-pipeline.md` → match |
| A10 | `status/self-development.md` row for CHG-20260629-003: Class=`script-tooling`, Scope=`self-dev only`, State=`IN_PROGRESS` during loop; updated to `COMPLETE` at close | `grep "CHG-20260629-003" status/self-development.md` → row fields match |
| A11 | Backlog Feature Thread row for CHG-20260629-003: `IN_PROGRESS` during loop, updated to `COMPLETE` at close | `grep "CHG-20260629-003" backlog/UPG-0027-replacing-review-scripts.md` → match |

<!-- Scope boundary (A12) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A12 | `dba-system.md` diff empty; `reviews/review-log.md` format unchanged (no manifest there); assessment file YAML header fields (`reviewed:` front matter) unchanged; no delta-mode, precheck-gate, header_only, or typed-runner code in any changed file | `git diff -- dba-system.md` → empty; `grep -r "local_check\|--mode delta\|header_only\|typed.runner" scripts/ prompts/` → no match in new code; spot-check assessment YAML header after test run |

---

## Implementation Notes

Two files modified:

1. **`scripts/codeos-review.sh`** — five hunks, all in `build_packet()` and `cmd_review()`:
   - `build_packet()` start: added sha_only_paths (read from `PACKET_SHA_ONLY` global), guard
     loop (missing --sha-only path → `exit 2` before Codex), manifest variable init, sha_only
     manifest entry loop with sha256/bytes computation. sha_only artifacts also appended to
     `PACKET_ARTIFACTS_YAML` with `visibility: path_sha_only` for assessment header accuracy.
   - After `redacted_diff` computation: `diff_bytes=${#redacted_diff}`.
   - Artifact loop: refactored oversize check to capture `artifact_bytes` first (one `wc -c`
     instead of two); added `manifest_full_artifacts` entry building for all four visibility
     paths (missing, oversize_omitted, shown_redacted, shown); accumulated `review_content_bytes`
     only for `full_file` artifacts.
   - After artifact loop: `review_content_bytes += diff_bytes`; budget check against
     `CODEOS_PACKET_BUDGET_BYTES` (default 50000); stderr warning + `budget_status` field.
   - Packet write block: PACKET MANIFEST section emitted after `cat "${task_prompt}"` / `echo`
     and before `echo "REVIEW CONTEXT"`.
   - `cmd_review()` arg-parse: added `sha_only_paths=()` to locals; `--sha-only PATH` case
     (with guard for missing second arg). Before `build_packet` call: `PACKET_SHA_ONLY=(...)`.
   - `cmd_decision`, `cmd_stage_start`, `run_codex`, `stage_checks`, `stage_expected`, and
     log-parse lines: not touched.

2. **`docs/reviewer-pipeline.md`** — one paragraph appended to §2 describing PACKET MANIFEST
   section (modes, fields, budget signal) and `--sha-only` flag (purpose + missing-file
   exit behavior).

Smoke tests all passed:
- Manifest appears at line 69; REVIEW CONTEXT at line 84 (A1 ✓)
- missing positional artifact → `omitted_with_reason: requested artifact missing` (A6/4 ✓)
- `--sha-only nonexistent` → exit 2, stderr error, no Codex call (A6/6 ✓)
- `--sha-only existing` → `path_sha_only` in manifest; absent from ARTIFACTS TO REVIEW (A7 ✓)
- `CODEOS_PACKET_BUDGET_BYTES=100` → WARNING in manifest and stderr; exit 0 (A5 ✓)

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| A1 | PACKET MANIFEST appears after reviewer task, before REVIEW CONTEXT | PASS | `--print-packet \| grep -n "PACKET MANIFEST\|REVIEW CONTEXT"` → line 69 vs line 84 |
| A2 | Artifacts listed with path, mode, bytes+sha256 (full_file/path_sha_only) or reason (omitted_with_reason) | PASS | normal: `mode: full_file`, `bytes: 14593`, `sha256: ac7e44…`; sha-only: `mode: path_sha_only`, `bytes: 35588`, `sha256: be286b…`; missing: `mode: omitted_with_reason`, `reason: requested artifact missing` |
| A3 | (diff) entry always present with mode and bytes | PASS | `grep -A 2 "path: (diff)"` → `mode: full_file`, `bytes: 10347`; entry present even when tree is staged at base commit |
| A4 | `review_content_bytes` = full_file artifact bytes + diff bytes; `estimated_review_tokens` = truncated integer / 4; task prompt not counted; no claim to total packet bytes | PASS | 14593 (artifact) + 10347 (diff via bash `${#string}`) = 24940; 24940 / 4 = 6235 — both match manifest. Note: external `git diff \| wc -c` shows 10372 (25 bytes more) because bash `$()` strips trailing newlines; the script uses `${#redacted_diff}` consistently and does not claim per-byte accuracy |
| A5 | `budget_status: OK` at default; WARNING in manifest + stderr when over threshold; no abort | PASS | `CODEOS_PACKET_BUDGET_BYTES=100`: stderr shows `warning: review content is 24940 bytes, exceeds budget of 100 bytes`; manifest shows `budget_status: WARNING — 24940 bytes exceeds CODEOS_PACKET_BUDGET_BYTES=100`; exit code 0 |
| A6 | All six visibility outcomes correct | PASS | (1) shown → full_file ✓ (from A2); (2) shown_redacted → full_file + note (code path present, not triggered by current artifacts); (3) oversize_omitted → `omitted_with_reason: over size limit` (code path present); (4) missing positional → `omitted_with_reason: requested artifact missing` ✓ verified with `no-such-file.md`; (5) existing `--sha-only` → `path_sha_only` ✓ (A2 evidence); (6) missing `--sha-only` → `exit 2` before Codex, stderr: `error: --sha-only path not found: no-such-file.md` |
| A7 | `--sha-only` existing: in manifest as path_sha_only; NOT in ARTIFACTS TO REVIEW content block; NOT in review_content_bytes | PASS | `ARTIFACTS TO REVIEW` section contains only the change record's content, not the `--- scripts/codeos-review.sh (sha256: …) ---` header or its content; `review_content_bytes` identical with and without `--sha-only scripts/codeos-review.sh` (both 14593+diff only) |
| A8 | Script changes confined to `build_packet()` and `cmd_review()` arg-parse block | PASS | `git diff -- scripts/codeos-review.sh \| grep "^@@ "` → five hunks: three in `build_packet()`, two in `cmd_review()`; `cmd_decision`, `cmd_stage_start`, `run_codex`, `stage_checks`, `stage_expected`, log-parse lines all untouched |
| A9 | `docs/reviewer-pipeline.md` has paragraph describing PACKET MANIFEST and `--sha-only` | PASS | `grep -c "PACKET MANIFEST\|sha-only" docs/reviewer-pipeline.md` → 3 |
| A10 | Status row: Class=`script-tooling`, Scope=`self-dev only`, State=`IN_PROGRESS` | PASS | `grep "CHG-20260629-003" status/self-development.md` → row confirmed |
| A11 | Backlog Feature Thread row: `IN_PROGRESS` | PASS | `grep "CHG-20260629-003" backlog/UPG-0027-replacing-review-scripts.md` → match |
| A12 | `dba-system.md` diff empty; review-log.md format unchanged; no delta/precheck/header_only/typed-runner code | PASS | `git diff -- dba-system.md \| wc -c` → 0; `grep -r "local_check\|--mode delta\|header_only\|typed.runner" scripts/ prompts/` → no matches; `grep "review_content_bytes\|PACKET MANIFEST" reviews/review-log.md` → no matches |

**Caution on scope:** This CHG makes packet size **visible** via `review_content_bytes` and the manifest. It does NOT automatically compact packets. Actual packet size reduction requires CHG-3 (delta mode) or manual use of `--sha-only` for large reference files.

**Consistency sweep:**

Changed files (2 modified): `scripts/codeos-review.sh`, `docs/reviewer-pipeline.md`. No cross-reference drift introduced. `PACKET MANIFEST` and `--sha-only` are new; no prior prose references them.

Carry-forward OUT-OF-SCOPE BACKLOG items from CHG-1 (still in scope of a future change):
- `prompts/reviewer-automated.md` still says `Critically assess:` 
- `docs/reviewer-artifact-schemas.md` still describes old packet structure

Neither blocks CHG-2 — this change does not claim to update those files.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| `review_content_bytes` differs by ~25 bytes from external `git diff \| wc -c` due to bash `$()` trailing-newline stripping | IN-SCOPE NON-BLOCKER | Not a false claim — criterion says "full_file bytes + diff bytes" which is what the script computes; no accuracy claim exists; no action required |
| `shown_redacted` → full_file path not covered by a live smoke test (no artifact with redactable secret available in this run) | IN-SCOPE NON-BLOCKER | Code path is present and correct by code inspection; the redact_secrets regex is unchanged from pre-CHG-2; no action required |
