import { test, expect, type BrowserContext } from '@playwright/test';
import { LoginPage, TestDataGenerator } from './page-objects/AuthPage';
import { DashboardPage, ResponsiveDashboard } from './page-objects/DashboardPage';

test.describe('Dashboard with Page Objects', () => {
  let authenticatedContext: BrowserContext;
  let userCredentials: { email: string; password: string };

  test.beforeAll(async ({ browser }) => {
    // Create authenticated regular user context for testing user view of /admin
    const page = await browser.newPage();
    const loginPage = new LoginPage(page);
    const testUser = TestDataGenerator.generateUniqueUser();
    
    userCredentials = { email: testUser.email, password: testUser.password };

    try {
      // Register and login as regular user
      await page.goto('/auth/register');
      await page.locator('input[placeholder*="username" i]').fill(testUser.username);
      await page.locator('input[type="email"]').fill(testUser.email);
      await page.locator('input[placeholder="Enter your password"]').fill(testUser.password);
      await page.locator('input[placeholder="Confirm your password"]').fill(testUser.password);
      await page.locator('button:has-text("Create Account")').click();
      
      await expect(page).toHaveURL(/.*\/auth\/login/, { timeout: 8000 });
      
      await loginPage.login(testUser.email, testUser.password);
      await loginPage.expectLoginSuccess();
      
      // Verify we're redirected to admin area (regular users get redirected here)
      await expect(page).toHaveURL(/.*\/admin/, { timeout: 8000 });

      // Store authenticated state
      authenticatedContext = await browser.newContext({ 
        storageState: await page.context().storageState() 
      });
    } catch (error) {
      console.log('Auth setup failed, tests will authenticate individually:', error);
    } finally {
      await page.close();
    }
  });

  test.afterAll(async () => {
    if (authenticatedContext) {
      await authenticatedContext.close();
    }
  });

  async function getAuthenticatedPage(browser: any) {
    if (authenticatedContext) {
      return await authenticatedContext.newPage();
    } else {
      // Fallback authentication
      const page = await browser.newPage();
      const loginPage = new LoginPage(page);
      await loginPage.goto('/auth/login');
      await loginPage.login(userCredentials.email, userCredentials.password);
      await loginPage.expectLoginSuccess();
      return page;
    }
  }

  test.describe('Dashboard Layout and Loading', () => {
    test('should load dashboard with all main components', async ({ browser }) => {
      const page = await getAuthenticatedPage(browser);
      const dashboard = new DashboardPage(page);

      await dashboard.goto();
      await dashboard.expectFullDashboardLoaded();

      await page.close();
    });

    test('should display stats cards correctly', async ({ browser }) => {
      const page = await getAuthenticatedPage(browser);
      const dashboard = new DashboardPage(page);

      await dashboard.goto();
      await dashboard.statsCards.expectVisible();
      
      // Wait for actual data to load
      await page.waitForTimeout(2000);
      await dashboard.statsCards.expectAllStatsVisible();

      await page.close();
    });

    test('should show system health information', async ({ browser }) => {
      const page = await getAuthenticatedPage(browser);
      const dashboard = new DashboardPage(page);

      await dashboard.goto();
      await dashboard.systemHealth.expectVisible();
      await dashboard.systemHealth.expectHealthStatusVisible();

      await page.close();
    });

    test('should render analytics charts', async ({ browser }) => {
      const page = await getAuthenticatedPage(browser);
      const dashboard = new DashboardPage(page);

      await dashboard.goto();
      await dashboard.analytics.expectVisible();
      await dashboard.analytics.expectChartsRendered();
      await dashboard.analytics.expectRealTimeData();

      await page.close();
    });
  });

  test.describe('Navigation and Interactions', () => {
    test('should have functional sidebar navigation', async ({ browser }) => {
      const page = await getAuthenticatedPage(browser);
      const dashboard = new DashboardPage(page);

      await dashboard.goto();
      await dashboard.sidebar.expectVisible();
      await dashboard.sidebar.expectNavigationItemsVisible();

      // Test navigation to different sections
      await dashboard.sidebar.navigateToUsers();
      await page.waitForTimeout(1000);
      
      // Navigate back to dashboard
      await dashboard.goto();

      await page.close();
    });

    test('should have working quick action buttons', async ({ browser }) => {
      const page = await getAuthenticatedPage(browser);
      const dashboard = new DashboardPage(page);

      await dashboard.goto();
      await dashboard.quickActions.expectVisible();
      await dashboard.quickActions.expectButtonsEnabled();

      // Test monitoring button
      if (await dashboard.quickActions.monitoringButton.count() > 0) {
        await dashboard.quickActions.clickMonitoring();
        await page.waitForTimeout(1000);
        
        // Navigate back
        await dashboard.goto();
      }

      await page.close();
    });

    test('should handle analytics interactions', async ({ browser }) => {
      const page = await getAuthenticatedPage(browser);
      const dashboard = new DashboardPage(page);

      await dashboard.goto();
      await dashboard.analytics.expectVisible();

      // Test view full analytics button
      if (await dashboard.analytics.viewFullAnalyticsButton.count() > 0) {
        await dashboard.analytics.clickViewFullAnalytics();
        await page.waitForTimeout(1000);
      }

      await page.close();
    });
  });

  test.describe('Responsive Design', () => {
    test('should work on mobile devices', async ({ browser }) => {
      const page = await getAuthenticatedPage(browser);
      const responsiveDashboard = new ResponsiveDashboard(page);

      await responsiveDashboard.testMobileLayout();
      await page.close();
    });

    test('should work on tablet devices', async ({ browser }) => {
      const page = await getAuthenticatedPage(browser);
      const responsiveDashboard = new ResponsiveDashboard(page);

      await responsiveDashboard.testTabletLayout();
      await page.close();
    });

    test('should work on desktop', async ({ browser }) => {
      const page = await getAuthenticatedPage(browser);
      const responsiveDashboard = new ResponsiveDashboard(page);

      await responsiveDashboard.testDesktopLayout();
      await page.close();
    });
  });

  test.describe('Loading States and Error Handling', () => {
    test('should handle slow loading gracefully', async ({ browser }) => {
      const page = await getAuthenticatedPage(browser);
      const dashboard = new DashboardPage(page);

      // Simulate slow network
      await page.route('**/api/**', async (route: any) => {
        await new Promise(resolve => setTimeout(resolve, 1000));
        await route.continue();
      });

      await dashboard.goto();
      
      // Should show loading states initially
      await dashboard.statsCards.expectLoadingState();
      
      // Eventually content should load
      await dashboard.waitForDashboardLoad();

      await page.close();
    });

    test('should display user information', async ({ browser }) => {
      const page = await getAuthenticatedPage(browser);
      const dashboard = new DashboardPage(page);

      await dashboard.goto();
      await dashboard.waitForDashboardLoad();

      // Check for user information section
      const userInfoSection = page.locator('text=Current User').locator('..');
      if (await userInfoSection.count() > 0) {
        await expect(userInfoSection).toBeVisible();
        await expect(page.getByText('Username:')).toBeVisible();
        await expect(page.getByText('Role:')).toBeVisible();
        await expect(page.getByText('Email:')).toBeVisible();
      }

      // Alternative: check for user indicators in header/sidebar
      const userIndicators = page.locator('[role="img"], .avatar, text=/user|admin/i');
      await expect(userIndicators.first()).toBeVisible();

      await page.close();
    });
  });
});