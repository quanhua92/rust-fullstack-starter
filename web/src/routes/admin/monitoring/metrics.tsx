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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import {
	useCurrentUser,
	useMonitoringMetrics,
	usePrometheusMetrics,
} from "@/hooks/useApiQueries";
import { apiClient } from "@/lib/api/client";
import { useAuth } from "@/lib/auth/context";
import { getRoleColorClasses, getRoleDisplayName } from "@/lib/rbac/types";
import type { components } from "@/types/api";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import {
	BarChart3,
	Download,
	Plus,
	RefreshCw,
	TrendingUp,
	X,
} from "lucide-react";
import { useState } from "react";
import {
	Line,
	LineChart,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";
import { toast } from "sonner";

// Type definition for Metric
type Metric = NonNullable<
	components["schemas"]["ApiResponse_Vec_Metric"]["data"]
>[number];

function MetricsDashboard() {
	const {} = useAuth();
	const { data: currentUser } = useCurrentUser(30000);
	const queryClient = useQueryClient();

	// Filter state
	const [filters, setFilters] = useState({
		name: "",
		metric_type: "",
		limit: 50,
	});

	// Create metric form state
	const [showCreateForm, setShowCreateForm] = useState(false);
	const [createForm, setCreateForm] = useState({
		name: "",
		metric_type: "counter" as "counter" | "gauge" | "histogram" | "summary",
		value: "",
		labels: "",
	});

	// Fetch metrics with current filters
	const {
		data: metrics,
		isLoading,
		refetch,
	} = useMonitoringMetrics(
		{
			...filters,
			name: filters.name || undefined,
			metric_type:
				(filters.metric_type as
					| "summary"
					| "counter"
					| "gauge"
					| "histogram"
					| undefined) || undefined,
		},
		15000, // 15 second refresh
	);

	// Fetch Prometheus metrics
	const { data: prometheusMetrics, isLoading: isLoadingPrometheus } =
		usePrometheusMetrics(30000);

	// Create metric mutation
	const createMetricMutation = useMutation({
		mutationFn: async (data: typeof createForm) => {
			const payload = {
				name: data.name,
				metric_type: data.metric_type,
				value: Number.parseFloat(data.value),
				labels: data.labels ? parseLabelsString(data.labels) : undefined,
			};
			return apiClient.createMetric(payload);
		},
		onSuccess: () => {
			toast.success("Metric created successfully");
			setShowCreateForm(false);
			setCreateForm({
				name: "",
				metric_type: "counter",
				value: "",
				labels: "",
			});
			queryClient.invalidateQueries({ queryKey: ["monitoring", "metrics"] });
		},
		onError: (error) => {
			toast.error(`Failed to create metric: ${error.message}`);
		},
	});

	// Parse labels string (key:value,key2:value2) to object
	const parseLabelsString = (labelString: string): Record<string, string> => {
		const labels: Record<string, string> = {};
		if (!labelString.trim()) return labels;

		labelString.split(",").forEach((pair) => {
			const [key, value] = pair.split(":");
			if (key && value) {
				labels[key.trim()] = value.trim();
			}
		});
		return labels;
	};

	// Format labels object to display string
	const formatLabels = (labels: Record<string, string>): string => {
		return Object.entries(labels)
			.map(([key, value]) => `${key}:${value}`)
			.join(", ");
	};

	const getMetricTypeColor = (type: string) => {
		switch (type) {
			case "counter":
				return "bg-blue-100 text-blue-800";
			case "gauge":
				return "bg-green-100 text-green-800";
			case "histogram":
				return "bg-purple-100 text-purple-800";
			case "summary":
				return "bg-orange-100 text-orange-800";
			default:
				return "bg-gray-100 text-gray-800";
		}
	};

	const clearFilters = () => {
		setFilters({
			name: "",
			metric_type: "",
			limit: 50,
		});
	};

	const downloadPrometheusMetrics = () => {
		if (prometheusMetrics) {
			const blob = new Blob([prometheusMetrics], { type: "text/plain" });
			const url = URL.createObjectURL(blob);
			const a = document.createElement("a");
			a.href = url;
			a.download = `prometheus-metrics-${new Date().toISOString().split("T")[0]}.txt`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);
		}
	};

	// Generate mock chart data for metrics visualization
	const generateChartData = () => {
		const now = Date.now();
		return Array.from({ length: 10 }, (_, i) => ({
			time: new Date(now - (9 - i) * 60000).toLocaleTimeString(),
			value: Math.random() * 100 + 50,
		}));
	};

	return (
		<AdminLayout>
			<div className="space-y-6">
				{/* Header */}
				<div className="flex items-center justify-between">
					<div>
						<h1 className="text-3xl font-bold tracking-tight">
							Metrics Dashboard
						</h1>
						<p className="text-muted-foreground">
							Performance metrics and time-series data visualization
						</p>
					</div>
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

				{/* Actions */}
				<div className="flex items-center justify-between">
					<div className="flex items-center space-x-2">
						<Button
							onClick={() => setShowCreateForm(!showCreateForm)}
							className="flex items-center space-x-2"
						>
							<Plus className="h-4 w-4" />
							<span>Submit Metric</span>
						</Button>
						<Button variant="outline" onClick={() => refetch()}>
							<RefreshCw className="h-4 w-4 mr-2" />
							Refresh
						</Button>
					</div>
					<Button
						variant="outline"
						onClick={downloadPrometheusMetrics}
						disabled={isLoadingPrometheus}
					>
						<Download className="h-4 w-4 mr-2" />
						Export Prometheus
					</Button>
				</div>

				{/* Create Metric Form */}
				{showCreateForm && (
					<Card>
						<CardHeader>
							<CardTitle>Submit New Metric</CardTitle>
							<CardDescription>
								Submit a performance or business metric to the monitoring system
							</CardDescription>
						</CardHeader>
						<CardContent className="space-y-4">
							<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
								<div className="space-y-2">
									<Label htmlFor="name">Metric Name</Label>
									<Input
										id="name"
										placeholder="e.g., http_requests_total, payment_duration_ms"
										value={createForm.name}
										onChange={(e) =>
											setCreateForm({ ...createForm, name: e.target.value })
										}
									/>
								</div>
								<div className="space-y-2">
									<Label htmlFor="metric_type">Metric Type</Label>
									<Select
										value={createForm.metric_type}
										onValueChange={(
											value: "counter" | "gauge" | "histogram" | "summary",
										) => setCreateForm({ ...createForm, metric_type: value })}
									>
										<SelectTrigger>
											<SelectValue />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value="counter">Counter</SelectItem>
											<SelectItem value="gauge">Gauge</SelectItem>
											<SelectItem value="histogram">Histogram</SelectItem>
											<SelectItem value="summary">Summary</SelectItem>
										</SelectContent>
									</Select>
								</div>
								<div className="space-y-2">
									<Label htmlFor="value">Value</Label>
									<Input
										id="value"
										type="number"
										placeholder="123.45"
										value={createForm.value}
										onChange={(e) =>
											setCreateForm({ ...createForm, value: e.target.value })
										}
									/>
								</div>
								<div className="space-y-2">
									<Label htmlFor="labels">Labels</Label>
									<Input
										id="labels"
										placeholder="method:POST,status:200"
										value={createForm.labels}
										onChange={(e) =>
											setCreateForm({ ...createForm, labels: e.target.value })
										}
									/>
								</div>
							</div>
							<div className="flex space-x-2">
								<Button
									onClick={() => createMetricMutation.mutate(createForm)}
									disabled={
										!createForm.name ||
										!createForm.value ||
										createMetricMutation.isPending
									}
								>
									{createMetricMutation.isPending
										? "Submitting..."
										: "Submit Metric"}
								</Button>
								<Button
									variant="outline"
									onClick={() => setShowCreateForm(false)}
								>
									Cancel
								</Button>
							</div>
						</CardContent>
					</Card>
				)}

				{/* Metric Type Guide */}
				<Card>
					<CardHeader>
						<CardTitle>Metric Types Guide</CardTitle>
						<CardDescription>
							Understanding different metric types and their use cases
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
							<div className="space-y-2">
								<Badge className="bg-blue-100 text-blue-800">Counter</Badge>
								<p className="text-sm">
									Monotonically increasing values (e.g., request count, errors)
								</p>
							</div>
							<div className="space-y-2">
								<Badge className="bg-green-100 text-green-800">Gauge</Badge>
								<p className="text-sm">
									Values that can go up or down (e.g., CPU usage, memory)
								</p>
							</div>
							<div className="space-y-2">
								<Badge className="bg-purple-100 text-purple-800">
									Histogram
								</Badge>
								<p className="text-sm">
									Distribution of values (e.g., request duration, response size)
								</p>
							</div>
							<div className="space-y-2">
								<Badge className="bg-orange-100 text-orange-800">Summary</Badge>
								<p className="text-sm">
									Quantiles over sliding time windows (e.g., response times)
								</p>
							</div>
						</div>
					</CardContent>
				</Card>

				{/* Filters */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<BarChart3 className="h-5 w-5" />
							<span>Metric Filters</span>
						</CardTitle>
						<CardDescription>Filter metrics by name or type</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="grid grid-cols-1 md:grid-cols-3 gap-4">
							<div className="space-y-2">
								<Label>Metric Name</Label>
								<Input
									placeholder="Filter by name"
									value={filters.name}
									onChange={(e) =>
										setFilters({ ...filters, name: e.target.value })
									}
								/>
							</div>
							<div className="space-y-2">
								<Label>Metric Type</Label>
								<Select
									value={filters.metric_type}
									onValueChange={(value) =>
										setFilters({ ...filters, metric_type: value })
									}
								>
									<SelectTrigger>
										<SelectValue placeholder="All types" />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value="">All types</SelectItem>
										<SelectItem value="counter">Counter</SelectItem>
										<SelectItem value="gauge">Gauge</SelectItem>
										<SelectItem value="histogram">Histogram</SelectItem>
										<SelectItem value="summary">Summary</SelectItem>
									</SelectContent>
								</Select>
							</div>
							<div className="flex items-end">
								<Button variant="outline" onClick={clearFilters}>
									<X className="h-4 w-4 mr-2" />
									Clear Filters
								</Button>
							</div>
						</div>
					</CardContent>
				</Card>

				{/* Sample Chart */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<TrendingUp className="h-5 w-5" />
							<span>Metrics Visualization</span>
						</CardTitle>
						<CardDescription>
							Sample time-series chart (last 10 minutes)
						</CardDescription>
					</CardHeader>
					<CardContent>
						<ResponsiveContainer width="100%" height={300}>
							<LineChart data={generateChartData()}>
								<XAxis dataKey="time" />
								<YAxis />
								<Tooltip />
								<Line
									type="monotone"
									dataKey="value"
									stroke="#3b82f6"
									strokeWidth={2}
									dot={{ fill: "#3b82f6" }}
								/>
							</LineChart>
						</ResponsiveContainer>
					</CardContent>
				</Card>

				{/* Metrics List */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<BarChart3 className="h-5 w-5" />
							<span>Recent Metrics</span>
						</CardTitle>
						<CardDescription>
							Latest performance metrics matching your filters
						</CardDescription>
					</CardHeader>
					<CardContent>
						{isLoading ? (
							<div className="space-y-4">
								{[...Array(5)].map((_, i) => (
									<Skeleton key={i} className="h-16" />
								))}
							</div>
						) : metrics && Array.isArray(metrics) && metrics.length > 0 ? (
							<div className="space-y-4">
								{metrics.map((metric: Metric, index) => (
									<div
										key={metric.id || index}
										className="border rounded-lg p-4 space-y-2"
									>
										<div className="flex items-start justify-between">
											<div className="flex items-center space-x-2">
												<Badge
													className={getMetricTypeColor(metric.metric_type)}
												>
													{metric.metric_type}
												</Badge>
												<span className="font-medium">{metric.name}</span>
												<span className="text-lg font-bold text-blue-600">
													{metric.value}
												</span>
											</div>
											<span className="text-sm text-muted-foreground">
												{metric.recorded_at
													? formatDistanceToNow(new Date(metric.recorded_at), {
															addSuffix: true,
														})
													: "Unknown time"}
											</span>
										</div>
										{metric.labels && Object.keys(metric.labels).length > 0 && (
											<div className="flex items-center space-x-2">
												<span className="text-xs text-muted-foreground">
													Labels:
												</span>
												<span className="text-xs bg-gray-100 px-2 py-1 rounded">
													{formatLabels(metric.labels)}
												</span>
											</div>
										)}
									</div>
								))}
							</div>
						) : (
							<div className="text-center py-8">
								<BarChart3 className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
								<p className="text-muted-foreground">
									No metrics found matching your filters.
									<br />
									Try adjusting your filters or submit a new metric.
								</p>
							</div>
						)}
					</CardContent>
				</Card>

				{/* Prometheus Export Preview */}
				{prometheusMetrics && (
					<Card>
						<CardHeader>
							<CardTitle>Prometheus Export Preview</CardTitle>
							<CardDescription>
								Sample of Prometheus-format metrics export
							</CardDescription>
						</CardHeader>
						<CardContent>
							<div className="bg-gray-50 p-4 rounded-lg overflow-x-auto">
								<pre className="text-xs">
									{prometheusMetrics.split("\n").slice(0, 20).join("\n")}
									{prometheusMetrics.split("\n").length > 20 &&
										"\n... (truncated)"}
								</pre>
							</div>
						</CardContent>
					</Card>
				)}
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/monitoring/metrics")({
	component: MetricsDashboard,
});
