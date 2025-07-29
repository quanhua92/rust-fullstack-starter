import { HealthStatusCards } from "@/components/admin/HealthStatusCards";
import { RecentActivity } from "@/components/admin/RecentActivity";
import { StatsCard } from "@/components/admin/StatsCard";
import { AdminLayout } from "@/components/layout/AdminLayout";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api/client";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { AlertTriangle, CheckSquare, Clock, TrendingUp } from "lucide-react";

function AdminDashboard() {
	// Fetch task statistics
	const { data: taskStats, isLoading: isLoadingStats } = useQuery({
		queryKey: ["taskStats"],
		queryFn: async () => {
			const response = await apiClient.getTaskStats();
			return response.data;
		},
	});


	// Fetch current user for user count (mock data for now)
	const { data: currentUser } = useQuery({
		queryKey: ["currentUser"],
		queryFn: async () => {
			const response = await apiClient.getCurrentUser();
			return response.data;
		},
	});


	return (
		<AdminLayout>
			<div className="space-y-6">
				{/* Header */}
				<div className="flex items-center justify-between">
					<div>
						<h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
						<p className="text-muted-foreground">
							Welcome back! Here's what's happening with your system.
						</p>
					</div>
				</div>

				{/* Stats Cards */}
				<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
					{isLoadingStats ? (
						<>
							<Skeleton className="h-32" />
							<Skeleton className="h-32" />
							<Skeleton className="h-32" />
							<Skeleton className="h-32" />
						</>
					) : (
						<>
							<StatsCard
								title="Total Tasks"
								value={taskStats?.total || 0}
								description="All time task count"
								icon={CheckSquare}
								trend={{ value: 12, isPositive: true }}
							/>
							<StatsCard
								title="Active Tasks"
								value={(taskStats?.pending || 0) + (taskStats?.running || 0)}
								description="Currently processing"
								icon={Clock}
							/>
							<StatsCard
								title="Failed Tasks"
								value={taskStats?.failed || 0}
								description="Requiring attention"
								icon={AlertTriangle}
							/>
							<StatsCard
								title="Success Rate"
								value={`${taskStats?.total ? Math.round(((taskStats.completed || 0) / taskStats.total) * 100) : 0}%`}
								description="Task completion rate"
								icon={TrendingUp}
								trend={{ value: 2.5, isPositive: true }}
							/>
						</>
					)}
				</div>

				{/* Health Status Cards */}
				<div className="space-y-4">
					<h2 className="text-xl font-semibold">System Health</h2>
					<HealthStatusCards />
				</div>

				{/* Activity Section */}
				<div className="space-y-4">
					<h2 className="text-xl font-semibold">Recent Activity</h2>
					<RecentActivity activities={[]} /> {/* Uses mock data internally */}
				</div>

				{/* Quick Actions */}
				<div className="grid gap-4 md:grid-cols-3">
					<div className="rounded-lg border p-4">
						<h3 className="font-medium mb-2">Quick Stats</h3>
						<div className="space-y-2 text-sm text-muted-foreground">
							<div className="flex justify-between">
								<span>Completed Tasks:</span>
								<span className="font-medium">{taskStats?.completed || 0}</span>
							</div>
							<div className="flex justify-between">
								<span>Pending Tasks:</span>
								<span className="font-medium">{taskStats?.pending || 0}</span>
							</div>
							<div className="flex justify-between">
								<span>Running Tasks:</span>
								<span className="font-medium">{taskStats?.running || 0}</span>
							</div>
						</div>
					</div>

					<div className="rounded-lg border p-4">
						<h3 className="font-medium mb-2">System Resources</h3>
						<div className="space-y-2 text-sm text-muted-foreground">
							<div className="flex justify-between">
								<span>Memory Usage:</span>
								<span className="font-medium">245 MB</span>
							</div>
							<div className="flex justify-between">
								<span>CPU Usage:</span>
								<span className="font-medium">12%</span>
							</div>
							<div className="flex justify-between">
								<span>Disk Usage:</span>
								<span className="font-medium">1.2 GB</span>
							</div>
						</div>
					</div>

					<div className="rounded-lg border p-4">
						<h3 className="font-medium mb-2">Current User</h3>
						<div className="space-y-2 text-sm text-muted-foreground">
							<div className="flex justify-between">
								<span>Username:</span>
								<span className="font-medium">
									{currentUser?.username || "Loading..."}
								</span>
							</div>
							<div className="flex justify-between">
								<span>Role:</span>
								<span className="font-medium">
									{currentUser?.role || "Loading..."}
								</span>
							</div>
							<div className="flex justify-between">
								<span>Email:</span>
								<span className="font-medium">
									{currentUser?.email || "Loading..."}
								</span>
							</div>
						</div>
					</div>
				</div>
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/")({
	component: AdminDashboard,
});
