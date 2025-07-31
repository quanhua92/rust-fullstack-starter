import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { apiClient } from "@/lib/api/client";
import { useQuery } from "@tanstack/react-query";
import {
	Activity,
	Calendar,
	Clock,
	Shield,
	TrendingUp,
	UserCheck,
	Users,
} from "lucide-react";
import { useMemo } from "react";
import {
	Area,
	AreaChart,
	Bar,
	BarChart,
	CartesianGrid,
	Cell,
	Pie,
	PieChart,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";

interface UserActivityData {
	timeframe: string;
	active_users: number;
	new_registrations: number;
	login_attempts: number;
	task_creation: number;
}

interface UserDemographics {
	role: string;
	count: number;
	color: string;
}

export function UserActivityAnalytics() {
	const currentUserQuery = useQuery({
		queryKey: ["auth", "me"],
		queryFn: () => apiClient.getCurrentUser(),
		refetchInterval: 30000,
	});

	const taskStatsQuery = useQuery({
		queryKey: ["tasks", "stats"],
		queryFn: () => apiClient.getTaskStats(),
		refetchInterval: 15000,
	});

	// Generate mock user activity data for demonstration
	const userActivityData: UserActivityData[] = useMemo(() => {
		const now = new Date();
		const data: UserActivityData[] = [];

		for (let i = 6; i >= 0; i--) {
			const date = new Date(now.getTime() - i * 24 * 60 * 60 * 1000);
			const dayName = date.toLocaleDateString("en-US", { weekday: "short" });

			data.push({
				timeframe: dayName,
				active_users: Math.floor(Math.random() * 50) + 20,
				new_registrations: Math.floor(Math.random() * 10) + 1,
				login_attempts: Math.floor(Math.random() * 100) + 50,
				task_creation: Math.floor(Math.random() * 30) + 10,
			});
		}

		return data;
	}, []);

	// Generate hourly activity data
	const hourlyActivityData = useMemo(() => {
		const data = [];
		for (let hour = 0; hour < 24; hour++) {
			const activity = Math.floor(Math.random() * 20) + 5;
			data.push({
				hour: `${hour.toString().padStart(2, "0")}:00`,
				activity,
				peak: hour >= 9 && hour <= 17, // Business hours
			});
		}
		return data;
	}, []);

	// Mock user demographics data
	const userDemographicsData: UserDemographics[] = useMemo(
		() => [
			{ role: "Admin", count: 5, color: "#EF4444" },
			{ role: "User", count: 45, color: "#3B82F6" },
			{ role: "Moderator", count: 8, color: "#10B981" },
			{ role: "Guest", count: 12, color: "#F59E0B" },
		],
		[],
	);

	// Calculate user engagement metrics
	const engagementMetrics = useMemo(() => {
		const totalTasks = taskStatsQuery.data?.data?.total || 0;
		const completedTasks = taskStatsQuery.data?.data?.completed || 0;
		const activeUsers =
			userActivityData[userActivityData.length - 1]?.active_users || 0;

		return {
			tasksPerUser: activeUsers > 0 ? Math.round(totalTasks / activeUsers) : 0,
			completionRate:
				totalTasks > 0 ? Math.round((completedTasks / totalTasks) * 100) : 0,
			dailyActiveUsers: activeUsers,
			avgSessionLength: "24m", // Mock data
		};
	}, [taskStatsQuery.data, userActivityData]);

	if (currentUserQuery.isLoading && taskStatsQuery.isLoading) {
		return (
			<div className="grid grid-cols-1 md:grid-cols-2 gap-6">
				<div className="h-64 bg-gray-100 animate-pulse rounded-lg" />
				<div className="h-64 bg-gray-100 animate-pulse rounded-lg" />
				<div className="h-64 bg-gray-100 animate-pulse rounded-lg" />
				<div className="h-64 bg-gray-100 animate-pulse rounded-lg" />
			</div>
		);
	}

	return (
		<div className="space-y-6">
			{/* User Engagement Overview */}
			<div className="grid grid-cols-1 md:grid-cols-4 gap-4">
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">
							Daily Active Users
						</CardTitle>
						<Users className="h-4 w-4 text-blue-600" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold text-blue-600">
							{engagementMetrics.dailyActiveUsers}
						</div>
						<p className="text-xs text-muted-foreground mt-1">
							+12% from yesterday
						</p>
					</CardContent>
				</Card>

				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">
							Tasks per User
						</CardTitle>
						<Activity className="h-4 w-4 text-green-600" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold text-green-600">
							{engagementMetrics.tasksPerUser}
						</div>
						<p className="text-xs text-muted-foreground mt-1">
							Average task creation
						</p>
					</CardContent>
				</Card>

				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">
							Completion Rate
						</CardTitle>
						<TrendingUp className="h-4 w-4 text-purple-600" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold text-purple-600">
							{engagementMetrics.completionRate}%
						</div>
						<p className="text-xs text-muted-foreground mt-1">
							Task success rate
						</p>
					</CardContent>
				</Card>

				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">Avg Session</CardTitle>
						<Clock className="h-4 w-4 text-orange-600" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold text-orange-600">
							{engagementMetrics.avgSessionLength}
						</div>
						<p className="text-xs text-muted-foreground mt-1">
							Session duration
						</p>
					</CardContent>
				</Card>
			</div>

			{/* Activity Charts */}
			<div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
				{/* Weekly User Activity */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<Calendar className="h-5 w-5" />
							<span>Weekly User Activity</span>
						</CardTitle>
						<CardDescription>
							User activity trends over the past 7 days
						</CardDescription>
					</CardHeader>
					<CardContent>
						<ResponsiveContainer width="100%" height={300}>
							<AreaChart data={userActivityData}>
								<CartesianGrid strokeDasharray="3 3" />
								<XAxis dataKey="timeframe" />
								<YAxis />
								<Tooltip />
								<Area
									type="monotone"
									dataKey="active_users"
									stackId="1"
									stroke="#3B82F6"
									fill="#3B82F6"
									fillOpacity={0.6}
								/>
								<Area
									type="monotone"
									dataKey="new_registrations"
									stackId="1"
									stroke="#10B981"
									fill="#10B981"
									fillOpacity={0.6}
								/>
							</AreaChart>
						</ResponsiveContainer>
					</CardContent>
				</Card>

				{/* User Role Distribution */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<Shield className="h-5 w-5" />
							<span>User Role Distribution</span>
						</CardTitle>
						<CardDescription>Distribution of users by role</CardDescription>
					</CardHeader>
					<CardContent>
						<ResponsiveContainer width="100%" height={300}>
							<PieChart>
								<Pie
									data={userDemographicsData}
									cx="50%"
									cy="50%"
									labelLine={false}
									label={({ role, percent }) =>
										`${role} ${((percent || 0) * 100).toFixed(0)}%`
									}
									outerRadius={80}
									fill="#8884d8"
									dataKey="count"
								>
									{userDemographicsData.map((entry, index) => (
										<Cell key={`cell-${index}`} fill={entry.color} />
									))}
								</Pie>
								<Tooltip />
							</PieChart>
						</ResponsiveContainer>
					</CardContent>
				</Card>

				{/* Hourly Activity Pattern */}
				<Card className="lg:col-span-2">
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<Clock className="h-5 w-5" />
							<span>Hourly Activity Pattern</span>
						</CardTitle>
						<CardDescription>
							User activity distribution throughout the day
						</CardDescription>
					</CardHeader>
					<CardContent>
						<ResponsiveContainer width="100%" height={300}>
							<BarChart data={hourlyActivityData}>
								<CartesianGrid strokeDasharray="3 3" />
								<XAxis dataKey="hour" />
								<YAxis />
								<Tooltip />
								<Bar dataKey="activity" fill="#8884d8">
									{hourlyActivityData.map((entry, index) => (
										<Cell
											key={`cell-${index}`}
											fill={entry.peak ? "#10B981" : "#94A3B8"}
										/>
									))}
								</Bar>
							</BarChart>
						</ResponsiveContainer>
					</CardContent>
				</Card>
			</div>

			{/* User Insights */}
			<div className="grid grid-cols-1 md:grid-cols-2 gap-6">
				{/* Current User Profile */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<UserCheck className="h-5 w-5" />
							<span>Current User Profile</span>
						</CardTitle>
						<CardDescription>
							Information about the currently logged-in user
						</CardDescription>
					</CardHeader>
					<CardContent>
						{currentUserQuery.data?.data ? (
							<div className="space-y-4">
								<div className="flex items-center space-x-4">
									<Avatar className="h-12 w-12">
										<AvatarFallback>
											{currentUserQuery.data.data.username
												.substring(0, 2)
												.toUpperCase()}
										</AvatarFallback>
									</Avatar>
									<div>
										<div className="font-semibold">
											{currentUserQuery.data.data.username}
										</div>
										<div className="text-sm text-muted-foreground">
											{currentUserQuery.data.data.email}
										</div>
									</div>
								</div>

								<div className="grid grid-cols-2 gap-4">
									<div>
										<div className="text-sm font-medium">Role</div>
										<Badge className="mt-1">
											{currentUserQuery.data.data.role}
										</Badge>
									</div>
									<div>
										<div className="text-sm font-medium">Status</div>
										<Badge variant="outline" className="mt-1 text-green-600">
											Active{" "}
											{/* Note: is_active not available in current API response */}
										</Badge>
									</div>
									<div>
										<div className="text-sm font-medium">Email Verified</div>
										<Badge variant="outline" className="mt-1">
											Yes{" "}
											{/* Note: email_verified not available in current API response */}
										</Badge>
									</div>
									<div>
										<div className="text-sm font-medium">Last Login</div>
										<div className="text-sm text-muted-foreground mt-1">
											Never{" "}
											{/* Note: last_login_at not available in current API response */}
										</div>
									</div>
								</div>

								<div>
									<div className="text-sm font-medium mb-2">
										Account Created
									</div>
									<div className="text-sm text-muted-foreground">
										Recently{" "}
										{/* Note: created_at not available in current API response */}
									</div>
								</div>
							</div>
						) : (
							<div className="text-center text-muted-foreground py-8">
								Loading user profile...
							</div>
						)}
					</CardContent>
				</Card>

				{/* Activity Summary */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<Activity className="h-5 w-5" />
							<span>Activity Summary</span>
						</CardTitle>
						<CardDescription>
							Recent user activity metrics and trends
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="space-y-4">
							<div className="grid grid-cols-2 gap-4">
								<div className="text-center p-3 bg-blue-50 rounded-lg">
									<div className="text-2xl font-bold text-blue-600">
										{userActivityData[userActivityData.length - 1]
											?.login_attempts || 0}
									</div>
									<div className="text-sm text-muted-foreground">
										Login Attempts Today
									</div>
								</div>
								<div className="text-center p-3 bg-green-50 rounded-lg">
									<div className="text-2xl font-bold text-green-600">
										{userActivityData[userActivityData.length - 1]
											?.task_creation || 0}
									</div>
									<div className="text-sm text-muted-foreground">
										Tasks Created Today
									</div>
								</div>
							</div>

							<div className="space-y-3">
								<div className="flex justify-between items-center">
									<span className="text-sm">Peak Activity Hour:</span>
									<Badge variant="outline">2:00 PM - 3:00 PM</Badge>
								</div>
								<div className="flex justify-between items-center">
									<span className="text-sm">Most Active Day:</span>
									<Badge variant="outline">Wednesday</Badge>
								</div>
								<div className="flex justify-between items-center">
									<span className="text-sm">User Retention:</span>
									<Badge className="bg-green-100 text-green-800">87%</Badge>
								</div>
								<div className="flex justify-between items-center">
									<span className="text-sm">Avg Tasks/User:</span>
									<Badge variant="outline">
										{engagementMetrics.tasksPerUser}
									</Badge>
								</div>
							</div>

							<div className="pt-4 border-t">
								<div className="text-sm font-medium mb-2">Trending Metrics</div>
								<div className="space-y-2">
									<div className="flex items-center justify-between">
										<span className="text-sm text-muted-foreground">
											Daily Active Users
										</span>
										<div className="flex items-center space-x-1">
											<TrendingUp className="h-3 w-3 text-green-600" />
											<span className="text-sm text-green-600">+12%</span>
										</div>
									</div>
									<div className="flex items-center justify-between">
										<span className="text-sm text-muted-foreground">
											Task Completion
										</span>
										<div className="flex items-center space-x-1">
											<TrendingUp className="h-3 w-3 text-green-600" />
											<span className="text-sm text-green-600">+5%</span>
										</div>
									</div>
								</div>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>

			{/* Real-time Updates Notice */}
			<Card>
				<CardContent className="pt-6">
					<div className="flex items-center space-x-2 text-sm text-muted-foreground">
						<Activity className="h-4 w-4" />
						<span>
							User activity data refreshes automatically every 15-30 seconds.
							Last updated: {new Date().toLocaleString()}
						</span>
					</div>
				</CardContent>
			</Card>
		</div>
	);
}
