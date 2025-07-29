import { createFileRoute } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { AdminLayout } from "@/components/layout/AdminLayout";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { apiClient } from "@/lib/api/client";
import {
	ArrowLeft,
	User,
	Mail,
	Calendar,
	Shield,
	Activity,
	Settings,
	Clock,
} from "lucide-react";
import { Link } from "@tanstack/react-router";

function UserDetailPage() {
	const { userId } = Route.useParams();

	// Fetch user details
	const { data: user, isLoading: isLoadingUser } = useQuery({
		queryKey: ["user", userId],
		queryFn: async () => {
			const response = await apiClient.getUser(userId);
			return response.data;
		},
	});

	// Fetch user's recent activities/sessions (mock data for now)
	const { data: activities } = useQuery({
		queryKey: ["userActivities", userId],
		queryFn: async () => {
			// Mock activity data - in real implementation, this would fetch from API
			return [
				{
					id: "1",
					action: "Login",
					timestamp: new Date().toISOString(),
					ip_address: "192.168.1.100",
					user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
				},
				{
					id: "2", 
					action: "Profile Update",
					timestamp: new Date(Date.now() - 86400000).toISOString(),
					ip_address: "192.168.1.100",
					user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
				},
			];
		},
	});

	if (isLoadingUser) {
		return (
			<AdminLayout>
				<div className="flex items-center justify-center min-h-[400px]">
					<div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900" />
				</div>
			</AdminLayout>
		);
	}

	if (!user) {
		return (
			<AdminLayout>
				<div className="text-center py-12">
					<h2 className="text-2xl font-semibold">User not found</h2>
					<p className="text-muted-foreground mt-2">
						The user you're looking for doesn't exist.
					</p>
					<Button asChild className="mt-4">
						<Link to="/admin/users">
							<ArrowLeft className="h-4 w-4 mr-2" />
							Back to Users
						</Link>
					</Button>
				</div>
			</AdminLayout>
		);
	}

	const getRoleBadge = (role: string) => {
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
					<div className="flex items-center space-x-4">
						<Button variant="ghost" size="sm" asChild>
							<Link to="/admin/users">
								<ArrowLeft className="h-4 w-4 mr-2" />
								Back to Users
							</Link>
						</Button>
						<div>
							<h1 className="text-3xl font-bold tracking-tight">
								{user.username}
							</h1>
							<p className="text-muted-foreground">
								User details and activity overview
							</p>
						</div>
					</div>
					<div className="flex items-center space-x-2">
						{getStatusBadge(user.is_active)}
						{getRoleBadge(user.role)}
					</div>
				</div>

				{/* User Info Cards */}
				<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Username</CardTitle>
							<User className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">{user.username}</div>
							<p className="text-xs text-muted-foreground">
								Display name
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Email</CardTitle>
							<Mail className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="font-bold truncate">{user.email}</div>
							<p className="text-xs text-muted-foreground">
								Contact email
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Role</CardTitle>
							<Shield className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="font-bold">{user.role || 'user'}</div>
							<p className="text-xs text-muted-foreground">
								Access level
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Member Since</CardTitle>
							<Calendar className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="font-bold">
								{user.created_at 
									? new Date(user.created_at).toLocaleDateString()
									: 'Unknown'
								}
							</div>
							<p className="text-xs text-muted-foreground">
								Registration date
							</p>
						</CardContent>
					</Card>
				</div>

				{/* Detailed Information Tabs */}
				<Tabs defaultValue="overview" className="space-y-4">
					<TabsList>
						<TabsTrigger value="overview">Overview</TabsTrigger>
						<TabsTrigger value="activity">Activity</TabsTrigger>
						<TabsTrigger value="settings">Settings</TabsTrigger>
					</TabsList>

					<TabsContent value="overview">
						<div className="grid gap-4 md:grid-cols-2">
							<Card>
								<CardHeader>
									<CardTitle className="flex items-center">
										<User className="h-5 w-5 mr-2" />
										Profile Information
									</CardTitle>
								</CardHeader>
								<CardContent className="space-y-4">
									<div className="grid grid-cols-2 gap-4">
										<div>
											<label className="text-sm font-medium text-muted-foreground">
												User ID
											</label>
											<p className="font-mono text-sm">{user.id}</p>
										</div>
										<div>
											<label className="text-sm font-medium text-muted-foreground">
												Username
											</label>
											<p>{user.username}</p>
										</div>
										<div>
											<label className="text-sm font-medium text-muted-foreground">
												Email
											</label>
											<p>{user.email}</p>
										</div>
										<div>
											<label className="text-sm font-medium text-muted-foreground">
												Role
											</label>
											<p>{user.role || 'user'}</p>
										</div>
									</div>
									<Separator />
									<div className="grid grid-cols-2 gap-4">
										<div>
											<label className="text-sm font-medium text-muted-foreground">
												Account Status
											</label>
											<p>{user.is_active ? 'Active' : 'Inactive'}</p>
										</div>
										<div>
											<label className="text-sm font-medium text-muted-foreground">
												Created At
											</label>
											<p>
												{user.created_at 
													? new Date(user.created_at).toLocaleString()
													: 'Unknown'
												}
											</p>
										</div>
									</div>
								</CardContent>
							</Card>

							<Card>
								<CardHeader>
									<CardTitle className="flex items-center">
										<Activity className="h-5 w-5 mr-2" />
										Account Activity
									</CardTitle>
								</CardHeader>
								<CardContent className="space-y-4">
									<div>
										<label className="text-sm font-medium text-muted-foreground">
											Last Login
										</label>
										<p>
											{user.last_login_at 
												? new Date(user.last_login_at).toLocaleString()
												: 'Never logged in'
											}
										</p>
									</div>
									<div>
										<label className="text-sm font-medium text-muted-foreground">
											Total Sessions
										</label>
										<p>N/A</p>
									</div>
									<div>
										<label className="text-sm font-medium text-muted-foreground">
											Account Age
										</label>
										<p>
											{user.created_at 
												? `${Math.floor((Date.now() - new Date(user.created_at).getTime()) / (1000 * 60 * 60 * 24))} days`
												: 'Unknown'
											}
										</p>
									</div>
								</CardContent>
							</Card>
						</div>
					</TabsContent>

					<TabsContent value="activity">
						<Card>
							<CardHeader>
								<CardTitle className="flex items-center">
									<Clock className="h-5 w-5 mr-2" />
									Recent Activity
								</CardTitle>
							</CardHeader>
							<CardContent>
								<Table>
									<TableHeader>
										<TableRow>
											<TableHead>Action</TableHead>
											<TableHead>Timestamp</TableHead>
											<TableHead>IP Address</TableHead>
											<TableHead>User Agent</TableHead>
										</TableRow>
									</TableHeader>
									<TableBody>
										{activities?.length === 0 ? (
											<TableRow>
												<TableCell colSpan={4} className="text-center py-4">
													No recent activity found.
												</TableCell>
											</TableRow>
										) : (
											activities?.map((activity) => (
												<TableRow key={activity.id}>
													<TableCell>
														<Badge variant="outline">{activity.action}</Badge>
													</TableCell>
													<TableCell>
														{new Date(activity.timestamp).toLocaleString()}
													</TableCell>
													<TableCell className="font-mono text-sm">
														{activity.ip_address}
													</TableCell>
													<TableCell className="max-w-xs truncate text-sm text-muted-foreground">
														{activity.user_agent}
													</TableCell>
												</TableRow>
											))
										)}
									</TableBody>
								</Table>
							</CardContent>
						</Card>
					</TabsContent>

					<TabsContent value="settings">
						<Card>
							<CardHeader>
								<CardTitle className="flex items-center">
									<Settings className="h-5 w-5 mr-2" />
									User Settings & Permissions
								</CardTitle>
							</CardHeader>
							<CardContent>
								<div className="space-y-4">
									<div className="flex items-center justify-between">
										<div>
											<h4 className="font-medium">Account Status</h4>
											<p className="text-sm text-muted-foreground">
												Enable or disable user account
											</p>
										</div>
										<Button variant="outline" size="sm">
											{user.is_active ? 'Deactivate' : 'Activate'} Account
										</Button>
									</div>
									<Separator />
									<div className="flex items-center justify-between">
										<div>
											<h4 className="font-medium">Role Management</h4>
											<p className="text-sm text-muted-foreground">
												Change user role and permissions
											</p>
										</div>
										<Button variant="outline" size="sm">
											Change Role
										</Button>
									</div>
									<Separator />
									<div className="flex items-center justify-between">
										<div>
											<h4 className="font-medium">Reset Password</h4>
											<p className="text-sm text-muted-foreground">
												Send password reset email to user
											</p>
										</div>
										<Button variant="outline" size="sm">
											Send Reset Link
										</Button>
									</div>
									<Separator />
									<div className="flex items-center justify-between">
										<div>
											<h4 className="font-medium">Delete Account</h4>
											<p className="text-sm text-muted-foreground">
												Permanently delete user account and data
											</p>
										</div>
										<Button variant="destructive" size="sm">
											Delete User
										</Button>
									</div>
								</div>
							</CardContent>
						</Card>
					</TabsContent>
				</Tabs>
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/users/$userId")({
	component: UserDetailPage,
});