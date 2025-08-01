// Typed API client using generated OpenAPI types
import type { components } from "@/types/api";

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "/api/v1";

// Type helpers for cleaner usage
export type ApiError = components["schemas"]["ErrorResponse"];
export type LoginRequest = components["schemas"]["LoginRequest"];
export type RegisterRequest = components["schemas"]["RegisterRequest"];
export type UserProfile = components["schemas"]["UserProfile"];
export type CreateTaskRequest = components["schemas"]["CreateTaskApiRequest"];
export type RegisterTaskTypeRequest =
	components["schemas"]["RegisterTaskTypeRequest"];

// Response types
export type LoginResponse = components["schemas"]["ApiResponse_LoginResponse"];
export type AuthUserResponse = components["schemas"]["ApiResponse_AuthUser"];
export type UserProfileResponse =
	components["schemas"]["ApiResponse_UserProfile"];
export type TaskResponse = components["schemas"]["ApiResponse_TaskResponse"];
export type TaskListResponse =
	components["schemas"]["ApiResponse_Vec_TaskResponse"];
export type TaskStatsResponse = components["schemas"]["ApiResponse_TaskStats"];
export type TaskTypeResponse =
	components["schemas"]["ApiResponse_TaskTypeResponse"];
export type TaskTypeListResponse =
	components["schemas"]["ApiResponse_Vec_TaskTypeResponse"];
export type HealthResponse =
	components["schemas"]["ApiResponse_HealthResponse"];
export type DetailedHealthResponse =
	components["schemas"]["ApiResponse_DetailedHealthResponse"];
export type RefreshResponse = components["schemas"]["ApiResponse_RefreshResponse"];
export type BasicResponse = components["schemas"]["ApiResponse_String"];

// Token management
let authToken: string | null = null;

export const setAuthToken = (token: string | null) => {
	authToken = token;
	if (token) {
		localStorage.setItem("auth_token", token);
	} else {
		localStorage.removeItem("auth_token");
	}
};

export const getAuthToken = (): string | null => {
	if (!authToken) {
		authToken = localStorage.getItem("auth_token");
	}
	return authToken;
};

// Base fetch wrapper with error handling
class ApiClient {
	private baseUrl: string;

	constructor(baseUrl: string = API_BASE_URL) {
		this.baseUrl = baseUrl;
	}

	private async request<T>(
		endpoint: string,
		options: RequestInit = {},
	): Promise<T> {
		const url = `${this.baseUrl}${endpoint}`;
		const token = getAuthToken();

		const headers: Record<string, string> = {
			"Content-Type": "application/json",
			...(options.headers as Record<string, string>),
		};

		if (token) {
			headers.Authorization = `Bearer ${token}`;
		}

		const config: RequestInit = {
			...options,
			headers,
		};

		try {
			const response = await fetch(url, config);

			if (!response.ok) {
				const errorData: ApiError = await response.json();
				throw new Error(errorData.error.message || `HTTP ${response.status}`);
			}

			return await response.json();
		} catch (error) {
			console.error(`API request failed: ${endpoint}`, error);
			throw error;
		}
	}

	// Authentication endpoints
	async register(data: RegisterRequest): Promise<UserProfileResponse> {
		return this.request<UserProfileResponse>("/auth/register", {
			method: "POST",
			body: JSON.stringify(data),
		});
	}

	async login(data: LoginRequest): Promise<LoginResponse> {
		const response = await this.request<LoginResponse>("/auth/login", {
			method: "POST",
			body: JSON.stringify(data),
		});

		// Auto-store token on successful login
		if (response.data?.session_token) {
			setAuthToken(response.data.session_token);
		}

		return response;
	}

	async logout(): Promise<BasicResponse> {
		const response = await this.request<BasicResponse>("/auth/logout", {
			method: "POST",
		});

		// Clear token on logout
		setAuthToken(null);

		return response;
	}

	async logoutAll(): Promise<BasicResponse> {
		const response = await this.request<BasicResponse>("/auth/logout-all", {
			method: "POST",
		});

		// Clear token on logout
		setAuthToken(null);

		return response;
	}

	async getCurrentUser(): Promise<AuthUserResponse> {
		return this.request<AuthUserResponse>("/auth/me");
	}

	async refreshToken(): Promise<RefreshResponse> {
		return this.request<RefreshResponse>("/auth/refresh", {
			method: "POST",
		});
	}

	// User endpoints
	async getUserById(id: string): Promise<UserProfileResponse> {
		return this.request<UserProfileResponse>(`/users/${id}`);
	}

	// Task endpoints
	async getTasks(params?: {
		task_type?: string;
		status?: string;
		limit?: number;
		offset?: number;
	}): Promise<TaskListResponse> {
		const searchParams = new URLSearchParams();
		if (params?.task_type) searchParams.set("task_type", params.task_type);
		if (params?.status) searchParams.set("status", params.status);
		if (params?.limit) searchParams.set("limit", params.limit.toString());
		if (params?.offset) searchParams.set("offset", params.offset.toString());

		const query = searchParams.toString();
		const endpoint = query ? `/tasks?${query}` : "/tasks";

		return this.request<TaskListResponse>(endpoint);
	}

	async createTask(data: CreateTaskRequest): Promise<TaskResponse> {
		return this.request<TaskResponse>("/tasks", {
			method: "POST",
			body: JSON.stringify(data),
		});
	}

	async getTask(id: string): Promise<TaskResponse> {
		return this.request<TaskResponse>(`/tasks/${id}`);
	}

	async deleteTask(id: string): Promise<BasicResponse> {
		return this.request<BasicResponse>(`/tasks/${id}`, {
			method: "DELETE",
		});
	}

	async cancelTask(id: string): Promise<BasicResponse> {
		return this.request<BasicResponse>(`/tasks/${id}/cancel`, {
			method: "POST",
		});
	}

	async retryTask(id: string): Promise<BasicResponse> {
		return this.request<BasicResponse>(`/tasks/${id}/retry`, {
			method: "POST",
		});
	}

	async getTaskStats(): Promise<TaskStatsResponse> {
		return this.request<TaskStatsResponse>("/tasks/stats");
	}

	async getDeadLetterQueue(params?: {
		limit?: number;
		offset?: number;
	}): Promise<TaskListResponse> {
		const searchParams = new URLSearchParams();
		if (params?.limit) searchParams.set("limit", params.limit.toString());
		if (params?.offset) searchParams.set("offset", params.offset.toString());

		const query = searchParams.toString();
		const endpoint = query
			? `/tasks/dead-letter?${query}`
			: "/tasks/dead-letter";

		return this.request<TaskListResponse>(endpoint);
	}

	// Task type endpoints
	async getTaskTypes(): Promise<TaskTypeListResponse> {
		return this.request<TaskTypeListResponse>("/tasks/types");
	}

	async registerTaskType(
		data: RegisterTaskTypeRequest,
	): Promise<TaskTypeResponse> {
		return this.request<TaskTypeResponse>("/tasks/types", {
			method: "POST",
			body: JSON.stringify(data),
		});
	}

	// Health endpoints
	async getHealth(): Promise<HealthResponse> {
		return this.request<HealthResponse>("/health");
	}

	async getDetailedHealth(): Promise<DetailedHealthResponse> {
		return this.request<DetailedHealthResponse>("/health/detailed");
	}

	async getLivenessProbe(): Promise<
		components["schemas"]["ApiResponse_Value"]
	> {
		return this.request<components["schemas"]["ApiResponse_Value"]>(
			"/health/live",
		);
	}

	async getReadinessProbe(): Promise<
		components["schemas"]["ApiResponse_Value"]
	> {
		return this.request<components["schemas"]["ApiResponse_Value"]>(
			"/health/ready",
		);
	}

	async getStartupProbe(): Promise<components["schemas"]["ApiResponse_Value"]> {
		return this.request<components["schemas"]["ApiResponse_Value"]>(
			"/health/startup",
		);
	}

	// User management endpoints
	async getUser(id: string): Promise<UserProfileResponse> {
		return this.request<UserProfileResponse>(`/users/${id}`);
	}

	// Self-service user profile management
	async updateOwnProfile(data: {
		username?: string;
		email?: string;
	}): Promise<UserProfileResponse> {
		return this.request<UserProfileResponse>("/users/me/profile", {
			method: "PUT",
			body: JSON.stringify(data),
		});
	}

	async changeOwnPassword(data: {
		current_password: string;
		new_password: string;
	}): Promise<BasicResponse> {
		return this.request<BasicResponse>("/users/me/password", {
			method: "PUT",
			body: JSON.stringify(data),
		});
	}

	async deleteOwnAccount(data: {
		password: string;
		confirmation: string;
	}): Promise<BasicResponse> {
		return this.request<BasicResponse>("/users/me", {
			method: "DELETE",
			body: JSON.stringify(data),
		});
	}

	// Moderator+ user management
	async getUsers(params?: {
		limit?: number;
		offset?: number;
	}): Promise<{ data: UserProfile[] }> {
		const searchParams = new URLSearchParams();
		if (params?.limit) searchParams.set("limit", params.limit.toString());
		if (params?.offset) searchParams.set("offset", params.offset.toString());

		const query = searchParams.toString();
		const endpoint = query ? `/users?${query}` : "/users";

		return this.request<{ data: UserProfile[] }>(endpoint);
	}

	async updateUserStatus(
		id: string,
		data: { is_active: boolean; reason?: string }
	): Promise<UserProfileResponse> {
		return this.request<UserProfileResponse>(`/users/${id}/status`, {
			method: "PUT",
			body: JSON.stringify(data),
		});
	}

	async resetUserPassword(
		id: string,
		data: { reason?: string }
	): Promise<BasicResponse> {
		return this.request<BasicResponse>(`/users/${id}/reset-password`, {
			method: "POST",
			body: JSON.stringify(data),
		});
	}

	// Admin-only user management
	async createUser(data: {
		username: string;
		email: string;
		password: string;
		role: "user" | "moderator" | "admin";
	}): Promise<UserProfileResponse> {
		return this.request<UserProfileResponse>("/users", {
			method: "POST",
			body: JSON.stringify(data),
		});
	}

	async updateUserProfile(
		id: string,
		data: {
			username?: string;
			email?: string;
			email_verified?: boolean;
		}
	): Promise<UserProfileResponse> {
		return this.request<UserProfileResponse>(`/users/${id}/profile`, {
			method: "PUT",
			body: JSON.stringify(data),
		});
	}

	async updateUserRole(
		id: string,
		data: { role: "user" | "moderator" | "admin" }
	): Promise<UserProfileResponse> {
		return this.request<UserProfileResponse>(`/users/${id}/role`, {
			method: "PUT",
			body: JSON.stringify(data),
		});
	}

	async deleteUser(
		id: string,
		data: { reason?: string }
	): Promise<BasicResponse> {
		return this.request<BasicResponse>(`/users/${id}`, {
			method: "DELETE",
			body: JSON.stringify(data),
		});
	}

	async getUserStats(): Promise<{
		data: {
			total_users: number;
			active_users: number;
			users_by_role: Record<string, number>;
			recent_registrations: number;
		};
	}> {
		return this.request<{
			data: {
				total_users: number;
				active_users: number;
				users_by_role: Record<string, number>;
				recent_registrations: number;
			};
		}>("/admin/users/stats");
	}
}

// Export singleton instance
export const apiClient = new ApiClient();

// Export class for testing/custom instances
export { ApiClient };

// Initialize token from localStorage on module load
const storedToken = localStorage.getItem("auth_token");
if (storedToken) {
	setAuthToken(storedToken);
}
