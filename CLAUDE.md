# Codeos Self-Development — Toolkit Operating Guide

> **What this file is.** This file governs development of the **Codeos toolkit repository
> itself** — its prompts, templates, docs, patterns, and scripts. It does **not** replace
> the downstream DBA doctrine in [`dba-system.md`](dba-system.md), which is what
> *downstream projects* load via `.codeos/dba-system.md`. Do not run the full 9-stage DBA
> loop on the toolkit; use the lean self-development loop below.

This is a **stable rulebook**. It contains no live workflow state. Current status lives in
[`status/self-development.md`](status/self-development.md); per-change detail lives in
`changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md`.

All paths in this file are **repo-relative** (`prompts/`, `templates/`, `scripts/`, …).
There is no `.codeos/` symlink inside this repo — that prefix is only for downstream projects.

---

## Mode Declaration

You are developing the Codeos toolkit. Codeos *defines* Declarative Behavioral Architecture
(DBA); it does not need to prove DBA to itself. Toolkit changes are prose (prompts,
templates, docs, patterns) and bash tooling (the reviewer pipeline, `dba-init.sh`). There
are no runtime events, no event schema, and no replay here — so the 9-stage loop does not
apply. Self-development keeps DBA's **philosophy** — intent primacy, approve-the-WHAT-before-
the-HOW, human-approved gates, advisory (never gatekeeping) review — in a leaner shape.

---

## At Session Start

1. Read this file.
2. Read [`status/self-development.md`](status/self-development.md) — the live Self-Development
   Status dashboard (what is in flight and at which step).
3. Inspect the active `changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md` for any IN_PROGRESS row.
4. Confirm the current state with the human, then **STOP** and ask what to work on.

---

## Truth Authority (inherited)

1. **Explicit human correction** at any gate overrides everything else.
2. **The artifacts on disk are authoritative** — never memory of them. Re-read before acting.
3. Surface unresolved conflicts to the human rather than silently resolving them.

The downstream doctrine in `dba-system.md` is the authority on DBA *substance*. When a
self-development change touches it, see the `downstream-doctrine` rules below.

---

## Triage Front-Door

Classify every change before doing anything. Class determines rigor.

| Class | Path |
|---|---|
| `trivial` | Direct edit. No loop, no review, no change record. |
| `backlog-only` | Direct edit, unless it changes accepted scope (then 4-step loop). |
| `documentation` (normative) | 4-step loop. |
| `template` / `prompt` / `script-tooling` | 4-step loop. |
| `downstream-doctrine` | 4-step loop **+ downstream-compatibility acceptance criteria + grep cross-reference verification + reviewer scope-triage**. |
| `self-dev-governance` | 4-step loop **+ scope-drift review** (changes to this file or the self-dev loop itself). |

**`trivial` means non-semantic only:** typo, broken link, formatting, a wording
clarification that does not change meaning, or a backlog note with no implementation.
**Anything** that touches process, policy, behavior, script behavior, template meaning,
prompt semantics, doctrine, stage names, approval rules, file layout, or generated-project
layout is **non-trivial** and uses the 4-step loop. When unsure, treat it as non-trivial.

**Scope axis** — every non-trivial change must declare its scope in Step 1:
`self-dev only` | `downstream doctrine only` | `both`. This prevents accidental drift into
the downstream master doctrine.

After triage, assign a **review profile** (Step 0a of `prompts/codeos-self-dev.md`); the
profile governs Codex review cadence and round limits.

---

## The 4-Step Self-Development Loop

Anchor each non-trivial change to a `backlog/` item (create one if none exists). Use
[`prompts/codeos-self-dev.md`](prompts/codeos-self-dev.md) for the detailed step prompt and
[`templates/codeos-change.md`](templates/codeos-change.md) for the change record. One change
record per non-trivial change: `changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md`.

Each step: **produce output → run the Codex review if required by profile → STOP at the
gate → human approves → next step.** (Profile governs review cadence — see Step 0a of
`prompts/codeos-self-dev.md`.)

1. **Change Intent** — Why (problem in the toolkit), what changes (named files), what stays
   the same (scope boundary), triage class, scope axis, originating backlog id. Start the
   change record; activate the row in `status/self-development.md`.
2. **Acceptance Criteria** — The consistency contracts the change must hold.
   *Doctrine/downstream:* cross-reference integrity, downstream-compatibility (the generated
   project still loads `.codeos/dba-system.md`; stage tables, prompt filenames, and
   references move together), no internal contradiction. *Tooling:* I/O behavior, exit-code /
   fail-closed cases, idempotency.
3. **Implement** — Edits constrained to the approved scope. Update **all** cross-references
   in the same change. No scope creep. If a change you discover is out of scope, stop and
   re-triage it as its own change.
4. **Reconcile** — Verify each acceptance criterion. Sweep the toolkit for stale references,
   orphaned links, and stage-table↔prompt-file drift (grep). For tooling, do a smoke run.
   Apply reviewer **scope triage** (IN-SCOPE BLOCKER / IN-SCOPE NON-BLOCKER /
   OUT-OF-SCOPE BACKLOG / REJECTED / SELF-REFERENCE / REVIEW-BOOKKEEPING). Mark the row COMPLETE in `status/self-development.md`;
   log the decision (see Review Logging).

### Review cadence and advisory verdict

Review cadence is governed by the **review profile** assigned in Step 0a of
`prompts/codeos-self-dev.md`. High-risk profiles (PROFILE-3 through PROFILE-5) require
a Codex review before each step gate. Lighter profiles (PROFILE-1, PROFILE-2) may
limit Codex review to Reconcile only or reduce the round budget, as defined by the
profile. Human approval at each step transition is required at every profile; reviewer
output is advisory and non-gatekeeping at every profile.

To run the reviewer:

```
bash scripts/codeos-review.sh review UPG-####__CHG-YYYYMMDD-NNN selfdev-step-<N> changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md <touched-files>
```

The verdict is **advisory** — NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE inform
the human's decision but never auto-block. The reviewer is independent, read-only, and
non-gatekeeping; the human decides at the gate.

### Gate discipline

After each step output (and its review), STOP and state:
`AWAITING HUMAN APPROVAL TO PROCEED TO STEP [N+1]`. Advance only on an explicit
"APPROVED" / "approved" / "yes proceed" / equivalent. Anything else is a revision request.

---

## Stack / Dependency Reconciliation

Evidence hygiene tied to watched files — not a second governance layer, not doctrine, and not
an independent approval authority. `status/stack-manifest.md` records the current observed
stack and dependency-policy status for Codeos self-development. Human approval at each gate
remains the authority; the manifest and reconciliation reports are evidence, not authority.

- Dependency/stack-file changes (watched files: `Cargo.toml`, `Cargo.lock` in this repo — see
  `status/stack-manifest.md` for the full list and which patterns actually apply here) must be
  declared in Step 1's "What changes," or explicitly re-triaged before implementation if
  discovered later.
- If Step 1 declares a watched-file change, Step 2 must include a verification criterion for
  it.
- Step 4 verifies that a `status/stack-reconciliation/<CHG-id>-stack-reconciliation-report.md`
  instance exists for the change, and runs `check-drift` against it.

---

## What You NEVER Do (self-development)

- Treat a non-trivial change as trivial to skip the loop.
- Advance a step without running the review required by your profile and getting explicit approval.
- Change the downstream doctrine (`dba-system.md`) as a side effect — that requires an
  explicit `downstream-doctrine` (or `both`) scope declaration in Step 1.
- Rewrite downstream 9-stage substance when only a path/location change is intended.
- Put live status into this file — it belongs in `status/self-development.md`.
- Turn `prompts/codeos-self-dev.md` into a second doctrine; keep it a practical step prompt.
- Let the reviewer become an enforcement engine — its verdict is advisory, scope-triaged.

---

## Minimum Complexity

Use the smallest solution, change, artifact, and response that satisfies the current need. Prefer
deletion, reuse, and one clear path. Do not obtain simplicity by changing exact meaning, names or
literals, normative strength, quantities, or unresolved decisions.

---

## Self-Development File Layout

```
Codeos/                          ← toolkit repo (this repo)
├── CLAUDE.md                    ← THIS FILE — stable self-development operating guide
├── dba-system.md                ← downstream DBA doctrine (loaded by downstream projects)
├── status/
│   ├── self-development.md       ← live Self-Development Status dashboard (mutable; Feature ID + Change ID)
│   ├── roadmap.md                ← dependency-aware wave plan, keyed by UPG-#### (mutable)
│   ├── stack-manifest.md         ← live stack/dependency status (evidence, not authority; mutable)
│   └── stack-reconciliation/     ← one *-stack-reconciliation-report.md per watched-file change
├── changes/
│   └── UPG-####__CHG-YYYYMMDD-NNN__slug.md  ← per-change source of truth (one per non-trivial change)
├── backlog/
│   ├── features.md               ← authoritative UPG-#### → file map (identity)
│   └── UPG-####-slug.md          ← feature briefs w/ trace header + Feature Thread (feed Step 1)
├── prompts/                     ← stage + self-dev step prompts
├── templates/                   ← artifact templates
├── patterns/                    ← structural patterns
├── docs/                        ← toolkit documentation
├── scripts/                     ← dba-init.sh, codeos-review.sh
└── reviews/
    ├── review-log.md             ← append-only review + decision log
    ├── architecture-journal.md   ← cross-cutting institutional memory (AJ-NNN)
    └── codex/                     ← reviewer assessments + packets
```

**Identity & IDs (Feature Thread model).** Work is traced by stable **`UPG-####`** feature ids,
per-execution **`CHG-YYYYMMDD-NNN`** change ids, **`REV__…__S<N>__R<N>`** review-round ids, and the
stable **`RVS__…__S<N>`** review-series id. **Self-Reference Boundary:** reviewed artifacts carry
`review_series` + `review_state`, never a live round — exact rounds live only in `reviews/`. The
dashboard separates Feature ID from Change ID, and every backlog brief carries a
`## Feature Thread` rollup. The full model is `backlog/UPG-0001-feature-thread-traceability.md`.
In-scope review fixes stay inside the same `CHG-*`; only OUT-OF-SCOPE BACKLOG findings spawn a new
`UPG-####` (see the Review-Fix Rule in `prompts/codeos-self-dev.md`).

---

## Review Logging

When the human gives a reviewer's assessment and their decision, before any other work:
show a brief (≈5-line) preview of what you will write, then write it.

1. **One entry** to `reviews/review-log.md` (append-only) capturing the reviewer's core
   insight (close to verbatim) and the human decision separately.
2. **One entry** to `reviews/architecture-journal.md` (`AJ-NNN`) only if the insight will
   still matter six months from now to someone who has forgotten this change. When unsure,
   journal only if future usefulness is clear.

**Human overrides:** "do not log this review" / "journal this" / "do not journal this".

**Fidelity:** preserve the insight verbatim; compress only context. Record conclusions and
rationale, not conversation history. Log entries are append-only — supersede with a new
entry, never rewrite.
