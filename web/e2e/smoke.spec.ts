import { test, expect } from '@playwright/test';

test.describe('Smoke Tests', () => {
  test('playwright basic functionality works', async ({ page }) => {
    // This is a basic smoke test that doesn't require any server
    // Just tests that Playwright is working correctly
    const version = await page.evaluate(() => navigator.userAgent);
    expect(typeof version).toBe('string');
    expect(version.length).toBeGreaterThan(0);
    
    const title = await page.evaluate(() => document.title);
    expect(typeof title).toBe('string');
  });
});