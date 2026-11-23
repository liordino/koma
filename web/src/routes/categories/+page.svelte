<script lang="ts">
	// Category config (wave 20): CRUD with EXPLICIT consent for the destructive flag;
	// purge wired to PurgeCategory behind a backup reminder (stop conditions).
	import { library } from '$lib/koma';
	import type { Category } from '../../gen/koma/v1/koma_pb';

	let cats = $state<Category[]>([]);
	let error = $state('');
	let notice = $state('');
	let newName = $state('');
	let flagDraft: Record<string, boolean> = $state({});
	let nameDraft: Record<string, string> = $state({});
	let purgePickKey = $state('');
	let assignIds: Record<string, number | null> = $state({});

	$effect(() => {
		library()
			.listCategories({})
			.then((r: import('$lib/koma_pb_types').ListCategoriesResponse) => {
				cats = r.items;
				flagDraft = Object.fromEntries(cats.map((c) => [String(c.id), c.deleteAfterRead]));
				nameDraft = Object.fromEntries(cats.map((c) => [String(c.id), c.name]));
			})
			.catch((e: unknown) => (error = String(e)));
	});

	async function create() {
		if (!newName.trim()) return;
		await library().createCategory({ name: newName.trim() });
		newName = '';
		cats = (await library().listCategories({})).items;
	}

	// The destructive flag requires typed consent — flipping it silently is forbidden.
	async function flipFlag(c: Category) {
		const next = flagDraft[c.id];
		if (next && !window.confirm(`Enable delete-after-read for "${c.name}"?\nChapters read here move to trash after the retention window.`)) {
			flagDraft[c.id] = !next; // revert the checkbox — consent denied
			return;
		}
		const updated = await library().updateCategory({ id: c.id, name: nameDraft[String(c.id)], deleteAfterRead: next });
		cats = cats.map((x) => (x.id === c.id ? updated : x));
	}

	async function rename(c: Category) {
		const updated = await library().updateCategory({ id: c.id, name: nameDraft[String(c.id)], deleteAfterRead: flagDraft[String(c.id)] });
		cats = cats.map((x) => (x.id === c.id ? updated : x));
	}

	async function exportBackup() {
		const res = await library().exportBackup({});
		notice = `Backup exported to ${res.path}`;
	}

	async function purge() {
		if (!purgePickKey) {
			error = 'Pick a category to purge first';
			return;
		}
		if (!window.confirm(`Permanently purge trash for the selected category? This cannot be undone.`)) {
			return;
		}
		try {
			const res = await library().purgeCategory({ categoryId: BigInt(purgePickKey), confirm: true });
			notice = `Purged ${res.chaptersPurged} chapter(s)`;
			error = '';
		} catch (e) {
			// Server-side gates surface here: stale backup → reminder to export first.
			error = String(e);
		}
	}

	async function assign(c: Category, mangaId: bigint) {
		await library().addMangaToCategory({ mangaId, categoryId: c.id });
		assignIds[String(c.id)] = null;
		notice = `Manga ${mangaId} added to ${c.name}`;
	}
</script>

<h1>Categories</h1>
{#if error}<p style="color:#f66">{error}</p>{/if}
{#if notice}<p style="color:#6f6">{notice}</p>{/if}

<h2>Create</h2>
<input bind:value={newName} placeholder="new category" />
<button onclick={create}>create</button>

<h2>Configured</h2>
<table>
	<tbody>
		{#each cats as c (c.id)}
			<tr>
				<td>
					<input type="text" bind:value={nameDraft[String(c.id)]} />
					<button onclick={() => rename(c)}>rename</button>
				</td>
				<td>
					<label>
						<input
							type="checkbox"
							bind:checked={flagDraft[String(c.id)]}
							onchange={() => flipFlag(c)}
						/>
						delete after read
					</label>
				</td>
				<td>
					<label>
						add manga id
						<input class="manga-for" data-cat={String(c.id)} type="number" style="width:80px" value={assignIds[String(c.id)] ?? null}
							oninput={(e) => { const v = (e.currentTarget as HTMLInputElement).value; assignIds[String(c.id)] = v === null || v === "" ? null : Number(v); }} placeholder="id" />
					</label>
					<button onclick={() => assignIds[String(c.id)] && assign(c, BigInt(assignIds[String(c.id)] ?? 0))}>add</button>
				</td>
			</tr>
		{/each}
	</tbody>
</table>

<h2>Purge trash (explicit)</h2>
<p>Backups gate this: the server refuses a stale backup — export one first if prompted.</p>
<select bind:value={purgePickKey}>
	<option value="">— pick category —</option>
	{#each cats as c (String(c.id))}
		<option value={String(c.id)}>{c.name}</option>
	{/each}
</select>
<button onclick={exportBackup}>export backup</button>
<button onclick={purge} disabled={!purgePickKey}>purge trash (permanent)</button>
