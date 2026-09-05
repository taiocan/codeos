import { defineConfig } from '@playwright/test';

// This project's e2e tests exercise the real integrated stack — real Postgres, real Rust backend,
// real Svelte app — never a Svelte-only preview standing in for it (Codeos Mechanics `playwright`).
export default defineConfig({
	webServer: {
		command: 'docker compose -f ../docker-compose.yml up --build',
		url: 'http://localhost:3000',
		reuseExistingServer: true,
		timeout: 180_000
	},
	use: { baseURL: 'http://localhost:3000' },
	testMatch: '**/*.e2e.{ts,js}'
});
