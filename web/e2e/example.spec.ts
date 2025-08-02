import { test, expect } from '@playwright/test';

test.describe('Basic Application Tests', () => {
  test('has title', async ({ page }) => {
    await page.goto('/');
    
    // Expect a title "to contain" a substring.
    await expect(page).toHaveTitle(/Rust Fullstack Starter/);
  });

  test('homepage loads successfully', async ({ page }) => {
    await page.goto('/');
    
    // Check that the page loads without errors
    await expect(page.locator('body')).toBeVisible();
  });

  test('navigation works', async ({ page }) => {
    await page.goto('/');
    
    // Test basic navigation - this will depend on your app structure
    // For now, just check if we can navigate to different routes
    const response = await page.goto('/auth/login');
    expect(response?.status()).toBeLessThan(400);
  });
});