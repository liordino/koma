import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
	testDir: './tests',
	timeout: 120_000,
	projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
	webServer: {
		command: 'npx vite dev --port 5173 --strictPort',
		port: 5173,
		reuseExistingServer: true,
		timeout: 60_000
	}
});
