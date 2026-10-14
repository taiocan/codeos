# WO-01 — Finish F-0001 and Record Receipt Invalidation

P1 · NORMAL · Charter: Phases P1

## Goal

Take JamFree F-0001 to F9 with CodeOS as it is, and record which human decisions it had to repeat.

## Steps

1. Run one `codeos-reviewer plan` readiness check, then one Codex Stage 8 assessment of candidate
   `03818120`. Do not change the candidate or the packet unless readiness reports a problem.
2. If the assessment is BLOCKED, classify each finding by the Charter's P1 categories before
   changing anything.
3. The human records the F8 and F9 decisions.

## Output

A table of the human receipts F-0001 needed in this cycle. For each receipt: the binding that
invalidated it, and whether what the receipt judged actually changed. This table is WO-02's input.

## Acceptance criteria

1. F-0001 is currently valid, or every BLOCKED finding is classified and routed.
2. The table is complete.

## Results

- **CodeOS conformance repair (2026-10-14):** historical F8/F9 receipts could satisfy current
  checkpoints without binding the candidate/product state they were required to judge.
  Required-binding evaluation now fails closed on absent bindings; status exposes the resulting
  human decision step (ECP WO-11). No new authority, DBA configuration, or recovery mechanism
  introduced. Measured impact: only F-0001's F8 and F9 receipts changed result.

### Human receipts F-0001 needed after its 2026-09-30 acceptance

Sources: JamFree `.codeos/06-workflow/decisions.jsonl` (binding diff against the previous receipt
for the same checkpoint) and JamFree Git history. The classification is diagnostic input to P2,
not authority over the future judged surface.

| Receipt | Invalidating trigger | Binding that changed | Observable consequence | Preliminary assessment |
|---|---|---|---|---|
| F4 `early_preview` 10-05 17:45Z | `de6872f` F-0002 recovery baseline (shared `main.rs`, `provider.rs`) | `implementation_state` | Human reconfirmed the direction after regression verification | NO EVIDENCE OF CHANGE |
| F4 `early_preview` 10-07 19:05Z | `633834d` DBA-22 product scoping; `509abca` F-0003 summary | `implementation_state` replaced by `product_state` | No rationale recorded | UNCERTAIN |
| F4 `early_preview` 10-09 09:44Z | `fdbe263` phone layout (shared `.grid` CSS); F-0004 return summary | `product_state` | Human: "introduces no new F-0001 behavior" | UNCERTAIN (shared presentation changed) |
| F6 `reconciliation` 10-09 09:57Z | DBA-24/25 adoption; fresh production evidence | `codeos_yaml`, `implementation_state` newly bound; `production_evidence` | "unchanged implementation conforms" | NO EVIDENCE OF CHANGE |
| F4 `early_preview` 10-10 05:28Z | `88dbde0` DBA-25 pilot (dependency versions) | `product_state` | "adds no product algorithm change" | NO EVIDENCE OF CHANGE |
| F6 `reconciliation` 10-10 05:34Z | `88dbde0` configuration; new production evidence | `implementation_state`, `production_evidence`, `codeos_yaml` | Conformance and OBS-0002 regression verified | NO EVIDENCE OF CHANGE |
| F4 `early_preview` 10-10 16:17Z | `0fd5954` routing receipts carry the sent request | `product_state` | "preview behavior unchanged" | NO EVIDENCE OF CHANGE |
| F6 `reconciliation` 10-10 16:32Z | `0fd5954`, `7c7a94e` request-bound provider evidence | `implementation_state`, `production_evidence` | Reconciled against request-bound evidence `c8a65397` | YES (judged evidence changed) |
| F8 `final_ux_validation` 10-14 09:36Z | Required-binding repair (`df0051c`); product changed since 09-30 | `candidate_bundle`, `product_state`, `codeos_yaml` newly bound | Human validated the running product for candidate `253fa57a` | UNCERTAIN (old receipt bound no product state) |
| F9 `acceptance` 10-14 09:42Z | Required-binding repair; architecture change after acceptance | `candidate_bundle`, `implementation_state`, `codeos_yaml` newly bound | Human accepted candidate `253fa57a` | YES (architecture and candidate changed) |

The C-6 qualification repair on 2026-10-14 (`082f442`, `79ac5b8`) changed no receipt binding;
F6 stayed current.

### Stage 8 assessments of F-0001

Five valid assessments ran (review record → Candidate Bundle → outcome). The last three ran after
the Charter's approval. One execution-invalid attempt on 2026-10-14 is preserved but not counted.

- `ASSESS__F-0001__8__CANDIDATE-cffd63b9459d` → `cffd63b9` → BLOCKED (2026-10-09)
- `ASSESS__F-0001__8__CANDIDATE-6449208fb72f` → `6449208f` → BLOCKED (2026-10-10)
- `ASSESS__F-0001__8__CANDIDATE-03818120b09c` → `03818120` → BLOCKED (C-6 limiting API; JamFree defect)
- `ASSESS__F-0001__8__CANDIDATE-505b0753aff4` → `505b0753` → BLOCKED (horizon report not inspectable; JamFree evidence-binding gap)
- `ASSESS__F-0001__8__CANDIDATE-253fa57af076` → `253fa57a` → PASSED

F-0001 accepted for `253fa57a` (JamFree `a54772d`); status reports it COMPLETE, current validity
VALID. F-0001 does not count toward WO-03's qualifying features.
