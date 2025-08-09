import {
	createMockFetch,
	createMockResponse,
	mockApiError,
	mockApiResponse,
	mockAuthUser,
	mockHealthResponse,
	mockTask,
	mockTaskStats,
	mockUserProfile,
} from "@/test/mocks";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ApiClient, getAuthToken, setAuthToken } from "../client";

describe("ApiClient Unit Tests", () => {
	let apiClient: ApiClient;
	let mockFetch: ReturnType<typeof createMockFetch>;

	beforeEach(() => {
		// Create fresh instance and mock fetch
		apiClient = new ApiClient();
		mockFetch = createMockFetch();

		// Clear auth token
		setAuthToken(null);
	});

	describe("Authentication Token Management", () => {
		it("should set and get auth token", () => {
			const token = "test-token-123";
			setAuthToken(token);
			expect(getAuthToken()).toBe(token);
		});

		it("should clear auth token", () => {
			setAuthToken("test-token");
			setAuthToken(null);
			expect(getAuthToken()).toBeNull();
		});

		it("should include Bearer token in requests when set", async () => {
			const token = "test-token-123";
			setAuthToken(token);

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockHealthResponse)),
			);

			await apiClient.getHealth();

			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/health",
				expect.objectContaining({
					headers: expect.objectContaining({
						Authorization: `Bearer ${token}`,
					}),
				}),
			);
		});

		it("should not include Authorization header when no token", async () => {
			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockHealthResponse)),
			);

			await apiClient.getHealth();

			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/health",
				expect.objectContaining({
					headers: expect.not.objectContaining({
						Authorization: expect.any(String),
					}),
				}),
			);
		});
	});

	describe("Request Construction", () => {
		it("should construct GET requests correctly", async () => {
			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockHealthResponse)),
			);

			await apiClient.getHealth();

			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/health",
				expect.objectContaining({
					headers: expect.objectContaining({
						"Content-Type": "application/json",
					}),
				}),
			);
		});

		it("should construct POST requests with body", async () => {
			const registerData = {
				username: "testuser",
				email: "test@example.com",
				password: "password123",
			};

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockUserProfile)),
			);

			await apiClient.register(registerData);

			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/auth/register",
				expect.objectContaining({
					method: "POST",
					headers: expect.objectContaining({
						"Content-Type": "application/json",
					}),
					body: JSON.stringify(registerData),
				}),
			);
		});

		it("should construct DELETE requests correctly", async () => {
			const taskId = "task-123";
			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse("Task deleted")),
			);

			await apiClient.deleteTask(taskId);

			expect(mockFetch).toHaveBeenCalledWith(
				`/api/v1/tasks/${taskId}`,
				expect.objectContaining({
					method: "DELETE",
				}),
			);
		});

		it("should handle query parameters correctly", async () => {
			const params = {
				task_type: "email",
				status: "pending",
				limit: 10,
				offset: 5, // Use non-zero offset to ensure it gets included
			};

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse([mockTask])),
			);

			await apiClient.getTasks(params);

			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/tasks?task_type=email&status=pending&limit=10&offset=5",
				expect.any(Object),
			);
		});
	});

	describe("Authentication Endpoints", () => {
		it("should register user successfully", async () => {
			const registerData = {
				username: "testuser",
				email: "test@example.com",
				password: "password123",
			};

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockUserProfile)),
			);

			const result = await apiClient.register(registerData);

			expect(result.data).toEqual(mockUserProfile);
			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/auth/register",
				expect.objectContaining({
					method: "POST",
					body: JSON.stringify(registerData),
				}),
			);
		});

		it("should login and store token automatically", async () => {
			const loginData = {
				email: "test@example.com",
				password: "password123",
			};
			const sessionToken = "session-token-123";
			const loginResponse = {
				user: mockAuthUser,
				session_token: sessionToken,
			};

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(loginResponse)),
			);

			const result = await apiClient.login(loginData);

			expect(result.data).toEqual(loginResponse);
			expect(getAuthToken()).toBe(sessionToken);
		});

		it("should logout and clear token", async () => {
			setAuthToken("test-token");

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse("Logged out")),
			);

			await apiClient.logout();

			expect(getAuthToken()).toBeNull();
		});

		it("should get current user", async () => {
			setAuthToken("test-token");

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockAuthUser)),
			);

			const result = await apiClient.getCurrentUser();

			expect(result.data).toEqual(mockAuthUser);
		});
	});

	describe("Task Endpoints", () => {
		beforeEach(() => {
			setAuthToken("test-token");
		});

		it("should create task", async () => {
			const taskData = {
				task_type: "email",
				payload: { to: "test@example.com", subject: "Test" },
				priority: "normal" as const,
			};

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockTask)),
			);

			const result = await apiClient.createTask(taskData);

			expect(result.data).toEqual(mockTask);
			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/tasks",
				expect.objectContaining({
					method: "POST",
					body: JSON.stringify(taskData),
				}),
			);
		});

		it("should get task by id", async () => {
			const taskId = "task-123";

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockTask)),
			);

			const result = await apiClient.getTask(taskId);

			expect(result.data).toEqual(mockTask);
			expect(mockFetch).toHaveBeenCalledWith(
				`/api/v1/tasks/${taskId}`,
				expect.any(Object),
			);
		});

		it("should get task stats", async () => {
			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockTaskStats)),
			);

			const result = await apiClient.getTaskStats();

			expect(result.data).toEqual(mockTaskStats);
		});

		it("should cancel task", async () => {
			const taskId = "task-123";

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse("Task cancelled")),
			);

			await apiClient.cancelTask(taskId);

			expect(mockFetch).toHaveBeenCalledWith(
				`/api/v1/tasks/${taskId}/cancel`,
				expect.objectContaining({ method: "POST" }),
			);
		});

		it("should retry task", async () => {
			const taskId = "task-123";

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse("Task retried")),
			);

			await apiClient.retryTask(taskId);

			expect(mockFetch).toHaveBeenCalledWith(
				`/api/v1/tasks/${taskId}/retry`,
				expect.objectContaining({ method: "POST" }),
			);
		});
	});

	describe("Health Endpoints", () => {
		it("should get basic health", async () => {
			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockHealthResponse)),
			);

			const result = await apiClient.getHealth();

			expect(result.data).toEqual(mockHealthResponse);
			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/health",
				expect.any(Object),
			);
		});

		it("should get detailed health", async () => {
			const detailedHealth = {
				...mockHealthResponse,
				version: "1.0.0",
				uptime: 3600,
			};

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(detailedHealth)),
			);

			const result = await apiClient.getDetailedHealth();

			expect(result.data).toEqual(detailedHealth);
			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/health/detailed",
				expect.any(Object),
			);
		});

		it("should get liveness probe", async () => {
			const probeResponse = { status: "alive" };

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(probeResponse)),
			);

			const result = await apiClient.getLivenessProbe();

			expect(result.data).toEqual(probeResponse);
			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/health/live",
				expect.any(Object),
			);
		});
	});

	describe("User Management Endpoints", () => {
		beforeEach(() => {
			setAuthToken("admin-token");
		});

		it("should update own profile", async () => {
			const updateData = {
				username: "newusername",
				email: "newemail@example.com",
			};

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockUserProfile)),
			);

			const result = await apiClient.updateOwnProfile(updateData);

			expect(result.data).toEqual(mockUserProfile);
			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/users/me/profile",
				expect.objectContaining({
					method: "PUT",
					body: JSON.stringify(updateData),
				}),
			);
		});

		it("should change own password", async () => {
			const passwordData = {
				current_password: "oldpass",
				new_password: "newpass",
			};

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse("Password changed")),
			);

			await apiClient.changeOwnPassword(passwordData);

			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/users/me/password",
				expect.objectContaining({
					method: "PUT",
					body: JSON.stringify(passwordData),
				}),
			);
		});

		it("should get users list", async () => {
			const params = { limit: 10, offset: 5 }; // Use non-zero offset

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse([mockUserProfile])),
			);

			const result = await apiClient.getUsers(params);

			expect(result.data).toEqual([mockUserProfile]);
			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/users?limit=10&offset=5",
				expect.any(Object),
			);
		});
	});

	describe("Error Handling", () => {
		it("should handle HTTP error responses", async () => {
			const errorResponse = mockApiError("Not found", "NOT_FOUND");

			mockFetch.mockResolvedValueOnce(createMockResponse(errorResponse, 404));

			await expect(apiClient.getHealth()).rejects.toThrow("Not found");
		});

		it("should handle network errors", async () => {
			mockFetch.mockRejectedValueOnce(new Error("Network error"));

			await expect(apiClient.getHealth()).rejects.toThrow("Network error");
		});

		it("should handle invalid JSON responses", async () => {
			const invalidResponse = {
				ok: true,
				status: 200,
				json: vi.fn().mockRejectedValue(new Error("Invalid JSON")),
			} as unknown as Response;

			mockFetch.mockResolvedValueOnce(invalidResponse);

			await expect(apiClient.getHealth()).rejects.toThrow("Invalid JSON");
		});

		it("should handle 401 unauthorized errors", async () => {
			const errorResponse = mockApiError("Unauthorized", "UNAUTHORIZED");

			mockFetch.mockResolvedValueOnce(createMockResponse(errorResponse, 401));

			await expect(apiClient.getCurrentUser()).rejects.toThrow("Unauthorized");
		});

		it("should handle 403 forbidden errors", async () => {
			const errorResponse = mockApiError("Forbidden", "FORBIDDEN");

			mockFetch.mockResolvedValueOnce(createMockResponse(errorResponse, 403));

			await expect(apiClient.getUsers()).rejects.toThrow("Forbidden");
		});
	});

	describe("Monitoring Endpoints", () => {
		beforeEach(() => {
			setAuthToken("test-token");
		});

		it("should create event", async () => {
			const eventData = {
				event_type: "log" as const,
				source: "test-source",
				message: "Test message",
				level: "info",
			};

			const mockEvent = {
				id: "event-123",
				...eventData,
				recorded_at: "2024-01-01T00:00:00Z",
			};

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockEvent)),
			);

			const result = await apiClient.createEvent(eventData);

			expect(result.data).toEqual(mockEvent);
			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/monitoring/events",
				expect.objectContaining({
					method: "POST",
					body: JSON.stringify(eventData),
				}),
			);
		});

		it("should get events with filters", async () => {
			const params = {
				event_type: "log" as const,
				source: "test-source",
				level: "info" as const,
				limit: 10,
			};

			mockFetch.mockResolvedValueOnce(createMockResponse(mockApiResponse([])));

			await apiClient.getEvents(params);

			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/monitoring/events?event_type=log&source=test-source&level=info&limit=10",
				expect.any(Object),
			);
		});

		it("should create metric", async () => {
			const metricData = {
				name: "test_metric",
				metric_type: "counter" as const,
				value: 1,
				labels: { service: "test" },
			};

			const mockMetric = {
				id: "metric-123",
				...metricData,
				recorded_at: "2024-01-01T00:00:00Z",
			};

			mockFetch.mockResolvedValueOnce(
				createMockResponse(mockApiResponse(mockMetric)),
			);

			const result = await apiClient.createMetric(metricData);

			expect(result.data).toEqual(mockMetric);
		});

		it("should get prometheus metrics as text", async () => {
			const prometheusData = "# HELP test_metric Test metric\ntest_metric 1\n";

			const mockResponse = {
				ok: true,
				status: 200,
				text: vi.fn().mockResolvedValue(prometheusData),
			} as unknown as Response;

			mockFetch.mockResolvedValueOnce(mockResponse);

			const result = await apiClient.getPrometheusMetrics();

			expect(result).toBe(prometheusData);
			expect(mockFetch).toHaveBeenCalledWith(
				"/api/v1/monitoring/metrics/prometheus",
				expect.any(Object),
			);
		});
	});
});
