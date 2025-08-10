import { describe, expect, it } from "vitest";
import {
	ROLE_HIERARCHY,
	canAccessResource,
	canAccessUser,
	canAssignRole,
	getAssignableRoles,
	getRoleColor,
	getRoleColorClasses,
	getRoleDisplayName,
	hasRoleOrHigher,
	isAdmin,
	isModeratorOrHigher,
} from "../types";
import type { AuthUser, UserRole } from "../types";

describe("RBAC Types and Utilities", () => {
	const mockUsers: Record<UserRole, AuthUser> = {
		user: {
			id: "user-1",
			username: "regularuser",
			email: "user@example.com",
			role: "user",
		},
		moderator: {
			id: "mod-1",
			username: "moderator",
			email: "mod@example.com",
			role: "moderator",
		},
		admin: {
			id: "admin-1",
			username: "administrator",
			email: "admin@example.com",
			role: "admin",
		},
	};

	describe("ROLE_HIERARCHY", () => {
		it("should have correct role hierarchy values", () => {
			expect(ROLE_HIERARCHY.user).toBe(1);
			expect(ROLE_HIERARCHY.moderator).toBe(2);
			expect(ROLE_HIERARCHY.admin).toBe(3);
		});

		it("should maintain proper ordering", () => {
			expect(ROLE_HIERARCHY.user).toBeLessThan(ROLE_HIERARCHY.moderator);
			expect(ROLE_HIERARCHY.moderator).toBeLessThan(ROLE_HIERARCHY.admin);
		});
	});

	describe("hasRoleOrHigher", () => {
		it("should return true for exact role match", () => {
			expect(hasRoleOrHigher("user", "user")).toBe(true);
			expect(hasRoleOrHigher("moderator", "moderator")).toBe(true);
			expect(hasRoleOrHigher("admin", "admin")).toBe(true);
		});

		it("should return true for higher roles", () => {
			expect(hasRoleOrHigher("moderator", "user")).toBe(true);
			expect(hasRoleOrHigher("admin", "user")).toBe(true);
			expect(hasRoleOrHigher("admin", "moderator")).toBe(true);
		});

		it("should return false for lower roles", () => {
			expect(hasRoleOrHigher("user", "moderator")).toBe(false);
			expect(hasRoleOrHigher("user", "admin")).toBe(false);
			expect(hasRoleOrHigher("moderator", "admin")).toBe(false);
		});
	});

	describe("isAdmin", () => {
		it("should return true for admin users", () => {
			expect(isAdmin(mockUsers.admin)).toBe(true);
		});

		it("should return false for non-admin users", () => {
			expect(isAdmin(mockUsers.user)).toBe(false);
			expect(isAdmin(mockUsers.moderator)).toBe(false);
		});
	});

	describe("isModeratorOrHigher", () => {
		it("should return true for moderator and admin", () => {
			expect(isModeratorOrHigher(mockUsers.moderator)).toBe(true);
			expect(isModeratorOrHigher(mockUsers.admin)).toBe(true);
		});

		it("should return false for regular users", () => {
			expect(isModeratorOrHigher(mockUsers.user)).toBe(false);
		});
	});

	describe("canAccessUser", () => {
		it("should allow users to access their own resources", () => {
			expect(canAccessUser(mockUsers.user, mockUsers.user.id)).toBe(true);
			expect(canAccessUser(mockUsers.moderator, mockUsers.moderator.id)).toBe(
				true,
			);
			expect(canAccessUser(mockUsers.admin, mockUsers.admin.id)).toBe(true);
		});

		it("should allow moderator to access other users", () => {
			expect(canAccessUser(mockUsers.moderator, mockUsers.user.id)).toBe(true);
			expect(canAccessUser(mockUsers.moderator, "other-user-id")).toBe(true);
		});

		it("should allow admin to access other users", () => {
			expect(canAccessUser(mockUsers.admin, mockUsers.user.id)).toBe(true);
			expect(canAccessUser(mockUsers.admin, mockUsers.moderator.id)).toBe(true);
			expect(canAccessUser(mockUsers.admin, "other-user-id")).toBe(true);
		});

		it("should not allow regular users to access other users", () => {
			expect(canAccessUser(mockUsers.user, mockUsers.moderator.id)).toBe(false);
			expect(canAccessUser(mockUsers.user, mockUsers.admin.id)).toBe(false);
			expect(canAccessUser(mockUsers.user, "other-user-id")).toBe(false);
		});
	});

	describe("canAccessResource", () => {
		describe("admin resource", () => {
			it("should only allow admin access", () => {
				expect(canAccessResource(mockUsers.admin, "admin", "read")).toBe(true);
				expect(canAccessResource(mockUsers.admin, "admin", "write")).toBe(true);
				expect(canAccessResource(mockUsers.moderator, "admin", "read")).toBe(
					false,
				);
				expect(canAccessResource(mockUsers.user, "admin", "read")).toBe(false);
			});
		});

		describe("self resource", () => {
			it("should allow all users to access self", () => {
				expect(canAccessResource(mockUsers.user, "self", "read")).toBe(true);
				expect(canAccessResource(mockUsers.user, "self", "write")).toBe(true);
				expect(canAccessResource(mockUsers.moderator, "self", "manage")).toBe(
					true,
				);
				expect(canAccessResource(mockUsers.admin, "self", "delete")).toBe(true);
			});
		});

		describe("users resource", () => {
			it("should allow moderator+ to read all users", () => {
				expect(canAccessResource(mockUsers.moderator, "users", "read")).toBe(
					true,
				);
				expect(canAccessResource(mockUsers.admin, "users", "read")).toBe(true);
			});

			it("should allow users to read their own profile", () => {
				expect(
					canAccessResource(mockUsers.user, "users", "read", mockUsers.user.id),
				).toBe(true);
			});

			it("should not allow users to read other users", () => {
				expect(canAccessResource(mockUsers.user, "users", "read")).toBe(false);
				expect(
					canAccessResource(mockUsers.user, "users", "read", "other-id"),
				).toBe(false);
			});

			it("should allow admin to manage all users", () => {
				expect(canAccessResource(mockUsers.admin, "users", "write")).toBe(true);
				expect(canAccessResource(mockUsers.admin, "users", "manage")).toBe(
					true,
				);
				expect(canAccessResource(mockUsers.admin, "users", "delete")).toBe(
					true,
				);
			});

			it("should allow users to manage themselves", () => {
				expect(
					canAccessResource(
						mockUsers.user,
						"users",
						"write",
						mockUsers.user.id,
					),
				).toBe(true);
				expect(
					canAccessResource(
						mockUsers.user,
						"users",
						"manage",
						mockUsers.user.id,
					),
				).toBe(true);
				expect(
					canAccessResource(
						mockUsers.user,
						"users",
						"delete",
						mockUsers.user.id,
					),
				).toBe(true);
			});

			it("should not allow moderator to manage other users", () => {
				expect(canAccessResource(mockUsers.moderator, "users", "write")).toBe(
					false,
				);
				expect(canAccessResource(mockUsers.moderator, "users", "manage")).toBe(
					false,
				);
				expect(canAccessResource(mockUsers.moderator, "users", "delete")).toBe(
					false,
				);
			});
		});

		describe("tasks resource", () => {
			it("should allow moderator+ to read all tasks", () => {
				expect(canAccessResource(mockUsers.moderator, "tasks", "read")).toBe(
					true,
				);
				expect(canAccessResource(mockUsers.admin, "tasks", "read")).toBe(true);
			});

			it("should allow users to read their own tasks", () => {
				expect(
					canAccessResource(mockUsers.user, "tasks", "read", mockUsers.user.id),
				).toBe(true);
			});

			it("should not allow users to read all tasks", () => {
				expect(canAccessResource(mockUsers.user, "tasks", "read")).toBe(false);
			});

			it("should allow all users to create tasks", () => {
				expect(canAccessResource(mockUsers.user, "tasks", "write")).toBe(true);
				expect(canAccessResource(mockUsers.moderator, "tasks", "write")).toBe(
					true,
				);
				expect(canAccessResource(mockUsers.admin, "tasks", "write")).toBe(true);
			});

			it("should allow moderator+ to manage all tasks", () => {
				expect(canAccessResource(mockUsers.moderator, "tasks", "manage")).toBe(
					true,
				);
				expect(canAccessResource(mockUsers.moderator, "tasks", "delete")).toBe(
					true,
				);
				expect(canAccessResource(mockUsers.admin, "tasks", "manage")).toBe(
					true,
				);
				expect(canAccessResource(mockUsers.admin, "tasks", "delete")).toBe(
					true,
				);
			});

			it("should allow users to manage their own tasks", () => {
				expect(
					canAccessResource(
						mockUsers.user,
						"tasks",
						"manage",
						mockUsers.user.id,
					),
				).toBe(true);
				expect(
					canAccessResource(
						mockUsers.user,
						"tasks",
						"delete",
						mockUsers.user.id,
					),
				).toBe(true);
			});

			it("should not allow users to manage other users' tasks", () => {
				expect(canAccessResource(mockUsers.user, "tasks", "manage")).toBe(
					false,
				);
				expect(canAccessResource(mockUsers.user, "tasks", "delete")).toBe(
					false,
				);
			});
		});
	});

	describe("getRoleDisplayName", () => {
		it("should return correct display names", () => {
			expect(getRoleDisplayName("user")).toBe("User");
			expect(getRoleDisplayName("moderator")).toBe("Moderator");
			expect(getRoleDisplayName("admin")).toBe("Administrator");
		});

		it("should handle unknown roles", () => {
			expect(getRoleDisplayName("unknown" as UserRole)).toBe("Unknown");
		});
	});

	describe("getRoleColorClasses", () => {
		it("should return correct color classes for each role", () => {
			const userColors = getRoleColorClasses("user");
			expect(userColors.text).toContain("blue-600");
			expect(userColors.border).toContain("blue-300");

			const modColors = getRoleColorClasses("moderator");
			expect(modColors.text).toContain("purple-600");
			expect(modColors.border).toContain("purple-300");

			const adminColors = getRoleColorClasses("admin");
			expect(adminColors.text).toContain("red-600");
			expect(adminColors.border).toContain("red-300");
		});

		it("should include dark mode variants", () => {
			const colors = getRoleColorClasses("user");
			expect(colors.text).toContain("dark:text-blue-400");
			expect(colors.border).toContain("dark:border-blue-600");
		});

		it("should handle unknown roles with gray colors", () => {
			const colors = getRoleColorClasses("unknown" as UserRole);
			expect(colors.text).toContain("gray-600");
			expect(colors.border).toContain("gray-300");
		});
	});

	describe("getRoleColor (deprecated)", () => {
		it("should return correct color strings", () => {
			expect(getRoleColor("user")).toBe("blue-600");
			expect(getRoleColor("moderator")).toBe("purple-600");
			expect(getRoleColor("admin")).toBe("red-600");
		});

		it("should handle unknown roles", () => {
			expect(getRoleColor("unknown" as UserRole)).toBe("gray-600");
		});
	});

	describe("getAssignableRoles", () => {
		it("should return all roles for admin", () => {
			const roles = getAssignableRoles("admin");
			expect(roles).toEqual(["user", "moderator", "admin"]);
		});

		it("should return user and moderator for moderator", () => {
			const roles = getAssignableRoles("moderator");
			expect(roles).toEqual(["user", "moderator"]);
		});

		it("should return only user for regular user", () => {
			const roles = getAssignableRoles("user");
			expect(roles).toEqual(["user"]);
		});
	});

	describe("canAssignRole", () => {
		it("should allow admin to assign any role", () => {
			expect(canAssignRole("admin", "user")).toBe(true);
			expect(canAssignRole("admin", "moderator")).toBe(true);
			expect(canAssignRole("admin", "admin")).toBe(true);
		});

		it("should allow moderator to assign user and moderator roles", () => {
			expect(canAssignRole("moderator", "user")).toBe(true);
			expect(canAssignRole("moderator", "moderator")).toBe(true);
			expect(canAssignRole("moderator", "admin")).toBe(false);
		});

		it("should only allow user to assign user role", () => {
			expect(canAssignRole("user", "user")).toBe(true);
			expect(canAssignRole("user", "moderator")).toBe(false);
			expect(canAssignRole("user", "admin")).toBe(false);
		});
	});
});
