# React Frontend Patterns Guide

This guide covers the React patterns, best practices, and architectural decisions used in the web frontend. These patterns ensure type safety, performance, maintainability, and consistency across the application.

## Table of Contents

1. [Project Architecture](#project-architecture)
2. [React Query Patterns](#react-query-patterns)
3. [State Management](#state-management)
4. [Component Patterns](#component-patterns)
5. [TypeScript Integration](#typescript-integration)
6. [Authentication & Authorization](#authentication--authorization)
7. [Routing Patterns](#routing-patterns)
8. [Performance Optimization](#performance-optimization)
9. [Error Handling](#error-handling)
10. [Testing Patterns](#testing-patterns)

## Project Architecture

### Tech Stack Overview

```
Frontend Stack:
├── React 18 (with Concurrent Features)
├── TypeScript (strict mode)
├── TanStack Router (file-based routing)
├── TanStack Query (server state)
├── shadcn/ui + Tailwind CSS 4
├── Vite (build tool + dev server)
└── Biome (linting + formatting)
```

### Directory Structure

```
web/src/
├── components/          # Reusable UI components
│   ├── ui/             # shadcn/ui components
│   ├── admin/          # Admin-specific components
│   ├── auth/           # Authentication components
│   └── layout/         # Layout components
├── hooks/              # Custom React hooks
│   ├── useApiQueries.ts # Centralized API queries
│   └── use-*.ts        # Feature-specific hooks
├── lib/                # Utility libraries
│   ├── api/            # API client & types
│   ├── auth/           # Authentication logic
│   └── utils.ts        # Helper functions
├── routes/             # File-based routing
│   ├── admin/          # Admin dashboard routes
│   ├── auth/           # Authentication routes
│   └── index.tsx       # Public routes
└── types/              # TypeScript definitions
    └── api.ts          # Generated API types
```

## React Query Patterns

### Centralized API Queries

**Problem**: Cache collisions, inconsistent data transformations, and manual query key management.

**Solution**: Centralized hooks with standardized patterns.

```typescript
// ❌ Bad: Manual queries with potential cache collisions
const { data: healthStatus } = useQuery({
  queryKey: ["health", "basic"],  // Same key, different transforms
  queryFn: async () => {
    const response = await apiClient.getHealth();
    return response.data;  // Some extract data
  },
});

const { data: healthData } = useQuery({
  queryKey: ["health", "basic"],  // COLLISION!
  queryFn: () => apiClient.getHealth(), // Some return full response
});

// ✅ Good: Centralized hooks with consistent behavior
import { useHealthBasic, useTaskStats } from "@/hooks/useApiQueries";

const { data: healthStatus } = useHealthBasic(15000); // Type-safe, consistent
const { data: taskStats } = useTaskStats(10000);     // No cache collisions
```

### Query Hook Patterns

```typescript
// web/src/hooks/useApiQueries.ts
import { useQuery } from "@tanstack/react-query";
import { apiClient } from "@/lib/api/client";

// Standard refetch intervals
const REFETCH_INTERVALS = {
  FAST: 5000,      // Real-time components
  NORMAL: 15000,   // Regular updates  
  SLOW: 30000,     // Less critical data
} as const;

// ✅ Consistent hook pattern
export function useHealthBasic(refetchInterval?: number) {
  return useQuery({
    queryKey: ["health", "basic"],
    queryFn: async () => {
      const response = await apiClient.getHealth();
      return response.data!; // Always extract data consistently
    },
    refetchInterval: refetchInterval ?? REFETCH_INTERVALS.NORMAL,
  });
}

// ✅ Type-safe query keys for cache invalidation
export const QUERY_KEYS = {
  health: {
    basic: ["health", "basic"] as const,
    detailed: ["health", "detailed"] as const,
  },
  tasks: {
    stats: ["tasks", "stats"] as const,
    list: (filters?: Record<string, string>) => ["tasks", "list", filters] as const,
  },
} as const;
```

### Cache Invalidation Patterns

```typescript
// ✅ Type-safe cache invalidation
import { QUERY_KEYS } from "@/hooks/useApiQueries";

const deleteTaskMutation = useMutation({
  mutationFn: (taskId: string) => apiClient.deleteTask(taskId),
  onSuccess: () => {
    // Use constants instead of hardcoded strings
    queryClient.invalidateQueries({ queryKey: QUERY_KEYS.tasks.list() });
    queryClient.invalidateQueries({ queryKey: QUERY_KEYS.tasks.stats });
  },
});
```

### Error Handling in Queries

```typescript
// ✅ Consistent error handling
export function useTasksWithError() {
  return useQuery({
    queryKey: ["tasks"],
    queryFn: async () => {
      try {
        const response = await apiClient.getTasks();
        return response.data;
      } catch (error) {
        // Log for debugging but let React Query handle the error state
        console.error("Failed to fetch tasks:", error);
        throw error;
      }
    },
    retry: (failureCount, error) => {
      // Don't retry on 4xx errors
      if (error instanceof Error && error.message.includes("4")) {
        return false;
      }
      return failureCount < 3;
    },
  });
}
```

## State Management

### Server State vs Client State

```typescript
// ✅ Clear separation of concerns

// Server State: Use React Query
const { data: tasks, isLoading } = useTaskStats();
const { data: currentUser } = useCurrentUser();

// Client State: Use useState/useReducer
const [selectedFilters, setSelectedFilters] = useState({
  status: "",
  type: "",
});

// Global Client State: Context when needed
const { theme, setTheme } = useTheme();
```

### Authentication State Pattern

```typescript
// web/src/lib/auth/context.tsx
interface AuthContextType {
  user: AuthUser | null;
  isLoading: boolean;
  login: (credentials: LoginRequest) => Promise<void>;
  logout: () => Promise<void>;
  hasRole: (role: UserRole) => boolean;
}

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [isInitialized, setIsInitialized] = useState(false);
  
  // Use React Query for user data
  const { data: user, isLoading } = useQuery({
    queryKey: ["auth", "me"],
    queryFn: async () => {
      const token = getAuthToken();
      if (!token) return null;
      
      const response = await apiClient.getCurrentUser();
      return response.data;
    },
    retry: false, // Don't retry failed auth requests
  });

  const login = async (credentials: LoginRequest) => {
    const response = await apiClient.login(credentials);
    setAuthToken(response.data!.session_token);
    queryClient.invalidateQueries({ queryKey: ["auth", "me"] });
  };

  return (
    <AuthContext.Provider value={{ user, isLoading, login, logout, hasRole }}>
      {children}
    </AuthContext.Provider>
  );
}
```

### Form State Management

```typescript
// ✅ React Hook Form with TypeScript and validation
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";

const taskSchema = z.object({
  task_type: z.string().min(1, "Task type is required"),
  priority: z.enum(["low", "normal", "high"]),
  payload: z.record(z.unknown()),
});

type TaskFormData = z.infer<typeof taskSchema>;

export function CreateTaskForm() {
  const form = useForm<TaskFormData>({
    resolver: zodResolver(taskSchema),
    defaultValues: {
      task_type: "",
      priority: "normal",
      payload: {},
    },
  });

  const createTaskMutation = useMutation({
    mutationFn: (data: TaskFormData) => apiClient.createTask(data),
    onSuccess: () => {
      form.reset();
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.tasks.list() });
    },
  });

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(createTaskMutation.mutate)}>
        {/* Form fields */}
      </form>
    </Form>
  );
}
```

## Component Patterns

### Container/Presentational Pattern

```typescript
// ✅ Separate data fetching from presentation

// Container Component (handles data & logic)
export function TaskListContainer() {
  const { data: tasks, isLoading, error } = useTasks();
  const deleteTaskMutation = useDeleteTask();

  if (isLoading) return <TaskListSkeleton />;
  if (error) return <ErrorMessage error={error} />;

  return (
    <TaskListPresentation 
      tasks={tasks || []}
      onDeleteTask={deleteTaskMutation.mutate}
      isDeleting={deleteTaskMutation.isPending}
    />
  );
}

// Presentational Component (pure, testable)
interface TaskListPresentationProps {
  tasks: Task[];
  onDeleteTask: (id: string) => void;
  isDeleting: boolean;
}

export function TaskListPresentation({ 
  tasks, 
  onDeleteTask, 
  isDeleting 
}: TaskListPresentationProps) {
  return (
    <div>
      {tasks.map(task => (
        <TaskCard 
          key={task.id}
          task={task}
          onDelete={() => onDeleteTask(task.id)}
          disabled={isDeleting}
        />
      ))}
    </div>
  );
}
```

### Compound Component Pattern

```typescript
// ✅ Flexible, composable components
export function DataTable({ children }: { children: React.ReactNode }) {
  return <div className="data-table">{children}</div>;
}

DataTable.Header = function DataTableHeader({ children }: { children: React.ReactNode }) {
  return <header className="data-table-header">{children}</header>;
};

DataTable.Body = function DataTableBody({ children }: { children: React.ReactNode }) {
  return <div className="data-table-body">{children}</div>;
};

// Usage
<DataTable>
  <DataTable.Header>
    <h2>Tasks</h2>
    <CreateTaskButton />
  </DataTable.Header>
  <DataTable.Body>
    <TaskList />
  </DataTable.Body>
</DataTable>
```

### Error Boundary Pattern

```typescript
// web/src/components/ErrorBoundary.tsx
interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<
  PropsWithChildren<{ fallback?: ComponentType<{ error: Error }> }>,
  ErrorBoundaryState
> {
  constructor(props: PropsWithChildren<{}>) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("Error boundary caught an error:", error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      const FallbackComponent = this.props.fallback || DefaultErrorFallback;
      return <FallbackComponent error={this.state.error!} />;
    }

    return this.props.children;
  }
}
```

## TypeScript Integration

### Generated API Types

```typescript
// ✅ Use generated types from OpenAPI schema
import type { components } from "@/types/api";

type TaskResponse = components["schemas"]["TaskResponse"];
type CreateTaskRequest = components["schemas"]["CreateTaskApiRequest"];

// Type-safe API client methods
export const apiClient = {
  async createTask(data: CreateTaskRequest): Promise<ApiResponse<TaskResponse>> {
    return this.request<ApiResponse<TaskResponse>>("/tasks", {
      method: "POST",
      body: JSON.stringify(data),
    });
  },
};
```

### Custom Hook Typing

```typescript
// ✅ Proper TypeScript for hooks
interface UseTaskFiltersReturn {
  filters: TaskFilters;
  setFilter: <K extends keyof TaskFilters>(key: K, value: TaskFilters[K]) => void;
  clearFilters: () => void;
  hasActiveFilters: boolean;
}

export function useTaskFilters(initialFilters: TaskFilters = {}): UseTaskFiltersReturn {
  const [filters, setFilters] = useState<TaskFilters>(initialFilters);

  const setFilter = useCallback(<K extends keyof TaskFilters>(
    key: K, 
    value: TaskFilters[K]
  ) => {
    setFilters(prev => ({ ...prev, [key]: value }));
  }, []);

  const clearFilters = useCallback(() => {
    setFilters({});
  }, []);

  const hasActiveFilters = useMemo(() => {
    return Object.keys(filters).length > 0;
  }, [filters]);

  return { filters, setFilter, clearFilters, hasActiveFilters };
}
```

## Authentication & Authorization

### Role-Based Component Protection

```typescript
// web/src/components/auth/RoleGuard.tsx
interface RoleGuardProps {
  children: React.ReactNode;
  requiredRole?: UserRole;
  fallback?: React.ReactNode;
}

export function RoleGuard({ children, requiredRole, fallback }: RoleGuardProps) {
  const { user, hasRole } = useAuth();

  if (!user) {
    return <Navigate to="/auth/login" />;
  }

  if (requiredRole && !hasRole(requiredRole)) {
    return fallback || <AccessDenied />;
  }

  return <>{children}</>;
}

// Usage
<RoleGuard requiredRole="admin">
  <AdminOnlyFeature />
</RoleGuard>
```

### Protected Route Pattern

```typescript
// web/src/lib/auth/ProtectedRoute.tsx
export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, isLoading } = useAuth();

  if (isLoading) {
    return <LoadingSpinner />;
  }

  if (!user) {
    return <Navigate to="/auth/login" replace />;
  }

  return <>{children}</>;
}
```

## Routing Patterns

### File-Based Routing with TanStack Router

```typescript
// web/src/routes/admin/tasks/$taskId.tsx
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/admin/tasks/$taskId")({
  component: TaskDetailPage,
  loader: ({ params }) => {
    // Pre-load data
    return queryClient.ensureQueryData({
      queryKey: ["task", params.taskId],
      queryFn: () => apiClient.getTask(params.taskId),
    });
  },
});

function TaskDetailPage() {
  const { taskId } = Route.useParams();
  const { data: task } = useQuery({
    queryKey: ["task", taskId],
    queryFn: () => apiClient.getTask(taskId),
  });

  return <TaskDetail task={task} />;
}
```

### Route-Level Error Handling

```typescript
// web/src/routes/__root.tsx
export const Route = createRootRoute({
  component: RootComponent,
  errorComponent: ({ error }) => (
    <ErrorBoundary>
      <div>Something went wrong: {error.message}</div>
    </ErrorBoundary>
  ),
});
```

## Performance Optimization

### Query Optimization

```typescript
// ✅ Smart data fetching patterns
export function useSmartHealthQueries() {
  // Basic health for most components
  const basicHealth = useHealthBasic();
  
  // Only fetch detailed health when needed
  const detailedHealth = useQuery({
    queryKey: ["health", "detailed"],
    queryFn: () => apiClient.getDetailedHealth(),
    enabled: basicHealth.data?.status !== "healthy", // Conditional fetching
    refetchInterval: basicHealth.data?.status === "healthy" ? false : 5000,
  });

  return { basicHealth, detailedHealth };
}
```

### Component Memoization

```typescript
// ✅ Proper memoization patterns
export const TaskCard = memo(function TaskCard({ task, onAction }: TaskCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>{task.task_type}</CardTitle>
      </CardHeader>
      <CardContent>
        <TaskActions task={task} onAction={onAction} />
      </CardContent>
    </Card>
  );
});

// ✅ Memoize expensive calculations
const taskMetrics = useMemo(() => {
  return calculateTaskMetrics(tasks);
}, [tasks]);
```

### Lazy Loading

```typescript
// ✅ Route-based code splitting
const AdminDashboard = lazy(() => import("./routes/admin/index"));
const TaskManagement = lazy(() => import("./routes/admin/tasks/index"));

function App() {
  return (
    <Router>
      <Suspense fallback={<LoadingSpinner />}>
        <Routes>
          <Route path="/admin" element={<AdminDashboard />} />
          <Route path="/admin/tasks" element={<TaskManagement />} />
        </Routes>
      </Suspense>
    </Router>
  );
}
```

## Error Handling

### Query Error Handling

```typescript
// ✅ Centralized error handling
export function useTasksWithErrorHandling() {
  return useQuery({
    queryKey: ["tasks"],
    queryFn: async () => {
      const response = await apiClient.getTasks();
      return response.data;
    },
    throwOnError: (error, query) => {
      // Log errors but don't throw for expected 4xx errors
      if (error instanceof Error && error.message.includes("401")) {
        // Redirect to login
        return false;
      }
      return true;
    },
  });
}
```

### Global Error Handler

```typescript
// web/src/lib/errorHandling.ts
export function setupGlobalErrorHandling() {
  // React Query global error handler
  queryClient.setMutationDefaults(["global"], {
    onError: (error) => {
      if (error instanceof Error) {
        toast({
          title: "Something went wrong",
          description: error.message,
          variant: "destructive",
        });
      }
    },
  });

  // Global unhandled errors
  window.addEventListener("unhandledrejection", (event) => {
    console.error("Unhandled promise rejection:", event.reason);
    event.preventDefault();
  });
}
```

## Testing Patterns

### Component Testing

```typescript
// web/src/components/__tests__/TaskCard.test.tsx
import { render, screen, fireEvent } from "@testing-library/react";
import { TaskCard } from "../TaskCard";

const mockTask = {
  id: "1",
  task_type: "email",
  status: "pending",
  created_at: new Date().toISOString(),
};

describe("TaskCard", () => {
  it("renders task information correctly", () => {
    render(<TaskCard task={mockTask} onAction={jest.fn()} />);
    
    expect(screen.getByText("email")).toBeInTheDocument();
    expect(screen.getByText("pending")).toBeInTheDocument();
  });

  it("calls onAction when action button is clicked", () => {
    const mockOnAction = jest.fn();
    render(<TaskCard task={mockTask} onAction={mockOnAction} />);
    
    fireEvent.click(screen.getByRole("button", { name: /cancel/i }));
    
    expect(mockOnAction).toHaveBeenCalledWith("cancel", mockTask.id);
  });
});
```

### Query Testing

```typescript
// web/src/hooks/__tests__/useApiQueries.test.ts
import { renderHook, waitFor } from "@testing-library/react";
import { useHealthBasic } from "../useApiQueries";
import { createQueryWrapper } from "../../test/utils";

describe("useHealthBasic", () => {
  it("fetches health data successfully", async () => {
    const { result } = renderHook(() => useHealthBasic(), {
      wrapper: createQueryWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    
    expect(result.current.data).toEqual({
      status: "healthy",
      version: "0.1.0",
      uptime: expect.any(Number),
    });
  });
});
```

## Best Practices Summary

### 🎯 Query Patterns
- ✅ Use centralized hooks (`useApiQueries.ts`)
- ✅ Consistent data extraction (`response.data`)
- ✅ Type-safe query keys (`QUERY_KEYS`)
- ✅ Proper refetch intervals
- ❌ Avoid manual `useQuery` in components

### 🎯 State Management
- ✅ Server state: React Query
- ✅ Client state: `useState`/`useReducer`
- ✅ Global state: Context (sparingly)
- ❌ Don't mix server and client state

### 🎯 Component Design
- ✅ Container/Presentational separation
- ✅ Compound components for flexibility
- ✅ Proper TypeScript typing
- ✅ Error boundaries for resilience

### 🎯 Performance
- ✅ Memoization where appropriate
- ✅ Lazy loading for routes
- ✅ Conditional query fetching
- ✅ Smart refetch strategies

### 🎯 TypeScript
- ✅ Generated API types
- ✅ Strict mode enabled
- ✅ Proper hook typing
- ✅ Type-safe query keys

## Future Extensions

This guide will be extended with:

- **Advanced Patterns**: Optimistic updates, infinite queries, parallel queries
- **Performance**: Virtual scrolling, image optimization, bundle analysis
- **Testing**: E2E testing patterns, visual regression testing
- **Accessibility**: ARIA patterns, keyboard navigation, screen reader support
- **Monitoring**: Error tracking, performance metrics, user analytics

---

**Next Steps**: Implement these patterns consistently across your components and create custom hooks for common use cases specific to your domain.