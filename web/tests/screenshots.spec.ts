import { test } from '@playwright/test';

// README screenshots — run explicitly (npm-run tooling for docs, not a gate):
//   npx playwright test screenshots.spec.ts
// Seed recipe: KOMA_ADDR=127.0.0.1:18080 server against a fresh DB,
//   koma-import <real-backup.tachibk>, then wire one chapter's storage_dir
//   (UPDATE chapter SET storage_dir='pages', downloaded=1) to real page files.
const TOKEN = 'shots';
const UI = 'http://localhost:5173';

async function setup(page: import('@playwright/test').Page) {
	await page.addInitScript(([api, token]: string[]) => {
		localStorage.setItem('koma_api', api);
		localStorage.setItem('koma_token', token);
	}, ['http://127.0.0.1:18080', TOKEN]);
}

test('library (desktop)', async ({ page }) => {
	await setup(page);
	await page.setViewportSize({ width: 1280, height: 800 });
	await page.goto(`${UI}/`);
	await page.waitForTimeout(1500);
	await page.screenshot({ path: '../.github/assets/screenshot-library.png' });
});

test('reader (desktop)', async ({ page }) => {
	await setup(page);
	await page.setViewportSize({ width: 1280, height: 800 });
	await page.goto(`${UI}/reader/1`);
	await page.waitForTimeout(2500);
	// sit a few pages into the chapter so the windowed reader shows loaded pages
	await page.evaluate(() => window.scrollTo({ top: 2 * 1200 }));
	await page.waitForTimeout(1500);
	await page.screenshot({ path: '../.github/assets/screenshot-reader.png' });
});

test('reader (mobile)', async ({ browser }) => {
	const ctx = await browser.newContext({ viewport: { width: 390, height: 844 } });
	const page = await ctx.newPage();
	await setup(page);
	await page.goto(`${UI}/reader/1`);
	await page.waitForTimeout(2500);
	await page.evaluate(() => window.scrollTo({ top: 1200 }));
	await page.waitForTimeout(1200);
	await page.screenshot({ path: '../.github/assets/screenshot-reader-mobile.png' });
	await ctx.close();
});

test('categories (desktop)', async ({ page }) => {
	await setup(page);
	await page.setViewportSize({ width: 1280, height: 800 });
	await page.goto(`${UI}/categories`);
	await page.waitForTimeout(1500);
	await page.screenshot({ path: '../.github/assets/screenshot-categories.png' });
});
