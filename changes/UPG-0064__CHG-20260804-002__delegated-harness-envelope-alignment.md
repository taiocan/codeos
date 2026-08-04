# Self-Development Change: UPG-0064__CHG-20260804-002 — delegated-harness-envelope-alignment

<!--
PURPOSE: CHG-A of UPG-0064. Makes the governed Stage-4 envelope (Architecture Baseline, Cohort
Logical Design, Implementation Profile, and UPG-0063's deferral rule) visible and binding to the
delegated implementer. Harness alignment only — no pilot, no measurement, no adoption claim. The
three-case pilot is CHG-B and does not begin until this is accepted.
Workflow: prompts/codeos-self-dev.md (4-step loop).
-->

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0064
primary_feature_id: UPG-0064
change_id: CHG-20260804-002
slug: delegated-harness-envelope-alignment
state: COMPLETE         # DRAFT | IN_REVIEW | IN_PROGRESS | BLOCKED | COMPLETE | ABANDONED | SUPERSEDED
current_step: 4-Reconcile  # 1-Intent | 2-Acceptance | 3-Implement | 4-Reconcile
implements:
  - UPG-0064
related_features: [UPG-0051, UPG-0052, UPG-0063, UPG-0060, UPG-0062]
review_series: RVS__UPG-0064__CHG-20260804-002__S4   # S1, S2, S3 ACCEPTED
review_profile: PROFILE-3   # prompt + script-tooling, self-dev only (Step 0a)
review_state: ACCEPTED  # DRAFT | IN_REVIEW | REVIEWED | ACCEPTED  (operational; NOT a round)
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round. Reference the stable review SERIES (review_series) + review_state; exact rounds live
only in reviews/review-log.md and reviews/codex/*. -->

---

## Change Intent

**Why (problem in the toolkit):**

UPG-0051, UPG-0052 and UPG-0063 built a governed envelope around Stage 4. The delegated
implementation path never received it. Measured against `prompts/codeos-implementer-task.md` — the
only prompt the delegate sees — "Architecture Baseline", "Implementation Profile", "Cohort" and
"deferral/deferred" each occur **zero** times. Its output contract asks for three loose sections and
had no notion of a deferral trace. `scripts/codeos-implement.sh` labelled every input identically as
`--- APPROVED ARTIFACT: <path> ---`, so a behavioral contract and an Architecture Baseline were
indistinguishable to the model. *(Past tense as of Step 3 — this is the state this change corrected;
see Implementation Notes for what replaced it.)*

This is an **integration defect in the delegation harness**, not another architecture-design problem.
The envelope is complete; the delegated execution path was never updated to carry it. UPG-0062 planned
the prompt rewrite that would have fixed part of it and closed on cost before Step 3.

**Why it must be fixed before the pilot.** The Stage-4 reviewer checklist now asks for the deferral
trace. A pilot today would hand the delegate a feature containing an explicit deferral, never tell it
deferrals exist or that resolving one incurs an obligation, then measure whether it recorded one — a
harness defect reported as a model defect. That is precisely UPG-0060's documented error, and
repeating it against our own written correction would be worse than making it the first time.

**What changes:**

- `scripts/codeos-implement.sh` — **modified.** Artifacts are labelled by **authority role** instead
  of the flat `APPROVED ARTIFACT`: behavioral contract, event schema, architecture baseline, cohort
  logical design, implementation profile, layout exemplar. The label states how each input binds. Role
  is supplied by the caller, not inferred from filenames — the tool must not guess an artifact's
  authority from a path.
- `prompts/codeos-implementer-task.md` — **modified.** States each role's authority: contract =
  behavior to satisfy; event schema = events to emit correctly; baseline and cohort logical design =
  binding architectural constraints, **not** behavior to invent; implementation profile = binding
  implementation constraint; exemplar = context, not authority. Adds UPG-0063's rule semantically —
  report a resolution only when an approved artifact **explicitly deferred** a material decision,
  using the five fields; exclude ordinary technique choices and matters merely unspecified rather than
  deferred; no phrase list is normative. Adds a `deferral_resolution` output section.
- `scripts/tests/codeos-implement-tests.sh` — **modified.** Coverage for role labelling and the new
  output section, including its absence when nothing was deferred.
- `changes/UPG-0064__CHG-20260804-002__delegated-harness-envelope-alignment.md` — **new**, this record.
- Lifecycle bookkeeping: `backlog/features.md`, `status/self-development.md`, `status/roadmap.md`.

**Scope boundary — what stays the same:**

- **No pilot, no measurement, no adoption claim.** CHG-A ships harness alignment only. CHG-B is the
  three-case pilot and does not begin until this is accepted.
- **The delegate produces a candidate, never the authoritative Stage-4 report.** It returns code and
  evidence; Codeos/Claude assembles the Review Package. Making the delegate emit the canonical
  artifact would change the experiment from "can it satisfy an approved envelope?" to "can it also
  operate Codeos's governance protocol?" — different questions, different failure modes.
- **Stage 5 delegation is out**, and does not inherit any Stage-4 result. A model can implement a
  contract reasonably while writing tests that confirm its own interpretation rather than falsify the
  contract. Separate experiment, only on positive Stage-4 evidence.
- **The mechanism stays off by default** — `config/delegated-implementation.yaml` remains
  `status: disabled`; no downstream status file is scaffolded.
- Every existing safety property is preserved: candidate staging only, never `modules/`/`tests/` in
  the real tree, never a commit, no key leakage, fail-closed preconditions, and **the tool executes no
  build, test, or project-supplied command** (UPG-0060's Option B boundary).
- **No downstream doctrine change.** `dba-system.md`, `prompts/04-implement.md`, and every other stage
  prompt are untouched — UPG-0063's Stage-4 output format is *consumed* here, never redefined.
- No change to `tools/reviewer/`. No new stage, gate, Stage ID, or Non-Negotiable Rule change.
- **UPG-0060's conclusion is not revisited.** That the delegate cannot *derive* an architecture from a
  contract stands. This asks whether it can operate inside one that is supplied and labelled.

**Class:** prompt + script-tooling
**Scope axis:** self-dev only
**Backlog item:** `backlog/UPG-0064-delegated-stage4-envelope-alignment.md`

---

## Open question for the gate

**How does the tool know an artifact's role?**

Roles cannot be inferred from filenames — `architecture/core-baseline.md` is conventional, not
guaranteed, and a downstream project may name things differently. Guessing would put the tool in the
business of classifying authority, which is exactly what it must not do.

| Option | |
|---|---|
| **A. Explicit per-artifact flags** *(recommended)* — `--contract PATH`, `--event-schema PATH`, `--architecture PATH`, `--cohort-design PATH`, `--profile PATH`, existing `--exemplar PATH` | The caller declares authority; the tool transports it. Verbose at the call site, unambiguous, and no inference |
| **B. Infer from path conventions** | Concise, but the tool would be deciding what is architecturally binding — wrong actor, and silently wrong on any project that names files differently |
| **C. A manifest file** | Another artifact to keep in sync; disproportionate for six inputs |

I recommend **A**, and note it makes the existing bare positional `<artifact-path>` form ambiguous —
Step 2 must decide whether unlabelled positionals stay supported (as "contract-or-schema, unspecified")
or become an error. My inclination is to keep them working and label them
`--- APPROVED ARTIFACT (role unspecified) ---`, so the caller sees the degradation rather than the
model silently receiving a mislabelled authority.

---

## Step 1 gate — decisions carried into Step 2

**Role resolution: option A, explicit per-artifact flags** (human, 2026-08-04). The caller declares
artifact authority; the tool transports it. Named flags rather than a generic `--artifact-role` —
the vocabulary is small and closed, and the CLI stays self-documenting:

`--contract` · `--event-schema` · `--architecture` · `--cohort-design` · `--profile` · `--exemplar`

**The tool infers authority from nothing** — not path, filename, content, headings, or directory.

**Positionals stay supported, with a semantic rule:** a positional artifact is *role unspecified* and
**must never silently satisfy any role-specific requirement**. Labelled
`--- APPROVED ARTIFACT (ROLE UNSPECIFIED): <path> ---`, and the prompt states the consequence. This
fixes the failure direction: a missing role declaration degrades authority **visibly**, rather than
the tool guessing.

**CHG-B may not use the compatibility path.** The pilot must declare every governed artifact with an
explicit role flag — otherwise we could align the harness correctly and then accidentally test the
delegate through the degraded path.

---

## Acceptance Criteria

<!-- prompt + script-tooling. These criteria were DEFINED at Step 2 and VERIFIED at Step 4 — see the
Reconciliation section for results and the embedded raw output. Verification method: reading the
changed text, plus fixture runs against the existing stub endpoint (no network, no API spend). -->

### Group 1 — caller-declared roles

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | **Six role flags exist**, each accepting a path: `--contract`, `--event-schema`, `--architecture`, `--cohort-design`, `--profile`, `--exemplar`. The vocabulary is closed — no generic role flag. | `--help`/usage text; fixture invocation of each. |
| 2 | **Each declared artifact is labelled with its role** in the packet, stating how it binds — contract = behavior to satisfy; event schema = events to emit; architecture baseline and cohort logical design = binding architectural constraints, not behavior to invent; implementation profile = binding implementation constraint; exemplar = context, not authority. | Inspect a generated `packet.txt`; each role's heading present and distinct. |
| 3 | **The tool infers authority from nothing.** No role is derived from path, filename, content, headings, or directory. | Read-through of the script; a fixture placing a file at `architecture/core-baseline.md` and passing it *positionally* is labelled ROLE UNSPECIFIED, not ARCHITECTURE BASELINE. |

### Group 2 — backward compatibility, degraded visibly

| # | Criterion | How it will be verified |
|---|---|---|
| 4 | **Positionals still work.** The pre-existing `<feature> <stage> <artifact-path>…` form runs and stages a candidate. | Re-run the existing suite's positional invocations unchanged. |
| 5 | **A positional is labelled `--- APPROVED ARTIFACT (ROLE UNSPECIFIED): <path> ---`** — visibly degraded, never silently promoted to a role. | Inspect `packet.txt` from a positional invocation. |
| 6 | **The prompt states the consequence:** role-unspecified artifacts may provide supporting context, and do not replace a Behavioral Contract, Event Schema, Architecture Baseline, Cohort Logical Design, or Implementation Profile when that role has been declared separately. | Read the prompt. |

### Group 3 — mechanical properties (pinned, no larger abstraction)

| # | Criterion | How it will be verified |
|---|---|---|
| 7 | **One path cannot acquire two conflicting authority roles.** The same artifact passed under two different role flags fails closed with a distinct, documented exit code, before any network call. | Fixture: same path via `--contract` and `--architecture`; assert the exit code and that no packet was sent. |
| 8 | **Role labels survive unchanged into the exact prompt sent.** The label text in `packet.txt` is byte-identical to what appears in the request body actually transmitted. | Compare the labels in `packet.txt` against `request.json`'s user message after a fixture run. |
| 9 | **Role labelling changes no artifact contents.** Each role block is a heading line, one generated binding-note line, then **the source file's bytes, unmodified**. The artifact-content region — from after the note line to the next heading, less the single blank line that separates blocks — is byte-identical to the file on disk. *(Corrected at S4 R1: an earlier wording claimed the whole region between headings equals the file, which is false because of the note line. The guarantee is about artifact content, not about the block containing nothing else.)* | Fixture run; extract the content region and byte-compare against the source file. |

### Group 4 — the prompt

| # | Criterion | How it will be verified |
|---|---|---|
| 10 | **UPG-0063's deferral rule is imported semantically**, with both exclusions: ordinary implementation technique choices, and matters merely *unspecified* rather than *explicitly deferred*. **No phrase list is normative.** | Read the prompt; grep confirms no phrase list presented as definitional. |
| 11 | **A `deferral_resolution` output section exists**, carrying UPG-0063's five fields, and is **omitted entirely** when nothing was deferred — no empty table, no "none". | Read the output contract; fixture responses with and without the section both parse. |
| 12 | **The delegate is told it produces a candidate, not the authoritative Stage-4 report.** | Read the prompt. |

### Group 5 — every existing property preserved (regression)

| # | Criterion | How it will be verified |
|---|---|---|
| 13 | **All existing fail-closed cases hold at their documented exit codes**; new failure modes get distinct, documented codes in the header table. | Re-run the existing fail-closed suite; trigger the new cases. |
| 14 | **Secret non-leakage, write-safety, idempotency, and the full audit set are unchanged**; the mechanism stays `status: disabled`. | Existing suite (positive-control-gated secret check included). |
| 15 | **Option B holds — the tool runs no build, test, or project-supplied command**, and its external-process allowlist is unchanged or updated together with the header. | The suite's allowlist scan. |

### Group 6 — scope, and the constraint on CHG-B

| # | Criterion | How it will be verified |
|---|---|---|
| 16 | **No pilot, no measurement, no adoption claim in this change.** No delegated run against a real feature; nothing asserts the delegate performs better. | Read the change record; `git diff --stat` shows no pilot evidence file. |
| 17 | **The constraint on CHG-B is recorded as a binding precondition** — that the pilot must declare every governed artifact with an explicit role flag, positionals being compatibility-only, so the experiment cannot run through the degraded path this change exists to remove. *CHG-A is verified by the constraint being **written down**, not by CHG-B obeying it: this change cannot be accepted on evidence that only exists after it completes.* CHG-B's own Step 2 owns compliance. | The precondition appears in `backlog/UPG-0064-…md` and in this record's scope boundary. |
| 18 | **No downstream doctrine change, no reviewer change.** `dba-system.md`, all stage prompts including `prompts/04-implement.md`, and `tools/reviewer/` byte-unchanged. | `git diff --stat`. |

**Explicitly not in scope for these criteria:** whether the delegate performs better with the aligned
harness. That is CHG-B's question and cannot be answered here — by criterion 16 there is no run to
answer it with.

---

## Implementation Notes

<!-- Factual reporting. The git diff is the source of truth. -->

**Script — five role flags, no inference.** `--contract`, `--event-schema`, `--architecture`,
`--cohort-design` added alongside the existing `--exemplar`, plus `--profile`. One flat array per
role: deliberately explicit rather than a generic role table, since the value here is that a reader
can see exactly which roles exist. The packet emits declared roles first, in fixed order, each with a
heading naming the authority it carries and a one-line note on how it binds; positionals follow as
`APPROVED ARTIFACT (ROLE UNSPECIFIED)` with their non-substitutability stated inline.

**Conflicting roles — exit 12**, before any network call, via a bash associative array. Same path
under the same role twice is not a conflict; under two different roles it is, and the tool refuses
rather than arbitrating which authority wins.

**The conflict check was rewritten to avoid widening the process allowlist.** My first version used
`grep` and `cut`, neither of which is in the documented external-tool list — the C9 allowlist scan
(added by UPG-0060 CHG-20260803-001 for exactly this) would have caught it, but rewriting in pure
bash was better than widening the list for a bookkeeping check.

**Prompt.** New "What the artifacts in this request mean" section: a binding table (contract, event
schema, architecture baseline, cohort logical design, implementation profile) and a non-authoritative
table (layout exemplar, role-unspecified). States that a binding constraint wins over the model's
preferred approach, and that disagreement goes to `notes` + `CANDIDATE_BLOCKED.md` rather than being
implemented around. Adds that the output is a *candidate*, not a Stage 4 report.

**`deferral_resolution` — optional, with the manufacture-pressure guarded explicitly.** UPG-0063's
rule is imported semantically with both exclusions (silence; implementation freedom) plus the
materiality gate. The section is introduced as *"Optional, and usually absent"*, and the prompt states
**"Most requests have no qualifying deferral. Omitting this section is the expected outcome and is
completely correct. Do not invent a deferral, stretch an ordinary choice to fit, or emit the section
empty, merely because it is named here. A fabricated entry is worse than none."**

The harness reinforces that: `deferral_resolution` is **excluded** from the uniform empty-sidecar
loop, so unlike the other three sidecars it is never created empty. Its presence in the staging
directory is itself the signal that the model reported one.

**Tests — 45 pass, up from 34.** Eleven added: five-role labelling; positional stays ROLE UNSPECIFIED
alongside declared roles; no inference from a conventional path; conflicting roles → 12 with nothing
staged; same-role duplicate is not a conflict; label byte-identical in `packet.txt` and the request
actually sent; content unmodified; missing role artifact → 7; deferral section parses when present;
absence is a clean success with no fabricated sidecar; and the prompt carries the anti-fabrication
wording.

**Mutation-verified.** Making the tool infer `ARCHITECTURE BASELINE` from a `*core-baseline*` path
failed exactly `ROLE inference`; removing the conflicting-role guard failed exactly `ROLE conflict`.
Restored byte-identically after each.

**A defect in my own tooling, not the change.** My first attempt at the test additions was written
through a non-raw Python triple-quoted string, where `\` + newline is a *Python* line continuation —
so every bash line-continuation was silently consumed and lines joined, producing a parse error 80
lines later. I initially masked it by filtering the suite output through `grep`, which hid the syntax
error while still showing the `ok` lines printed before bash reached it. Reverted and re-applied with
raw strings. The lesson is about the filtering, not the escaping: a filtered test run can look green
while the file does not parse.

**Assumptions:** bash 4+ for the associative array used by the conflict check — already required by
the existing suite and by `mapfile`-free array handling elsewhere in the script.

---

## Reconciliation

<!-- Layer D1: advisory verdict, evidence separated from inference. -->

**All 18 acceptance criteria PASS. Accepted by the human 2026-08-05**, after S4 R4 returned NO OBJECTION — in that order. Verified from the **full unfiltered** suite output with shell
syntax validation run first. Per the human's Step 4 instruction, filtered output is not used as
acceptance evidence — that instruction exists because Step 3 produced a false-green signal exactly
that way.

### Verification surface — proved, not assumed

| Check | Result |
|---|---|
| `bash -n scripts/codeos-implement.sh` | PARSES — run **before** the behavioral suite |
| `bash -n scripts/tests/codeos-implement-tests.sh` | PARSES — the check that would have caught Step 3's masked defect |
| Full suite, unfiltered, exit code captured | **exit 0 — 47 passed, 0 failed** |
| Executed artifact is current | `codeos-implement.sh` is bash executed directly by path (`TOOL="${CODEOS_ROOT}/scripts/codeos-implement.sh"`, line 14) — **no build step, so no stale-artifact class here**, unlike `tools/reviewer`'s compiled binary in UPG-0063. Working-tree sha256 **`12bda60e992d30f7`**, which is the file the suite executed. It differs from `HEAD` because S4 R1's fixes (role-flags-only invocation; byte-exact content test) are applied and not yet committed — that is expected mid-step. **Corrected at S4 R2**: an earlier draft recorded the pre-fix hash and claimed no uncommitted diff, which became false the moment Step 4 changed the script. The reviewer caught it, which is the verification-surface check working |

### Acceptance

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Six role flags, closed vocabulary | PASS | Usage text; `ROLE_*+=` assignments at lines 90/92/94/96/98 |
| 2 | Each declared artifact labelled with its role | PASS | `ROLE each declared artifact is labelled with its authority` — all five headings asserted |
| 3 | **No authority inference from anything** | PASS | No path pattern-matching exists in role assignment; role arrays are populated *only* by their flags. `ROLE no authority inferred from a conventional path` passes a file at `architecture/core-baseline.md` positionally and asserts it is **not** promoted. **Mutation-verified**: adding `*core-baseline*` inference fails exactly this test |
| 4 | Positionals still work, **and a role-flags-only call needs none** | PASS | `ROLE role-flags-only call needs no positional artifact`; `ROLE zero artifacts by any route -> exit 3`. **Fixed at S4 R1**: `$# -lt 3` required a positional artifact, so a fully role-declared call exited 3 — which made CHG-B's own precondition unsatisfiable. Now `$# -lt 2` (feature + stage), with a separate check that at least one artifact arrived by either route |
| 5 | Positional labelled ROLE UNSPECIFIED | PASS | `ROLE positional stays ROLE UNSPECIFIED alongside declared roles`; `C3` updated to the new label |
| 6 | Prompt states the consequence | PASS | Non-authoritative table: role-unspecified "does not replace a Behavioral Contract, Event Schema, Architecture Baseline, Cohort Logical Design, or Implementation Profile" |
| 7 | **One path, two roles → fail closed pre-network** | PASS | `ROLE conflicting roles on one path -> exit 12, nothing staged`. Ordering proved by line number: conflict check **200**, key check 216, `curl` **331** — the check cannot be reached after a request. **Mutation-verified** |
| 8 | Labels survive byte-identical into the request sent | PASS | `ROLE label byte-identical in packet.txt and the request actually sent` — compares `packet.txt` against `request.json`'s user message |
| 9 | Labelling changes no artifact content | PASS | `ROLE content region byte-identical to source file` — extracts the region after the binding note up to the next heading and `cmp`s it against the file. Wording corrected at S4 R1: the region between headings also contains the generated note, so the guarantee is about the *artifact-content* region, which is exact |
| 10 | Deferral rule semantic, both exclusions, no phrase list | PASS | Prompt carries silence + implementation-freedom exclusions and the materiality gate; no phrase list is definitional |
| 11 | `deferral_resolution` optional, omitted when absent | PASS | `DEFERRAL section parses and is staged when present`; `DEFERRAL absent is a clean success; no empty sidecar fabricated`. Reinforced structurally — the section is **excluded** from the uniform empty-sidecar loop (line 453), so its presence is itself the signal |
| 12 | Delegate told it produces a candidate | PASS | "What you produce is a candidate, not a Stage 4 report" |
| 13 | Existing fail-closed codes hold; new ones documented | PASS | `C10` ×9 and `C18` ×3 all pass; exit 12 added to the header table |
| 14 | Secret non-leakage, write-safety, idempotency, audit set, off-by-default | PASS | `C13` ×2 (positive-control-gated), `C14`, `C15`, `C16`, `C17` |
| 15 | Option B — no build/test command; allowlist intact | PASS | `C9` ×2. The allowlist scan **caught a real drift during Step 3** — my first conflict check used `grep`/`cut`; rewritten in pure bash rather than widening the list |
| 16 | No pilot, no measurement, no adoption claim | PASS | No delegated run against a real feature; no evidence file claiming delegate performance |
| 17 | CHG-B precondition recorded (not CHG-B compliance) | PASS | Stated in the brief's CHG-B section and this record's scope boundary. De-circularised at S2 R1 — CHG-A cannot be accepted on post-CHG-B evidence |
| 18 | No downstream doctrine or reviewer change | PASS | `dba-system.md`, all stage prompts including `prompts/04-implement.md`, and `tools/reviewer/` byte-unchanged |

### Raw verification output

Embedded rather than summarised. A prose claim of "47 tests pass" is not the same evidence as the run
that produced it (AJ-016), and Step 3 of this very change produced a false-green from filtered output.

Syntax validation, run **before** the behavioural suite:

```
scripts/codeos-implement.sh : PARSES
scripts/tests/codeos-implement-tests.sh : PARSES
```

Full suite, unfiltered, against working-tree `codeos-implement.sh` sha256 `12bda60e992d30f7`:

```
== Group 1: the five corrections ==
  ok   C2 manifest staged byte-identical
  ok   C6 source emitted verbatim (no JSON escape artifacts)
  ok   C16 audit set complete + tokens recorded
  ok   C13 secret absent from staging tree (checker passed positive control)
  ok   C13 secret absent from invocation log
  ok   C3 exemplar labeled distinctly from approved artifacts
  ok   C8 repair input rendered as feedback
  ok   C9 script invokes no build/test/package-manager command
  ok   C9 no external tool outside the documented allowlist
  ok   REG packet of 288001 bytes (>128 KiB) builds and runs
== UPG-0064: artifact authority roles ==
  ok   ROLE each declared artifact is labelled with its authority
  ok   ROLE positional stays ROLE UNSPECIFIED alongside declared roles
  ok   ROLE no authority inferred from a conventional path
  ok   ROLE conflicting roles on one path -> exit 12, nothing staged
  ok   ROLE same path twice under one role is not a conflict
  ok   ROLE label byte-identical in packet.txt and the request actually sent
  ok   ROLE content region byte-identical to source file
  ok   ROLE missing role artifact -> exit 7
  ok   DEFERRAL section parses and is staged when present
  ok   DEFERRAL absent is a clean success; no empty sidecar fabricated
  ok   DEFERRAL prompt states absence is expected and forbids fabrication
  ok   ROLE role-flags-only call needs no positional artifact
  ok   ROLE zero artifacts by any route -> exit 3
== Group 1: protocol robustness (criterion 7) ==
  ok   C7a wrong-nonce marker treated as content, file not truncated
  ok   C7b nested marker -> exit 11, nothing staged
  ok   C7c unterminated block -> exit 11, nothing staged
  ok   C7d duplicate path -> exit 11, nothing staged
  ok   C7e no file blocks -> exit 11
== Group 2: preserved CHG-A properties ==
  ok   C11 absolute path rejected (exit 8), nothing staged
  ok   C11 traversal path rejected (exit 8), nothing staged
  ok   C11 outside-stage-area path rejected (exit 8), nothing staged
  ok   C12 CANDIDATE_BLOCKED.md escape hatch works
  ok   C15 two runs -> two distinct staging dirs
  ok   C14 nothing written under modules/ or tests/, no commit
  ok   C10 non-git dir -> exit 1
  ok   C10 missing args -> exit 3
  ok   C10 stage=6 -> exit 3
  ok   C10 status:disabled -> exit 4
  ok   C10 status file absent -> exit 4
  ok   C10 malformed status -> exit 5
  ok   C10 unset key -> exit 6 (pre-network)
  ok   C10 missing artifact -> exit 7
== Group 2: new exit codes (criterion 18) ==
  ok   C18 missing exemplar -> exit 9
  ok   C18 missing repair input -> exit 10
  ok   C18 unknown option -> exit 3
  ok   C10 HTTP 500 -> exit 8
  ok   C17 self-dev activation file still status: disabled

47 passed, 0 failed
```

exit code: **0**

### Stale-reference sweep

Swept for references to the superseded positional-only behaviour. Live (non-`reviews/`) results:

- **Two corrected**: this record and the feature brief described the flat `APPROVED ARTIFACT` label in
  the **present tense** in their "Why" sections — true when written, false once Step 3 landed. Both
  now read in past tense with an explicit note that this is the pre-change state.
- **One left standing**: `changes/UPG-0060__CHG-20260803-001__implementer-harness-correction.md:176`,
  an acceptance criterion of a **COMPLETE** change. It records what was verified at the time and is
  not rewritten — the same reasoning applied to `reviews/codex/*`, whose many hits are immutable
  historical assessments, not stale references to repair.

### Findings scope-triage

| Finding | Triage | Action |
|---|---|---|
| S1 R1: record header `DRAFT` while dashboard said `IN_PROGRESS` | IN-SCOPE BLOCKER | Fixed. AJ-020 applied to the dashboard but not the record |
| S2 R1: **AC-17 circular** — CHG-A acceptance depended on post-CHG-B evidence | IN-SCOPE BLOCKER | Fixed. A genuine logic error, not bookkeeping: the criterion made the stage unsatisfiable |
| S2 R1/R2: state stale in `features.md` and the brief's YAML front matter | IN-SCOPE BLOCKER | Fixed by enumerating all five state-carrying locations at once |
| S2 R2: past-tense verification claim in a section that only *defines* criteria | IN-SCOPE BLOCKER | Reworded |
| Step 3: allowlist drift (`grep`/`cut` in the conflict check) | IN-SCOPE BLOCKER (self-found, caught by the suite) | Rewritten in pure bash |
| S4 R1: **role-flags-only invocation exited 3** | IN-SCOPE BLOCKER | Fixed. A real functional defect, not bookkeeping: the tool could not be driven the way CHG-B is required to drive it, so the precondition recorded at S2 was unsatisfiable by the implementation meant to enable it. Caught by the reviewer, not by my 45 tests — none of them called the tool without a positional |
| S4 R1: AC-9's byte-preservation wording was false | IN-SCOPE BLOCKER | Fixed. The region between headings also contains the generated binding note and one separator newline; the criterion now states the guarantee precisely and the test `cmp`s the exact content region rather than asserting the bytes appear somewhere |
| Step 3: **false-green from filtered output** | VERIFICATION-PROCESS FINDING | Not a behavioural defect. Step 4 verifies from unfiltered output with syntax validation first, per the human's instruction. Step 3 is not reopened |

**Honest assessment.** The change is small and its guarantees are the checkable kind: no inference,
fail-closed conflict, byte-preservation, and absence-without-pressure — each tested, two
mutation-verified. What it is **not** is evidence that delegation works. Nothing here was run against
a real feature or a real model; CHG-B owns that question and remains unopened.
