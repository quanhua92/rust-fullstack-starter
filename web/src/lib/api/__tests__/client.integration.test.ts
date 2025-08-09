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

	// Helper to ensure task types are registered (matches test-with-curl.sh pattern)
	const ensureTaskTypesRegistered = async () => {
		const requiredTaskTypes = ["email", "data_processing", "webhook"];

		try {
			// Check if task types are already registered
			const response = await fetch(`${baseUrl}/tasks/types`);
			if (response.ok) {
				const data = await response.json();
				const registeredTypes =
					data.data?.map((t: { task_type: string }) => t.task_type) || [];

				// Register missing task types (fallback registration like curl script)
				for (const taskType of requiredTaskTypes) {
					if (!registeredTypes.includes(taskType)) {
						await fetch(`${baseUrl}/tasks/types`, {
							method: "POST",
							headers: { "Content-Type": "application/json" },
							body: JSON.stringify({
								task_type: taskType,
								description: `${taskType} task type (test fallback registration)`,
							}),
						});
					}
				}
			}
		} catch (error) {
			// Ignore registration errors - workers might handle this
			console.warn(
				"Task type registration check failed, proceeding anyway:",
				error,
			);
		}
	};

	afterEach(() => {
		// Clear auth token only - no cleanup needed since we use unique data each time
		// This matches test-with-curl.sh pattern: create fresh data, no cleanup required
		setAuthToken(null);
		createdUsers.length = 0;
		createdTasks.length = 0;
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
		it("should register a new user", async () => {
			// Create unique user for this test (like curl script)
			const testUserData = createTestUser();
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
			// Create unique user for this test (like curl script)
			const testUserData = createTestUser();
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
			// Create unique user for this test (like curl script)
			const testUserData = createTestUser();
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
			// Create unique user for this test (like curl script)
			const testUserData = createTestUser();
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
		// Helper to create authenticated user for each test (like curl script)
		const createAuthenticatedUser = async () => {
			const testUser = createTestUser();
			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			const token = loginResponse.data?.session_token || "";
			setAuthToken(token);
			return token;
		};

		beforeAll(async () => {
			// Ensure task types are registered (like curl script fallback)
			await ensureTaskTypesRegistered();
		});

		it("should create a new task", async () => {
			await createAuthenticatedUser();

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
			await createAuthenticatedUser();

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
			await createAuthenticatedUser();

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

		it("should handle task stats permission requirement", async () => {
			await createAuthenticatedUser();

			// Task stats require moderator+ permissions (like curl script shows for regular users)
			try {
				const response = await apiClient.getTaskStats();
				// If this succeeds, it means user has elevated permissions
				expect(response.success).toBe(true);
				expect(response.data?.total).toBeGreaterThanOrEqual(0);
			} catch (error: unknown) {
				// Accept 403 Forbidden for regular user (matches curl script logic)
				const errorMessage =
					error instanceof Error ? error.message : String(error);
				expect(errorMessage).toMatch(/403|moderator|forbidden/i);
			}
		});

		it("should cancel a task", async () => {
			await createAuthenticatedUser();

			// Create a task first
			const createResponse = await apiClient.createTask({
				task_type: "email",
				payload: { to: "test@example.com", subject: "Test" },
				priority: "normal",
			});
			const taskId = createResponse.data?.id;
			if (taskId) createdTasks.push(taskId);

			// Attempt to cancel - might return 400 if task already processed (like curl script)
			try {
				const cancelResponse = await apiClient.cancelTask(taskId as string);
				expect(cancelResponse.success).toBe(true);

				// Verify task is cancelled (if still available)
				const getResponse = await apiClient.getTask(taskId as string);
				expect(getResponse.data?.status).toBe("cancelled");
			} catch (error: unknown) {
				// Accept 400 Bad Request if task already processed (matches curl script logic)
				const errorMessage =
					error instanceof Error ? error.message : String(error);
				expect(errorMessage).toMatch(/400|already/i);
			}
		});

		it("should delete a task", async () => {
			await createAuthenticatedUser();

			// Create a task first
			const createResponse = await apiClient.createTask({
				task_type: "email",
				payload: { to: "test@example.com", subject: "Test" },
				priority: "normal",
			});
			const taskId = createResponse.data?.id;

			// Attempt to delete - might return 404 if task already processed (like curl script)
			try {
				const deleteResponse = await apiClient.deleteTask(taskId as string);
				expect(deleteResponse.success).toBe(true);

				// Verify task is deleted - should throw error
				await expect(apiClient.getTask(taskId as string)).rejects.toThrow();
			} catch (error: unknown) {
				// Accept 404 Not Found if task already completed and removed (matches curl script logic)
				const errorMessage =
					error instanceof Error ? error.message : String(error);
				expect(errorMessage).toMatch(/404|not found/i);
			}
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
		it("should update own profile", async () => {
			// Create unique user for this test (like curl script)
			const testUser = createTestUser();
			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			setAuthToken(loginResponse.data?.session_token || null);

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
			// Create unique user for this test (like curl script)
			const testUser = createTestUser();
			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			setAuthToken(loginResponse.data?.session_token || null);

			const passwordData = {
				current_password: testUser.password,
				new_password: "NewSecurePassword123!",
			};

			const response = await apiClient.changeOwnPassword(passwordData);

			expect(response.success).toBe(true);

			// Verify we can login with new password
			await apiClient.logout();
			const loginResponse2 = await apiClient.login({
				email: testUser.email,
				password: passwordData.new_password,
			});

			expect(loginResponse2.success).toBe(true);
		});
	});

	describe("Monitoring Endpoints", () => {
		// Helper to create authenticated user for each test (like curl script)
		const createAuthenticatedUser = async () => {
			const testUser = createTestUser();
			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			const token = loginResponse.data?.session_token || "";
			setAuthToken(token);
			return { token, username: testUser.username };
		};

		beforeAll(async () => {
			// Ensure task types are registered for monitoring tests too
			await ensureTaskTypesRegistered();
		});

		it("should create and retrieve events", async () => {
			// Create user and use username in source (like curl script - "test-script")
			const { username } = await createAuthenticatedUser();
			const eventData = {
				event_type: "log" as const,
				source: username, // Use username as source for authorization
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
			// Create user and use username in source for authorization
			const { username } = await createAuthenticatedUser();

			// Create a test event first
			await apiClient.createEvent({
				event_type: "log",
				source: username, // Use username as source for authorization
				message: "Test event for filtering",
				level: "debug",
			});

			// Get events with filters
			const response = await apiClient.getEvents({
				event_type: "log",
				source: username,
				level: "debug",
				limit: 10,
			});

			expect(response.success).toBe(true);
			expect(Array.isArray(response.data)).toBe(true);
			expect(response.data?.length).toBeGreaterThan(0);

			// Verify filtering worked
			const event = response.data?.[0];
			expect(event?.source).toBe(username);
			expect(event?.level).toBe("debug");
		});

		it("should create and retrieve metrics", async () => {
			// Create user and use username prefix in metric name for authorization
			const { username } = await createAuthenticatedUser();
			const metricData = {
				name: `${username}_integration_test_counter`, // Use username prefix for authorization
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

			// Try to create task with invalid task type (like curl script)
			await expect(
				apiClient.createTask({
					task_type: "absolutely_unknown_type_9999",
					payload: {},
					priority: "normal",
				}),
			).rejects.toThrow();
		});
	});

	describe("Rate Limiting and Performance", () => {
		// Helper to create authenticated user for each test (like curl script)
		const createAuthenticatedUser = async () => {
			const testUser = createTestUser();
			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			const token = loginResponse.data?.session_token || "";
			setAuthToken(token);
			return token;
		};

		beforeAll(async () => {
			// Ensure task types are registered for performance tests too
			await ensureTaskTypesRegistered();
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
			await createAuthenticatedUser();

			const start = Date.now();

			// Use health endpoint instead of task stats (no permission required)
			for (let i = 0; i < 3; i++) {
				await apiClient.getHealth();
			}

			const duration = Date.now() - start;

			// Should complete 3 requests within reasonable time (10 seconds)
			expect(duration).toBeLessThan(10000);
		});
	});
});
