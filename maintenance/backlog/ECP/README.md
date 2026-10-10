# ECP — Engineering Control Panel Work Orders

Source: [`charter.md`](charter.md) (ECP-2 to ECP-6), approved 2026-10-10.
Baseline: Control Panel v1 (ECP-1), commit `534c0fe`.

These work orders are planning context, not workflow state or implementation authority.
The Charter and Git remain authoritative.

**Prerequisite:** Charter approval — satisfied 2026-10-10.

## Sequence

| WO | Charter scope | Class | Title | Status |
|---|---|---|---|---|
| [WO-01](WO-01-engineering-health.md) | ECP-2 | NORMAL | Engineering Health projection | Proposed |
| [WO-02](WO-02-attention-explanation.md) | ECP-3 | NORMAL | Attention class, reason, authority | Proposed |
| [WO-03](WO-03-changed-surfaces.md) | ECP-3 | NORMAL | Changed surfaces and evidence identities | Proposed |
| [WO-04](WO-04-process-history.md) | ECP-4 | NORMAL | Process history | Proposed |
| [WO-05](WO-05-structured-action.md) | ECP-5 | NORMAL | Structured action | Proposed |
| [WO-06](WO-06-jamfree-validation.md) | Real use | NORMAL | JamFree pilot validation | Proposed |
| [WO-07](WO-07-auto-execution-decision.md) | ECP-6 | PROTECTED | Bounded AUTO execution — decision brief | Proposed |

Each WO depends on the one before it. WO-04 technically needs only WO-01, but it stays in sequence
to keep delivery controlled.

## Rules for every WO

1. **Basis rule.** Every derived value MUST have an explicit authoritative basis or an explicit
   absence state (`UNOBSERVED`, `NOT_CONFIGURED`, `NOT_APPLICABLE`, `UNRESOLVED`). Silence, or
   inference from unrelated facts, is never a basis.
2. **Read-only (C-1).** `status` creates or modifies no file. The snapshot assertion in
   `engine/tests/monitor_status.rs` covers each new output.
3. **Compatibility (C-11).** Additions to status JSON schema v3 are additive. A rename, a removal,
   or a change of meaning requires a schema version bump. Exit codes, progression, and `check`
   behavior do not change.
4. **Escalation.** A WO that needs a new persisted record or a new authority becomes PROTECTED.
   It records the need as an open decision and does not build it.
5. **Delivery.** Each WO is independently reviewable and revertible. Prefer one commit where
   practical. Commit messages cite the WO ID.
6. **Verification baseline.** In `dba/04-tools/workflow/engine`: `cargo test` and
   `cargo clippy --all-targets`. Also run `dba/04-tools/configuration/guidance-contract.sh` and
   `layout-contract.sh`. A new governed term needs a glossary entry
   (`dba/05-guidance/terminology.md`) in the same change.

## Charter coverage

| Charter item | WO |
|---|---|
| ECP-2, O-1 | WO-01 |
| ECP-3, O-2, §6 attention model | WO-02, WO-03 |
| ECP-4, O-3 | WO-04 |
| ECP-5, O-4 | WO-05 |
| Real use, success criteria 1–10 | WO-06 |
| ECP-6, O-5, success criteria 11–13 | WO-07 (decision only) |
| C-1 read-only, C-2 existing truth, C-11 compatibility, C-12 lean runtime | Rules 1–3, all WOs |
| C-3 acceptance vs validity | WO-01, WO-03 |
| C-4 no fabricated health, C-5 project-owned thresholds | WO-01 |
| C-6 facts over scores | WO-01, WO-04 |
| C-7 deterministic derivation | all WOs; WO-05 |
| C-8 human authority | WO-05, WO-07 |
| C-9 no domain leakage | WO-01, WO-06 |
| C-10 explainability | WO-02, WO-03 |
