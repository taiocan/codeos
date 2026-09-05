import { env } from '$env/dynamic/private';
import type { PageServerLoad } from './$types';

/** Matches docker-compose's `backend` service; override via BACKEND_URL for other topologies. */
const BACKEND_URL = env.BACKEND_URL ?? 'http://localhost:8080';

export const load: PageServerLoad = async ({ fetch }) => {
	try {
		const response = await fetch(`${BACKEND_URL}/health`);
		if (!response.ok) {
			return { status: 'unreachable', database: 'unreachable' };
		}
		const health: { status: string; database: string } = await response.json();
		return health;
	} catch {
		return { status: 'unreachable', database: 'unreachable' };
	}
};
