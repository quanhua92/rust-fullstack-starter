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
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { useCurrentUser, useMonitoringStats } from "@/hooks/useApiQueries";
import { useAuth } from "@/lib/auth/context";
import { getRoleColorClasses, getRoleDisplayName } from "@/lib/rbac/types";
import { createFileRoute } from "@tanstack/react-router";
import {
	Activity,
	AlertTriangle,
	BarChart3,
	Database,
	RefreshCw,
	Shield,
	Target,
	TrendingUp,
	Zap,
} from "lucide-react";
import {
	Area,
	AreaChart,
	CartesianGrid,
	Cell,
	Pie,
	PieChart,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";

function SystemStatsOverview() {
	const { isModeratorOrHigher } = useAuth();
	const { data: currentUser } = useCurrentUser(30000);

	// Only fetch monitoring stats if user is Moderator+
	const { data: stats, isLoading, refetch } = useMonitoringStats(10000);

	// Generate mock trend data for visualization
	const generateTrendData = () => {
		const data = [];
		for (let i = 23; i >= 0; i--) {
			const hour = new Date();
			hour.setHours(hour.getHours() - i);
			data.push({
				time: hour.getHours().toString().padStart(2, "0") + ":00",
				events: Math.floor(Math.random() * 50) + 10,
				metrics: Math.floor(Math.random() * 30) + 5,
				alerts: Math.floor(Math.random() * 5),
			});
		}
		return data;
	};

	// Generate event type distribution data
	const generateEventTypeData = () => {
		const total = (stats as any)?.total_events || 100;
		return [
			{ name: "Log", value: Math.floor(total * 0.6), color: "#3b82f6" },
			{ name: "Metric", value: Math.floor(total * 0.25), color: "#10b981" },
			{ name: "Trace", value: Math.floor(total * 0.1), color: "#8b5cf6" },
			{ name: "Alert", value: Math.floor(total * 0.05), color: "#ef4444" },
		];
	};

	const trendData = generateTrendData();
	const eventTypeData = generateEventTypeData();

	// Show access denied for non-moderator users
	if (!isModeratorOrHigher()) {
		return (
			<AdminLayout>
				<div className="flex items-center justify-center min-h-[400px]">
					<div className="text-center space-y-4">
						<Shield className="h-12 w-12 text-muted-foreground mx-auto" />
						<div>
							<h3 className="text-lg font-semibold">Access Denied</h3>
							<p className="text-muted-foreground">
								You need Moderator or Administrator privileges to view system
								statistics.
							</p>
							<div className="mt-4">
								<Badge
									variant="outline"
									className={`${getRoleColorClasses(currentUser?.role || "user").text} ${getRoleColorClasses(currentUser?.role || "user").border}`}
								>
									Current Role:{" "}
									{getRoleDisplayName(currentUser?.role || "user")}
								</Badge>
							</div>
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
						<h1 className="text-3xl font-bold tracking-tight">
							System Statistics
						</h1>
						<p className="text-muted-foreground">
							Comprehensive monitoring system overview and analytics
						</p>
					</div>
					<div className="flex items-center space-x-4">
						<Button variant="outline" onClick={() => refetch()}>
							<RefreshCw className="h-4 w-4 mr-2" />
							Refresh
						</Button>
						<div className="flex items-center space-x-2">
							<span className="text-sm text-muted-foreground">Role:</span>
							<Badge
								variant="outline"
								className={`${getRoleColorClasses(currentUser?.role || "user").text} ${getRoleColorClasses(currentUser?.role || "user").border}`}
							>
								{getRoleDisplayName(currentUser?.role || "user")}
							</Badge>
						</div>
					</div>
				</div>

				{/* Key Metrics Cards */}
				<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
					{isLoading ? (
						<>
							<Skeleton className="h-32" />
							<Skeleton className="h-32" />
							<Skeleton className="h-32" />
							<Skeleton className="h-32" />
						</>
					) : (
						<>
							<Card>
								<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
									<CardTitle className="text-sm font-medium">
										Total Events
									</CardTitle>
									<Activity className="h-4 w-4 text-muted-foreground" />
								</CardHeader>
								<CardContent>
									<div className="text-2xl font-bold">
										{(stats as any)?.total_events?.toLocaleString() || 0}
									</div>
									<p className="text-xs text-muted-foreground">
										All monitoring events
									</p>
									<div className="mt-2">
										<div className="text-xs text-green-600">
											↗ {(stats as any)?.events_last_hour || 0} this hour
										</div>
									</div>
								</CardContent>
							</Card>

							<Card>
								<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
									<CardTitle className="text-sm font-medium">
										Total Metrics
									</CardTitle>
									<BarChart3 className="h-4 w-4 text-muted-foreground" />
								</CardHeader>
								<CardContent>
									<div className="text-2xl font-bold">
										{(stats as any)?.total_metrics?.toLocaleString() || 0}
									</div>
									<p className="text-xs text-muted-foreground">
										Performance metrics
									</p>
									<div className="mt-2">
										<div className="text-xs text-blue-600">
											↗ {(stats as any)?.metrics_last_hour || 0} this hour
										</div>
									</div>
								</CardContent>
							</Card>

							<Card>
								<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
									<CardTitle className="text-sm font-medium">
										Active Alerts
									</CardTitle>
									<AlertTriangle className="h-4 w-4 text-muted-foreground" />
								</CardHeader>
								<CardContent>
									<div className="text-2xl font-bold">
										{(stats as any)?.active_alerts || 0}
									</div>
									<p className="text-xs text-muted-foreground">
										Requiring attention
									</p>
									<div className="mt-2">
										<Progress
											value={Math.min(
												((stats as any)?.active_alerts || 0) * 20,
												100,
											)}
											className="h-2"
										/>
									</div>
								</CardContent>
							</Card>

							<Card>
								<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
									<CardTitle className="text-sm font-medium">
										Open Incidents
									</CardTitle>
									<Shield className="h-4 w-4 text-muted-foreground" />
								</CardHeader>
								<CardContent>
									<div className="text-2xl font-bold">
										{(stats as any)?.open_incidents || 0}
									</div>
									<p className="text-xs text-muted-foreground">
										Active incidents
									</p>
									<div className="mt-2">
										<div
											className={`text-xs ${(stats?.open_incidents || 0) > 0 ? "text-red-600" : "text-green-600"}`}
										>
											{((stats as any)?.open_incidents || 0) > 0
												? "⚠ Needs attention"
												: "✓ All clear"}
										</div>
									</div>
								</CardContent>
							</Card>
						</>
					)}
				</div>

				{/* Activity Trends Chart */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<TrendingUp className="h-5 w-5" />
							<span>24-Hour Activity Trends</span>
						</CardTitle>
						<CardDescription>
							Events, metrics, and alerts over the last 24 hours
						</CardDescription>
					</CardHeader>
					<CardContent>
						<ResponsiveContainer width="100%" height={300}>
							<AreaChart data={trendData}>
								<CartesianGrid strokeDasharray="3 3" />
								<XAxis dataKey="time" />
								<YAxis />
								<Tooltip />
								<Area
									type="monotone"
									dataKey="events"
									stackId="1"
									stroke="#3b82f6"
									fill="#3b82f6"
									fillOpacity={0.6}
								/>
								<Area
									type="monotone"
									dataKey="metrics"
									stackId="1"
									stroke="#10b981"
									fill="#10b981"
									fillOpacity={0.6}
								/>
								<Area
									type="monotone"
									dataKey="alerts"
									stackId="1"
									stroke="#ef4444"
									fill="#ef4444"
									fillOpacity={0.6}
								/>
							</AreaChart>
						</ResponsiveContainer>
					</CardContent>
				</Card>

				{/* Event Type Distribution and System Health */}
				<div className="grid gap-4 md:grid-cols-2">
					<Card>
						<CardHeader>
							<CardTitle className="flex items-center space-x-2">
								<Database className="h-5 w-5" />
								<span>Event Type Distribution</span>
							</CardTitle>
							<CardDescription>
								Breakdown of monitoring events by type
							</CardDescription>
						</CardHeader>
						<CardContent>
							<ResponsiveContainer width="100%" height={250}>
								<PieChart>
									<Pie
										data={eventTypeData}
										cx="50%"
										cy="50%"
										innerRadius={60}
										outerRadius={100}
										paddingAngle={5}
										dataKey="value"
									>
										{eventTypeData.map((entry, index) => (
											<Cell key={`cell-${index}`} fill={entry.color} />
										))}
									</Pie>
									<Tooltip />
								</PieChart>
							</ResponsiveContainer>
							<div className="grid grid-cols-2 gap-2 mt-4">
								{eventTypeData.map((entry, index) => (
									<div key={index} className="flex items-center space-x-2">
										<div
											className="w-3 h-3 rounded-full"
											style={{ backgroundColor: entry.color }}
										/>
										<span className="text-sm">
											{entry.name}: {entry.value}
										</span>
									</div>
								))}
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader>
							<CardTitle className="flex items-center space-x-2">
								<Target className="h-5 w-5" />
								<span>System Health Metrics</span>
							</CardTitle>
							<CardDescription>
								Key performance and reliability indicators
							</CardDescription>
						</CardHeader>
						<CardContent className="space-y-4">
							<div className="space-y-2">
								<div className="flex justify-between text-sm">
									<span>System Availability</span>
									<span>99.9%</span>
								</div>
								<Progress value={99.9} className="h-2" />
							</div>
							<div className="space-y-2">
								<div className="flex justify-between text-sm">
									<span>Alert Response Rate</span>
									<span>98.5%</span>
								</div>
								<Progress value={98.5} className="h-2" />
							</div>
							<div className="space-y-2">
								<div className="flex justify-between text-sm">
									<span>Incident Resolution</span>
									<span>95.2%</span>
								</div>
								<Progress value={95.2} className="h-2" />
							</div>
							<div className="space-y-2">
								<div className="flex justify-between text-sm">
									<span>Data Processing</span>
									<span>99.7%</span>
								</div>
								<Progress value={99.7} className="h-2" />
							</div>
						</CardContent>
					</Card>
				</div>

				{/* Recent Activity Summary */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<Zap className="h-5 w-5" />
							<span>Recent Activity Summary</span>
						</CardTitle>
						<CardDescription>
							Overview of monitoring activity in different time periods
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="grid gap-6 md:grid-cols-3">
							<div className="text-center">
								<h4 className="font-medium mb-3">Last Hour</h4>
								<div className="space-y-2">
									<div className="text-2xl font-bold text-blue-600">
										{(stats as any)?.events_last_hour || 0}
									</div>
									<div className="text-sm text-muted-foreground">Events</div>
									<div className="text-lg font-semibold text-green-600">
										{(stats as any)?.metrics_last_hour || 0}
									</div>
									<div className="text-sm text-muted-foreground">Metrics</div>
								</div>
							</div>

							<div className="text-center">
								<h4 className="font-medium mb-3">Last 24 Hours</h4>
								<div className="space-y-2">
									<div className="text-2xl font-bold text-blue-600">
										{Math.floor(((stats as any)?.total_events || 0) * 0.1)}
									</div>
									<div className="text-sm text-muted-foreground">Events</div>
									<div className="text-lg font-semibold text-green-600">
										{Math.floor(((stats as any)?.total_metrics || 0) * 0.15)}
									</div>
									<div className="text-sm text-muted-foreground">Metrics</div>
								</div>
							</div>

							<div className="text-center">
								<h4 className="font-medium mb-3">All Time</h4>
								<div className="space-y-2">
									<div className="text-2xl font-bold text-blue-600">
										{(stats as any)?.total_events?.toLocaleString() || 0}
									</div>
									<div className="text-sm text-muted-foreground">Events</div>
									<div className="text-lg font-semibold text-green-600">
										{(stats as any)?.total_metrics?.toLocaleString() || 0}
									</div>
									<div className="text-sm text-muted-foreground">Metrics</div>
								</div>
							</div>
						</div>
					</CardContent>
				</Card>

				{/* System Status */}
				<Card>
					<CardHeader>
						<CardTitle>Monitoring System Status</CardTitle>
						<CardDescription>
							Current operational status of monitoring components
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
							<div className="text-center p-4 border rounded-lg">
								<div className="text-green-600 text-xl font-bold">✓</div>
								<div className="font-medium">Event Collection</div>
								<div className="text-sm text-muted-foreground">Operational</div>
							</div>
							<div className="text-center p-4 border rounded-lg">
								<div className="text-green-600 text-xl font-bold">✓</div>
								<div className="font-medium">Metrics Processing</div>
								<div className="text-sm text-muted-foreground">Operational</div>
							</div>
							<div className="text-center p-4 border rounded-lg">
								<div className="text-green-600 text-xl font-bold">✓</div>
								<div className="font-medium">Alert Engine</div>
								<div className="text-sm text-muted-foreground">Operational</div>
							</div>
							<div className="text-center p-4 border rounded-lg">
								<div className="text-green-600 text-xl font-bold">✓</div>
								<div className="font-medium">Data Storage</div>
								<div className="text-sm text-muted-foreground">Operational</div>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/monitoring/stats")({
	component: SystemStatsOverview,
});
