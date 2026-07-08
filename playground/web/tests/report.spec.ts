import { test, expect } from '@playwright/test';

test('report: generate and verify hash stability', async ({ page }) => {
  await page.goto('http://localhost:3000');

  // Run all dialects to populate results
  await page.click('button:has-text("Run all dialects")');
  await page.waitForTimeout(1000);

  // Click Generate Report
  await page.click('button:has-text("Generate report")');

  // Wait for report dialog/panel to appear
  await page.waitForSelector('[data-testid="report-panel"]');

  // Extract report hash
  const reportHash1 = await page.locator('[data-testid="report-hash"]').textContent();

  // Generate report again (same data)
  await page.click('button:has-text("Generate report")');
  await page.waitForTimeout(500);

  const reportHash2 = await page.locator('[data-testid="report-hash"]').textContent();

  // Hashes should be identical
  expect(reportHash1).toBe(reportHash2);

  // Assert report contains capability matrix
  const reportText = await page.locator('[data-testid="report-content"]').textContent();
  expect(reportText).toContain('Turtle');  // or other dialect name
});
