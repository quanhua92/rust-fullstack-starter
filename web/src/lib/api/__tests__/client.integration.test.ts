import {
	describeIntegration,
	setupIntegrationTest,
} from "@/test/integration-setup";
import { createTestUser } from "@/test/mocks";
import { afterEach, beforeAll, describe, expect, it } from "vitest";
import { ApiClient, getAuthToken, setAuthToken } from "../client";

describeIntegration("API Client Integration Tests", () => {
	let apiClient: ApiClient;
	const { baseUrl } = setupIntegrationTest();

	// Keep track of created resources for cleanup
	const createdUsers: string[] = [];
	const createdTasks: string[] = [];

	beforeAll(() => {
		apiClient = new ApiClient(baseUrl);
	});

	afterEach(async () => {
		// Clean up created resources
		for (const taskId of createdTasks) {
			try {
				await apiClient.deleteTask(taskId);
			} catch (error) {
				// Task might not exist or be already deleted
				console.warn(`Failed to cleanup task ${taskId}:`, error);
			}
		}
		createdTasks.length = 0;

		// Clean up created users (requires admin privileges - skip if not available)
		for (const userId of createdUsers) {
			try {
				// Note: This would require admin API endpoint for user deletion
				// For now, we'll just warn about the cleanup limitation
				console.warn(`User cleanup not implemented for test isolation: ${userId}`);
			} catch (error) {
				console.warn(`Failed to cleanup user ${userId}:`, error);
			}
		}
		createdUsers.length = 0;

		// Clear auth token
		setAuthToken(null);
	});

	describe("Health Endpoints", () => {
		it("should get basic health status", async () => {
			const response = await apiClient.getHealth();

			expect(response.success).toBe(true);
			expect(response.data).toBeDefined();
			expect(response.data?.status).toBe("healthy");
			// Basic health doesn't have timestamp/checks
			expect(response.data?.version).toBeDefined();
			expect(response.data?.uptime).toBeGreaterThan(0);
		});

		it("should get detailed health status", async () => {
			const response = await apiClient.getDetailedHealth();

			expect(response.success).toBe(true);
			expect(response.data).toBeDefined();
			expect(response.data?.status).toBe("healthy");
			expect(response.data?.checks).toBeDefined();
			// Detailed health may include additional fields like version, uptime
		});

		it("should get health probes", async () => {
			const [liveness, readiness, startup] = await Promise.all([
				apiClient.getLivenessProbe(),
				apiClient.getReadinessProbe(),
				apiClient.getStartupProbe(),
			]);

			expect(liveness.success).toBe(true);
			expect(readiness.success).toBe(true);
			expect(startup.success).toBe(true);
		});
	});

	describe("Authentication Flow", () => {
		let testUserData: ReturnType<typeof createTestUser>;

		beforeAll(() => {
			testUserData = createTestUser();
		});

		it("should register a new user", async () => {
			const response = await apiClient.register(testUserData);

			expect(response.success).toBe(true);
			expect(response.data).toBeDefined();
			expect(response.data?.username).toBe(testUserData.username);
			expect(response.data?.email).toBe(testUserData.email);
			expect(response.data?.role).toBe("user");
			expect(response.data?.is_active).toBe(true);

			if (response.data?.id) {
				createdUsers.push(response.data.id);
			}
		});

		it("should login with valid credentials", async () => {
			// First register the user
			await apiClient.register(testUserData);

			const loginResponse = await apiClient.login({
				email: testUserData.email,
				password: testUserData.password,
			});

			expect(loginResponse.success).toBe(true);
			expect(loginResponse.data?.session_token).toBeDefined();
			expect(loginResponse.data?.user).toBeDefined();
			expect(loginResponse.data?.user.email).toBe(testUserData.email);

			// Token should be automatically stored
			expect(getAuthToken()).toBe(loginResponse.data?.session_token);
		});

		it("should reject login with invalid credentials", async () => {
			await expect(
				apiClient.login({
					email: "nonexistent@example.com",
					password: "wrongpassword",
				}),
			).rejects.toThrow();
		});

		it("should get current user when authenticated", async () => {
			// Register and login
			await apiClient.register(testUserData);
			const loginResponse = await apiClient.login({
				email: testUserData.email,
				password: testUserData.password,
			});
			setAuthToken(loginResponse.data?.session_token || null);

			const userResponse = await apiClient.getCurrentUser();

			expect(userResponse.success).toBe(true);
			expect(userResponse.data?.email).toBe(testUserData.email);
			expect(userResponse.data?.role).toBe("user");
		});

		it("should reject getCurrentUser when not authenticated", async () => {
			setAuthToken(null);

			await expect(apiClient.getCurrentUser()).rejects.toThrow();
		});

		it("should logout successfully", async () => {
			// Register and login first
			await apiClient.register(testUserData);
			await apiClient.login({
				email: testUserData.email,
				password: testUserData.password,
			});

			const logoutResponse = await apiClient.logout();

			expect(logoutResponse.success).toBe(true);
			expect(getAuthToken()).toBeNull();
		});
	});

	describe("Task Management", () => {
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

		it("should create a new task", async () => {
			const taskData = {
				task_type: "email",
				payload: {
					to: "test@example.com",
					subject: "Test Email",
					body: "This is a test email",
				},
				priority: "normal" as const,
			};

			const response = await apiClient.createTask(taskData);

			expect(response.success).toBe(true);
			expect(response.data?.id).toBeDefined();
			expect(response.data?.task_type).toBe(taskData.task_type);
			expect(response.data?.status).toBe("pending");
			expect(response.data?.priority).toBe(taskData.priority);

			if (response.data?.id) {
				createdTasks.push(response.data.id);
			}
		});

		it("should get task by ID", async () => {
			// Create a task first
			const createResponse = await apiClient.createTask({
				task_type: "email",
				payload: { to: "test@example.com", subject: "Test" },
				priority: "normal",
			});
			const taskId = createResponse.data?.id;
			if (taskId) createdTasks.push(taskId);

			const getResponse = await apiClient.getTask(taskId as string);

			expect(getResponse.success).toBe(true);
			expect(getResponse.data?.id).toBe(taskId);
			expect(getResponse.data?.task_type).toBe("email");
		});

		it("should get tasks list", async () => {
			// Create a task first
			const createResponse = await apiClient.createTask({
				task_type: "email",
				payload: { to: "test@example.com", subject: "Test" },
				priority: "high",
			});
			if (createResponse.data?.id) createdTasks.push(createResponse.data.id);

			const listResponse = await apiClient.getTasks({
				limit: 10,
				offset: 0,
			});

			expect(listResponse.success).toBe(true);
			expect(Array.isArray(listResponse.data)).toBe(true);
			expect(listResponse.data?.length).toBeGreaterThan(0);
		});

		it("should get task stats", async () => {
			const response = await apiClient.getTaskStats();

			expect(response.success).toBe(true);
			expect(response.data?.total).toBeGreaterThanOrEqual(0);
			expect(response.data?.pending).toBeGreaterThanOrEqual(0);
			expect(response.data?.running).toBeGreaterThanOrEqual(0);
			expect(response.data?.completed).toBeGreaterThanOrEqual(0);
			expect(response.data?.failed).toBeGreaterThanOrEqual(0);
			expect(response.data?.cancelled).toBeGreaterThanOrEqual(0);
		});

		it("should cancel a task", async () => {
			// Create a task first
			const createResponse = await apiClient.createTask({
				task_type: "email",
				payload: { to: "test@example.com", subject: "Test" },
				priority: "normal",
			});
			const taskId = createResponse.data?.id;
			if (taskId) createdTasks.push(taskId);

			const cancelResponse = await apiClient.cancelTask(taskId as string);

			expect(cancelResponse.success).toBe(true);

			// Verify task is cancelled
			const getResponse = await apiClient.getTask(taskId as string);
			expect(getResponse.data?.status).toBe("cancelled");
		});

		it("should delete a task", async () => {
			// Create a task first
			const createResponse = await apiClient.createTask({
				task_type: "email",
				payload: { to: "test@example.com", subject: "Test" },
				priority: "normal",
			});
			const taskId = createResponse.data?.id;

			const deleteResponse = await apiClient.deleteTask(taskId as string);

			expect(deleteResponse.success).toBe(true);

			// Verify task is deleted - should throw error
			await expect(apiClient.getTask(taskId as string)).rejects.toThrow();
		});

		it("should reject task operations without authentication", async () => {
			const originalToken = getAuthToken();
			setAuthToken(null);

			await expect(
				apiClient.createTask({
					task_type: "email",
					payload: { to: "test@example.com" },
					priority: "normal",
				}),
			).rejects.toThrow();

			// Restore token
			setAuthToken(originalToken);
		});
	});

	describe("User Profile Management", () => {
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

		it("should update own profile", async () => {
			const newUsername = `updated_${testUser.username}`;
			const updateData = {
				username: newUsername,
			};

			const response = await apiClient.updateOwnProfile(updateData);

			expect(response.success).toBe(true);
			expect(response.data?.username).toBe(newUsername);
			expect(response.data?.email).toBe(testUser.email); // Should remain unchanged
		});

		it("should change own password", async () => {
			const passwordData = {
				current_password: testUser.password,
				new_password: "NewSecurePassword123!",
			};

			const response = await apiClient.changeOwnPassword(passwordData);

			expect(response.success).toBe(true);

			// Verify we can login with new password
			await apiClient.logout();
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: passwordData.new_password,
			});

			expect(loginResponse.success).toBe(true);
		});
	});

	describe("Monitoring Endpoints", () => {
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

		it("should create and retrieve events", async () => {
			const eventData = {
				event_type: "log" as const,
				source: "integration-test",
				message: "Test event from integration test",
				level: "info",
				tags: { test: "true", suite: "integration" },
			};

			// Create event
			const createResponse = await apiClient.createEvent(eventData);

			expect(createResponse.success).toBe(true);
			expect(createResponse.data?.id).toBeDefined();
			expect(createResponse.data?.event_type).toBe(eventData.event_type);
			expect(createResponse.data?.source).toBe(eventData.source);

			const eventId = createResponse.data?.id;

			// Retrieve the specific event
			const getResponse = await apiClient.getEvent(eventId as string);

			expect(getResponse.success).toBe(true);
			expect(getResponse.data?.id).toBe(eventId);
			expect(getResponse.data?.message).toBe(eventData.message);
		});

		it("should get events with filters", async () => {
			// Create a test event first
			await apiClient.createEvent({
				event_type: "log",
				source: "integration-filter-test",
				message: "Test event for filtering",
				level: "debug",
			});

			// Get events with filters
			const response = await apiClient.getEvents({
				event_type: "log",
				source: "integration-filter-test",
				level: "debug",
				limit: 10,
			});

			expect(response.success).toBe(true);
			expect(Array.isArray(response.data)).toBe(true);
			expect(response.data?.length).toBeGreaterThan(0);

			// Verify filtering worked
			const event = response.data?.[0];
			expect(event?.source).toBe("integration-filter-test");
			expect(event?.level).toBe("debug");
		});

		it("should create and retrieve metrics", async () => {
			const metricData = {
				name: "integration_test_counter",
				metric_type: "counter" as const,
				value: 5,
				labels: {
					test_suite: "integration",
					endpoint: "api_client",
				},
			};

			// Create metric
			const createResponse = await apiClient.createMetric(metricData);

			expect(createResponse.success).toBe(true);
			expect(createResponse.data?.id).toBeDefined();
			expect(createResponse.data?.name).toBe(metricData.name);
			expect(createResponse.data?.value).toBe(metricData.value);

			// Get metrics by name
			const getResponse = await apiClient.getMetrics({
				name: metricData.name,
				limit: 5,
			});

			expect(getResponse.success).toBe(true);
			expect(Array.isArray(getResponse.data)).toBe(true);
			expect(getResponse.data?.length).toBeGreaterThan(0);

			const metric = getResponse.data?.find((m) => m.name === metricData.name);
			expect(metric).toBeDefined();
		});

		it("should get prometheus metrics format", async () => {
			const response = await apiClient.getPrometheusMetrics();

			expect(typeof response).toBe("string");
			expect(response).toContain("# HELP");
			// Basic validation that it's prometheus format
			expect(response.includes("\n")).toBe(true);
		});
	});

	describe("Error Handling", () => {
		it("should handle 404 errors gracefully", async () => {
			await expect(apiClient.getTask("non-existent-task-id")).rejects.toThrow();
		});

		it("should handle authentication errors", async () => {
			setAuthToken("invalid-token");

			await expect(apiClient.getCurrentUser()).rejects.toThrow();
		});

		it("should handle validation errors on task creation", async () => {
			const testUser = createTestUser();
			await apiClient.register(testUser);
			await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});

			// Try to create task with invalid data
			await expect(
				apiClient.createTask({
					task_type: "invalid-task-type",
					payload: {},
					priority: "invalid-priority" as "normal",
				}),
			).rejects.toThrow();
		});
	});

	describe("Rate Limiting and Performance", () => {
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

		it("should handle multiple concurrent requests", async () => {
			const promises = Array(5)
				.fill(null)
				.map(() => apiClient.getHealth());

			const results = await Promise.all(promises);

			expect(results).toHaveLength(5);
			for (const result of results) {
				expect(result.success).toBe(true);
				expect(result.data?.status).toBe("healthy");
			}
		});

		it("should maintain performance on repeated requests", async () => {
			const start = Date.now();

			for (let i = 0; i < 3; i++) {
				await apiClient.getTaskStats();
			}

			const duration = Date.now() - start;

			// Should complete 3 requests within reasonable time (10 seconds)
			expect(duration).toBeLessThan(10000);
		});
	});
});
