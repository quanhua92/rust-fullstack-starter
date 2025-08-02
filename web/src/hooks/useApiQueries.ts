/**
 * Centralized API query hooks for consistent data fetching and caching
 * 
 * This file ensures:
 * 1. Consistent query keys across the application
 * 2. Standardized data transformations  
 * 3. TypeScript safety for data shapes
 * 4. No cache collisions between components
 */

import { useQuery, type UseQueryResult } from "@tanstack/react-query";
import { apiClient } from "@/lib/api/client";
import type { components } from "@/types/api";

// Extract the inner data types for consistency
type HealthData = NonNullable<components["schemas"]["ApiResponse_HealthResponse"]["data"]>;
type TaskStatsData = NonNullable<components["schemas"]["ApiResponse_TaskStats"]["data"]>;
type DetailedHealthData = NonNullable<components["schemas"]["ApiResponse_DetailedHealthResponse"]["data"]>;

// Standard refetch intervals
const REFETCH_INTERVALS = {
  FAST: 5000,      // 5 seconds - for real-time components
  NORMAL: 15000,   // 15 seconds - for regular updates
  SLOW: 30000,     // 30 seconds - for less critical data
} as const;

/**
 * Health Queries - All return consistent data shapes
 */

// Basic health with extracted data (most common usage)
export function useHealthBasic(refetchInterval?: number): UseQueryResult<HealthData> {
  return useQuery({
    queryKey: ["health", "basic"],
    queryFn: async () => {
      const response = await apiClient.getHealth();
      return response.data!; // TypeScript knows this is HealthData
    },
    refetchInterval: refetchInterval ?? REFETCH_INTERVALS.NORMAL,
  });
}

// Detailed health with extracted data
export function useHealthDetailed(refetchInterval?: number): UseQueryResult<DetailedHealthData> {
  return useQuery({
    queryKey: ["health", "detailed"],
    queryFn: async () => {
      const response = await apiClient.getDetailedHealth();
      return response.data!; // TypeScript knows this is DetailedHealthData
    },
    refetchInterval: refetchInterval ?? REFETCH_INTERVALS.NORMAL,
  });
}

// Health probes (return unknown data, need type guards)
export function useHealthLiveness(refetchInterval?: number) {
  return useQuery({
    queryKey: ["health", "liveness"],
    queryFn: () => apiClient.getLivenessProbe(),
    refetchInterval: refetchInterval ?? REFETCH_INTERVALS.FAST,
  });
}

export function useHealthReadiness(refetchInterval?: number) {
  return useQuery({
    queryKey: ["health", "readiness"],
    queryFn: () => apiClient.getReadinessProbe(),
    refetchInterval: refetchInterval ?? REFETCH_INTERVALS.FAST,
  });
}

export function useHealthStartup(refetchInterval?: number) {
  return useQuery({
    queryKey: ["health", "startup"],
    queryFn: () => apiClient.getStartupProbe(),
    refetchInterval: refetchInterval ?? REFETCH_INTERVALS.NORMAL,
  });
}

/**
 * Task Stats Queries - Consistent data extraction
 */

// Task stats with extracted data (most common usage)
export function useTaskStats(refetchInterval?: number): UseQueryResult<TaskStatsData> {
  return useQuery({
    queryKey: ["tasks", "stats"],
    queryFn: async () => {
      const response = await apiClient.getTaskStats();
      return response.data!; // TypeScript knows this is TaskStatsData
    },
    refetchInterval: refetchInterval ?? REFETCH_INTERVALS.NORMAL,
  });
}

/**
 * User Queries
 */

export function useCurrentUser(refetchInterval?: number) {
  return useQuery({
    queryKey: ["auth", "me"],
    queryFn: async () => {
      const response = await apiClient.getCurrentUser();
      return response.data!;
    },
    refetchInterval: refetchInterval ?? REFETCH_INTERVALS.SLOW,
  });
}

/**
 * Utility: Get standard query keys for cache invalidation
 */
export const QUERY_KEYS = {
  health: {
    basic: ["health", "basic"] as const,
    detailed: ["health", "detailed"] as const,
    liveness: ["health", "liveness"] as const,
    readiness: ["health", "readiness"] as const,
    startup: ["health", "startup"] as const,
  },
  tasks: {
    stats: ["tasks", "stats"] as const,
    list: (filters?: Record<string, string>) => ["tasks", "list", filters] as const,
    detail: (id: string) => ["tasks", "detail", id] as const,
    types: ["tasks", "types"] as const,
    deadLetter: (filters?: Record<string, string>) => ["tasks", "deadLetter", filters] as const,
  },
  users: {
    me: ["auth", "me"] as const,
    list: (filters?: Record<string, string>) => ["users", "list", filters] as const,
    detail: (id: string) => ["users", "detail", id] as const,
    stats: ["users", "stats"] as const,
  },
} as const;

/**
 * Type-safe query key utilities
 */
export type QueryKey = readonly string[];