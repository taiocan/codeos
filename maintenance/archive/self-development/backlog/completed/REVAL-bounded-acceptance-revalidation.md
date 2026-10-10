# REVAL — Bounded Acceptance Revalidation

**CLOSED — 2026-10-10: existing capability confirmed under Workflow Governance v4; the original
gap was ECP action projection, repaired in [ECP WO-10](../../../../backlog/ECP/WO-10-atomic-revalidation-actions.md).**

Source: ECP WO-06 gap E reported `revalidate_acceptance` (AUTO) with no mechanism. The mechanism
exists: `check --revalidation --trigger`, `prepare-candidate --revalidation --trigger`, and an
independent Stage 8 assessment whose `PASSED` outcome restores current validity without an F9
receipt or any rewrite of the accepted snapshot.
