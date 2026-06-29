# Self-Development Change: UPG-0027__CHG-20260629-002 — focused-reviewer-prompt

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
change_id: CHG-20260629-002
slug: focused-reviewer-prompt
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
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round (which does not exist until after the packet is built). Reference the stable review
SERIES (review_series) + review_state; exact rounds live only in reviews/review-log.md and
reviews/codex/*. See prompts/codeos-self-dev.md → "Feature Thread & IDs" / "Self-Reference Boundary". -->


## Change Intent

**Why (problem in the toolkit):**

The reviewer prompt in `scripts/codeos-review.sh` (lines 259–278) is a generic "Critically
assess:" block with a scope contract and triage rules, but no focused task. The reviewer is not
asked to: verify acceptance criteria are satisfied, evaluate evidence quality, grade evidence,
cap blocker findings, or avoid flagging style issues as blockers. Without a focused task, Codex
acts as a general-purpose critic — flagging prose consistency, wording, and peripheral issues at
the same priority as real in-scope blockers. This drives repeated review rounds.

Additionally, the triage rule in the script has only four categories; the fifth category
(`SELF-REFERENCE / REVIEW-BOOKKEEPING`) added by UPG-0030 to the governance docs is missing
from the actual reviewer prompt injected into packets.

**What changes:**

1. `prompts/codeos-reviewer-task.md` — new file; focused five-question reviewer task definition,
   what-NOT-to-do instructions, required output shape with parser-compatible labels
   (`PR decision`, `Scope drift warning`, `LOG SUMMARY`, `EVIDENCE`), and complete
   five-category triage table.

2. `scripts/codeos-review.sh` — static "Critically assess:" + SCOPE CONTRACT + TRIAGE RULE block
   (lines 259–278) replaced by injection of `prompts/codeos-reviewer-task.md` content. Dynamic
   sections (REVIEW CONTEXT, DBA RULES, STAGE-SPECIFIC CHECKS, EXPECTED STAGE OUTPUT, ARTIFACTS,
   DIFF) remain inline. Script must fail if the template file is missing.

3. `docs/reviewer-pipeline.md` — one sentence added to the reviewer prompt section referencing
   `prompts/codeos-reviewer-task.md` as the canonical task definition.

**Step 1 pre-work (in-scope per AJ-001):**

4. `backlog/UPG-0027-replacing-review-scripts.md` — updated to new title, slug, content, and
   Feature Thread (old "Migrating Bash scripts" brief replaced with "Lean Review Runner and
   Packet Architecture" brief).

5. `backlog/features.md` — UPG-0027 row updated (new title; status IN_PROGRESS).

6. `status/self-development.md` — CHG-20260629-002 row activated.

**Scope boundary — what stays the same:**

- `dba-system.md` — untouched
- `scripts/codeos-review.sh` dynamic packet sections (REVIEW CONTEXT, DBA RULES with coverage
  conditional, STAGE-SPECIFIC CHECKS, EXPECTED STAGE OUTPUT, ARTIFACTS, DIFF) — remain inline
- Stage-specific check functions (`stage_checks`, `stage_expected`) — unchanged
- `reviews/review-log.md` format — no change (manifest is in assessment files only)
- Packet structure changes (manifest, modes, budget) — out of scope; CHG-2
- Local check gate, delta mode — out of scope; CHG-3
- Verdict tokens (`NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE`) — unchanged; output labels
  `PR decision`, `Scope drift warning`, `LOG SUMMARY`, `EVIDENCE` remain unchanged (parser-required)
- No changes to downstream projects
- `backlog/UPG-0027-replacing-review-scripts.md` physical filename retained for continuity;
  slug field in frontmatter updated to `lean-review-runner-packet-architecture`; rename via
  `git mv` is deferred to Step 3 implementation

**Class:** `prompt / script-tooling` (primary: new prompt file; supporting: script injection)
**Scope axis:** self-dev only
**Backlog item:** `backlog/UPG-0027-replacing-review-scripts.md`

---

## Acceptance Criteria

<!-- Prompt file content (A1–A4) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A1 | `prompts/codeos-reviewer-task.md` exists and contains all five reviewer task questions | Read file; verify five question headings present |
| A2 | Required output shape uses exact parser-compatible labels: per-finding `Finding:` block, then `PR decision:`, `Scope drift warning:`, and last two lines `LOG SUMMARY:` / `EVIDENCE:` | `grep "PR decision\|LOG SUMMARY\|EVIDENCE" prompts/codeos-reviewer-task.md` → all three present; labels match script lines 321–327 verbatim |
| A3 | Five-category triage table present, including `SELF-REFERENCE / REVIEW-BOOKKEEPING` as the fifth category | `grep "SELF-REFERENCE" prompts/codeos-reviewer-task.md` → match |
| A4 | What-NOT-to-do section explicitly prohibits: (a) style/wording issues as blockers, (b) re-reviewing unchanged full context in delta mode, (c) treating local-only review history as a blocker unless the artifact falsely claims durability | Read file; verify all three prohibitions present by name |

<!-- Script change properties (A5–A8) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A5 | Three static blocks — SCOPE CONTRACT, TRIAGE RULE, and INSTRUCTIONS (output shape) — are removed from `build_packet()` and replaced by template injection | `grep "SCOPE CONTRACT\|TRIAGE RULE\|INSTRUCTIONS" scripts/codeos-review.sh` → none echoed inside `build_packet`; `grep "codeos-reviewer-task" scripts/codeos-review.sh` → match |
| A6 | Script exits non-zero with error message before any `codex` invocation if `prompts/codeos-reviewer-task.md` is missing; test is reversible | `mv prompts/codeos-reviewer-task.md /tmp/chg002-task-bak.md`; run review; capture exit code and stderr; `mv /tmp/chg002-task-bak.md prompts/codeos-reviewer-task.md`; verify non-zero exit and error message with no `codex` call in output |
| A7 | Dynamic sections — REVIEW CONTEXT, DBA RULES (with coverage conditional), STAGE-SPECIFIC CHECKS, EXPECTED STAGE OUTPUT, ARTIFACTS, DIFF — remain generated inline in `build_packet()` | `grep "REVIEW CONTEXT\|STAGE-SPECIFIC CHECKS\|EXPECTED STAGE OUTPUT" scripts/codeos-review.sh` → matches inside `build_packet` |
| A8 | Changes to `scripts/codeos-review.sh` confined to `build_packet()` only; `cmd_decision`, `cmd_stage_start`, and log-parse lines (grep/sed for `LOG SUMMARY`/`EVIDENCE`, lines ~435–475) unchanged | `git diff -- scripts/codeos-review.sh` → changed hunks only inside `build_packet`; no changed lines outside that function |

<!-- Docs and bookkeeping (A9–A12) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A9 | `docs/reviewer-pipeline.md` has a reference to `prompts/codeos-reviewer-task.md` as the canonical reviewer task definition | `grep "codeos-reviewer-task" docs/reviewer-pipeline.md` → match |
| A10 | Backlog brief frontmatter `slug` is `lean-review-runner-packet-architecture`; `features.md` UPG-0027 row title is "Lean Review Runner and Packet Architecture"; physical filename retained | `grep "^slug:" backlog/UPG-0027-replacing-review-scripts.md` → match; `grep "UPG-0027" backlog/features.md` → new title |
| A11 | A packet generated after the change starts with task content from the template (five questions visible before REVIEW CONTEXT) | Generate a test packet; inspect that task questions appear before REVIEW CONTEXT section |
| A12 | `status/self-development.md` row for CHG-20260629-002 shows Class=`prompt / script-tooling`, Scope=`self-dev only`, State=`IN_PROGRESS` | `grep "CHG-20260629-002" status/self-development.md` → row matches |

<!-- Scope boundary (A13) -->

| # | Criterion | How it will be verified |
|---|---|---|
| A13 | `dba-system.md` diff is empty; no packet-manifest, local-check-gate, delta-mode, or typed-runner code present in any changed file | `git diff -- dba-system.md` → empty; `grep -r "packet_manifest\|local_check\|--mode delta\|typed.runner" scripts/ prompts/` → no match in new code |

---

## Implementation Notes

Three files created or modified:

1. **`prompts/codeos-reviewer-task.md`** (new) — plain-text reviewer task, same formatting style as existing packet sections. Sections: SCOPE CONTRACT (retained verbatim from old inline prose), YOUR TASK (five questions), TRIAGE RULE (five categories; SELF-REFERENCE / REVIEW-BOOKKEEPING added as fifth), WHAT NOT TO DO (three named prohibitions), INSTRUCTIONS (output shape with exact parser labels).

2. **`scripts/codeos-review.sh`** — two hunks in `build_packet()` only:
   - Lines 258–279 replaced: added `task_prompt` local + guard (`[[ -f ]] || exit 2`) before the `{` block; replaced 21 echo lines (SCOPE CONTRACT + TRIAGE RULE preamble) with `cat "${task_prompt}"` + `echo`.
   - Lines 311–327 removed: INSTRUCTIONS block (19 echo lines) deleted; output shape now lives in the template.
   `cmd_decision`, `cmd_stage_start`, and log-parse lines unchanged.

3. **`docs/reviewer-pipeline.md`** — §2 rewritten to reference `prompts/codeos-reviewer-task.md` as the canonical task definition and remove the now-stale "Critically assess:" description.

Backlog and status bookkeeping updated as Step 1 pre-work (already complete).

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| A1 | `prompts/codeos-reviewer-task.md` exists with five question headings | PASS | `grep -c "^\s*[1-5]\." prompts/codeos-reviewer-task.md` → 5 |
| A2 | Parser-compatible labels: `PR decision:`, `Scope drift warning:`, `LOG SUMMARY:`, `EVIDENCE:` | PASS | `grep "PR decision\|LOG SUMMARY\|EVIDENCE" prompts/codeos-reviewer-task.md` → all four present; `PR decision:` and `Scope drift warning:` in INSTRUCTIONS; `LOG SUMMARY:` and `EVIDENCE:` on last two lines |
| A3 | Five-category triage with `SELF-REFERENCE / REVIEW-BOOKKEEPING` | PASS | `grep "SELF-REFERENCE" prompts/codeos-reviewer-task.md` → match in TRIAGE RULE section |
| A4 | What-NOT-to-do: (a) style/wording, (b) delta re-review, (c) false durability | PASS | `grep "style or wording\|unchanged full context\|local-only review history" prompts/codeos-reviewer-task.md` → all three present |
| A5 | SCOPE CONTRACT, TRIAGE RULE, INSTRUCTIONS removed from `build_packet()`; template injected | PASS | `grep "SCOPE CONTRACT\|TRIAGE RULE\|echo \"INSTRUCTIONS\"" scripts/codeos-review.sh` → no echo of these labels; `grep "cat.*task_prompt" scripts/codeos-review.sh` → match at line 262 |
| A6 | Guard exits non-zero before codex if template missing; reversible test | PASS | Line 259: `[[ -f "${task_prompt}" ]] \|\| { echo "error: reviewer task template not found: ..." >&2; exit 2; }` — precedes `{` block and any codex invocation |
| A7 | Dynamic sections remain inline | PASS | `grep "REVIEW CONTEXT\|STAGE-SPECIFIC CHECKS\|EXPECTED STAGE OUTPUT\|ARTIFACTS TO REVIEW\|DIFF TO REVIEW" scripts/codeos-review.sh` → all five at lines 264, 284, 287, 290, 292 |
| A8 | Script changes confined to `build_packet()` only | PASS | `git diff -- scripts/codeos-review.sh \| grep "^@@ "` → two hunks, both headed `build_packet()` |
| A9 | `docs/reviewer-pipeline.md` references `prompts/codeos-reviewer-task.md` | PASS | `grep "codeos-reviewer-task" docs/reviewer-pipeline.md` → match in §2 |
| A10 | Backlog slug = `lean-review-runner-packet-architecture`; features.md title updated; file retained | PASS | `grep "^slug:" backlog/UPG-0027-replacing-review-scripts.md` → match; `grep "UPG-0027" backlog/features.md` → "Lean Review Runner and Packet Architecture"; physical filename unchanged |
| A11 | Packet starts with template (five questions before REVIEW CONTEXT) | PASS | `head -5 $(ls -t reviews/codex/*.md | head -1)` on a freshly generated packet shows "Reviewer task:" on line 1 and "YOUR TASK" before any "REVIEW CONTEXT" line; confirmed by `grep -n "YOUR TASK\|REVIEW CONTEXT" <packet>` ordering |
| A12 | Status row: Class=`prompt / script-tooling`, Scope=`self-dev only`, State=`IN_PROGRESS` | PASS | `grep "CHG-20260629-002" status/self-development.md` → row confirmed |
| A13 | `dba-system.md` diff empty; no packet-manifest/local-check/delta-mode/typed-runner code | PASS | `git diff -- dba-system.md \| wc -c` → 0; `grep -r "packet_manifest\|local_check\|--mode delta\|typed.runner" scripts/ prompts/` → no match |

**Consistency sweep (grep):**

Changed files (5 modified + 2 new, all in scope):
`backlog/UPG-0027-replacing-review-scripts.md`, `backlog/features.md`, `docs/reviewer-pipeline.md`, `scripts/codeos-review.sh`, `status/self-development.md`, `prompts/codeos-reviewer-task.md` (new), `changes/UPG-0027__CHG-20260629-002__focused-reviewer-prompt.md` (new).

Stale references found and triaged:
- `prompts/reviewer-automated.md`: still contains `Critically assess:` (companion prompt doc, pre-existing, not in scope for CHG-1).
- `docs/reviewer-artifact-schemas.md`: description of packet structure references `` `Critically assess:` `` (pre-existing, not in scope for CHG-1).

Both filed as OUT-OF-SCOPE BACKLOG below.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| `prompts/reviewer-automated.md` still says `Critically assess:` — stale after template extraction | OUT-OF-SCOPE BACKLOG | File as follow-up; not a false claim in this CHG's scope |
| `docs/reviewer-artifact-schemas.md` references `Critically assess:` in packet description — stale | OUT-OF-SCOPE BACKLOG | File as follow-up; not a false claim in this CHG's scope |
