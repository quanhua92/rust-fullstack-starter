import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { formatDistanceToNow } from "date-fns";
import { CheckCircle, XCircle, User, Settings } from "lucide-react";

type ActivityType =
	| "task_completed"
	| "task_failed"
	| "user_login"
	| "system_event";

interface Activity {
	id: string;
	type: ActivityType;
	title: string;
	description: string;
	timestamp: string;
	user?: string;
}

interface RecentActivityProps {
	activities: Activity[];
}

const activityConfig = {
	task_completed: {
		icon: CheckCircle,
		color: "text-green-600",
		badge: "default" as const,
	},
	task_failed: {
		icon: XCircle,
		color: "text-red-600",
		badge: "destructive" as const,
	},
	user_login: {
		icon: User,
		color: "text-blue-600",
		badge: "secondary" as const,
	},
	system_event: {
		icon: Settings,
		color: "text-gray-600",
		badge: "outline" as const,
	},
};

// Mock data for demonstration
const mockActivities: Activity[] = [
	{
		id: "1",
		type: "task_completed",
		title: "Email task completed",
		description: "Email sent to user@example.com",
		timestamp: new Date(Date.now() - 1000 * 60 * 5).toISOString(),
		user: "worker-1",
	},
	{
		id: "2",
		type: "user_login",
		title: "User login",
		description: "admin logged in from 192.168.1.1",
		timestamp: new Date(Date.now() - 1000 * 60 * 15).toISOString(),
		user: "admin",
	},
	{
		id: "3",
		type: "task_failed",
		title: "Webhook task failed",
		description: "Failed to deliver webhook to api.example.com",
		timestamp: new Date(Date.now() - 1000 * 60 * 30).toISOString(),
	},
	{
		id: "4",
		type: "system_event",
		title: "Database backup",
		description: "Scheduled database backup completed",
		timestamp: new Date(Date.now() - 1000 * 60 * 60).toISOString(),
	},
	{
		id: "5",
		type: "task_completed",
		title: "Data processing task completed",
		description: "Processed 1,000 records",
		timestamp: new Date(Date.now() - 1000 * 60 * 90).toISOString(),
	},
];

export function RecentActivity({
	activities = mockActivities,
}: RecentActivityProps) {
	return (
		<Card>
			<CardHeader>
				<CardTitle>Recent Activity</CardTitle>
			</CardHeader>
			<CardContent>
				<ScrollArea className="h-[300px]">
					<div className="space-y-4">
						{activities.map((activity) => {
							const config = activityConfig[activity.type];
							const Icon = config.icon;

							return (
								<div key={activity.id} className="flex items-start space-x-3">
									<Icon className={`h-4 w-4 mt-1 ${config.color}`} />
									<div className="flex-1 space-y-1">
										<div className="flex items-center justify-between">
											<p className="text-sm font-medium leading-none">
												{activity.title}
											</p>
											<Badge variant={config.badge} className="text-xs">
												{activity.type.replace("_", " ")}
											</Badge>
										</div>
										<p className="text-sm text-muted-foreground">
											{activity.description}
										</p>
										<div className="flex items-center justify-between text-xs text-muted-foreground">
											<span>
												{formatDistanceToNow(new Date(activity.timestamp), {
													addSuffix: true,
												})}
											</span>
											{activity.user && <span>by {activity.user}</span>}
										</div>
									</div>
								</div>
							);
						})}
					</div>
				</ScrollArea>
			</CardContent>
		</Card>
	);
}
