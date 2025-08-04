/**
 * Centralized API query hooks for consistent data fetching and caching
 *
 * This file ensures:
 * 1. Consistent query keys across the application
 * 2. Standardized data transformations
 * 3. TypeScript safety for data shapes
 * 4. No cache collisions between components
 */

import { apiClient } from "@/lib/api/client";
import type { components } from "@/types/api";
import { type UseQueryResult, useQuery } from "@tanstack/react-query";

// Extract the inner data types for consistency
type HealthData = NonNullable<
	components["schemas"]["ApiResponse_HealthResponse"]["data"]
>;
type TaskStatsData = NonNullable<
	components["schemas"]["ApiResponse_TaskStats"]["data"]
>;
type DetailedHealthData = NonNullable<
	components["schemas"]["ApiResponse_DetailedHealthResponse"]["data"]
>;

// Monitoring types (allowing undefined for proper error handling)
type MonitoringEventsData =
	components["schemas"]["ApiResponse_Vec_Event"]["data"];
type MonitoringEventData = components["schemas"]["ApiResponse_Event"]["data"];
type MonitoringMetricsData =
	components["schemas"]["ApiResponse_Vec_Metric"]["data"];
type MonitoringAlertsData =
	components["schemas"]["ApiResponse_Vec_Alert"]["data"];
type MonitoringAlertData = components["schemas"]["ApiResponse_Alert"]["data"];
type MonitoringIncidentsData =
	components["schemas"]["ApiResponse_Vec_Incident"]["data"];
type MonitoringIncidentData =
	components["schemas"]["ApiResponse_Incident"]["data"];
type MonitoringStatsData =
	components["schemas"]["ApiResponse_MonitoringStats"]["data"];
type IncidentTimelineData =
	components["schemas"]["ApiResponse_IncidentTimeline"]["data"];

// Standard refetch intervals
const REFETCH_INTERVALS = {
	FAST: 5000, // 5 seconds - for real-time components
	NORMAL: 15000, // 15 seconds - for regular updates
	SLOW: 30000, // 30 seconds - for less critical data
} as const;

/**
 * Health Queries - All return consistent data shapes
 */

// Basic health with extracted data (most common usage)
export function useHealthBasic(
	refetchInterval?: number,
): UseQueryResult<HealthData> {
	return useQuery({
		queryKey: ["health", "basic"],
		queryFn: async () => {
			const response = await apiClient.getHealth();
			if (!response.data) {
				throw new Error("No health data received");
			}
			return response.data;
		},
		refetchInterval: refetchInterval ?? REFETCH_INTERVALS.NORMAL,
	});
}

// Detailed health with extracted data
export function useHealthDetailed(
	refetchInterval?: number,
): UseQueryResult<DetailedHealthData> {
	return useQuery({
		queryKey: ["health", "detailed"],
		queryFn: async () => {
			const response = await apiClient.getDetailedHealth();
			if (!response.data) {
				throw new Error("No detailed health data received");
			}
			return response.data;
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
export function useTaskStats(
	refetchInterval?: number,
): UseQueryResult<TaskStatsData> {
	return useQuery({
		queryKey: ["tasks", "stats"],
		queryFn: async () => {
			const response = await apiClient.getTaskStats();
			if (!response.data) {
				throw new Error("No task stats data received");
			}
			return response.data;
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
			if (!response.data) {
				throw new Error("No user data received");
			}
			return response.data;
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
		list: (filters?: Record<string, string>) =>
			["tasks", "list", filters] as const,
		detail: (id: string) => ["tasks", "detail", id] as const,
		types: ["tasks", "types"] as const,
		deadLetter: (filters?: Record<string, string>) =>
			["tasks", "deadLetter", filters] as const,
	},
	users: {
		me: ["auth", "me"] as const,
		list: (filters?: Record<string, string>) =>
			["users", "list", filters] as const,
		detail: (id: string) => ["users", "detail", id] as const,
		stats: ["users", "stats"] as const,
	},
	monitoring: {
		events: (params?: Record<string, unknown>) =>
			["monitoring", "events", params] as const,
		event: (id: string) => ["monitoring", "events", id] as const,
		metrics: (params?: Record<string, unknown>) =>
			["monitoring", "metrics", params] as const,
		prometheus: ["monitoring", "prometheus"] as const,
		alerts: ["monitoring", "alerts"] as const,
		alert: (id: string) => ["monitoring", "alerts", id] as const,
		incidents: (params?: Record<string, unknown>) =>
			["monitoring", "incidents", params] as const,
		incident: (id: string) => ["monitoring", "incidents", id] as const,
		incidentTimeline: (id: string, params?: Record<string, unknown>) =>
			["monitoring", "incidents", id, "timeline", params] as const,
		stats: ["monitoring", "stats"] as const,
	},
} as const;

/**
 * Monitoring Queries
 */

// Events
export function useMonitoringEvents(
	params?: {
		event_type?: "log" | "metric" | "trace" | "alert";
		source?: string;
		level?: "error" | "warn" | "info" | "debug";
		tags?: string;
		start_time?: string;
		end_time?: string;
		limit?: number;
		offset?: number;
	},
	refetchInterval?: number,
): UseQueryResult<MonitoringEventsData> {
	return useQuery({
		queryKey: ["monitoring", "events", params],
		queryFn: async () => {
			const response = await apiClient.getEvents(params);
			return response.data;
		},
		refetchInterval: refetchInterval ?? REFETCH_INTERVALS.NORMAL,
	});
}

export function useMonitoringEvent(
	id: string,
): UseQueryResult<MonitoringEventData> {
	return useQuery({
		queryKey: ["monitoring", "events", id],
		queryFn: async () => {
			const response = await apiClient.getEvent(id);
			return response.data;
		},
		enabled: !!id,
	});
}

// Metrics
export function useMonitoringMetrics(
	params?: {
		name?: string;
		metric_type?: "counter" | "gauge" | "histogram" | "summary";
		start_time?: string;
		end_time?: string;
		limit?: number;
		offset?: number;
	},
	refetchInterval?: number,
): UseQueryResult<MonitoringMetricsData> {
	return useQuery({
		queryKey: ["monitoring", "metrics", params],
		queryFn: async () => {
			const response = await apiClient.getMetrics(params);
			return response.data;
		},
		refetchInterval: refetchInterval ?? REFETCH_INTERVALS.NORMAL,
	});
}

export function usePrometheusMetrics(refetchInterval?: number) {
	return useQuery({
		queryKey: ["monitoring", "prometheus"],
		queryFn: () => apiClient.getPrometheusMetrics(),
		refetchInterval: refetchInterval ?? REFETCH_INTERVALS.SLOW,
	});
}

// Alerts (Moderator+ only)
export function useMonitoringAlerts(
	refetchInterval?: number,
): UseQueryResult<MonitoringAlertsData> {
	return useQuery({
		queryKey: ["monitoring", "alerts"],
		queryFn: async () => {
			const response = await apiClient.getAlerts();
			return response.data;
		},
		refetchInterval: refetchInterval ?? REFETCH_INTERVALS.NORMAL,
	});
}

export function useMonitoringAlert(
	id: string,
): UseQueryResult<MonitoringAlertData> {
	return useQuery({
		queryKey: ["monitoring", "alerts", id],
		queryFn: async () => {
			const response = await apiClient.getAlert(id);
			return response.data;
		},
		enabled: !!id,
	});
}

// Incidents
export function useMonitoringIncidents(
	params?: {
		limit?: number;
		offset?: number;
	},
	refetchInterval?: number,
): UseQueryResult<MonitoringIncidentsData> {
	return useQuery({
		queryKey: ["monitoring", "incidents", params],
		queryFn: async () => {
			const response = await apiClient.getIncidents(params);
			return response.data;
		},
		refetchInterval: refetchInterval ?? REFETCH_INTERVALS.NORMAL,
	});
}

export function useMonitoringIncident(
	id: string,
): UseQueryResult<MonitoringIncidentData> {
	return useQuery({
		queryKey: ["monitoring", "incidents", id],
		queryFn: async () => {
			const response = await apiClient.getIncident(id);
			return response.data;
		},
		enabled: !!id,
	});
}

export function useMonitoringIncidentTimeline(
	id: string,
	params?: {
		limit?: number;
		offset?: number;
	},
): UseQueryResult<IncidentTimelineData> {
	return useQuery({
		queryKey: ["monitoring", "incidents", id, "timeline", params],
		queryFn: async () => {
			const response = await apiClient.getIncidentTimeline(id, params);
			return response.data;
		},
		enabled: !!id,
	});
}

// System Statistics (Moderator+ only)
export function useMonitoringStats(
	refetchInterval?: number,
): UseQueryResult<MonitoringStatsData> {
	return useQuery({
		queryKey: ["monitoring", "stats"],
		queryFn: async () => {
			const response = await apiClient.getMonitoringStats();
			return response.data;
		},
		refetchInterval: refetchInterval ?? REFETCH_INTERVALS.NORMAL,
	});
}

/**
 * Type-safe query key utilities
 */
export type QueryKey = readonly string[];
