import { expect, test } from '@playwright/test';

// Wave 26 gate: categories as cards — create, rename, consent-gated destructive flag,
// backup export, per-card purge with confirm. The safety flows are the point.
// Input values are asserted via toHaveValue (the DOM property): Svelte's bind:value
// never writes the value attribute in client-only render.
const API = process.env.KOMA_API ?? 'http://127.0.0.1:18100';
const TOKEN = process.env.KOMA_TOKEN ?? 't';
const UI = 'http://localhost:5173';

async function init(page: import('@playwright/test').Page) {
	await page.addInitScript(([api, token]: string[]) => {
		localStorage.setItem('koma_api', api);
		localStorage.setItem('koma_token', token);
	}, [API, TOKEN]);
}

test('categories: create, rename, consent flag, backup + per-card purge', async ({ page }) => {
	await init(page);
	await page.goto(`${UI}/categories`);
	await expect(page.getByRole('heading', { name: 'Categories' })).toBeVisible({ timeout: 30_000 });

	// create (unique name per run) — fresh DB, so it is the only card
	const catName = `test-cat-${Date.now().toString(36)}`;
	await page.getByPlaceholder('new category').fill(catName);
	await page.getByRole('button', { name: 'create' }).click();
	await expect(page.locator('.cat-card')).toHaveCount(1, { timeout: 15_000 });
	const nameInput = page.locator('.cat-card input[type="text"]');
	await expect(nameInput).toHaveValue(catName);

	// rename via the card's name input, then reload for server persistence
	await nameInput.fill(`${catName}-renamed`);
	await page.getByRole('button', { name: 'rename' }).click();
	await page.waitForTimeout(400);
	await page.reload();
	await expect(page.locator('.cat-card')).toHaveCount(1);
	await expect(page.locator('.cat-card input[type="text"]')).toHaveValue(`${catName}-renamed`);

	// consent-gated destructive flag: the typed warning must be accepted
	page.once('dialog', (d) => d.accept());
	await page.locator('.cat-card input[type="checkbox"]').check();
	await page.waitForTimeout(400);

	// backup first (the purge gate requires freshness), then per-card purge with confirm
	await page.getByRole('button', { name: 'export backup' }).click();
	await expect(page.getByText(/Backup exported to/)).toBeVisible({ timeout: 15_000 });

	page.once('dialog', (d) => d.accept());
	await page.getByRole('button', { name: /purge trash/ }).click();
	await expect(page.getByText(/Purged \d+ chapter/)).toBeVisible({ timeout: 15_000 });
});

test('categories: consent denial reverts the destructive flag', async ({ page }) => {
	await init(page);
	await page.goto(`${UI}/categories`);
	await expect(page.getByRole('heading', { name: 'Categories' })).toBeVisible({ timeout: 30_000 });

	// 'deny-cat-…' sorts before the first test's 'test-cat-…' (ORDER BY name)
	const catName = `deny-cat-${Date.now().toString(36)}`;
	await page.getByPlaceholder('new category').fill(catName);
	await page.getByRole('button', { name: 'create' }).click();
	await expect(page.locator('.cat-card')).toHaveCount(2, { timeout: 15_000 });
	const first = page.locator('.cat-card input[type="text"]').first();
	await expect(first).toHaveValue(catName);

	// deny the consent dialog -> the checkbox must revert
	page.once('dialog', (d) => d.dismiss());
	await page.locator('.cat-card').first().locator('input[type="checkbox"]').click();
	await page.waitForTimeout(400);
	const checked = await page.locator('.cat-card').first().locator('input[type="checkbox"]').isChecked();
	expect(checked).toBe(false);
});
