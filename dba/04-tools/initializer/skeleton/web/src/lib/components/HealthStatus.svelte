<script lang="ts">
	let { status, database }: { status: string; database: string } = $props();
</script>

<p role="status">Backend: {status} · Database: {database}</p>
