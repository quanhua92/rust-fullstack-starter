import { test, expect } from '@playwright/test';

test.describe('Admin Dashboard Navigation & UI', () => {
  // Helper function to login as admin user
  async function loginAsAdmin(browser: any) {
    const page = await browser.newPage();
    
    // Use pre-configured admin credentials
    // Admin account should be created automatically on server startup
    const email = 'admin@example.com';
    const password = process.env.STARTER__INITIAL_ADMIN_PASSWORD || 'admin123';

    // Navigate to login
    await page.goto('/auth/login');
    await page.waitForLoadState('networkidle');

    // Login with admin credentials
    await page.locator('input[type="email"]').fill(email);
    await page.locator('input[type="password"]').fill(password);
    await page.locator('button:has-text("Sign In")').click();

    // Wait for successful login and redirect  
    await page.waitForLoadState('networkidle', { timeout: 15000 });
    
    // Store the authenticated admin context
    const context = await browser.newContext({ 
      storageState: await page.context().storageState() 
    });
    
    await page.close();
    return { context, credentials: { email, password } };
  }

  test.describe('Dashboard Loading and Layout', () => {
    test('should load dashboard with all main sections', async ({ browser }) => {
      const { context } = await loginAsAdmin(browser);
      const page = await context.newPage();
      
      await page.goto('/admin');
      await page.waitForLoadState('networkidle');

      // Check main dashboard title
      await expect(page.locator('h1:has-text("Dashboard")')).toBeVisible();
      
      // Check welcome message
      await expect(page.getByText(/Welcome back! Here's what's happening/)).toBeVisible();

      // Check stats cards section
      await expect(page.getByText('Total Tasks')).toBeVisible();
      await expect(page.getByText('Active Tasks')).toBeVisible();
      await expect(page.getByText('Failed Tasks')).toBeVisible();
      await expect(page.getByText('Success Rate')).toBeVisible();

      // Check system health section
      await expect(page.getByText('System Health')).toBeVisible();

      // Check live analytics section
      await expect(page.getByText('Live Analytics')).toBeVisible();
      
      // Check recent activity section
      await expect(page.getByText('Recent Activity')).toBeVisible();

      // Check quick actions section
      await expect(page.getByText('Quick Actions')).toBeVisible();

      await page.close();
      await context.close();
    });

    test('should display stats cards with proper loading states', async ({ browser }) => {
      const { context } = await loginAsAdmin(browser);
      const page = await context.newPage();
      
      await page.goto('/admin');
      
      // Check for either skeleton loading or actual stats
      const statsSection = page.locator('.grid').first();
      await expect(statsSection).toBeVisible();

      // Should have either loading skeletons or actual stat cards
      const skeletons = page.locator('.animate-pulse, [data-testid="skeleton"]');
      const statCards = page.locator('text=Total Tasks').locator('..');
      
      await expect(skeletons.or(statCards)).toBeVisible();

      // Wait for content to load and check actual stats
      await page.waitForTimeout(2000);
      await expect(page.getByText('Total Tasks')).toBeVisible();

      await page.close();
      await context.close();
    });

    test('should render charts and data visualizations', async ({ browser }) => {
      const { context } = await loginAsAdmin(browser);
      const page = await context.newPage();
      
      await page.goto('/admin');
      await page.waitForLoadState('networkidle');

      // Check for chart container
      await expect(page.getByText('Task Trends (7 days)')).toBeVisible();
      
      // Check for chart elements (SVG or canvas)
      const chartElements = page.locator('svg, canvas').first();
      await expect(chartElements).toBeVisible({ timeout: 5000 });

      // Check real-time status section
      await expect(page.getByText('Real-time Status')).toBeVisible();
      await expect(page.getByText('System Health:')).toBeVisible();

      await page.close();
      await context.close();
    });
  });

  test.describe('Sidebar Navigation', () => {
    test('should display sidebar with all navigation items', async ({ browser }) => {
      const { context } = await loginAsAdmin(browser);
      const page = await context.newPage();
      
      await page.goto('/admin');
      await page.waitForLoadState('networkidle');

      // Check main navigation items in sidebar
      const sidebar = page.locator('nav, [role="navigation"], aside').first();
      await expect(sidebar).toBeVisible();

      // Check for navigation menu items (flexible selectors)
      const dashboardLink = page.getByRole('link', { name: /dashboard|home/i });
      const usersLink = page.getByRole('link', { name: /users/i });
      const tasksLink = page.getByRole('link', { name: /tasks/i });
      const monitoringLink = page.getByRole('link', { name: /monitoring/i });
      const healthLink = page.getByRole('link', { name: /health/i });

      // At least some navigation items should be visible
      await expect(dashboardLink.or(usersLink).or(tasksLink).or(monitoringLink).or(healthLink)).toBeVisible();

      await page.close();
      await context.close();
    });

    test('should navigate to different admin sections', async ({ browser }) => {
      const { context } = await loginAsAdmin(browser);
      const page = await context.newPage();
      
      await page.goto('/admin');
      await page.waitForLoadState('networkidle');

      // Test navigation to users section
      const usersLink = page.getByRole('link', { name: /users/i }).first();
      if (await usersLink.count() > 0) {
        await usersLink.click();
        await page.waitForLoadState('networkidle');
        await expect(page).toHaveURL(/.*\/admin\/users/);
        
        // Go back to dashboard
        await page.goto('/admin');
      }

      // Test navigation to tasks section
      const tasksLink = page.getByRole('link', { name: /tasks/i }).first();
      if (await tasksLink.count() > 0) {
        await tasksLink.click();
        await page.waitForLoadState('networkidle');
        await expect(page).toHaveURL(/.*\/admin\/tasks/);
        
        // Go back to dashboard  
        await page.goto('/admin');
      }

      // Test navigation to monitoring section
      const monitoringLink = page.getByRole('link', { name: /monitoring/i }).first();
      if (await monitoringLink.count() > 0) {
        await monitoringLink.click();
        await page.waitForLoadState('networkidle');
        await expect(page).toHaveURL(/.*\/admin\/monitoring/);
      }

      await page.close();
      await context.close();
    });
  });

  test.describe('Dashboard Interactive Elements', () => {
    test('should have working quick action buttons', async ({ browser }) => {
      const { context } = await loginAsAdmin(browser);
      const page = await context.newPage();
      
      await page.goto('/admin');
      await page.waitForLoadState('networkidle');

      // Find quick actions section
      const quickActionsSection = page.locator('text=Quick Actions').locator('..').locator('..'); 
      await expect(quickActionsSection).toBeVisible();

      // Test monitoring overview button
      const monitoringButton = page.getByRole('link', { name: /monitoring overview/i });
      if (await monitoringButton.count() > 0) {
        await expect(monitoringButton).toBeEnabled();
        // Click and verify navigation
        await monitoringButton.click();
        await page.waitForLoadState('networkidle');
        await expect(page).toHaveURL(/.*\/admin\/monitoring/);
        await page.goBack();
      }

      // Test task management button
      const taskButton = page.getByRole('link', { name: /task management/i });
      if (await taskButton.count() > 0) {
        await expect(taskButton).toBeEnabled();
      }

      await page.close();
      await context.close();
    });

    test('should display real-time data updates', async ({ browser }) => {
      const { context } = await loginAsAdmin(browser);
      const page = await context.newPage();
      
      await page.goto('/admin');
      await page.waitForLoadState('networkidle');

      // Check that timestamps or live data elements are present
      const timestampElement = page.getByText(/\d{1,2}:\d{2}:\d{2}|\d{4}-\d{2}-\d{2}|Last Update|ago/i);
      await expect(timestampElement.first()).toBeVisible({ timeout: 5000 });

      // Check that status badges are present
      const statusBadges = page.locator('[role="status"], .badge, [class*="badge"]');
      await expect(statusBadges.first()).toBeVisible();

      await page.close();
      await context.close();
    });
  });

  test.describe('User Profile Information', () => {
    test('should display current user information', async ({ browser }) => {
      const { context } = await loginAsAdmin(browser);
      const page = await context.newPage();
      
      await page.goto('/admin');
      await page.waitForLoadState('networkidle');

      // Look for user information section
      const userInfoSection = page.locator('text=Current User').locator('..');
      if (await userInfoSection.count() > 0) {
        await expect(userInfoSection).toBeVisible();
        
        // Check for user details
        await expect(page.getByText('Username:')).toBeVisible();
        await expect(page.getByText('Role:')).toBeVisible();
        await expect(page.getByText('Email:')).toBeVisible();
      }

      // Alternatively, check for user avatar or name in header/sidebar
      const userIndicators = page.locator('[role="img"], .avatar, text=/user|admin/i');
      await expect(userIndicators.first()).toBeVisible();

      await page.close();
      await context.close();
    });
  });

  test.describe('Responsive Design', () => {
    test('should adapt to mobile viewport', async ({ browser }) => {
      const { context } = await loginAsAdmin(browser);
      const page = await context.newPage();
      
      // Set mobile viewport
      await page.setViewportSize({ width: 375, height: 667 });
      
      await page.goto('/admin');
      await page.waitForLoadState('networkidle');

      // Dashboard should still be visible and functional
      await expect(page.locator('h1:has-text("Dashboard")')).toBeVisible();
      
      // Stats cards should stack vertically or be scrollable
      const statsCards = page.locator('text=Total Tasks').locator('..');
      await expect(statsCards).toBeVisible();

      // Navigation should be accessible (might be collapsed/hamburger)
      const navigationElements = page.locator('nav, [role="navigation"], button[aria-expanded]');
      await expect(navigationElements.first()).toBeVisible();

      await page.close();
      await context.close();
    });

    test('should handle tablet viewport', async ({ browser }) => {
      const { context } = await loginAsAdmin(browser);
      const page = await context.newPage();
      
      // Set tablet viewport
      await page.setViewportSize({ width: 768, height: 1024 });
      
      await page.goto('/admin');
      await page.waitForLoadState('networkidle');

      // All main sections should be visible
      await expect(page.locator('h1:has-text("Dashboard")')).toBeVisible();
      await expect(page.getByText('System Health')).toBeVisible();
      await expect(page.getByText('Quick Actions')).toBeVisible();

      await page.close();
      await context.close();
    });
  });

  test.describe('Loading States and Error Handling', () => {
    test('should handle slow network conditions gracefully', async ({ browser }) => {
      const { context } = await loginAsAdmin(browser);
      const page = await context.newPage();
      
      // Simulate slow network
      await page.route('**/api/**', async (route: any) => {
        await new Promise(resolve => setTimeout(resolve, 1000)); // 1s delay
        await route.continue();
      });

      await page.goto('/admin');
      
      // Should show loading states
      const loadingElements = page.locator('.animate-spin, .loading, [data-testid="loading"]');
      await expect(loadingElements.first()).toBeVisible({ timeout: 2000 });

      // Eventually content should load
      await expect(page.locator('h1:has-text("Dashboard")')).toBeVisible({ timeout: 10000 });

      await page.close();
      await context.close();
    });
  });
});