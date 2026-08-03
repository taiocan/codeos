# Self-Development Change: UPG-0060__CHG-20260803-001 — implementer-harness-correction

<!--
PURPOSE: Per-change source of truth for the harness correction that the CHG-B gate measurement
identified as a prerequisite to any further delegation comparison (condition 0 in the feature brief;
AJ-022 same-day amendment). Self-development toolkit change (prompt + script-tooling); no downstream
behavioral contract, event schema, or replay. Workflow: prompts/codeos-self-dev.md (4-step loop).
The live status row lives in status/self-development.md, not here.

This change corrects the delegation harness. It does NOT re-open CHG-B, does not enable the mechanism
anywhere, and does not by itself decide whether delegation is adopted — the re-test that follows it is
a separate measurement, judged on the three axes named in the feature brief.
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0060
primary_feature_id: UPG-0060
change_id: CHG-20260803-001
slug: implementer-harness-correction
state: COMPLETE         # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 4-Reconcile  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0060
related_features: [UPG-0056, UPG-0057]
review_series: RVS__UPG-0060__CHG-20260803-001__S4   # S1, S2, S3 ACCEPTED
review_profile: PROFILE-3   # prompt + script-tooling, same as CHG-A (Step 0a)
review_state: ACCEPTED  # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round)
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: CHG-20260802-001
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round. Reference the stable review SERIES (review_series) + review_state; exact rounds live
only in reviews/review-log.md and reviews/codex/*. -->

---

## Change Intent

**Why (problem in the toolkit):**

The CHG-B gate measurement (`changes/UPG-0060__CHG-B-GATE__realistic-feature-evidence.md`) ran a real
downstream feature through delegated Stage 4 and returned a negative result. Re-reading the harness
afterwards showed part of that result was caused by CHG-A's own packet, not by the delegate:

| Observed in the measurement | Harness cause |
|---|---|
| Candidate arrived with no build manifest and could not be built | `prompts/codeos-implementer-task.md` forbids it — *"Never emit a path that is not a source or test file"* and *"Add no … files … not traced to the approved artifacts."* `Cargo.toml` appears 0 times in the 105,510-byte packet. The **script already permits** `modules/<name>/Cargo.toml` under its stage-area allowlist; only the prompt blocks it |
| Module named against the project's convention | No layout exemplar was in the packet — the only `modules/` string in it comes from the prompt itself |
| The candidate delegated every contract invariant to its caller | The prompt says *"Add no behavior, no files, no abstractions … not traced to the approved artifacts,"* which reads as an instruction to omit the invariant-carrying structure the contract actually requires |
| Two `E0599` compile errors reached the human | Single shot, JSON-escaped source, no compiler feedback — all harness choices, all known to degrade generated code |

Until these are fixed, no comparison between delegate models is interpretable: a re-test that changes
the model while keeping the packet cannot separate model capability from harness handicap. That is why
the feature brief makes this **condition 0**, gating conditions 1-3.

**What changes:**

- `prompts/codeos-implementer-task.md` — **modified.**
  - Permit non-source files the target language requires in order to build (build manifest, module
    config), scoped to files required for the candidate to build, still inside the stage area.
  - Replace the blanket *"add no abstractions"* with *"no unnecessary capability; implement the
    abstractions the contract's explicit invariants require"* — the current wording pushes against
    exactly the structure a rigorous contract needs, while the intended constraint (do not invent
    capabilities the artifacts never asked for) is preserved.
  - Replace the single-JSON-object output contract with a delimited plain-text file protocol, so
    source is emitted as source rather than JSON string escapes.
  - Describe the layout exemplar section so the model knows the packet's exemplar is a convention to
    follow, not an artifact to implement.
- `scripts/codeos-implement.sh` — **modified.**
  - Accept and label a repository-layout exemplar distinctly from approved artifacts, so it is never
    mistaken for something to implement.
  - Parse the new delimited output protocol in place of `jq`-extracted JSON, keeping every existing
    safety property: stage-area allowlist, absolute-path and `..` rejection, `CANDIDATE_BLOCKED.md`
    escape hatch, key never in argv/body/packet/response/candidate, timestamped non-overwriting
    staging, full audit set, token instrumentation.
  - Accept a repair input (previous candidate + build/test output) so one bounded repair iteration can
    be run — under Option B (below), the tool consumes that output and never produces it.
  - Documented exit codes extended for the new failure modes, in the existing header-table style.
- `scripts/tests/codeos-implement-tests.sh` and `scripts/tests/stub-deepseek-server.py` — **new**,
  added at Step 3. Step 1 did not name them; Step 2's criteria call for fixture-driven verification
  (criteria 2, 7, 11, 12, 18) and this repository had no shell-test harness, so the vehicle for those
  criteria had to be built. Disclosed here rather than left for the Step 3 review to discover. They
  add no capability to the tool and are not invoked by it.
- `changes/UPG-0060__CHG-20260803-001__verification-evidence.md` — **new**, added at Step 3 R2. An
  in-repo record of the suite run and the mutation testing, carrying the verbatim output behind this
  change's factual pass claims rather than leaving them asserted (the AJ-016 / CHG-A R1-01 lesson).
  It is written into the repository and travels with this change; like every other file here it is
  uncommitted until the change closes. The claims it supports are reproducible by running the suite;
  the file records what that run produced, and a reader who reruns it should get the same result.
- `changes/UPG-0060__CHG-20260803-001__implementer-harness-correction.md` — **new**, this record.
- Lifecycle bookkeeping: `status/self-development.md` — the live row for this change, updated at each
  step. **Not in this change's remaining diff:** `status/roadmap.md` and
  `backlog/UPG-0060-deepseek-delegated-implementation.md` also carry rows for this change (and the
  brief's two stale abandonment sentences were corrected), but that bookkeeping was written during
  Step 1 and was committed in `6899e69` together with the accepted CHG-A work. Those files are
  therefore unmodified now and are context, not pending changes.

**Baseline note (resolved at Step 2/3 boundary).** Steps 1-2 were reviewed against a dirty worktree
that still carried the completed, accepted CHG-A work and the CHG-B gate record, which the Step 1
review correctly read as apparent scope drift — the AJ-017 pattern. That work was committed at
`6899e69` before Step 3 began, on the human's instruction, so this change now starts from a clean
baseline and its reviewed diff contains only its own files. `backlog/features.md` is **not** modified
by this change.

**Scope boundary — what stays the same:**

- **No re-opening of CHG-B.** No change to `dba-system.md`, any downstream stage prompt, or
  `scripts/dba-init.sh`. The feature stays PILOTED (negative) until a re-test clears all three axes.
- **The mechanism stays off by default everywhere.** `config/delegated-implementation.yaml` stays
  `status: disabled`; no downstream status file is scaffolded.
- **Every existing safety property is preserved, not renegotiated** — candidate staging only, never
  `modules/`/`tests/` in the real tree, never a commit, no key leakage, fail-closed preconditions,
  **and the tool continues to execute nothing on the local machine.** The repair iteration is
  therefore scoped to **option B below**: the tool accepts build output as *input*; it does not run a
  build. Options A and C are recorded as alternatives the human may substitute at this gate — if A is
  chosen, this bullet no longer holds and the scope boundary must be rewritten before Step 2.
- No change to `tools/reviewer/`; no DeepSeek provider added there.
- No new mandatory human-approval gate, no new Stage ID, no Non-Negotiable Rule change.
- This change makes no claim about whether delegation is net-positive. It only makes the next
  measurement interpretable.

**Class:** prompt + script-tooling
**Scope axis:** self-dev only
**Backlog item:** `backlog/UPG-0060-deepseek-delegated-implementation.md` (re-test condition 0)

---

## Design position taken, and the alternative the human may substitute

**This change takes option B: the tool accepts build output as input and never runs a build itself.**
That position is what the scope boundary above asserts, and it is what Step 2's acceptance criteria
will be written against. The alternatives are recorded here so the human can substitute one at this
gate; A in particular would require rewriting the scope boundary first, because it gives up a safety
property this change otherwise claims to preserve.

The repair iteration needs compiler output. Today the tool executes nothing local — it makes one
network call and writes files. Running `cargo build` (or any project build) on a downstream repository
means executing arbitrary project code: build scripts, proc macros, test harnesses. That is a
categorical change to what this tool is, which is why it is surfaced at the gate rather than decided
inside the implementation.

| Option | Effect |
|---|---|
| **A. Tool runs the build** | Fully automatic repair loop. Gives up the "executes nothing local" property and inherits arbitrary-code-execution risk from any downstream repo it is pointed at |
| **B. Tool accepts build output as input** *(recommended)* | The human or Claude runs the build and re-invokes with the errors. Preserves "executes nothing local," delivers the same feedback signal, costs a small amount of Claude token budget per iteration |
| **C. No repair iteration in this change** | Smallest change; leaves re-test condition 0 only partly satisfied, so the re-test still cannot show whether repair converges |

I recommend **B**. The feedback signal is what the measurement needs, and it does not cost the tool its
read-only-locally character — which is the same property that makes the Codex reviewer safe to point at
any repository. The token cost of running a build and re-invoking is small relative to the generation
it is meant to protect.

---

## Acceptance Criteria

<!-- The consistency contracts this change must satisfy; each checkable in Reconcile. prompt +
script-tooling: I/O behavior, exit-code / fail-closed cases, idempotency, secret non-leakage, plus
prompt-wording criteria that must be true of the text a model actually receives. -->

Terms: the **activation status file**, **staging directory**, and **stage area** carry the same
meanings as in `changes/UPG-0060__CHG-20260802-001__deepseek-implement-tool.md`. **Exemplar** means a
real file from the target repository supplied to show layout/naming convention. **Repair input** means
a previous candidate plus the build/test output it produced. **Option B** is the pinned design
position: the tool accepts build output as input and runs no build itself.

### Group 1 — the five corrections (why this change exists)

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | **Build manifests are permitted, and scoped.** The prompt no longer forbids non-source files outright; it permits exactly those files the target language needs for the candidate to build (build manifest, module config), still confined to the stage area. The old blanket sentence "Never emit a path that is not a source or test file" is gone. | Read the prompt; grep confirms the old sentence's absence and the new permission's presence. |
| 2 | **A manifest actually survives end to end.** A candidate containing `modules/<name>/Cargo.toml` is staged intact, unaltered, at that path. | Fixture run with a stubbed model response containing a manifest; assert the staged file exists byte-identical. |
| 3 | **A layout exemplar reaches the packet, labeled as convention.** The tool accepts exemplar paths and renders them in the packet under a section distinct from `--- APPROVED ARTIFACT: … ---`, so an exemplar can never be mistaken for something to implement. | Inspect a generated `packet.txt`: exemplar content present under its own distinct heading. |
| 4 | **The prompt explains the exemplar's role.** The prompt tells the model the exemplar shows layout and naming to follow, and is not a specification to implement. | Read the prompt. |
| 5 | **The anti-abstraction instruction is corrected, not merely softened.** No sentence forbids abstractions outright. The replacement preserves the real constraint (invent no capability the artifacts do not require) while requiring the abstractions the contract's explicit invariants demand. | Read the prompt; grep confirms no remaining "no abstractions"-style prohibition. |
| 6 | **The model no longer authors JSON.** The prompt's output contract is a plain-text delimited protocol, and `response_format: json_object` is gone from the request, so the model never hand-escapes source into a JSON string — which is what the original defect was. The provider's transport envelope is still JSON (`choices[0].message.content`) and is decoded losslessly by `jq -r`; that is the wire format, not something the model writes. | Read the prompt and the request body; inspect a staged candidate for absence of the escape artifacts (`\n`, `\"`) that model-authored JSON produced. |
| 7 | **The delimited protocol is not corruptible by content.** A candidate whose file content contains a line resembling the delimiter either parses correctly or fails closed with a distinct exit code — it never silently truncates a file, misattributes content, or writes a partial candidate. | Fixture run with a stubbed response whose content embeds a delimiter-lookalike line; assert correct parse or clean fail-closed, and assert no partial file is left staged. |
| 8 | **Repair input is accepted and labeled as feedback.** The tool accepts a previous candidate plus its build/test output and renders both in the packet under a section that identifies them as feedback on a prior attempt, distinct from approved artifacts and from the exemplar. | Inspect a generated `packet.txt` from a repair invocation. |
| 9 | **Option B holds: the tool runs no build.** `scripts/codeos-implement.sh` invokes no build, test, compile, or package-manager command, never `eval`s, and never shells out to a project-supplied command. Its external processes are exactly: `git`, `curl`, `jq`, `awk`, `sed`, and the coreutils `cat`, `tr`, `od`, `head`, `date`, `mkdir`, `mktemp`, `rmdir`, `dirname`. | Read-through plus an automated allowlist scan of the script (any external tool outside the documented set fails the test); a repair run observed to issue no build. |

### Group 2 — every CHG-A property preserved (regression)

| # | Criterion | How it will be verified |
|---|---|---|
| 10 | **All CHG-A fail-closed cases still hold at their documented exit codes** — non-git (1), missing dependency (2), usage/stage ∉ {4,5} (3), disabled or absent status (4), malformed status (5), missing key before any network call (6), missing artifact (7). | Re-run the CHG-A fail-closed suite unchanged; assert the same codes. |
| 11 | **Stage-area allowlist and path safety still enforced, manifests included.** Absolute paths, `..` traversal, and any path outside the stage area are still rejected — the manifest permission widens *file kind*, never *location*. | Fixture runs with an absolute path, a `..` path, and an in-stage-area manifest; assert reject / reject / accept. |
| 12 | **The `CANDIDATE_BLOCKED.md` escape hatch still works** under the new protocol. | Fixture run with a blocked response; assert the file stages and the run reports it. |
| 13 | **Secret non-leakage.** `DEEPSEEK_API_KEY` appears in no packet, request, response, candidate, repair input echo, or log — and is still absent from argv and the request body. | `grep -rF "$DEEPSEEK_API_KEY"` over the whole staging tree and log after a run. |
| 14 | **Write-safety unchanged.** Nothing is written under `modules/` or `tests/` in the real tree; no `git add`/`git commit` is run. | `git status` after a run; grep the script for `git add`/`git commit`. |
| 15 | **Idempotency unchanged.** Re-running for the same feature/stage produces a distinct, non-overwriting, timestamped audit directory. | Run twice; assert two distinct staging dirs each with a full audit set. |
| 16 | **Auditability and token instrumentation survive the protocol change.** Each run still preserves the exact packet sent and the raw response, and still records prompt/completion/total token counts and one `implement-log.md` record. | Inspect the audit set after a run. |
| 17 | **Still off by default.** `config/delegated-implementation.yaml` remains exactly `status: disabled`; no downstream status file is created or scaffolded by this change. | Read the file; `git diff --stat` shows no `scripts/dba-init.sh` change. |
| 18 | **New failure modes get distinct, documented exit codes** in the header table, in CHG-A's style — at minimum: exemplar path missing, repair-input path missing, unparseable candidate under the new protocol. | Read the header table; trigger each case and assert the documented code. |

### Group 3 — scope preservation

| # | Criterion | How it will be verified |
|---|---|---|
| 19 | **CHG-B is not re-opened and no doctrine moves.** `dba-system.md`, `prompts/04-implement.md`, `prompts/05-tests.md`, `prompts/06-observe.md`, `scripts/dba-init.sh`, and `tools/reviewer/` are byte-unchanged. The feature's state stays PILOTED (negative). | `git diff --stat` over the change; read the feature brief's state. |
| 20 | **This change claims no measurement result.** No artifact in this change asserts that delegation is or will be net-positive; the re-test remains a separate, later measurement judged on the brief's three axes. | Read the change record and the prompt. |

**Explicitly not in scope for these criteria:** whether the corrected harness improves candidate
quality. That is the re-test's question, not this change's. Criterion 20 exists to keep the two from
being conflated.

---

## Implementation Notes

<!-- Factual reporting. The git diff is the source of truth. -->

**The output protocol.** The JSON object is replaced by a **nonce-delimited plain-text frame**. The
tool mints a fresh 16-hex-character nonce per run (`/dev/urandom`), states it in the request as
`output_nonce`, and accepts a marker line only when it carries that exact value:

```
<<<CODEOS:<nonce>:FILE:relative/path>>>   …content verbatim…   <<<CODEOS:<nonce>:ENDFILE>>>
<<<CODEOS:<nonce>:SECTION:notes>>>        …               …   <<<CODEOS:<nonce>:ENDSECTION>>>
```

A per-run random nonce is what makes criterion 7 satisfiable rather than merely hoped for: candidate
content cannot collide with a marker by accident, because the model would have to reproduce a value
generated milliseconds earlier.

`response_format: json_object` is gone from the request, so **the model no longer authors JSON**. To
be exact about what did and did not change: the provider's transport envelope is still JSON, and the
reply is still read out of `choices[0].message.content` with `jq -r`. That decoding is mechanical and
lossless. The defect this fixes was never the wire format — it was requiring the *model* to hand-escape
newlines and quotes into a JSON string literal while also writing correct source.

**All-or-nothing staging.** Parsing is deliberately two passes. Pass 1 (awk) validates the entire
frame and emits a manifest of `kind / name / start-line / end-line`; pass 2 writes. Path safety is
checked against the manifest *between* the passes. Nothing reaches `candidate/` until the whole
response is known to be well formed and every path is known to be legal, so no malformed frame and no
illegal path can leave a partial, truncated, or misattributed candidate staged. Pass 1 rejects:
a `FILE`/`SECTION` marker inside an open block, an `ENDFILE`/`ENDSECTION` with no open block, an
unterminated block at EOF, a duplicate path, a duplicate or unknown section name, and an empty path.

**Exit codes.** 8 narrows to API/transport errors and unsafe candidate paths (its CHG-A meaning for
paths is preserved). Added: **9** exemplar path missing, **10** repair-input path missing, **11**
output-protocol violation. All are in the header table in CHG-A's style.

**Option B is a property of the script, not a promise.** The tool's external processes are exactly
`git`, `curl`, `jq`, `awk`, `sed`, and the coreutils `cat`, `tr`, `od`, `head`, `date`, `mkdir`,
`mktemp`, `rmdir`, `dirname` — `awk` runs the frame parser and `sed` extracts line ranges, both on the
model's own reply. It runs no build, test, compile, or package-manager command, never `eval`s, and
never shells out to anything project-supplied. Build output is an *input* the caller obtains as an
explicit external step (`--repair-output`).

**Manifest permission widens file kind, not location.** The prompt now requires the model to emit
whatever the target language needs to build. The script needed no change for this — its allowlist
already constrained *where* a file may go, never *what kind* it is, so `modules/<name>/Cargo.toml`
was always permitted and only the prompt was blocking it. Criterion 11's three path tests confirm the
location constraint still holds.

**Findings addressed (Step 3 R1).** Both were overclaims in this record, not defects in the tool, and
both were corrected by making the claim true rather than by softening it:

- **R1-01 (High) — criterion 6 claimed no content passes through JSON escaping "at any point" / "in
  either direction."** False: the provider's transport envelope is still JSON and `jq -r` decodes it.
  Criterion 6 and the notes now state the real change — the *model* no longer authors JSON — and name
  the transport explicitly. The substantive improvement is unaffected; the claim was simply wider
  than the work.
- **R1-02 (Medium) — criterion 9 listed a process set that omitted `awk` and `sed`.** Also false: the
  frame parser is awk and line extraction is sed. Both criterion 9 and the notes now carry the exact
  set, and a new allowlist scan in the test suite fails if any external tool outside the documented
  set appears — so this drift is caught by a test next time instead of by a reviewer.

**Verification.** `scripts/tests/codeos-implement-tests.sh` — **33 tests, all passing**, no network
and no API spend. The verbatim run output, the verbatim mutation-testing output, and a
criterion-to-test coverage map are in
`changes/UPG-0060__CHG-20260803-001__verification-evidence.md`. `stub-deepseek-server.py` stands in for the endpoint and recovers the run's own
nonce from the posted body, so the nonce round trip is genuinely exercised rather than replayed.

**The suite was mutation-tested before being trusted.** Four guards were deliberately broken, one at
a time, each failing exactly the expected test with the rest still green: the nested-marker check
(`C7b`), the stage-area allowlist (`C11 outside-stage-area`), the documented-process allowlist (`C9
undocumented external tool`), and the absolute/traversal rejection (`C11 traversal`). The script was
restored byte-identically after each (`diff -q` clean). A green suite that has never been shown to
fail is not evidence. Verbatim output in the evidence file, including why mutation 4 trips the
traversal case but not the absolute one — absolute paths have a second, independent defense.

**Secret non-leakage is deterministic, per the Step 3 instruction.** The earlier ad-hoc check was
faulty — `if grep … | head` tests the *last* command's status, so it reported a match unconditionally.
The suite's assertion avoids that class of error two ways: it tests `grep -rqF`'s own exit status
directly with no pipeline, and it runs a **positive control** first — a planted key that the checker
must detect before its report of absence is trusted. A checker that cannot find a key it was just
handed fails the test rather than silently passing.

**Assumptions:**
- `python3` is available for the test stub (tests only — the tool itself does not use it).
- The stub binds `127.0.0.1:8931`; override with `CODEOS_STUB_PORT` if that port is busy.

**Out-of-scope items filed:** none. No `dba-system.md`, downstream stage prompt, `scripts/dba-init.sh`,
or `tools/reviewer/` change was made; `config/delegated-implementation.yaml` remains `status: disabled`.

---

## Reconciliation

<!-- Layer D1: advisory verdict, evidence separated from inference. -->

**Acceptance verification.** All 20 criteria PASS. Runtime criteria are verified by
`scripts/tests/codeos-implement-tests.sh` (33 tests, verbatim output and criterion-to-test map in
`changes/UPG-0060__CHG-20260803-001__verification-evidence.md`); text criteria by reading the prompt
and the diff. Step 3 R6 independently assessed criteria 1-20 as supported with **zero findings**.

| Group | Criteria | Result | Evidence |
|---|---|---|---|
| Corrections | 1, 4, 5 (prompt wording: manifests permitted, exemplar role, abstraction rule corrected) | PASS | `prompts/codeos-implementer-task.md`; no "no abstractions" prohibition remains |
| Corrections | 2, 3, 6, 7, 8, 9 | PASS | `C2`, `C3`, `C6`, `C7a`-`C7e`, `C8`, `C9`×2 — evidence §1, §3 |
| Regression | 10-18 | PASS | `C10`×9 (exits 1,3,3,4,4,5,6,7,8), `C11`×3, `C12`, `C13`×2, `C14`, `C15`, `C16`, `C17`, `C18`×3 |
| Scope | 19, 20 | PASS | `dba-system.md`, `prompts/04/05/06-*.md`, `scripts/dba-init.sh`, `tools/reviewer/` all unchanged; `config/delegated-implementation.yaml` = `status: disabled`; no measurement result claimed anywhere |

**Findings scope-triage.**

| Finding | Triage | Action |
|---|---|---|
| S3 R1-01/02, R2-01/02/03, R3-01/02, R5-01 — eight in-scope blockers | IN-SCOPE BLOCKER | All fixed. Every one was a false or unsupported claim in *this record*, the script header, or the packet — **none was a defect in the tool**. The implementation passed its criteria from the first run; six rounds were spent on artifact governance |
| S3 R6 — `SECRET_REDACTION` coverage bars NO OBJECTION **in any packet containing the test suite** (Step 3's packets did; this Step 4 packet does not, and reports FULL_COVERAGE) | REJECTED / structural limitation | Accepted, not worked around. The suite must name `DEEPSEEK_API_KEY` to test a tool that reads it, and the redactor keys on that name. Two lines are hidden (a dummy canary and an intentional empty assignment); no test logic or output is redacted. Obfuscating the name to satisfy a scanner was rejected as a worse practice. Precedent: UPG-0037, UPG-0039 |
| S4 R1 — reconcile text read as claiming the *current* packet is redacted | SELF-REFERENCE / REVIEW-BOOKKEEPING | Scoped the sentence to packets containing the suite. Not a defect in the change; the ambiguity exists only because this record is itself reviewed across packets with different file sets. Fixed once, not re-reviewed — per the human's Step 4 instruction to stop iterating on artifact governance |
| Review effort was disproportionate to defect severity | OUT-OF-SCOPE BACKLOG | Filed as **UPG-0061**. The reviewer weights behavioral defects, stale comments, evidence wording, and packet omissions alike; eight "blockers" here were overwhelmingly record hygiene. Not fixed inside this change |

**Honest assessment of this change.** The harness is materially better and the corrections are real. But
the round count reflects premature hardening: this integration was treated as production
infrastructure before the experiment justifying its existence had been rerun. The corrected harness is
worth exactly as much as the next measurement says it is, and nothing more.
