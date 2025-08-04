import { DependencyMonitor } from "@/components/admin/DependencyMonitor";
import { SystemMetrics } from "@/components/admin/SystemMetrics";
import { AdminLayout } from "@/components/layout/AdminLayout";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { apiClient } from "@/lib/api/client";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import {
	Activity,
	AlertTriangle,
	CheckCircle,
	Clock,
	Database,
	Heart,
	Play,
	Server,
	Settings,
	Shield,
	XCircle,
	Zap,
} from "lucide-react";

import type { components } from "@/types/api";

type ComponentHealth = components["schemas"]["ComponentHealth"];

// Type for probe responses that return unknown data
interface ProbeResponse {
	probe?: string;
	status?: string;
	timestamp?: string;
	checks?: Record<string, ComponentHealth>;
}

// Type for health check details
type HealthCheckDetails = Record<string, string | number | boolean | unknown>;

// Type guards for API responses
const isProbeResponse = (data: unknown): data is ProbeResponse => {
	return (
		typeof data === "object" &&
		data !== null &&
		typeof (data as ProbeResponse).status === "string"
	);
};

const isHealthCheckDetails = (data: unknown): data is HealthCheckDetails => {
	return typeof data === "object" && data !== null;
};

const getProbeStatus = (data: unknown): string => {
	if (isProbeResponse(data)) {
		return data.status || "unknown";
	}
	return "unknown";
};

const getProbeTimestamp = (data: unknown): string | undefined => {
	if (isProbeResponse(data)) {
		return data.timestamp;
	}
	return undefined;
};

const getProbeChecks = (
	data: unknown,
): Record<string, ComponentHealth> | undefined => {
	if (isProbeResponse(data) && data.checks) {
		return data.checks;
	}
	return undefined;
};

const renderDetails = (details: unknown) => {
	if (!details || typeof details !== "object" || details === null) {
		return null;
	}

	return (
		<div className="text-xs text-muted-foreground">
			<pre className="whitespace-pre-wrap">
				{JSON.stringify(details, null, 2)}
			</pre>
		</div>
	);
};

const renderCheckDetails = (details: unknown) => {
	if (!isHealthCheckDetails(details)) {
		return null;
	}

	return (
		<div className="text-xs text-muted-foreground">
			{Object.entries(details).map(
				([key, value]) => (
					<div key={key}>
						{key}:{" "}
						{typeof value === "string" ||
						typeof value === "number" ||
						typeof value === "boolean"
							? String(value)
							: JSON.stringify(value)}
					</div>
				),
			)}
		</div>
	);
};

export const Route = createFileRoute("/admin/health")({
	component: () => (
		<AdminLayout>
			<HealthDashboard />
		</AdminLayout>
	),
});

function HealthDashboard() {
	const basicHealthQuery = useQuery({
		queryKey: ["health", "basic"],
		queryFn: () => apiClient.getHealth(),
		refetchInterval: 30000, // Refresh every 30 seconds
	});

	const detailedHealthQuery = useQuery({
		queryKey: ["health", "detailed"],
		queryFn: () => apiClient.getDetailedHealth(),
		refetchInterval: 30000,
	});

	const livenessQuery = useQuery({
		queryKey: ["health", "liveness"],
		queryFn: () => apiClient.getLivenessProbe(),
		refetchInterval: 10000, // More frequent for critical probes
	});

	const readinessQuery = useQuery({
		queryKey: ["health", "readiness"],
		queryFn: () => apiClient.getReadinessProbe(),
		refetchInterval: 15000,
	});

	const startupQuery = useQuery({
		queryKey: ["health", "startup"],
		queryFn: () => apiClient.getStartupProbe(),
		refetchInterval: 20000,
	});

	const getStatusColor = (status: string) => {
		switch (status.toLowerCase()) {
			case "healthy":
			case "alive":
			case "ready":
			case "started":
				return "text-green-600";
			case "unhealthy":
			case "down":
			case "failed":
				return "text-red-600";
			case "degraded":
			case "warning":
				return "text-yellow-600";
			default:
				return "text-gray-600";
		}
	};

	const getStatusIcon = (status: string) => {
		switch (status.toLowerCase()) {
			case "healthy":
			case "alive":
			case "ready":
			case "started":
				return <CheckCircle className="h-4 w-4 text-green-600" />;
			case "unhealthy":
			case "down":
			case "failed":
				return <XCircle className="h-4 w-4 text-red-600" />;
			case "degraded":
			case "warning":
				return <AlertTriangle className="h-4 w-4 text-yellow-600" />;
			default:
				return <Activity className="h-4 w-4 text-gray-600" />;
		}
	};

	const formatUptime = (uptime: number) => {
		const uptimeMs = uptime * 1000;
		const now = new Date();
		const startTime = new Date(now.getTime() - uptimeMs);
		return formatDistanceToNow(startTime, { addSuffix: true });
	};

	return (
		<div className="space-y-6">
			<div>
				<h2 className="text-3xl font-bold tracking-tight">System Health</h2>
				<p className="text-muted-foreground">
					Monitor application health, dependencies, and system status
				</p>
			</div>

			{/* Health Overview Cards */}
			<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">Application</CardTitle>
						<Heart className="h-4 w-4 text-muted-foreground" />
					</CardHeader>
					<CardContent>
						<div className="flex items-center space-x-2">
							{basicHealthQuery.data?.data &&
								getStatusIcon(basicHealthQuery.data.data.status)}
							<div
								className={`text-2xl font-bold ${basicHealthQuery.data?.data ? getStatusColor(basicHealthQuery.data.data.status) : "text-gray-400"}`}
							>
								{basicHealthQuery.isLoading
									? "Loading..."
									: basicHealthQuery.data?.data?.status || "Unknown"}
							</div>
						</div>
						<p className="text-xs text-muted-foreground">
							Version {basicHealthQuery.data?.data?.version || "Unknown"}
						</p>
					</CardContent>
				</Card>

				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">Liveness</CardTitle>
						<Zap className="h-4 w-4 text-muted-foreground" />
					</CardHeader>
					<CardContent>
						<div className="flex items-center space-x-2">
							{getStatusIcon(getProbeStatus(livenessQuery.data?.data))}
							<div
								className={`text-2xl font-bold ${livenessQuery.data?.data ? getStatusColor(getProbeStatus(livenessQuery.data.data)) : "text-gray-400"}`}
							>
								{livenessQuery.isLoading
									? "Loading..."
									: getProbeStatus(livenessQuery.data?.data) || "Unknown"}
							</div>
						</div>
						<p className="text-xs text-muted-foreground">
							Kubernetes liveness probe
						</p>
					</CardContent>
				</Card>

				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">Readiness</CardTitle>
						<Shield className="h-4 w-4 text-muted-foreground" />
					</CardHeader>
					<CardContent>
						<div className="flex items-center space-x-2">
							{getStatusIcon(getProbeStatus(readinessQuery.data?.data))}
							<div
								className={`text-2xl font-bold ${readinessQuery.data?.data ? getStatusColor(getProbeStatus(readinessQuery.data.data)) : "text-gray-400"}`}
							>
								{readinessQuery.isLoading
									? "Loading..."
									: getProbeStatus(readinessQuery.data?.data) || "Unknown"}
							</div>
						</div>
						<p className="text-xs text-muted-foreground">
							Kubernetes readiness probe
						</p>
					</CardContent>
				</Card>

				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">Startup</CardTitle>
						<Play className="h-4 w-4 text-muted-foreground" />
					</CardHeader>
					<CardContent>
						<div className="flex items-center space-x-2">
							{getStatusIcon(getProbeStatus(startupQuery.data?.data))}
							<div
								className={`text-2xl font-bold ${startupQuery.data?.data ? getStatusColor(getProbeStatus(startupQuery.data.data)) : "text-gray-400"}`}
							>
								{startupQuery.isLoading
									? "Loading..."
									: getProbeStatus(startupQuery.data?.data) || "Unknown"}
							</div>
						</div>
						<p className="text-xs text-muted-foreground">
							Kubernetes startup probe
						</p>
					</CardContent>
				</Card>
			</div>

			{/* System Metrics */}
			<Card>
				<CardHeader>
					<CardTitle className="flex items-center space-x-2">
						<Server className="h-5 w-5" />
						<span>System Metrics</span>
					</CardTitle>
					<CardDescription>
						Core application metrics and uptime information
					</CardDescription>
				</CardHeader>
				<CardContent>
					<div className="grid grid-cols-1 md:grid-cols-2 gap-6">
						<div className="space-y-2">
							<div className="flex items-center justify-between">
								<span className="text-sm font-medium">Uptime</span>
								<Clock className="h-4 w-4 text-muted-foreground" />
							</div>
							<div className="text-2xl font-bold">
								{basicHealthQuery.data?.data?.uptime
									? formatUptime(basicHealthQuery.data.data.uptime)
									: "Loading..."}
							</div>
							<Progress value={100} className="h-2" />
						</div>

						<div className="space-y-2">
							<div className="flex items-center justify-between">
								<span className="text-sm font-medium">API Version</span>
								<Settings className="h-4 w-4 text-muted-foreground" />
							</div>
							<div className="text-2xl font-bold">
								{basicHealthQuery.data?.data?.version || "Unknown"}
							</div>
							<div className="text-sm text-muted-foreground">
								Documentation:{" "}
								<a
									href="/api-docs"
									className="text-blue-600 hover:underline"
									target="_blank"
									rel="noopener noreferrer"
								>
									API Docs
								</a>
							</div>
						</div>
					</div>
				</CardContent>
			</Card>

			{/* System Metrics Overview */}
			<SystemMetrics />

			{/* Dependency Monitoring */}
			<DependencyMonitor />

			{/* Detailed Health Checks */}
			<Tabs defaultValue="detailed" className="w-full">
				<TabsList className="grid w-full grid-cols-4">
					<TabsTrigger value="detailed">Detailed</TabsTrigger>
					<TabsTrigger value="liveness">Liveness</TabsTrigger>
					<TabsTrigger value="readiness">Readiness</TabsTrigger>
					<TabsTrigger value="startup">Startup</TabsTrigger>
				</TabsList>

				<TabsContent value="detailed" className="space-y-4">
					<Card>
						<CardHeader>
							<CardTitle className="flex items-center space-x-2">
								<Database className="h-5 w-5" />
								<span>Detailed Health Checks</span>
							</CardTitle>
							<CardDescription>
								Comprehensive dependency and service health validation
							</CardDescription>
						</CardHeader>
						<CardContent>
							{detailedHealthQuery.isLoading ? (
								<div>Loading detailed health checks...</div>
							) : detailedHealthQuery.error ? (
								<Alert>
									<AlertTriangle className="h-4 w-4" />
									<AlertDescription>
										Failed to load detailed health checks:{" "}
										{detailedHealthQuery.error.message}
									</AlertDescription>
								</Alert>
							) : (
								<ScrollArea className="h-[300px]">
									<div className="space-y-4">
										{detailedHealthQuery.data?.data?.checks &&
											Object.entries(detailedHealthQuery.data.data.checks).map(
												([name, check]: [string, ComponentHealth]) => (
													<div
														key={name}
														className="flex items-start justify-between p-4 border rounded-lg"
													>
														<div className="space-y-1">
															<div className="flex items-center space-x-2">
																{getStatusIcon(check.status)}
																<h4 className="font-semibold capitalize">
																	{name}
																</h4>
																<Badge
																	variant={
																		check.status === "healthy"
																			? "default"
																			: "destructive"
																	}
																>
																	{check.status}
																</Badge>
															</div>
															<p className="text-sm text-muted-foreground">
																{check.message}
															</p>
															{renderDetails(check.details)}
														</div>
													</div>
												),
											)}
									</div>
								</ScrollArea>
							)}
						</CardContent>
					</Card>
				</TabsContent>

				<TabsContent value="liveness" className="space-y-4">
					<Card>
						<CardHeader>
							<CardTitle>Liveness Probe</CardTitle>
							<CardDescription>
								Kubernetes liveness probe - determines if the application is
								running
							</CardDescription>
						</CardHeader>
						<CardContent>
							{livenessQuery.isLoading ? (
								<div>Loading liveness status...</div>
							) : livenessQuery.error ? (
								<Alert>
									<AlertTriangle className="h-4 w-4" />
									<AlertDescription>
										Liveness probe failed: {livenessQuery.error.message}
									</AlertDescription>
								</Alert>
							) : (
								<div className="space-y-4">
									<div className="flex items-center space-x-4">
										{getStatusIcon(getProbeStatus(livenessQuery.data?.data))}
										<div>
											<div className="font-semibold">
												Status: {getProbeStatus(livenessQuery.data?.data)}
											</div>
											<div className="text-sm text-muted-foreground">
												Last checked:{" "}
												{getProbeTimestamp(livenessQuery.data?.data)
													? new Date(
															getProbeTimestamp(livenessQuery.data?.data) || "",
														).toLocaleString()
													: "Unknown"}
											</div>
										</div>
									</div>
								</div>
							)}
						</CardContent>
					</Card>
				</TabsContent>

				<TabsContent value="readiness" className="space-y-4">
					<Card>
						<CardHeader>
							<CardTitle>Readiness Probe</CardTitle>
							<CardDescription>
								Kubernetes readiness probe - determines if the application is
								ready to serve traffic
							</CardDescription>
						</CardHeader>
						<CardContent>
							{readinessQuery.isLoading ? (
								<div>Loading readiness status...</div>
							) : readinessQuery.error ? (
								<Alert>
									<AlertTriangle className="h-4 w-4" />
									<AlertDescription>
										Readiness probe failed: {readinessQuery.error.message}
									</AlertDescription>
								</Alert>
							) : (
								<div className="space-y-4">
									<div className="flex items-center space-x-4">
										{getStatusIcon(getProbeStatus(readinessQuery.data?.data))}
										<div>
											<div className="font-semibold">
												Status: {getProbeStatus(readinessQuery.data?.data)}
											</div>
											<div className="text-sm text-muted-foreground">
												Last checked:{" "}
												{getProbeTimestamp(readinessQuery.data?.data)
													? new Date(
															getProbeTimestamp(readinessQuery.data?.data) ||
																"",
														).toLocaleString()
													: "Unknown"}
											</div>
										</div>
									</div>
									{getProbeChecks(readinessQuery.data?.data) && <Separator />}
									{getProbeChecks(readinessQuery.data?.data) &&
										Object.entries(
											getProbeChecks(readinessQuery.data?.data) || {},
										).map(([name, check]: [string, ComponentHealth]) => (
											<div
												key={name}
												className="flex items-start justify-between p-3 border rounded"
											>
												<div className="space-y-1">
													<div className="flex items-center space-x-2">
														{getStatusIcon(check.status)}
														<h4 className="font-medium capitalize">{name}</h4>
														<Badge
															variant={
																check.status === "healthy"
																	? "default"
																	: "destructive"
															}
														>
															{check.status}
														</Badge>
													</div>
													<p className="text-sm text-muted-foreground">
														{check.message}
													</p>
													{renderCheckDetails(check.details)}
												</div>
											</div>
										))}
								</div>
							)}
						</CardContent>
					</Card>
				</TabsContent>

				<TabsContent value="startup" className="space-y-4">
					<Card>
						<CardHeader>
							<CardTitle>Startup Probe</CardTitle>
							<CardDescription>
								Kubernetes startup probe - determines if the application has
								started successfully
							</CardDescription>
						</CardHeader>
						<CardContent>
							{startupQuery.isLoading ? (
								<div>Loading startup status...</div>
							) : startupQuery.error ? (
								<Alert>
									<AlertTriangle className="h-4 w-4" />
									<AlertDescription>
										Startup probe failed: {startupQuery.error.message}
									</AlertDescription>
								</Alert>
							) : (
								<div className="space-y-4">
									<div className="flex items-center space-x-4">
										{getStatusIcon(getProbeStatus(startupQuery.data?.data))}
										<div>
											<div className="font-semibold">
												Status: {getProbeStatus(startupQuery.data?.data)}
											</div>
											<div className="text-sm text-muted-foreground">
												Last checked:{" "}
												{getProbeTimestamp(startupQuery.data?.data)
													? new Date(
															getProbeTimestamp(startupQuery.data?.data) || "",
														).toLocaleString()
													: "Unknown"}
											</div>
										</div>
									</div>
									{getProbeChecks(startupQuery.data?.data) && <Separator />}
									{getProbeChecks(startupQuery.data?.data) &&
										Object.entries(
											getProbeChecks(startupQuery.data?.data) || {},
										).map(([name, check]: [string, ComponentHealth]) => (
											<div
												key={name}
												className="flex items-start justify-between p-3 border rounded"
											>
												<div className="space-y-1">
													<div className="flex items-center space-x-2">
														{getStatusIcon(check.status)}
														<h4 className="font-medium capitalize">{name}</h4>
														<Badge
															variant={
																check.status === "healthy"
																	? "default"
																	: "destructive"
															}
														>
															{check.status}
														</Badge>
													</div>
													<p className="text-sm text-muted-foreground">
														{check.message}
													</p>
													{renderCheckDetails(check.details)}
												</div>
											</div>
										))}
								</div>
							)}
						</CardContent>
					</Card>
				</TabsContent>
			</Tabs>
		</div>
	);
}
