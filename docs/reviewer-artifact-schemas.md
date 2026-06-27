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
| `provenance_integrity` | `COMMIT_BOUND` \| `WORKSPACE_BOUND` \| `REDACTED_BOUND` \| `PARTIAL_BOUND` \| `UNBOUND` |
| artifact `visibility` | `shown` \| `shown_redacted` \| `oversize_omitted` \| `missing` |
| booleans | `true` \| `false` |

Concern severity order (used to compute the effective concern floor):
`NO OBJECTION` < `CHANGES ADVISED` < `UNCLASSIFIED` < `DO NOT ADVANCE`.

`coverage_state` is a single value — most severe wins, in the order
`EMPTY_PACKET` > `CRITICAL_OMISSION` > `SECRET_REDACTION` > `PARTIAL_COVERAGE` > `FULL_COVERAGE`.

## THE AUTHORITATIVE MATRIX

Every review resolves to exactly one row. `effective_concern` = the more severe of the Codex
concern and the row's **minimum**. A self-contradictory state (base == review SHA, non-empty
diff, clean tree) is treated as `UNBOUND` (row 7 semantics).

| # | Input condition | Packet rendering | `coverage_state` | `provenance_integrity` | Min `effective_concern` | Approval eligibility | Rollback meaning |
|---|---|---|---|---|---|---|---|
| 1 | All requested artifacts shown fully; no redaction; clean workspace; `review_commit` contains the reviewed content | every artifact `shown`; full diff | `FULL_COVERAGE` | `COMMIT_BOUND` | Codex concern as emitted | **Eligible** | exact git commit (`review_commit`) |
| 2 | Same as 1 but the reviewed artifact/workspace is **uncommitted**; exact artifact text, artifact SHA, diff hash, packet SHA, `workspace_dirty` all saved | every artifact `shown`; full diff | `FULL_COVERAGE` | `WORKSPACE_BOUND` | Codex concern as emitted | **Eligible** *iff* the decision command re-verifies artifact SHA + diff hash + packet SHA + `workspace_dirty` still match | NOT exact until a stage commit is made; the log must say *workspace-bound* |
| 3 | A requested artifact exists; a secret-like value is redacted in place; artifact record preserved | artifact `shown_redacted` | `SECRET_REDACTION` | `REDACTED_BOUND` | `CHANGES ADVISED` | Requires an explicit human **security waiver** (`--force "<reason>"`) | depends on commit/workspace mode |
| 4 | A requested artifact path does not exist or cannot be read | artifact `missing` (no contents) | `CRITICAL_OMISSION` | `UNBOUND` | `DO NOT ADVANCE` | **Not eligible** | none (reviewer never saw it) |
| 5 | A requested artifact exceeds the packet size threshold | artifact `oversize_omitted` (no contents) | `CRITICAL_OMISSION` | `UNBOUND` (unless separately hashed and manually reviewed) | `DO NOT ADVANCE` | **Not eligible** without `--force` + reason | none until separately captured |
| 6 | All requested artifacts shown, but some **non-requested / supplemental** diff path was excluded | artifacts shown; diff partial | `PARTIAL_COVERAGE` | `PARTIAL_BOUND` | `CHANGES ADVISED` | Eligible only with a conscious **coverage waiver** (`--force`) if the excluded path is relevant | per commit/workspace mode |
| 7 | No requested artifact content shown and no useful diff | empty | `EMPTY_PACKET` | `UNBOUND` | `UNCLASSIFIED` | **Not eligible** | none |

**Path exclusion vs requested artifacts.** Secret *path* rules (`.env*`, `*.pem`, `secrets/*`,
size limit, …) apply to **non-requested diff paths and incidental files** only. A *requested*
artifact is always represented in the artifact section — `shown`, `shown_redacted`,
`oversize_omitted`, or `missing` — never silently dropped.

**Two provenance modes for approval.** `COMMIT_BOUND` means the reviewed state is a clean Git
commit (rollback is exact). `WORKSPACE_BOUND` means the reviewed artifact/workspace was
uncommitted; it is approvable, but only if the decision command re-verifies the saved artifact
SHA, diff hash, packet SHA, and `workspace_dirty` state still match — and the decision log must
state that rollback is **not exact until a stage commit is made**.

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
Decision: <APPROVE_STAGE|REQUEST_CHANGES|STOP>[ [<STALE|WORKSPACE|SECURITY WAIVER|COVERAGE WAIVER|UNBOUND> OVERRIDE — ...]]
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
