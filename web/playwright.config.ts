import { defineConfig, devices } from '@playwright/test';

/**
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './e2e',
  testMatch: '**/*.spec.ts',
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : undefined,
  /* Stop immediately after first test failure - fail fast */
  maxFailures: 1,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ['html'],
    ['junit', { outputFile: 'test-results/junit.xml' }],
    ['json', { outputFile: 'test-results/results.json' }]
  ],
  /* Global test timeout - max 15 seconds per test */
  timeout: 15 * 1000,
  /* Expect timeout for assertions */
  expect: {
    /* Timeout for expect() calls - max 3 seconds per assertion */
    timeout: 3000,
    /* Screenshot comparison threshold */
    toHaveScreenshot: { 
      threshold: 0.2  // Allow small visual differences
    },
    toMatchSnapshot: { 
      threshold: 0.2 
    }
  },
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: process.env.PLAYWRIGHT_BASE_URL || 'http://localhost:5173',
    
    /* Action timeout - max 5 seconds per action */
    actionTimeout: 5 * 1000,
    
    /* Navigation timeout - max 10 seconds per page load */
    navigationTimeout: 10 * 1000,

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: 'on-first-retry',
    
    /* Screenshots on failure */
    screenshot: 'only-on-failure',
    
    /* Video recording on failure */
    video: 'retain-on-failure',
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },

    // Additional browsers for comprehensive testing (commented out for speed)
    // ...(process.env.PLAYWRIGHT_SMOKE_ONLY ? [] : [
    //   {
    //     name: 'firefox',
    //     use: { ...devices['Desktop Firefox'] },
    //   },

    //   {
    //     name: 'webkit',
    //     use: { ...devices['Desktop Safari'] },
    //   },

    //   /* Test against mobile viewports. */
    //   {
    //     name: 'Mobile Chrome',
    //     use: { ...devices['Pixel 5'] },
    //   },
    //   {
    //     name: 'Mobile Safari',
    //     use: { ...devices['iPhone 12'] },
    //   },
    // ]),

    /* Test against branded browsers. */
    // {
    //   name: 'Microsoft Edge',
    //   use: { ...devices['Desktop Edge'], channel: 'msedge' },
    // },
    // {
    //   name: 'Google Chrome',
    //   use: { ...devices['Desktop Chrome'], channel: 'chrome' },
    // },
  ],

  /* Run your local dev server before starting the tests */
  // webServer: {
  //   command: 'pnpm run dev', 
  //   url: 'http://localhost:5173',
  //   reuseExistingServer: true,  // Always reuse existing server - don't start new one
  //   timeout: 5 * 1000,  // 5 seconds max to check if server exists
  // },
});