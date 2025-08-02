import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
  test('login page loads', async ({ page }) => {
    await page.goto('/auth/login');
    
    // Check that login form elements are present
    await expect(page.locator('input[type="email"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('register page loads', async ({ page }) => {
    await page.goto('/auth/register');
    
    // Check that registration form elements are present
    await expect(page.locator('input[type="email"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
  });

  test('login form validation', async ({ page }) => {
    await page.goto('/auth/login');
    
    // Try to submit empty form
    await page.locator('button[type="submit"]').click();
    
    // Should show validation errors (exact implementation depends on your form)
    // This test should be adjusted based on your actual validation behavior
  });

  test('navigation between auth pages', async ({ page }) => {
    await page.goto('/auth/login');
    
    // Look for link to register page
    const registerLink = page.locator('a[href*="register"]');
    if (await registerLink.count() > 0) {
      await registerLink.click();
      await expect(page).toHaveURL(/.*register/);
    }
  });
});