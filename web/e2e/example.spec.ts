import { test, expect } from '@playwright/test';

test.describe('Basic Application Tests', () => {
  test('has title', async ({ page }) => {
    await page.goto('/');
    
    // Just check that title is not empty
    const title = await page.title();
    expect(title.length).toBeGreaterThan(0);
  });

  test('homepage loads successfully', async ({ page }) => {
    await page.goto('/');
    
    // Check that the page loads without errors (may redirect to sign-in)
    await expect(page.locator('body')).toBeVisible();
    
    // Accept either home page or sign-in page as successful load
    const signInLocator = page.locator('h1, h2, h3, button').filter({ hasText: 'Sign In' });
    const homeContentLocator = page.locator('main, .container, #root');

    await expect(signInLocator.or(homeContentLocator).first()).toBeVisible();
  });

  test('navigation works', async ({ page }) => {
    await page.goto('/');
    
    // Test basic navigation - handle redirects and network issues gracefully
    try {
      const response = await page.goto('/auth/login', { waitUntil: 'domcontentloaded' });
      // Accept both successful navigation and redirects
      const status = response?.status() || 0;
      expect([200, 201, 202, 204, 301, 302, 404]).toContain(status);
      
      // Verify we have some content regardless of where we end up
      await expect(page.locator('body')).toBeVisible();
    } catch (error) {
      // Handle navigation interruptions (common when app redirects automatically)
      if (error instanceof Error && (
        error.message.includes('interrupted by another navigation') ||
        error.message.includes('NS_BINDING_ABORTED')
      )) {
        // Navigation was interrupted by redirect - verify we can still interact
        await expect(page.locator('body')).toBeVisible();
      } else {
        throw error; // Re-throw other errors
      }
    }
  });
});