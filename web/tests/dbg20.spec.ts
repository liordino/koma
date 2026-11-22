import { test, expect } from '@playwright/test';
test('dbg categories row count', async ({ page }) => {
	await page.addInitScript(() => {
		localStorage.setItem('koma_api', 'http://127.0.0.1:18100');
		localStorage.setItem('koma_token', 't');
	});
	await page.goto('http://localhost:5173/categories');
	await page.waitForTimeout(1500);
	const rows = await page.locator('tr:has(input.manga-for)').count();
	const anyTr = await page.locator('tr').count();
	const bodyText = await page.locator('body').innerText();
	console.log(`ROWS_MANGA_FOR=${rows} TR=${anyTr} BODY=${bodyText.slice(0, 400).replace(/\n/g, '|')}`);
});
