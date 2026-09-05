# web

The GUI tier of the Codeos Platform Baseline: SvelteKit, built with `@sveltejs/adapter-node` for
the `web` service in `../docker-compose.yml`.

```sh
npm install
npm run dev            # local dev server
npm run check           # svelte-check
npm run test:unit       # Vitest (unit + component, Browser Mode via Playwright/Chromium)
npm run test:e2e        # boots the full stack (docker compose) and runs Playwright against it
```

`BACKEND_URL` (default `http://localhost:8080`, set to `http://backend:8080` in compose) points at
the Rust backend. See `dba/05-guidance/patterns/svelte-gui-verification.md` in the toolkit for
verification technique.
