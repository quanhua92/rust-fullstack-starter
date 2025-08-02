import { test, expect } from '@playwright/test';

test.describe('Smoke Tests', () => {
  test('can run playwright tests', async ({ page }) => {
    // This is a basic smoke test that doesn't require any server
    // Just tests that Playwright is working correctly
    const version = await page.evaluate(() => navigator.userAgent);
    expect(version).toContain('Chrome');
  });

  test('basic browser functionality works', async ({ page }) => {
    // Test basic browser capabilities
    const title = await page.evaluate(() => document.title);
    expect(typeof title).toBe('string');
    
    const url = await page.evaluate(() => window.location.href);
    expect(url).toMatch(/^https?:\/\//);
  });
});