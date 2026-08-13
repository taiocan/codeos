# Self-Development Change: 0001-claude-split

<!--
PURPOSE: Per-change source of truth for a non-trivial change to the Codeos toolkit itself.
This is the dogfood record for the change that introduced the self-development loop, so it
is recorded retroactively against the work approved via plan-mode review.
-->

## Change Intent

**Why (problem in the toolkit):**
The repo-root `CLAUDE.md` did double duty — it was both (a) the file Claude auto-reads when
developing the toolkit itself, and (b) the master 9-stage DBA doctrine that downstream
projects load via `.codeos/CLAUDE.md`. That conflation forced the heavy 9-stage behavioral
loop onto Codeos's own (prose + bash) development and used `.codeos/` paths that are wrong
for in-repo work.

**What changes:**
- New `dba-system.md` — the downstream doctrine (moved from `CLAUDE.md`, body verbatim).
- Rewritten `CLAUDE.md` — the lean Codeos Self-Development guide (stable, instructional).
- New `prompts/codeos-self-dev.md`, `templates/codeos-change.md`.
- New `status/self-development.md` (live dashboard), `changes/` + `status/` dirs.
- Repointed doctrine path `.codeos/CLAUDE.md` → `.codeos/dba-system.md` in
  `templates/project-CLAUDE.md`, `scripts/dba-init.sh`, `prompts/00-session-start.md`,
  `docs/codeos-manual.md`, `README.md`.
- Reviewer wiring: guarded non-numeric `stage` in `scripts/codeos-review.sh`.

**Scope boundary — what stays the same:**
The downstream 9-stage doctrine substance is unchanged (path/location split only). No stage
prompt logic, no template semantics besides the new additions. The descriptive doc-attribution
pass (codeos-manual.md / oap docs) is explicitly deferred (see backlog).

**Class:** self-dev-governance + downstream-doctrine
**Scope axis:** both
**Backlog item:** — (governance change; spawned `backlog/doc-consistency-doctrine-rename.md`)

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | No doctrine-loading path still points at `.codeos/CLAUDE.md` | `grep -rn "\.codeos/CLAUDE\.md"` (excl. Archive) → empty |
| 2 | Generated project `CLAUDE.md` strongly loads `.codeos/dba-system.md` | `dba-init.sh` smoke test in temp dir |
| 3 | `.codeos/dba-system.md` resolves via the symlink | smoke test stat |
| 4 | Downstream 9-stage doctrine substance preserved | diff old `CLAUDE.md` body vs `dba-system.md` body |
| 5 | Root `CLAUDE.md` does not re-describe the 9 stages; points to `dba-system.md` | read-through |
| 6 | Live status is in `status/self-development.md`, not `CLAUDE.md` | read-through |
| 7 | Reviewer tolerates a non-numeric self-dev `stage` token | bash arithmetic guard test |

---

## Implementation Notes

All edits completed as scoped. `dba-system.md` body from `## Mode Declaration` onward is a
verbatim copy of the prior `CLAUDE.md`; only the title + a downstream-facing header note were
added. Root `CLAUDE.md` rewritten as the self-development guide with repo-relative paths and
no live status. Reviewer guard added at `build_packet` (only numeric stages compute a
predecessor; non-numeric stages display `n/a`). One out-of-scope item discovered and filed:
descriptive doc attributions in `docs/codeos-manual.md` and `docs/oap-*.md` still name
`CLAUDE.md` as the master doctrine — deferred to `backlog/doc-consistency-doctrine-rename.md`.

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | No stray `.codeos/CLAUDE.md` | PASS | grep returned none (excl. Archive) |
| 2 | Strong downstream bridge | PASS | dba-init smoke: generated CLAUDE.md reads `.codeos/dba-system.md` first |
| 3 | Symlink resolves to doctrine | PASS | `.codeos/dba-system.md` EXISTS in temp project |
| 4 | Doctrine substance preserved | PASS | body diff IDENTICAL from `## Mode Declaration` onward (PATH/CONTEXT ONLY) |
| 5 | Root CLAUDE.md = self-dev only | PASS | read-through; no 9-stage re-description; points to `dba-system.md` |
| 6 | Status separation | PASS | table in `status/self-development.md`; none in `CLAUDE.md` |
| 7 | Reviewer free-token stage | PASS | guard test: `selfdev-step-1` → `n/a (non-numeric stage)`, no crash |

**Consistency sweep (grep):**
Loading-path refs clean. One documentation gap (descriptive doctrine attributions in
`docs/codeos-manual.md` + `docs/oap-*.md`) filed to backlog.

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| `docs/codeos-manual.md` / `oap-*.md` attribute master doctrine to `CLAUDE.md` | OUT-OF-SCOPE BACKLOG | filed `backlog/doc-consistency-doctrine-rename.md` |
| Reviewer crashed on non-numeric stage | IN-SCOPE BLOCKER | fixed (guard in `build_packet`) |

**Note on process:** This bootstrap change predates a runnable self-dev loop, so the
compulsory per-step Codex reviews were not executed at the time; the change was instead gated
through the plan-mode review cycle (multiple human-relayed reviewer passes + explicit
approval). A retroactive doctrine-split **series** Codex review was subsequently run at HEAD
2563e37 (verdict **CHANGES ADVISED**, evidence A; see `reviews/review-log.md`); it flagged
that this record's status-dashboard header still named `backlog/features.md` as "the roadmap"
— addressed by change `0004-review-fixes`. Future non-trivial changes follow the per-step
compulsory-review discipline.

---

<!-- METADATA -->
status: COMPLETE
change_id: 0001-claude-split
type: SELF_DEVELOPMENT
class: self-dev-governance + downstream-doctrine
scope: both
backlog_item: —
step_completed: 4
approved_by: human (plan-mode approval, 2026-06-27)
approved_at: 2026-06-27
