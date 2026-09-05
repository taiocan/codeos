import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import HealthStatus from './HealthStatus.svelte';

test('shows the backend and database status at the decision point', async () => {
	const screen = await render(HealthStatus, { status: 'ok', database: 'connected' });

	await expect.element(screen.getByRole('status')).toHaveTextContent(
		'Backend: ok · Database: connected'
	);
});
