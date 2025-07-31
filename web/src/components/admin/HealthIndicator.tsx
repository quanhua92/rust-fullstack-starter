import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { AlertCircle, CheckCircle, Clock, XCircle } from "lucide-react";

type HealthStatus = "healthy" | "unhealthy" | "warning" | "unknown";

interface HealthIndicatorProps {
	title: string;
	status: HealthStatus;
	message?: string;
	uptime?: string;
	version?: string;
}

const statusConfig = {
	healthy: {
		icon: CheckCircle,
		color: "text-green-600",
		bgColor: "bg-green-50",
		badge: "default" as const,
		text: "Healthy",
	},
	unhealthy: {
		icon: XCircle,
		color: "text-red-600",
		bgColor: "bg-red-50",
		badge: "destructive" as const,
		text: "Unhealthy",
	},
	warning: {
		icon: AlertCircle,
		color: "text-yellow-600",
		bgColor: "bg-yellow-50",
		badge: "secondary" as const,
		text: "Warning",
	},
	unknown: {
		icon: Clock,
		color: "text-gray-600",
		bgColor: "bg-gray-50",
		badge: "outline" as const,
		text: "Unknown",
	},
};

export function HealthIndicator({
	title,
	status,
	message,
	uptime,
	version,
}: HealthIndicatorProps) {
	const config = statusConfig[status];
	const Icon = config.icon;

	return (
		<Card>
			<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle className="text-sm font-medium">{title}</CardTitle>
				<Badge variant={config.badge}>{config.text}</Badge>
			</CardHeader>
			<CardContent>
				<div className="flex items-center space-x-2">
					<Icon className={`h-5 w-5 ${config.color}`} />
					<div className="flex-1">
						{message && (
							<p className="text-sm text-muted-foreground">{message}</p>
						)}
						<div className="flex flex-col space-y-1 mt-2">
							{version && (
								<div className="text-xs text-muted-foreground">
									<span className="font-medium">Version:</span> {version}
								</div>
							)}
							{uptime && (
								<div className="text-xs text-muted-foreground">
									<span className="font-medium">Uptime:</span> {uptime}
								</div>
							)}
						</div>
					</div>
				</div>
			</CardContent>
		</Card>
	);
}
