import { test, expect } from '@playwright/test';

test('smoke: load fixture and run all dialects', async ({ page }) => {
  await page.goto('http://localhost:3000');  // adjust baseURL if needed
  // (Or use baseURL from playwright.config)

  // Load the counterparty case fixture
  await page.click('button:has-text("Load case")');  // or similar
  // (exact selector depends on UI; use page.locator() or role-based if needed)

  // Run all dialects
  await page.click('button:has-text("Run all dialects")');

  // Wait for results to populate
  await page.waitForTimeout(2000);  // or use waitForSelector

  // Assert capability matrix shows ADMITTED (or real status, not loading/placeholder)
  const matrixStatus = await page.locator('[data-testid="status-admitted"]').count();
  expect(matrixStatus).toBeGreaterThan(0);  // at least one ADMITTED chip

  // Assert ReceiptPanel shows a non-empty graph_hash
  const hashText = await page.locator('[data-testid="graph-hash"]').textContent();
  expect(hashText).toBeTruthy();
  expect(hashText?.length).toBeGreaterThan(10);  // realistic hash length
});
