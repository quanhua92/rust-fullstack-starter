// RBAC types and utilities
import type { components } from "@/types/api";

export type UserRole = components["schemas"]["UserRole"];

export type AuthUser = {
	id: string;
	username: string;
	email: string;
	role: UserRole;
};

// Role hierarchy: User < Moderator < Admin
export const ROLE_HIERARCHY: Record<UserRole, number> = {
	user: 1,
	moderator: 2,
	admin: 3,
} as const;

// Permission types for resource access
export type Resource = "users" | "tasks" | "admin" | "self";
export type Permission = "read" | "write" | "delete" | "manage";

/**
 * Check if a role has the required level or higher
 */
export function hasRoleOrHigher(
	userRole: UserRole,
	requiredRole: UserRole,
): boolean {
	return ROLE_HIERARCHY[userRole] >= ROLE_HIERARCHY[requiredRole];
}

/**
 * Check if user is admin
 */
export function isAdmin(user: AuthUser): boolean {
	return user.role === "admin";
}

/**
 * Check if user is moderator or higher
 */
export function isModeratorOrHigher(user: AuthUser): boolean {
	return hasRoleOrHigher(user.role, "moderator");
}

/**
 * Check if user can access another user's resources
 */
export function canAccessUser(
	currentUser: AuthUser,
	targetUserId: string,
): boolean {
	// Users can always access their own resources
	if (currentUser.id === targetUserId) {
		return true;
	}

	// Moderator and admin can access other users
	return isModeratorOrHigher(currentUser);
}

/**
 * Check if user can perform an action on a resource
 */
export function canAccessResource(
	user: AuthUser,
	resource: Resource,
	permission: Permission,
	targetUserId?: string,
): boolean {
	// Admin can do everything
	if (isAdmin(user)) {
		return true;
	}

	switch (resource) {
		case "admin":
			// Only admin can access admin resources
			return isAdmin(user);

		case "self":
			// Users can manage their own resources
			return true;

		case "users":
			if (permission === "read") {
				// Moderator+ can read all users, users can read their own
				return (
					isModeratorOrHigher(user) ||
					(targetUserId && user.id === targetUserId)
				);
			}
			if (permission === "write" || permission === "manage") {
				// Only admin can create/manage users, users can update themselves
				return isAdmin(user) || (targetUserId && user.id === targetUserId);
			}
			if (permission === "delete") {
				// Only admin can delete users, users can delete themselves
				return isAdmin(user) || (targetUserId && user.id === targetUserId);
			}
			return false;

		case "tasks":
			if (permission === "read") {
				// Moderator+ can read all tasks, users can read their own
				return isModeratorOrHigher(user);
			}
			if (permission === "write") {
				// All authenticated users can create tasks
				return true;
			}
			if (permission === "manage" || permission === "delete") {
				// Moderator+ can manage all tasks, users can manage their own
				return isModeratorOrHigher(user);
			}
			return false;

		default:
			return false;
	}
}

/**
 * Get user role display name
 */
export function getRoleDisplayName(role: UserRole): string {
	switch (role) {
		case "user":
			return "User";
		case "moderator":
			return "Moderator";
		case "admin":
			return "Administrator";
		default:
			return "Unknown";
	}
}

/**
 * Get user role color for UI
 */
export function getRoleColor(role: UserRole): string {
	switch (role) {
		case "user":
			return "blue";
		case "moderator":
			return "purple";
		case "admin":
			return "red";
		default:
			return "gray";
	}
}

/**
 * Get available roles for a user to assign (can only assign equal or lower roles)
 */
export function getAssignableRoles(currentUserRole: UserRole): UserRole[] {
	const allRoles: UserRole[] = ["user", "moderator", "admin"];
	const currentLevel = ROLE_HIERARCHY[currentUserRole];

	return allRoles.filter((role) => ROLE_HIERARCHY[role] <= currentLevel);
}

/**
 * Check if user can assign a specific role
 */
export function canAssignRole(
	currentUserRole: UserRole,
	targetRole: UserRole,
): boolean {
	return ROLE_HIERARCHY[currentUserRole] >= ROLE_HIERARCHY[targetRole];
}