import { AdminLayout } from "@/components/layout/AdminLayout";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import {
	Pagination,
	PaginationContent,
	PaginationEllipsis,
	PaginationItem,
	PaginationLink,
	PaginationNext,
	PaginationPrevious,
} from "@/components/ui/pagination";
import { apiClient } from "@/lib/api/client";
import { useAuth } from "@/lib/auth/context";
import { RoleGuard } from "@/components/auth/RoleGuard";
import {
	getRoleDisplayName,
	getRoleColorClasses,
	type UserRole,
} from "@/lib/rbac/types";
import { useToast } from "@/hooks/use-toast";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Link } from "@tanstack/react-router";
import {
	Eye,
	MoreHorizontal,
	Plus,
	Search,
	Shield,
	Trash2,
	UserCheck,
	UserX,
	Key,
} from "lucide-react";
import { useState } from "react";

function UsersPage() {
	const [searchTerm, setSearchTerm] = useState("");
	const [currentPage, setCurrentPage] = useState(1);
	const { user: currentUser, isModeratorOrHigher } = useAuth();
	const { toast } = useToast();
	const queryClient = useQueryClient();

	const pageSize = 10; // Users per page
	const offset = (currentPage - 1) * pageSize;

	// Fetch users list with pagination and RBAC check
	const { data: usersResponse, isLoading } = useQuery({
		queryKey: ["admin", "users", currentPage, searchTerm],
		queryFn: async () => {
			const response = await apiClient.getUsers({
				limit: pageSize,
				offset,
			});
			return response;
		},
		enabled: isModeratorOrHigher(), // Only fetch if user has permission
	});

	const users = usersResponse?.data || [];

	// For search functionality, we'll need to implement server-side search
	// For now, keeping client-side search on current page
	const filteredUsers = users.filter(
		(user) =>
			user.username.toLowerCase().includes(searchTerm.toLowerCase()) ||
			user.email.toLowerCase().includes(searchTerm.toLowerCase()),
	);

	// Calculate total pages (since API doesn't return total count, we'll estimate)
	// In production, the API should return total count for accurate pagination
	const hasNextPage = users.length === pageSize;
	const totalPages = hasNextPage ? currentPage + 1 : currentPage;

	// User status update mutation (Moderator+)
	const updateUserStatusMutation = useMutation({
		mutationFn: ({
			id,
			is_active,
			reason,
		}: { id: string; is_active: boolean; reason?: string }) =>
			apiClient.updateUserStatus(id, { is_active, reason }),
		onSuccess: () => {
			toast({
				title: "User status updated",
				description: "User status has been updated successfully.",
			});
			queryClient.invalidateQueries({ queryKey: ["admin", "users"] });
		},
		onError: (error: Error) => {
			toast({
				title: "Failed to update user status",
				description:
					error.message || "An error occurred while updating user status.",
				variant: "destructive",
			});
		},
	});

	// Generate secure random password using crypto API
	const generateSecurePassword = (length: number = 12): string => {
		const charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
		const values = new Uint32Array(length);
		window.crypto.getRandomValues(values);
		let password = "";
		for (let i = 0; i < length; i++) {
			password += charset[values[i] % charset.length];
		}
		return password;
	};

	// Password reset mutation (Moderator+)
	const resetPasswordMutation = useMutation({
		mutationFn: ({ id, reason }: { id: string; reason?: string }) => {
			const newPassword = generateSecurePassword();
			return apiClient.resetUserPassword(id, { new_password: newPassword, reason }).then(response => ({
				...response,
				newPassword, // Return the generated password for display
			}));
		},
		onSuccess: (data) => {
			toast({
				title: "Password reset successful",
				description: `New password: ${data.newPassword} (Please copy this and share with the user)`,
			});
		},
		onError: (error: Error) => {
			toast({
				title: "Failed to reset password",
				description:
					error.message || "An error occurred while resetting password.",
				variant: "destructive",
			});
		},
	});

	// User deletion mutation (Admin only)
	const deleteUserMutation = useMutation({
		mutationFn: ({ id, reason }: { id: string; reason?: string }) =>
			apiClient.deleteUser(id, { reason }),
		onSuccess: () => {
			toast({
				title: "User deleted",
				description: "User has been deleted successfully.",
			});
			queryClient.invalidateQueries({ queryKey: ["admin", "users"] });
		},
		onError: (error: Error) => {
			toast({
				title: "Failed to delete user",
				description: error.message || "An error occurred while deleting user.",
				variant: "destructive",
			});
		},
	});

	const handleUserAction = (userId: string, action: string) => {
		switch (action) {
			case "activate":
				updateUserStatusMutation.mutate({ id: userId, is_active: true });
				break;
			case "deactivate":
				updateUserStatusMutation.mutate({
					id: userId,
					is_active: false,
					reason: "Deactivated by admin",
				});
				break;
			case "reset-password":
				resetPasswordMutation.mutate({
					id: userId,
					reason: "Password reset by admin",
				});
				break;
			case "delete":
				if (
					window.confirm(
						"Are you sure you want to delete this user? This action cannot be undone.",
					)
				) {
					deleteUserMutation.mutate({ id: userId, reason: "Deleted by admin" });
				}
				break;
		}
	};

	const getUserRoleBadge = (role: string) => {
		const colorClasses = getRoleColorClasses(role as UserRole);
		return (
			<Badge
				variant="outline"
				className={`${colorClasses.text} ${colorClasses.border}`}
			>
				{getRoleDisplayName(role as UserRole)}
			</Badge>
		);
	};

	const getStatusBadge = (isActive: boolean) => {
		return (
			<Badge variant={isActive ? "default" : "secondary"}>
				{isActive ? "Active" : "Inactive"}
			</Badge>
		);
	};

	// Show error message if user doesn't have permission
	if (!isModeratorOrHigher()) {
		return (
			<AdminLayout>
				<div className="flex items-center justify-center min-h-[400px]">
					<div className="text-center space-y-4">
						<Shield className="h-12 w-12 text-muted-foreground mx-auto" />
						<div>
							<h3 className="text-lg font-semibold">Access Denied</h3>
							<p className="text-muted-foreground">
								You need moderator or admin privileges to view user management.
							</p>
						</div>
					</div>
				</div>
			</AdminLayout>
		);
	}

	return (
		<AdminLayout>
			<div className="space-y-6">
				{/* Header */}
				<div className="flex items-center justify-between">
					<div>
						<h1 className="text-3xl font-bold tracking-tight">Users</h1>
						<p className="text-muted-foreground">
							Manage user accounts, roles, and permissions
						</p>
					</div>
					<RoleGuard requiredRole="admin">
						<Button asChild>
							<Link to="/admin/users/new">
								<Plus className="h-4 w-4 mr-2" />
								Add User
							</Link>
						</Button>
					</RoleGuard>
				</div>

				{/* Current Page Stats */}
				<div className="grid gap-4 md:grid-cols-3">
					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">
								Current Page
							</CardTitle>
							<UserCheck className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">{users?.length || 0}</div>
							<p className="text-xs text-muted-foreground">
								Users on page {currentPage}
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">
								Active (Page)
							</CardTitle>
							<UserCheck className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">
								{users?.filter((u) => u.is_active).length || 0}
							</div>
							<p className="text-xs text-muted-foreground">
								Active on this page
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">
								Admins (Page)
							</CardTitle>
							<Shield className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">
								{users?.filter((u) => u.role === "admin").length || 0}
							</div>
							<p className="text-xs text-muted-foreground">
								Admins on this page
							</p>
						</CardContent>
					</Card>
				</div>

				{/* Search and Filters */}
				<div className="flex items-center space-x-2">
					<div className="relative flex-1 max-w-sm">
						<Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
						<Input
							placeholder="Search users..."
							value={searchTerm}
							onChange={(e) => setSearchTerm(e.target.value)}
							className="pl-8"
						/>
					</div>
				</div>

				{/* Users Table */}
				<Card>
					<CardContent className="p-0">
						<Table>
							<TableHeader>
								<TableRow>
									<TableHead>User</TableHead>
									<TableHead>Role</TableHead>
									<TableHead>Status</TableHead>
									<TableHead>Created</TableHead>
									<TableHead>Last Login</TableHead>
									<TableHead className="w-[70px]">Actions</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{isLoading ? (
									<TableRow>
										<TableCell colSpan={6} className="text-center py-4">
											Loading users...
										</TableCell>
									</TableRow>
								) : filteredUsers.length === 0 ? (
									<TableRow>
										<TableCell colSpan={6} className="text-center py-4">
											{searchTerm
												? "No users match your search."
												: "No users found."}
										</TableCell>
									</TableRow>
								) : (
									filteredUsers.map((user) => (
										<TableRow key={user.id}>
											<TableCell>
												<div className="flex items-center space-x-2">
													<div>
														<div className="font-medium">{user.username}</div>
														<div className="text-sm text-muted-foreground">
															{user.email}
														</div>
													</div>
												</div>
											</TableCell>
											<TableCell>{getUserRoleBadge(user.role)}</TableCell>
											<TableCell>{getStatusBadge(user.is_active)}</TableCell>
											<TableCell>
												<div className="text-sm">
													{user.created_at
														? new Date(user.created_at).toLocaleDateString()
														: "Unknown"}
												</div>
											</TableCell>
											<TableCell>
												<div className="text-sm text-muted-foreground">
													{user.last_login_at
														? new Date(user.last_login_at).toLocaleDateString()
														: "Never"}
												</div>
											</TableCell>
											<TableCell>
												<DropdownMenu>
													<DropdownMenuTrigger asChild>
														<Button variant="ghost" className="h-8 w-8 p-0">
															<MoreHorizontal className="h-4 w-4" />
														</Button>
													</DropdownMenuTrigger>
													<DropdownMenuContent align="end">
														<DropdownMenuItem asChild>
															<Link
																to="/admin/users/$userId"
																params={{ userId: user.id }}
															>
																<Eye className="mr-2 h-4 w-4" />
																View Details
															</Link>
														</DropdownMenuItem>

														{/* Moderator+ actions */}
														<RoleGuard requiredRole="moderator">
															{user.id !== currentUser?.id && (
																<>
																	<DropdownMenuSeparator />
																	<DropdownMenuItem
																		onClick={() =>
																			handleUserAction(
																				user.id,
																				user.is_active
																					? "deactivate"
																					: "activate",
																			)
																		}
																	>
																		{user.is_active ? (
																			<>
																				<UserX className="mr-2 h-4 w-4" />
																				Deactivate
																			</>
																		) : (
																			<>
																				<UserCheck className="mr-2 h-4 w-4" />
																				Activate
																			</>
																		)}
																	</DropdownMenuItem>

																	<DropdownMenuItem
																		onClick={() =>
																			handleUserAction(
																				user.id,
																				"reset-password",
																			)
																		}
																	>
																		<Key className="mr-2 h-4 w-4" />
																		Reset Password
																	</DropdownMenuItem>
																</>
															)}
														</RoleGuard>

														{/* Admin-only actions */}
														<RoleGuard requiredRole="admin">
															{user.id !== currentUser?.id && (
																<>
																	<DropdownMenuSeparator />
																	<DropdownMenuItem
																		onClick={() =>
																			handleUserAction(user.id, "delete")
																		}
																		className="text-red-600"
																	>
																		<Trash2 className="mr-2 h-4 w-4" />
																		Delete User
																	</DropdownMenuItem>
																</>
															)}
														</RoleGuard>
													</DropdownMenuContent>
												</DropdownMenu>
											</TableCell>
										</TableRow>
									))
								)}
							</TableBody>
						</Table>
					</CardContent>
				</Card>

				{/* Pagination */}
				{!isLoading && users.length > 0 && (
					<div className="flex items-center justify-between">
						<div className="text-sm text-muted-foreground">
							Showing {(currentPage - 1) * pageSize + 1} to{" "}
							{Math.min(
								currentPage * pageSize,
								(currentPage - 1) * pageSize + users.length,
							)}{" "}
							of users
						</div>
						<Pagination>
							<PaginationContent>
								<PaginationItem>
									<PaginationPrevious
										onClick={() => setCurrentPage(Math.max(1, currentPage - 1))}
										className={
											currentPage === 1
												? "pointer-events-none opacity-50"
												: "cursor-pointer"
										}
									/>
								</PaginationItem>

								{/* Page numbers */}
								{Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
									let pageNumber: number;
									if (totalPages <= 5) {
										pageNumber = i + 1;
									} else if (currentPage <= 3) {
										pageNumber = i + 1;
									} else if (currentPage >= totalPages - 2) {
										pageNumber = totalPages - 4 + i;
									} else {
										pageNumber = currentPage - 2 + i;
									}

									return (
										<PaginationItem key={pageNumber}>
											<PaginationLink
												onClick={() => setCurrentPage(pageNumber)}
												isActive={currentPage === pageNumber}
												className="cursor-pointer"
											>
												{pageNumber}
											</PaginationLink>
										</PaginationItem>
									);
								})}

								{totalPages > 5 && currentPage < totalPages - 2 && (
									<PaginationItem>
										<PaginationEllipsis />
									</PaginationItem>
								)}

								<PaginationItem>
									<PaginationNext
										onClick={() => setCurrentPage(currentPage + 1)}
										className={
											!hasNextPage
												? "pointer-events-none opacity-50"
												: "cursor-pointer"
										}
									/>
								</PaginationItem>
							</PaginationContent>
						</Pagination>
					</div>
				)}
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/users/")({
	component: UsersPage,
});
