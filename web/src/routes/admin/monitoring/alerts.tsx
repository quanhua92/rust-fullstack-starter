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
import { Skeleton } from "@/components/ui/skeleton";
import { Textarea } from "@/components/ui/textarea";
import { useCurrentUser, useMonitoringAlerts } from "@/hooks/useApiQueries";
import { apiClient } from "@/lib/api/client";
import { useAuth } from "@/lib/auth/context";
import { getRoleColorClasses, getRoleDisplayName } from "@/lib/rbac/types";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import {
	AlertTriangle,
	Bell,
	Edit,
	Plus,
	RefreshCw,
	Shield,
	Trash2,
} from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

function AlertsManagement() {
	const { isModeratorOrHigher } = useAuth();
	const { data: currentUser } = useCurrentUser(30000);
	const queryClient = useQueryClient();

	// Create alert form state
	const [showCreateForm, setShowCreateForm] = useState(false);
	const [editingAlert, setEditingAlert] = useState<any>(null);
	const [createForm, setCreateForm] = useState({
		name: "",
		description: "",
		query: "",
		threshold_value: "",
	});

	// Fetch alerts (Moderator+ only)
	const { data: alerts, isLoading, refetch } = useMonitoringAlerts(15000);

	// Create alert mutation
	const createAlertMutation = useMutation({
		mutationFn: async (data: typeof createForm) => {
			const payload = {
				name: data.name,
				description: data.description || undefined,
				query: data.query,
				threshold_value: Number.parseFloat(data.threshold_value),
			};
			return apiClient.createAlert(payload);
		},
		onSuccess: () => {
			toast.success("Alert created successfully");
			setShowCreateForm(false);
			setCreateForm({
				name: "",
				description: "",
				query: "",
				threshold_value: "",
			});
			queryClient.invalidateQueries({ queryKey: ["monitoring", "alerts"] });
		},
		onError: (error) => {
			toast.error(`Failed to create alert: ${error.message}`);
		},
	});

	// Update alert mutation
	const updateAlertMutation = useMutation({
		mutationFn: async (data: {
			id: string;
			updates: Partial<typeof createForm>;
		}) => {
			const payload = {
				name: data.updates.name,
				description: data.updates.description || undefined,
				query: data.updates.query,
				threshold_value: data.updates.threshold_value
					? Number.parseFloat(data.updates.threshold_value)
					: undefined,
			};
			return apiClient.updateAlert(data.id, payload);
		},
		onSuccess: () => {
			toast.success("Alert updated successfully");
			setEditingAlert(null);
			queryClient.invalidateQueries({ queryKey: ["monitoring", "alerts"] });
		},
		onError: (error) => {
			toast.error(`Failed to update alert: ${error.message}`);
		},
	});

	// Delete alert mutation
	const deleteAlertMutation = useMutation({
		mutationFn: (id: string) => apiClient.deleteAlert(id),
		onSuccess: () => {
			toast.success("Alert deleted successfully");
			queryClient.invalidateQueries({ queryKey: ["monitoring", "alerts"] });
		},
		onError: (error) => {
			toast.error(`Failed to delete alert: ${error.message}`);
		},
	});

	const handleEdit = (alert: any) => {
		setEditingAlert(alert);
		setCreateForm({
			name: alert.name || "",
			description: alert.description || "",
			query: alert.query || "",
			threshold_value: alert.threshold_value?.toString() || "",
		});
		setShowCreateForm(true);
	};

	const handleSubmit = () => {
		if (editingAlert) {
			updateAlertMutation.mutate({
				id: editingAlert.id,
				updates: createForm,
			});
		} else {
			createAlertMutation.mutate(createForm);
		}
	};

	const handleCancel = () => {
		setShowCreateForm(false);
		setEditingAlert(null);
		setCreateForm({
			name: "",
			description: "",
			query: "",
			threshold_value: "",
		});
	};

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
								You need Moderator or Administrator privileges to manage alerts.
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
							Alert Management
						</h1>
						<p className="text-muted-foreground">
							Create and manage monitoring alert rules and thresholds
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
					<Button
						onClick={() => setShowCreateForm(!showCreateForm)}
						className="flex items-center space-x-2"
					>
						<Plus className="h-4 w-4" />
						<span>Create Alert Rule</span>
					</Button>
					<Button variant="outline" onClick={() => refetch()}>
						<RefreshCw className="h-4 w-4 mr-2" />
						Refresh
					</Button>
				</div>

				{/* Create/Edit Alert Form */}
				{showCreateForm && (
					<Card>
						<CardHeader>
							<CardTitle>
								{editingAlert ? "Edit Alert Rule" : "Create New Alert Rule"}
							</CardTitle>
							<CardDescription>
								{editingAlert
									? "Update the alert rule configuration"
									: "Define conditions and thresholds for monitoring alerts"}
							</CardDescription>
						</CardHeader>
						<CardContent className="space-y-4">
							<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
								<div className="space-y-2">
									<Label htmlFor="name">Alert Name</Label>
									<Input
										id="name"
										placeholder="e.g., High Error Rate, Memory Usage Alert"
										value={createForm.name}
										onChange={(e) =>
											setCreateForm({ ...createForm, name: e.target.value })
										}
									/>
								</div>
								<div className="space-y-2">
									<Label htmlFor="threshold_value">Threshold Value</Label>
									<Input
										id="threshold_value"
										type="number"
										placeholder="e.g., 0.05, 90, 1000"
										value={createForm.threshold_value}
										onChange={(e) =>
											setCreateForm({
												...createForm,
												threshold_value: e.target.value,
											})
										}
									/>
								</div>
							</div>
							<div className="space-y-2">
								<Label htmlFor="description">Description</Label>
								<Input
									id="description"
									placeholder="Brief description of what this alert monitors"
									value={createForm.description}
									onChange={(e) =>
										setCreateForm({
											...createForm,
											description: e.target.value,
										})
									}
								/>
							</div>
							<div className="space-y-2">
								<Label htmlFor="query">Alert Query</Label>
								<Textarea
									id="query"
									placeholder="e.g., error_rate > 0.05, cpu_usage > 90, response_time > 1000"
									value={createForm.query}
									onChange={(e) =>
										setCreateForm({ ...createForm, query: e.target.value })
									}
								/>
								<p className="text-xs text-muted-foreground">
									Define the condition that triggers this alert. Use metric
									names and comparison operators.
								</p>
							</div>
							<div className="flex space-x-2">
								<Button
									onClick={handleSubmit}
									disabled={
										!createForm.name ||
										!createForm.query ||
										!createForm.threshold_value ||
										createAlertMutation.isPending ||
										updateAlertMutation.isPending
									}
								>
									{createAlertMutation.isPending ||
									updateAlertMutation.isPending
										? editingAlert
											? "Updating..."
											: "Creating..."
										: editingAlert
											? "Update Alert"
											: "Create Alert"}
								</Button>
								<Button variant="outline" onClick={handleCancel}>
									Cancel
								</Button>
							</div>
						</CardContent>
					</Card>
				)}

				{/* Alert Configuration Guide */}
				<Card>
					<CardHeader>
						<CardTitle>Alert Configuration Guide</CardTitle>
						<CardDescription>
							Best practices for creating effective monitoring alerts
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="grid grid-cols-1 md:grid-cols-2 gap-6">
							<div className="space-y-3">
								<h4 className="font-medium">Common Alert Types</h4>
								<div className="space-y-2 text-sm">
									<div>
										🔴 <strong>Error Rate:</strong> error_rate &gt; 0.05
									</div>
									<div>
										🟡 <strong>Response Time:</strong> avg_response_time &gt;
										1000
									</div>
									<div>
										🟠 <strong>CPU Usage:</strong> cpu_usage &gt; 80
									</div>
									<div>
										🔵 <strong>Memory Usage:</strong> memory_usage &gt; 90
									</div>
								</div>
							</div>
							<div className="space-y-3">
								<h4 className="font-medium">Alert Best Practices</h4>
								<div className="space-y-2 text-sm text-muted-foreground">
									<div>• Use descriptive names that indicate the problem</div>
									<div>• Set thresholds that avoid false positives</div>
									<div>• Include context in the description</div>
									<div>• Test alert queries before deploying</div>
									<div>• Review and adjust thresholds regularly</div>
								</div>
							</div>
						</div>
					</CardContent>
				</Card>

				{/* Alerts List */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<Bell className="h-5 w-5" />
							<span>Alert Rules</span>
						</CardTitle>
						<CardDescription>
							Configured monitoring alert rules and their current status
						</CardDescription>
					</CardHeader>
					<CardContent>
						{isLoading ? (
							<div className="space-y-4">
								{[...Array(3)].map((_, i) => (
									<Skeleton key={i} className="h-24" />
								))}
							</div>
						) : alerts &&
							Array.isArray(alerts) &&
							(alerts as any[]).length > 0 ? (
							<div className="space-y-4">
								{(alerts as any[]).map((alert: any, index) => (
									<div
										key={alert.id || index}
										className="border rounded-lg p-4 space-y-3"
									>
										<div className="flex items-start justify-between">
											<div className="space-y-1">
												<div className="flex items-center space-x-2">
													<h4 className="font-semibold">{alert.name}</h4>
													<Badge variant="outline">
														{alert.is_active ? "Active" : "Inactive"}
													</Badge>
												</div>
												{alert.description && (
													<p className="text-sm text-muted-foreground">
														{alert.description}
													</p>
												)}
											</div>
											<div className="flex items-center space-x-2">
												<Button
													variant="outline"
													size="sm"
													onClick={() => handleEdit(alert)}
												>
													<Edit className="h-4 w-4" />
												</Button>
												<Button
													variant="outline"
													size="sm"
													onClick={() => deleteAlertMutation.mutate(alert.id)}
													disabled={deleteAlertMutation.isPending}
												>
													<Trash2 className="h-4 w-4" />
												</Button>
											</div>
										</div>
										<div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
											<div>
												<span className="font-medium">Query: </span>
												<code className="bg-gray-100 px-2 py-1 rounded">
													{alert.query}
												</code>
											</div>
											<div>
												<span className="font-medium">Threshold: </span>
												<span className="text-red-600 font-mono">
													{alert.threshold_value}
												</span>
											</div>
										</div>
										{alert.created_at && (
											<div className="text-xs text-muted-foreground">
												Created{" "}
												{formatDistanceToNow(new Date(alert.created_at), {
													addSuffix: true,
												})}
											</div>
										)}
									</div>
								))}
							</div>
						) : (
							<div className="text-center py-8">
								<AlertTriangle className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
								<p className="text-muted-foreground">
									No alert rules configured yet.
									<br />
									Create your first alert rule to start monitoring.
								</p>
							</div>
						)}
					</CardContent>
				</Card>

				{/* Alert Status Info */}
				<Card>
					<CardHeader>
						<CardTitle>Alert System Status</CardTitle>
						<CardDescription>
							Current state of the alert monitoring system
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="grid grid-cols-1 md:grid-cols-3 gap-4">
							<div className="text-center">
								<div className="text-2xl font-bold text-blue-600">
									{alerts ? (alerts as any[]).length : 0}
								</div>
								<p className="text-sm text-muted-foreground">Total Rules</p>
							</div>
							<div className="text-center">
								<div className="text-2xl font-bold text-green-600">
									{alerts
										? (alerts as any[]).filter((a: any) => a.is_active).length
										: 0}
								</div>
								<p className="text-sm text-muted-foreground">Active Rules</p>
							</div>
							<div className="text-center">
								<div className="text-2xl font-bold text-orange-600">0</div>
								<p className="text-sm text-muted-foreground">
									Triggered (Last 24h)
								</p>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/monitoring/alerts")({
	component: AlertsManagement,
});
