<script lang="ts">
	// Search — the core loop's front door: query the tier-A source, add to the library
	// (optionally into a category), hotlinked source covers until the title is added.
	import { library } from '$lib/koma';
	import type { Category, MangaSummary } from '../../gen/koma/v1/koma_pb';

	let q = $state('');
	let results = $state<MangaSummary[]>([]);
	let cats = $state<Category[]>([]);
	let pick = $state<bigint>(0n);
	let searching = $state(false);
	let searched = $state(false);
	let added = $state<Set<string>>(new Set());
	let error = $state('');

	$effect(() => {
		library()
			.listCategories({})
			.then((r) => (cats = r.items))
			.catch(() => {});
	});

	async function run(e: SubmitEvent) {
		e.preventDefault();
		if (!q.trim() || searching) return;
		searching = true;
		error = '';
		results = [];
		try {
			const r = await library().searchManga({ query: q.trim(), limit: 24 });
			results = r.items;
		} catch (err) {
			error = String(err);
		}
		searched = true;
		searching = false;
	}

	async function add(m: MangaSummary) {
		try {
			await library().addManga({
				sourceKey: m.sourceKey,
				externalId: m.externalId,
				title: m.title,
				coverUrl: m.coverUrl,
				description: m.description,
				categoryId: pick
			});
			added = new Set([...added, m.externalId]);
		} catch (err) {
			error = String(err);
		}
	}
</script>

<h1 class="display">Search</h1>

<form class="searchbar" onsubmit={run}>
	<input
		type="search"
		bind:value={q}
		placeholder="find a title on MangaDex…"
		aria-label="Search query"
	/>
	<button type="submit" class="go" class:busy={searching}>
		{searching ? 'searching…' : 'search'}
	</button>
	<select aria-label="Add new titles to category" bind:value={pick}>
		<option value="0n">add to: no category</option>
		{#each cats as c (String(c.id))}
			<option value={c.id}>add to: {c.name}</option>
		{/each}
	</select>
</form>

{#if error}
	<p class="state" role="alert">{error}</p>
{:else if !searched}
	<p class="state">Type a title — results come straight from MangaDex (tier A).</p>
{:else if searching}
	<p class="state">searching…</p>
{:else if results.length === 0}
	<p class="state">No results.</p>
{:else}
	<div class="grid">
		{#each results as m (m.externalId)}
			<div class="card">
				<div class="panel">
					{#if m.coverUrl}
						<img src={m.coverUrl} alt={m.title} loading="lazy" referrerpolicy="no-referrer" />
					{:else}
						<svg class="placeholder" viewBox="0 0 512 512" aria-hidden="true">
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
				{#if added.has(m.externalId)}
					<span class="in-library" role="status">in library ✓</span>
				{:else}
					<button class="add" onclick={() => add(m)}>add</button>
				{/if}
			</div>
		{/each}
	</div>
{/if}

<style>
	.searchbar {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		margin: 4px 0 16px;
	}

	.searchbar input[type='search'] {
		flex: 1;
		min-width: 200px;
	}

	.go {
		padding: 8px 16px;
		border-radius: var(--radius-s);
		background: var(--amber-dim);
		color: var(--amber);
		font-weight: 700;
		box-shadow: inset 0 0 0 1px var(--amber);
	}

	.go.busy {
		opacity: 0.6;
	}

	.state {
		color: var(--text-2);
		padding: 48px 0;
		text-align: center;
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

	.add {
		padding: 6px 0;
		border-radius: var(--radius-s);
		background: var(--panel-2);
		color: var(--text);
		font-size: 13px;
		font-weight: 500;
		box-shadow: inset 0 0 0 1px var(--line);
	}

	.add:hover {
		box-shadow: inset 0 0 0 1px var(--amber);
		color: var(--amber);
	}

	.in-library {
		text-align: center;
		padding: 6px 0;
		font-size: 13px;
		color: var(--ok);
	}
</style>
