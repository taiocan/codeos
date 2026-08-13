# Codeos Self-Development Workflow

## Your Role

You are changing the **Codeos toolkit itself** — prompts, templates, docs, patterns, or
scripts. This is governed by the repo-root `CLAUDE.md` (the Codeos Self-Development guide),
not by the downstream 9-stage DBA doctrine in `dba-system.md`. You are a constrained change
guide: every step requires explicit human approval, and Codex review cadence is governed by
the review profile assigned in Step 0a.

This is a practical step prompt, not a second doctrine. Keep your outputs lean.

---

## Step 0 — Triage

Classify the change before doing anything:

| Class | Path |
|---|---|
| `trivial` | Direct edit. No loop, no review, no change record. |
| `backlog-only` | Direct edit, unless it changes accepted scope (then run the loop). |
| `documentation` (normative) | 4-step loop. |
| `template` / `prompt` / `script-tooling` | 4-step loop. |
| `downstream-doctrine` | 4-step loop + downstream-compatibility acceptance criteria + grep cross-reference verification + reviewer scope-triage. |
| `self-dev-governance` | 4-step loop + scope-drift review (changes to `CLAUDE.md` or the self-dev loop itself). |

`trivial` = non-semantic only (typo, link, formatting, meaning-preserving wording, backlog
note). Anything touching process, policy, behavior, script behavior, template/prompt
meaning, doctrine, stage names, approval rules, or file layout is non-trivial. When unsure,
treat as non-trivial.

For a trivial change: make the edit and stop. For everything else: **select or create the
Feature Thread** — the `UPG-####` backlog feature this change implements (create a
`backlog/UPG-####-slug.md` brief with a fresh, never-reused id if none exists) — assign a unique
`CHG-YYYYMMDD-NNN`, open `changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md` from
`templates/codeos-change.md` (fill its trace header), and run the 4-step loop below. See
**Feature Thread & IDs** below for the nomenclature.

---

## Step 0a — Review Profile

After triage, assign a **review profile**. Profiles govern Codex review cadence and round
budgets. Human approval at each step transition is required at every profile.

| Profile | Applies when | Codex review cadence | Max rounds/step |
|---|---|---|---|
| PROFILE-0 | `trivial`; or `backlog-only` that stays a direct edit | No review, no loop | — |
| PROFILE-1 | `backlog-only` that escalates into the 4-step loop (changes accepted scope) | 1 review, at Reconcile only | 2 |
| PROFILE-2 | `documentation` (normative) | 1 review per step | 2 |
| PROFILE-3 | `template` / `prompt` / `script-tooling` | 1 review per step, R2+ delta | 3 |
| PROFILE-4 | `downstream-doctrine` | 1 review per step, R2+ delta | 3 |
| PROFILE-5 | `self-dev-governance` | 1 review per step, R2+ delta | 3 |

**Budget exceeded:** fix remaining findings inline and escalate to human decision. Do not
run further Codex rounds automatically. See `docs/reviewer-pipeline.md §4d` for the full
budget table and escalation procedure.

Record `review_profile: PROFILE-N` in the change record trace header.

---

## Step 0b — Writing Discipline Check

Read `config/writing-discipline.yaml` (Codeos-repo-local). Per the Optional Mechanism Status
Convention's four-outcome table (`templates/conventions.md`): absent or exact `status: disabled` →
disabled; exact `status: enabled` → enabled; anything else → stop and report a configuration error.

When enabled, apply `CLAUDE.md`'s "Writing Discipline (Controlled Plain English)" per-section rule
table to this change's own artifacts (Layer B for Change Intent / Acceptance Criteria /
Implementation Plan; factual reporting for Implementation Notes; Layer D1 always + D2 when enabled
for review findings and Reconciliation). No new change-record trace-header field is added for
this — non-retroactivity is the one-sentence rule already stated in
`patterns/controlled-plain-english.md`, nothing to stamp per change.

`scripts/codeos-review.sh` reads this same file automatically and injects its resolved status into
every review packet built for a self-development change — see `docs/reviewer-pipeline.md §12a`.
This step is about applying the discipline while *writing* the artifact; the wrapper's injection is
about what the *reviewer* is told, a separate but related mechanism.

---

## The 4-Step Loop

Each step requires explicit human approval before the next.
After each step output (**and its Codex review, if required by profile**), state:
**`AWAITING HUMAN APPROVAL TO PROCEED TO STEP [N+1]`**

For profiles that require a review at this step: run the review before the gate (see
Reviewer Handling). Advance only on an explicit "APPROVED" / "approved" / "yes proceed"
/ equivalent.

---

### Step 1 — Change Intent

**Task:** Select/create the Feature Thread, then produce the Change Intent section of
`changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md`.

**Rules:**
- **Feature Thread first:** choose the primary `UPG-####` (create a `backlog/UPG-####-slug.md`
  brief with a fresh, never-reused id if none exists), assign a unique `CHG-YYYYMMDD-NNN`, and
  fill the change record's trace header (`feature_id`, `primary_feature_id`, `change_id`,
  `implements`, …).
- State *why* (the problem in the toolkit) and *what changes* (name every file touched).
- State the scope boundary — what will NOT change. Anything not listed is in scope.
- Record the triage **class**, the **scope axis** (`self-dev only` | `downstream doctrine
  only` | `both`), and the originating `UPG-####`.
- Activate the row in `status/self-development.md` with **both** Feature ID and Change ID
  (State: IN_PROGRESS, Loop step: 1-Intent).

**Complete when:**
- [ ] Why + what-changes stated; every touched file named
- [ ] Scope boundary explicit
- [ ] Class, scope axis, and backlog item recorded
- [ ] Status row activated

Output: Change Intent section + review + **`AWAITING HUMAN APPROVAL TO PROCEED TO STEP 2`**

---

### Step 2 — Acceptance Criteria

**Task:** Produce the Acceptance Criteria section — the consistency contracts the change
must satisfy.

**Rules:**
- *Doctrine / `downstream-doctrine` / `both`:* require **downstream-compatibility** —
  the generated project `CLAUDE.md` still loads `.codeos/dba-system.md`; stage tables,
  prompt filenames, and references move together; no internal contradiction. Plan the
  grep cross-reference checks you will run in Step 4.
- *`script-tooling`:* state expected I/O behavior, exit codes / fail-closed cases, idempotency.
- Each criterion must be checkable in Step 4 (name how you will verify it).

**Complete when:**
- [ ] Every criterion is concrete and verifiable
- [ ] Downstream-compatibility criteria present for any doctrine-scoped change

Output: Acceptance Criteria section + review + **`AWAITING HUMAN APPROVAL TO PROCEED TO STEP 3`**

---

### Step 3 — Implement

**Task:** Make the change, constrained to the approved scope.

**Rules:**
- Update **all** cross-references (paths, stage tables, prompt names, doc links) in the
  same change.
- No scope creep. If you discover an out-of-scope change is needed, stop and re-triage it
  as its own change — do not fold it in here.
- A `downstream-doctrine` edit must change only what Step 1 declared; preserve 9-stage
  substance unless the change *is* a deliberate doctrine change.

**Complete when:**
- [ ] All planned edits done
- [ ] All cross-references updated
- [ ] No out-of-scope changes introduced

Output: summary of changes + review + **`AWAITING HUMAN APPROVAL TO PROCEED TO STEP 4`**

---

### Step 4 — Reconcile

**Task:** Produce the Reconciliation section and verify the change.

**What to check:**
1. Each acceptance criterion from Step 2 — PASS / FAIL with evidence.
2. Toolkit-wide consistency sweep (grep) for stale references, orphaned links, and
   stage-table ↔ prompt-file drift.
3. For `script-tooling`: a smoke run of the script.
4. Apply reviewer **scope triage** to any findings:
   IN-SCOPE BLOCKER / IN-SCOPE NON-BLOCKER / OUT-OF-SCOPE BACKLOG / REJECTED /
   SELF-REFERENCE / REVIEW-BOOKKEEPING. File out-of-scope items to `backlog/`.

**Complete when:**
- [ ] Every acceptance criterion verified
- [ ] Consistency sweep clean (or gaps filed/fixed)
- [ ] Findings scope-triaged
- [ ] Reconciliation written and the Codex review run (if required by profile)

Output: Reconciliation section + review + **`AWAITING HUMAN APPROVAL — SELF-DEVELOPMENT CHANGE COMPLETE`**

**Only after the human approves at this final gate** (the review is advisory; it never closes the
change by itself): mark the row **COMPLETE** in `status/self-development.md`, set
`state: COMPLETE` in the change record trace header, and log the decision per CLAUDE.md. Until
then the change stays **IN_PROGRESS** — matching the dashboard rule that `COMPLETE` requires human
acceptance.

---

## Feature Thread & IDs (nomenclature)

See `backlog/UPG-0001-feature-thread-traceability.md` for the full model. In short:

| Kind | Format | Meaning |
|---|---|---|
| Feature | `UPG-####` | Stable backlog feature/upgrade. Assigned once, never reused or renumbered. |
| Change | `CHG-YYYYMMDD-NNN` | One self-dev execution against a feature. A change id is **not** a feature id. |
| Review round | `REV__UPG-####__CHG-…__S<N>__R<N>` | One reviewer run for step `N`, round `M`. Lives only in `review-log` / `reviews/codex/*`. Documented id (not auto-emitted yet — see `UPG-0029`). |
| Review series | `RVS__UPG-####__CHG-…__S<N>` | The **stable** set of all Step-`N` rounds for a change. Reviewed artifacts reference *this* + `review_state`, never a round (see Self-Reference Boundary). |
| Finding | `FND__REV__…__NN` | An individual reviewer finding, when explicit tracking is warranted. |

- **Filenames:** features `backlog/UPG-####-slug.md`; changes
  `changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md`; the primary `UPG-####` is always visible.
- **Backlog feature header** carries `feature_id`, `slug`, `title`, `status`, `priority`, and the
  thread-linkage fields (`depends_on`, `related_features`, `supersedes`, `superseded_by`).
  **`class` and `scope` are declared per change** (Step 1 / the change record + dashboard), not in
  the backlog header — include them in a feature header only when already known.
- Every backlog feature file has a `## Feature Thread` rollup (Changes / Reviews / Findings /
  Follow-up). Keep it compact: ids and links, not full review text.
- Findings are triaged: IN-SCOPE BLOCKER / IN-SCOPE NON-BLOCKER / OUT-OF-SCOPE BACKLOG / REJECTED / SELF-REFERENCE / REVIEW-BOOKKEEPING.

### Review-Fix Rule

A fix for a reviewer finding **stays inside the same `CHG-*`** when it addresses an IN-SCOPE
finding, does not alter the approved scope, and only repairs implementation / docs / status /
acceptance / review-record consistency for the current change.

A fix **creates or links a new `UPG-####`** only when it is OUT-OF-SCOPE BACKLOG, materially
changes the approved intent/acceptance, introduces a new feature/policy/workflow/file-type/tool
behavior, or would make the current change unreviewably broad. **A review fix never receives the
next feature id merely because it happened after a review.**

### Surface ownership (what each file may contain)

| Surface | Owns | Must NOT contain |
|---|---|---|
| `changes/UPG-*__CHG-*.md` trace header | feature/change id, `review_series`, current step, `review_state` | exact latest review round |
| `backlog/UPG-*.md` Feature Thread | related change ids, review-**series** rows, accepted verdict summary | every live round |
| `status/self-development.md` | operational state, current step, **review state** | round-by-round history |
| `reviews/review-log.md` | exact rounds, verdicts, packet hashes, human decisions | design prose |
| `reviews/codex/*.md` | raw reviewer assessment | canonical status |

### Self-Reference Boundary

The Codex review assesses the very artifacts that record it, so those artifacts **cannot
freshly name the review currently assessing them.** Therefore:

- Reviewed artifacts carry a stable `review_series` (`RVS__…__S<N>`) + `review_state`
  (`DRAFT | IN_REVIEW | REVIEWED | ACCEPTED`) — **never** an exact latest round. Exact
  `REV__…__R<N>` rounds + the human decision live only in `reviews/review-log.md` / `reviews/codex/*`.
- **Self-reference rule:** an artifact under review is *not required* to contain the current review
  round; a reviewer finding that it omits the current round is valid **only** if the artifact
  explicitly claims exact latest-round authority.
- **Stop rule:** if two consecutive rounds find *only* stale review-bookkeeping caused by the
  previous round's existence, stop editing the reviewed artifact and resolve by human decision —
  this recognizes a causal loop, it does not weaken discipline.

(Enforcing this inside the reviewer/packet is tracked in `UPG-0028`; the script is unchanged here.)

---

## Reviewer Handling (advisory)

Run the Codex reviewer when required by the assigned profile (see Step 0a). The round `R<N>`
increments on each re-review of the same step.

### Local pre-review checklist (run before every Codex call)

Run ALL applicable checks and fix failures before invoking `codeos-review.sh`:

```
grep -rn "UPG-####\|CHG-YYYYMMDD-NNN" <changed-files>   # no literal placeholders
grep -n "latest_review\|review_round" <changed-files>     # no live round IDs embedded
grep -n "TBD\|FIXME\|\[to be filled\]" <changed-files>   # no unresolved placeholders
git diff -- dba-system.md scripts/codeos-review.sh        # scope boundary clean (empty)
# trace header state/current_step/review_series matches dashboard row
# all referenced paths in new prose exist (ls / grep to verify)
# all internal cross-references (e.g. "see §4a") resolve to real section names
```

### Claim audit (before sending)

Scan all new or modified prose for universal quantifiers: "all", "every", "never", "always",
"no X", "any", "none". For each: provide evidence, weaken to "most"/"typically", or remove.
Universal claims without evidence are the most common Codex-flagged false-claim source.
See `docs/reviewer-pipeline.md §4c` for the full audit procedure.

### Running the reviewer

```
bash scripts/codeos-review.sh review UPG-####__CHG-YYYYMMDD-NNN selfdev-step-<N> \
  changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md <touched-files>
```

The logical review id is `REV__UPG-####__CHG-YYYYMMDD-NNN__S<N>__R<N>` (documented
convention; script still writes legacy assessment filename — `REV__` emission deferred to
`UPG-0029`).

- The verdict (NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE) is **advisory** — it informs
  the human, never auto-blocks. The human decides at the gate.
- Present the verdict and any findings, scope-triaged, alongside your step output.

### R2+ delta reviews

For R2 and later rounds at the same step, send a **delta packet** (not the full context).
Include only: the acceptance criterion under challenge, changed lines since the previous round
(exact unified diff of affected files only), one-line per-finding summary from the previous
round with resolution, and the current trace header state/step/review_state. See
`docs/reviewer-pipeline.md §4b` for the exact spec and what to omit.

### Findings triage

Classify every reviewer finding as exactly one of:

| Category | When |
|---|---|
| **IN-SCOPE BLOCKER** | Breaks the stated goal; creates a false claim; weakens advisory/human-gate guarantees; scope-boundary violation |
| **IN-SCOPE NON-BLOCKER** | Improvement, not required for this step |
| **OUT-OF-SCOPE BACKLOG** | Valid, but belongs to a future change — file to `backlog/` |
| **REJECTED** | Conflicts with the stated scope or Codeos philosophy |
| **SELF-REFERENCE / REVIEW-BOOKKEEPING** | Stale bookkeeping caused by the previous round's own existence (causal loop). If two consecutive rounds find only this category → resolve by human decision; do not run another round |

---

## Stopping Rules

- Stop at every gate; never self-advance.
- Stop and re-triage if a change grows beyond its declared scope.
- Stop and flag if a `self-dev only` change turns out to touch `dba-system.md` — that needs
  a `downstream-doctrine` or `both` scope, re-declared in Step 1.
- Stop adding Codex rounds when the profile's max rounds/step is reached; fix remaining
  findings inline and escalate to human decision (see Step 0a and `docs/reviewer-pipeline.md §4d`).
