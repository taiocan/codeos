# Automated Reviewer — Review Packet Spec (Codex)

This file documents the **prompt convention** the automated reviewer pipeline
(`scripts/codeos-review.sh`, see `docs/reviewer-pipeline.md`) sends to Codex. It is not pasted by
a human — the script builds it. The design goal is a **minimal instruction with a rich evidence
packet**: Codex's default-model critical assessment is the best feedback, so we do not role-prime
it; we give it the right evidence, the **scope it must assess against**, and ask it to critically
assess and **triage every finding by scope**.

The companion `prompts/pipeline-reviewer.md` (the human-pasted, interactive Reviewer Activation
Package) remains the canonical statement of the reviewer's *role and stance*. This file is the
**automated** path and inherits that stance: the reviewer is an independent, advisory, read-only
critical assessor. It never approves stages and never edits artifacts.

---

## Why scope triage (the scope-drift brake)

An adversarial reviewer will always find a stronger system that "should also" exist. Left
unscoped, every such finding gets treated as in-scope and the work item grows without bound
(this happened: a manual advisory reviewer MVP drifted toward a formal approval-integrity
subsystem). The brake: the reviewer assesses against the **stated work order scope**, not against
an ideal final system, and **classifies every finding by scope before anyone acts on it.** The PR
decision is based **only on in-scope blockers.**

## The prompt

The visible instruction is literally:

```
Critically assess:
```

…followed by the structured **Review Packet**. The packet is built in this order:

```
SCOPE CONTRACT
  What this PR/work order delivers and explicitly does NOT deliver. Items listed as "does NOT
  deliver" are deferred future work; a finding that the PR lacks one of them is OUT-OF-SCOPE
  BACKLOG, not a blocker — unless the PR falsely claims to provide it.

TRIAGE RULE
  Classify every finding as exactly one of:
    - IN-SCOPE BLOCKER       breaks the stated MVP goal; creates a false claim in this PR;
                             weakens the advisory/read-only/human-gated guarantees; prevents the
                             reviewer MVP from running; or violates an explicit safety constraint.
    - IN-SCOPE NON-BLOCKER   improves the MVP but is not required for this PR.
    - OUT-OF-SCOPE BACKLOG   valid, but belongs to a future feature / stronger guarantee.
    - REJECTED               conflicts with the stated scope or Codeos philosophy.

  Findings about the following are OUT-OF-SCOPE BACKLOG unless the current PR claims to provide
  them: formal approval-binding enforcement; rollback correctness; COMMIT_BOUND/WORKSPACE_BOUND
  enforcement; JSON Schema validation; CI validation; exact decision-integrity; per-feature
  decision ledgers; autonomous approval; enabled hooks.

REVIEW CONTEXT
  Feature / Stage / Branch / Base commit / Review commit / Current approved stage /
  Evidence coverage (coverage_state) / Workspace dirty

DBA RULES RELEVANT TO THIS STAGE
  - Human approval is required for every stage transition; you are advisory only.
  - Memory is not truth — assess only the artifacts/diff provided, pinned to the review commit.
  - Implementation must trace to approved artifacts; no behavior beyond intent+contract+schema.
  - No events outside the approved event schema; no hidden behavior.

STAGE-SPECIFIC CHECKS
  <the Stage-N checklist from backlog/UPG-0003-reviewer-decision-brief.md>

EXPECTED STAGE OUTPUT
  <expected artifact type + stage exit condition for Stage N>

ARTIFACTS TO REVIEW
  <for each artifact: path, SHA256, then full contents per its visibility>

DIFF TO REVIEW
  <secret/size-filtered unified diff, base->review; excluded/redacted paths noted>

INSTRUCTIONS
  <full critical assessment + per-finding triage + PR decision + LOG SUMMARY — see below>
```

## Required reviewer output format

Per finding:

```
Finding:        <short statement>
Severity:       High | Medium | Low
Classification: IN-SCOPE BLOCKER | IN-SCOPE NON-BLOCKER | OUT-OF-SCOPE BACKLOG | REJECTED
Evidence:       <file/line or artifact reference>
Why:            <short explanation>
Required action: fix now | optional fix | backlog | reject
Scope reason:   <why this finding does or does not belong to the current PR scope>
```

Then a final decision section:

```
PR decision: ADVANCE | REQUEST CHANGES | DO NOT ADVANCE
Decision basis: <based ONLY on IN-SCOPE BLOCKER findings. OUT-OF-SCOPE BACKLOG findings must NOT
                 cause DO NOT ADVANCE unless the current PR falsely claims to solve that
                 out-of-scope problem.>
Scope drift warning: yes | no — <explanation: is anything pulling this PR beyond its scope?>
```

The reviewer may still be adversarial and critical, but it must distinguish **"this PR is unsafe
or false"** from **"a stronger future system would also need X."** Only the former is a blocker.

## Machine-parsed summary line (parser compatibility)

After the triage section, the reviewer emits, on the **last two lines**, exactly:

```
LOG SUMMARY: <NO OBJECTION | CHANGES ADVISED | DO NOT ADVANCE | UNCLASSIFIED> — <single most important point>
EVIDENCE: <A|B|C|D|E>     (optional)
```

The pipeline parses only this `LOG SUMMARY` line (no new parser/validator is added in v0). Map the
`PR decision` onto it:

| PR decision | LOG SUMMARY token |
|---|---|
| ADVANCE | `NO OBJECTION` |
| REQUEST CHANGES | `CHANGES ADVISED` |
| DO NOT ADVANCE | `DO NOT ADVANCE` |
| (cannot classify safely) | `UNCLASSIFIED` |

## Concern vocabulary (advisory, NOT gatekeeping)

`APPROVE` / `BLOCK` are **reserved for the human**. The reviewer uses:

- **NO OBJECTION** — no material reason to stop found; *this is not approval*.
- **CHANGES ADVISED** — issues that should be addressed or consciously waived.
- **DO NOT ADVANCE** — a material DBA risk; the human should not approve without resolving or
  explicitly overriding.
- **UNCLASSIFIED** — set by the pipeline when no parseable `LOG SUMMARY` line is found; treated as
  **HIGH attention / manual review required**, never neutral.

The reviewer emits one concern. The pipeline may then compute an **effective concern** that is at
least as severe, when evidence coverage was partial or content was withheld (see the coverage
rules in `docs/reviewer-artifact-schemas.md`). Both the raw Codex concern and the effective concern
are logged; the human acts on the effective concern.

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
[`backlog/UPG-0003-reviewer-decision-brief.md`](../backlog/UPG-0003-reviewer-decision-brief.md) (the single source
of truth for per-stage checks) plus the expected-stage-output line for Stage N.
