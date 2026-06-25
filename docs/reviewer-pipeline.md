# Codeos Reviewer Pipeline — Manual Advisory Codex Reviewer

*A read-only, advisory, cross-model reviewer for the DBA stage gates. It compresses each
stage artifact into a critical assessment and an append-only log entry so the human decides
faster — without ever becoming a gate.*

```yaml
status: PILOT — manual operation; no Claude Code hooks wired
scope: implements backlog/reviewer-decision-brief.md (#1), pulls in #13 (evidence grade)
binding: changes no Codeos non-negotiable rule; CLAUDE.md and the stage prompts untouched
```

---

## 1. Roles

- **Claude Code** runs the DBA development loop (Stages 1–9) and STOPs at
  `AWAITING HUMAN APPROVAL`.
- **Codex** is the independent reviewer, invoked **read-only** (`-s read-only`) by
  `scripts/codeos-review.sh`. Running a different model family gives cross-model adversarial
  review (less self-review circularity); running read-only means it *physically cannot edit
  artifacts* — it can only assess.
- **The human** decides. The reviewer recommends with non-gatekeeping vocabulary; `APPROVE`
  is reserved for the human.

This inherits the stance of `prompts/pipeline-reviewer.md` (the interactive Reviewer
Activation Package). This pipeline is the *automated* path; `prompts/reviewer-automated.md`
documents the prompt/packet convention.

## 2. The minimal prompt, the rich packet

The visible instruction is just `Critically assess:` — Codex's default-model critical
assessment is the best feedback, so it is not role-primed. What makes the review
*DBA-specific* rather than generic is the **evidence packet** beneath that line: review
context (feature/stage/branch/base+review SHA), the DBA rules relevant to the stage, the
stage-specific checklist (sourced from `backlog/reviewer-decision-brief.md`), the expected
stage output, the artifact contents with hashes, and the secret-filtered diff. See
`prompts/reviewer-automated.md` for the exact shape.

## 3. Session continuity — feasibility

**Question:** can one Codex session be opened at the start and reused across every stage,
instead of a fresh session per stage?

**Answer: yes — via `codex exec resume <session_id>`, not a held-open process.** This was
verified against Codex 0.114: the first review for a feature runs `codex exec` and the
session id is captured from the Codex startup banner (`session id: <uuid>`); later reviews
run `codex exec resume <id>`, which rehydrates the full prior conversation (a resumed session
correctly recalled context across separate processes in testing) while each call is a
crash-safe fresh process. This is the "same continuous session" semantically, achieved
durably. (`codex mcp-server` is a future alternative; a held-open live process was rejected
as fragile.)

Invocation details that matter (0.114): `codex exec` takes `-s read-only` and `--cd`;
`codex exec resume` takes **neither** — sandbox is set via `-c sandbox_mode="read-only"` and
the working dir is the current dir. The script handles this difference.

**Sessions are feature-scoped** — `.codeos-state/codex-sessions/<feature>.json`. Continuity
is valuable *within* a feature and dangerous *across* features, so a different feature gets
its own session and `--fresh` forces a brand-new one (use it for safety-sensitive stages,
reviewer/human disagreement, or suspected anchoring on stale/pre-correction context).

**Memory is never truth.** Every review re-reads the artifacts and diff from disk pinned to
the review commit SHA, and records that SHA + per-artifact SHA256 + a diff hash. Session
memory aids cross-stage drift detection; the disk + hashes are authoritative. This is the
guardrail against the stale-context failure mode that DBA otherwise warns about.

Session-id capture is **deterministic / fail-closed**: the id is parsed from the bootstrap
call's own banner output, so it is exactly the session just created. If no id can be parsed,
the script aborts and logs nothing.

## 4. Evidence durability + append-only log

- The **full** Codex assessment is saved under `reviews/codex/<ts>-<feature>-stage-<N>-<sha>.md`,
  opening with a self-contained YAML metadata header (feature/stage/branch/base+review
  commit/artifacts+sha256/diff_hash/excluded_paths/concern/evidence) so the file is auditable
  on its own. Real stage reviews are committed with the feature branch; pilot/test runs use
  `reviews/codex/_scratch/` (gitignored).
- `reviews/review-log.md` is **append-only**. The script appends a short REVIEW entry
  (summary + concern + hashes + link). The human decision is a **separately appended** entry
  via `codeos-review.sh decision …` — prior entries are never edited. The REVIEW entry's
  base/review SHA + the appended HUMAN DECISION entry are what let a human later identify the
  last sound "OK point" (commit/branch) to return to.

## 5. Safety — secret + oversized-diff filtering

Two layers before anything reaches Codex:
1. **Path exclusion** — `.env*`, `*.pem`, `*.key`, `secrets/*`, `credentials/*`, raw runtime
   logs, files over a size threshold.
2. **Content redaction** — secret-like values (`OPENAI_API_KEY=`, `ANTHROPIC_API_KEY=`,
   `AWS_SECRET_ACCESS_KEY`, `BEGIN … PRIVATE KEY`, `password=`/`token=`/`secret=`) are
   redacted from the diff.

When anything is excluded or redacted, the packet and the log entry flag **"manual security
review required"**, so the reviewer's coverage gap is explicit.

## 6. Concern-level semantics + human responsibility

- **NO OBJECTION** — no material reason to stop found; *this is not approval*.
- **CHANGES ADVISED** — issues that should be addressed or consciously waived.
- **DO NOT ADVANCE** — a material DBA risk; the human should not approve without resolving or
  explicitly overriding.
- **UNCLASSIFIED** — malformed/insufficient reviewer output (no parseable `LOG SUMMARY`);
  treated as **HIGH attention / manual review required**, never neutral.

Evidence grade (optional, backlog #13): `EVIDENCE: A–E` — concern level is *what the reviewer
thinks*; evidence grade is *how well supported it is*. If absent, the log records
`Evidence: not reported`; #13 is not "done" until the reviewer reliably emits it.

> **The reviewer reduces human reading load; it does not reduce human responsibility.** A
> human may approve a stage against the reviewer, but must record the reason in the HUMAN
> DECISION entry when doing so. The reviewer is evidence compression, not decision transfer.

## 7. What a good review looks like (calibration)

This pipeline was itself shaped by several rounds of real Codex review of its own plan. The
qualities that made those reviews valuable are the bar the automated reviewer aims at:

- **Operational, not only philosophical** — it named concrete bugs (append-only violations,
  wrong state locations, brittle session capture), not just abstractions.
- **Ranked by severity** — required corrections separated from optional improvements.
- **Concrete better-designs** — every objection came with a specific proposed fix.
- **Honest about tradeoffs** — e.g. flagging when a "one cheap call" claim was really a
  mini-pilot.
- **Ends with a clear decision** — approve / approve-with-fixes / do-not-approve, per area.

The stage-specific checklists encode this intent; the packet's INSTRUCTIONS line asks for
exactly this shape.

## 8. DBA-philosophy scorecard

| Capability | DBA impact | Why |
|---|---|---|
| Cross-model Codex reviewer, read-only | **Positive** | Adversarial second model; cannot edit artifacts |
| Feature-scoped session via `exec resume` | **Neutral** | Re-reads artifacts + SHA-pins every review; `--fresh` escape hatch; no cross-feature bleed |
| Durable assessments + append-only log (no mutable fields) | **Aligned** | Mirrors `runtime_events.jsonl` + existing append-only Decision Log |
| Advisory concern field (non-gatekeeping words) | **Neutral** | `APPROVE` reserved for the human |
| Secret/diff filtering | **Positive** | Prevents credential leakage into the review packet |
| Automated hooks | **Risky → kept inert** | Documented (Appendix), not wired |
| Autonomous stage approval | **Negative — violates rule #1** | Rejected/deferred (Appendix) |

## 9. Acceptance criteria (mini-design gate)

read-only reviewer edits no artifacts · review output durable (full assessment saved) ·
sessions feature-scoped · reviewed state pinned (base+review SHA, artifact hash) · malformed
output → UNCLASSIFIED/high-attention · secret/large-diff filtering present · no hooks active ·
no core rules changed.

## 10. Usage

```bash
# record the base commit for a stage (so review diffs base->review, not just HEAD)
scripts/codeos-review.sh stage-start listing-ingestion 2

# review an artifact (resumes the feature's Codex session; --fresh starts a new one)
scripts/codeos-review.sh review listing-ingestion 2 contracts/listing-ingestion_contract.md

# after the human decides, append the decision (never edits prior log entries)
scripts/codeos-review.sh decision listing-ingestion 2 REQUEST_CHANGES "missing failure scenario"
```

---

## Appendix A — Inert hook snippets (NOT part of the pilot)

These are provided for a *future* phase only. **Do not add them to `.claude/settings.json`
yet** — the pilot runs the script manually until the advisory reviewer has a proven track
record. A guarded `Stop` hook keyed on a sentinel avoids reviewing every stop.

```jsonc
// .claude/settings.json — illustrative ONLY, not enabled
{
  "hooks": {
    "Stop": [
      {
        // only fires when Claude wrote .codeos-state/review-request.json at a stage gate
        "command": "test -f .codeos-state/review-request.json && scripts/codeos-review.sh review \"$(jq -r .feature .codeos-state/review-request.json)\" \"$(jq -r .stage .codeos-state/review-request.json)\" $(jq -r '.artifacts[]' .codeos-state/review-request.json) ; rm -f .codeos-state/review-request.json"
      }
    ]
  }
}
```

## Appendix B — Rejected / Deferred — Not Approved for Implementation

**Autonomous stage approval.** Letting the reviewer approve stages for "simple" features
contradicts **non-negotiable rule #1** (every stage transition requires explicit human
approval) and converts DBA from *synchronous prevention* (the gate blocks a bad transition)
to *asynchronous detection + rollback* (it happens, is caught later, is reverted). Recorded
for traceability only. It would separately require: per-feature human opt-in, one commit per
stage, feature-registry support (#14), a rollback design, low-risk-only scope that **never**
includes safety/authorization/invariant contracts, a hard stop on any `DO NOT ADVANCE` or
low-evidence review, and an amendment to the human-approval invariant. Not built toward now.
