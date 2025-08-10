# Frontend Web Application

Modern React/TypeScript frontend built with [TanStack Router](https://tanstack.com/router) and comprehensive testing infrastructure.

## Stack

- **React 18** with TypeScript
- **TanStack Router** - Type-safe file-based routing
- **TanStack Query** - Server state management  
- **Tailwind CSS** - Utility-first styling
- **shadcn/ui** - High-quality component library
- **Biome** - Linting and formatting
- **Vitest** - Unit and integration testing
- **Playwright** - End-to-end testing

## Getting Started

```bash
# Install dependencies
pnpm install

# Start development server (frontend only)
pnpm dev                    # http://localhost:5173

# Start with backend integration  
cd .. && ./scripts/dev-server.sh    # Full-stack setup
```

## Testing - 194 Total Tests

Comprehensive testing strategy with unit, integration, and E2E coverage:

```bash
# Unit Tests (135) - Fast feedback, mocked dependencies
pnpm test:unit                    # ~2s execution

# Integration Tests (46) - Real backend API communication  
pnpm test:integration             # Requires running server

# E2E Tests (13) - Complete user workflows
pnpm test:e2e                     # Full browser testing

# All tests
pnpm test                         # Run everything
```

**Test Categories:**
- **API Client** (32 tests) - Complete endpoint coverage with error handling
- **React Hooks** (25 tests) - TanStack Query integration with mock/real modes
- **RBAC & Auth** (57 tests) - Permission systems and authentication guards  
- **Type Guards & Utils** (16 tests) - TypeScript utilities and validation
- **E2E Workflows** (13 tests) - Registration → login → dashboard flows
- **Integration** (46 tests) - Real API communication with server health checks

**Key Features:**
- **Stateless Design** - Tests use unique data (no cleanup dependencies)
- **Resilient Patterns** - Handle race conditions and async operations gracefully
- **Browser Coverage** - E2E tests pass on Chromium, Firefox, WebKit, Mobile
- **CI-Ready** - Fast unit tests for development, comprehensive for validation

## Development Commands

```bash
# Development
pnpm dev                         # Start dev server (port 5173)
pnpm build                       # Production build
pnpm serve                       # Preview production build

# Code Quality
pnpm lint                        # Biome linting
pnpm format                      # Code formatting  
pnpm check                       # Comprehensive checks
pnpm typecheck                   # TypeScript validation

# Testing (detailed)
pnpm test:unit                   # 135 unit tests (~2s)
pnpm test:integration            # 46 integration tests (needs server)
pnpm test:e2e                    # 13 E2E tests (all browsers)
pnpm test:watch                  # Watch mode for unit tests
pnpm test:coverage               # Coverage reports

# Quality Validation (comprehensive)
./scripts/check-web.sh           # All checks: deps, types, lint, build, tests
```

## Project Structure

```
web/
├── src/
│   ├── components/          # Reusable UI components
│   │   ├── ui/             # shadcn/ui components
│   │   ├── auth/           # Authentication forms  
│   │   └── layout/         # Layout components
│   ├── hooks/              # Custom React hooks
│   │   └── __tests__/      # Hook unit & integration tests
│   ├── lib/
│   │   ├── api/            # API client & types
│   │   ├── auth/           # Authentication context
│   │   └── rbac/           # Role-based access control
│   ├── routes/             # File-based routing (TanStack Router)
│   ├── test/               # Test utilities & setup
│   │   ├── mocks.ts        # Mock factories
│   │   ├── setup.ts        # Test configuration
│   │   └── integration-setup.ts  # Integration test helpers
│   └── types/              # Generated API types
├── e2e/                    # Playwright E2E tests  
├── scripts/                # Development scripts
└── public/                 # Static assets
```

## API Integration

The frontend integrates with the Rust backend API:

- **Base URL**: `http://localhost:3000/api/v1`
- **Authentication**: Session-based with bearer tokens
- **Generated Types**: Auto-generated from OpenAPI spec (`src/types/api.ts`)
- **Query Hooks**: TanStack Query integration (`src/hooks/useApiQueries.ts`)

```typescript
// API client usage
import { apiClient } from '@/lib/api/client';

const user = await apiClient.getCurrentUser();
const tasks = await apiClient.getTasks();
```

## Component Library

Built with [shadcn/ui](https://ui.shadcn.com/) components:

```bash
# Add new components  
pnpx shadcn@latest add button
pnpx shadcn@latest add dialog
pnpx shadcn@latest add form
```

## Routing

File-based routing with [TanStack Router](https://tanstack.com/router):

- Routes defined in `src/routes/`  
- Type-safe navigation and params
- Layout support with `__root.tsx`
- Automatic route generation

```tsx
// Navigation
import { Link } from '@tanstack/react-router';

<Link to="/auth/login">Login</Link>
```

## State Management

- **Server State**: TanStack Query for API data
- **Local State**: React hooks and context
- **Authentication**: Auth context provider
- **Forms**: React Hook Form with Zod validation

## Styling

**Tailwind CSS** with shadcn/ui design system:

- Utility-first CSS framework
- Dark/light mode support  
- Consistent design tokens
- Responsive design utilities

## Environment Configuration

```bash
# Environment files
.env.local                   # Local development overrides
.env                         # Development defaults

# Key variables
VITE_API_BASE_URL           # Backend API URL (default: auto-detected)
```

## Production Build

```bash
# Build for production
pnpm build                   # Output: dist/

# Preview build locally
pnpm serve                   # Test production build

# Serve with backend (recommended)
cd .. && ./scripts/dev-server.sh    # Rust serves frontend
```

The production build is optimized and can be served directly by the Rust backend or any static file server.