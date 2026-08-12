# Self-Development Change: UPG-0060__CHG-20260802-001 — deepseek-implement-tool

<!--
PURPOSE: Per-change source of truth for CHG-A of UPG-0060 — build and pilot the out-of-band
DeepSeek Stage 4/5 implementer tool. Self-development toolkit change (script-tooling); no downstream
behavioral contract, event schema, or replay. Workflow: prompts/codeos-self-dev.md (4-step loop).
The live status row lives in status/self-development.md, not here.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0060
primary_feature_id: UPG-0060
change_id: CHG-20260802-001
slug: deepseek-implement-tool
state: COMPLETE      # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 4-Reconcile   # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0060
related_features: [UPG-0056, UPG-0032, UPG-0057]
review_series: RVS__UPG-0060__CHG-20260802-001__S4   # ALL Step-4 reviews for this change (stable); S1-S3 ACCEPTED
review_profile: PROFILE-3   # script-tooling (Step 0a)
review_state: ACCEPTED  # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round)
review_history: reviews/review-log.md   # exact per-round REV__…__R<N> verdicts + human decisions live here
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round. Reference the stable review SERIES (review_series) + review_state; exact rounds live
only in reviews/review-log.md and reviews/codex/*. -->


## Change Intent

**Why (problem in the toolkit):**
Codeos offloads advisory *review* work to the Codex CLI to keep it off Claude Code's token budget, but
has no equivalent for the most token-expensive AI work in the downstream loop: Stage 4 (Implementation)
and Stage 5 (Tests), where the AI writes code and tests as a constrained satisfier. A cheaper model is
available (`DEEPSEEK_API_KEY` is present in the environment), but two facts shape the design: (1)
delegating implementation differs from delegating review — the draft is the primary artifact that flows
through the gates, so net token benefit must be *measured*, not assumed; and (2) the reviewer is a
local-CLI, read-only integration with no HTTP or API-key path in the repo, so a DeepSeek-via-API tool is
a new kind of integration that *writes* code. This change builds and pilots the tool, off by default,
and captures token/quality evidence — with no downstream-doctrine change (that is CHG-B, contingent on
this pilot).

**What changes:**
- `scripts/codeos-implement.sh` — **new.** Out-of-band tool mirroring `scripts/codeos-review.sh`'s
  entry-point discipline: `set -euo pipefail`; git-repo precondition; self-dev-vs-downstream context
  resolution via `pwd -P` symlink comparison against `CODEOS_ROOT`; explicit, documented exit codes;
  fail-closed preconditions. Reads the activation status file per the Optional Mechanism Status
  Convention (UPG-0056) and refuses to run unless `status: enabled`. When enabled: builds the packet
  from `prompts/codeos-implementer-task.md` plus the approved artifacts, calls the DeepSeek API with
  `$DEEPSEEK_API_KEY`, writes the candidate to a staging directory (never `modules/`/`tests/`, never
  committed), enforces the stage-area path allowlist on every candidate path (Stage 4 → `modules/`,
  Stage 5 → `tests/`), preserves the sent packet and raw response for audit, and logs DeepSeek token
  usage.
- `prompts/codeos-implementer-task.md` — **new.** The packet task prompt pinning DeepSeek to the
  Stage-4 constrained-satisfier role (`prompts/04-implement.md`) — implement only what the three
  approved artifacts specify, emit the required events, satisfy the contract clauses; for stage 5,
  author behavioral and replay tests only.
- `config/delegated-implementation.yaml` — **new.** Self-dev activation status file, scaffolded at
  `status: disabled` (mechanism off by default; missing means disabled per the convention).
- `backlog/UPG-0060-deepseek-delegated-implementation.md` — **new** feature brief (this change's
  Feature Thread).
- `changes/UPG-0060__CHG-20260802-001__deepseek-implement-tool.md` — **new**, this record.
- `changes/UPG-0060__CHG-20260802-001__pilot-evidence.md` — **new.** Durable, committed record of the
  fail-closed suite (embedded runner + verbatim output) and the Stage-4 pilot, so the empirical
  Implementation-Notes claims are auditable in-repo (the live candidate stays under gitignored
  `.codeos-state/`).
- Lifecycle bookkeeping: `backlog/features.md` (new UPG-0060 row), `status/self-development.md` (new
  IN_PROGRESS row), `status/roadmap.md` (record the in-flight change).
- Pilot evidence: one captured `codeos-implement.sh` run against a sample feature with a
  token-and-quality comparison (DeepSeek tokens spent vs. Claude generation avoided; does the candidate
  pass the existing Stage 5 tests and `codeos-review.sh review`?), recorded in Implementation Notes.

**Scope boundary — what stays the same:**
- No change to `dba-system.md` or any downstream stage prompt (`prompts/04-implement.md`,
  `prompts/05-tests.md`, `prompts/06-observe.md`). Doctrine wiring is CHG-B, contingent on this pilot.
- No change to the Rust reviewer engine (`tools/reviewer/`); no DeepSeek provider is added there.
- No new mandatory human-approval gate, no new Stage ID, no Non-Negotiable Rule change.
- The tool never promotes a candidate into `modules/`/`tests/` and never commits; the human gate,
  advisory review, and reconciliation are unchanged.
- Stage 6 (Observation) is out — it is human-run runtime execution with no implementation to delegate.
- The mechanism is not enabled by default anywhere.

**Class:** script-tooling
**Scope axis:** self-dev only
**Backlog item:** backlog/UPG-0060-deepseek-delegated-implementation.md

---

## Acceptance Criteria

<!-- The consistency contracts this change must satisfy; each checkable in Reconcile. script-tooling:
I/O behavior, exit-code / fail-closed cases, idempotency, secret non-leakage. -->

Terms used below: the **activation status file** is `config/delegated-implementation.yaml` when the
caller's git root is `CODEOS_ROOT` (self-dev), else `architecture/delegated-implementation.yaml`
(downstream), resolved the same way `scripts/codeos-review.sh` resolves its writing-discipline file.
The **staging directory** is `.codeos-state/deepseek-candidates/<feature>-stage-<n>/`.

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | **Off by default.** When the activation status file is absent, the tool refuses to run, writes nothing, makes no network call, and exits non-zero with a message naming the resolved status-file path. | Run with no status file present; assert non-zero exit, stderr names the path, and no staging dir / no network call. |
| 2 | **Explicit disable honored.** When the status file is exactly `status: disabled`, the tool refuses to run via the same fail-closed path as criterion 1. | Set the file to `status: disabled`; assert the criterion-1 behavior. |
| 3 | **Malformed config is a distinct, fail-closed error.** When the status file is not exactly one non-blank line of `status: enabled` or `status: disabled` (per the UPG-0056 four-outcome convention: CRLF normalized, leading/trailing blank lines allowed, nothing else), the tool exits with a distinct configuration-error code and a message, before any network call. | Provide a malformed file (extra line / other value); assert the distinct exit code and message. |
| 4 | **Enabled is the only run path.** The tool builds a packet and calls DeepSeek only when the status file is exactly `status: enabled`. | With `status: enabled` and a valid key, confirm a packet is built and the API is called (observed via the preserved packet + response). |
| 5 | **Git-repo precondition.** Outside a git repository the tool fails closed with a distinct exit code, mirroring `scripts/codeos-review.sh`. | Invoke from a non-git directory; assert the distinct exit code. |
| 6 | **Missing key fails closed before any network call.** With the mechanism enabled but `DEEPSEEK_API_KEY` unset or empty, the tool exits non-zero with a clear message and makes no network call. | Unset `DEEPSEEK_API_KEY`; assert non-zero exit, message, and no request issued. |
| 7 | **Usage contract.** The tool accepts `<feature_id> <stage> [artifact paths…]` and rejects any `stage` other than `4` or `5` with a usage error and a distinct exit code. | Invoke with `stage=6` and with missing args; assert usage error + exit code. |
| 8 | **Documented, distinct exit codes.** Every fail-closed case (git precondition, missing dependency, status disabled, status malformed, missing key, bad usage, API/transport error) has a distinct exit code enumerated in the script header, in the style of `codeos-review.sh`. | Read-through of the header table; trigger each case and assert the documented code. |
| 9 | **Write-safety — staging only, never promote, never commit.** On a successful run the candidate is written only under the staging directory; the tool writes nothing under `modules/` or `tests/` and runs no `git add`/`git commit`. | After a run, `git status` shows only untracked additions under `.codeos-state/`; no tracked file under `modules/`/`tests/` is modified; no new commit exists. |
| 10 | **Secret non-leakage.** The literal value of `DEEPSEEK_API_KEY` never appears in the preserved packet, the raw response dump, the invocation log, or any candidate file. | `grep -r "$DEEPSEEK_API_KEY"` over the staging dir + log + packet returns no match after a run. |
| 11 | **Auditability + token instrumentation.** Each run preserves the exact packet sent and the raw model response under the staging/audit path, and records DeepSeek token usage (prompt, completion, total) for the run. | Inspect the preserved packet + response files and the logged token counts after a run. |
| 12 | **Harness idempotency.** Re-running for the same feature/stage does not overwrite or corrupt a prior run's audit artifacts (distinct, timestamped paths, as the reviewer names packets) and leaves the repo's tracked files unchanged. | Run twice; assert distinct audit artifacts and a tracked-file-clean `git status`. |
| 13 | **Scope preservation.** CHG-A changes no downstream doctrine or stage prompt — `dba-system.md`, `prompts/04-implement.md`, `prompts/05-tests.md`, `prompts/06-observe.md` are byte-unchanged — and `tools/reviewer/` is unchanged. | `git diff --stat` over the change shows none of these paths touched. |
| 14 | **Scaffolded off.** The self-dev status file `config/delegated-implementation.yaml` shipped by this change contains exactly `status: disabled`. | Read the file. |
| 15 | **Pilot evidence recorded.** Implementation Notes record one real pilot run: the DeepSeek token usage; a quality check of the candidate against the existing gates (does it pass the relevant Stage 5 tests and/or `codeos-review.sh review`?); and an explicit net-token assessment (DeepSeek tokens spent vs. Claude generation tokens avoided) with a stated verdict on whether CHG-B is warranted. | Read Implementation Notes; confirm the run's artifacts are referenced and the net-token verdict is stated. |

---

## Implementation Notes

<!-- Factual reporting. The git diff is the source of truth. -->

**Built:**
- `scripts/codeos-implement.sh` — the tool. Fail-closed order: git-repo precondition → usage/stage
  (4|5) → dependencies (curl, jq) → context-resolved activation status → artifacts exist →
  `DEEPSEEK_API_KEY` present → DeepSeek call → stage candidate. Status resolution copies
  `codeos-review.sh`'s `pwd -P` self-dev-vs-downstream logic and the UPG-0056 four-outcome parse. The
  DeepSeek request uses the OpenAI-compatible `chat/completions` shape with
  `response_format: json_object`; the key is passed only through a `curl -K -` config on stdin, never
  in argv, the request body, the packet, the response, or any candidate. Output is parsed with `jq`
  and each candidate file is written under a `mktemp`-unique staging leaf
  `.codeos-state/deepseek-candidates/<feature>-stage-<n>/<ts>.XXXXXX/candidate/…`, with candidate
  paths validated against `/`-absolute paths, `..` traversal, and the stage-area allowlist (Stage 4 →
  `modules/`, Stage 5 → `tests/`; `CANDIDATE_BLOCKED.md` excepted). Documented exit codes 0–8.
- `prompts/codeos-implementer-task.md` — pins the model to the constrained-satisfier role (Stage 4
  implements only what the approved Intent/Contract/Event-Schema specify and emits only schema events;
  Stage 5 writes behavioral + replay tests of observable behavior) and defines the strict JSON output
  contract (`files[]` + short `contract_satisfaction` / `event_emission` / `notes`).
- `config/delegated-implementation.yaml` — self-dev activation status file, shipped `status: disabled`.

**Assumptions:**
- DeepSeek exposes an OpenAI-compatible `chat/completions` endpoint honoring
  `response_format: {type: json_object}`. Model and URL are overridable via `CODEOS_DEEPSEEK_MODEL`
  (default `deepseek-chat`) and `CODEOS_DEEPSEEK_URL`. `curl` and `jq` are on PATH (else exit 2).

**Fail-closed verification (no network in any case):** 9/9 cases pass — non-git → 1; missing args and
`stage=6` → 3; `status: disabled` and absent file → 4; malformed (bad value, extra line) → 5;
enabled + unset key → 6 (before any network call); enabled + key + missing artifact → 7 (before any
network call). The runner and its verbatim output are embedded in the durable evidence file
`changes/UPG-0060__CHG-20260802-001__pilot-evidence.md`. Exit 2 (missing dependency) and exit 8
(API/transport error) are demonstrated separately in that file's §1b, so all documented codes 1–8
(plus 0 = success in §3) are exercised.

**Pilot run (AC-15):** a minimal approved sample feature — a bounded counter (max 100), Python, with
Intent/Contract/Event-Schema marked `APPROVED` — was implemented at Stage 4 with the mechanism
temporarily enabled (restored to `disabled` immediately after, via an `EXIT` trap).
- Result: `deepseek-chat` returned exactly **1** candidate file, `modules/counter.py`, in
  ~1.8K tokens per run (exact per-run counts are in the pilot-evidence file and `implement-log.md`).
- Quality: the candidate traces to every contract clause — C1 (increment when `value < MAX`), C2
  (`CounterIncremented` carrying `new_value`), C3 (unchanged at max), and the `AtMaximum` failure
  (`IncrementRejected`, reason `at_maximum`). It emits only the two permitted schema events.
  `python3 -m py_compile` clean. `notes.txt` recorded the single internal choice not fixed by the
  artifacts (returning event dicts rather than assuming an event-emitter infrastructure). It was
  **not** promoted; the existing Stage 4 gate + advisory review would still apply.
- Gate check (AC-15): `codeos-review.sh review` run on the candidate against its approved artifacts
  returned **CHANGES ADVISED (evidence A)** with two in-scope findings — the candidate added a required
  `correlation_id` argument outside the approved interface, and event-schema conformance for it was not
  guaranteed. The existing advisory gate caught real issues in the DeepSeek draft before any human
  approval — the candidate stays a candidate. Full record in the evidence file §4.
- Write-safety (AC-9): `git status` showed no writes under `modules/` or `tests/` and no commit — the
  candidate lives under gitignored `.codeos-state/`.
- Secret non-leakage (AC-10): `grep -rF "$DEEPSEEK_API_KEY"` over packet, response, candidate, and log
  found nothing; no `Bearer` token string is present in any staged artifact.
- Auditability (AC-11): each run preserved `packet.txt`, `request.json`, `response.json`,
  `model_content.json`, `tokens.txt`, and the `notes`/`contract_satisfaction`/`event_emission`
  sidecars, and appended one record to `implement-log.md`.
- Idempotency (AC-12): re-running produced a distinct, non-overwriting staging dir each time (the
  `mktemp` suffix makes same-second re-runs distinct), each with its own audit set; the exact run
  count is in `implement-log.md` and the evidence file.

**Findings addressed (Step 3 R1):** R1-01 (High) and R1-02 (Medium) — pilot and fail-closed evidence
was not auditable from the review packet — resolved by the durable, committed evidence file
`changes/UPG-0060__CHG-20260802-001__pilot-evidence.md`, which embeds the runner and verbatim outputs.
R1-03 (Medium) — the "strict path" contract was stated but not enforced — resolved by making the tool
reject any candidate path outside the stage area (`modules/` for Stage 4, `tests/` for Stage 5), with
the `CANDIDATE_BLOCKED.md` escape hatch, and matching the prompt wording.

**Net-token verdict (AC-15):** the mechanism works end-to-end and produced a plausible Stage-4 draft for
~1.8K DeepSeek tokens that the existing advisory gate then reviewed (CHANGES ADVISED — real issues
caught before human approval; see the gate check above). On a toy feature the absolute Claude saving is
small — the offloaded work is
the *generation*, which is minimal here — so this pilot demonstrates **viability and correctness**, not
a headline saving. The saving concentrates on realistic features (hundreds of lines of implementation),
where DeepSeek's far lower per-token price applies to the bulk generation while Claude keeps only the
approval, reconciliation, and review. Honest conclusion: **mechanism viable; CHG-B (doctrine wiring)
warranted, but conditioned on a net-token measurement against a realistic downstream feature**, per the
feature's stated contingency. The abandonment path stays open if that measurement is not net-positive.

**Out-of-scope items filed:** none. No `dba-system.md`, downstream stage prompt, or `tools/reviewer/`
change was made (AC-13). The sample feature and test runner live in the session scratchpad, not the
repo.

---

## Reconciliation

<!-- Layer D1: advisory verdict, evidence separated from inference. -->

**Acceptance verification:** (evidence file = `changes/UPG-0060__CHG-20260802-001__pilot-evidence.md`)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Off by default (absent → refuse, no network) | PASS | Fail-closed suite: absent → exit 4 (evidence §1) |
| 2 | Explicit `status: disabled` honored | PASS | Fail-closed suite: disabled → exit 4 (evidence §1) |
| 3 | Malformed status → distinct config error | PASS | Fail-closed suite: bad value & extra line → exit 5 (evidence §1) |
| 4 | `status: enabled` sole run path | PASS | Pilot ran only with enabled; disabled/absent refused (evidence §1, §3) |
| 5 | Git-repo precondition | PASS | Fail-closed suite: non-git dir → exit 1 (evidence §1) |
| 6 | Missing key fails closed pre-network | PASS | Fail-closed suite: enabled + unset key → exit 6, no request (evidence §1) |
| 7 | Usage contract; stage ∉ {4,5} rejected | PASS | Fail-closed suite: missing args & `stage=6` → exit 3 (evidence §1) |
| 8 | Documented distinct exit codes | PASS | Header table codes 0–8 (`scripts/codeos-implement.sh:34-42`); codes 1,3–7 triggered in evidence §1, codes 2 and 8 in §1b, code 0 (success) by the pilot in §3 |
| 9 | Write-safety — staging only, no promote, no commit | PASS | `git status`: no writes under `modules/`/`tests/`, no commit (evidence §3) |
| 10 | Secret non-leakage | PASS | `grep -rF "$DEEPSEEK_API_KEY"` over packet/response/candidate/log: no match (evidence §3) |
| 11 | Auditability + token instrumentation | PASS | `packet.txt`/`request.json`/`response.json`/`tokens.txt`/sidecars + `implement-log.md` record (evidence §3) |
| 12 | Harness idempotency | PASS | Distinct `mktemp` staging dirs, each with its own audit set; run count in `implement-log.md` (evidence §3) |
| 13 | Scope preservation | PASS | `dba-system.md`, `prompts/04,05,06`, `tools/reviewer/` all unchanged (git-verified in Step 4) |
| 14 | Scaffolded off | PASS | `config/delegated-implementation.yaml` = `status: disabled` |
| 15 | Pilot evidence recorded (token, gate check, net-token verdict) | PASS | Canonical pilot + `codeos-review.sh` gate (CHANGES ADVISED) + net-token verdict (evidence §3–5; Implementation Notes) |

**Consistency sweep (grep):** the change record's "What changes" matches the actual changed-file set
exactly (git: `backlog/features.md`, `status/roadmap.md`, `status/self-development.md` modified; brief,
this record, pilot-evidence, `config/delegated-implementation.yaml`, `prompts/codeos-implementer-task.md`,
`scripts/codeos-implement.sh` created). No stale placeholder tokens in either change artifact; candidate
path (`modules/counter.py`), token figure (~1.8K / evidence 1863), and quality language are consistent
across the change record and evidence file after the single-canonical-run regeneration. Reviewer
review-log/codex outputs are auto-generated bookkeeping, not part of "What changes".

**Findings scope-triage:**

| Finding | Triage | Action |
|---|---|---|
| S3 R1: pilot/fail-closed evidence not auditable in-packet; "9/9" unbacked | IN-SCOPE BLOCKER | Fixed — durable committed evidence file with embedded runner + verbatim outputs |
| S3 R1: strict source-or-test-path contract stated but not enforced | IN-SCOPE BLOCKER | Fixed — tool enforces the stage-area allowlist (`modules/` / `tests/`), matching prompt |
| S3 R2: AC-15 gate check not actually run | IN-SCOPE BLOCKER | Fixed — ran `codeos-review.sh review` on the candidate (CHANGES ADVISED, recorded) |
| S3 R2: Implementation Notes vs evidence count drift | IN-SCOPE BLOCKER | Fixed — drift-prone counts deferred to the evidence file |
| S3 R3: candidate path inconsistent across pilot runs | IN-SCOPE BLOCKER | Fixed — regenerated all evidence from one canonical run (`modules/counter.py`) |
| S3 R3: "gate-quality" overclaim vs CHANGES-ADVISED gate | IN-SCOPE BLOCKER | Fixed — honest "plausible draft"; gate verdict reported as-is |
| Gate-check findings on the pilot *candidate* (`correlation_id` arg; schema-invalid events) | REVIEW-BOOKKEEPING | Not a defect in CHG-A — they are the intended demonstration that the gate catches DeepSeek-draft issues; the candidate was never promoted |
| Deeper hardening surfaced beyond confirmatory R4 scope | OUT-OF-SCOPE BACKLOG | None material raised; any future improvements tracked under UPG-0060 CHG-B or a new follow-up |

---
