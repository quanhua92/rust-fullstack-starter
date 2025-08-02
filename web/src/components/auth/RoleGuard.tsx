import type { ReactNode } from "react";
import { useAuth } from "@/lib/auth/context";
import type {
	UserRole,
	Resource,
	Permission,
	AuthUser,
} from "@/lib/rbac/types";
import {
	hasRoleOrHigher,
	canAccessResource,
	canAccessUser,
} from "@/lib/rbac/types";

interface RoleGuardProps {
	children: ReactNode;
	/** Required role level */
	requiredRole?: UserRole;
	/** Alternative: require specific resource permission */
	resource?: Resource;
	permission?: Permission;
	/** For user-specific resources */
	targetUserId?: string;
	/** Custom permission check function */
	customCheck?: (user: AuthUser) => boolean;
	/** What to render when access is denied */
	fallback?: ReactNode;
	/** Show loading state */
	loading?: ReactNode;
}

/**
 * RoleGuard component for conditional rendering based on user permissions
 *
 * @example
 * // Require admin role
 * <RoleGuard requiredRole="admin">
 *   <AdminPanel />
 * </RoleGuard>
 *
 * @example
 * // Require user management permission
 * <RoleGuard resource="users" permission="manage">
 *   <UserManagementTools />
 * </RoleGuard>
 *
 * @example
 * // Check if user can access specific user's profile
 * <RoleGuard resource="users" permission="read" targetUserId={userId}>
 *   <UserProfile />
 * </RoleGuard>
 *
 * @example
 * // Custom permission check
 * <RoleGuard customCheck={(user) => user.id === ownerId}>
 *   <EditButton />
 * </RoleGuard>
 */
export function RoleGuard({
	children,
	requiredRole,
	resource,
	permission,
	targetUserId,
	customCheck,
	fallback = null,
	loading = null,
}: RoleGuardProps) {
	const { user, authenticated, loading: authLoading } = useAuth();

	// Show loading state while auth is loading
	if (authLoading) {
		return <>{loading}</>;
	}

	// Not authenticated - deny access
	if (!authenticated || !user) {
		return <>{fallback}</>;
	}

	let hasAccess = false;

	// Custom check takes precedence
	if (customCheck) {
		hasAccess = customCheck(user);
	}
	// Role-based check
	else if (requiredRole) {
		hasAccess = hasRoleOrHigher(user.role, requiredRole);
	}
	// Resource-based check
	else if (resource && permission) {
		hasAccess = canAccessResource(user, resource, permission, targetUserId);
	}
	// Target user access check
	else if (targetUserId) {
		hasAccess = canAccessUser(user, targetUserId);
	}
	// Default: allow access if authenticated
	else {
		hasAccess = true;
	}

	return hasAccess ? <>{children}</> : <>{fallback}</>;
}

/**
 * Hook for imperative permission checking
 */
export function usePermissions() {
	const { user, authenticated } = useAuth();

	const checkRole = (requiredRole: UserRole): boolean => {
		if (!authenticated || !user) return false;
		return hasRoleOrHigher(user.role, requiredRole);
	};

	const checkResource = (
		resource: Resource,
		permission: Permission,
		targetUserId?: string,
	): boolean => {
		if (!authenticated || !user) return false;
		return canAccessResource(user, resource, permission, targetUserId);
	};

	const checkUser = (targetUserId: string): boolean => {
		if (!authenticated || !user) return false;
		return canAccessUser(user, targetUserId);
	};

	const isAdmin = (): boolean => {
		return checkRole("admin");
	};

	const isModerator = (): boolean => {
		if (!authenticated || !user) return false;
		return user.role === "moderator";
	};

	const isModeratorOrHigher = (): boolean => {
		return checkRole("moderator");
	};

	return {
		checkRole,
		checkResource,
		checkUser,
		isAdmin,
		isModerator,
		isModeratorOrHigher,
		user,
		authenticated,
	};
}

/**
 * Higher-order component for role-based route protection
 */
export function withRoleGuard<P extends object>(
	Component: React.ComponentType<P>,
	options: Omit<RoleGuardProps, "children">,
) {
	return function ProtectedComponent(props: P) {
		return (
			<RoleGuard {...options}>
				<Component {...props} />
			</RoleGuard>
		);
	};
}
