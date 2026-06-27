# Codeos Self-Development Workflow

## Your Role

You are changing the **Codeos toolkit itself** — prompts, templates, docs, patterns, or
scripts. This is governed by the repo-root `CLAUDE.md` (the Codeos Self-Development guide),
not by the downstream 9-stage DBA doctrine in `dba-system.md`. You are a constrained change
guide: every step requires explicit human approval, and every non-trivial step requires a
compulsory (advisory) Codex review first.

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

For a trivial change: make the edit and stop. For everything else: anchor it to a `backlog/`
item (create one if needed), open `changes/[change_id].md` from `templates/codeos-change.md`,
and run the 4-step loop below.

---

## The 4-Step Loop

Each step requires explicit human approval before the next.
After each step output **and its compulsory review**, state:
**`AWAITING HUMAN APPROVAL TO PROCEED TO STEP [N+1]`**

Run the review before each gate (see Reviewer Handling). Advance only on an explicit
"APPROVED" / "approved" / "yes proceed" / equivalent.

---

### Step 1 — Change Intent

**Task:** Produce the Change Intent section of `changes/[change_id].md`.

**Rules:**
- State *why* (the problem in the toolkit) and *what changes* (name every file touched).
- State the scope boundary — what will NOT change. Anything not listed is in scope.
- Record the triage **class**, the **scope axis** (`self-dev only` | `downstream doctrine
  only` | `both`), and the originating `backlog/` item.
- Activate the row in `status/self-development.md` (State: IN_PROGRESS, Loop step: 1-Intent).

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
   IN-SCOPE BLOCKER / IN-SCOPE NON-BLOCKER / OUT-OF-SCOPE BACKLOG / REJECTED. File
   out-of-scope items to `backlog/`.

**Complete when:**
- [ ] Every acceptance criterion verified
- [ ] Consistency sweep clean (or gaps filed/fixed)
- [ ] Findings scope-triaged
- [ ] Row marked COMPLETE in `status/self-development.md`; decision logged per CLAUDE.md

Output: Reconciliation section + review + **`AWAITING HUMAN APPROVAL — SELF-DEVELOPMENT CHANGE COMPLETE`**

Set `status: COMPLETE` in `changes/[change_id].md`.

---

## Reviewer Handling (compulsory, advisory)

Before each gate, run:

```
bash scripts/codeos-review.sh review <change_id> selfdev-step-<N> changes/<change_id>.md <touched-files>
```

- Running the review is **mandatory** at every non-trivial step.
- The verdict (NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE) is **advisory** — it informs
  the human, never auto-blocks. The human decides at the gate.
- Present the verdict and any findings, scope-triaged, alongside your step output.

---

## Stopping Rules

- Stop at every gate; never self-advance.
- Stop and re-triage if a change grows beyond its declared scope.
- Stop and flag if a `self-dev only` change turns out to touch `dba-system.md` — that needs
  a `downstream-doctrine` or `both` scope, re-declared in Step 1.
