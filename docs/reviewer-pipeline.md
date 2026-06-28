# Codeos Reviewer Pipeline — Manual Advisory Codex Reviewer

*A read-only, advisory, cross-model reviewer for the DBA stage gates. It compresses each
stage artifact into a critical assessment and an append-only log entry so the human decides
faster — without ever becoming a gate.*

> **v0 is a manual advisory review logging pilot.** It records review evidence and human
> decisions, but it does **not** enforce complete approval-integrity or rollback correctness.
> The persisted hashes, packet copy, and `workspace_dirty` flag are **audit aids**, not a formal
> guarantee that a human approval is bound to an unchanged repository state. The deeper binding
> model (approval bound to a reproducible reviewed state, decision-time reverification, rollback
> semantics) is **deferred** — see `backlog/UPG-0015-reviewer-decision-integrity.md`.

> **The Bash implementation is a manual pilot wrapper.** It validates the workflow but is not the
> intended long-term review engine; the future typed engine is `backlog/UPG-0018-reviewer-engine-v1.md`.

```yaml
status: PILOT — manual operation; no Claude Code hooks wired
scope: implements backlog/UPG-0003-reviewer-decision-brief.md, pulls in UPG-0006 (evidence grade)
binding: changes no Codeos non-negotiable rule; CLAUDE.md and the stage prompts untouched
guarantees: advisory logging only — NOT approval-integrity or rollback (deferred to backlog)
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
*DBA-specific* rather than generic is the **evidence packet** beneath that line: the **Scope
Contract + Triage Rule** (so the reviewer classifies each finding as IN-SCOPE BLOCKER /
IN-SCOPE NON-BLOCKER / OUT-OF-SCOPE BACKLOG / REJECTED and bases its PR decision only on
in-scope blockers — this is the scope-drift brake), review context
(feature/stage/branch/base+review SHA), the DBA rules relevant to the stage, the
stage-specific checklist (sourced from `backlog/UPG-0003-reviewer-decision-brief.md`), the expected
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

Sessions are also **version-pinned**: the session record stores the `codex --version` that
created it, and a resume across a changed Codex build is refused — the script starts a fresh
session instead of resuming under a different parser/behavior.

## 4. Evidence durability + append-only log

The exact on-disk shapes of every persisted artifact below — packet, assessment header,
session state, REVIEW entry, HUMAN DECISION entry — are specified as **v0 normative schemas**
in [`reviewer-artifact-schemas.md`](reviewer-artifact-schemas.md), including required fields,
allowed enum values, and the lightweight fail-closed validation the script applies. Full
JSON Schema validation is deferred until pilot use shows it is needed.

- The **full** Codex assessment is saved under `reviews/codex/<ts>-<feature>-stage-<N>-<sha>.md`,
  opening with a self-contained YAML metadata header (feature/stage/branch/base+review
  commit/artifacts+sha256/diff_hash/coverage_state/redaction_count/secret_redaction/
  excluded_paths/reviewed_packet+sha256/codex_concern/effective_concern/evidence) so the file
  is auditable on its own.
- The **exact bytes that were reviewed** are persisted as the canonical packet under
  `reviews/codex/packets/<ts>-<feature>-stage-<N>-<sha>.packet.txt`, with its own SHA256
  recorded in the assessment and the log. It contains the full context, artifact contents,
  filtered diff, and the exclusion/redaction list — so for uncommitted artifacts, since-edited
  files, and filtered diffs you can prove precisely what Codex saw, not merely a hash of it.
  Real stage reviews are committed with the feature branch; pilot/test runs use
  `reviews/codex/_scratch/` (gitignored).
- `reviews/review-log.md` is **append-only**. The REVIEW entry records **both** the raw
  `Codex concern` and the `Effective concern` (after the coverage floor), the coverage state,
  redaction count, hashes, and links. The human decision is a **separately appended** entry
  via `codeos-review.sh decision …` — prior entries are never edited. As a **best-effort audit
  aid**, that command re-hashes the named reviewed artifacts and records MATCH / CHANGED per
  artifact; a `CHANGED` line is flagged (and a warning printed) but **never blocks** the
  decision. **The decision command records the human's choice; it does not enforce approval
  eligibility, refuse approvals, or bind approval to a reproducible reviewed state.** `APPROVE`
  is the human's word (Non-Negotiable Rule 1). Stronger guarantees — binding approval to a
  durable reviewed state (commit + diff hash + workspace snapshot), decision-time reverification,
  hard stops, and rollback semantics — are **deferred** and tracked in
  `backlog/UPG-0015-reviewer-decision-integrity.md`.

## 5. Coverage and effective concern

Defined normatively in [`reviewer-artifact-schemas.md`](reviewer-artifact-schemas.md) (coverage
rules). In summary: two filtering layers run before anything reaches Codex — **path exclusion**
(`.env*`, `*.pem`, `secrets/*`, size limit, …) applies to the **diff and incidental files only**,
and **content redaction** blanks secret-like values in place in both the diff and requested
artifacts. A requested artifact is therefore never silently dropped — it is `shown`,
`shown_redacted`, `oversize_omitted`, or `missing`. The resulting `coverage_state` fixes the
**minimum effective concern** (an evidence-coverage gap is a verdict floor, not a footnote). This
is about *how complete the evidence was*, not approval eligibility. When anything is
excluded/redacted, or on a critical/empty state, the log flags **MANUAL SECURITY REVIEW
REQUIRED**. `workspace_dirty` is recorded as descriptive audit context only.

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
| Secret/diff filtering | **Positive** | Reduces common credential-leakage risk in the review packet (heuristic, not a guarantee) |
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

The hook delegates to a small wrapper rather than an inline one-liner, because the cleanup
must be **success-gated** (do not consume the request if the review failed) and the artifact
paths must be **quoted/arrayed** (never word-split a `jq` expansion):

```jsonc
// .claude/settings.json — illustrative ONLY, not enabled
{
  "hooks": {
    "Stop": [
      { "command": "scripts/codeos-review-hook.sh" }   // no-op unless the sentinel exists
    ]
  }
}
```

```bash
# scripts/codeos-review-hook.sh — illustrative ONLY
#!/usr/bin/env bash
set -euo pipefail
req=".codeos-state/review-request.json"
[[ -f "${req}" ]] || exit 0                       # nothing requested → no-op
feature="$(jq -r '.feature' "${req}")"
stage="$(jq -r '.stage' "${req}")"
mapfile -t paths < <(jq -r '.artifacts[]' "${req}")   # array-safe, no word-splitting
# abort WITHOUT deleting the sentinel if the request is unparseable or names no artifacts
[[ -n "${feature}" && -n "${stage}" && ${#paths[@]} -gt 0 ]] || {
  echo "review-hook: malformed/empty request; sentinel preserved" >&2; exit 1; }
if scripts/codeos-review.sh review "${feature}" "${stage}" "${paths[@]}"; then
  rm -f "${req}"                                  # consume the sentinel ONLY on success
fi
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
