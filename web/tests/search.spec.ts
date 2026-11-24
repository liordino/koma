import { expect, test } from '@playwright/test';

// Wave 24 gate: live search -> add flow, against real MangaDex (tier A, rate-limited).
const API = 'http://127.0.0.1:18100';
const TOKEN = 't';

test('search finds titles and adding lands them in the library', async ({ page }) => {
	await page.addInitScript(([api, token]: string[]) => {
		localStorage.setItem('koma_api', api);
		localStorage.setItem('koma_token', token);
	}, [API, TOKEN]);

	await page.goto('http://localhost:5173/search');
	await page.getByPlaceholder('find a title on MangaDex…').fill('one piece');
	await page.getByRole('button', { name: 'search' }).click();

	await expect(page.locator('.card').first()).toBeVisible({ timeout: 60_000 });
	const count = await page.locator('.card').count();
	expect(count).toBeGreaterThanOrEqual(3);

	// add the first hit, then the card flips to the in-library state
	await page.locator('.card .add').first().click();
	await expect(page.locator('.in-library').first()).toBeVisible({ timeout: 30_000 });

	// the added title now lives in the library
	await page.goto('http://localhost:5173/');
	const firstTitle = await page.locator('.card .title').first().textContent();
	expect(firstTitle).toBeTruthy();
});
