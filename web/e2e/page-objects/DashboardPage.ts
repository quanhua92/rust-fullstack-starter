import { type Locator, type Page } from '@playwright/test';

export class DashboardPage {
  readonly page: Page;
  readonly title: Locator;
  readonly welcomeMessage: Locator;
  readonly statsCards: DashboardStatsCards;
  readonly systemHealth: DashboardSystemHealth;
  readonly quickActions: DashboardQuickActions;
  readonly sidebar: DashboardSidebar;
  readonly analytics: DashboardAnalytics;

  constructor(page: Page) {
    this.page = page;
    this.title = page.locator('h1:has-text("Dashboard")');
    this.welcomeMessage = page.getByText(/Welcome back! Here's what's happening/);
    this.statsCards = new DashboardStatsCards(page);
    this.systemHealth = new DashboardSystemHealth(page);
    this.quickActions = new DashboardQuickActions(page);
    this.sidebar = new DashboardSidebar(page);
    this.analytics = new DashboardAnalytics(page);
  }

  async goto() {
    await this.page.goto('/admin');
    await this.page.waitForLoadState('networkidle');
  }

  async waitForDashboardLoad() {
    await this.title.waitFor({ state: 'visible' });
    await this.welcomeMessage.waitFor({ state: 'visible' });
  }

  async expectFullDashboardLoaded() {
    await this.waitForDashboardLoad();
    await this.statsCards.expectVisible();
    await this.systemHealth.expectVisible();
    await this.quickActions.expectVisible();
  }
}

export class DashboardStatsCards {
  readonly page: Page;
  readonly totalTasks: Locator;
  readonly activeTasks: Locator;
  readonly failedTasks: Locator;
  readonly successRate: Locator;
  readonly loadingSkeletons: Locator;

  constructor(page: Page) {
    this.page = page;
    this.totalTasks = page.getByText('Total Tasks');
    this.activeTasks = page.getByText('Active Tasks');
    this.failedTasks = page.getByText('Failed Tasks');
    this.successRate = page.getByText('Success Rate');
    this.loadingSkeletons = page.locator('.animate-pulse, [data-testid="skeleton"]');
  }

  async expectVisible() {
    // Either loading skeletons OR actual stats should be visible
    const skeletonsOrStats = this.loadingSkeletons.or(this.totalTasks);
    await skeletonsOrStats.waitFor({ state: 'visible' });
    
    // Eventually stats should load
    await this.totalTasks.waitFor({ state: 'visible', timeout: 10000 });
  }

  async expectAllStatsVisible() {
    await this.totalTasks.waitFor({ state: 'visible' });
    await this.activeTasks.waitFor({ state: 'visible' });
    await this.failedTasks.waitFor({ state: 'visible' });
    await this.successRate.waitFor({ state: 'visible' });
  }

  async expectLoadingState() {
    await this.loadingSkeletons.first().waitFor({ state: 'visible' });
  }
}

export class DashboardSystemHealth {
  readonly page: Page;
  readonly healthSection: Locator;
  readonly healthStatus: Locator;
  readonly healthBadges: Locator;

  constructor(page: Page) {
    this.page = page;
    this.healthSection = page.getByText('System Health');
    this.healthStatus = page.getByText(/healthy|unhealthy|unknown/i);
    this.healthBadges = page.locator('.badge, [role="status"], [class*="badge"]');
  }

  async expectVisible() {
    await this.healthSection.waitFor({ state: 'visible' });
  }

  async expectHealthStatusVisible() {
    await this.healthBadges.first().waitFor({ state: 'visible' });
  }
}

export class DashboardQuickActions {
  readonly page: Page;
  readonly quickActionsSection: Locator;
  readonly monitoringButton: Locator;
  readonly tasksButton: Locator;
  readonly usersButton: Locator;
  readonly eventsButton: Locator;

  constructor(page: Page) {
    this.page = page;
    this.quickActionsSection = page.getByText('Quick Actions');
    this.monitoringButton = page.getByRole('link', { name: /monitoring overview/i });
    this.tasksButton = page.getByRole('link', { name: /task management/i });
    this.usersButton = page.getByRole('link', { name: /user management/i });
    this.eventsButton = page.getByRole('link', { name: /live events/i });
  }

  async expectVisible() {
    await this.quickActionsSection.waitFor({ state: 'visible' });
  }

  async clickMonitoring() {
    await this.monitoringButton.click();
    await this.page.waitForURL(/.*\/admin\/monitoring/);
  }

  async clickTasks() {
    await this.tasksButton.click();
    await this.page.waitForURL(/.*\/admin\/tasks/);
  }

  async clickUsers() {
    await this.usersButton.click();
    await this.page.waitForURL(/.*\/admin\/users/);
  }

  async expectButtonsEnabled() {
    const buttons = [this.monitoringButton, this.tasksButton, this.usersButton, this.eventsButton];
    
    for (const button of buttons) {
      if (await button.count() > 0) {
        await button.waitFor({ state: 'visible' });
        // Button should be enabled (not disabled)
        await this.page.waitForFunction((btn) => {
          const element = document.evaluate(
            btn,
            document,
            null,
            XPathResult.FIRST_ORDERED_NODE_TYPE,
            null
          ).singleNodeValue as HTMLElement;
          return element && !element.hasAttribute('disabled');
        }, button);
      }
    }
  }
}

export class DashboardSidebar {
  readonly page: Page;
  readonly sidebar: Locator;
  readonly dashboardLink: Locator;
  readonly usersLink: Locator;
  readonly tasksLink: Locator;
  readonly monitoringLink: Locator;
  readonly healthLink: Locator;
  readonly analyticsLink: Locator;

  constructor(page: Page) {
    this.page = page;
    this.sidebar = page.locator('nav, [role="navigation"], aside').first();
    this.dashboardLink = page.getByRole('link', { name: /dashboard|home/i });
    this.usersLink = page.getByRole('link', { name: /users/i });
    this.tasksLink = page.getByRole('link', { name: /tasks/i });
    this.monitoringLink = page.getByRole('link', { name: /monitoring/i });
    this.healthLink = page.getByRole('link', { name: /health/i });
    this.analyticsLink = page.getByRole('link', { name: /analytics/i });
  }

  async expectVisible() {
    await this.sidebar.waitFor({ state: 'visible' });
  }

  async expectNavigationItemsVisible() {
    // At least some navigation items should be present
    const anyNavItem = this.dashboardLink.or(this.usersLink).or(this.tasksLink).or(this.monitoringLink);
    await anyNavItem.waitFor({ state: 'visible' });
  }

  async navigateToUsers() {
    if (await this.usersLink.count() > 0) {
      await this.usersLink.first().click();
      await this.page.waitForURL(/.*\/admin\/users/);
    }
  }

  async navigateToTasks() {
    if (await this.tasksLink.count() > 0) {
      await this.tasksLink.first().click();
      await this.page.waitForURL(/.*\/admin\/tasks/);
    }
  }

  async navigateToMonitoring() {
    if (await this.monitoringLink.count() > 0) {
      await this.monitoringLink.first().click();
      await this.page.waitForURL(/.*\/admin\/monitoring/);
    }
  }

  async navigateToHealth() {
    if (await this.healthLink.count() > 0) {
      await this.healthLink.first().click();
      await this.page.waitForURL(/.*\/admin\/health/);
    }
  }
}

export class DashboardAnalytics {
  readonly page: Page;
  readonly analyticsSection: Locator;
  readonly chartElements: Locator;
  readonly taskTrendsChart: Locator;
  readonly realTimeStatus: Locator;
  readonly viewFullAnalyticsButton: Locator;

  constructor(page: Page) {
    this.page = page;
    this.analyticsSection = page.getByText('Live Analytics');
    this.chartElements = page.locator('svg, canvas');
    this.taskTrendsChart = page.getByText('Task Trends (7 days)');
    this.realTimeStatus = page.getByText('Real-time Status');
    this.viewFullAnalyticsButton = page.getByRole('link', { name: /view full analytics/i });
  }

  async expectVisible() {
    await this.analyticsSection.waitFor({ state: 'visible' });
  }

  async expectChartsRendered() {
    await this.taskTrendsChart.waitFor({ state: 'visible' });
    await this.chartElements.first().waitFor({ state: 'visible', timeout: 5000 });
  }

  async expectRealTimeData() {
    await this.realTimeStatus.waitFor({ state: 'visible' });
    
    // Check for live data indicators (timestamps, status badges)
    const liveDataIndicators = this.page.locator('[role="status"], .badge, [class*="badge"], text=/\\d{1,2}:\\d{2}:/');
    await liveDataIndicators.first().waitFor({ state: 'visible' });
  }

  async clickViewFullAnalytics() {
    if (await this.viewFullAnalyticsButton.count() > 0) {
      await this.viewFullAnalyticsButton.click();
      await this.page.waitForURL(/.*\/admin\/analytics/);
    }
  }
}

// Helper class for responsive testing
export class ResponsiveDashboard {
  readonly page: Page;
  readonly dashboard: DashboardPage;

  constructor(page: Page) {
    this.page = page;
    this.dashboard = new DashboardPage(page);
  }

  async testMobileLayout() {
    await this.page.setViewportSize({ width: 375, height: 667 });
    await this.dashboard.goto();
    await this.dashboard.expectFullDashboardLoaded();
    
    // On mobile, navigation might be collapsed
    const mobileNavElements = this.page.locator('nav, [role="navigation"], button[aria-expanded]');
    await mobileNavElements.first().waitFor({ state: 'visible' });
  }

  async testTabletLayout() {
    await this.page.setViewportSize({ width: 768, height: 1024 });
    await this.dashboard.goto();
    await this.dashboard.expectFullDashboardLoaded();
  }

  async testDesktopLayout() {
    await this.page.setViewportSize({ width: 1920, height: 1080 });
    await this.dashboard.goto();
    await this.dashboard.expectFullDashboardLoaded();
  }
}