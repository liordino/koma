<script lang="ts">
	// Manga detail: hero (cover panel + title + source), chapter list with state as data
	// (downloaded / queued / downloading / failed / pending), download + read actions.
	// While the queue is active, the list polls — the self-driving worker is visible.
	import { onDestroy } from 'svelte';
	import { page } from '$app/state';
	import { library } from '$lib/koma';
	import { coverObjectUrl } from '$lib/covers';
	import type { ChapterInfo, MangaSummary } from '$lib/pb';

	// Route params are a record at runtime; the cast sidesteps narrow per-route typing.
	let mangaId = $derived(BigInt((page.params as Record<string, string>).id ?? '0'));
	let manga = $state<MangaSummary | null>(null);
	let chapters = $state<ChapterInfo[]>([]);
	let cover = $state<string | null>(null);
	let error = $state('');
	let loading = $state(true);
	let timer: ReturnType<typeof setInterval> | undefined;

	async function refresh() {
		try {
			const [m, ch] = await Promise.all([
				library().getManga({ id: mangaId }),
				library().listChapters({ mangaId })
			]);
			manga = m.manga ?? null;
			chapters = ch.items;
			cover = await coverObjectUrl(mangaId);
		} catch (e) {
			error = String(e);
		}
		loading = false;
	}

	$effect(() => {
		void mangaId;
		loading = true;
		refresh();
	});

	// Poll while the queue is working so downloads visibly complete.
	$effect(() => {
		const active = chapters.some((c) => c.state === 'queued' || c.state === 'downloading');
		if (active && !timer) {
			timer = setInterval(refresh, 2500);
		} else if (!active && timer) {
			clearInterval(timer);
			timer = undefined;
		}
	});

	onDestroy(() => {
		if (timer) clearInterval(timer);
	});

	async function download(chapterId: bigint) {
		try {
			await library().downloadChapter({ chapterId });
			chapters = chapters.map((c) =>
				c.id === chapterId && c.state === 'pending' ? { ...c, state: 'queued' } : c
			);
		} catch (e) {
			error = String(e);
		}
	}

	function stateClass(state: string) {
		return `st-${state}`;
	}
</script>

{#if error}
	<p class="state" role="alert">{error}</p>
{/if}

{#if loading}
	<p class="state">loading…</p>
{:else if manga}
	<header class="hero">
		<div class="cover">
			{#if cover}
				<img src={cover} alt={manga.title} />
			{:else}
				<svg class="placeholder" viewBox="0 0 512 512" aria-hidden="true">
					<rect x="64" y="64" width="150" height="384" rx="28" fill="#232c44" />
					<rect x="242" y="64" width="206" height="180" rx="28" fill="#1a2138" />
					<rect x="242" y="268" width="206" height="180" rx="28" fill="#232c44" />
				</svg>
			{/if}
		</div>
		<div class="head">
			<h1 class="display">{manga.title}</h1>
			<p class="source">source: {manga.sourceKey}</p>
			<p class="count">{chapters.length} chapter{chapters.length === 1 ? '' : 's'} tracked</p>
		</div>
	</header>

	<h2>Chapters</h2>
	{#if chapters.length === 0}
		<p class="state">No chapters yet — run an update pass or add from Search.</p>
	{:else}
		<ul class="chapters">
			{#each chapters as c (String(c.id))}
				<li class="chapter">
					<span class="num">{c.number}</span>
					<span class="badge {stateClass(c.state)}" title={c.state}>{c.state}</span>
					{#if c.pages > 0}
						<span class="pages">{c.pages}p</span>
					{/if}
					<span class="spacer"></span>
					{#if c.state === 'downloaded'}
						<a class="read" href="/reader/{c.id}">read</a>
					{:else if c.state === 'queued' || c.state === 'downloading'}
						<span class="busy">working…</span>
					{:else}
						<button class="dl" onclick={() => download(c.id)}>download</button>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
{/if}

<style>
	.hero {
		display: flex;
		gap: var(--pad);
		align-items: flex-end;
		margin-bottom: var(--pad);
	}

	.cover {
		position: relative;
		width: 160px;
		flex: none;
		aspect-ratio: 2 / 3;
		border-radius: var(--radius);
		overflow: hidden;
		background: var(--panel);
		box-shadow:
			inset 0 0 0 1px var(--line),
			0 10px 28px rgba(0, 0, 0, 0.4);
	}

	.cover img,
	.placeholder {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.head h1 {
		margin: 0;
		font-size: 28px;
		line-height: 1.15;
	}

	.source,
	.count {
		color: var(--text-2);
		margin: 4px 0 0;
	}

	h2 {
		margin: 8px 0 8px;
	}

	.state {
		color: var(--text-2);
		padding: 48px 0;
		text-align: center;
	}

	.chapters {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
	}

	.chapter {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 8px;
		border-bottom: 1px solid var(--line);
	}

	.chapter:first-child {
		border-top: 1px solid var(--line);
	}

	.num {
		font-weight: 700;
		font-variant-numeric: tabular-nums;
		min-width: 64px;
	}

	.badge {
		font-size: 11px;
		border-radius: 4px;
		padding: 1px 8px;
		border: 1px solid var(--line);
		color: var(--text-2);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.badge.st-downloaded {
		color: var(--ok);
		border-color: color-mix(in srgb, var(--ok) 40%, transparent);
	}

	.badge.st-queued,
	.badge.st-downloading {
		color: var(--queued);
		border-color: color-mix(in srgb, var(--queued) 40%, transparent);
	}

	.badge.st-failed {
		color: var(--danger);
		border-color: color-mix(in srgb, var(--danger) 40%, transparent);
	}

	.pages {
		color: var(--text-2);
		font-variant-numeric: tabular-nums;
		font-size: 13px;
	}

	.spacer {
		flex: 1;
	}

	.read {
		color: var(--amber);
		font-weight: 700;
		padding: 4px 12px;
		border-radius: var(--radius-s);
		box-shadow: inset 0 0 0 1px var(--amber);
	}

	.read:hover {
		background: var(--amber-dim);
	}

	.dl {
		padding: 4px 12px;
		border-radius: var(--radius-s);
		background: var(--panel-2);
		box-shadow: inset 0 0 0 1px var(--line);
		color: var(--text);
		font-size: 13px;
	}

	.dl:hover {
		box-shadow: inset 0 0 0 1px var(--amber);
		color: var(--amber);
	}

	.busy {
		color: var(--text-2);
		font-size: 13px;
	}
</style>
