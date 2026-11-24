<script lang="ts">
	// Categories — cards, not rows (Ink & Amber). Every safety flow survives the redesign:
	// consent-gated destructive flag, backup-gated purge, explicit confirm on permanent actions.
	import { library } from '$lib/koma';
	import type { Category } from '$lib/pb';

	let cats = $state<Category[]>([]);
	let counts = $state<Record<string, number>>({});
	let error = $state('');
	let notice = $state('');
	let newName = $state('');
	let flagDraft: Record<string, boolean> = $state({});
	let nameDraft: Record<string, string> = $state({});
	let assignIds: Record<string, number | null> = $state({});
	let busy = $state<string | null>(null);

	async function refresh() {
		try {
			const r = await library().listCategories({});
			cats = r.items;
			flagDraft = Object.fromEntries(cats.map((c) => [String(c.id), c.deleteAfterRead]));
			nameDraft = Object.fromEntries(cats.map((c) => [String(c.id), c.name]));
			for (const c of cats) {
				library()
					.listLibrary({ categoryId: c.id })
					.then((lib) => (counts[String(c.id)] = lib.items.length))
					.catch(() => {});
			}
		} catch (e) {
			error = String(e);
		}
	}

	refresh();

	async function create() {
		if (!newName.trim()) return;
		await library().createCategory({ name: newName.trim() });
		newName = '';
		await refresh();
	}

	// The destructive flag requires typed consent — flipping silently is forbidden.
	async function flipFlag(c: Category) {
		const next = flagDraft[String(c.id)];
		if (next && !window.confirm(`Enable delete-after-read for "${c.name}"? Chapters read here move to trash after the retention window.`)) {
			flagDraft[String(c.id)] = !next; // consent denied — revert the checkbox
			return;
		}
		const updated = await library().updateCategory({ id: c.id, name: nameDraft[String(c.id)], deleteAfterRead: next });
		const cat = updated.category ?? c;
		flagDraft[String(c.id)] = cat.deleteAfterRead;
		notice = `${cat.name}: delete after read ${cat.deleteAfterRead ? 'enabled' : 'disabled'}`;
	}

	async function rename(c: Category) {
		const updated = await library().updateCategory({ id: c.id, name: nameDraft[String(c.id)], deleteAfterRead: flagDraft[String(c.id)] });
		notice = `renamed to ${(updated.category ?? c).name}`;
		await refresh();
	}

	async function assign(c: Category, mangaId: number) {
		await library().addMangaToCategory({ mangaId: BigInt(mangaId), categoryId: c.id });
		assignIds[String(c.id)] = null;
		notice = `added title #${mangaId} to ${c.name}`;
		counts[String(c.id)] = (counts[String(c.id)] ?? 0) + 1;
	}

	async function exportBackup() {
		const res = await library().exportBackup({});
		notice = `Backup exported to ${res.path}`;
	}

	// Permanent purge: explicit confirm; the server refuses stale backups (surfaced here).
	async function purge(c: Category) {
		if (!window.confirm(`Permanently purge trash for "${c.name}"? This cannot be undone.`)) {
			return;
		}
		busy = `purge-${String(c.id)}`;
		try {
			const res = await library().purgeCategory({ categoryId: c.id, confirm: true });
			notice = `Purged ${res.chaptersPurged} chapter(s) from ${c.name}`;
			error = '';
		} catch (e) {
			error = String(e);
		}
		busy = null;
	}
</script>

<h1 class="display">Categories</h1>

{#if error}<p class="msg err" role="alert">{error}</p>{/if}
{#if notice}<p class="msg ok" role="status">{notice}</p>{/if}

<div class="toolbar">
	<input placeholder="new category" bind:value={newName} aria-label="New category name" />
	<button class="primary" onclick={create}>create</button>
	<button class="ghost" onclick={exportBackup}>export backup</button>
</div>

{#if cats.length === 0}
	<p class="empty">No categories yet — create one to organize the library.</p>
{:else}
	<div class="cards">
		{#each cats as c (String(c.id))}
			<div class="cat-card">
				<div class="card-head">
					<input type="text" bind:value={nameDraft[String(c.id)]} aria-label={`Rename ${c.name}`} />
					<button class="ghost" onclick={() => rename(c)}>rename</button>
				</div>
				<p class="count">{counts[String(c.id)] ?? '…'} title{counts[String(c.id)] === 1 ? '' : 's'}</p>
				<label class="flag">
					<input
						type="checkbox"
						bind:checked={flagDraft[String(c.id)]}
						onchange={() => flipFlag(c)}
					/>
					delete after read
				</label>
				<div class="assign">
					<input
						class="manga-for"
						data-cat={String(c.id)}
						type="number"
						style="width:80px"
						value={assignIds[String(c.id)] ?? null}
						oninput={(e) => {
							const v = (e.currentTarget as HTMLInputElement).value;
							assignIds[String(c.id)] = v === '' ? null : Number(v);
						}}
						placeholder="id"
					/>
					<button class="ghost" onclick={() => assignIds[String(c.id)] && assign(c, assignIds[String(c.id)] ?? 0)}>add</button>
				</div>
				<div class="card-foot">
					<button class="danger" class:busy={busy === `purge-${String(c.id)}`} onclick={() => purge(c)}>
						purge trash (permanent)
					</button>
				</div>
			</div>
		{/each}
	</div>
	<p class="hint">Backups gate purges: the server refuses a stale backup — export one first if prompted.</p>
{/if}

<style>
	.toolbar {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		margin: 4px 0 16px;
	}

	.toolbar input {
		flex: 1;
		min-width: 180px;
	}

	.primary {
		padding: 8px 16px;
		border-radius: var(--radius-s);
		background: var(--amber-dim);
		color: var(--amber);
		font-weight: 700;
		box-shadow: inset 0 0 0 1px var(--amber);
	}

	.ghost {
		padding: 8px 12px;
		border-radius: var(--radius-s);
		background: var(--panel-2);
		box-shadow: inset 0 0 0 1px var(--line);
		color: var(--text);
		font-size: 14px;
	}

	.ghost:hover {
		color: var(--amber);
		box-shadow: inset 0 0 0 1px var(--amber);
	}

	.msg {
		border-radius: var(--radius-s);
		padding: 10px 14px;
		margin: 0 0 12px;
	}

	.msg.err {
		background: color-mix(in srgb, var(--danger) 12%, transparent);
		color: var(--danger);
	}

	.msg.ok {
		background: var(--amber-dim);
		color: var(--amber);
	}

	.empty {
		color: var(--text-2);
		padding: 48px 0;
		text-align: center;
	}

	.cards {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
		gap: var(--gutter);
	}

	.cat-card {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: var(--pad);
		border-radius: var(--radius);
		background: var(--panel);
		box-shadow: inset 0 0 0 1px var(--line);
	}

	.card-head {
		display: flex;
		gap: 8px;
	}

	.card-head input {
		flex: 1;
	}

	.count {
		margin: 0;
		color: var(--text-2);
		font-size: 13px;
	}

	.flag {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--text-2);
		font-size: 14px;
	}

	.assign {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.card-foot {
		margin-top: auto;
		padding-top: 6px;
		border-top: 1px solid var(--line);
	}

	.danger {
		padding: 8px 0;
		width: 100%;
		border-radius: var(--radius-s);
		background: color-mix(in srgb, var(--danger) 10%, transparent);
		color: var(--danger);
		font-size: 13px;
		box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--danger) 40%, transparent);
	}

	.danger.busy {
		opacity: 0.6;
	}

	.hint {
		color: var(--text-2);
		font-size: 12px;
		margin-top: 12px;
	}
</style>
