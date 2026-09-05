import { expect, test } from '@playwright/test';

// Integrated smoke test (Codeos Mechanics `smoke` / `playwright`). Requires the real stack running
// — `docker compose up` — so this proves the actual DB<->backend<->GUI path, not a mocked tier.
test('home page shows a connected backend and database', async ({ page }) => {
	await page.goto('/');

	await expect(page.getByRole('status')).toHaveText('Backend: ok · Database: connected');
});
