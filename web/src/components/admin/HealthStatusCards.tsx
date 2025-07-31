import { Badge } from "@/components/ui/badge";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api/client";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import {
	AlertTriangle,
	CheckCircle2,
	Database,
	ExternalLink,
	Heart,
	Server,
	Shield,
	XCircle,
	Zap,
} from "lucide-react";
import { memo } from "react";

import type { components } from "@/types/api";

type ComponentHealth = components["schemas"]["ComponentHealth"];

// Type for probe responses that return unknown data
interface ProbeResponse {
	probe?: string;
	status?: string;
	timestamp?: string;
	checks?: Record<string, ComponentHealth>;
}

// Type guards for API responses
const isProbeResponse = (data: unknown): data is ProbeResponse => {
	return (
		typeof data === "object" &&
		data !== null &&
		typeof (data as ProbeResponse).status === "string"
	);
};

const getProbeStatus = (data: unknown): string => {
	if (isProbeResponse(data)) {
		return data.status || "unknown";
	}
	return "unknown";
};

export const HealthStatusCards = memo(function HealthStatusCards() {
	const basicHealthQuery = useQuery({
		queryKey: ["health", "basic"],
		queryFn: () => apiClient.getHealth(),
		refetchInterval: 30000,
	});

	const detailedHealthQuery = useQuery({
		queryKey: ["health", "detailed"],
		queryFn: () => apiClient.getDetailedHealth(),
		refetchInterval: 30000,
	});

	const livenessQuery = useQuery({
		queryKey: ["health", "liveness"],
		queryFn: () => apiClient.getLivenessProbe(),
		refetchInterval: 15000,
	});

	const readinessQuery = useQuery({
		queryKey: ["health", "readiness"],
		queryFn: () => apiClient.getReadinessProbe(),
		refetchInterval: 15000,
	});

	const getStatusIcon = (status: string) => {
		switch (status?.toLowerCase()) {
			case "healthy":
			case "alive":
			case "ready":
				return <CheckCircle2 className="h-4 w-4 text-green-600" />;
			case "unhealthy":
			case "down":
				return <XCircle className="h-4 w-4 text-red-600" />;
			case "degraded":
				return <AlertTriangle className="h-4 w-4 text-yellow-600" />;
			default:
				return <Heart className="h-4 w-4 text-gray-600" />;
		}
	};

	const getStatusBadge = (status: string) => {
		switch (status?.toLowerCase()) {
			case "healthy":
			case "alive":
			case "ready":
				return <Badge className="bg-green-100 text-green-800">{status}</Badge>;
			case "unhealthy":
			case "down":
				return <Badge variant="destructive">{status}</Badge>;
			case "degraded":
				return (
					<Badge className="bg-yellow-100 text-yellow-800">{status}</Badge>
				);
			default:
				return <Badge variant="secondary">{status || "Unknown"}</Badge>;
		}
	};

	const formatUptime = (uptime: number) => {
		const hours = Math.floor(uptime / 3600);
		const minutes = Math.floor((uptime % 3600) / 60);
		return `${hours}h ${minutes}m`;
	};

	const getDatabaseStatus = () => {
		const dbCheck = detailedHealthQuery.data?.data?.checks?.database;
		return {
			status: dbCheck?.status || "unknown",
			message: dbCheck?.message || "Checking connection...",
		};
	};

	const getOverallHealthScore = () => {
		if (!detailedHealthQuery.data?.data?.checks) return 0;

		const checks = Object.values(detailedHealthQuery.data.data.checks);
		const healthyChecks = checks.filter(
			(check: ComponentHealth) => check.status === "healthy",
		).length;
		return Math.round((healthyChecks / checks.length) * 100);
	};

	if (basicHealthQuery.isLoading || detailedHealthQuery.isLoading) {
		return (
			<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
				<Skeleton className="h-32" />
				<Skeleton className="h-32" />
				<Skeleton className="h-32" />
				<Skeleton className="h-32" />
			</div>
		);
	}

	const dbStatus = getDatabaseStatus();

	return (
		<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
			{/* Application Health */}
			<Card>
				<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle className="text-sm font-medium">Application</CardTitle>
					<Heart className="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div className="flex items-center space-x-2 mb-2">
						{getStatusIcon(basicHealthQuery.data?.data?.status || "unknown")}
						<div className="text-2xl font-bold">
							{basicHealthQuery.data?.data?.status || "Unknown"}
						</div>
					</div>
					{getStatusBadge(basicHealthQuery.data?.data?.status || "unknown")}
					<p className="text-xs text-muted-foreground mt-2">
						v{basicHealthQuery.data?.data?.version || "Unknown"}
					</p>
					<p className="text-xs text-muted-foreground">
						Uptime:{" "}
						{basicHealthQuery.data?.data?.uptime
							? formatUptime(basicHealthQuery.data.data.uptime)
							: "Unknown"}
					</p>
				</CardContent>
			</Card>

			{/* Database Health */}
			<Card>
				<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle className="text-sm font-medium">Database</CardTitle>
					<Database className="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div className="flex items-center space-x-2 mb-2">
						{getStatusIcon(dbStatus.status)}
						<div className="text-2xl font-bold capitalize">
							{dbStatus.status}
						</div>
					</div>
					{getStatusBadge(dbStatus.status)}
					<p className="text-xs text-muted-foreground mt-2">
						{dbStatus.message}
					</p>
				</CardContent>
			</Card>

			{/* Liveness Probe */}
			<Card>
				<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle className="text-sm font-medium">Liveness</CardTitle>
					<Zap className="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div className="flex items-center space-x-2 mb-2">
						{getStatusIcon(
							getProbeStatus(livenessQuery.data?.data) || "unknown",
						)}
						<div className="text-2xl font-bold capitalize">
							{getProbeStatus(livenessQuery.data?.data) || "Unknown"}
						</div>
					</div>
					{getStatusBadge(
						getProbeStatus(livenessQuery.data?.data) || "unknown",
					)}
					<p className="text-xs text-muted-foreground mt-2">
						Kubernetes liveness probe
					</p>
				</CardContent>
			</Card>

			{/* Readiness Probe */}
			<Card>
				<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle className="text-sm font-medium">Readiness</CardTitle>
					<Shield className="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div className="flex items-center space-x-2 mb-2">
						{getStatusIcon(
							getProbeStatus(readinessQuery.data?.data) || "unknown",
						)}
						<div className="text-2xl font-bold capitalize">
							{getProbeStatus(readinessQuery.data?.data) || "Unknown"}
						</div>
					</div>
					{getStatusBadge(
						getProbeStatus(readinessQuery.data?.data) || "unknown",
					)}
					<p className="text-xs text-muted-foreground mt-2">
						Kubernetes readiness probe
					</p>
				</CardContent>
			</Card>

			{/* Health Summary Card */}
			<Card className="md:col-span-2 lg:col-span-4">
				<CardHeader>
					<div className="flex items-center justify-between">
						<div>
							<CardTitle className="flex items-center space-x-2">
								<Server className="h-5 w-5" />
								<span>System Health Overview</span>
							</CardTitle>
							<CardDescription>
								Overall system health score: {getOverallHealthScore()}% of
								dependencies are healthy
							</CardDescription>
						</div>
						<Link
							to="/admin/health"
							className="flex items-center space-x-1 text-sm text-blue-600 hover:text-blue-800"
						>
							<span>View Details</span>
							<ExternalLink className="h-4 w-4" />
						</Link>
					</div>
				</CardHeader>
				<CardContent>
					<div className="grid grid-cols-2 md:grid-cols-4 gap-4">
						<div className="text-center">
							<div className="text-lg font-semibold text-green-600">
								{detailedHealthQuery.data?.data?.checks
									? Object.values(detailedHealthQuery.data.data.checks).filter(
											(c: ComponentHealth) => c.status === "healthy",
										).length
									: 0}
							</div>
							<div className="text-sm text-muted-foreground">Healthy</div>
						</div>
						<div className="text-center">
							<div className="text-lg font-semibold text-yellow-600">
								{detailedHealthQuery.data?.data?.checks
									? Object.values(detailedHealthQuery.data.data.checks).filter(
											(c: ComponentHealth) => c.status === "degraded",
										).length
									: 0}
							</div>
							<div className="text-sm text-muted-foreground">Degraded</div>
						</div>
						<div className="text-center">
							<div className="text-lg font-semibold text-red-600">
								{detailedHealthQuery.data?.data?.checks
									? Object.values(detailedHealthQuery.data.data.checks).filter(
											(c: ComponentHealth) => c.status === "unhealthy",
										).length
									: 0}
							</div>
							<div className="text-sm text-muted-foreground">Unhealthy</div>
						</div>
						<div className="text-center">
							<div className="text-lg font-semibold text-blue-600">
								{getOverallHealthScore()}%
							</div>
							<div className="text-sm text-muted-foreground">Score</div>
						</div>
					</div>
					<div className="mt-4 text-xs text-muted-foreground">
						Last updated:{" "}
						{detailedHealthQuery.data?.data?.timestamp
							? new Date(
									detailedHealthQuery.data.data.timestamp,
								).toLocaleString()
							: "Unknown"}
					</div>
				</CardContent>
			</Card>
		</div>
	);
});
