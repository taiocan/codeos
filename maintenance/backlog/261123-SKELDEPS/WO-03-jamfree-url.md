# WO-SD-03 — JamFree url 2.5.9 (Project-Owned)

- **Class:** project-owned; deferred to JamFree's next feature cycle.
- **Scope:** JamFree updates url to 2.5.9 in `backend/Cargo.lock` under its own DBA workflow, in
  the same cycle as its next feature.
- **Why not now:** the lockfile is a dependency input to candidate bundles and evidence scope, so a
  standalone bump forces revalidation, with its cost and TomTom calls. Backend platform assurance
  and supply-chain health are already STALE and will be refreshed in that cycle anyway.
- **Codeos role:** tracking only. The next static snapshot confirms the result.
- **Re-checked 2026-11-23:** JamFree had no feature in progress, and `backend/src` never calls
  `Url::origin()`, the vulnerable function, so practical exposure is low. The human kept the
  deferral.
