# Enhanced Playwright E2E Testing

**📖 See [../../docs/TESTING-GUIDE.md](../../docs/TESTING-GUIDE.md) for comprehensive testing documentation including the complete 7-layer testing architecture, workflows, and best practices.**

This directory contains comprehensive end-to-end tests using Playwright that focus on browser-specific functionality and user interactions that cannot be tested through API calls alone.

## Test Structure

### Core Test Files

#### `auth-enhanced.spec.ts`
Comprehensive authentication flow testing with detailed form validation:
- Real-time form validation testing
- Form submission states and loading indicators  
- Form field interactions and keyboard navigation
- Error handling for client and server-side validation
- Success states and redirect flows

#### `admin-dashboard.spec.ts`
Full dashboard UI and interaction testing:
- Dashboard component loading and layout
- Interactive elements (buttons, charts, navigation)
- Real-time data updates and loading states
- Responsive design testing
- Navigation between admin sections

#### `visual-regression.spec.ts`
Screenshot comparison testing for visual consistency:
- Login/register page screenshots
- Dashboard layout screenshots
- Form error state screenshots  
- Responsive design screenshots (mobile, tablet, desktop)
- Dark theme screenshots (if available)
- Loading and success state screenshots

#### `accessibility.spec.ts`
Comprehensive accessibility testing:
- Keyboard navigation through all interactive elements
- Screen reader support (ARIA labels, roles, descriptions)
- Focus management and visual focus indicators
- Form validation accessibility
- Color contrast and theme support
- Touch/mobile accessibility

### Page Object Models

#### `page-objects/AuthPage.ts`
Reusable page objects for authentication:
- `LoginPage` - Login form interactions and validations
- `RegisterPage` - Registration form with all field validations
- `TestDataGenerator` - Utility for generating unique test data

#### `page-objects/DashboardPage.ts`
Modular dashboard page objects:
- `DashboardPage` - Main dashboard orchestrator
- `DashboardStatsCards` - Stats section testing
- `DashboardSystemHealth` - Health status components
- `DashboardQuickActions` - Quick action buttons
- `DashboardSidebar` - Navigation sidebar
- `DashboardAnalytics` - Charts and real-time data
- `ResponsiveDashboard` - Mobile/tablet/desktop testing

### Legacy Tests (Still Available)

#### `auth.spec.ts` 
Original authentication tests (comprehensive user journey)
#### `api-health.spec.ts`
API endpoint health checking
#### `example.spec.ts` 
Basic application functionality tests
#### `smoke.spec.ts`
Minimal smoke tests

## Running Tests

### Individual Test Suites
```bash
# Enhanced form validation and interaction tests
npm run test:e2e:enhanced

# Page object model tests (cleaner, more maintainable)
npm run test:e2e:page-objects

# Visual regression testing
npm run test:e2e:visual

# Accessibility testing
npm run test:e2e:accessibility

# Quick smoke test
npm run test:e2e:smoke
```

### All Tests
```bash
# All E2E tests
npm run test:e2e

# Interactive test runner
npm run test:e2e:ui

# Debug mode
npm run test:e2e:debug

# With browser visible
npm run test:e2e:headed
```

### Visual Testing
```bash
# Update baseline screenshots
npm run test:e2e:update-snapshots

# View test report with screenshots
npm run test:e2e:report
```

## Test Philosophy

### What These Tests Focus On
- **UI-Specific Behaviors**: Form validation feedback, loading states, visual indicators
- **Browser Interactions**: Keyboard navigation, focus management, touch interactions  
- **Visual Consistency**: Layout, responsive design, theme variations
- **Accessibility**: Screen reader support, keyboard accessibility, ARIA compliance
- **User Experience**: Complete user journeys through the actual UI

### What These Tests Don't Duplicate
- **API Logic**: Covered by the existing 46 integration tests
- **Business Logic**: Covered by 135 unit tests
- **Data Validation**: Backend validation is tested in integration tests

## Key Features

### 1. Realistic User Interactions
- Tests use unique user data for each run (no cleanup dependencies)
- Forms are tested through actual UI interactions, not API calls
- Real browser rendering and interaction testing

### 2. Robust Error Handling  
- Tests gracefully handle authentication failures
- Network delays and API errors are handled appropriately
- Flexible selectors accommodate UI changes

### 3. Visual Regression Protection
- Baseline screenshots for key UI states
- Cross-device visual consistency testing
- Theme variation testing

### 4. Accessibility Compliance
- WCAG compliance testing
- Keyboard-only navigation testing
- Screen reader compatibility testing

### 5. Maintainable Architecture
- Page Object Model for reusable components
- Centralized test data generation
- Modular dashboard testing approach

## Browser Support

Tests run on:
- **Chromium** (primary - fastest feedback)
- **Firefox** (cross-browser compatibility)
- **WebKit/Safari** (Apple ecosystem)
- **Mobile Chrome** (responsive/touch testing)
- **Mobile Safari** (iOS testing)

Use `PLAYWRIGHT_SMOKE_ONLY=true` to run only on Chromium for faster feedback.

## Screenshots and Visual Testing

Screenshots are stored in `test-results/` and are automatically generated for:
- Test failures (debugging)
- Visual regression baselines
- Different viewport sizes
- Theme variations

Update baselines when UI intentionally changes:
```bash
npm run test:e2e:update-snapshots
```

## Integration with Existing Tests

These E2E tests complement the existing test suite:

| Test Type | Count | Purpose | Tools |
|-----------|-------|---------|--------|
| **Unit Tests** | 135 | Component logic, utilities | Vitest |
| **Integration Tests** | 46 | API workflows, business logic | Vitest + API client |  
| **E2E Tests** | 13+ | Browser UI, user interactions | Playwright |

Together, this provides comprehensive coverage from unit → integration → end-to-end.

## Tips for Writing New Tests

1. **Use Page Objects**: Extend existing page objects rather than writing raw selectors
2. **Focus on UI-Specific Features**: Don't duplicate what integration tests cover
3. **Be Resilient**: Use flexible selectors that can handle minor UI changes
4. **Test Real User Flows**: Complete journeys, not isolated actions
5. **Consider Accessibility**: Every new UI feature should be keyboard accessible

## Debugging Failed Tests

1. **Run with UI**: `npm run test:e2e:ui` for interactive debugging
2. **Run headed**: `npm run test:e2e:headed` to watch browser
3. **Check screenshots**: Failed tests automatically capture screenshots
4. **Use debug mode**: `npm run test:e2e:debug` for step-through debugging
5. **Check test report**: `npm run test:e2e:report` for detailed failure analysis