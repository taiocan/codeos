# Reviewer Pipeline — Artifact Schemas (v0)

Normative v0 shapes for the on-disk artifacts produced by `scripts/codeos-review.sh`
(see `docs/reviewer-pipeline.md` for the design). Companion to the pilot; intentionally minimal.

## Schema authority

- These v0 schemas define the **intended on-disk shape** for the manual reviewer pilot. They
  are the reference the script writes to and the lightweight validator checks against.
- **Full machine-readable JSON Schema validation is deferred** unless it proves needed after
  pilot use. If richer/formal schemas are required, that is tracked separately, not in the
  pilot (see the stopping rule in the originating work order).
- When a field's value is one of a fixed set, the **Enums** below are authoritative.

## Enums

| Field | Allowed values |
|---|---|
| `concern` (Codex) | `NO OBJECTION` \| `CHANGES ADVISED` \| `DO NOT ADVANCE` \| `UNCLASSIFIED` |
| `effective_concern` | `NO OBJECTION` \| `CHANGES ADVISED` \| `DO NOT ADVANCE` \| `UNCLASSIFIED` |
| `evidence` | `A` \| `B` \| `C` \| `D` \| `E` \| `not reported` |
| `human decision` | `APPROVE STAGE` \| `REQUEST CHANGES` \| `STOP` (CLI tokens: `APPROVE_STAGE` \| `REQUEST_CHANGES` \| `STOP`) |
| `coverage_state` | `FULL_COVERAGE` \| `PARTIAL_COVERAGE` \| `SECRET_REDACTION` \| `CRITICAL_OMISSION` \| `EMPTY_PACKET` |
| `provenance_integrity` | `OK` \| `CONTRADICTION` |
| booleans | `true` \| `false` |

**Coverage-state precedence (single value, most severe wins).** A review may have several
degradations at once; exactly one `coverage_state` is recorded, chosen in this order:
`EMPTY_PACKET` > `CRITICAL_OMISSION` > `SECRET_REDACTION` > `PARTIAL_COVERAGE` >
`FULL_COVERAGE`. `effective_concern` is then derived from `codex_concern`:
`EMPTY_PACKET`/`CRITICAL_OMISSION` force `DO NOT ADVANCE`; `SECRET_REDACTION`/`PARTIAL_COVERAGE`
downgrade only `NO OBJECTION` → `CHANGES ADVISED`; a `CONTRADICTION` provenance integrity
overrides all of these to `DO NOT ADVANCE`.

## 1. Review packet (text sent to Codex; canonical copy under `reviews/codex/packets/`)

Required structure — a `Critically assess:` line, then:

- **REVIEW CONTEXT** — required fields: `Feature`, `Stage`, `Branch`, `Base commit`,
  `Review commit` (the packet text may append a human-readable `(+ uncommitted workspace
  changes)` marker for the reviewer; the persisted `review_commit` field is the pure SHA),
  `Current approved stage`, `Evidence coverage` (= `coverage_state`), `Provenance integrity`.
- **DBA RULES RELEVANT TO THIS STAGE** — present.
- **STAGE-SPECIFIC CHECKS** — present (may be the generic line for non-1–9 stages).
- **EXPECTED STAGE OUTPUT** — present.
- **ARTIFACTS TO REVIEW** — each requested artifact has exactly one **visibility**:
  `SHOWN` (full contents) or `SHOWN_REDACTED` (contents with secret values redacted in place) —
  both render as `--- <path> (sha256: <hex>) ---` + indented contents; or `MISSING`; or
  `EXCLUDED_SIZE` (over the size limit) — both render as an exclusion marker with no contents.
  Requested artifacts are **never** dropped by the secret *path* rules (those apply to the diff
  only); a secret inside a requested artifact yields `SHOWN_REDACTED`, not exclusion. Only
  `MISSING` / `EXCLUDED_SIZE` force `CRITICAL_OMISSION`; `SHOWN_REDACTED` contributes to
  `SECRET_REDACTION`.
- **DIFF TO REVIEW** — the secret/size-filtered diff (may be empty); an exclusion/redaction
  note when any path/hunk was withheld.
- **INSTRUCTIONS** — requests the `LOG SUMMARY:` and optional `EVIDENCE:` trailing lines.

The persisted packet file is the **canonical reviewed bytes**; its SHA256 is recorded in the
assessment header and the REVIEW log entry.

## 2. Saved Codex assessment — YAML metadata header

File: `reviews/codex/<ts>-<feature>-stage-<N>-<sha>.md`. Opens with a `---` YAML block whose
**required keys** are:

| Key | Type | Notes |
|---|---|---|
| `feature` | string | |
| `stage` | integer | |
| `branch` | string | |
| `base_commit` | string | git SHA, or `(uncommitted artifact)` |
| `review_commit` | string | git SHA — **machine-pure** (no suffix); the dirty bit is `workspace_dirty` |
| `artifacts` | list of `{path, sha256}` | one entry per **shown** artifact; may be empty only for `CRITICAL_OMISSION`/`EMPTY_PACKET` |
| `diff_hash` | string (sha256) | |
| `coverage_state` | enum | |
| `provenance_integrity` | enum | |
| `workspace_dirty` | bool | |
| `redaction_count` | integer | |
| `secret_redaction` | bool | |
| `excluded_paths` | string | space-separated; may be `""` |
| `reviewed_packet` | string | `packets/<file>.packet.txt` |
| `reviewed_packet_sha256` | string (sha256) | |
| `reviewer` | string | e.g. `codex (session <uuid>)` |
| `codex_concern` | enum | pure enum value |
| `effective_concern` | enum | pure enum value (validated against the enum) |
| `effective_concern_note` | string | optional; present only when a downgrade/override applied — the coverage/integrity explanation. Never part of `effective_concern` itself |
| `evidence` | enum | |

Body (after the closing `---`): the full Codex assessment text, verbatim.

## 3. Feature-scoped Codex session state — JSON

File: `.codeos-state/codex-sessions/<feature>.json` (runtime state, gitignored). Required keys:

| Key | Type |
|---|---|
| `feature` | string |
| `session_id` | string (UUID) |
| `codex_version` | string |
| `created_at` | string (ISO 8601 UTC) |

A session file that exists but lacks `session_id` is **malformed** → fail-closed.

## 4. REVIEW entry — `reviews/review-log.md` (append-only)

Required lines:

```
## <ISO ts> REVIEW — <feature> — Stage <N>
Base: <sha|(uncommitted artifact)>  Review: <sha>  Branch: <branch>
Diff-hash: <sha256>
Reviewer: codex <model> (session <uuid>)
Codex concern: <enum>
Effective concern: <enum>
Evidence: <enum>
Coverage: <coverage_state> (redactions: <int>); integrity: <enum>; workspace_dirty: <bool>[; note: <text>]
Log summary: <text>
Full assessment: <path> (sha256:<hex>)
Reviewed packet: <path> (sha256:<hex>)
[Coverage gap: <coverage_state> — excluded/redacted [...] — MANUAL SECURITY REVIEW REQUIRED]   # when applicable
Human decision: (append with: codeos-review.sh decision ...)
```

## 5. HUMAN DECISION entry — `reviews/review-log.md` (append-only)

Required lines:

```
## <ISO ts> HUMAN DECISION — <feature> — Stage <N>
Commit reviewed: <sha>
Decision: <APPROVE_STAGE|REQUEST_CHANGES|STOP>[ [STALE OVERRIDE — ...] | [DIRTY OVERRIDE — ...]]
Reason/next: <text>
[Verified against: <assessment path>]            # when a prior assessment exists
[Artifact integrity:                              # when verified
  MATCH   <path>
  CHANGED <path> (reviewed <hex> / now <hex>)]
```

Entries are **append-only**; a decision is never written into a prior entry.

## Lightweight validation (v0)

Before writing the assessment and REVIEW entry, the script **fails closed** (exit 4) if:

- a required scalar metadata field is empty (`feature`, `stage`, `base_commit`,
  `review_commit`, `diff_hash`, `coverage_state`, `reviewed_packet_sha256`);
- an enum value (`coverage_state`, `codex_concern`, `effective_concern`, `evidence`) is
  off-list;
- the packet SHA256 is missing, or a **shown** artifact has no SHA256.

This is a required-field/enum guard only — not full structural JSON Schema validation, which is
deferred per the schema-authority note above.
