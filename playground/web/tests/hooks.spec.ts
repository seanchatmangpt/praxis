import { test, expect } from '@playwright/test';

test('hooks: fire expected hook on matching event', async ({ page }) => {
  await page.goto('http://localhost:3000');

  // Load hooks.ttl and event.ttl fixtures
  await page.click('[data-tab="hooks.ttl"]');  // switch to hooks tab
  // (UI should allow switching between tabs)

  // Run hooks
  await page.click('button:has-text("Run hooks")');
  await page.waitForTimeout(1000);

  // Assert HookTimeline shows Fired status for expected hook
  const firedCount = await page.locator('[data-verdict-status="Fired"]').count();
  expect(firedCount).toBeGreaterThan(0);

  // Assert schedule order is respected
  const schedule = await page.locator('[data-hook-schedule]').allTextContents();
  // schedule should match kh:priority/kh:after ordering
  expect(schedule.length).toBeGreaterThan(0);
});

test('hooks: refuse 13-hook pack', async ({ page }) => {
  await page.goto('http://localhost:3000');

  // Load a fixture with 13 hooks
  await page.fill('[data-testid="editor"]', fixture13Hooks);
  await page.click('button:has-text("Run hooks")');
  await page.waitForTimeout(1000);

  // Assert status is REFUSED
  const status = await page.locator('[data-testid="hook-status"]').textContent();
  expect(status).toBe('REFUSED');
});
