<script lang="ts">
	// Library — the manga page: covers as panels in a grid (direction contract FIRST VIEWPORT).
	import { library } from '$lib/koma';
	import { coverObjectUrl } from '$lib/covers';
	import type { Category, MangaSummary } from '../gen/koma/v1/koma_pb';

	let items = $state<MangaSummary[]>([]);
	let cats = $state<Category[]>([]);
	let chip = $state<bigint>(0n);
	let covers = $state<Record<string, string | null>>({});
	let loading = $state(true);
	let error = $state('');

	async function refresh() {
		loading = true;
		error = '';
		try {
			const [lib, cat] = await Promise.all([
				library().listLibrary({ categoryId: chip }),
				library().listCategories({})
			]);
			items = lib.items;
			cats = cat.items;
			covers = {};
			for (const m of items) {
				const id = String(m.id);
				coverObjectUrl(m.id).then((u) => (covers[id] = u));
			}
		} catch (e) {
			error = String(e);
		}
		loading = false;
	}

	// $effect tracks the chip filter and also covers the initial mount.
	$effect(() => {
		void chip;
		refresh();
	});
</script>

<h1 class="display">Library</h1>

<div class="chips" role="tablist" aria-label="Categories">
	<button class="chip" class:active={chip === 0n} onclick={() => (chip = 0n)}>All</button>
	{#each cats as c (String(c.id))}
		<button class="chip" class:active={chip === c.id} onclick={() => (chip = c.id)}>{c.name}</button>
	{/each}
</div>

{#if error}
	<p class="state" role="alert">API not reachable: {error}</p>
{:else if loading}
	<p class="state">loading…</p>
{:else if items.length === 0}
	<p class="state">Nothing here yet — find titles in <a href="/search">Search</a>.</p>
{:else}
	<div class="grid">
		{#each items as m (String(m.id))}
			<a class="card" href="/manga/{m.id}" aria-label={m.title}>
				<div class="panel">
					{#if covers[String(m.id)]}
						<img src={covers[String(m.id)]} alt={m.title} loading="lazy" />
					{:else}
						<!-- placeholder: the panel mark itself, until the cover resolves -->
						<svg
							class="placeholder"
							viewBox="0 0 512 512"
							aria-hidden="true"
						>
							<rect x="64" y="64" width="150" height="384" rx="28" fill="#232c44" />
							<rect x="242" y="64" width="206" height="180" rx="28" fill="#1a2138" />
							<rect x="242" y="268" width="206" height="180" rx="28" fill="#232c44" />
						</svg>
					{/if}
				</div>
				<div class="meta">
					<span class="title">{m.title}</span>
					<span class="badge">{m.sourceKey}</span>
				</div>
			</a>
		{/each}
	</div>
{/if}

<style>
	.chips {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		margin: 4px 0 16px;
	}

	.chip {
		padding: 6px 14px;
		border-radius: 999px;
		background: var(--panel-2);
		color: var(--text-2);
		font-size: 14px;
		font-weight: 500;
	}

	.chip.active {
		background: var(--amber-dim);
		color: var(--amber);
		box-shadow: inset 0 0 0 1px var(--amber);
	}

	.state {
		color: var(--text-2);
		padding: 48px 0;
		text-align: center;
	}

	.state a {
		color: var(--amber);
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
		gap: var(--gutter);
	}

	.card {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.panel {
		position: relative;
		aspect-ratio: 2 / 3;
		border-radius: var(--radius-s);
		overflow: hidden;
		background: var(--panel);
		box-shadow: inset 0 0 0 1px var(--line);
		transition:
			box-shadow 160ms ease,
			transform 160ms ease;
	}

	.card:hover .panel {
		box-shadow:
			inset 0 0 0 1px var(--amber),
			0 8px 24px rgba(0, 0, 0, 0.35);
		transform: translateY(-2px);
	}

	.panel img,
	.placeholder {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.meta {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 8px;
		padding: 0 2px;
	}

	.title {
		font-size: 14px;
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.badge {
		font-size: 11px;
		color: var(--text-2);
		border: 1px solid var(--line);
		border-radius: 4px;
		padding: 1px 6px;
		flex: none;
	}
</style>
