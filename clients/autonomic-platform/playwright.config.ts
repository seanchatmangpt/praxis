import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright harness for the OCEL v2 evidence pass (v26.7.6).
 *
 * The dev server is the real Vite app with the praxisArtifacts() middleware,
 * so /praxis-artifacts/* responses observed by the spec are real repo files
 * (receipt ledger, plan.json), not mocks.
 *
 * Run order: `node tests/run-evidence-pass.mjs` first (CLI evidence driver),
 * then `npx playwright test` (browser pass merges the driver's OCEL events).
 */
export default defineConfig({
  testDir: 'tests/playwright',
  fullyParallel: false,
  workers: 1,
  reporter: [['list']],
  webServer: {
    command: 'npx vite --port 5173',
    url: 'http://localhost:5173',
    reuseExistingServer: true,
    timeout: 60000,
  },
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
