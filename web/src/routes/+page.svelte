<script lang="ts">
	import { library } from '$lib/koma';
	import type { MangaSummary } from '../gen/koma/v1/koma_pb';

	let items = $state<MangaSummary[]>([]);
	let error = $state('');

	$effect(() => {
		library()
			.listLibrary({ categoryId: 0 })
			.then((res) => (items = res.items))
			.catch((e) => (error = String(e)));
	});
</script>

<h1>Koma</h1>
{#if error}<p>API not reachable: {error}</p>{/if}
<ul>
	{#each items as m (m.id)}
		<!-- Chapter 1 is the bench chapter seeded by the server seed tool (M1 test setup). -->
		<li><a href={`/reader/1`}>{m.title} <small>({m.sourceKey})</small></a></li>
	{/each}
</ul>
