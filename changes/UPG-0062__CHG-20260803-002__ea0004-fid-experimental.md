> ## ⚠ NON-AUTHORITATIVE EXPERIMENTAL ARTIFACT
>
> This document exists **only to measure the cost of producing it** (UPG-0062 Q1). It **governs
> nothing**. It is not an approved artifact, it is not part of EvidenceAtlas's artifact set, no
> downstream work may cite it as authority, and it does not override or supplement the approved
> Intent, Contract, Event Schema, Architecture Baseline, or Cohort Logical Design in any way. Whether
> an artifact of this kind should exist at all, and under what governance, is the open question
> (UPG-0062 Q2) that this document is evidence for — not a question it answers by existing.
>
> Produced from approved artifacts only. No implementation of EA-0004 existed or was consulted.

# Feature Implementation Design (experimental) — EA-0004 evidence_extraction

**Sources read:** `intents/EA-0004-evidence_extraction.md`,
`contracts/EA-0004-evidence_extraction_contract.md` (APPROVED R3),
`events/EA-0004-evidence_extraction_schema.md`, `architecture/core-baseline.md`,
`architecture/cohort-logical-design.md`, `architecture/implementation-profile.yaml`.

**Classification key.** `SOURCE-DERIVED` — determined by an approved artifact, which is cited.
`NEW DESIGN` — a mechanism choice no approved artifact determines. The distinction is the point of
this document: a `NEW DESIGN` row is a decision someone is making, not a restatement of something
already approved.

---

## A. Frame — what the approved artifacts already fix

| # | Element | Class | Source |
|---|---|---|---|
| A1 | Language is Rust | `SOURCE-DERIVED` | `implementation-profile.yaml`, `scope: all`, no exception |
| A2 | Feature module owns validation of its own contract invariants for its own aggregate | `SOURCE-DERIVED` | Cohort Logical Design §8 (Validation ownership) |
| A3 | Aggregate state change and its canonical event commit atomically in one transaction | `SOURCE-DERIVED` | Cohort Logical Design §9 (Event-emission rules) |
| A4 | `correlation_id` inherited, never minted here | `SOURCE-DERIVED` | Architecture Baseline, "Correlation and causation" |
| A5 | `events/runtime_events.jsonl` is observational only, not a bus | `SOURCE-DERIVED` | Architecture Baseline, "Integration style" |
| A6 | Four events, with exactly the payload fields listed | `SOURCE-DERIVED` | Event Schema, Event Definitions |
| A7 | Corpus version identity arrives from EA-0003; this feature performs no retrieval | `SOURCE-DERIVED` | Contract, Preconditions + Invariant 1 |
| A8 | Module directory `modules/evidence_extraction/`, crate name matching | `SOURCE-DERIVED` | Existing repository convention (`modules/research_brief/`) + workspace `members` |

Nothing in section A required a decision. Everything below did.

---

## B. Invariant → mechanism allocation

Each row states the enforcing mechanism and where it lives. Every `NEW DESIGN` row is a decision that
no approved artifact determines — the artifacts say *that* the invariant holds, never *how*.

### B1 — No new source discovery (Invariant 1; falsification row 1)

- **Mechanism:** the extraction entry point accepts an already-resolved corpus view and the module
  declares no I/O, network, or filesystem capability. Absence of the capability, not a runtime check,
  is what makes discovery impossible.
- **Class:** `NEW DESIGN`. The contract states the invariant; no artifact says it is enforced by
  capability-absence rather than by a guard.

### B2 — Unresolvable items never presented (Invariant 2; Failure Path 1; falsification row 2)

- **Mechanism:** a fallible smart constructor is the *only* way to obtain a `CandidateEvidence`.
  Resolution of `source_id` against the supplied corpus view, and presence of every required field,
  are checked there. Failure returns the rejection outcome carrying `unresolvable_fields`; it cannot
  return an item. No public constructor, no public fields, no `Default`.
- **Class:** `NEW DESIGN`. "Never presented" is the contract's requirement; "unrepresentable via a
  private constructor" is a chosen mechanism.
- **Schema tie:** `unresolvable_fields` is `array[string]` naming which of `source_id`/`provenance`/
  other required fields failed — so the checker collects *all* failures rather than short-circuiting
  on the first. `NEW DESIGN` (collect-all vs fail-fast is not specified).

### B3 — Source / interpretation stay distinguishable (Invariant 3; Failure Path 2; falsification row 3)

- **Mechanism:** `source_text`, `context`, `interpretation`, `rationale` are four distinct
  single-purpose newtypes, not four `String` fields. They cannot be assigned to one another or
  concatenated into a shared field. Distinguishability is structural, so the
  `SourceInterpretationConflated` rejection is reachable only for *externally supplied* candidate
  input, never for an item this module constructed.
- **Class:** `NEW DESIGN`. The contract requires the distinction; newtypes-over-strings is a choice.
- **Open sub-question:** the contract's conflation failure is about *presentation* ("phrased and
  formatted identically… with no distinguishing marker"). Types prevent field-mixing but not a
  rendering that visually merges them. **Rendering is out of this module's scope** — but no approved
  artifact says where it belongs. Flagged rather than silently decided.

### B4 — Review-facing judgments never self-asserted (Invariant 4; falsification: ReviewFacingJudgmentNotSelfAsserted; falsification row 4)

- **Mechanism:** enforced by *absence*. No sufficiency, independence, or unsupported-content field
  exists anywhere in the item or the events; no function computes one. The item carries the material
  (`context`, `interpretation`, `rationale`) and nothing that scores it.
- **Class:** `NEW DESIGN`. Absence-as-mechanism is a decision; the contract only forbids the assertion.

### B5 — Explicit absence, never a silent gap (Invariant 5; boundary: NoUsableCandidateForResearchQuestion; falsification row 5)

- **Mechanism:** extraction is driven by iterating the **scope's** Research Question set, not by
  iterating produced items. Each question yields either ≥1 `CandidateEvidenceCreated` or exactly one
  `NoUsableCandidateFound`. A completion function that receives the scope can therefore prove total
  coverage; one that receives only the produced items cannot.
- **Class:** `NEW DESIGN`. This is the EA-0003 lesson generalised — iterate the requirement set, not
  the results — and no approved artifact states it.
- **Guard:** `NoUsableCandidateFound` is emitted only when examination for that question is
  *complete*. Completion is an input, never inferred from an empty result.

### B6 — Derivative material never inflates coverage (Invariant 6; falsification: DerivativePresentationNeverBecomesIndependentEvidence; falsification row 6)

- **Mechanism:** coverage for a Research Question is counted over **distinct underlying sources**, not
  over items. `underlying_source_id` resolves an item to its distinct underlying source; items sharing
  one collapse to a single contribution.
- **Schema constraint, exactly two states:** `underlying_source_id` is `null` (this item's own source
  *is* the distinct underlying source, or no derivative relationship is classified) or a value that
  **must resolve** to an identifiable distinct source. A non-null-but-unresolvable value is rejected —
  the schema says such a value "would let the derivative item appear independent, which this field
  exists to prevent". Modelled as `Option<UnderlyingSourceId>` with resolution validated at
  construction; no third "unknown" state is representable.
- **Class:** `NEW DESIGN` for the counting mechanism; `SOURCE-DERIVED` for the two-state rule
  (Event Schema, `underlying_source_id`).
- **Boundary this feature does not cross:** it never *determines* that two items are derivative — the
  contract explicitly leaves that to whatever classification mechanism is in effect. It consumes the
  classification and enforces only non-inflation.

### B7 — Reuse vs distinct interpretations (boundary: TransparentReuseAcrossResearchQuestions; falsification row 10)

- **Mechanism:** the item is keyed by *interpretation*, not by Research Question.
  `research_question_ids` is a non-empty set on one item — one interpretation addressing three
  questions is one item with three links, never three items. Separately, two materially different
  interpretations of one passage are two items with distinct `candidate_evidence_id`s. The two cases
  are distinguished by whether the interpretation differs, and by nothing else.
- **Class:** `NEW DESIGN`. The schema permits both shapes and explicitly declines to conflate them;
  keying on interpretation is the mechanism that realises it.
- **Non-emptiness:** `research_question_ids` has ≥1 entry — enforced by a non-empty collection type,
  not a runtime length check. `SOURCE-DERIVED` (schema) / `NEW DESIGN` (type-level enforcement).

### B8 — Full traceability (Invariant 7; falsification row 9)

- **Mechanism:** every element the contract enumerates — corpus version, source, passage, context,
  interpretation, rationale, and every linked Research Question — is a required field on the item and
  on `CandidateEvidenceCreated`. No aggregate or summary substitute is permitted, and no field is
  optional except `underlying_source_id` per B6.
- **Class:** `SOURCE-DERIVED` for *what* must be traceable (Contract Invariant 7 + Event Schema);
  `NEW DESIGN` for "required non-optional fields rather than a lookup".

### B9 — Never self-decides acceptance (Invariant 8; Postconditions; falsification row 7)

- **Mechanism:** enforced by absence, as B4. No accept/reject/revise field, no status enum, no
  default. A newly produced item has no status *because no status exists to have* — EA-0008 owns that
  decision entirely.
- **Class:** `NEW DESIGN`.

### B10 — Vocabulary: concept identity, not representation (Vocabulary Dependency; falsification row 8)

- **Mechanism:** the duplication/overlap classification enters as a resolved **opaque concept**
  carrying a canonical representation for display only. Domain logic — specifically B6's
  non-inflation grouping — reads concept identity and never a string. `"Derivative"` and
  `"derivative"` resolve to one concept and therefore produce identical treatment. Resolution is
  supplied through a seam, because the contract explicitly leaves ownership (Policy Registry vs
  feature-local) open; the seam lets ownership be settled later without touching domain logic.
- **Class:** `NEW DESIGN` for the resolver seam and opaque-concept representation. `SOURCE-DERIVED`
  for the three invariants themselves (Contract, Vocabulary Dependency).

---

## C. Summary of classification

| Class | Count | Notes |
|---|---|---|
| `SOURCE-DERIVED` | 8 frame rows (A1-A8) + 4 partial ties in B | The approved artifacts fix language, ownership, transactionality, correlation, event shapes, field lists, and layout |
| `NEW DESIGN` | 10 of 10 mechanism allocations (B1-B10), 4 of them mixed with a derived component | **Every "how" in section B was a decision.** No approved artifact determines any of them |

**The finding this produces for Q2:** the approved artifacts fully determine *what* must be true and
*who* is responsible, and determine *none* of the ten mechanisms by which EA-0004's invariants would
actually be enforced. Two sub-questions surfaced that no artifact answers at all — where conflation-
resistant *rendering* belongs (B3), and who owns duplication classification (B10, explicitly left open
by the contract itself).

---

## D. Deliberately not included

No code, no signatures, no type definitions beyond naming a mechanism, no test plan. This document
specifies *what must be true and by what means*, not the implementation. Keeping that line is what
makes the Q1 cost comparison meaningful — a document that drifts into being the implementation would
measure nothing.
