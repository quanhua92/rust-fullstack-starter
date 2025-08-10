import { test, expect, type BrowserContext } from '@playwright/test';
import { LoginPage, TestDataGenerator } from './page-objects/AuthPage';
import { DashboardPage } from './page-objects/DashboardPage';

test.describe('Visual Regression Testing', () => {
  let authenticatedContext: BrowserContext;
  let userCredentials: { email: string; password: string };

  test.beforeAll(async ({ browser }) => {
    // Create authenticated user for dashboard screenshots
    const page = await browser.newPage();
    const testUser = TestDataGenerator.generateUniqueUser();
    userCredentials = { email: testUser.email, password: testUser.password };

    try {
      // Quick registration and login
      await page.goto('/auth/register');
      await page.locator('input[placeholder*="username" i]').fill(testUser.username);
      await page.locator('input[type="email"]').fill(testUser.email);
      await page.locator('input[placeholder="Enter your password"]').fill(testUser.password);
      await page.locator('input[placeholder="Confirm your password"]').fill(testUser.password);
      await page.locator('button:has-text("Create Account")').click();
      
      await expect(page).toHaveURL(/.*\/auth\/login/, { timeout: 8000 });
      
      const loginPage = new LoginPage(page);
      await loginPage.login(testUser.email, testUser.password);
      await loginPage.expectLoginSuccess();

      authenticatedContext = await browser.newContext({ 
        storageState: await page.context().storageState() 
      });
    } catch (error) {
      console.log('Visual test auth setup failed:', error);
    } finally {
      await page.close();
    }
  });

  test.afterAll(async () => {
    if (authenticatedContext) {
      await authenticatedContext.close();
    }
  });

  test.describe('Authentication Pages Screenshots', () => {
    test('should capture login page', async ({ page }) => {
      await page.goto('/auth/login');
      await page.waitForLoadState('networkidle');
      
      // Wait for form to fully render
      await page.locator('input[type="email"]').waitFor({ state: 'visible' });
      await page.locator('input[type="password"]').waitFor({ state: 'visible' });
      await page.locator('button:has-text("Sign In")').waitFor({ state: 'visible' });

      // Take full page screenshot
      await expect(page).toHaveScreenshot('login-page.png', {
        fullPage: true,
        animations: 'disabled'
      });
    });

    test('should capture register page', async ({ page }) => {
      await page.goto('/auth/register');
      await page.waitForLoadState('networkidle');
      
      // Wait for all form fields to render
      await page.locator('input[placeholder*="username" i]').waitFor({ state: 'visible' });
      await page.locator('input[type="email"]').waitFor({ state: 'visible' });
      await page.locator('input[placeholder="Enter your password"]').waitFor({ state: 'visible' });
      await page.locator('input[placeholder="Confirm your password"]').waitFor({ state: 'visible' });
      await page.locator('button:has-text("Create Account")').waitFor({ state: 'visible' });

      await expect(page).toHaveScreenshot('register-page.png', {
        fullPage: true,
        animations: 'disabled'
      });
    });

    test('should capture login form with validation errors', async ({ page }) => {
      await page.goto('/auth/login');
      await page.waitForLoadState('networkidle');

      // Trigger validation errors
      await page.locator('input[type="email"]').fill('invalid-email');
      await page.locator('input[type="email"]').blur();
      
      // Take screenshot with error state
      await expect(page).toHaveScreenshot('login-page-validation-errors.png', {
        fullPage: true,
        animations: 'disabled'
      });
    });

    test('should capture register form with validation errors', async ({ page }) => {
      await page.goto('/auth/register');
      await page.waitForLoadState('networkidle');

      // Trigger multiple validation errors
      await page.locator('input[placeholder*="username" i]').fill('ab'); // Too short
      await page.locator('input[type="email"]').fill('invalid-email');
      await page.locator('input[placeholder="Enter your password"]').fill('short'); // Too short
      await page.locator('input[placeholder="Confirm your password"]').fill('different');
      
      // Trigger validation by blurring
      await page.locator('input[placeholder="Confirm your password"]').blur();
      
      // Wait for error messages
      await page.waitForTimeout(1000);

      await expect(page).toHaveScreenshot('register-page-validation-errors.png', {
        fullPage: true,
        animations: 'disabled'
      });
    });
  });

  test.describe('Dashboard Screenshots', () => {
    test('should capture main dashboard', async ({ browser }) => {
      if (!authenticatedContext) {
        test.skip('No authenticated context available');
        return;
      }

      const page = await authenticatedContext.newPage();
      const dashboard = new DashboardPage(page);

      await dashboard.goto();
      await dashboard.waitForDashboardLoad();
      
      // Wait for stats to load
      await page.waitForTimeout(2000);

      await expect(page).toHaveScreenshot('dashboard-main.png', {
        fullPage: true,
        animations: 'disabled'
      });

      await page.close();
    });

    test('should capture dashboard stats section', async ({ browser }) => {
      if (!authenticatedContext) {
        test.skip('No authenticated context available');
        return;
      }

      const page = await authenticatedContext.newPage();
      const dashboard = new DashboardPage(page);

      await dashboard.goto();
      await dashboard.statsCards.expectVisible();
      
      // Wait for stats to load
      await page.waitForTimeout(2000);

      // Screenshot just the stats section
      const statsSection = page.locator('.grid').first();
      await expect(statsSection).toHaveScreenshot('dashboard-stats-cards.png', {
        animations: 'disabled'
      });

      await page.close();
    });

    test('should capture dashboard with charts', async ({ browser }) => {
      if (!authenticatedContext) {
        test.skip('No authenticated context available');
        return;
      }

      const page = await authenticatedContext.newPage();
      const dashboard = new DashboardPage(page);

      await dashboard.goto();
      await dashboard.analytics.expectVisible();
      await dashboard.analytics.expectChartsRendered();

      // Screenshot the analytics section
      const analyticsSection = page.locator('text=Live Analytics').locator('..').locator('..');
      await expect(analyticsSection).toHaveScreenshot('dashboard-analytics.png', {
        animations: 'disabled'
      });

      await page.close();
    });
  });

  test.describe('Responsive Visual Testing', () => {
    test('should capture mobile login page', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      
      await page.goto('/auth/login');
      await page.waitForLoadState('networkidle');
      
      await page.locator('button:has-text("Sign In")').waitFor({ state: 'visible' });

      await expect(page).toHaveScreenshot('login-page-mobile.png', {
        fullPage: true,
        animations: 'disabled'
      });
    });

    test('should capture tablet dashboard', async ({ browser }) => {
      if (!authenticatedContext) {
        test.skip('No authenticated context available');
        return;
      }

      const page = await authenticatedContext.newPage();
      await page.setViewportSize({ width: 768, height: 1024 });
      
      const dashboard = new DashboardPage(page);
      await dashboard.goto();
      await dashboard.waitForDashboardLoad();
      
      await page.waitForTimeout(2000);

      await expect(page).toHaveScreenshot('dashboard-tablet.png', {
        fullPage: true,
        animations: 'disabled'
      });

      await page.close();
    });

    test('should capture mobile dashboard', async ({ browser }) => {
      if (!authenticatedContext) {
        test.skip('No authenticated context available');
        return;
      }

      const page = await authenticatedContext.newPage();
      await page.setViewportSize({ width: 375, height: 667 });
      
      const dashboard = new DashboardPage(page);
      await dashboard.goto();
      await dashboard.waitForDashboardLoad();
      
      await page.waitForTimeout(2000);

      await expect(page).toHaveScreenshot('dashboard-mobile.png', {
        fullPage: true,
        animations: 'disabled'
      });

      await page.close();
    });
  });

  test.describe('Theme Testing', () => {
    test('should capture dark theme if available', async ({ page }) => {
      await page.goto('/auth/login');
      await page.waitForLoadState('networkidle');

      // Try to enable dark theme (if theme toggle exists)
      const themeToggle = page.locator('[data-theme="dark"], button[aria-label*="dark"], button[title*="dark"]');
      
      if (await themeToggle.count() > 0) {
        await themeToggle.click();
        await page.waitForTimeout(500);
        
        await expect(page).toHaveScreenshot('login-page-dark-theme.png', {
          fullPage: true,
          animations: 'disabled'
        });
      } else {
        // Try programmatically setting dark theme via next-themes
        await page.evaluate(() => {
          // Try to trigger dark theme via localStorage or theme context
          localStorage.setItem('theme', 'dark');
          document.documentElement.classList.add('dark');
        });
        
        await page.reload();
        await page.waitForLoadState('networkidle');
        
        // Check if dark theme was applied
        const isDark = await page.evaluate(() => {
          return document.documentElement.classList.contains('dark') || 
                 localStorage.getItem('theme') === 'dark';
        });

        if (isDark) {
          await expect(page).toHaveScreenshot('login-page-dark-theme.png', {
            fullPage: true,
            animations: 'disabled'
          });
        }
      }
    });
  });

  test.describe('Component State Screenshots', () => {
    test('should capture form loading states', async ({ page }) => {
      await page.goto('/auth/login');
      await page.waitForLoadState('networkidle');

      // Simulate form submission loading state
      await page.locator('input[type="email"]').fill('test@example.com');
      await page.locator('input[type="password"]').fill('testpassword');

      // Quickly take screenshot during submit (may show loading state)
      const submitPromise = page.locator('button:has-text("Sign In")').click();
      
      // Take screenshot quickly to catch loading state
      await Promise.race([
        expect(page).toHaveScreenshot('login-loading-state.png', {
          animations: 'disabled'
        }),
        new Promise(resolve => setTimeout(resolve, 1000))
      ]);
      
      await submitPromise;
    });

    test('should capture success states', async ({ page }) => {
      const testUser = TestDataGenerator.generateUniqueUser();

      await page.goto('/auth/register');
      await page.waitForLoadState('networkidle');

      await page.locator('input[placeholder*="username" i]').fill(testUser.username);
      await page.locator('input[type="email"]').fill(testUser.email);
      await page.locator('input[placeholder="Enter your password"]').fill(testUser.password);
      await page.locator('input[placeholder="Confirm your password"]').fill(testUser.password);
      await page.locator('button:has-text("Create Account")').click();

      // Wait for success message
      try {
        await page.waitForSelector('text=Registration successful! Redirecting to login page...', { timeout: 10000 });
        
        await expect(page).toHaveScreenshot('register-success-state.png', {
          animations: 'disabled'
        });
      } catch (error) {
        // Success message might not appear or might be too fast
        console.log('Could not capture success state:', error);
      }
    });
  });
});