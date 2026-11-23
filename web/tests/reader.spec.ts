import { expect, test } from '@playwright/test';

// Wave 13 gate: windowed vertical reader with page-anchored, resumable progress.
// Requires a Koma server seeded with chapter 1 (bench-seed: 10 pages) and token "t".
const API = process.env.KOMA_API ?? 'http://127.0.0.1:18100';
const TOKEN = process.env.KOMA_TOKEN ?? 't';
const PAGE_H = 1200;

test('read to a deep page, reload, resume in <= 2 interactions', async ({ page }) => {
	await page.addInitScript(([api, token]: string[]) => {
		localStorage.setItem('koma_api', api);
		localStorage.setItem('koma_token', token);
	}, [API, TOKEN]);

	await page.goto('http://localhost:5173/reader/1');
	await expect(page.getByText(/page 1 \/ 10/)).toBeVisible({ timeout: 30_000 });

	// Read forward: jump to a deep page (interaction 1), then scroll a bit (interaction 2).
	await page.evaluate((h) => window.scrollTo({ top: 8 * h }), PAGE_H);
	await page.waitForTimeout(600); // debounced save (200ms) + RPC round trip
	await page.evaluate(() => window.scrollBy({ top: 600 }));

	await page.reload();
	await page.waitForLoadState('networkidle');
	await expect(page.getByText(/page 10 \/ 10|page 9 \/ 10/)).toBeVisible({ timeout: 30_000 });

	// The scroll position must be restored to the deep anchor — zero further interactions.
	const y = await page.evaluate(() => window.scrollY);
	expect(Math.abs(y - 9 * PAGE_H)).toBeLessThan(2 * PAGE_H);
});
