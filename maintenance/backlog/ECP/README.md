# ECP — Engineering Control Panel Work Orders

[Charter](charter.md) (approved 2026-10-10) · baseline: Control Panel v1 (`534c0fe`). Planning
context only; the Charter and Git are authoritative.

| WO | Scope | Class | Title | Status |
|---|---|---|---|---|
| [WO-01](WO-01-engineering-health.md) | ECP-2 | NORMAL | Engineering Health projection | Implemented |
| [WO-02](WO-02-attention-explanation.md) | ECP-3 | NORMAL | Attention class, reason, authority | Implemented |
| [WO-03](WO-03-changed-surfaces.md) | ECP-3 | NORMAL | Changed surfaces and evidence identities | Implemented |
| [WO-04](WO-04-process-history.md) | ECP-4 | NORMAL | Process history | Proposed |
| [WO-05](WO-05-structured-action.md) | ECP-5 | NORMAL | Structured action | Proposed |
| [WO-06](WO-06-jamfree-validation.md) | Real use | NORMAL | JamFree pilot validation | Proposed |
| [WO-07](WO-07-auto-execution-decision.md) | ECP-6 | PROTECTED | Bounded AUTO execution — decision brief | Proposed |

Each WO follows the previous one (WO-04 needs only WO-01 but stays in sequence).

## Rules for every WO

1. **Basis.** Every derived value MUST have an explicit authoritative basis or an absence state
   (`UNOBSERVED`, `NOT_CONFIGURED`, `NOT_APPLICABLE`, `UNRESOLVED`). Silence or inference from
   unrelated facts is never a basis.
2. **Read-only (C-1).** `status` writes no file; the `monitor_status.rs` snapshot test covers new
   output.
3. **Compatibility (C-11).** Status JSON schema v3 changes are additive; rename, removal, or changed
   meaning needs a version bump. Exit codes, progression, and `check` are unchanged.
4. **Escalation.** Needing a new persisted record or authority makes a WO PROTECTED; record it as
   an open decision, do not build it.
5. **Delivery.** Each WO is independently reviewable and revertible; prefer one commit citing the
   WO ID.
6. **Verification baseline.** `cargo test` and `cargo clippy --all-targets` in
   `dba/04-tools/workflow/engine`; guidance and layout contract scripts; a new governed term gets a
   glossary entry in the same change.

## Charter coverage

| Charter item | WO |
|---|---|
| ECP-2, O-1 | WO-01 |
| ECP-3, O-2, §6 | WO-02, WO-03 |
| ECP-4, O-3 | WO-04 |
| ECP-5, O-4 | WO-05 |
| Success criteria 1–10 | WO-06 |
| ECP-6, O-5, success criteria 11–13 | WO-07 (decision only) |
| C-1, C-2, C-11, C-12 | Rules 1–3 |
| C-3 | WO-01, WO-03 |
| C-4, C-5 | WO-01 |
| C-6 | WO-01, WO-04 |
| C-7 | all; WO-05 |
| C-8 | WO-05, WO-07 |
| C-9 | WO-01, WO-06 |
| C-10 | WO-02, WO-03 |
