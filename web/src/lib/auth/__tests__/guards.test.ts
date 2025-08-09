import { mockApiResponse, mockAuthUser } from "@/test/mocks";
import { redirect } from "@tanstack/react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
	requireAdmin,
	requireAuth,
	requireModeratorOrHigher,
	requireRole,
} from "../guards";

// Mock dependencies
vi.mock("@tanstack/react-router", () => ({
	redirect: vi.fn(),
}));

vi.mock("@/lib/api/client", () => ({
	apiClient: {
		getCurrentUser: vi.fn(),
	},
	getAuthToken: vi.fn(),
}));

import { apiClient, getAuthToken } from "@/lib/api/client";

describe("Auth Guards", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe("requireAuth", () => {
		it("should redirect to login if no token", async () => {
			vi.mocked(getAuthToken).mockReturnValue(null);

			await expect(requireAuth()).rejects.toThrow();
			expect(redirect).toHaveBeenCalledWith({
				to: "/auth/login",
			});
		});

		it("should return user data if authenticated", async () => {
			vi.mocked(getAuthToken).mockReturnValue("valid-token");
			vi.mocked(apiClient.getCurrentUser).mockResolvedValue(
				mockApiResponse(mockAuthUser),
			);

			const result = await requireAuth();

			expect(result).toEqual(mockAuthUser);
			expect(apiClient.getCurrentUser).toHaveBeenCalled();
		});

		it("should redirect to login if no user data", async () => {
			vi.mocked(getAuthToken).mockReturnValue("token");
			vi.mocked(apiClient.getCurrentUser).mockResolvedValue({
				...mockApiResponse(mockAuthUser),
				data: undefined,
			});

			await expect(requireAuth()).rejects.toThrow();
			expect(redirect).toHaveBeenCalledWith({
				to: "/auth/login",
			});
		});

		it("should redirect to login if API call fails", async () => {
			vi.mocked(getAuthToken).mockReturnValue("token");
			vi.mocked(apiClient.getCurrentUser).mockRejectedValue(
				new Error("Unauthorized"),
			);

			await expect(requireAuth()).rejects.toThrow();
			expect(redirect).toHaveBeenCalledWith({
				to: "/auth/login",
			});
		});

		it("should log errors when API call fails", async () => {
			const consoleSpy = vi
				.spyOn(console, "error")
				.mockImplementation(() => {});
			vi.mocked(getAuthToken).mockReturnValue("token");
			vi.mocked(apiClient.getCurrentUser).mockRejectedValue(
				new Error("Network error"),
			);

			await expect(requireAuth()).rejects.toThrow();
			expect(consoleSpy).toHaveBeenCalledWith(
				"Authentication check failed:",
				new Error("Network error"),
			);

			consoleSpy.mockRestore();
		});
	});

	describe("requireModeratorOrHigher", () => {
		it("should return user if user is moderator", async () => {
			const moderatorUser = { ...mockAuthUser, role: "moderator" as const };
			vi.mocked(getAuthToken).mockReturnValue("token");
			vi.mocked(apiClient.getCurrentUser).mockResolvedValue(
				mockApiResponse(moderatorUser),
			);

			const result = await requireModeratorOrHigher();

			expect(result).toEqual(moderatorUser);
		});

		it("should return user if user is admin", async () => {
			const adminUser = { ...mockAuthUser, role: "admin" as const };
			vi.mocked(getAuthToken).mockReturnValue("token");
			vi.mocked(apiClient.getCurrentUser).mockResolvedValue(
				mockApiResponse(adminUser),
			);

			const result = await requireModeratorOrHigher();

			expect(result).toEqual(adminUser);
		});

		it("should redirect to admin if user is regular user", async () => {
			const regularUser = { ...mockAuthUser, role: "user" as const };
			vi.mocked(getAuthToken).mockReturnValue("token");
			vi.mocked(apiClient.getCurrentUser).mockResolvedValue(
				mockApiResponse(regularUser),
			);

			await expect(requireModeratorOrHigher()).rejects.toThrow();
			expect(redirect).toHaveBeenCalledWith({
				to: "/admin",
			});
		});

		it("should redirect to login if not authenticated", async () => {
			vi.mocked(getAuthToken).mockReturnValue(null);

			await expect(requireModeratorOrHigher()).rejects.toThrow();
			expect(redirect).toHaveBeenCalledWith({
				to: "/auth/login",
			});
		});
	});

	describe("requireAdmin", () => {
		it("should return user if user is admin", async () => {
			const adminUser = { ...mockAuthUser, role: "admin" as const };
			vi.mocked(getAuthToken).mockReturnValue("token");
			vi.mocked(apiClient.getCurrentUser).mockResolvedValue(
				mockApiResponse(adminUser),
			);

			const result = await requireAdmin();

			expect(result).toEqual(adminUser);
		});

		it("should redirect with error if user is moderator", async () => {
			const moderatorUser = { ...mockAuthUser, role: "moderator" as const };
			vi.mocked(getAuthToken).mockReturnValue("token");
			vi.mocked(apiClient.getCurrentUser).mockResolvedValue(
				mockApiResponse(moderatorUser),
			);

			await expect(requireAdmin()).rejects.toThrow();
			expect(redirect).toHaveBeenCalledWith({
				to: "/admin",
				search: {
					error: "insufficient_permissions",
				},
			});
		});

		it("should redirect with error if user is regular user", async () => {
			const regularUser = { ...mockAuthUser, role: "user" as const };
			vi.mocked(getAuthToken).mockReturnValue("token");
			vi.mocked(apiClient.getCurrentUser).mockResolvedValue(
				mockApiResponse(regularUser),
			);

			await expect(requireAdmin()).rejects.toThrow();
			expect(redirect).toHaveBeenCalledWith({
				to: "/admin",
				search: {
					error: "insufficient_permissions",
				},
			});
		});
	});

	describe("requireRole", () => {
		it("should return user if user has exact required role", async () => {
			const moderatorUser = { ...mockAuthUser, role: "moderator" as const };
			vi.mocked(getAuthToken).mockReturnValue("token");
			vi.mocked(apiClient.getCurrentUser).mockResolvedValue(
				mockApiResponse(moderatorUser),
			);

			const guardFunction = requireRole("moderator");
			const result = await guardFunction();

			expect(result).toEqual(moderatorUser);
		});

		it("should return user if user has higher role than required", async () => {
			const adminUser = { ...mockAuthUser, role: "admin" as const };
			vi.mocked(getAuthToken).mockReturnValue("token");
			vi.mocked(apiClient.getCurrentUser).mockResolvedValue(
				mockApiResponse(adminUser),
			);

			const guardFunction = requireRole("user");
			const result = await guardFunction();

			expect(result).toEqual(adminUser);
		});

		it("should redirect if user has lower role than required", async () => {
			const regularUser = { ...mockAuthUser, role: "user" as const };
			vi.mocked(getAuthToken).mockReturnValue("token");
			vi.mocked(apiClient.getCurrentUser).mockResolvedValue(
				mockApiResponse(regularUser),
			);

			const guardFunction = requireRole("admin");
			await expect(guardFunction()).rejects.toThrow();
			expect(redirect).toHaveBeenCalledWith({
				to: "/admin",
			});
		});

		it("should create a new guard function for each role", () => {
			const userGuard = requireRole("user");
			const moderatorGuard = requireRole("moderator");
			const adminGuard = requireRole("admin");

			expect(typeof userGuard).toBe("function");
			expect(typeof moderatorGuard).toBe("function");
			expect(typeof adminGuard).toBe("function");
			expect(userGuard).not.toBe(moderatorGuard);
			expect(moderatorGuard).not.toBe(adminGuard);
		});
	});
});
