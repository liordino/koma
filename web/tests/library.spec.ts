import { expect, test } from '@playwright/test';

// Wave 23 gate: the shell + the library-as-manga-page grid, against a server seeded
// with the real fixture import (26 titles, no covers -> placeholder panels).
const API = 'http://127.0.0.1:18100';
const TOKEN = 't';

test('shell renders: brand, tabs, active panel mark — mobile and desktop viewports', async ({
	browser
}) => {
	const init = (page: import('@playwright/test').Page) =>
		page.addInitScript(([api, token]: string[]) => {
			localStorage.setItem('koma_api', api);
			localStorage.setItem('koma_token', token);
		}, [API, TOKEN]);

	// Phone viewport: bottom tab bar with the amber panel mark lit on the active tab.
	const mobile = await (await browser.newContext({ viewport: { width: 390, height: 844 } })).newPage();
	await init(mobile);
	await mobile.goto('http://localhost:5173/');
	await expect(mobile.locator('.brand-name')).toHaveText('Koma');
	await expect(mobile.locator('.tabs-mobile a')).toHaveCount(3);
	await expect(mobile.locator('.tabs-mobile a.active .tab-dot.lit')).toBeVisible();

	// Desktop viewport: the tab bar moves to the top.
	const desktop = await (
		await browser.newContext({ viewport: { width: 1280, height: 800 } })
	).newPage();
	await init(desktop);
	await desktop.goto('http://localhost:5173/');
	await expect(desktop.locator('.tabs-desktop a')).toHaveCount(3);
	await expect(desktop.locator('.tabs-desktop a.active')).toHaveText('Library');
});

test('library grid: imported titles as cover panels with placeholders', async ({ page }) => {
	await page.addInitScript(([api, token]: string[]) => {
		localStorage.setItem('koma_api', api);
		localStorage.setItem('koma_token', token);
	}, [API, TOKEN]);

	await page.goto('http://localhost:5173/');
	await expect(page.locator('.card')).toHaveCount(26, { timeout: 30_000 });
	await expect(page.locator('.placeholder').first()).toBeVisible();
	await expect(page.locator('.chip.active')).toHaveText('All');
	await expect(page.locator('.card .title').first()).not.toBeEmpty();
});
