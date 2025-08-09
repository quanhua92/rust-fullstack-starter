import { apiClient, getAuthToken, setAuthToken } from "@/lib/api/client";
import {
	describeIntegration,
	setupIntegrationTest,
} from "@/test/integration-setup";
import { createTestUser } from "@/test/mocks";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React, { type ReactNode } from "react";
import { afterEach, beforeAll, describe, expect, it } from "vitest";
import {
	useCurrentUser,
	useHealthBasic,
	useHealthDetailed,
	useMonitoringEvents,
	useMonitoringMetrics,
	useTaskStats,
} from "../useApiQueries";

describeIntegration("useApiQueries Hook Integration Tests", () => {
	let queryClient: QueryClient;
	let wrapper: ({ children }: { children: ReactNode }) => JSX.Element;
	const { baseUrl } = setupIntegrationTest();

	// Keep track of created resources for cleanup
	const createdTasks: string[] = [];

	beforeAll(() => {
		// Replace the default apiClient with one pointing to test server
		Object.assign(
			apiClient,
			new (
				apiClient.constructor as unknown as new (
					baseUrl: string,
				) => typeof apiClient
			)(baseUrl),
		);
	});

	beforeEach(() => {
		// Create fresh QueryClient for each test
		queryClient = new QueryClient({
			defaultOptions: {
				queries: {
					retry: 1, // Allow one retry for integration tests
					gcTime: 1000 * 60, // Keep cache for 1 minute
					staleTime: 1000 * 30, // Consider data stale after 30 seconds
				},
			},
		});

		wrapper = ({ children }) =>
			React.createElement(
				QueryClientProvider,
				{ client: queryClient },
				children,
			);
	});

	afterEach(async () => {
		// Clean up created resources
		for (const taskId of createdTasks) {
			try {
				await apiClient.deleteTask(taskId);
			} catch (error) {
				console.warn(`Failed to cleanup task ${taskId}:`, error);
			}
		}
		createdTasks.length = 0;

		// Clear auth token
		setAuthToken(null);

		// Note: No need to clear query cache since we create fresh QueryClient for each test
	});

	describe("Health Hooks Integration", () => {
		it("should fetch real health data with useHealthBasic", async () => {
			const { result } = renderHook(() => useHealthBasic(), { wrapper });

			await waitFor(
				() => {
					expect(result.current.isSuccess).toBe(true);
				},
				{ timeout: 5000 },
			);

			expect(result.current.data?.status).toBe("healthy");
			// Basic health doesn't have timestamp/checks - only detailed health does
			expect(result.current.data?.version).toBeDefined();
			expect(result.current.error).toBeNull();
		});

		it("should fetch detailed health data", async () => {
			const { result } = renderHook(() => useHealthDetailed(), { wrapper });

			await waitFor(
				() => {
					expect(result.current.isSuccess).toBe(true);
				},
				{ timeout: 5000 },
			);

			expect(result.current.data?.status).toBe("healthy");
			expect(result.current.data?.checks).toBeDefined();
		});

		it("should handle health endpoint errors gracefully", async () => {
			// Store original client for restoration
			const originalClient = { ...apiClient };
			
			try {
				// Create a client with wrong base URL to trigger error
				const badClient = new (
					apiClient.constructor as unknown as new (
						baseUrl: string,
					) => typeof apiClient
				)("http://localhost:9999/api/v1");
				Object.assign(apiClient, badClient);

				const { result } = renderHook(() => useHealthBasic(), { wrapper });

				await waitFor(
					() => {
						expect(result.current.isError).toBe(true);
					},
					{ timeout: 10000 },
				);

				expect(result.current.error).toBeDefined();
				expect(result.current.data).toBeUndefined();
			} finally {
				// Always restore correct client
				Object.assign(
					apiClient,
					new (
						apiClient.constructor as unknown as new (
							baseUrl: string,
						) => typeof apiClient
					)(baseUrl),
				);
			}
		});

		it("should refetch health data when manually triggered", async () => {
			const { result } = renderHook(() => useHealthBasic(), { wrapper });

			// Wait for initial load
			await waitFor(() => {
				expect(result.current.isSuccess).toBe(true);
			});

			const firstUptime = result.current.data?.uptime;

			// Manually refetch
			await result.current.refetch();

			await waitFor(() => {
				expect(result.current.data?.version).toBeDefined();
			});

			// Should get fresh data (uptime should be different, version same)
			expect(result.current.data?.uptime).toBeGreaterThan(firstUptime || 0);
		});
	});

	describe("Task Stats Integration", () => {
		let authToken: string;

		beforeAll(async () => {
			// Create and authenticate a user for task tests
			const testUser = createTestUser();
			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			authToken = loginResponse.data?.session_token || "";
			setAuthToken(authToken);
		});

		it("should fetch real task stats", async () => {
			const { result } = renderHook(() => useTaskStats(), { wrapper });

			await waitFor(
				() => {
					expect(result.current.isSuccess).toBe(true);
				},
				{ timeout: 5000 },
			);

			expect(result.current.data).toBeDefined();
			expect(typeof result.current.data?.total).toBe("number");
			expect(typeof result.current.data?.pending).toBe("number");
			expect(typeof result.current.data?.running).toBe("number");
			expect(typeof result.current.data?.completed).toBe("number");
			expect(typeof result.current.data?.failed).toBe("number");
			expect(typeof result.current.data?.cancelled).toBe("number");
		});

		it("should update stats when new tasks are created", async () => {
			const { result } = renderHook(() => useTaskStats(), { wrapper });

			// Wait for initial stats
			await waitFor(() => {
				expect(result.current.isSuccess).toBe(true);
			});

			const initialStats = result.current.data;
			const initialTotal = initialStats?.total || 0;

			// Create a new task
			const createResponse = await apiClient.createTask({
				task_type: "email",
				payload: { to: "test@example.com", subject: "Test" },
				priority: "normal",
			});
			if (createResponse.data?.id) createdTasks.push(createResponse.data.id);

			// Refetch stats
			await result.current.refetch();

			await waitFor(() => {
				expect(result.current.data?.total).toBeGreaterThan(initialTotal);
			});

			expect(result.current.data?.pending).toBeGreaterThanOrEqual(1);
		});

		it("should handle auth errors for task stats", async () => {
			const originalToken = getAuthToken();
			setAuthToken(null);

			const { result } = renderHook(() => useTaskStats(), { wrapper });

			await waitFor(
				() => {
					expect(result.current.isError).toBe(true);
				},
				{ timeout: 5000 },
			);

			expect(result.current.error).toBeDefined();

			// Restore token
			setAuthToken(originalToken);
		});
	});

	describe("Current User Integration", () => {
		let testUser: ReturnType<typeof createTestUser>;

		beforeAll(async () => {
			testUser = createTestUser();
			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			setAuthToken(loginResponse.data?.session_token || null);
		});

		it("should fetch current user data", async () => {
			const { result } = renderHook(() => useCurrentUser(), { wrapper });

			await waitFor(
				() => {
					expect(result.current.isSuccess).toBe(true);
				},
				{ timeout: 5000 },
			);

			expect(result.current.data).toBeDefined();
			expect(result.current.data?.email).toBe(testUser.email);
			expect(result.current.data?.username).toBe(testUser.username);
			expect(result.current.data?.role).toBe("user");
		});

		it("should update user data after profile change", async () => {
			const { result } = renderHook(() => useCurrentUser(), { wrapper });

			// Wait for initial load
			await waitFor(() => {
				expect(result.current.isSuccess).toBe(true);
			});

			const originalUsername = result.current.data?.username;

			// Update profile
			const newUsername = `updated_${originalUsername}`;
			await apiClient.updateOwnProfile({ username: newUsername });

			// Refetch user data
			await result.current.refetch();

			await waitFor(() => {
				expect(result.current.data?.username).toBe(newUsername);
			});
		});

		it("should handle unauthenticated requests", async () => {
			setAuthToken(null);

			const { result } = renderHook(() => useCurrentUser(), { wrapper });

			await waitFor(
				() => {
					expect(result.current.isError).toBe(true);
				},
				{ timeout: 5000 },
			);

			expect(result.current.error).toBeDefined();
		});
	});

	describe("Monitoring Events Integration", () => {
		let authToken: string;

		beforeAll(async () => {
			// Create and authenticate a user for monitoring tests
			const testUser = createTestUser();
			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			authToken = loginResponse.data?.session_token || "";
			setAuthToken(authToken);
		});

		it("should fetch monitoring events", async () => {
			// Create a test event first
			await apiClient.createEvent({
				event_type: "log",
				source: "hook-integration-test",
				message: "Test event for hook integration",
				level: "info",
			});

			const { result } = renderHook(
				() =>
					useMonitoringEvents({
						source: "hook-integration-test",
						limit: 10,
					}),
				{ wrapper },
			);

			await waitFor(
				() => {
					expect(result.current.isSuccess).toBe(true);
				},
				{ timeout: 5000 },
			);

			expect(Array.isArray(result.current.data)).toBe(true);
			expect(result.current.data?.length).toBeGreaterThan(0);

			const event = result.current.data?.[0];
			expect(event?.source).toBe("hook-integration-test");
		});

		it("should filter events correctly", async () => {
			// Create events with different levels
			await Promise.all([
				apiClient.createEvent({
					event_type: "log",
					source: "hook-filter-test",
					message: "Error event",
					level: "error",
				}),
				apiClient.createEvent({
					event_type: "log",
					source: "hook-filter-test",
					message: "Info event",
					level: "info",
				}),
			]);

			const { result } = renderHook(
				() =>
					useMonitoringEvents({
						source: "hook-filter-test",
						level: "error",
						limit: 5,
					}),
				{ wrapper },
			);

			await waitFor(
				() => {
					expect(result.current.isSuccess).toBe(true);
				},
				{ timeout: 5000 },
			);

			expect(Array.isArray(result.current.data)).toBe(true);

			// All events should be error level
			if (result.current.data) {
				for (const event of result.current.data) {
					expect(event.level).toBe("error");
				}
			}
		});
	});

	describe("Monitoring Metrics Integration", () => {
		let authToken: string;

		beforeAll(async () => {
			// Create and authenticate a user
			const testUser = createTestUser();
			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			authToken = loginResponse.data?.session_token || "";
			setAuthToken(authToken);
		});

		it("should fetch monitoring metrics", async () => {
			// Create a test metric first
			await apiClient.createMetric({
				name: "hook_integration_test_metric",
				metric_type: "gauge",
				value: 42,
				labels: { test: "hook-integration" },
			});

			const { result } = renderHook(
				() =>
					useMonitoringMetrics({
						name: "hook_integration_test_metric",
						limit: 10,
					}),
				{ wrapper },
			);

			await waitFor(
				() => {
					expect(result.current.isSuccess).toBe(true);
				},
				{ timeout: 5000 },
			);

			expect(Array.isArray(result.current.data)).toBe(true);
			expect(result.current.data?.length).toBeGreaterThan(0);

			const metric = result.current.data?.find(
				(m) => m.name === "hook_integration_test_metric",
			);
			expect(metric).toBeDefined();
			expect(metric?.value).toBe(42);
		});

		it("should filter metrics by type", async () => {
			// Create metrics with different types
			await Promise.all([
				apiClient.createMetric({
					name: "hook_counter_metric",
					metric_type: "counter",
					value: 10,
				}),
				apiClient.createMetric({
					name: "hook_gauge_metric",
					metric_type: "gauge",
					value: 50,
				}),
			]);

			const { result } = renderHook(
				() =>
					useMonitoringMetrics({
						metric_type: "counter",
						limit: 10,
					}),
				{ wrapper },
			);

			await waitFor(
				() => {
					expect(result.current.isSuccess).toBe(true);
				},
				{ timeout: 5000 },
			);

			expect(Array.isArray(result.current.data)).toBe(true);

			// All metrics should be counter type
			if (result.current.data) {
				for (const metric of result.current.data) {
					expect(metric.metric_type).toBe("counter");
				}
			}
		});
	});

	describe("Cache Behavior Integration", () => {
		let authToken: string;

		beforeAll(async () => {
			const testUser = createTestUser();
			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			authToken = loginResponse.data?.session_token || "";
			setAuthToken(authToken);
		});

		it("should cache health data between renders", async () => {
			// First hook instance
			const { result: result1, unmount: unmount1 } = renderHook(
				() => useHealthBasic(),
				{ wrapper },
			);

			await waitFor(() => {
				expect(result1.current.isSuccess).toBe(true);
			});

			const firstUptime = result1.current.data?.uptime;
			unmount1();

			// Second hook instance should use cached data initially
			const { result: result2 } = renderHook(() => useHealthBasic(), {
				wrapper,
			});

			// Should have cached data immediately
			expect(result2.current.data?.uptime).toBe(firstUptime);
		});

		it("should invalidate cache and refetch after mutations", async () => {
			// Get initial task stats
			const { result } = renderHook(() => useTaskStats(), { wrapper });

			await waitFor(() => {
				expect(result.current.isSuccess).toBe(true);
			});

			const initialTotal = result.current.data?.total || 0;

			// Create a task (mutation)
			const createResponse = await apiClient.createTask({
				task_type: "email",
				payload: { to: "test@example.com", subject: "Cache test" },
				priority: "normal",
			});
			if (createResponse.data?.id) createdTasks.push(createResponse.data.id);

			// Manually invalidate cache to simulate what a mutation would do
			queryClient.invalidateQueries({ queryKey: ["tasks", "stats"] });

			await waitFor(
				() => {
					expect(result.current.data?.total).toBeGreaterThan(initialTotal);
				},
				{ timeout: 5000 },
			);
		});
	});

	describe("Loading States Integration", () => {
		it("should show proper loading states during real requests", async () => {
			const { result } = renderHook(() => useHealthBasic(), { wrapper });

			// Initially should be loading
			expect(result.current.isLoading).toBe(true);
			expect(result.current.data).toBeUndefined();
			expect(result.current.error).toBeNull();

			// Wait for completion
			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			expect(result.current.isSuccess).toBe(true);
			expect(result.current.data).toBeDefined();
		});

		it("should handle slow network conditions", async () => {
			const { result } = renderHook(
				() => useMonitoringEvents({ limit: 100 }), // Large limit to simulate slow query
				{ wrapper },
			);

			// Should handle loading state properly
			expect(result.current.isLoading).toBe(true);

			await waitFor(
				() => {
					expect(result.current.isLoading).toBe(false);
				},
				{ timeout: 10000 },
			); // Longer timeout for slow queries

			// Should complete successfully
			expect(result.current.isSuccess || result.current.isError).toBe(true);
		});
	});

	describe("Error Recovery Integration", () => {
		let authToken: string;

		beforeAll(async () => {
			const testUser = createTestUser();
			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			authToken = loginResponse.data?.session_token || "";
			setAuthToken(authToken);
		});

		it("should retry failed requests automatically", async () => {
			// Store original baseUrl for restoration
			const originalBaseUrl = (apiClient as unknown as { baseUrl: string })
				.baseUrl;

			try {
				// First, break the connection
				(apiClient as unknown as { baseUrl: string }).baseUrl =
					"http://localhost:9999/api/v1";

				const { result } = renderHook(() => useHealthBasic(), { wrapper });

				// Should eventually error out after retries
				await waitFor(
					() => {
						expect(result.current.isError).toBe(true);
					},
					{ timeout: 10000 },
				);

				// Restore connection
				(apiClient as unknown as { baseUrl: string }).baseUrl = originalBaseUrl;

				// Manually trigger refetch
				await result.current.refetch();

				// Should recover
				await waitFor(
					() => {
						expect(result.current.isSuccess).toBe(true);
					},
					{ timeout: 5000 },
				);
			} finally {
				// Always restore original baseUrl
				(apiClient as unknown as { baseUrl: string }).baseUrl = originalBaseUrl;
			}
		});
	});
});
