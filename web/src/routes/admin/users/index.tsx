import { createFileRoute } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { AdminLayout } from "@/components/layout/AdminLayout";
import { Button } from "@/components/ui/button";
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
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { apiClient } from "@/lib/api/client";
import {
	Search,
	Plus,
	MoreHorizontal,
	UserCheck,
	UserX,
	Shield,
	Eye,
	Trash2,
} from "lucide-react";
import { useState } from "react";
import { Link } from "@tanstack/react-router";

function UsersPage() {
	const [searchTerm, setSearchTerm] = useState("");

	// Fetch users list
	const { data: users, isLoading } = useQuery({
		queryKey: ["users", searchTerm],
		queryFn: async () => {
			// Note: This would be enhanced with search/pagination in real implementation
			const response = await apiClient.getUsers();
			return response.data?.users || [];
		},
	});

	// Fetch current user for admin check
	const { data: currentUser } = useQuery({
		queryKey: ["currentUser"],
		queryFn: async () => {
			const response = await apiClient.getCurrentUser();
			return response.data;
		},
	});

	const filteredUsers = users?.filter((user) =>
		user.username?.toLowerCase().includes(searchTerm.toLowerCase()) ||
		user.email?.toLowerCase().includes(searchTerm.toLowerCase())
	) || [];

	const handleUserAction = async (userId: string, action: string) => {
		console.log(`${action} user:`, userId);
		// TODO: Implement user actions (activate, deactivate, delete, etc.)
	};

	const getUserRoleBadge = (role: string) => {
		const roleColors: Record<string, string> = {
			admin: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300",
			user: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300",
			moderator: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300",
		};

		return (
			<Badge 
				variant="secondary" 
				className={roleColors[role] || roleColors.user}
			>
				{role || 'user'}
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
					<Button asChild>
						<Link to="/admin/users/new">
							<Plus className="h-4 w-4 mr-2" />
							Add User
						</Link>
					</Button>
				</div>

				{/* Stats Cards */}
				<div className="grid gap-4 md:grid-cols-3">
					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Total Users</CardTitle>
							<UserCheck className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">{users?.length || 0}</div>
							<p className="text-xs text-muted-foreground">
								Registered accounts
							</p>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Active Users</CardTitle>
							<UserCheck className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">
								{users?.filter(u => u.is_active).length || 0}
							</div>
							<p className="text-xs text-muted-foreground">
								Currently active
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Admins</CardTitle>
							<Shield className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">
								{users?.filter(u => u.role === 'admin').length || 0}
							</div>
							<p className="text-xs text-muted-foreground">
								Administrator accounts
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
											{searchTerm ? "No users match your search." : "No users found."}
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
											<TableCell>
												{getUserRoleBadge(user.role)}
											</TableCell>
											<TableCell>
												{getStatusBadge(user.is_active)}
											</TableCell>
											<TableCell>
												<div className="text-sm">
													{user.created_at 
														? new Date(user.created_at).toLocaleDateString()
														: 'Unknown'
													}
												</div>
											</TableCell>
											<TableCell>
												<div className="text-sm text-muted-foreground">
													{user.last_login_at 
														? new Date(user.last_login_at).toLocaleDateString()
														: 'Never'
													}
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
															<Link to="/admin/users/$userId" params={{ userId: user.id }}>
																<Eye className="mr-2 h-4 w-4" />
																View Details
															</Link>
														</DropdownMenuItem>
														
														{currentUser?.role === 'admin' && user.id !== currentUser.id && (
															<>
																<DropdownMenuSeparator />
																<DropdownMenuItem
																	onClick={() => handleUserAction(user.id, user.is_active ? 'deactivate' : 'activate')}
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
																	onClick={() => handleUserAction(user.id, 'delete')}
																	className="text-red-600"
																>
																	<Trash2 className="mr-2 h-4 w-4" />
																	Delete User
																</DropdownMenuItem>
															</>
														)}
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
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/users/")({
	component: UsersPage,
});