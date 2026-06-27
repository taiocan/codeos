# Automated Reviewer — Review Packet Spec (Codex)

This file documents the **prompt convention** the automated reviewer pipeline
(`scripts/codeos-review.sh`, see `docs/reviewer-pipeline.md`) sends to Codex. It is not
pasted by a human — the script builds it. The design goal is a **minimal instruction with a
rich evidence packet**: Codex's default-model critical assessment is the best feedback, so we
do not role-prime it; we just give it the right evidence and ask it to critically assess.

The companion `prompts/pipeline-reviewer.md` (the human-pasted, interactive Reviewer
Activation Package) remains the canonical statement of the reviewer's *role and stance*. This
file is the **automated** path and inherits that stance: the reviewer is an independent,
advisory, read-only critical assessor. It never approves stages and never edits artifacts.

---

## The prompt

The visible instruction is literally:

```
Critically assess:
```

…followed by the structured **Review Packet**:

```
REVIEW CONTEXT
  Feature:              <feature_id>
  Stage:                <N>
  Branch:               <branch>
  Base commit:          <sha or "(uncommitted artifact)">
  Review commit:        <sha>
  Current approved stage: <N-1 or as recorded>

DBA RULES RELEVANT TO THIS STAGE
  - Human approval is required for every stage transition; you are advisory only.
  - Memory is not truth — assess only the artifacts/diff provided, pinned to the review commit.
  - Implementation must trace to approved artifacts; no behavior beyond intent+contract+schema.
  - No events outside the approved event schema; no hidden behavior.

STAGE-SPECIFIC CHECKS
  <the Stage-N checklist from backlog/reviewer-decision-brief.md>

EXPECTED STAGE OUTPUT
  <expected artifact type + stage exit condition for Stage N>

ARTIFACTS TO REVIEW
  <for each artifact: path, SHA256, then full contents>

DIFF TO REVIEW
  <secret/size-filtered unified diff, base->review>
  <note of any excluded paths / redacted hunks, if present>

INSTRUCTIONS
  Give your full critical assessment first (operational, ranked by severity, with concrete
  better-designs where you propose changes; separate required fixes from optional ones; end
  with a clear judgement). Then, on the LAST two lines, emit exactly:
    LOG SUMMARY: <NO OBJECTION | CHANGES ADVISED | DO NOT ADVANCE> — <single most important point>
    EVIDENCE: <A|B|C|D|E>     (optional)
```

## Concern vocabulary (advisory, NOT gatekeeping)

`APPROVE` / `BLOCK` are **reserved for the human**. The reviewer uses:

- **NO OBJECTION** — no material reason to stop found; *this is not approval*.
- **CHANGES ADVISED** — issues that should be addressed or consciously waived.
- **DO NOT ADVANCE** — a material DBA risk; the human should not approve without resolving or
  explicitly overriding.
- **UNCLASSIFIED** — set by the pipeline when no parseable `LOG SUMMARY` line is found;
  treated as **HIGH attention / manual review required**, never neutral.

The reviewer emits one concern. The pipeline may then compute an **effective concern** that
is at least as severe, when evidence coverage was partial or content was withheld (see the
coverage-state table in `docs/reviewer-pipeline.md`). Both the raw Codex concern and the
effective concern are logged; the human acts on the effective concern.

## Evidence grade (optional — backlog #13)

- **A** — Direct evidence from artifact/diff/test/runtime log
- **B** — Strong inference from code and tests
- **C** — Plausible but not directly proven
- **D** — Speculative
- **E** — Unknown / not reviewed

If the reviewer does not emit `EVIDENCE:`, the log records `Evidence: not reported`. Evidence
grading is only considered "implemented" once the reviewer actually emits it.

## Stage-specific checklists

The pipeline injects the Stage-N checklist from
[`backlog/reviewer-decision-brief.md`](../backlog/reviewer-decision-brief.md) (the single
source of truth for per-stage checks) plus the expected-stage-output line for Stage N.
