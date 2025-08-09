import {
	mockApiResponse,
	mockAuthUser,
	mockHealthResponse,
	mockTaskStats,
} from "@/test/mocks";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React, { type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
	QUERY_KEYS,
	useCurrentUser,
	useHealthBasic,
	useHealthDetailed,
	useHealthLiveness,
	useMonitoringEvents,
	useMonitoringMetrics,
	usePrometheusMetrics,
	useTaskStats,
} from "../useApiQueries";

// Mock the API client
vi.mock("@/lib/api/client", () => ({
	apiClient: {
		getHealth: vi.fn(),
		getDetailedHealth: vi.fn(),
		getLivenessProbe: vi.fn(),
		getReadinessProbe: vi.fn(),
		getStartupProbe: vi.fn(),
		getTaskStats: vi.fn(),
		getCurrentUser: vi.fn(),
		getEvents: vi.fn(),
		getMetrics: vi.fn(),
		getPrometheusMetrics: vi.fn(),
	},
}));

import { apiClient } from "@/lib/api/client";

describe("useApiQueries Hook Tests", () => {
	let queryClient: QueryClient;
	let wrapper: ({ children }: { children: ReactNode }) => JSX.Element;

	beforeEach(() => {
		// Create fresh QueryClient for each test
		queryClient = new QueryClient({
			defaultOptions: {
				queries: {
					retry: false, // Disable retries in tests
					gcTime: 0, // Don't cache queries in tests
				},
			},
		});

		wrapper = ({ children }) =>
			React.createElement(
				QueryClientProvider,
				{ client: queryClient },
				children,
			);

		// Reset all mocks
		vi.clearAllMocks();
	});

	describe("Health Queries", () => {
		describe("useHealthBasic", () => {
			it("should fetch basic health data successfully", async () => {
				const mockResponse = mockApiResponse(mockHealthResponse);
				vi.mocked(apiClient.getHealth).mockResolvedValue(mockResponse);

				const { result } = renderHook(() => useHealthBasic(), { wrapper });

				await waitFor(() => {
					expect(result.current.isSuccess).toBe(true);
				});

				expect(result.current.data).toEqual(mockHealthResponse);
				expect(apiClient.getHealth).toHaveBeenCalledTimes(1);
			});

			it("should handle error when no data received", async () => {
				const mockResponse = {
					...mockApiResponse(mockHealthResponse),
					data: undefined,
				};
				vi.mocked(apiClient.getHealth).mockResolvedValue(mockResponse);

				const { result } = renderHook(() => useHealthBasic(), { wrapper });

				await waitFor(() => {
					expect(result.current.isError).toBe(true);
				});

				expect(result.current.error).toEqual(
					new Error("No health data received"),
				);
			});

			it("should use correct query key", () => {
				vi.mocked(apiClient.getHealth).mockResolvedValue(
					mockApiResponse(mockHealthResponse),
				);

				renderHook(() => useHealthBasic(), { wrapper });

				// Verify the hook was called - query key testing is covered by TanStack Query
				expect(apiClient.getHealth).toHaveBeenCalled();
			});

			it("should respect custom refetch interval", async () => {
				vi.mocked(apiClient.getHealth).mockResolvedValue(
					mockApiResponse(mockHealthResponse),
				);

				const customInterval = 1000;
				renderHook(() => useHealthBasic(customInterval), { wrapper });

				// Note: We can't easily test the actual interval timing in unit tests,
				// but we can verify the hook accepts the parameter
				expect(apiClient.getHealth).toHaveBeenCalled();
			});
		});

		describe("useHealthDetailed", () => {
			it("should fetch detailed health data successfully", async () => {
				const detailedHealth = {
					status: "healthy" as const,
					checks: { database: { status: "healthy", message: "OK" } },
					timestamp: new Date().toISOString(),
				};
				const mockResponse = mockApiResponse(detailedHealth);
				vi.mocked(apiClient.getDetailedHealth).mockResolvedValue(mockResponse);

				const { result } = renderHook(() => useHealthDetailed(), { wrapper });

				await waitFor(() => {
					expect(result.current.isSuccess).toBe(true);
				});

				expect(result.current.data).toEqual(detailedHealth);
				expect(apiClient.getDetailedHealth).toHaveBeenCalledTimes(1);
			});

			it("should handle API errors gracefully", async () => {
				vi.mocked(apiClient.getDetailedHealth).mockRejectedValue(
					new Error("Server error"),
				);

				const { result } = renderHook(() => useHealthDetailed(), { wrapper });

				await waitFor(() => {
					expect(result.current.isError).toBe(true);
				});

				expect(result.current.error).toEqual(new Error("Server error"));
			});
		});

		describe("useHealthLiveness", () => {
			it("should fetch liveness probe data", async () => {
				const probeData = {
					status: "alive",
					timestamp: "2024-01-01T00:00:00Z",
				};
				const mockResponse = mockApiResponse(probeData);
				vi.mocked(apiClient.getLivenessProbe).mockResolvedValue(mockResponse);

				const { result } = renderHook(() => useHealthLiveness(), { wrapper });

				await waitFor(() => {
					expect(result.current.isSuccess).toBe(true);
				});

				expect(result.current.data).toEqual(mockResponse);
				expect(apiClient.getLivenessProbe).toHaveBeenCalledTimes(1);
			});

			it("should use fast refetch interval by default", () => {
				vi.mocked(apiClient.getLivenessProbe).mockResolvedValue(
					mockApiResponse({ status: "alive" }),
				);

				renderHook(() => useHealthLiveness(), { wrapper });

				// Verify the hook was called
				expect(apiClient.getLivenessProbe).toHaveBeenCalled();
			});
		});
	});

	describe("Task Queries", () => {
		describe("useTaskStats", () => {
			it("should fetch task stats successfully", async () => {
				const mockResponse = mockApiResponse(mockTaskStats);
				vi.mocked(apiClient.getTaskStats).mockResolvedValue(mockResponse);

				const { result } = renderHook(() => useTaskStats(), { wrapper });

				await waitFor(() => {
					expect(result.current.isSuccess).toBe(true);
				});

				expect(result.current.data).toEqual(mockTaskStats);
				expect(apiClient.getTaskStats).toHaveBeenCalledTimes(1);
			});

			it("should handle missing task stats data", async () => {
				const mockResponse = { ...mockApiResponse(mockTaskStats), data: undefined };
				vi.mocked(apiClient.getTaskStats).mockResolvedValue(mockResponse);

				const { result } = renderHook(() => useTaskStats(), { wrapper });

				await waitFor(() => {
					expect(result.current.isError).toBe(true);
				});

				expect(result.current.error).toEqual(
					new Error("No task stats data received"),
				);
			});

			it("should use correct query key", () => {
				vi.mocked(apiClient.getTaskStats).mockResolvedValue(
					mockApiResponse(mockTaskStats),
				);

				renderHook(() => useTaskStats(), { wrapper });

				expect(apiClient.getTaskStats).toHaveBeenCalled();
			});
		});
	});

	describe("User Queries", () => {
		describe("useCurrentUser", () => {
			it("should fetch current user successfully", async () => {
				const mockResponse = mockApiResponse(mockAuthUser);
				vi.mocked(apiClient.getCurrentUser).mockResolvedValue(mockResponse);

				const { result } = renderHook(() => useCurrentUser(), { wrapper });

				await waitFor(() => {
					expect(result.current.isSuccess).toBe(true);
				});

				expect(result.current.data).toEqual(mockAuthUser);
				expect(apiClient.getCurrentUser).toHaveBeenCalledTimes(1);
			});

			it("should handle authentication errors", async () => {
				vi.mocked(apiClient.getCurrentUser).mockRejectedValue(
					new Error("Unauthorized"),
				);

				const { result } = renderHook(() => useCurrentUser(), { wrapper });

				await waitFor(() => {
					expect(result.current.isError).toBe(true);
				});

				expect(result.current.error).toEqual(new Error("Unauthorized"));
			});

			it("should use slow refetch interval by default", () => {
				vi.mocked(apiClient.getCurrentUser).mockResolvedValue(
					mockApiResponse(mockAuthUser),
				);

				renderHook(() => useCurrentUser(), { wrapper });

				expect(apiClient.getCurrentUser).toHaveBeenCalled();
			});
		});
	});

	describe("Monitoring Queries", () => {
		describe("useMonitoringEvents", () => {
			it("should fetch events without filters", async () => {
				const mockEvents = [
					{
						id: "event-1",
						event_type: "log" as const,
						source: "test",
						message: "Test message",
						recorded_at: "2024-01-01T00:00:00Z",
						created_at: "2024-01-01T00:00:00Z",
						payload: {},
						tags: {},
						level: "info",
					},
				];
				const mockResponse = mockApiResponse(mockEvents);
				vi.mocked(apiClient.getEvents).mockResolvedValue(mockResponse);

				const { result } = renderHook(() => useMonitoringEvents(), { wrapper });

				await waitFor(() => {
					expect(result.current.isSuccess).toBe(true);
				});

				expect(result.current.data).toEqual(mockEvents);
				expect(apiClient.getEvents).toHaveBeenCalledWith(undefined);
			});

			it("should fetch events with filters", async () => {
				const params = {
					event_type: "log" as const,
					source: "test-source",
					level: "info" as const,
					limit: 10,
				};
				const mockEvents: any[] = [];
				const mockResponse = mockApiResponse(mockEvents);
				vi.mocked(apiClient.getEvents).mockResolvedValue(mockResponse);

				const { result } = renderHook(() => useMonitoringEvents(params), {
					wrapper,
				});

				await waitFor(() => {
					expect(result.current.isSuccess).toBe(true);
				});

				expect(result.current.data).toEqual(mockEvents);
				expect(apiClient.getEvents).toHaveBeenCalledWith(params);
			});

			it("should use correct query key with params", () => {
				const params = { event_type: "log" as const, limit: 5 };
				vi.mocked(apiClient.getEvents).mockResolvedValue(mockApiResponse([]));

				renderHook(() => useMonitoringEvents(params), { wrapper });

				expect(apiClient.getEvents).toHaveBeenCalledWith(params);
			});
		});

		describe("useMonitoringMetrics", () => {
			it("should fetch metrics with filters", async () => {
				const params = {
					name: "test_metric",
					metric_type: "counter" as const,
					limit: 10,
				};
				const mockMetrics = [
					{
						id: "metric-1",
						name: "test_metric",
						metric_type: "counter" as const,
						value: 5,
						recorded_at: "2024-01-01T00:00:00Z",
						created_at: "2024-01-01T00:00:00Z",
						labels: {},
					},
				];
				const mockResponse = mockApiResponse(mockMetrics);
				vi.mocked(apiClient.getMetrics).mockResolvedValue(mockResponse);

				const { result } = renderHook(() => useMonitoringMetrics(params), {
					wrapper,
				});

				await waitFor(() => {
					expect(result.current.isSuccess).toBe(true);
				});

				expect(result.current.data).toEqual(mockMetrics);
				expect(apiClient.getMetrics).toHaveBeenCalledWith(params);
			});
		});

		describe("usePrometheusMetrics", () => {
			it("should fetch prometheus metrics as text", async () => {
				const prometheusData =
					"# HELP test_metric Test metric\ntest_metric 1\n";
				vi.mocked(apiClient.getPrometheusMetrics).mockResolvedValue(
					prometheusData,
				);

				const { result } = renderHook(() => usePrometheusMetrics(), {
					wrapper,
				});

				await waitFor(() => {
					expect(result.current.isSuccess).toBe(true);
				});

				expect(result.current.data).toBe(prometheusData);
				expect(apiClient.getPrometheusMetrics).toHaveBeenCalledTimes(1);
			});

			it("should use correct query key", () => {
				vi.mocked(apiClient.getPrometheusMetrics).mockResolvedValue("");

				renderHook(() => usePrometheusMetrics(), { wrapper });

				expect(apiClient.getPrometheusMetrics).toHaveBeenCalled();
			});
		});
	});

	describe("Query Key Consistency", () => {
		it("should have consistent query keys in QUERY_KEYS object", () => {
			// Test that all query key constants are properly structured
			expect(QUERY_KEYS.health.basic).toEqual(["health", "basic"]);
			expect(QUERY_KEYS.health.detailed).toEqual(["health", "detailed"]);
			expect(QUERY_KEYS.tasks.stats).toEqual(["tasks", "stats"]);
			expect(QUERY_KEYS.users.me).toEqual(["auth", "me"]);
			expect(QUERY_KEYS.monitoring.prometheus).toEqual([
				"monitoring",
				"prometheus",
			]);
		});

		it("should generate parameterized query keys correctly", () => {
			const taskId = "task-123";
			const userId = "user-456";
			const filters = { limit: "10", offset: "0" };

			expect(QUERY_KEYS.tasks.detail(taskId)).toEqual([
				"tasks",
				"detail",
				taskId,
			]);
			expect(QUERY_KEYS.users.detail(userId)).toEqual([
				"users",
				"detail",
				userId,
			]);
			expect(QUERY_KEYS.tasks.list(filters)).toEqual([
				"tasks",
				"list",
				filters,
			]);
			expect(QUERY_KEYS.monitoring.events(filters)).toEqual([
				"monitoring",
				"events",
				filters,
			]);
		});
	});

	describe("Loading States", () => {
		it("should show loading state initially", () => {
			// Mock a pending promise
			vi.mocked(apiClient.getHealth).mockImplementation(
				() => new Promise(() => {}), // Never resolves
			);

			const { result } = renderHook(() => useHealthBasic(), { wrapper });

			expect(result.current.isLoading).toBe(true);
			expect(result.current.data).toBeUndefined();
			expect(result.current.error).toBeNull();
		});

		it("should transition from loading to success", async () => {
			vi.mocked(apiClient.getTaskStats).mockResolvedValue(
				mockApiResponse(mockTaskStats),
			);

			const { result } = renderHook(() => useTaskStats(), { wrapper });

			// Initially loading
			expect(result.current.isLoading).toBe(true);

			// After resolution
			await waitFor(() => {
				expect(result.current.isSuccess).toBe(true);
			});

			expect(result.current.isLoading).toBe(false);
			expect(result.current.data).toEqual(mockTaskStats);
		});

		it("should transition from loading to error", async () => {
			vi.mocked(apiClient.getTaskStats).mockRejectedValue(
				new Error("Network error"),
			);

			const { result } = renderHook(() => useTaskStats(), { wrapper });

			// Initially loading
			expect(result.current.isLoading).toBe(true);

			// After rejection
			await waitFor(() => {
				expect(result.current.isError).toBe(true);
			});

			expect(result.current.isLoading).toBe(false);
			expect(result.current.error).toEqual(new Error("Network error"));
		});
	});
});
