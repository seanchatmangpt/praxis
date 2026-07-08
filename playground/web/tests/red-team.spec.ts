import { test, expect } from '@playwright/test';

test('red-team: unknown predicate produces REFUSED', async ({ page }) => {
  await page.goto('http://localhost:3000');

  // Apply "unknown-predicate" mutator from RedTeamPanel
  await page.click('button:has-text("Red-team")');
  await page.click('text=unknown predicate');  // or similar UI

  // Run all dialects
  await page.click('button:has-text("Run all dialects")');
  await page.waitForTimeout(1000);

  // Assert at least one dialect shows REFUSED
  const refusedCount = await page.locator('[data-testid="status-refused"]').count();
  expect(refusedCount).toBeGreaterThan(0);
});

test('red-team: 13-hook overflow produces REFUSED', async ({ page }) => {
  await page.goto('http://localhost:3000');

  // Apply "13-hook" mutator
  await page.click('button:has-text("Red-team")');
  await page.click('text=13 hooks');

  await page.click('button:has-text("Run hooks")');
  await page.waitForTimeout(1000);

  const status = await page.locator('[data-testid="hook-status"]').textContent();
  expect(status).toBe('REFUSED');
});
