import { AdminLayout } from "@/components/layout/AdminLayout";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { apiClient } from "@/lib/api/client";
import { useAuth } from "@/lib/auth/context";
import {
	type UserRole,
	getRoleColorClasses,
	getRoleDisplayName,
} from "@/lib/rbac/types";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import {
	Activity,
	BarChart3,
	Shield,
	TrendingUp,
	UserCheck,
	UserPlus,
	Users,
} from "lucide-react";

function UserAnalyticsPage() {
	const { isAdmin } = useAuth();

	// Fetch user statistics (Admin only)
	const {
		data: userStats,
		isLoading,
		error,
	} = useQuery({
		queryKey: ["admin", "users", "stats"],
		queryFn: async () => {
			const response = await apiClient.getUserStats();
			return response.data;
		},
		enabled: isAdmin(), // Only fetch if user is admin
	});

	// All metrics are now provided by the backend UserStats API

	// Show access denied for non-admin users
	if (!isAdmin()) {
		return (
			<AdminLayout>
				<div className="flex items-center justify-center min-h-[400px]">
					<div className="text-center space-y-4">
						<Shield className="h-12 w-12 text-muted-foreground mx-auto" />
						<div>
							<h3 className="text-lg font-semibold">Access Denied</h3>
							<p className="text-muted-foreground">
								You need administrator privileges to view user analytics.
							</p>
						</div>
					</div>
				</div>
			</AdminLayout>
		);
	}

	if (error) {
		return (
			<AdminLayout>
				<div className="flex items-center justify-center min-h-[400px]">
					<div className="text-center space-y-4">
						<BarChart3 className="h-12 w-12 text-muted-foreground mx-auto" />
						<div>
							<h3 className="text-lg font-semibold">
								Failed to Load Analytics
							</h3>
							<p className="text-muted-foreground">
								There was an error loading user analytics. Please try again
								later.
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
				<div>
					<h1 className="text-3xl font-bold tracking-tight">User Analytics</h1>
					<p className="text-muted-foreground">
						Comprehensive insights into user registration, activity, and
						demographics
					</p>
				</div>

				{isLoading ? (
					<div className="flex items-center justify-center min-h-[400px]">
						<div className="text-center space-y-4">
							<BarChart3 className="h-12 w-12 text-muted-foreground mx-auto animate-pulse" />
							<p>Loading analytics...</p>
						</div>
					</div>
				) : (
					<>
						{/* Overview Stats */}
						<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
							<Card>
								<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
									<CardTitle className="text-sm font-medium">
										Total Users
									</CardTitle>
									<Users className="h-4 w-4 text-muted-foreground" />
								</CardHeader>
								<CardContent>
									<div className="text-2xl font-bold">
										{userStats?.total_users || 0}
									</div>
									<p className="text-xs text-muted-foreground">
										All registered accounts
									</p>
								</CardContent>
							</Card>

							<Card>
								<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
									<CardTitle className="text-sm font-medium">
										Active Users
									</CardTitle>
									<UserCheck className="h-4 w-4 text-muted-foreground" />
								</CardHeader>
								<CardContent>
									<div className="text-2xl font-bold">
										{userStats?.active_users || 0}
									</div>
									<p className="text-xs text-muted-foreground">
										{userStats?.total_users && userStats.total_users > 0
											? `${((userStats.active_users / userStats.total_users) * 100).toFixed(1)}% of total`
											: "Currently active"}
									</p>
								</CardContent>
							</Card>

							<Card>
								<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
									<CardTitle className="text-sm font-medium">
										Recent Registrations
									</CardTitle>
									<UserPlus className="h-4 w-4 text-muted-foreground" />
								</CardHeader>
								<CardContent>
									<div className="text-2xl font-bold">
										{userStats?.recent_registrations?.last_30d || 0}
									</div>
									<p className="text-xs text-muted-foreground">Last 30 days</p>
								</CardContent>
							</Card>

							<Card>
								<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
									<CardTitle className="text-sm font-medium">
										This Week
									</CardTitle>
									<TrendingUp className="h-4 w-4 text-muted-foreground" />
								</CardHeader>
								<CardContent>
									<div className="text-2xl font-bold">
										{userStats?.recent_registrations?.last_7d || 0}
									</div>
									<p className="text-xs text-muted-foreground">
										New users this week
									</p>
								</CardContent>
							</Card>
						</div>

						{/* Role Distribution */}
						<div className="grid gap-4 md:grid-cols-2">
							<Card>
								<CardHeader>
									<CardTitle>Users by Role</CardTitle>
								</CardHeader>
								<CardContent>
									<div className="space-y-4">
										{userStats?.by_role ? (
											Object.entries(userStats.by_role).map(([role, count]) => {
												return (
													<div
														key={role}
														className="flex items-center justify-between"
													>
														<div className="flex items-center space-x-2">
															<Badge
																variant="outline"
																className={`${getRoleColorClasses(role as UserRole).text} ${getRoleColorClasses(role as UserRole).border}`}
															>
																{getRoleDisplayName(role as UserRole)}
															</Badge>
														</div>
														<div className="text-right">
															<div className="text-2xl font-bold">{count}</div>
															<div className="text-xs text-muted-foreground">
																{userStats.total_users > 0
																	? `${((Number(count) / userStats.total_users) * 100).toFixed(1)}%`
																	: "0%"}
															</div>
														</div>
													</div>
												);
											})
										) : (
											<p className="text-muted-foreground text-center py-4">
												No role data available
											</p>
										)}
									</div>
								</CardContent>
							</Card>

							<Card>
								<CardHeader>
									<CardTitle>Account Status</CardTitle>
								</CardHeader>
								<CardContent>
									<div className="space-y-4">
										<div className="flex items-center justify-between">
											<div className="flex items-center space-x-2">
												<UserCheck className="h-4 w-4 text-green-500" />
												<span>Active Accounts</span>
											</div>
											<div className="text-right">
												<div className="text-2xl font-bold">
													{userStats?.active_users || 0}
												</div>
												<div className="text-xs text-muted-foreground">
													{userStats?.total_users && userStats.total_users > 0
														? `${((userStats.active_users / userStats.total_users) * 100).toFixed(1)}%`
														: "0%"}
												</div>
											</div>
										</div>

										<div className="flex items-center justify-between">
											<div className="flex items-center space-x-2">
												<Activity className="h-4 w-4 text-blue-500" />
												<span>Email Verified</span>
											</div>
											<div className="text-right">
												<div className="text-2xl font-bold">
													{userStats?.email_verified || 0}
												</div>
												<div className="text-xs text-muted-foreground">
													{userStats?.total_users && userStats.total_users > 0
														? `${((userStats.email_verified / userStats.total_users) * 100).toFixed(1)}%`
														: "0%"}
												</div>
											</div>
										</div>

										<div className="flex items-center justify-between">
											<div className="flex items-center space-x-2">
												<UserPlus className="h-4 w-4 text-purple-500" />
												<span>Inactive Accounts</span>
											</div>
											<div className="text-right">
												<div className="text-2xl font-bold">
													{userStats?.inactive_users || 0}
												</div>
												<div className="text-xs text-muted-foreground">
													{userStats?.total_users && userStats.total_users > 0
														? `${((userStats.inactive_users / userStats.total_users) * 100).toFixed(1)}%`
														: "0%"}
												</div>
											</div>
										</div>
									</div>
								</CardContent>
							</Card>
						</div>

						{/* Recent Activity Summary */}
						<Card>
							<CardHeader>
								<CardTitle>Registration Trends</CardTitle>
							</CardHeader>
							<CardContent>
								<div className="grid gap-4 md:grid-cols-3">
									<div className="text-center">
										<div className="text-2xl font-bold text-blue-600">
											{userStats?.recent_registrations?.last_7d || 0}
										</div>
										<p className="text-sm text-muted-foreground">This Week</p>
									</div>
									<div className="text-center">
										<div className="text-2xl font-bold text-green-600">
											{userStats?.recent_registrations?.last_30d || 0}
										</div>
										<p className="text-sm text-muted-foreground">
											Last 30 Days
										</p>
									</div>
									<div className="text-center">
										<div className="text-2xl font-bold text-purple-600">
											{(
												(userStats?.recent_registrations?.last_30d || 0) / 30
											).toFixed(1)}
										</div>
										<p className="text-sm text-muted-foreground">
											Daily Average
										</p>
									</div>
								</div>
							</CardContent>
						</Card>
					</>
				)}
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/analytics")({
	component: UserAnalyticsPage,
});
