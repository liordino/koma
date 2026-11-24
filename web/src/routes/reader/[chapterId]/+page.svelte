<script lang="ts">
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { library } from '$lib/koma';
	import { fetchPageBlob } from '$lib/pages';

	// Windowed reader (decision 10): preload ±2–3 pages, fixed-size placeholders, unmount far
	// pages; page-anchored progress so resume survives reload and device changes.
	const PAGE_H = 1200; // fixed px per page — makes page math exact regardless of native image size
	const WINDOW = 2; // loaded pages: current ±2

	let chapterId = $derived(BigInt(page.params.chapterId ?? '0'));
	let pagesCount = $state(0);
	let current = $state(0);
	let blobs: (string | undefined)[] = [];
	let scrollTimer: ReturnType<typeof setTimeout> | undefined;

	// The set of page indexes currently in the DOM — far pages unmount, blobs stay cached.
	let rendered = $derived(
		pagesCount === 0 ? [] : range(Math.max(0, current - WINDOW), Math.min(pagesCount - 1, current + WINDOW))
	);

	// The amber strip: reading position as panel progress (the signature move).
	let progress = $derived(pagesCount <= 1 ? 0 : (current / (pagesCount - 1)) * 100);

	function range(a: number, b: number): number[] {
		const out: number[] = [];
		for (let i = a; i <= b; i++) out.push(i);
		return out;
	}

	async function countPages() {
		// Progressive discovery: pages are a contiguous run ending at the first miss.
		let n = 0;
		while (n < 64) {
			const blob = await fetchPageBlob(chapterId, n);
			if (!blob) break;
			blobs[n] = blob;
			n++;
		}
		pagesCount = n;
	}

	async function saveProgress() {
		const ratio = pagesCount <= 1 ? 0 : Math.min(1, current / (pagesCount - 1));
		await library().upsertProgress({ chapterId, pageN: current, scrollRatio: ratio });
	}

	function onScroll() {
		if (scrollTimer) return;
		scrollTimer = setTimeout(() => {
			scrollTimer = undefined;
			const y = window.scrollY;
			const next = Math.min(pagesCount - 1, Math.max(0, Math.round(y / PAGE_H)));
			if (next !== current) {
				current = next;
				saveProgress();
			}
		}, 200);
	}

	onMount(async () => {
		// Resume BEFORE layout: read stored progress, discover page count, jump to the anchor.
		const progress = await library().getProgress({ chapterId }).catch(() => null);
		await countPages();
		if (progress?.exists && progress.pageN < pagesCount) {
			current = progress.pageN;
		}
		await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
		if (current > 0) {
			window.scrollTo({ top: current * PAGE_H });
		}
	});
</script>

<svelte:window onscroll={onScroll} />

<div class="pill" aria-live="polite">
	<!-- decision 30: strip-only v1 — stated in the UI per the recorded decision -->
	<small title="decision 30: vertical strip reading is the only mode in v1">strip · </small>
	{pagesCount === 0 ? 'loading…' : `page ${current + 1} / ${pagesCount}`}
</div>

{#each pagesCount === 0 ? [] : range(0, pagesCount - 1) as i (i)}
	<div class="page-slot">
		{#if rendered.includes(i)}
			<img src={blobs[i]} alt={`page ${i + 1}`} />
		{:else}
			<!-- fixed-size placeholder: scroll stays stable while far pages are unmounted -->
			<span class="dot">·</span>
		{/if}
	</div>
{/each}

<div class="strip" aria-hidden="true">
	<div class="strip-fill" style="width: {progress}%"></div>
</div>

<style>
	.pill {
		position: sticky;
		top: 10px;
		z-index: 5;
		margin: 0 auto;
		width: fit-content;
		padding: 6px 16px;
		border-radius: 999px;
		background: color-mix(in srgb, var(--ink) 82%, transparent);
		backdrop-filter: blur(8px);
		border: 1px solid var(--line);
		color: var(--text-2);
		font-size: 14px;
		font-variant-numeric: tabular-nums;
		text-align: center;
	}

	.pill small {
		text-transform: uppercase;
		font-size: 10px;
		letter-spacing: 0.08em;
		color: var(--amber);
		margin-right: 8px;
	}

	.page-slot {
		height: var(--page-h, 1200px);
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--panel);
	}

	.page-slot img {
		max-width: 100%;
		max-height: 100%;
	}

	.dot {
		color: var(--line);
	}

	/* the amber strip: reading position along the bottom edge (above the mobile tab bar) */
	.strip {
		position: fixed;
		left: 0;
		right: 0;
		bottom: calc(52px + env(safe-area-inset-bottom, 0px));
		height: 3px;
		background: var(--panel-2);
		z-index: 6;
	}

	@media (min-width: 768px) {
		.strip {
			bottom: 0;
		}
	}

	.strip-fill {
		height: 100%;
		background: var(--amber);
		transition: width 240ms ease;
	}
</style>
