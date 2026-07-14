# Reviewer Pipeline — Artifact Schemas + Coverage Rules (v0)

Descriptive v0 shapes for the on-disk artifacts produced by `scripts/codeos-review.sh`, plus the
**coverage rules** that set the effective concern. `docs/reviewer-pipeline.md` is the prose design
and refers here for exact shapes.

> **v0 is a manual advisory review *logging* pilot.** These schemas describe what the pilot writes
> so the evidence is self-describing and auditable. They are **not** an approval-integrity contract:
> the hashes, packet copy, and `workspace_dirty` flag are **audit aids**, not a guarantee that a
> human approval is bound to an unchanged repository state. Binding approval to a reproducible
> reviewed state (commit + diff hash + workspace snapshot, decision-time reverification, rollback)
> is **deferred** — see `backlog/UPG-0015-reviewer-decision-integrity.md`.

## Schema authority

- These v0 schemas define the **intended on-disk shape** for the manual reviewer pilot, and the
  coverage rules below are what the script implements.
- **Full machine-readable JSON Schema validation is deferred** unless pilot use shows it is needed;
  that and parser hardening / CI validation are tracked as backlog, not in this pilot.
- The **Enums** and the **coverage rules** are authoritative for this pilot; prose elsewhere must
  not restate conflicting rules.

## Enums

| Field | Allowed values |
|---|---|
| `codex_concern` | `NO OBJECTION` \| `CHANGES ADVISED` \| `DO NOT ADVANCE` \| `UNCLASSIFIED` |
| `effective_concern` | `NO OBJECTION` \| `CHANGES ADVISED` \| `DO NOT ADVANCE` \| `UNCLASSIFIED` |
| `evidence` | `A` \| `B` \| `C` \| `D` \| `E` \| `not reported` |
| `human decision` | `APPROVE STAGE` \| `REQUEST CHANGES` \| `STOP` (CLI tokens: `APPROVE_STAGE` \| `REQUEST_CHANGES` \| `STOP`) |
| `coverage_state` | `FULL_COVERAGE` \| `PARTIAL_COVERAGE` \| `SECRET_REDACTION` \| `CRITICAL_OMISSION` \| `EMPTY_PACKET` |
| artifact `visibility` | `shown` \| `shown_redacted` \| `oversize_omitted` \| `missing` |
| booleans | `true` \| `false` |

Concern severity order (used for the effective-concern floor):
`NO OBJECTION` < `CHANGES ADVISED` < `UNCLASSIFIED` < `DO NOT ADVANCE`.

`UNCLASSIFIED` is a **first-class** verdict: the reviewer may emit it in `LOG SUMMARY` to mean
"I cannot classify this safely", and the pipeline also assigns it to malformed/unparseable output.

## Coverage state → effective-concern floor

`coverage_state` records **how complete the evidence was** that reached the reviewer. It is a single
value (most severe wins):
`EMPTY_PACKET` > `CRITICAL_OMISSION` > `SECRET_REDACTION` > `PARTIAL_COVERAGE` > `FULL_COVERAGE`.

`effective_concern` = the more severe of the Codex concern and the **floor** below. Both the raw
Codex concern and the effective concern are recorded, so an evidence gap never silently passes as
the verdict. This is the only adjustment the pilot makes; it is about evidence completeness, **not**
approval eligibility.

| `coverage_state` | Condition / packet rendering | Min `effective_concern` (floor) |
|---|---|---|
| `FULL_COVERAGE` | every requested artifact `shown`; nothing redacted/excluded | none (Codex concern as emitted) |
| `PARTIAL_COVERAGE` | a non-requested/supplemental diff path was path/size-excluded | `CHANGES ADVISED` |
| `SECRET_REDACTION` | a requested artifact `shown_redacted` (secret value blanked in place) | `CHANGES ADVISED` |
| `CRITICAL_OMISSION` | a requested artifact is `missing` or `oversize_omitted` (not shown) | `DO NOT ADVANCE` |
| `EMPTY_PACKET` | no requested artifact content and no useful diff | `UNCLASSIFIED` |

Secret *path* rules (`.env*`, `*.pem`, `secrets/*`, size limit, …) apply to **non-requested diff
paths and incidental files only**. A *requested* artifact is never silently dropped — it is `shown`,
`shown_redacted`, `oversize_omitted`, or `missing`. When anything is excluded/redacted, or on a
critical/empty state, the REVIEW log flags **MANUAL SECURITY REVIEW REQUIRED**.

`workspace_dirty` (whether the tree had uncommitted changes at review time) is recorded as plain
**descriptive audit context**. It is not a binding mode and does not gate any decision.

---

## 1. Review packet (text sent to Codex; canonical copy under `reviews/codex/packets/`)

`Critically assess:` then: the **Scope Contract + Triage Rule** (so the reviewer classifies findings
by scope — see `prompts/reviewer-automated.md`); **REVIEW CONTEXT** (`Feature`, `Stage`, `Branch`,
`Base commit`, `Review commit` — the packet text may append a human-readable `(+ uncommitted
workspace changes)` marker; the persisted `review_commit` field is the pure SHA — `Current approved
stage`, `Evidence coverage` = `coverage_state`, `Workspace dirty`); **DBA RULES**; **STAGE-SPECIFIC
CHECKS**; **EXPECTED STAGE OUTPUT**; **ARTIFACTS TO REVIEW** (each requested artifact rendered per
its `visibility`); **DIFF TO REVIEW** (secret/size-filtered; withholding noted); **INSTRUCTIONS**
(requests the triage output + `LOG SUMMARY:` + optional `EVIDENCE:`). The persisted packet file is
the canonical reviewed bytes; its SHA256 is recorded in the assessment and log as an audit aid.

## 2. Saved Codex assessment — YAML metadata header

File `reviews/codex/<ts>-<feature>-stage-<N>-<sha>.md`, opening with a `---` YAML block; required keys:

| Key | Type | Notes |
|---|---|---|
| `review_id` | string | `REV__<feature>__<stage>__R<N>` — mechanically derived (`UPG-0046`) by counting prior `REVIEW` entries for this exact feature+stage in `reviews/review-log.md`; `<stage>` is the raw `--stage` value verbatim, no `S<N>` conversion. First frontmatter key. Not yet part of the Lightweight validation (v0) required-field list below. |
| `findings` | list of `{finding_id, severity, classification, summary, acceptance_criterion?, required_action}` | mechanically parsed (`UPG-0047`) from the raw response's `Finding:`/`Evidence:`/`Why:`/`Required action:` blocks (before the response body, verbatim, below the frontmatter). `finding_id` is `FND__<review_id>__NN` (2-digit, deterministic). `classification` is one of the five TRIAGE RULE labels verbatim. **Deliberately excludes `evidence`/`why`/`scope_reason`** — that full prose stays only in the body, not duplicated into frontmatter. `[]` when the round raised no findings. No `status`/`resolved_by` field — an assessment is an immutable snapshot; resolution is derived later from an accepted change record's `fixes_findings` list naming this `finding_id`, never by editing this file. |
| `unparsed_findings_count` | integer | count of `Finding:` lines that did not match any of the three currently-supported `Evidence:`/`Why:`/`Required action:` shapes — never silently dropped, always counted. `0` in the overwhelming common case; a nonzero value is advisory only and never blocks the review. |
| `feature` | string | |
| `stage` | integer | |
| `branch` | string | |
| `base_commit` | string | git SHA, or `(no base pin)` when no stage-start was recorded |
| `review_commit` | string | git SHA — **machine-pure** (no suffix); dirty bit is `workspace_dirty` |
| `artifacts` | list of `{path, visibility[, sha256]}` | one entry per **requested** artifact; `sha256` present for `shown`/`shown_redacted`; may be `[]` only for `EMPTY_PACKET` |
| `diff_hash` | string (sha256) | |
| `coverage_state` | enum | |
| `workspace_dirty` | bool | descriptive only |
| `redaction_count` | integer | |
| `secret_redaction` | bool | |
| `excluded_paths` | list of `{path, reason, affected_section}` | `[]` when none; `affected_section` is `diff` or `artifact` |
| `reviewed_packet` | string | `packets/<file>.packet.txt` |
| `reviewed_packet_sha256` | string (sha256) | |
| `reviewer` | string | e.g. `codex (session <uuid>)` |
| `codex_concern` | enum | pure enum value |
| `effective_concern` | enum | pure enum value (validated) |
| `effective_concern_note` | string | optional; present only when the coverage floor changed the value |
| `evidence` | enum | |

Body (after the closing `---`): the full Codex assessment text, verbatim.

## 3. Feature-scoped Codex session state — JSON

File `.codeos-state/codex-sessions/<feature>.json` (gitignored). Required: `feature`,
`session_id` (UUID), `codex_version`, `created_at` (ISO 8601 UTC). A session file that exists
but lacks `session_id` is **malformed → fail-closed**.

## 4. REVIEW entry — `reviews/review-log.md` (append-only)

```
## <ISO ts> REVIEW — <feature> — Stage <N>
Review ID: <review_id>
Base: <sha|(no base pin)>  Review: <sha>  Branch: <branch>
Diff-hash: <sha256>
Reviewer: codex <model> (session <uuid>)
Codex concern: <enum>
Effective concern: <enum>
Evidence: <enum>
Coverage: <coverage_state>; redactions: <int>; workspace_dirty: <bool>[; note: <text>]
Log summary: <text>
Full assessment: <path> (sha256:<hex>)
Reviewed packet: <path> (sha256:<hex>)
[Coverage gap: <coverage_state> — ... — MANUAL SECURITY REVIEW REQUIRED]   # when applicable
Human decision: (append with: codeos-review.sh decision ...)
```

## 5. HUMAN DECISION entry — `reviews/review-log.md` (append-only)

```
## <ISO ts> HUMAN DECISION — <feature> — Stage <N>
Commit at decision: <sha>
Decision: <APPROVE_STAGE|REQUEST_CHANGES|STOP>
Reason/next: <text>
[Verified against: <assessment path>]
[Artifact integrity (informational audit, not a gate):
  MATCH   <path>
  CHANGED <path> (reviewed <hex> / now <hex>)]
```

Entries are **append-only**; a decision is never written into a prior entry. The decision command
**records** the human's choice — it does not enforce eligibility or refuse approvals. The artifact
integrity block is a **best-effort audit aid**: a `CHANGED` line is flagged (and a warning printed),
but it never blocks the decision. `APPROVE` is the human's word (Non-Negotiable Rule 1).

## Lightweight validation (v0)

Before writing the assessment + REVIEW entry, the script **fails closed** (exit 4) if: a required
scalar is empty (`feature`, `stage`, `base_commit`, `review_commit`, `diff_hash`, `coverage_state`,
`reviewed_packet_sha256`); an enum value (`coverage_state`, `codex_concern`, `effective_concern`,
`evidence`) is off-list; or the packet SHA256 / a shown artifact's record is missing. This is a
required-field/enum guard only — full structural JSON Schema validation is deferred.
