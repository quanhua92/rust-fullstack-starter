# Frontend E2E Testing with Playwright

This guide covers the comprehensive Playwright end-to-end testing setup for the web frontend, including configuration options, test modes, and integration with the quality check pipeline.

## Overview

The Playwright testing setup provides multiple testing modes to balance speed and coverage:

- **Smoke Tests**: Ultra-fast basic functionality validation (~400ms E2E, ~16s total with quality checks)
- **Single Browser**: Comprehensive tests (Chromium only, ~1.2s E2E, ~12s total with optimized timeouts)
- **Multi-Browser**: Full cross-browser testing (Chrome, Firefox, Safari, Mobile, ~5-10min estimated)

## Quick Start

```bash
# Install Playwright browsers (one-time setup)
npx playwright install

# Quality check scripts with smart server management:
./scripts/check-web.sh --skip-lint --smoke    # Ultra-fast (~16s)
./scripts/check-web.sh --skip-lint            # Fast single-browser (~12s)
./scripts/check-web.sh --skip-lint --full     # Full multi-browser (~5-10min)

# Direct Playwright commands (requires manual server setup):
pnpm run test:e2e:smoke                       # Smoke tests (~400ms)
pnpm run test:e2e --project=chromium          # Single browser (~1.2s)
pnpm run test:e2e                             # All browsers (~5-10min)
pnpm run test:e2e:ui                          # Interactive debugging
```

## Test Scripts Reference

### Available Scripts

| Script | Description | Speed | Browsers |
|--------|-------------|-------|----------|
| `pnpm run test:e2e:smoke` | Basic functionality check | ~400ms | Chromium only |
| `pnpm run test:e2e` | Full test suite | ~5-10min | All browsers |
| `pnpm run test:e2e --project=chromium` | Single browser tests | ~1.2s | Chromium only |
| `pnpm run test:e2e:ui` | Interactive test runner | Interactive | All browsers |
| `pnpm run test:e2e:debug` | Debug mode with DevTools | Interactive | Chromium only |
| `pnpm run test:e2e:headed` | Run with visible browser | Normal | All browsers |
| `pnpm run test:e2e:report` | Show test results report | Instant | N/A |

### Advanced Parameters

| Parameter | Effect | Default | Example |
|-----------|--------|---------|----------|
| `--skip-lint` | Skip linting and formatting checks | false | `./scripts/check-web.sh --skip-lint` |
| `--smoke` | Run only smoke tests | false | `./scripts/check-web.sh --smoke` |
| `--full` | Run multi-browser tests | false | `./scripts/check-web.sh --full` |
| `--max-failures=N` | Stop after N test failures | 1 | `./scripts/check-web.sh --max-failures=3` |
| `--no-fail-fast` | Run all tests regardless of failures | false | `./scripts/check-web.sh --no-fail-fast` |
| `--timeout=N` | Timeout per test in milliseconds | 5000 | `./scripts/check-web.sh --timeout=10000` |
| `--global-timeout=N` | Global timeout for entire suite in seconds | 120/30/600 | `./scripts/check-web.sh --global-timeout=60` |

### Environment Variables

| Variable | Effect | Example |
|----------|--------|---------|
| `PLAYWRIGHT_SMOKE_ONLY=true` | Limits to Chromium-only for all tests | `PLAYWRIGHT_SMOKE_ONLY=true pnpm run test:e2e` |
| `PLAYWRIGHT_BASE_URL` | Override base URL for tests | `PLAYWRIGHT_BASE_URL=http://localhost:8080` |
| `PLAYWRIGHT_SKIP=true` | Skip all E2E tests in check-web.sh | `PLAYWRIGHT_SKIP=true ./scripts/check-web.sh` |

## Integration with Quality Checks

The E2E tests are integrated into the `check-web.sh` quality pipeline with smart server management.

### Quality Check Options

```bash
# Default: Fast single-browser E2E tests (~12s total)
./scripts/check-web.sh --skip-lint

# Ultra-fast smoke tests only (~16s total)
./scripts/check-web.sh --skip-lint --smoke

# Full multi-browser testing (~5-10min total)
./scripts/check-web.sh --skip-lint --full

# Custom timeouts and failure control
./scripts/check-web.sh --skip-lint --global-timeout=60 --max-failures=3
./scripts/check-web.sh --skip-lint --timeout=3000 --global-timeout=90
./scripts/check-web.sh --skip-lint --no-fail-fast  # Run all tests regardless of failures

# Alternative syntax
PLAYWRIGHT_SMOKE_ONLY=true ./scripts/check-web.sh --skip-lint
PLAYWRIGHT_SKIP=true ./scripts/check-web.sh --skip-lint  # Skip E2E entirely
```

### Smart Server Management

The quality check script automatically manages the development environment:

1. **Backend Server (Port 3000)**: Auto-starts `scripts/server.sh` if not running
2. **Worker Process**: Auto-starts `scripts/worker.sh` if backend was started
3. **Frontend Dev Server (Port 5173)**: Auto-starts `pnpm run dev` if not running
4. **Cleanup**: Automatically stops any servers it started on exit

## Test Structure

### Test Files

```
web/e2e/
├── smoke.spec.ts        # Basic Playwright functionality (no server needed)
├── example.spec.ts      # Application navigation and UI tests
├── auth.spec.ts         # Authentication flow tests
└── api-health.spec.ts   # Backend API health endpoint tests
```

### Test Categories

#### 1. Smoke Tests (`smoke.spec.ts`)
- **Purpose**: Verify Playwright is working correctly
- **Server Requirements**: None
- **Speed**: ~400ms
- **Coverage**: Basic browser automation

```typescript
test('playwright basic functionality works', async ({ page }) => {
  const version = await page.evaluate(() => navigator.userAgent);
  expect(typeof version).toBe('string');
  expect(version.length).toBeGreaterThan(0);
});
```

#### 2. Application Tests (`example.spec.ts`)
- **Purpose**: Test core application functionality
- **Server Requirements**: Backend + Frontend
- **Coverage**: Navigation, page loading, basic UI

```typescript
test('homepage loads successfully', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('body')).toBeVisible();
});
```

#### 3. Authentication Tests (`auth.spec.ts`)
- **Purpose**: Test user authentication flows
- **Server Requirements**: Backend + Frontend
- **Coverage**: Login, registration, form validation

```typescript
test('login page loads', async ({ page }) => {
  await page.goto('/auth/login');
  await expect(page.locator('input[type="email"]')).toBeVisible();
  await expect(page.locator('input[type="password"]')).toBeVisible();
});
```

#### 4. API Health Tests (`api-health.spec.ts`)
- **Purpose**: Test backend API endpoints
- **Server Requirements**: Backend only
- **Coverage**: Health endpoints, API documentation

```typescript
test('health endpoint responds', async ({ request }) => {
  const response = await request.get('/api/v1/health');
  expect(response.status()).toBe(200);
  
  const data = await response.json();
  expect(data.data.status).toBe('healthy');
  expect(data.success).toBe(true);
});
```

## Configuration

### Playwright Config (`playwright.config.ts`)

The configuration supports multiple testing modes:

```typescript
// Multi-browser configuration
projects: [
  { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  
  // Additional browsers (skipped when PLAYWRIGHT_SMOKE_ONLY=true)
  ...(process.env.PLAYWRIGHT_SMOKE_ONLY ? [] : [
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
    { name: 'Mobile Chrome', use: { ...devices['Pixel 5'] } },
    { name: 'Mobile Safari', use: { ...devices['iPhone 12'] } },
  ]),
]
```

### Key Configuration Features

- **Base URL**: Configurable via `PLAYWRIGHT_BASE_URL` (default: `http://localhost:3000`)
- **Parallel Execution**: Tests run in parallel for speed
- **Retry Logic**: Automatic retries on CI environments
- **Artifacts**: Screenshots, videos, and traces on failure
- **Reporters**: HTML, JUnit XML, and JSON output formats

## Development Workflow

### Local Development

1. **Start Development Environment**:
   ```bash
   # Option 1: Let check-web.sh manage everything
   ./scripts/check-web.sh --skip-lint
   
   # Option 2: Manual setup
   ./scripts/server.sh 3000
   ./scripts/worker.sh
   pnpm run dev
   ```

2. **Run Tests During Development**:
   ```bash
   # Quick validation
   pnpm run test:e2e:smoke
   
   # Test specific functionality
   pnpm run test:e2e auth.spec.ts --project=chromium
   
   # Debug failing tests
   pnpm run test:e2e:debug
   ```

3. **Before Committing**:
   ```bash
   # Ultra-fast smoke test validation
   ./scripts/check-web.sh --skip-lint --smoke
   
   # Full quality checks with fast E2E tests
   ./scripts/check-web.sh --skip-lint
   
   # Or comprehensive testing
   ./scripts/check-web.sh --skip-lint --full
   ```

### CI/CD Integration

For continuous integration environments:

```bash
# Ultra-fast CI validation (smoke tests)
./scripts/check-web.sh --skip-lint --smoke

# Fast CI validation (skip E2E entirely)
PLAYWRIGHT_SKIP=true ./scripts/check-web.sh --skip-lint

# Alternative smoke test syntax
PLAYWRIGHT_SMOKE_ONLY=true ./scripts/check-web.sh --skip-lint

# Full testing (nightly builds)
CI=true ./scripts/check-web.sh --skip-lint --full
```

## Troubleshooting

### Common Issues

#### 1. Server Connection Errors
```
Error: connect ECONNREFUSED ::1:3000
```
**Solution**: Ensure backend server is running on port 3000:
```bash
./scripts/server.sh 3000
```

#### 2. Test Conflicts with vitest
**Solution**: Tests are properly separated:
- E2E tests: `e2e/**/*.spec.ts` (Playwright)
- Unit tests: `src/**/*.test.ts` (vitest)

#### 3. Slow Test Execution
**Solution**: Use appropriate test mode:
```bash
# Instead of full multi-browser tests
pnpm run test:e2e

# Use single-browser for speed
pnpm run test:e2e --project=chromium

# Or smoke tests for validation
pnpm run test:e2e:smoke
```

#### 4. Browser Installation Issues
```bash
# Reinstall browsers
npx playwright install

# Install system dependencies (Linux)
npx playwright install-deps
```

### Debug Mode

For debugging failing tests:

```bash
# Debug mode with DevTools
pnpm run test:e2e:debug

# Headed mode to see browser
pnpm run test:e2e:headed

# Interactive UI
pnpm run test:e2e:ui
```

## Performance Optimization

### Test Speed Comparison

| Mode | E2E Duration | Total Quality Check | Browsers | Use Case |
|------|-------------|---------------------|----------|----------|
| Smoke | ~400ms | **~16s** ✅ | Chromium | Quick validation |
| Single Browser | **~1.2s** ✅ | **~12s** ✅ | Chromium | Development |
| Multi-Browser | Estimated | **~5-10min** (estimated) | All 5 | Pre-release |

**Note**: 
- ✅ **Verified**: Performance optimizations implemented with fast timeouts and parallel execution
- **Global Timeout**: Each mode has configurable global timeout limits (default: 30s/120s/600s)
- **Fail-Fast**: Tests stop on first failure by default for rapid feedback
- Total times include dependencies, API generation, TypeScript checking, building, and unit tests

### Speed Optimization Tips

1. **Use appropriate test mode** for your needs:
   - **Development**: `./scripts/check-web.sh --skip-lint --smoke` (~16s)
   - **Pre-commit**: `./scripts/check-web.sh --skip-lint` (~12s) 
   - **Pre-release**: `./scripts/check-web.sh --skip-lint --full` (~5-10min)
2. **Customize timeouts** for your environment:
   - **Fast feedback**: `./scripts/check-web.sh --skip-lint --global-timeout=60`
   - **Thorough testing**: `./scripts/check-web.sh --skip-lint --no-fail-fast --timeout=10000`
   - **CI/CD optimization**: `./scripts/check-web.sh --skip-lint --max-failures=1 --global-timeout=90`
2. **Run smoke tests** during active development for fastest feedback
3. **Use single-browser mode** for comprehensive testing without multi-browser overhead
4. **Reserve multi-browser mode** for final validation before releases
5. **Skip E2E tests** when only testing backend changes with `PLAYWRIGHT_SKIP=true`
6. **Smart server management** automatically handles server lifecycle - no manual setup needed

## Advanced Configuration

### Custom Test Environments

```bash
# Test against staging environment
PLAYWRIGHT_BASE_URL=https://staging.example.com pnpm run test:e2e

# Test with custom timeout
pnpm run test:e2e --timeout=60000

# Test specific browser only
pnpm run test:e2e --project=firefox
```

### Adding New Tests

1. **Create test file** in `e2e/` directory:
   ```typescript
   // e2e/my-feature.spec.ts
   import { test, expect } from '@playwright/test';
   
   test.describe('My Feature', () => {
     test('feature works correctly', async ({ page }) => {
       await page.goto('/my-feature');
       // Test implementation
     });
   });
   ```

2. **Follow naming convention**: `*.spec.ts`
3. **Use appropriate test category** (smoke, app, auth, api)
4. **Consider server requirements** when writing tests

### Integration with Backend

The E2E tests can validate full-stack functionality:

```typescript
test('end-to-end user workflow', async ({ page, request }) => {
  // API call
  const response = await request.post('/api/v1/users', {
    data: { username: 'testuser', email: 'test@example.com' }
  });
  
  // UI interaction
  await page.goto('/users');
  await expect(page.locator('text=testuser')).toBeVisible();
});
```

## Best Practices

1. **Start with smoke tests** for new features
2. **Use page object pattern** for complex interactions
3. **Keep tests independent** - don't rely on test order
4. **Use appropriate selectors** - prefer data-testid attributes
5. **Handle async operations** properly with waitFor methods
6. **Clean up test data** to avoid test pollution
7. **Use environment variables** for configuration
8. **Document test purpose** and requirements clearly

This comprehensive testing setup ensures reliable frontend quality while maintaining development velocity through flexible testing modes.