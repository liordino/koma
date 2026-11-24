import { expect, test } from '@playwright/test';

// Wave 25 gate: the detail page renders the real imported title, shows chapter state
// as data, and the downloaded chapter's read link lands in the reader.
const API = 'http://127.0.0.1:18100';
const TOKEN = 't';

test('detail: title, chapter list with states, read link into the reader', async ({ page }) => {
	await page.addInitScript(([api, token]: string[]) => {
		localStorage.setItem('koma_api', api);
		localStorage.setItem('koma_token', token);
	}, [API, TOKEN]);

	await page.goto('http://localhost:5173/manga/1');
	await expect(page.locator('h1')).toHaveText('One Piece', { timeout: 30_000 });
	await expect(page.locator('.chapter').first()).toBeVisible({ timeout: 30_000 });
	const rows = await page.locator('.chapter').count();
	expect(rows).toBeGreaterThanOrEqual(20);

	// the seeded downloaded chapter carries the green state + a read link
	await expect(page.locator('.badge.st-downloaded').first()).toBeVisible();
	const read = page.locator('.chapter', { has: page.locator('.st-downloaded') }).locator('a.read').first();
	await read.click();
	await expect(page.getByText(/page 1 \/ 10/)).toBeVisible({ timeout: 30_000 });
});

test('detail: download button enqueues (live MangaDex flow)', async ({ page }) => {
	await page.addInitScript(([api, token]: string[]) => {
		localStorage.setItem('koma_api', api);
		localStorage.setItem('koma_token', token);
	}, [API, TOKEN]);

	await page.goto('http://localhost:5173/manga/1');
	// a pending (not yet downloaded) chapter row offers download; enqueue flips it to queued
	const dl = page.locator('.chapter .dl').first();
	if (await dl.count()) {
		await dl.click();
		await expect(page.locator('.badge.st-queued').first()).toBeVisible({ timeout: 15_000 });
	}
});
