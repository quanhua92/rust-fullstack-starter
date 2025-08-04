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
import { useCurrentUser, useMonitoringStats } from "@/hooks/useApiQueries";
import { useAuth } from "@/lib/auth/context";
import { getRoleColorClasses, getRoleDisplayName } from "@/lib/rbac/types";
import { Link, createFileRoute } from "@tanstack/react-router";
import {
	Activity,
	AlertTriangle,
	BarChart3,
	Database,
	Eye,
	Shield,
	TrendingUp,
	Zap,
} from "lucide-react";

function MonitoringDashboard() {
	const { isModeratorOrHigher } = useAuth();
	const { data: currentUser } = useCurrentUser(30000);

	// Only fetch monitoring stats if user is Moderator+
	const { data: monitoringStats, isLoading: isLoadingStats } =
		useMonitoringStats(15000);

	return (
		<AdminLayout>
			<div className="space-y-6">
				{/* Header with User Role Visibility */}
				<div className="flex items-center justify-between">
					<div>
						<h1 className="text-3xl font-bold tracking-tight">
							Monitoring & Observability
						</h1>
						<p className="text-muted-foreground">
							Comprehensive application monitoring and incident management
						</p>
					</div>
					{/* User Role Display for RBAC debugging */}
					<div className="flex items-center space-x-2">
						<span className="text-sm text-muted-foreground">Current Role:</span>
						<Badge
							variant="outline"
							className={`${getRoleColorClasses(currentUser?.role || "user").text} ${getRoleColorClasses(currentUser?.role || "user").border}`}
						>
							{getRoleDisplayName(currentUser?.role || "user")}
						</Badge>
					</div>
				</div>

				{/* System Statistics Cards (Moderator+ only) */}
				{isModeratorOrHigher() ? (
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
								<Card>
									<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
										<CardTitle className="text-sm font-medium">
											Total Events
										</CardTitle>
										<Activity className="h-4 w-4 text-muted-foreground" />
									</CardHeader>
									<CardContent>
										<div className="text-2xl font-bold">
											{monitoringStats?.total_events || 0}
										</div>
										<p className="text-xs text-muted-foreground">
											All monitoring events
										</p>
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
											{monitoringStats?.active_alerts || 0}
										</div>
										<p className="text-xs text-muted-foreground">
											Requiring attention
										</p>
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
											{monitoringStats?.open_incidents || 0}
										</div>
										<p className="text-xs text-muted-foreground">
											Active incidents
										</p>
									</CardContent>
								</Card>

								<Card>
									<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
										<CardTitle className="text-sm font-medium">
											Metrics Count
										</CardTitle>
										<TrendingUp className="h-4 w-4 text-muted-foreground" />
									</CardHeader>
									<CardContent>
										<div className="text-2xl font-bold">
											{monitoringStats?.total_metrics || 0}
										</div>
										<p className="text-xs text-muted-foreground">
											Performance metrics
										</p>
									</CardContent>
								</Card>
							</>
						)}
					</div>
				) : (
					<Card>
						<CardHeader>
							<CardTitle className="flex items-center space-x-2">
								<Eye className="h-5 w-5" />
								<span>Monitoring Access</span>
							</CardTitle>
							<CardDescription>
								Your current access level and available monitoring features
							</CardDescription>
						</CardHeader>
						<CardContent>
							<div className="space-y-4">
								<div className="flex items-center justify-between">
									<span>Current Role:</span>
									<Badge
										variant="outline"
										className={`${getRoleColorClasses(currentUser?.role || "user").text} ${getRoleColorClasses(currentUser?.role || "user").border}`}
									>
										{getRoleDisplayName(currentUser?.role || "user")}
									</Badge>
								</div>
								<div className="text-sm text-muted-foreground">
									<p>✅ You can create and view events</p>
									<p>✅ You can submit metrics</p>
									<p>✅ You can create and view your own incidents</p>
									<p>❌ System statistics require Moderator+ role</p>
									<p>❌ Alert management requires Moderator+ role</p>
								</div>
							</div>
						</CardContent>
					</Card>
				)}

				{/* Quick Actions */}
				<div className="space-y-4">
					<h2 className="text-xl font-semibold">Monitoring Features</h2>
					<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
						<Button asChild className="h-20 flex-col space-y-2">
							<Link to="/admin/monitoring/events">
								<Activity className="h-6 w-6" />
								<span>Events Dashboard</span>
							</Link>
						</Button>
						<Button
							asChild
							variant="outline"
							className="h-20 flex-col space-y-2"
						>
							<Link to="/admin/monitoring/metrics">
								<BarChart3 className="h-6 w-6" />
								<span>Metrics & Charts</span>
							</Link>
						</Button>
						<Button
							asChild
							variant={isModeratorOrHigher() ? "outline" : "secondary"}
							className="h-20 flex-col space-y-2"
							disabled={!isModeratorOrHigher()}
						>
							<Link to="/admin/monitoring/alerts">
								<AlertTriangle className="h-6 w-6" />
								<span>Alert Management</span>
								{!isModeratorOrHigher() && (
									<span className="text-xs">(Moderator+)</span>
								)}
							</Link>
						</Button>
						<Button
							asChild
							variant="outline"
							className="h-20 flex-col space-y-2"
						>
							<Link to="/admin/monitoring/incidents">
								<Shield className="h-6 w-6" />
								<span>Incident Tracking</span>
							</Link>
						</Button>
					</div>
				</div>

				{/* Recent Activity (mockup) */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<Zap className="h-5 w-5" />
							<span>Recent Activity</span>
						</CardTitle>
						<CardDescription>
							Latest monitoring events and incidents
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="space-y-4">
							{isModeratorOrHigher() && monitoringStats ? (
								<div className="grid gap-4 md:grid-cols-2">
									<div>
										<h4 className="font-medium mb-2">Events (Last Hour)</h4>
										<div className="text-2xl font-bold text-blue-600">
											{monitoringStats?.events_last_hour || 0}
										</div>
									</div>
									<div>
										<h4 className="font-medium mb-2">Metrics (Last Hour)</h4>
										<div className="text-2xl font-bold text-green-600">
											{monitoringStats?.metrics_last_hour || 0}
										</div>
									</div>
								</div>
							) : (
								<p className="text-muted-foreground text-center py-4">
									Start by exploring the available monitoring features above.
									<br />
									Create events and metrics to see activity here.
								</p>
							)}
						</div>
					</CardContent>
				</Card>

				{/* Integration Information */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<Database className="h-5 w-5" />
							<span>Monitoring Integration</span>
						</CardTitle>
						<CardDescription>
							Available monitoring capabilities and integrations
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="grid gap-4 md:grid-cols-2">
							<div className="space-y-2">
								<h4 className="font-medium">Event Types</h4>
								<div className="flex flex-wrap gap-2">
									<Badge variant="outline">log</Badge>
									<Badge variant="outline">metric</Badge>
									<Badge variant="outline">trace</Badge>
									<Badge variant="outline">alert</Badge>
								</div>
							</div>
							<div className="space-y-2">
								<h4 className="font-medium">Metric Types</h4>
								<div className="flex flex-wrap gap-2">
									<Badge variant="outline">counter</Badge>
									<Badge variant="outline">gauge</Badge>
									<Badge variant="outline">histogram</Badge>
									<Badge variant="outline">summary</Badge>
								</div>
							</div>
							<div className="space-y-2">
								<h4 className="font-medium">Tag Filtering</h4>
								<p className="text-sm text-muted-foreground">
									Advanced filtering with key:value pairs
									<br />
									Example: user_id:123,environment:production
								</p>
							</div>
							<div className="space-y-2">
								<h4 className="font-medium">Integrations</h4>
								<div className="flex flex-wrap gap-2">
									<Badge variant="outline">Prometheus Export</Badge>
									<Badge variant="outline">Timeline Correlation</Badge>
									<Badge variant="outline">RBAC Protected</Badge>
								</div>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/monitoring/")({
	component: MonitoringDashboard,
});
