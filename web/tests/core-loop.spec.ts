import { expect, test } from '@playwright/test';

// Wave 14 gate, host half: read 10 pages over LAN, close, resume — timed.
// Requires: koma-server ON the device at KOMA_API (192.168.15.x:18100), SPA served from
// http://localhost:4173 (the device's CORS-allowed origin).
const API = process.env.KOMA_API ?? 'http://192.168.15.26:18100';
const TOKEN = process.env.KOMA_TOKEN ?? 't';
const UI = 'http://localhost:4173';
const PAGE_H = 1200;

test('core loop: read 10 pages over LAN, close, resume', async ({ page }) => {
	await page.addInitScript(([api, token]: string[]) => {
		localStorage.setItem('koma_api', api);
		localStorage.setItem('koma_token', token);
	}, [API, TOKEN]);

	const t0 = Date.now();
	await page.goto(`${UI}/reader/1`);
	// page count is discovered progressively over LAN; wait for the counter to settle
	await expect(page.getByText(/page 1 \/ 23/)).toBeVisible({ timeout: 60_000 });
	const afterOpen = Date.now();

	// read 10 pages: interaction 1 jumps to page 10
	await page.evaluate((h) => window.scrollTo({ top: 9 * h }), PAGE_H);
	await page.waitForTimeout(800); // debounced save (200ms) + gRPC round trip over LAN

	// close
	await page.close();

	// resume on a fresh page
	const page2 = await page.context().newPage();
	await page2.addInitScript(([api, token]: string[]) => {
		localStorage.setItem('koma_api', api);
		localStorage.setItem('koma_token', token);
	}, [API, TOKEN]);
	await page2.goto(`${UI}/reader/1`);
	await expect(page2.getByText(/page 10 \/ 23/)).toBeVisible({ timeout: 60_000 });
	const afterResume = Date.now();

	const total = afterResume - t0;
	console.log(`READ_OPEN_CLOSE_RESUME_MS=${total} (open=${afterOpen - t0}, resume_tail=${afterResume - afterOpen})`);
	expect(total).toBeLessThan(90_000);
});
