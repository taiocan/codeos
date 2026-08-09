# Pattern: Controlled Plain English

## When This Pattern Applies

This pattern documents a writing discipline for AI-generated prose in Codeos artifacts: plain
language where precision doesn't require otherwise, exact modal verbs and quantifiers where it
does, and protection for literal content (event names, field names, quoted normative text) from
paraphrase drift. It applies wherever a downstream project or Codeos's own self-development has
enabled it via the Optional Mechanism Status Convention (`.codeos/templates/conventions.md` →
"Optional Mechanism Status Convention"; see `.codeos/dba/policies/controlled-plain-english/v1.md`
for the activation mechanics and call-site map).

**Consulted by:** every Stage 1-10 prompt and `pipeline-reviewer.md` — each check line names which
layers below apply to it and applies the enabled-but-pattern-unavailable rule (below) directly.
`codeos-reviewer-task.md` is **not** a consumer of this file — it stays configuration-neutral (see
"Reviewer Model" and `.codeos/dba/policies/controlled-plain-english/v1.md`'s call-site map): it
never reads this pattern or any status
file itself, and therefore never performs the enabled-but-pattern-unavailable check. It only reacts
to a status line automatically injected by `scripts/codeos-review.sh` (or `.codeos/scripts/
codeos-review.sh` downstream) before the reviewer is invoked, using Layer D2's rule text, which is
restated inline in `codeos-reviewer-task.md` itself rather than requiring it to read this file.
This pattern's Layer B/C2/D2 rules are toggle-gated (see "Layers," below); Layer A/C1/D1 are always
active regardless of the toggle, since they restate expectations that already exist elsewhere in
this environment or in Codeos's own Non-Negotiable Rules — naming them here consolidates them, it
does not create new authority.

---

## Layers

### Layer A — Plain communication (always active, advisory)

For ordinary chat, explanations, and session updates: short sentences, common words, conclusion
first, preserve exact names and facts. Preserve exact technical terms (e.g. "idempotent" never
loosens to "safe"); define an uncommon technical term in one plain sentence on first use, then use
it consistently afterward.

This restates tone expectations already present in this environment's own system instructions
(terse, direct responses) — it is advisory guidance for general communication, never toggle-gated,
and never a gate on artifact approval.

### Layer B — Specification and planning precision (toggle-gated)

Applies to: Intent, Contract, Event Schema prose, Architecture Baseline, Cohort Logical Design,
implementation plans, refinement records, and — self-development only — the Change Intent,
Acceptance Criteria, and Implementation Plan sections of backlog briefs and change records.

Rules: precise normative modal verbs (state whether something *must*, *should*, or *may* happen —
never blur the three); explicit quantifiers (state exact counts and bounds — "exactly one," "at
least one," "zero or more" — never loosen a stated quantifier to a vaguer one); separate
requirements from design decisions explicitly; use a decision table for branching/conditional logic
rather than prose that hides a missed case; state quality requirements in measurable terms; do not
invent a value for anything left unresolved — record it as unresolved (e.g. `[TBD]`) instead.

**This is the toggle-gated layer, and it is a generation discipline, not a review-compliance
regime** — see "Reviewer Model," below, for what that distinction means in practice. It does
**not** apply to Implementation Notes / implementation evidence (downstream `04-implement.md`
Review Package free text; self-development Implementation Notes), or to Stage 5 Tests / Stage 6
Observation / Stage 8 Replay — those report what happened or was observed, and use plain factual
reporting instead: state it, cite exact identifiers/evidence, no modal-verb obligations. Layer B
also does not weaken any stage's existing implementation-leak ban — Stage 1 Intent output still
excludes implementation/design detail regardless of whether Layer B is active.

### Layer C — Literal-content protection

- **C1 — always active** (already has existing cited authority elsewhere in this toolkit):
  - Never rename or alter an event name, field name, or enum value from an approved Event Schema —
    restates Non-Negotiable Rule #4 ("only schema events").
  - Never omit or rename a column in an existing structured table — restates `07-reconcile.md`'s
    own "use EXACTLY this column structure... do not omit or rename" instruction.
  - Never invent or silently drop behavior absent from the approved artifact — restates
    Non-Negotiable Rule #5.
- **C2 — toggle-gated** (new rules, not pre-existing authority — stated openly as new):
  - Never paraphrase a directly-quoted normative passage when restating it elsewhere.
  - Never simplify code or schema examples "to be easier to read" at the cost of accuracy.

### Layer D — Reviewer and reconciliation precision

- **D1 — always active** (restates the "advisory, never gatekeeping" principle already binding
  throughout `dba-system.md`/`CLAUDE.md`/`pipeline-reviewer.md` — naming it here doesn't create new
  authority): the reviewer's verdict stays advisory; a finding never invents a new requirement; a
  recommendation never silently becomes an obligation; evidence stays separate from inference.
- **D2 — toggle-gated**: short sentences, common words, direct explanations in a Findings Summary
  or reviewer free-text field — governed by the **current** activation status at review time,
  nothing else (see "Reviewer Model," below, for why this is deliberately not tied to what any
  specific assessed artifact's own history was).

---

## Reviewer Model

Controlled Plain English does not create a separate review authority. Reviewers report meaning
loss, authority mixing, and unverifiable requirements under the approved artifact and existing
Codeos rules — exactly as they do today. They do not classify a finding as "a Controlled Plain
English violation"; if "exactly one" silently becomes "one or more" somewhere, the reviewer reports
that the Contract's (or Schema's) stated quantifier changed, using existing review authority, not a
Controlled-Plain-English-specific category. Pure writing-style preference remains advisory only.

This produces four distinct concerns, none blended into another:

| Concern | Source of authority |
|---|---|
| How the review is written | The **current** activation status (Layer D2, if enabled) |
| Whether the artifact is correct | Approved Intent, Contract, Schema, Architecture, and existing doctrine |
| Whether a finding blocks approval | Human decision at the gate |
| Historical Controlled Plain English compliance | Not assessed — there is no stamp or version record to audit against |

A material Controlled Plain English violation is an in-scope finding when it changes or obscures
approved meaning, mixes artifact authority, or makes a requirement unverifiable — the reviewer
reports it, the human decides at the gate, exactly like any other finding. Pure style preference,
with no meaning effect, remains advisory only.

---

## Non-Retroactivity

Enabling this mechanism affects only text generated or revised while it is enabled. It does not
invalidate previously approved artifacts.

---

## Requirement-Set and Data/Interface-Specification Mapping

Codeos does not create separate Requirement Set or Data/Interface Specification artifact types for
this discipline. Apply the relevant writing rules wherever those concerns are legitimately
represented by an existing approved artifact — a performance requirement may belong in a Contract
or an Architecture Baseline; a security constraint may span several artifacts. Do not move content
into a different stage merely to match an external document category.

---

## Adaptation Matrix — 15-Section Traceability

| # | Source concern | Codeos treatment | Disposition |
|---|---|---|---|
| 1 | Identify the requested artifact type | Reuses existing DBA stage separation — each stage prompt already declares its own output artifact | Reused |
| 2 | Plain-but-exact language | Layer B | Retained |
| 3 | Preserve important meaning | Layer B + Layer C1/C2 | Retained |
| 4 | Do not invent missing information | Layer B (no-invented-value rule) | Retained |
| 5 | Precise normative modal verbs | Layer B | Retained |
| 6 | Explicit scope and quantity | Layer B (quantifier-preservation rule) | Retained |
| 7 | Condition patterns | Layer B (decision-table guidance) | Retained |
| 8 | Observable behaviour definition | **Stage 2 Contract** is the primary owner (observable behavior, conditions, outcomes, failure behavior) — not Stage 1 Intent's implementation-leak rule, which protects a boundary but does not itself implement this requirement | Reassigned to Contract |
| 9 | Separate requirements from design | Layer B, reinforcing the existing Intent/Contract boundary | Reinforced |
| 10 | Artifact-appropriate structure | Each stage's own existing template remains authoritative; Layer B never restructures a template | Reused |
| 11 | Edge-case consideration | **Stage 2 Contract** is the primary owner, with architecture-specific cases resolved at Architecture Synthesis; **Stage 5 Tests verifies**, it does not decide edge-case behavior for the first time | Reassigned to Contract, verified at Tests |
| 12 | Measurable quality requirements | Layer B | Retained |
| 13 | Verification-method statements | Layer B | Retained |
| 14 | Trailing "Precision Check" section | Not implemented as one merged section — ambiguities, assumptions, tensions, and open decisions stay in whichever distinct field each stage already has for that concept; a narrowly-scoped Assumptions field applies only in self-development backlog briefs/change records, only when needed, never empty | Adapted (split) |
| 15 | Honest uncertainty, no false completeness | Layer B (no-false-completeness rule) + Layer D1 (reviewer preserves uncertainty) | Retained |

---

## Enabled-but-Pattern-Unavailable Rule

The activation status file only says whether this mechanism is turned on; it says nothing about
whether this pattern file itself is reachable. After a check line resolves `status: enabled`:

1. Read this pattern at the context-specific path (`.codeos/patterns/controlled-plain-english.md`
   downstream; `patterns/controlled-plain-english.md` self-development).
2. If it is missing or unreadable (including a broken `.codeos` symlink), **stop and report a
   Controlled Plain English pattern-access error** — do not proceed as if disabled, and do not
   proceed as if enabled with assumed rules.
3. Apply Layer B/C2/D2 only from this file's actual content, as successfully read.
4. Never reconstruct these rules from `dba-system.md`, `CLAUDE.md`, or model memory — those
   documents describe the mechanism; they are not a fallback copy of this pattern's content.

This rule applies to every check line that reads this pattern directly (all Stage 1-10 prompts,
`pipeline-reviewer.md`). It does not apply to `codeos-reviewer-task.md`, which never reads this
pattern or any status file itself (see "Consulted by," above).
