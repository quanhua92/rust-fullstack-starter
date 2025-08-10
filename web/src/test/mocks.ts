import type { components } from "@/types/api";
// Test mocks and utilities for API testing
import { vi } from "vitest";

// Mock fetch globally for unit tests
export const createMockFetch = () => {
	const mockFetch = vi.fn();
	global.fetch = mockFetch;
	return mockFetch;
};

// Helper to create mock API responses
export const createMockResponse = <T>(data: T, status = 200): Response => {
	const response = {
		ok: status >= 200 && status < 300,
		status,
		json: vi.fn().mockResolvedValue(data),
		text: vi.fn().mockResolvedValue(JSON.stringify(data)),
	} as unknown as Response;
	return response;
};

// Mock data factories
export const mockAuthUser: components["schemas"]["AuthUser"] = {
	id: "user-123",
	username: "testuser",
	email: "test@example.com",
	role: "user",
};

export const mockUserProfile: components["schemas"]["UserProfile"] = {
	id: "user-123",
	username: "testuser",
	email: "test@example.com",
	role: "user",
	is_active: true,
	email_verified: true,
	created_at: "2024-01-01T00:00:00Z",
};

export const mockTask: components["schemas"]["TaskResponse"] = {
	id: "task-123",
	task_type: "email",
	status: "pending",
	priority: "normal",
	metadata: {},
	current_attempt: 0,
	max_attempts: 3,
	created_by: "user-123",
	created_at: "2024-01-01T00:00:00Z",
	updated_at: "2024-01-01T00:00:00Z",
	scheduled_at: null,
	started_at: null,
	completed_at: null,
	last_error: null,
};

export const mockHealthResponse: components["schemas"]["HealthResponse"] = {
	status: "healthy",
	version: "0.1.0",
	uptime: 123.456,
	documentation: {
		api_docs: "/api-docs",
		openapi_json: "/api-docs/openapi.json",
	},
};

export const mockTaskStats: components["schemas"]["TaskStats"] = {
	total: 100,
	pending: 10,
	running: 5,
	completed: 80,
	failed: 3,
	cancelled: 2,
	retrying: 0,
};

// API response wrappers
export const mockApiResponse = <T>(data: T) => ({
	success: true,
	data,
	message: null,
});

export const mockApiError = (message: string, code = "UNKNOWN_ERROR") => ({
	success: false,
	error: {
		code,
		message,
	},
	message: null,
});

// Mock localStorage
export const mockLocalStorage = () => {
	const storage: Record<string, string> = {};
	return {
		getItem: vi.fn((key: string) => storage[key] || null),
		setItem: vi.fn((key: string, value: string) => {
			storage[key] = value;
		}),
		removeItem: vi.fn((key: string) => {
			delete storage[key];
		}),
		clear: vi.fn(() => {
			for (const key in storage) {
				delete storage[key];
			}
		}),
	};
};

// Integration test utilities
export const testServerConfig = {
	baseUrl: "http://localhost:3000/api/v1",
	timeout: 10000,
};

// Helper to create test users for integration tests
export const createTestUser = () => ({
	username: `testuser_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
	email: `test_${Date.now()}_${Math.random().toString(36).substr(2, 9)}@example.com`,
	password: "TestPassword123!",
});

// Helper to wait for server to be ready
export const waitForServer = async (url: string, maxAttempts = 30) => {
	for (let i = 0; i < maxAttempts; i++) {
		try {
			const response = await fetch(`${url}/health`);
			if (response.ok) {
				return true;
			}
		} catch {
			// Server not ready, continue waiting
		}
		await new Promise((resolve) => setTimeout(resolve, 1000));
	}
	throw new Error(`Server not ready after ${maxAttempts} attempts`);
};
