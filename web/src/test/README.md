# Frontend Testing Guide

This directory contains the comprehensive testing setup for the frontend API client, hooks, and utilities.

## Testing Strategy

We use a **dual testing approach** with both unit tests (mocked) and integration tests (real server) to ensure comprehensive coverage and fast feedback loops.

## Test Types

### 1. Unit Tests (Fast - Mocked)
- **Location**: `**/*.test.ts` files
- **Speed**: ~50-200ms per suite
- **Mock Strategy**: Mock `fetch()` and API client
- **Run with**: `npm run test:unit`

#### What's Tested:
- API client HTTP request construction
- Authentication token handling
- Error handling and response parsing
- Hook behavior with TanStack Query
- RBAC utilities and type guards

### 2. Integration Tests (Real Server)
- **Location**: `**/*.integration.test.ts` files  
- **Speed**: ~2-5s per suite
- **Real Dependencies**: Requires running backend server
- **Run with**: `npm run test:integration`

#### What's Tested:
- Full API client against real server
- End-to-end data flow through hooks
- Real authentication and task workflows
- Network error handling
- Cache behavior with live data

## Available Scripts

```bash
# Run all unit tests (fast, no server needed)
npm run test:unit

# Run all integration tests (requires server)
npm run test:integration

# Skip integration tests (useful in CI when server unavailable)
npm run test:integration:skip

# Run both unit and integration tests
npm run test:all

# Watch mode for development (unit tests only)
npm run test:watch:unit

# Generate coverage report
npm run test:coverage
npm run test:coverage:unit

# Original test command (runs all tests)
npm test
```

## Integration Test Setup

Integration tests require a running backend server. Before running integration tests:

1. **Start the development server:**
   ```bash
   # In the root directory
   ./scripts/dev-server.sh
   ```

2. **Or start individual services:**
   ```bash
   ./scripts/server.sh    # API server on port 3000
   ./scripts/worker.sh    # Background worker
   ```

3. **Run integration tests:**
   ```bash
   cd web
   npm run test:integration
   ```

### Server Auto-Detection

Integration tests will:
- Wait up to 30 seconds for server to be ready
- Skip if `SKIP_INTEGRATION=true` environment variable is set
- Automatically clean up created resources after each test

## File Structure

```
src/test/
├── README.md                     # This file
├── setup.ts                      # Global test setup
├── mocks.ts                      # Mock utilities and test data
└── integration-setup.ts          # Integration test helpers

src/lib/api/__tests__/
├── client.test.ts                # API client unit tests
└── client.integration.test.ts    # API client integration tests

src/hooks/__tests__/
├── useApiQueries.test.ts         # Hook unit tests
└── useApiQueries.integration.test.ts # Hook integration tests

src/lib/auth/__tests__/
└── guards.test.ts                # Auth guard unit tests

src/lib/rbac/__tests__/
└── types.test.ts                 # RBAC utility unit tests
```

## Test Data and Mocks

### Mock Factories (`src/test/mocks.ts`)
- `createMockResponse()` - HTTP response mocks
- `mockAuthUser`, `mockTask`, etc. - Typed test data
- `mockLocalStorage()` - localStorage mock
- `createTestUser()` - Generate unique test users for integration

### Test Setup
- Automatic localStorage mocking
- Fetch mock utilities for unit tests
- Server readiness checking for integration tests
- Resource cleanup after each test

## Writing Tests

### Unit Test Example
```typescript
import { vi, describe, it, expect, beforeEach } from "vitest";
import { apiClient } from "../client";
import { createMockFetch, mockApiResponse, mockHealthResponse } from "@/test/mocks";

describe("API Client", () => {
  let mockFetch: ReturnType<typeof createMockFetch>;

  beforeEach(() => {
    mockFetch = createMockFetch();
  });

  it("should fetch health data", async () => {
    mockFetch.mockResolvedValueOnce(
      createMockResponse(mockApiResponse(mockHealthResponse))
    );

    const result = await apiClient.getHealth();

    expect(result.data).toEqual(mockHealthResponse);
  });
});
```

### Integration Test Example
```typescript
import { describe, it, expect, beforeAll } from "vitest";
import { describeIntegration, setupIntegrationTest } from "@/test/integration-setup";
import { apiClient, setAuthToken } from "../client";
import { createTestUser } from "@/test/mocks";

describeIntegration("API Integration", () => {
  const { baseUrl } = setupIntegrationTest();

  beforeAll(() => {
    // Configure client for integration tests
    Object.assign(apiClient, new (apiClient.constructor as any)(baseUrl));
  });

  it("should authenticate and fetch data", async () => {
    const testUser = createTestUser();
    await apiClient.register(testUser);
    const loginResponse = await apiClient.login(testUser);
    
    setAuthToken(loginResponse.data?.session_token);
    
    const userResponse = await apiClient.getCurrentUser();
    expect(userResponse.data?.email).toBe(testUser.email);
  });
});
```

## Best Practices

### Unit Tests
- Mock external dependencies (`fetch`, `apiClient`)
- Test error conditions and edge cases
- Verify request construction and response handling
- Keep tests fast and deterministic

### Integration Tests
- Use real API client against running server
- Clean up created resources in `afterEach`
- Test full user workflows and error scenarios
- Use unique test data to avoid conflicts

### Hook Tests
- Use `renderHook` from `@testing-library/react`
- Provide `QueryClientProvider` wrapper
- Test loading, success, and error states
- Verify cache behavior and query keys

## Debugging Tests

### Unit Test Issues
- Check mock setup in `beforeEach`
- Verify mock return values match expected types
- Ensure cleanup in `afterEach` to avoid test pollution

### Integration Test Issues
- Verify server is running on correct port (3000)
- Check network connectivity to server
- Review server logs for API errors
- Ensure database is in clean state

### Common Problems
- **Tests hanging**: Server not responding, check server status
- **Auth errors**: Token not set or expired
- **Resource conflicts**: Previous tests didn't clean up properly
- **Type errors**: API response shape changed, update mocks

## CI/CD Integration

For continuous integration:

1. **PR Checks**: Run unit tests only
   ```bash
   npm run test:unit
   ```

2. **Release Pipeline**: Run full test suite
   ```bash
   npm run test:all
   ```

3. **Skip Integration**: When server unavailable
   ```bash
   SKIP_INTEGRATION=true npm run test:integration
   ```

This dual approach ensures fast feedback during development while maintaining confidence in the full system integration.