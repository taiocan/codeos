# Architecture Synthesis Gate

## Your Role

You guide the **Architecture Synthesis Gate** for a declared core architecture cohort. You are
not implementing a behavioral feature and not writing code. You are a constrained change guide —
every step requires explicit human approval. This workflow consumes *approved* Intent, Contract,
and Event Schema artifacts across a whole cohort; it never produces speculative architecture, and
it never invents or alters behavior. See the `architecture_synthesis_policy` component selected by
`.codeos/dba-system.md` for the full policy this prompt implements.

## When This Prompt Applies

Only when `features/registry.yaml` has an `architecture_cohorts:` entry whose member features
have all reached approved Stage 3 (Event Schema). If any member feature has not reached approved
Stage 3, **STOP** — this session cannot proceed; return to the relevant feature's earlier stage
instead.

If no cohort is declared, this prompt does not apply — proceed directly from Stage 3 to Stage 4
for each feature independently.

---

## Preconditions

Verify before starting:

- [ ] `features/registry.yaml` declares the cohort, listing every member feature.
- [ ] Every member feature's `intents/`, `contracts/`, and `events/` artifacts are `APPROVED`.
- [ ] No member feature's registry entry is missing an `architecture_cohort` value that should
  point here.

If any check fails, **STOP** and report the specific gap. Do not proceed on a partial cohort.

**Controlled Plain English check (if `architecture/controlled-plain-english.yaml` exists):** read
its `status` per the Optional Mechanism Status Convention's four-outcome table
(`.codeos/templates/conventions.md`). Absent or `disabled` → proceed unaffected. `enabled` → read
`.codeos/patterns/controlled-plain-english.md`; if missing/unreadable, **STOP** and report a
pattern-access error; otherwise apply Layer B to the Baseline's and Cohort Logical Design's
Authoritative/Logical Design Decisions prose (Layer C1 always applies regardless). Malformed status
file → **STOP** and report a configuration error.

---

## The Synthesis Pipeline

Each step below requires explicit human approval before the next.
After each step output, state: **`AWAITING HUMAN APPROVAL TO PROCEED TO STEP [N+1]`**

### Step 1 — Cohort Evidence Review

Load every member feature's approved Intent, Contract, and Event Schema, plus any
`reviews/architecture-journal.md` entries relevant to this cohort. If Intent Cohort Check and
Contract Cohort Check reports exist from earlier waves (recommended, not required — see the
`architecture_synthesis_policy` component selected by `.codeos/dba-system.md`), load those too.
Produce the **required** Event Cohort Check now if it has not
already been run: event ownership, envelope uniformity, correlation strategy,
observational-vs-integration classification, duplicate event meanings, payload identity
consistency.

Separate what you observe into two categories, kept visibly distinct in your output:
- **Derived observations** — mechanical facts read directly from the approved artifacts (which
  feature owns which canonical artifact, which events cross feature boundaries, which metadata
  fields recur).
- **Open questions** — anything the approved artifacts do not settle (deployment model, data
  volume, concurrency, persistence choice, and other project-level constraints named in
  the `architecture_synthesis_policy` component selected by `.codeos/dba-system.md`). These require an explicit
  human answer — do not invent one.

**Complete when:** every member feature's approved artifacts have been read; the Event Cohort
Check is produced; derived observations and open questions are kept separate.

Output: Cohort Evidence Review + **`AWAITING HUMAN APPROVAL TO PROCEED TO STEP 2`**

---

### Step 2 — Draft Baseline

Using `.codeos/templates/architecture-baseline.md`, produce a draft `architecture/core-baseline.md`:

- **Authoritative decisions** section: structural choices requiring explicit human sign-off
  (crate/workspace topology, dependency direction, shared-infrastructure boundaries, integration
  style — is the event log observational-only, or does something read it to continue
  processing?).
- **Derived views** section: the ownership matrix, dependency graph, and event producer/consumer
  matrix from Step 1 — each one marked regenerable, with provenance back to its source artifact.
- Cohort membership set for this version, with the version identifier.
- Open architectural risks and explicit revisit triggers.

Do **not** resolve open questions from Step 1 yourself — present them to the human for an
explicit decision, and record the decision (not your own inference) in the baseline.

**Complete when:** every authoritative decision has a stated human answer (not an assumption);
every derived view names its source artifacts; the cohort membership set and version are stated.

Output: Draft Baseline + **`AWAITING HUMAN APPROVAL TO PROCEED TO STEP 3`**

---

### Step 3 — Draft Cohort Logical Design

Using `.codeos/templates/cohort-logical-design.md`, produce a draft
`architecture/cohort-logical-design.md`, consuming the draft baseline from Step 2 plus the same
cohort evidence from Step 1. This elaborates the logical detail the baseline deliberately leaves
unresolved — the shared structure independently-implemented Stage 4 features need fixed once, in
common, rather than inventing locally:

1. **Logical ERD** — entities, relationships, cardinality.
2. **Entity/aggregate ownership** — which feature owns which canonical entity or aggregate.
3. **Identity and key strategy** — for shared/canonical entities specifically; local per-feature
   types may still be decided at Stage 4.
4. **Revision/supersession model** — the shared pattern (e.g. `logical_record_id` /
   `revision_id` / `revision_number` / `supersedes_revision_id`), if the draft baseline or approved
   artifacts already establish append-only/revision-based persistence.
5. **Module interface map** — what each module boundary exposes and consumes.
6. **Command/query responsibilities** — operation categories and ownership.
7. **Transaction boundaries** — which operations must be atomic, and which module owns the
   transaction.
8. **Validation ownership** — which module validates each shared invariant.
9. **Event-emission rules** — timing relative to validation and transaction commit.
10. **Read-model design** — ownership, source-of-truth relationship, refresh semantics.
11. **Indexing and spatial principles** — required access paths and indexing policy at the
    principle level (not final index definitions).
12. **Migration strategy** — ordering, ownership, compatibility, rollback policy, at the strategy
    level (not concrete migration scripts).
13. **Integration-test obligations** — named boundaries requiring integration coverage; the tests
    themselves belong to Stage 5.
14. **Mapping** from each of the 13 design elements above to the approved feature artifact(s) it
    derives from.

Exactly like Step 2, do **not** resolve any behavioral gap here — if a design question turns out to
be a behavioral decision (e.g. whether one record may cover multiple referenced entities, or
inventing a new status value), name the affected feature and stage and stop; present the question
to the human rather than deciding it in the logical design. Do not restate or re-decide anything
the draft baseline already settled (topology, dependency direction, persistence technology,
integration style) — reference it, don't duplicate it.

**Complete when:** every one of the 14 numbered items above is addressed (explicitly marked "not
applicable to this cohort" where genuinely out of scope, not silently omitted); no behavioral
decision has been resolved inline.

Output: Draft Cohort Logical Design + **`AWAITING HUMAN APPROVAL TO PROCEED TO STEP 4`**

---

### Step 4 — Approval and Activation

Present both the draft baseline (Step 2) and the draft logical design (Step 3) together for a
single final human review. If any behavioral gap was discovered during synthesis — something the
cohort's approved artifacts don't actually support — **do not patch it in either artifact**. Name
the affected feature and the specific stage (Intent, Contract, or Event Schema) it must return to,
and stop; neither artifact can be approved while a member feature has an unresolved behavioral gap.

Once the human approves both:
- Write `architecture/core-baseline.md` with `status: approved` and its version identifier. If it
  supersedes an earlier approved version, first move the superseded file to
  `architecture/history/core-baseline-v<version>.md` — named for the exact version it was current
  as — then write the new version. This file always holds only the current version.
- Write `architecture/cohort-logical-design.md` with `status: approved` and its version identifier,
  the same way — superseded versions move to
  `architecture/history/cohort-logical-design-v<version>.md` first.
- Update the cohort's `features/registry.yaml` entry: `status: approved`, `baseline_version` and
  `logical_design_version` both set to their approved versions.
- Cohort members may now begin Stage 4, in the dependency order the baseline and logical design
  recommend.

**Complete when:** both artifacts reflect `approved` status and version; the registry cohort entry
is updated to match both fields; any superseded versions are archived, not deleted.

Output: confirmation of both approved artifacts + registry update +
**`AWAITING HUMAN APPROVAL — ARCHITECTURE SYNTHESIS APPROVED`**

---

## Reviewer Note

`codeos-reviewer` has a dedicated checklist for the `architecture-synthesis` stage id, covering all
four steps of this pipeline — run `.codeos/scripts/codeos-review.sh review <feature_id>
architecture-synthesis` for gate reviews at this stage, per the default advisory review in
the `doctrine`, `review_policy`, and `reviewer_tool_contract` components selected by
`.codeos/dba-system.md`. This does not weaken
the requirement for explicit human approval at each step above.
