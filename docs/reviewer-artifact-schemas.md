# Reviewer Pipeline — Artifact Schemas + Provenance/Coverage Matrix (v0)

Normative v0 shapes for the on-disk artifacts produced by `scripts/codeos-review.sh`, and the
**single authoritative matrix** that governs coverage, provenance, the effective concern, and
approval eligibility. `docs/reviewer-pipeline.md` is descriptive and refers here; where the two
disagree, **this document wins**.

## Schema authority

- These v0 schemas define the **intended on-disk shape** for the manual reviewer pilot, and the
  matrix below is the authoritative rule set the script implements.
- **Full machine-readable JSON Schema validation is deferred** unless pilot use shows it is
  needed; that and parser hardening / CI validation are tracked as backlog, not in this pilot.
- The **Enums** and the **matrix** are authoritative; prose elsewhere must not restate
  conflicting rules.

## Enums

| Field | Allowed values |
|---|---|
| `codex_concern` | `NO OBJECTION` \| `CHANGES ADVISED` \| `DO NOT ADVANCE` \| `UNCLASSIFIED` |
| `effective_concern` | `NO OBJECTION` \| `CHANGES ADVISED` \| `DO NOT ADVANCE` \| `UNCLASSIFIED` |
| `evidence` | `A` \| `B` \| `C` \| `D` \| `E` \| `not reported` |
| `human decision` | `APPROVE STAGE` \| `REQUEST CHANGES` \| `STOP` (CLI tokens: `APPROVE_STAGE` \| `REQUEST_CHANGES` \| `STOP`) |
| `coverage_state` | `FULL_COVERAGE` \| `PARTIAL_COVERAGE` \| `SECRET_REDACTION` \| `CRITICAL_OMISSION` \| `EMPTY_PACKET` |
| `provenance_integrity` | `COMMIT_BOUND` \| `WORKSPACE_BOUND` \| `UNBOUND` |
| artifact `visibility` | `shown` \| `shown_redacted` \| `oversize_omitted` \| `missing` |
| booleans | `true` \| `false` |

Concern severity order (used for the effective-concern floor):
`NO OBJECTION` < `CHANGES ADVISED` < `UNCLASSIFIED` < `DO NOT ADVANCE`.

`UNCLASSIFIED` is a **first-class** verdict: the reviewer may emit it in `LOG SUMMARY` to mean
"I cannot classify this safely", and the pipeline also assigns it to malformed/unparseable
output.

## THE AUTHORITATIVE MATRIX

**Two orthogonal axes.** `provenance_integrity` is the *binding mode* (is the reviewed state a
durable, re-verifiable thing?). `coverage_state` is *how complete the evidence was*. They are
recorded independently and combine to decide approval eligibility.

### Axis A — `coverage_state` → packet rendering, effective-concern floor, waiver

`coverage_state` is a single value (most severe wins):
`EMPTY_PACKET` > `CRITICAL_OMISSION` > `SECRET_REDACTION` > `PARTIAL_COVERAGE` > `FULL_COVERAGE`.
`effective_concern` = the more severe of the Codex concern and the **floor** below.

| `coverage_state` | Condition / packet rendering | Min `effective_concern` (floor) | Eligibility effect |
|---|---|---|---|
| `FULL_COVERAGE` | every requested artifact `shown`; nothing redacted/excluded | none (Codex concern as emitted) | none |
| `SECRET_REDACTION` | a requested artifact `shown_redacted` (secret value blanked in place) | `CHANGES ADVISED` | requires a **security waiver** (`--force`) |
| `PARTIAL_COVERAGE` | a non-requested/supplemental diff path was path/size-excluded | `CHANGES ADVISED` | requires a **coverage waiver** (`--force`) |
| `CRITICAL_OMISSION` | a requested artifact is `missing` or `oversize_omitted` (not shown) | `DO NOT ADVANCE` | **HARD STOP** (see Axis B) |
| `EMPTY_PACKET` | no requested artifact content and no useful diff | `UNCLASSIFIED` | **HARD STOP** |

Secret *path* rules (`.env*`, `*.pem`, `secrets/*`, size limit, …) apply to **non-requested
diff paths and incidental files only**. A *requested* artifact is never silently dropped — it is
`shown`, `shown_redacted`, `oversize_omitted`, or `missing`.

### Axis B — `provenance_integrity` → reverification, rollback, eligibility

| `provenance_integrity` | When | Decision-time reverification | Rollback meaning | Approvable? |
|---|---|---|---|---|
| `COMMIT_BOUND` | coverage not critical/empty; clean workspace; `review_commit` holds the reviewed content | `HEAD == review_commit`, tree clean, artifacts hash-match | exact git commit (`review_commit`) | yes (subject to Axis-A waiver) |
| `WORKSPACE_BOUND` | coverage not critical/empty; reviewed content uncommitted; artifact text + SHA + diff hash + packet SHA + `workspace_dirty` saved | re-verify artifact SHA **and** diff hash **and** packet SHA **and** `workspace_dirty` still match | NOT exact until a stage commit is made (logged as workspace-bound) | yes (subject to Axis-A waiver) |
| `UNBOUND` | `CRITICAL_OMISSION` / `EMPTY_PACKET`, or a self-contradictory state (base == review SHA, non-empty diff, clean tree) | n/a | none | **no — HARD STOP, not `--force`'able** |

### Approval-eligibility rule (single source of truth)

`APPROVE_STAGE` is appended **iff**:
1. `provenance_integrity != UNBOUND` and `coverage_state ∉ {CRITICAL_OMISSION, EMPTY_PACKET}` —
   otherwise it is a **hard stop** that **`--force` cannot override** (approval must trace to
   evidence the reviewer actually saw); **and**
2. the Axis-B reverification for the binding mode passes (or `--force` records a
   `[STALE OVERRIDE]` / `[WORKSPACE OVERRIDE]`); **and**
3. any Axis-A waiver is supplied — `SECRET_REDACTION` needs `--force` (`[SECURITY WAIVER]`),
   `PARTIAL_COVERAGE` needs `--force` (`[COVERAGE WAIVER]`).

`REQUEST_CHANGES` / `STOP` are always recorded.

---

## 1. Review packet (text sent to Codex; canonical copy under `reviews/codex/packets/`)

`Critically assess:` then: **REVIEW CONTEXT** (`Feature`, `Stage`, `Branch`, `Base commit`,
`Review commit` — the packet text may append a human-readable `(+ uncommitted workspace
changes)` marker; the persisted `review_commit` field is the pure SHA — `Current approved
stage`, `Evidence coverage` = `coverage_state`, `Provenance integrity`); **DBA RULES**;
**STAGE-SPECIFIC CHECKS**; **EXPECTED STAGE OUTPUT**; **ARTIFACTS TO REVIEW** (each requested
artifact rendered per its `visibility`); **DIFF TO REVIEW** (secret/size-filtered; withholding
noted); **INSTRUCTIONS** (requests `LOG SUMMARY:` + optional `EVIDENCE:`). The persisted packet
file is the **canonical reviewed bytes**; its SHA256 is recorded in the assessment and log.

## 2. Saved Codex assessment — YAML metadata header

File `reviews/codex/<ts>-<feature>-stage-<N>-<sha>.md`, opening with a `---` YAML block; required keys:

| Key | Type | Notes |
|---|---|---|
| `feature` | string | |
| `stage` | integer | |
| `branch` | string | |
| `base_commit` | string | git SHA, or `(no base pin)` when no stage-start was recorded |
| `review_commit` | string | git SHA — **machine-pure** (no suffix); dirty bit is `workspace_dirty` |
| `artifacts` | list of `{path, visibility[, sha256]}` | one entry per **requested** artifact; `sha256` present for `shown`/`shown_redacted`; may be `[]` only for `EMPTY_PACKET` |
| `diff_hash` | string (sha256) | |
| `coverage_state` | enum | |
| `provenance_integrity` | enum | |
| `workspace_dirty` | bool | |
| `redaction_count` | integer | |
| `secret_redaction` | bool | |
| `excluded_paths` | list of `{path, reason, affected_section}` | `[]` when none; `affected_section` is `diff` or `artifact` |
| `reviewed_packet` | string | `packets/<file>.packet.txt` |
| `reviewed_packet_sha256` | string (sha256) | |
| `reviewer` | string | e.g. `codex (session <uuid>)` |
| `codex_concern` | enum | pure enum value |
| `effective_concern` | enum | pure enum value (validated) |
| `effective_concern_note` | string | optional; present only when the floor/override changed the value |
| `evidence` | enum | |

Body (after the closing `---`): the full Codex assessment text, verbatim.

## 3. Feature-scoped Codex session state — JSON

File `.codeos-state/codex-sessions/<feature>.json` (gitignored). Required: `feature`,
`session_id` (UUID), `codex_version`, `created_at` (ISO 8601 UTC). A session file that exists
but lacks `session_id` is **malformed → fail-closed**.

## 4. REVIEW entry — `reviews/review-log.md` (append-only)

```
## <ISO ts> REVIEW — <feature> — Stage <N>
Base: <sha|(no base pin)>  Review: <sha>  Branch: <branch>
Diff-hash: <sha256>
Reviewer: codex <model> (session <uuid>)
Codex concern: <enum>
Effective concern: <enum>
Evidence: <enum>
Coverage: <coverage_state>; provenance: <provenance_integrity>; redactions: <int>; workspace_dirty: <bool>[; note: <text>]
[Rollback: NOT exact until a stage commit is made (workspace-bound review)]   # WORKSPACE_BOUND only
Log summary: <text>
Full assessment: <path> (sha256:<hex>)
Reviewed packet: <path> (sha256:<hex>)
[Coverage gap: <coverage_state> — ... — MANUAL SECURITY REVIEW REQUIRED]   # when applicable
Human decision: (append with: codeos-review.sh decision ...)
```

## 5. HUMAN DECISION entry — `reviews/review-log.md` (append-only)

```
## <ISO ts> HUMAN DECISION — <feature> — Stage <N>
Commit reviewed: <sha>
Decision: <APPROVE_STAGE|REQUEST_CHANGES|STOP>[ [<STALE OVERRIDE|WORKSPACE OVERRIDE|SECURITY WAIVER|COVERAGE WAIVER|UNREVIEWED OVERRIDE> — ...]]
Reason/next: <text>
[Verified against: <assessment path>]
[Rollback: exact git commit <sha> | NOT exact until a stage commit is made (workspace-bound review)]
[Artifact integrity:
  MATCH   <path>
  CHANGED <path> (reviewed <hex> / now <hex>)]
```

Entries are **append-only**; a decision is never written into a prior entry. `APPROVE_STAGE`
eligibility follows the matrix's *Approval eligibility* column; an ineligible mode is refused
(nothing logged) unless `--force "<reason>"` records the named override/waiver.

## Lightweight validation (v0)

Before writing the assessment + REVIEW entry, the script **fails closed** (exit 4) if: a
required scalar is empty (`feature`, `stage`, `base_commit`, `review_commit`, `diff_hash`,
`coverage_state`, `provenance_integrity`, `reviewed_packet_sha256`); an enum value
(`coverage_state`, `provenance_integrity`, `codex_concern`, `effective_concern`, `evidence`) is
off-list; or the packet SHA256 / a shown artifact's record is missing. This is a
required-field/enum guard only — full structural JSON Schema validation is deferred.
