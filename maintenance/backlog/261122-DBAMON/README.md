# DBAMON — DBA Monitor (Power BI projection)

Planning context only. Git remains authoritative. Approved by the human 2026-11-22.

**Principle:** CodeOS owns truth; the monitor owns historical projection and analysis.

The monitor gives one project's DBA system, and CodeOS itself, a historical view in Power BI:
flow and rework, test health, review effectiveness, and later code health and governance-surface
growth. It reuses the AII snapshot pattern: an immutable `snapshots/<id>/` folder whose
`manifest.json` holds sha256 hashes and row counts, an atomically written `current.json`, and a
`verify-active` check that blocks the Power BI refresh when it fails.

## Boundaries (apply to every WO)

1. **No second authority.** Validity, health, attention and acceptance come verbatim from CodeOS.
   DAX may only count, sum, take time differences and filter. The report is never evidence.
2. **Additive-field test.** A WO may add a field to CodeOS records or projections only if CodeOS
   would want that field even if Power BI did not exist.
3. **Diagnostics only.** No composite score and no targets. Proxies are labelled as
   CodeOS-specific approximations, not DORA metrics.
4. **Small samples.** Use counts, lists and drill-down; show a median only where n ≥ 10.
5. **UNOBSERVED stays null.** Nothing is estimated.
6. **Segments.** Comparisons are split by `dba_configuration`, release, reviewer build and Codex CLI
   version.
7. **Disclosure.** Metadata only, no packet text. The report is marked not publishable.

Out of scope: coverage, mutation score, complexity metrics, Anthropic-token attribution,
cross-project comparison, and any use of the monitor as workflow, validity or acceptance input.

| WO | Class | Title | Status |
|---|---|---|---|
| [WO-01](WO-01-review-status-enrichment.md) | NORMAL | Enrich `review-status --json` per assessment | Accepted 2026-11-22 |
| [WO-02](WO-02-structured-test-durations.md) | NORMAL (PROTECTED if identity inputs change) | Structured test-evidence durations | Accepted 2026-11-22 — first real run pending observation |
| [WO-03](WO-03-project-monitor-snapshot.md) | NORMAL | Immutable project-monitor snapshot | Accepted 2026-11-22 |
| [WO-04](WO-04-toolkit-and-code-analytics.md) | NORMAL | Toolkit governance growth and git analytics | Accepted 2026-11-22 |
| [WO-05](WO-05-power-bi-first-pages.md) | PowerBI repo | First three Power BI pages | Accepted 2026-11-22 |
| [WO-06](WO-06-deferred-pages.md) | Deferred | Further pages and instrumentation | Deferred until real use shows value |
| [WO-07](WO-07-static-and-security-summaries.md) | NORMAL | Static and security summaries (split from WO-04) | Accepted 2026-11-22 |
| [WO-08](WO-08-report-guidance.md) | PowerBI repo | In-report guidance for novice and CodeOS-developer readers | Approved 2026-11-23 — not started |
| [WO-09](WO-09-monthly-assessment.md) | PowerBI repo | Monthly development assessment (advisory narrative) | Approved 2026-11-23 — not started |

This table is the only place WO status is kept.
