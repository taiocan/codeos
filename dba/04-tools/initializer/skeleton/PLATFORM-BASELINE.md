# Platform Baseline

This is the Codeos Platform Baseline: an empty, integrated PostgreSQL + Rust + Svelte + Docker
starting point (`dba/01-doctrine/v5.md`, Platform Baseline). It exists so a feature is visible
end-to-end from the first cycle that implements it, instead of the GUI arriving separately later.

```sh
docker compose up --build
```

brings up all three tiers. Visit <http://localhost:3000> — it shows the backend and database
status, proving the integration actually works before any feature is added.

| Directory | Tier | README |
|---|---|---|
| `backend/` | Rust (axum + sqlx) | `backend/README.md` |
| `web/` | Svelte (SvelteKit) | `web/README.md` |

Add your first feature through the normal DBA lifecycle
(`.codeos/toolkit/dba/03-prompts/workflow/support-solution-charter.md`, then Stage 1 onward). Each
feature's Stage 4 implementation extends whichever tiers its Feature Impact Accounting marks
changed — this file does not get bigger as features are added; the two READMEs above do not either.
