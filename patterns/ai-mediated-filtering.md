# Pattern: AI-Mediated Filtering Observability

## When This Pattern Applies

Use when a feature:
1. Calls an external AI/LLM to generate candidate proposals, suggestions, or results
2. Applies domain validation rules to filter those candidates post-generation
3. Emits only the surviving candidates in a behavioral event
4. Has a contract invariant requiring that filtering is enforced post-generation

If the LLM output is used directly without domain filtering, this pattern does not apply.
If filtering happens before the LLM call (input pruning, not output filtering), this
pattern does not apply.

---

## The Problem

A behavioral event with `proposal_count: 8` is consistent with two very different
implementations:

- **Implementation A:** LLM generated 12; domain filter rejected 4; 8 emitted
- **Implementation B:** LLM generated exactly 8 valid proposals; filter never exercised; 8 emitted

Runtime observation cannot distinguish them. The contract invariant "filtering enforced
post-generation" is structurally verified (code exists) and test-verified (deterministic
mock), but never *observed* at runtime.

This is a **GAP (observability)** in Stage 7 terms: the behavior may be occurring but
cannot be proven from emitted events alone.

---

## The Fix: generated_count

Add a `generated_count` field alongside `proposal_count` (or equivalent `surfaced_count`)
in the behavioral event payload:

```json
{
  "generated_count": 12,
  "proposal_count": 8,
  "proposals": [...]
}
```

**Invariant:** `generated_count >= proposal_count` always.
- If equal: filtering was a no-op for this invocation (or the LLM was already constrained).
- If greater: `generated_count - proposal_count` candidates were discarded by domain rules.

This closes the observability gap without introducing new event types and without requiring
a live LLM that generates specific invalid proposals in tests.

---

## Contract Implications

When adding this field, the contract must state:
- `generated_count` = number of candidates returned by the AI before domain filtering
- `proposal_count` = number of candidates that passed domain validation and were surfaced
- `generated_count >= proposal_count` is a schema invariant

Add to the Invariant Falsification table:

| Invariant | Falsifying fixture | Observable when correct | Wrong assumption |
|---|---|---|---|
| Post-generation filtering enforced | Mock LLM returns one invalid + one valid candidate | `generated_count=2, proposal_count=1` | Filter assumed from input pre-screening alone |

---

## Event Schema Implications

The field must be added to the behavioral event definition and traced to the contract
clause asserting post-generation enforcement. It is not a speculative addition — it
requires a contract clause before it can appear in the schema.

The `generated_count >= proposal_count` constraint belongs in the Schema Invariants
section alongside the existing `proposal_count == proposals.length` constraint.

---

## When NOT to Use

- The system intentionally avoids exposing raw AI outputs, rejected candidates, or
  internal generation statistics (privacy, security, cost, UX reasons)
- The feature contract has no post-generation enforcement invariant
- The LLM output is used directly without domain filtering

This is a **pattern, not a requirement**. Apply it when post-generation filtering is
contractually important and runtime observability is desired.

---

## Alternative: Per-rejection audit events

For per-candidate traceability (which proposals were rejected and why), emit an
OBSERVATIONAL event per rejection:

```
ProposalRejectedByVocabulary
  payload: { proposal_id, reason }
```

Higher cost (new event type, Stage 3 schema amendment, Stage 5 test updates) but enables
per-proposal observability and produces a richer audit trail.

Use when:
- Per-rejection reason has operational or diagnostic value
- The number of rejected candidates is large enough that aggregate counts are insufficient
- The contract requires individual rejection traceability

`generated_count` is sufficient when only the aggregate filter rate matters.
