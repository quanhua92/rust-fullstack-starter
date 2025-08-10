import {
	describeIntegration,
	setupIntegrationTest,
} from "@/test/integration-setup";
import { createTestUser } from "@/test/mocks";
import { afterEach, beforeAll, describe, expect, it } from "vitest";
import { ApiClient, setAuthToken } from "../api/client";

describeIntegration("User Journey Integration Tests", () => {
	let apiClient: ApiClient;
	const { baseUrl } = setupIntegrationTest();

	// Keep track of created resources for cleanup
	const createdUsers: string[] = [];
	const createdTasks: string[] = [];

	beforeAll(() => {
		apiClient = new ApiClient(baseUrl);
	});

	afterEach(() => {
		// Clear auth token between tests
		setAuthToken(null);
	});

	describe("Complete User Onboarding Journey", () => {
		it("should complete registration → login → profile update → first task creation", async () => {
			// Step 1: User Registration
			const testUser = createTestUser();

			const registerResponse = await apiClient.register(testUser);
			expect(registerResponse.success).toBe(true);
			expect(registerResponse.data).toHaveProperty("id");
			expect(registerResponse.data?.email).toBe(testUser.email);

			const userId = registerResponse.data?.id;
			if (userId) createdUsers.push(userId);

			// Step 2: User Login
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			expect(loginResponse.success).toBe(true);
			expect(loginResponse.data?.session_token).toBeDefined();

			// Fail fast if server returns misaligned response
			const sessionToken = loginResponse.data?.session_token;
			if (!sessionToken) {
				throw new Error(
					`Server response malformed: missing session_token in login response: ${JSON.stringify(loginResponse)}`,
				);
			}
			setAuthToken(sessionToken);

			// Step 3: Verify authenticated user can access their profile
			const profileResponse = await apiClient.getCurrentUser();
			expect(profileResponse.success).toBe(true);
			expect(profileResponse.data?.email).toBe(testUser.email);
			expect(profileResponse.data?.role).toBe("user");

			// Step 4: Update user profile
			const profileUpdate = {
				username: `${testUser.username}_updated`,
			};
			const updateResponse = await apiClient.updateOwnProfile(profileUpdate);
			expect(updateResponse.success).toBe(true);
			expect(updateResponse.data?.username).toBe(profileUpdate.username);

			// Step 5: Create first task (represents user's first action)
			const taskData = {
				task_type: "email",
				payload: {
					to: "user@example.com",
					subject: "Welcome to the platform!",
					body: "This is your first task",
				},
				priority: "normal" as const,
			};

			const taskResponse = await apiClient.createTask(taskData);
			expect(taskResponse.success).toBe(true);
			expect(taskResponse.data?.task_type).toBe(taskData.task_type);
			expect(taskResponse.data?.created_by).toBe(userId);
			expect(taskResponse.data?.status).toBe("pending");

			// Fail fast if server returns malformed task response
			const taskId = taskResponse.data?.id;
			if (!taskId) {
				throw new Error(
					`Server response malformed: missing task ID in create response: ${JSON.stringify(taskResponse)}`,
				);
			}
			createdTasks.push(taskId);

			// Step 6: Verify user can retrieve their created task
			const tasksListResponse = await apiClient.getTasks({});
			expect(tasksListResponse.success).toBe(true);
			expect(tasksListResponse.data).toBeDefined();
			expect(Array.isArray(tasksListResponse.data)).toBe(true);
			expect(tasksListResponse.data?.length).toBeGreaterThanOrEqual(1);

			const createdTask = tasksListResponse.data?.find(
				(task) => task.id === taskId,
			);
			expect(createdTask).toBeDefined();
			expect(createdTask?.task_type).toBe(taskData.task_type);

			// Step 7: User can view task details
			const taskDetailResponse = await apiClient.getTask(taskId);
			expect(taskDetailResponse.success).toBe(true);
			expect(taskDetailResponse.data?.id).toBe(taskId);
			expect(taskDetailResponse.data?.task_type).toBe(taskData.task_type);
		}, 15000); // Allow extra time for multi-step workflow

		it("should handle user authentication workflow with logout", async () => {
			// Step 1: Register and login
			const testUser = createTestUser();

			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});

			if (loginResponse.data?.session_token) {
				setAuthToken(loginResponse.data.session_token);
			} else {
				console.error("Login response missing session token:", loginResponse);
				throw new Error("Session token not found in login response");
			}

			// Step 2: Verify authenticated access works
			const profileResponse = await apiClient.getCurrentUser();
			expect(profileResponse.success).toBe(true);

			// Step 3: Logout
			const logoutResponse = await apiClient.logout();
			expect(logoutResponse.success).toBe(true);

			// Step 4: Verify access is denied after logout
			setAuthToken(null);
			await expect(apiClient.getCurrentUser()).rejects.toThrow("Unauthorized");

			// Step 5: Verify re-login works
			const reLoginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			expect(reLoginResponse.success).toBe(true);
			expect(reLoginResponse.data?.session_token).toBeDefined();
		}, 10000);
	});

	describe("Password Management Journey", () => {
		it("should complete password change workflow", async () => {
			// Step 1: Register and login
			const testUser = createTestUser();

			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});

			if (loginResponse.data?.session_token) {
				setAuthToken(loginResponse.data.session_token);
			} else {
				console.error("Login response missing session token:", loginResponse);
				throw new Error("Session token not found in login response");
			}

			// Step 2: Change password
			const newPassword = "NewSecurePassword123!";
			const changePasswordResponse = await apiClient.changeOwnPassword({
				current_password: testUser.password,
				new_password: newPassword,
			});
			expect(changePasswordResponse.success).toBe(true);

			// Step 3: Verify old password no longer works
			setAuthToken(null);
			await expect(
				apiClient.login({
					email: testUser.email,
					password: testUser.password, // old password
				}),
			).rejects.toThrow("Invalid credentials");

			// Step 4: Verify new password works
			const newLoginResponse = await apiClient.login({
				email: testUser.email,
				password: newPassword, // new password
			});
			expect(newLoginResponse.success).toBe(true);
			expect(newLoginResponse.data?.session_token).toBeDefined();
		}, 10000);
	});

	describe("Task Management Journey", () => {
		it("should complete task lifecycle: create → view → cancel → verify", async () => {
			// Setup: Register and login
			const testUser = createTestUser();
			await apiClient.register(testUser);
			const loginResponse = await apiClient.login({
				email: testUser.email,
				password: testUser.password,
			});
			if (loginResponse.data?.session_token) {
				setAuthToken(loginResponse.data.session_token);
			} else {
				console.error("Login response missing session token:", loginResponse);
				throw new Error("Session token not found in login response");
			}

			// Step 1: Create task
			const taskData = {
				task_type: "email", // Use email task type which is commonly supported
				payload: {
					to: "test@example.com",
					subject: "Task Lifecycle Test",
					body: "Testing task creation and cancellation",
				},
				priority: "high" as const,
			};

			const createResponse = await apiClient.createTask(taskData);
			expect(createResponse.success).toBe(true);
			// Fail fast if server returns malformed task response
			const taskId = createResponse.data?.id;
			if (!taskId) {
				throw new Error(
					`Server response malformed: missing task ID: ${JSON.stringify(createResponse)}`,
				);
			}
			createdTasks.push(taskId);

			// Step 2: View task details (taskId validated above)
			const viewResponse = await apiClient.getTask(taskId);
			expect(viewResponse.success).toBe(true);
			expect(viewResponse.data?.status).toBe("pending");
			expect(viewResponse.data?.priority).toBe("high");

			// Step 3: Cancel task
			const cancelResponse = await apiClient.cancelTask(taskId);
			expect(cancelResponse.success).toBe(true);

			// Step 4: Verify task is cancelled
			const verifyResponse = await apiClient.getTask(taskId);
			expect(verifyResponse.success).toBe(true);
			expect(verifyResponse.data?.status).toBe("cancelled");

			// Step 5: Verify cancelled task appears in user's task list
			const tasksResponse = await apiClient.getTasks({ status: "cancelled" });
			expect(tasksResponse.success).toBe(true);
			expect(tasksResponse.data).toBeDefined();
			expect(Array.isArray(tasksResponse.data)).toBe(true);
			expect(tasksResponse.data?.some((task) => task.id === taskId)).toBe(true);
		}, 10000);
	});
});
