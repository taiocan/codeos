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
binding: changes no Codeos non-negotiable rule; stage prompts untouched
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

The static reviewer task — five focused questions, what-not-to-do guidance, five-category
triage rule, and required output shape — is defined in `prompts/codeos-reviewer-task.md`
and injected at the top of every packet by `scripts/codeos-review.sh`. What makes the
review *DBA-specific* rather than generic is the **evidence packet** that follows: review
context (feature/stage/branch/base+review SHA), the DBA rules relevant to the stage, the
stage-specific checklist (sourced from `backlog/UPG-0003-reviewer-decision-brief.md`), the
expected stage output, the artifact contents with hashes, and the secret-filtered diff. See
`prompts/reviewer-automated.md` for the full packet shape.

Immediately after the reviewer task template and before `REVIEW CONTEXT`, every packet
includes a **`PACKET MANIFEST`** section listing each artifact with its inclusion mode
(`full_file`, `path_sha_only`, or `omitted_with_reason`), byte count, and sha256 (where
content is present), plus the diff size, `review_content_bytes` (full_file artifact bytes +
diff bytes), `estimated_review_tokens` (~`review_content_bytes / 4`), and `budget_status`.
If `review_content_bytes` exceeds `CODEOS_PACKET_BUDGET_BYTES` (default 50 000), a warning
is emitted to stderr and recorded in `budget_status`; the review is never aborted.

Pass `--sha-only PATH` (repeatable) to include a file in the manifest as `path_sha_only`
(path + sha256 + original byte count) without sending its content to the reviewer. Useful
for large unchanged reference files where the reviewer only needs to verify identity. If a
`--sha-only` path does not exist on disk, the script exits non-zero before any Codex
invocation — a missing guard file is an error, not a quietly omitted artifact.

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
  Going forward (see §4a), stage reviews that are part of the official audit trail are
  committed with the feature branch; pilot/test runs use `reviews/codex/_scratch/` (gitignored).
  Pre-policy reviews were not committed — see §4a and the `reviews/review-log.md` header.
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

## 4a. Review artifact durability policy

Not all review artifacts need to be committed. This policy (established by UPG-0029 /
CHG-20260629-001) defines which artifacts are **committed (durable)**, which are **scratch
(local-only)**, and what `reviews/review-log.md` references must satisfy.

**Committed / durable:** A review assessment or packet is *committed (durable)* when it is
cited in `reviews/review-log.md` by path+sha without a `[local-only]` marker, and the file
is committed to the repository. Such files are verifiable from a fresh checkout — another
reviewer can locate and read them. Place durable files under `reviews/codex/` (root) and
`reviews/codex/packets/`; commit them with the feature branch.

**Scratch / local-only:** Pilot runs, test reviews, and exploratory assessments that will
not be cited in the official log are kept local-only. Place them under
`reviews/codex/_scratch/` (gitignored); they are never committed.

**The rule for `review-log.md` references:** A log entry that references a full assessment
by path+sha must *either* point to a file that is committed to the repository, *or* the log
entry must explicitly mark the reference as `[local-only]` / non-durable. A reference that
does neither creates a fake audit trail — a checkout cannot verify it, and another reviewer
cannot read it.

See `reviews/review-log.md` header for the retroactive classification of pre-policy entries.

## 4b. Delta review mode (R2+)

CLI: `bash scripts/codeos-review.sh review <feature> <stage> <artifacts…> --mode delta --base <sha>`

When running R2 or later for the same step, send a **delta packet** rather than the full
context. A delta packet contains only:

1. The specific acceptance criterion or claim under challenge (verbatim from the change record)
2. Changed lines since the previous round — exact unified diff of affected files only; no
   surrounding unchanged context
3. One-line per-finding summary from the previous round: finding description,
   IN-SCOPE BLOCKER|NON-BLOCKER classification, and what was changed to resolve it
4. Current trace header state: `state`, `current_step`, `review_state` only

A delta packet must **NOT** include:
- Full backlog catalogs or feature briefs
- Full prior assessment prose
- Unchanged file contents
- Unrelated documents (roadmap, dba-system.md, downstream doctrine, etc.)
- The full change record if only one section changed

**Round trigger:** use delta mode for R2 and every subsequent round at the same step. R1 always
gets the full packet.

## 4c. Claim audit

Before every Codex call, scan all new or modified prose for **universal quantifiers**: "all",
"every", "never", "always", "no X", "any", "none". For each instance:

1. **Provide evidence** — confirm the claim is literally true; state how you would verify it.
2. **Weaken** — replace with "most", "typically", "in most cases", or a conditional if the
   claim is not universal.
3. **Remove** — if the claim adds no value or cannot be defended.

Universal claims without evidence are the most common source of Codex-flagged false claims
across UPG-0001 and UPG-0029. Running this audit before calling Codex prevents a class of
blockers that would otherwise cost a full review round.

## 4d. Review-round budget table

| Profile | Applies when | Max rounds/step | Budget-exceeded action |
|---|---|---|---|
| PROFILE-0 | `trivial` / direct-edit `backlog-only` | 0 (no review) | — |
| PROFILE-1 | Escalated `backlog-only` | 2 (Reconcile only) | Fix inline; escalate to human |
| PROFILE-2 | `documentation` | 2 (per step) | Fix inline; escalate to human |
| PROFILE-3 | `template` / `prompt` / `script-tooling` | 3 (per step) | Fix inline; escalate to human |
| PROFILE-4 | `downstream-doctrine` | 3 (per step) | Fix inline; escalate to human |
| PROFILE-5 | `self-dev-governance` | 3 (per step) | Fix inline; escalate to human |

**Budget-exceeded escalation procedure:**
1. Fix any remaining in-scope findings inline without running another Codex round.
2. Append a budget-exhausted entry to `reviews/review-log.md` describing what was fixed and
   what remains.
3. Escalate to the human at the gate — present the findings, what was fixed, and what requires
   human judgment.
4. Do not run further Codex rounds automatically. The human decides whether any remaining issue
   warrants another round (which counts against the budget) or can be accepted as-is.

SELF-REFERENCE / REVIEW-BOOKKEEPING findings found at budget exhaustion are always resolved by
human decision without further review.

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

**Full Context Diff (supplementary context only):** The Full Context Diff section
(appended in `--mode delta --base` runs) is supplementary and informational, not primary
evidence. Its redactions increment `redaction_count` in the assessment header, but they do
**not** change `coverage_state` and do not trigger the manual-security-review flag — because
`coverage_state` reflects named-artifact evidence completeness, not the supplementary diff.
A git error in the full-diff fetch is marked explicitly as `[ERROR: git diff failed — …]`
in the packet, so it is never silent.

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

## 10. Architecture: `codeos-review.sh` is a static locator shim

```bash
# scripts/codeos-review.sh — the entire file
exec "${BINARY}" "$@"
```

`codeos-review.sh` is a **15-line static locator shim**. It finds the compiled Rust binary
(`tools/reviewer/target/release/codeos-reviewer`) and passes all arguments through verbatim
(`"$@"`). It contains no argument preprocessing, no conditional logic, and no reviewer
capability.

**Consequence for upgrades:** any reviewer capability change — new packet sections, new
subcommand behavior, new flags, new decision-log fields — lives in the **Rust engine**
(`tools/reviewer/src/`). Changing only the bash script cannot add or modify reviewer
behavior. The bash script only needs to change if the binary location or build instructions
change.

## 11. Usage

```bash
# record the base commit for a stage (so review diffs base->review, not just HEAD)
scripts/codeos-review.sh stage-start listing-ingestion 2

# review an artifact (resumes the feature's Codex session; --fresh starts a new one)
scripts/codeos-review.sh review listing-ingestion 2 contracts/listing-ingestion_contract.md

# after the human decides, append the decision (never edits prior log entries)
scripts/codeos-review.sh decision listing-ingestion 2 REQUEST_CHANGES "missing failure scenario"
```

**Local prechecks** run automatically before the packet is built and before any Codex
invocation. They scan only the positional artifact paths passed to `review`. Two hard-fail
checks exit non-zero immediately: (1) a literal unfilled template placeholder (`UPG-####` or
`CHG-YYYYMMDD-NNN`) in any artifact, and (2) a line-anchored `latest_review:` field (a
schema field superseded by UPG-0001). A warning is emitted to stderr (but Codex is still
invoked) for unresolved draft markers: `TODO`, `FIXME`, `TBD`, `[to be filled]`. Pass
`--guard-clean PATH` (repeatable) to assert that a specific file — e.g. `dba-system.md`
during a `self-dev only` change — has no uncommitted changes; a non-existent path or a dirty
path both exit non-zero before Codex. Pass `--skip-prechecks` to bypass all checks (emits a
visible `warning: prechecks skipped` to stderr); useful for inspecting draft artifacts with
`--print-packet`.

---

## 12. Downstream usage (DBA projects, not Codeos self-development)

Everything above documents this pipeline from Codeos's own self-development perspective. A
downstream project — one that ran `dba-init.sh` and loads `.codeos/dba-system.md` — uses the
exact same `codeos-reviewer` binary and `review`/`decision`/`diagnose` subcommands, with two
differences:

1. **Stage identifiers are the downstream Stage IDs**, not `selfdev-step-N`: `discovery`,
   `brief`, `onboarding`, `1` through `9`, and `10` — see `dba-system.md`'s "What You Do at
   Each Stage" table for the full mapping and "Default Advisory Review" for when each is used.
2. **Cadence is the flat rule in `dba-system.md`'s "Default Advisory Review" section** —
   round 1 before the gate, rounds 2-3 for fixes/deltas, stop after 3 and escalate to a human.
   This is a separate, uniform cadence from the review-round-budget table used for triaging
   Codeos's own toolkit changes (§4d above) — that internal triage system never appears in
   downstream-facing doctrine or prompts.

**Known limitation — invoke the binary directly for now, not the shim.**
`.codeos/scripts/codeos-review.sh` currently resolves its binary path via the *calling*
project's git root, which breaks when run from within a downstream project (it looks for the
binary under that project's own nonexistent `tools/reviewer/`, instead of resolving through
the `.codeos` symlink to Codeos). Until this is fixed (tracked as `UPG-0038`), invoke the
compiled binary directly, e.g. reviewing a Stage 2 contract:
```bash
/path/to/Codeos/tools/reviewer/target/release/codeos-reviewer review checkout-flow 2 \
  contracts/checkout-flow_contract.md
```
Reviewing a Feature Brief before confirming it:
```bash
/path/to/Codeos/tools/reviewer/target/release/codeos-reviewer review checkout-flow brief \
  backlog/checkout-flow.md
```
(`/path/to/Codeos` is wherever `.codeos` resolves to — check with `readlink -f .codeos`.)

**If reviewer tooling isn't built or configured** for a downstream project, see
`dba-system.md`'s Review Waiver practice — record a plain reason in that feature's review
log and proceed; the human-approval gate (Non-Negotiable Rule #1) still applies regardless.

---

## 13. Verification round-trip

Every review round already ends with a `HIGHEST-IMPACT UNCERTAINTY:` line (mandated by
`prompts/codeos-reviewer-task.md`'s output format) — one sentence naming the single thing
that, if wrong, most affects the assessment. Separately, `prompts/verify-only.md` implements
a full read-only verification mode: a no-edit rule, before/after anti-blur `git status`/`git
diff --exit-code` checks proving the tree wasn't mutated, and a structured Verification-Only
Report.

When that uncertainty names something mechanically checkable — a specific file, command, or
repository state — the acting agent may run a `verify-only.md` pass targeting exactly that
uncertainty, then feed the resulting report back as evidence for the next review round. This
session used exactly this pattern more than once: UPG-0019's Step 3 rounds re-ran the review
with `check_drift.rs` (then `main.rs`) shown directly after the prior round's uncertainty
named an unverified claim about their behavior; UPG-0024's Step 2 rounds resolved two
internal-contradiction findings the same way — show more, re-review, resolve.

This is judgment, not automation: the acting agent decides whether an uncertainty is
checkable and whether running verification is worth the round-trip. It is never mandatory,
and a verification pass never substitutes for the human's decision at the gate — it only
adds evidence to it. A verification pass does not itself count against the round-budget table
in §4d; only the review round that follows it does.

`dba-system.md`'s "Default Advisory Review" section carries the same practice, in the same
terms, for downstream DBA projects — the two are kept in sync deliberately, not maintained
independently.

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
