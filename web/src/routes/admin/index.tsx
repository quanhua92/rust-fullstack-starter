import { HealthStatusCards } from "@/components/admin/HealthStatusCards";
import { RecentActivity } from "@/components/admin/RecentActivity";
import { StatsCard } from "@/components/admin/StatsCard";
import { AdminLayout } from "@/components/layout/AdminLayout";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import {
	useHealthBasic,
	useTaskStats,
	useCurrentUser,
} from "@/hooks/useApiQueries";
import { Link, createFileRoute } from "@tanstack/react-router";
import {
	Activity,
	AlertTriangle,
	BarChart3,
	CheckSquare,
	Clock,
	TrendingUp,
	Users,
	Zap,
} from "lucide-react";
import { useMemo } from "react";
import {
	Area,
	AreaChart,
	CartesianGrid,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";

function AdminDashboard() {
	// Fetch data with consistent hooks - no more cache collisions!
	const { data: taskStats, isLoading: isLoadingStats } = useTaskStats(10000);
	const { data: healthStatus } = useHealthBasic(15000);
	const { data: currentUser } = useCurrentUser(30000);

	// Generate trend data for mini charts (mock historical data)
	const trendData = useMemo(() => {
		const data = [];
		for (let i = 7; i >= 0; i--) {
			const completed =
				Math.floor(Math.random() * 20) + (taskStats?.completed || 0) * 0.1;
			const failed =
				Math.floor(Math.random() * 5) + (taskStats?.failed || 0) * 0.1;
			data.push({
				day: i,
				completed,
				failed,
				total: completed + failed,
			});
		}
		return data;
	}, [taskStats]);

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

				{/* Real-time Analytics Preview */}
				<div className="space-y-4">
					<div className="flex items-center justify-between">
						<h2 className="text-xl font-semibold">Live Analytics</h2>
						<Button asChild variant="outline">
							<Link to="/admin/analytics">
								<BarChart3 className="h-4 w-4 mr-2" />
								View Full Analytics
							</Link>
						</Button>
					</div>

					<div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
						{/* Task Trends Mini Chart */}
						<Card>
							<CardHeader>
								<CardTitle className="flex items-center space-x-2">
									<Activity className="h-5 w-5" />
									<span>Task Trends (7 days)</span>
								</CardTitle>
								<CardDescription>
									Real-time task completion and failure trends
								</CardDescription>
							</CardHeader>
							<CardContent>
								<ResponsiveContainer width="100%" height={200}>
									<AreaChart data={trendData}>
										<CartesianGrid strokeDasharray="3 3" />
										<XAxis dataKey="day" />
										<YAxis />
										<Tooltip />
										<Area
											type="monotone"
											dataKey="completed"
											stackId="1"
											stroke="#10B981"
											fill="#10B981"
											fillOpacity={0.6}
										/>
										<Area
											type="monotone"
											dataKey="failed"
											stackId="1"
											stroke="#EF4444"
											fill="#EF4444"
											fillOpacity={0.6}
										/>
									</AreaChart>
								</ResponsiveContainer>
							</CardContent>
						</Card>

						{/* Real-time Status */}
						<Card>
							<CardHeader>
								<CardTitle className="flex items-center space-x-2">
									<Zap className="h-5 w-5" />
									<span>Real-time Status</span>
								</CardTitle>
								<CardDescription>
									Live system and task monitoring
								</CardDescription>
							</CardHeader>
							<CardContent>
								<div className="space-y-4">
									<div className="flex items-center justify-between">
										<span className="text-sm font-medium">System Health:</span>
										<Badge
											className={
												healthStatus?.status === "healthy"
													? "bg-green-100 text-green-800"
													: "bg-red-100 text-red-800"
											}
										>
											{healthStatus?.status || "Unknown"}
										</Badge>
									</div>
									<div className="flex items-center justify-between">
										<span className="text-sm font-medium">Active Tasks:</span>
										<Badge variant="outline">
											{(taskStats?.pending || 0) + (taskStats?.running || 0)}
										</Badge>
									</div>
									<div className="flex items-center justify-between">
										<span className="text-sm font-medium">Success Rate:</span>
										<Badge className="bg-blue-100 text-blue-800">
											{taskStats?.total
												? Math.round(
														((taskStats.completed || 0) / taskStats.total) *
															100,
													)
												: 0}
											%
										</Badge>
									</div>
									<div className="flex items-center justify-between">
										<span className="text-sm font-medium">Last Update:</span>
										<span className="text-xs text-muted-foreground">
											{new Date().toLocaleTimeString()}
										</span>
									</div>
								</div>
							</CardContent>
						</Card>
					</div>
				</div>

				{/* Activity Section */}
				<div className="space-y-4">
					<h2 className="text-xl font-semibold">Recent Activity</h2>
					<RecentActivity activities={[]} /> {/* Uses mock data internally */}
				</div>

				{/* Quick Actions & Phase 4 Features */}
				<div className="space-y-4">
					<h2 className="text-xl font-semibold">Quick Actions</h2>
					<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
						<Button asChild className="h-20 flex-col space-y-2">
							<Link to="/admin/analytics">
								<BarChart3 className="h-6 w-6" />
								<span>Analytics Dashboard</span>
							</Link>
						</Button>
						<Button
							asChild
							variant="outline"
							className="h-20 flex-col space-y-2"
						>
							<Link to="/admin/tasks">
								<Activity className="h-6 w-6" />
								<span>Live Task Monitor</span>
							</Link>
						</Button>
						<Button
							asChild
							variant="outline"
							className="h-20 flex-col space-y-2"
						>
							<Link to="/admin/health">
								<Zap className="h-6 w-6" />
								<span>Health Trends</span>
							</Link>
						</Button>
						<Button
							asChild
							variant="outline"
							className="h-20 flex-col space-y-2"
						>
							<Link to="/admin/users">
								<Users className="h-6 w-6" />
								<span>User Analytics</span>
							</Link>
						</Button>
					</div>
				</div>

				{/* System Information */}
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
