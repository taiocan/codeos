<script lang="ts">
	import HealthStatus from '$lib/components/HealthStatus.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<h1>Codeos Platform Baseline</h1>
<p>An empty, integrated Postgres + Rust + Svelte starting point. Add your first feature here.</p>
<HealthStatus status={data.status} database={data.database} />
