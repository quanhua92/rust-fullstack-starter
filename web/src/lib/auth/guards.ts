import { apiClient, getAuthToken } from "@/lib/api/client";
import type { UserRole } from "@/lib/rbac/types";
import { hasRoleOrHigher } from "@/lib/rbac/types";
import { redirect } from "@tanstack/react-router";

/**
 * Route guard that requires authentication and a specific role or higher
 */
export const requireAuth = async () => {
	const token = getAuthToken();
	if (!token) {
		throw redirect({
			to: "/auth/login",
		});
	}

	try {
		const response = await apiClient.getCurrentUser();
		if (!response.data) {
			throw redirect({
				to: "/auth/login",
			});
		}
		return response.data;
	} catch (error) {
		console.error("Authentication check failed:", error);
		throw redirect({
			to: "/auth/login",
		});
	}
};

/**
 * Route guard that requires moderator role or higher
 */
export const requireModeratorOrHigher = async () => {
	const user = await requireAuth();

	if (!hasRoleOrHigher(user.role, "moderator")) {
		throw redirect({
			to: "/admin",
		});
	}

	return user;
};

/**
 * Route guard that requires admin role
 */
export const requireAdmin = async () => {
	const user = await requireAuth();

	if (!hasRoleOrHigher(user.role, "admin")) {
		throw redirect({
			to: "/admin",
			search: {
				error: "insufficient_permissions",
			},
		});
	}

	return user;
};

/**
 * Generic role-based route guard
 */
export const requireRole = (requiredRole: UserRole) => async () => {
	const user = await requireAuth();

	if (!hasRoleOrHigher(user.role, requiredRole)) {
		throw redirect({
			to: "/admin",
		});
	}

	return user;
};
