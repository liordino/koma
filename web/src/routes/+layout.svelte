<script lang="ts">
	// App shell — Ink & Amber. Top bar with the panel mark + wordmark, desktop nav inline;
	// phone gets the bottom tab bar (the reading device is the phone — decision 30's scene).
	import '../app.css';
	import { page } from '$app/state';

	let pathname = $derived(page.url.pathname);

	const tabs = [
		{ href: '/', label: 'Library' },
		{ href: '/search', label: 'Search' },
		{ href: '/categories', label: 'Categories' }
	] as const;

	const active = (href: string) =>
		href === '/' ? pathname === '/' : pathname.startsWith(href);

	// Svelte 5 snippet API (<slot/> is deprecated).
	let { children }: { children: import('svelte').Snippet } = $props();
</script>

<div class="shell">
	<header class="topbar">
		<a class="brand" href="/" aria-label="Koma home">
			<svg viewBox="0 0 512 512" width="28" height="28" aria-hidden="true">
				<rect width="512" height="512" rx="112" fill="#1a2138" />
				<rect x="64" y="64" width="150" height="384" rx="28" fill="#e7ebf3" />
				<rect x="242" y="64" width="206" height="180" rx="28" fill="#f59e0b" />
				<rect x="242" y="268" width="206" height="180" rx="28" fill="#e7ebf3" />
				<circle cx="334" cy="149" r="26" fill="#1a2138" />
			</svg>
			<span class="brand-name">Koma</span>
		</a>
		<nav class="tabs-desktop" aria-label="Primary">
			{#each tabs as t (t.href)}
				<a href={t.href} class:active={active(t.href)}>{t.label}</a>
			{/each}
		</nav>
	</header>

	<main>
		{@render children()}
	</main>

	<nav class="tabs-mobile" aria-label="Primary">
		{#each tabs as t (t.href)}
			<a href={t.href} class:active={active(t.href)}>
				<span class="tab-dot" class:lit={active(t.href)}></span>
				{t.label}
			</a>
		{/each}
	</nav>
</div>

<style>
	.shell {
		min-height: 100dvh;
		display: flex;
		flex-direction: column;
	}

	.topbar {
		position: sticky;
		top: 0;
		z-index: 10;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--pad);
		padding: 10px var(--pad);
		background: color-mix(in srgb, var(--ink) 86%, transparent);
		backdrop-filter: blur(10px);
		border-bottom: 1px solid var(--line);
	}

	.brand {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.brand-name {
		font-family: 'Outfit', system-ui, sans-serif;
		font-weight: 700;
		font-size: 20px;
		letter-spacing: -0.02em;
	}

	.tabs-desktop {
		display: none;
		gap: 4px;
	}

	.tabs-desktop a {
		padding: 8px 14px;
		border-radius: var(--radius-s);
		color: var(--text-2);
		font-weight: 500;
	}

	.tabs-desktop a.active {
		color: var(--text);
		background: var(--panel-2);
		box-shadow: inset 0 0 0 1px var(--line);
	}

	main {
		flex: 1;
		padding: var(--pad);
		padding-bottom: calc(64px + env(safe-area-inset-bottom, 0px));
		max-width: 1200px;
		width: 100%;
		margin: 0 auto;
	}

	.tabs-mobile {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		background: color-mix(in srgb, var(--ink) 92%, transparent);
		backdrop-filter: blur(10px);
		border-top: 1px solid var(--line);
		padding-bottom: env(safe-area-inset-bottom, 0px);
		z-index: 10;
	}

	.tabs-mobile a {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
		padding: 10px 0 8px;
		color: var(--text-2);
		font-size: 12px;
		font-weight: 500;
	}

	.tabs-mobile a.active {
		color: var(--text);
	}

	.tab-dot {
		width: 8px;
		height: 8px;
		border-radius: 2px;
		background: var(--line);
	}

	/* the signature move, small: the active tab's panel mark lit amber */
	.tab-dot.lit {
		background: var(--amber);
	}

	@media (min-width: 768px) {
		.tabs-desktop {
			display: flex;
		}

		.tabs-mobile {
			display: none;
		}

		main {
			padding-bottom: var(--pad);
		}
	}
</style>
