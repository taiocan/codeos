---
change_id: CHG-20260702-002
feature_id: UPG-0015
slug: decision-provenance-binding
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0015__CHG-20260702-002__S4
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0015 / CHG-20260702-002 — Decision Provenance Binding

## TRACE HEADER

```yaml
feature_id: UPG-0015
primary_feature_id: UPG-0015
change_id: CHG-20260702-002
slug: decision-provenance-binding
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0015
related_features:
  - UPG-0032
review_series: RVS__UPG-0015__CHG-20260702-002__S4
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

The v0 `decision` command records `APPROVE_STAGE` without binding the approval to the
reviewed provenance. It does a best-effort artifact SHA check (MATCH/CHANGED) but does
not verify:

- That the saved **review packet** on disk is unmodified (`reviewed_packet_sha256` is
  stored in the assessment but never re-checked at decision time)
- Whether the reviewer ever saw **complete** evidence (`coverage_state` CRITICAL_OMISSION
  or EMPTY_PACKET means the reviewer operated blind — yet `APPROVE_STAGE` is currently
  not gated on this)
- Whether the current **HEAD** matches the `review_commit` the reviewer saw (the
  assessment records this; the decision ignores it)

The result: a logged `APPROVE_STAGE` is a record of human intent, not proof that approval
traces to an intact, reviewer-seen evidence state. A reviewer that was given an
`EMPTY_PACKET` and still issued `NO OBJECTION` can have its output approved without
challenge.

### What changes

| File | Change |
|---|---|
| `tools/reviewer/src/cmd/decision.rs` | Add `--override <reason>` flag; parse assessment frontmatter; add full provenance check; enforce coverage gate |
| `tools/reviewer/src/log.rs` | Add structured provenance block to the decision log entry |
| `tools/reviewer/src/main.rs` | Add optional `override_reason: Option<String>` to `Commands::Decision` |
| `tools/reviewer/tests/smoke.rs` | Add tests: coverage-gate block, --override accepted, packet-hash mismatch warning |
| `backlog/UPG-0015-reviewer-decision-integrity.md` | Update Feature Thread + guardrail |
| `backlog/features.md` | UPG-0015 status PROPOSED → IN_PROGRESS |
| `status/self-development.md` | Activate row |
| `status/roadmap.md` | UPG-0015 → IN_PROGRESS |

### Scope boundary — what stays the same

The following UPG-0015 sub-items are **explicitly deferred** to follow-on changes:

- Formal `COMMIT_BOUND` / `WORKSPACE_BOUND` binding mode types
- Dirty-workspace decision policy (forbid or snapshot-required while dirty)
- Durable workspace snapshots (stash/tree object)
- Rollback semantics / "last sound OK point" naming
- Per-feature structured decision ledger (vs global append-only log)
- JSON Schema for assessment header + log records
- CI enforcement of schemas/validation
- `[STALE OVERRIDE]` / `[WORKSPACE OVERRIDE]` / `[SECURITY WAIVER]` / `[COVERAGE WAIVER]`
  vocabulary (only coverage gate added here)
- Override vocabulary for artifact-hash mismatch (stays a warning, not a gate)

`dba-system.md` is not touched. Downstream doctrine unchanged.

### Behavior introduced by this change

**At `decision APPROVE_STAGE` time:**

1. Find the most recent assessment file for `feature + stage` in `reviews/codex/`
   (same logic as today, already in `verify_artifacts`).
2. Parse the assessment YAML frontmatter. Extract:
   - `coverage_state` (string)
   - `reviewed_packet_sha256` (string)
   - `review_commit` (string)
   - `reviewed_packet` path (relative, to locate the packet file)
3. **Packet integrity check**: re-hash the saved packet file on disk; compare to
   `reviewed_packet_sha256`. If mismatch: warn in log and stderr (advisory; not a gate).
4. **Commit drift check**: compare `review_commit` to current HEAD. If different: warn
   in log and stderr (advisory; not a gate — HEAD legitimately moves during fast
   iteration).
5. **Coverage gate** (software-enforced stop; human-overridable with mandatory rationale):
   if `coverage_state` is `CRITICAL_OMISSION` or `EMPTY_PACKET` → refuse `APPROVE_STAGE`
   and print:
   ```
   error: APPROVE_STAGE refused — reviewer saw incomplete evidence (coverage_state: CRITICAL_OMISSION)
          Automated progression requires complete evidence. To record explicit human
          acceptance of the associated risk, pass: --override "<rationale>"
   ```
   If `--override <reason>` is present: the stop is lifted; both the coverage state AND the
   override rationale are written to the log (the finding is NOT suppressed — the log records
   that the human intentionally accepted the risk). Consistent with Rule 1: human authority
   is preserved; automated progression is not. See AJ-011.
6. For `REQUEST_CHANGES` and `STOP`: no gate. Provenance block still written (informational).
7. Add a structured **`Provenance:`** block to the log entry:
   ```
   Provenance:
     assessment: <path>
     review_commit: <sha>  [HEAD_MATCH | HEAD_DRIFT: current=<sha>]
     packet_sha256: [MATCH | MISMATCH: stored=<s> / recomputed=<r>]
     coverage_state: <state>  [OK | COVERAGE_GATE_TRIGGERED | COVERAGE_GATE_OVERRIDDEN: <reason>]
   ```

**For `REQUEST_CHANGES` / `STOP`:** provenance block written informational; no gate.

**If no assessment exists for the feature+stage:** existing behavior (note + proceed).

### Triage

- Class: `script-tooling`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-3` (1 Codex review per step, max 3 rounds)
- Originating backlog id: `UPG-0015`

### What stays the same

- `dba-system.md` — not touched
- 9-stage DBA flow for downstream projects — not affected
- Existing `APPROVE_STAGE` behavior when coverage is FULL/PARTIAL/SECRET_REDACTION
- Advisory nature of the reviewer — nothing about this change makes the *reviewer*
  binding; it only makes the *human's approval record* carry stronger provenance
- All other subcommands (`review`, `stage-start`, `diagnose`) — unchanged

---

## Step 2 — Acceptance Criteria

Each criterion is independently verifiable against the implementation. Exit codes use the
constants in `main.rs`: EXIT_SUCCESS=0, EXIT_USAGE=1, EXIT_WRITE=5.

### AC-1 — Coverage gate blocks APPROVE_STAGE for incomplete-evidence states

When the most recent assessment for `feature+stage` has `coverage_state: CRITICAL_OMISSION`
or `coverage_state: EMPTY_PACKET`, running `decision APPROVE_STAGE` without `--override`:
- prints an error to stderr naming the coverage_state
- exits EXIT_USAGE (1)
- does **not** append anything to the review log

### AC-2 — `--override <rationale>` lifts the gate; both finding and rationale recorded

With `--override "<rationale>"` present:
- APPROVE_STAGE proceeds and exits EXIT_SUCCESS (0)
- the log entry contains **both** the coverage_state that triggered the stop AND the
  override rationale; the finding is NOT suppressed
- the Provenance block marks the gate line as `COVERAGE_GATE_OVERRIDDEN: <rationale>`

### AC-3 — Provenance block present in all decision log entries when assessment found

When an assessment file exists for the feature+stage, every decision log entry (regardless
of verdict) contains a structured `Provenance:` block with at minimum:
```
Provenance:
  assessment: <path>
  review_commit: <sha>  [HEAD_MATCH | HEAD_DRIFT: current=<sha>]
  packet_sha256: [MATCH | MISMATCH: stored=<prefix> / recomputed=<prefix>]
  coverage_state: <state>  [OK | COVERAGE_GATE_TRIGGERED | COVERAGE_GATE_OVERRIDDEN: <rationale> | INFORMATIONAL]
```
Gate markers (`COVERAGE_GATE_TRIGGERED`, `COVERAGE_GATE_OVERRIDDEN`) only appear when
`decision == APPROVE_STAGE`. For `REQUEST_CHANGES` / `STOP`, the gate line shows
`INFORMATIONAL` (no gate applied). `OK` appears when `decision == APPROVE_STAGE` and
coverage is FULL_COVERAGE / PARTIAL_COVERAGE / SECRET_REDACTION.

### AC-4 — Packet integrity check is advisory (warns; does not block)

When `reviewed_packet_sha256` in the assessment does not match the re-hashed packet file:
- a warning is printed to stderr
- the Provenance block records `MISMATCH`
- the decision is **not** blocked; exit code is unaffected
- the log entry reflects the mismatch

### AC-5 — Commit-drift check is advisory (warns; does not block)

When `review_commit` in the assessment does not equal current HEAD:
- a warning is printed to stderr
- the Provenance block records `HEAD_DRIFT: current=<sha>`
- the decision is **not** blocked; exit code is unaffected

### AC-6 — No assessment found → existing behavior preserved, no coverage gate

When no assessment file exists for the feature+stage in `reviews/codex/`:
- existing "no review on record" behavior is preserved
- no Provenance block is written
- APPROVE_STAGE proceeds; exits EXIT_SUCCESS (0)
- no coverage gate is applied (cannot gate without evidence of what the reviewer saw)

### AC-6b — Assessment file exists but is unreadable/unparseable → fail-closed

When a matching assessment file exists but cannot be read or its YAML frontmatter cannot be
parsed (missing `coverage_state` and `review_commit`):
- exit EXIT_USAGE (1) unless `--override "<rationale>"` is provided
- error to stderr names the issue and instructs the human to investigate or override
- log is NOT written when blocking (same as AC-1)
- with `--override`: decision proceeds; log Provenance block shows `PROVENANCE_UNVERIFIABLE`
  and records the override rationale

**Rationale:** a broken existing assessment is evidence that a review was attempted but its
recording failed. Silent fallthrough would lose the coverage gate without the human's knowledge.
This rule does NOT apply when no assessment exists (AC-6 above).

### AC-6c — Packet unverifiable cases are explicit in the log

When the packet file is missing, hash computation fails, or no stored sha is in the
assessment, the Provenance block records `PROVENANCE_UNVERIFIABLE: <reason>` (not silent
`not-verified`). Advisory warning is also emitted to stderr.

### AC-7 — REQUEST_CHANGES and STOP are never coverage-gated

Regardless of `coverage_state`, `decision REQUEST_CHANGES` and `decision STOP`:
- are never refused by the coverage gate
- Provenance block is written informational if assessment exists
- exit EXIT_SUCCESS (0)

### AC-8 — Override does not suppress the finding

The log entry produced when `--override` is used must contain:
- the original `coverage_state` value (e.g. `CRITICAL_OMISSION`)
- the override rationale verbatim
- the `COVERAGE_GATE_OVERRIDDEN` marker

Any log entry format that would make the coverage_state invisible (e.g. silently omitting
it when override is present) fails this criterion.

### AC-9 — Backward compatibility: non-gated path exit codes unchanged

For all decisions that are not coverage-gated (FULL_COVERAGE, PARTIAL_COVERAGE,
SECRET_REDACTION, no assessment found), exit codes match today's behavior:
- success → EXIT_SUCCESS (0)
- log append failure → EXIT_WRITE (5)
- bad verdict string → EXIT_USAGE (1)

### AC-10 — `--override` with REQUEST_CHANGES or STOP is graceful (no error)

If `--override` is passed alongside REQUEST_CHANGES or STOP, it is either silently ignored
or produces an informational note. It must not change the exit code or block the decision.

### AC-11 — Tests pass

`cargo test` passes with tests covering at minimum:
- AC-1: coverage gate blocks APPROVE_STAGE for CRITICAL_OMISSION (exit 1, no log write)
- AC-1: coverage gate blocks APPROVE_STAGE for EMPTY_PACKET (exit 1, no log write)
- AC-2: `--override` lifts gate; log contains coverage_state and rationale both
- APPROVE_STAGE not gated when coverage is FULL_COVERAGE (AC-9)
- AC-7: REQUEST_CHANGES not gated even when coverage is CRITICAL_OMISSION
- AC-6: no-assessment fallback — APPROVE_STAGE proceeds, exits 0
- AC-4: packet-hash mismatch — decision proceeds, MISMATCH in Provenance block
- AC-5: commit-drift — decision proceeds, HEAD_DRIFT in Provenance block

---

## Step 3 — Implementation

### Files changed

| File | Change |
|---|---|
| `tools/reviewer/src/main.rs` | Added `--override <RATIONALE>` option to `Commands::Decision`; pass to `cmd::decision::run` |
| `tools/reviewer/src/cmd/decision.rs` | Rewritten: load provenance, apply coverage gate (APPROVE_STAGE only), pass `DecisionProvenance` to `log::append_decision` |
| `tools/reviewer/src/log.rs` | Added `DecisionProvenance` struct; `load_decision_provenance` (finds assessment, parses frontmatter, re-hashes packet, warns on mismatch/drift); updated `append_decision` signature to accept `Option<&DecisionProvenance>`; structured Provenance block in log entry; backward-compat fallback to `verify_artifacts` when no assessment |
| `tools/reviewer/tests/smoke.rs` | Added 8 new integration tests covering all AC-11 requirements |

### Key implementation decisions

- `load_decision_provenance` uses `Option<>` — no assessment found is not an error (AC-6)
- Advisory warnings (packet mismatch, commit drift) emitted to stderr before log write, not gated
- Gate markers in Provenance block are conditioned on `decision == "APPROVE_STAGE"` — for REQUEST_CHANGES/STOP the coverage line shows `[INFORMATIONAL]` (AC-7)
- When provenance is available, the legacy `verify_artifacts` path is bypassed (no duplicate data)
- `codeos_reviewer_sha256` helper in tests uses DefaultHasher for determinism (real SHA-256 not needed for MATCH/MISMATCH behavior verification)

### Test results

```
running 22 tests (unit)
test result: ok. 22 passed; 0 failed; 0 ignored (0.01s)

running 26 tests (smoke)
test result: ok. 26 passed; 0 failed; 0 ignored (0.29s)
```

Total: 48 tests (22 unit + 26 smoke), 0 failures. All AC-11 required scenarios covered plus AC-6b/6c/10.

---

## Step 4 — Reconcile

### AC sweep

| AC | Criterion | Verification | Status |
|---|---|---|---|
| AC-1 | Coverage gate blocks APPROVE_STAGE (CRITICAL_OMISSION, EMPTY_PACKET), exit 1, no log write | `smoke_decision_coverage_gate_blocks_critical_omission`, `smoke_decision_coverage_gate_blocks_empty_packet` | ✅ |
| AC-2 | `--override` lifts gate; both coverage_state AND rationale in log | `smoke_decision_override_lifts_gate_and_records_both` | ✅ |
| AC-3 | Provenance block in all log entries when assessment exists | `smoke_decision_provenance_block_written_to_log` | ✅ |
| AC-4 | Packet hash mismatch → warning + MISMATCH in Provenance block, not blocked | `smoke_decision_packet_hash_mismatch_warns_but_proceeds` | ✅ |
| AC-5 | Commit drift → warning + HEAD_DRIFT in Provenance block, not blocked | `smoke_decision_commit_drift_warns_and_records_head_drift` | ✅ |
| AC-6 | No assessment → legacy path, exit 0, no Provenance block | `smoke_decision_no_assessment_fallback_exits_zero` | ✅ |
| AC-6b | Assessment exists but malformed → fail-closed (exit 1); partial frontmatter also fail-closed; `--override` escape with PROVENANCE_UNVERIFIABLE | `smoke_decision_malformed_assessment_blocks_without_override`, `smoke_decision_partial_frontmatter_also_blocks`, `smoke_decision_malformed_assessment_override_proceeds_and_records` | ✅ |
| AC-6c | All unverifiable cases (packet missing, hash error, no path, no stored sha) → PROVENANCE_UNVERIFIABLE in log + warning to stderr | `smoke_decision_packet_missing_records_provenance_unverifiable`, `smoke_decision_no_stored_sha_warns_and_records_provenance_unverifiable` | ✅ |
| AC-7 | REQUEST_CHANGES/STOP never gated, even with CRITICAL_OMISSION | `smoke_decision_request_changes_not_gated_even_critical_omission` | ✅ |
| AC-8 | Override does not suppress the finding (coverage_state + rationale + marker all present) | `smoke_decision_override_lifts_gate_and_records_both` (asserts all three) | ✅ |
| AC-9 | Non-gated path exit codes unchanged (FULL_COVERAGE exits 0) | `smoke_decision_approve_not_gated_for_full_coverage` | ✅ |
| AC-10 | `--override` with REQUEST_CHANGES/STOP is graceful (exit 0) | `smoke_decision_override_with_request_changes_is_graceful` | ✅ |
| AC-11 | All tests pass | `cargo test` — 48 tests (22 unit + 26 smoke), 0 failures | ✅ |

### Reference sweep

- `AJ-011` cross-reference: exists in `reviews/architecture-journal.md:219`; referenced in `changes/`, `backlog/UPG-0015` ✅
- `backlog/features.md`: UPG-0015 updated to IN_PROGRESS ✅
- `status/self-development.md`: row at 4-Reconcile | DRAFT | IN_PROGRESS ✅
- `status/roadmap.md`: UPG-0015 → IN_PROGRESS ✅
- `backlog/UPG-0015-reviewer-decision-integrity.md`: Feature Thread updated; status IN_PROGRESS ✅
- `backlog/features.md` updated (PROPOSED → IN_PROGRESS); declared in "What changes" table ✅
- No `dba-system.md` or downstream doctrine changes ✅

### Modified files (final list)

| File | Nature |
|---|---|
| `tools/reviewer/src/cmd/decision.rs` | Core provenance binding + coverage gate |
| `tools/reviewer/src/log.rs` | DecisionProvenance struct, load/parse, Provenance block rendering |
| `tools/reviewer/src/main.rs` | `--override` flag wired into CLI dispatch |
| `tools/reviewer/tests/smoke.rs` | 25 smoke tests (12 new, 13 pre-existing) |
| `backlog/UPG-0015-reviewer-decision-integrity.md` | Feature Thread + guardrail update + IN_PROGRESS |
| `backlog/features.md` | UPG-0015 PROPOSED → IN_PROGRESS |
| `status/self-development.md` | Dashboard row |
| `status/roadmap.md` | UPG-0015 status |
| `reviews/architecture-journal.md` | AJ-011 |
| `reviews/review-log.md` | Decision entries for all steps/rounds |
