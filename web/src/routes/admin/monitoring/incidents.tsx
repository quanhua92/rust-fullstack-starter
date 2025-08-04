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
import { Textarea } from "@/components/ui/textarea";
import {
	useCurrentUser,
	useMonitoringIncidentTimeline,
	useMonitoringIncidents,
} from "@/hooks/useApiQueries";
import { apiClient } from "@/lib/api/client";
// import { useAuth } from "@/lib/auth/context"; // Not currently needed
import { getRoleColorClasses, getRoleDisplayName } from "@/lib/rbac/types";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import {
	Clock,
	Edit,
	Eye,
	Plus,
	RefreshCw,
	Shield,
	Clock as Timeline,
	User,
} from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

function IncidentsTracking() {
	// Auth context (not currently needed since incidents are accessible to all authenticated users)
	// const { isModeratorOrHigher } = useAuth();
	const { data: currentUser } = useCurrentUser(30000);
	const queryClient = useQueryClient();

	// State management
	const [showCreateForm, setShowCreateForm] = useState(false);
	const [selectedIncident, setSelectedIncident] = useState<any>(null);
	const [showTimeline, setShowTimeline] = useState(false);
	const [editingIncident, setEditingIncident] = useState<any>(null);

	// Create incident form state
	const [createForm, setCreateForm] = useState({
		title: "",
		description: "",
		severity: "medium" as "low" | "medium" | "high" | "critical",
		assigned_to: "",
	});

	// Update incident form state
	const [updateForm, setUpdateForm] = useState({
		status: "open" as "open" | "investigating" | "resolved" | "closed",
		root_cause: "",
		assigned_to: "",
	});

	// Fetch incidents
	const {
		data: incidents,
		isLoading,
		refetch,
	} = useMonitoringIncidents(undefined, 15000);

	// Fetch incident timeline when selected
	const { data: timeline, isLoading: isLoadingTimeline } =
		useMonitoringIncidentTimeline(selectedIncident?.id || "", undefined);

	// Create incident mutation
	const createIncidentMutation = useMutation({
		mutationFn: async (data: typeof createForm) => {
			const payload = {
				title: data.title,
				description: data.description || undefined,
				severity: data.severity,
				assigned_to: data.assigned_to || undefined,
			};
			return apiClient.createIncident(payload);
		},
		onSuccess: () => {
			toast.success("Incident created successfully");
			setShowCreateForm(false);
			setCreateForm({
				title: "",
				description: "",
				severity: "medium",
				assigned_to: "",
			});
			queryClient.invalidateQueries({ queryKey: ["monitoring", "incidents"] });
		},
		onError: (error) => {
			toast.error(`Failed to create incident: ${error.message}`);
		},
	});

	// Update incident mutation
	const updateIncidentMutation = useMutation({
		mutationFn: async (data: {
			id: string;
			updates: Partial<typeof updateForm>;
		}) => {
			const payload = {
				status: data.updates.status,
				root_cause: data.updates.root_cause || undefined,
				assigned_to: data.updates.assigned_to || undefined,
			};
			return apiClient.updateIncident(data.id, payload);
		},
		onSuccess: () => {
			toast.success("Incident updated successfully");
			setEditingIncident(null);
			queryClient.invalidateQueries({ queryKey: ["monitoring", "incidents"] });
		},
		onError: (error) => {
			toast.error(`Failed to update incident: ${error.message}`);
		},
	});

	const getSeverityColor = (severity: string) => {
		switch (severity) {
			case "low":
				return "bg-blue-100 text-blue-800";
			case "medium":
				return "bg-yellow-100 text-yellow-800";
			case "high":
				return "bg-orange-100 text-orange-800";
			case "critical":
				return "bg-red-100 text-red-800";
			default:
				return "bg-gray-100 text-gray-800";
		}
	};

	const getStatusColor = (status: string) => {
		switch (status) {
			case "open":
				return "bg-red-100 text-red-800";
			case "investigating":
				return "bg-yellow-100 text-yellow-800";
			case "resolved":
				return "bg-green-100 text-green-800";
			case "closed":
				return "bg-gray-100 text-gray-800";
			default:
				return "bg-gray-100 text-gray-800";
		}
	};

	const handleEdit = (incident: any) => {
		setEditingIncident(incident);
		setUpdateForm({
			status: incident.status || "open",
			root_cause: incident.root_cause || "",
			assigned_to: incident.assigned_to || "",
		});
	};

	const handleViewTimeline = (incident: any) => {
		setSelectedIncident(incident);
		setShowTimeline(true);
	};

	const handleUpdateSubmit = () => {
		if (editingIncident) {
			updateIncidentMutation.mutate({
				id: editingIncident.id,
				updates: updateForm,
			});
		}
	};

	return (
		<AdminLayout>
			<div className="space-y-6">
				{/* Header */}
				<div className="flex items-center justify-between">
					<div>
						<h1 className="text-3xl font-bold tracking-tight">
							Incident Tracking
						</h1>
						<p className="text-muted-foreground">
							Manage incidents and track resolution with timeline correlation
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
						<span>Create Incident</span>
					</Button>
					<Button variant="outline" onClick={() => refetch()}>
						<RefreshCw className="h-4 w-4 mr-2" />
						Refresh
					</Button>
				</div>

				{/* Create Incident Form */}
				{showCreateForm && (
					<Card>
						<CardHeader>
							<CardTitle>Create New Incident</CardTitle>
							<CardDescription>
								Report a new incident for tracking and resolution
							</CardDescription>
						</CardHeader>
						<CardContent className="space-y-4">
							<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
								<div className="space-y-2">
									<Label htmlFor="title">Incident Title</Label>
									<Input
										id="title"
										placeholder="e.g., Payment Gateway Degradation"
										value={createForm.title}
										onChange={(e) =>
											setCreateForm({ ...createForm, title: e.target.value })
										}
									/>
								</div>
								<div className="space-y-2">
									<Label htmlFor="severity">Severity</Label>
									<Select
										value={createForm.severity}
										onValueChange={(
											value: "low" | "medium" | "high" | "critical",
										) => setCreateForm({ ...createForm, severity: value })}
									>
										<SelectTrigger>
											<SelectValue />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value="low">Low</SelectItem>
											<SelectItem value="medium">Medium</SelectItem>
											<SelectItem value="high">High</SelectItem>
											<SelectItem value="critical">Critical</SelectItem>
										</SelectContent>
									</Select>
								</div>
								<div className="space-y-2">
									<Label htmlFor="assigned_to">Assigned To</Label>
									<Input
										id="assigned_to"
										placeholder="User ID (optional)"
										value={createForm.assigned_to}
										onChange={(e) =>
											setCreateForm({
												...createForm,
												assigned_to: e.target.value,
											})
										}
									/>
								</div>
							</div>
							<div className="space-y-2">
								<Label htmlFor="description">Description</Label>
								<Textarea
									id="description"
									placeholder="Detailed description of the incident"
									value={createForm.description}
									onChange={(e) =>
										setCreateForm({
											...createForm,
											description: e.target.value,
										})
									}
								/>
							</div>
							<div className="flex space-x-2">
								<Button
									onClick={() => createIncidentMutation.mutate(createForm)}
									disabled={
										!createForm.title || createIncidentMutation.isPending
									}
								>
									{createIncidentMutation.isPending
										? "Creating..."
										: "Create Incident"}
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

				{/* Edit Incident Form */}
				{editingIncident && (
					<Card>
						<CardHeader>
							<CardTitle>Update Incident: {editingIncident.title}</CardTitle>
							<CardDescription>
								Update incident status and resolution details
							</CardDescription>
						</CardHeader>
						<CardContent className="space-y-4">
							<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
								<div className="space-y-2">
									<Label htmlFor="status">Status</Label>
									<Select
										value={updateForm.status}
										onValueChange={(
											value: "open" | "investigating" | "resolved" | "closed",
										) => setUpdateForm({ ...updateForm, status: value })}
									>
										<SelectTrigger>
											<SelectValue />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value="open">Open</SelectItem>
											<SelectItem value="investigating">
												Investigating
											</SelectItem>
											<SelectItem value="resolved">Resolved</SelectItem>
											<SelectItem value="closed">Closed</SelectItem>
										</SelectContent>
									</Select>
								</div>
								<div className="space-y-2">
									<Label htmlFor="assigned_to_update">Assigned To</Label>
									<Input
										id="assigned_to_update"
										placeholder="User ID"
										value={updateForm.assigned_to}
										onChange={(e) =>
											setUpdateForm({
												...updateForm,
												assigned_to: e.target.value,
											})
										}
									/>
								</div>
							</div>
							<div className="space-y-2">
								<Label htmlFor="root_cause">Root Cause</Label>
								<Textarea
									id="root_cause"
									placeholder="Description of the root cause and resolution"
									value={updateForm.root_cause}
									onChange={(e) =>
										setUpdateForm({ ...updateForm, root_cause: e.target.value })
									}
								/>
							</div>
							<div className="flex space-x-2">
								<Button
									onClick={handleUpdateSubmit}
									disabled={updateIncidentMutation.isPending}
								>
									{updateIncidentMutation.isPending
										? "Updating..."
										: "Update Incident"}
								</Button>
								<Button
									variant="outline"
									onClick={() => setEditingIncident(null)}
								>
									Cancel
								</Button>
							</div>
						</CardContent>
					</Card>
				)}

				{/* Incident Timeline Modal */}
				{showTimeline && selectedIncident && (
					<Card>
						<CardHeader>
							<CardTitle className="flex items-center space-x-2">
								<Timeline className="h-5 w-5" />
								<span>Incident Timeline: {selectedIncident.title}</span>
							</CardTitle>
							<CardDescription>
								Correlated events and activities related to this incident
							</CardDescription>
						</CardHeader>
						<CardContent>
							{isLoadingTimeline ? (
								<div className="space-y-4">
									{[...Array(5)].map((_, i) => (
										<Skeleton key={i} className="h-16" />
									))}
								</div>
							) : timeline &&
								(timeline as any).entries &&
								(timeline as any).entries.length > 0 ? (
								<div className="space-y-4 max-h-96 overflow-y-auto">
									{(timeline as any).entries.map(
										(entry: any, index: number) => (
											<div
												key={entry.id || index}
												className="border-l-2 border-blue-200 pl-4 pb-4"
											>
												<div className="flex items-start justify-between">
													<div className="space-y-1">
														<div className="flex items-center space-x-2">
															<Badge variant="outline">
																{entry.event_type}
															</Badge>
															<span className="font-medium">
																{entry.source}
															</span>
														</div>
														<p className="text-sm">{entry.message}</p>
														{entry.tags &&
															Object.keys(entry.tags).length > 0 && (
																<div className="text-xs text-muted-foreground">
																	Tags:{" "}
																	{Object.entries(entry.tags)
																		.map(([k, v]) => `${k}:${v}`)
																		.join(", ")}
																</div>
															)}
													</div>
													<span className="text-xs text-muted-foreground">
														{formatDistanceToNow(new Date(entry.recorded_at), {
															addSuffix: true,
														})}
													</span>
												</div>
											</div>
										),
									)}
								</div>
							) : (
								<p className="text-muted-foreground text-center py-4">
									No timeline events found for this incident.
								</p>
							)}
							<div className="flex justify-end mt-4">
								<Button
									variant="outline"
									onClick={() => setShowTimeline(false)}
								>
									Close Timeline
								</Button>
							</div>
						</CardContent>
					</Card>
				)}

				{/* Incidents List */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<Shield className="h-5 w-5" />
							<span>Recent Incidents</span>
						</CardTitle>
						<CardDescription>
							Active and resolved incidents with current status
						</CardDescription>
					</CardHeader>
					<CardContent>
						{isLoading ? (
							<div className="space-y-4">
								{[...Array(3)].map((_, i) => (
									<Skeleton key={i} className="h-24" />
								))}
							</div>
						) : incidents &&
							Array.isArray(incidents) &&
							incidents.length > 0 ? (
							<div className="space-y-4">
								{incidents.map((incident: any, index) => (
									<div
										key={incident.id || index}
										className="border rounded-lg p-4 space-y-3"
									>
										<div className="flex items-start justify-between">
											<div className="space-y-2">
												<div className="flex items-center space-x-2">
													<h4 className="font-semibold">{incident.title}</h4>
													<Badge
														className={getSeverityColor(incident.severity)}
													>
														{incident.severity}
													</Badge>
													<Badge className={getStatusColor(incident.status)}>
														{incident.status}
													</Badge>
												</div>
												{incident.description && (
													<p className="text-sm text-muted-foreground">
														{incident.description}
													</p>
												)}
											</div>
											<div className="flex items-center space-x-2">
												<Button
													variant="outline"
													size="sm"
													onClick={() => handleViewTimeline(incident)}
												>
													<Eye className="h-4 w-4" />
												</Button>
												<Button
													variant="outline"
													size="sm"
													onClick={() => handleEdit(incident)}
												>
													<Edit className="h-4 w-4" />
												</Button>
											</div>
										</div>
										<div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
											{incident.assigned_to && (
												<div className="flex items-center space-x-2">
													<User className="h-4 w-4 text-muted-foreground" />
													<span>Assigned to: {incident.assigned_to}</span>
												</div>
											)}
											<div className="flex items-center space-x-2">
												<Clock className="h-4 w-4 text-muted-foreground" />
												<span>
													Created{" "}
													{formatDistanceToNow(new Date(incident.started_at), {
														addSuffix: true,
													})}
												</span>
											</div>
											{incident.resolved_at && (
												<div className="flex items-center space-x-2">
													<Clock className="h-4 w-4 text-green-600" />
													<span>
														Resolved{" "}
														{formatDistanceToNow(
															new Date(incident.resolved_at),
															{
																addSuffix: true,
															},
														)}
													</span>
												</div>
											)}
										</div>
										{incident.root_cause && (
											<div className="text-sm">
												<strong>Root Cause:</strong> {incident.root_cause}
											</div>
										)}
									</div>
								))}
							</div>
						) : (
							<div className="text-center py-8">
								<Shield className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
								<p className="text-muted-foreground">
									No incidents reported yet.
									<br />
									Create an incident to start tracking issues.
								</p>
							</div>
						)}
					</CardContent>
				</Card>

				{/* Incident Statistics */}
				<Card>
					<CardHeader>
						<CardTitle>Incident Statistics</CardTitle>
						<CardDescription>
							Overview of incident metrics and resolution times
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="grid grid-cols-1 md:grid-cols-4 gap-4">
							<div className="text-center">
								<div className="text-2xl font-bold text-red-600">
									{incidents
										? (incidents as any[]).filter(
												(i: any) => i.status === "open",
											).length
										: 0}
								</div>
								<p className="text-sm text-muted-foreground">Open</p>
							</div>
							<div className="text-center">
								<div className="text-2xl font-bold text-yellow-600">
									{incidents
										? (incidents as any[]).filter(
												(i: any) => i.status === "investigating",
											).length
										: 0}
								</div>
								<p className="text-sm text-muted-foreground">Investigating</p>
							</div>
							<div className="text-center">
								<div className="text-2xl font-bold text-green-600">
									{incidents
										? (incidents as any[]).filter(
												(i: any) => i.status === "resolved",
											).length
										: 0}
								</div>
								<p className="text-sm text-muted-foreground">Resolved</p>
							</div>
							<div className="text-center">
								<div className="text-2xl font-bold text-gray-600">
									{incidents
										? (incidents as any[]).filter(
												(i: any) => i.status === "closed",
											).length
										: 0}
								</div>
								<p className="text-sm text-muted-foreground">Closed</p>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/monitoring/incidents")({
	component: IncidentsTracking,
});
