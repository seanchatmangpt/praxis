import { test, expect } from '@playwright/test';

test('replay: deterministic across two runs', async ({ page }) => {
  await page.goto('http://localhost:3000');

  // Run all dialects
  await page.click('button:has-text("Run all dialects")');
  await page.waitForTimeout(1000);

  // Get first hash
  const firstHash = await page.locator('[data-testid="graph-hash"]').textContent();

  // Click Replay
  await page.click('button:has-text("Replay")');
  await page.waitForTimeout(1000);

  // Get second hash
  const secondHash = await page.locator('[data-testid="graph-hash"]').textContent();

  // Assert hashes are identical
  expect(firstHash).toBe(secondHash);

  // Assert replay badge shows "stable" or success
  const replayBadge = await page.locator('[data-testid="replay-badge"]').textContent();
  expect(replayBadge).toContain('ADMITTED');  // or "stable" or whatever the UI shows
});
