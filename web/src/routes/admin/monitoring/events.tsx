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
import { useCurrentUser, useMonitoringEvents } from "@/hooks/useApiQueries";
import { apiClient } from "@/lib/api/client";
// import { useAuth } from "@/lib/auth/context"; // Not currently needed
import { getRoleColorClasses, getRoleDisplayName } from "@/lib/rbac/types";
import type { components } from "@/types/api";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import { Activity, Filter, Plus, RefreshCw, Search, X } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

// Type definition for Event
type Event = NonNullable<
	components["schemas"]["ApiResponse_Vec_Event"]["data"]
>[number];

// Type definitions for event data structures
type EventTags = Record<string, string | number | boolean>;
type EventPayload = Record<string, string | number | boolean | object>;

function EventsDashboard() {
	// Authentication context (currently not used)
	const { data: currentUser } = useCurrentUser(30000);
	const queryClient = useQueryClient();

	// Filter state
	const [filters, setFilters] = useState({
		event_type: "",
		source: "",
		level: "",
		tags: "",
		limit: 50,
	});

	// Create event form state
	const [showCreateForm, setShowCreateForm] = useState(false);
	const [createForm, setCreateForm] = useState({
		event_type: "log" as "log" | "metric" | "trace" | "alert",
		source: "",
		message: "",
		level: "info",
		tags: "",
		payload: "",
	});

	// Fetch events with current filters
	const {
		data: events,
		isLoading,
		refetch,
	} = useMonitoringEvents(
		{
			...filters,
			event_type:
				(filters.event_type as
					| "trace"
					| "alert"
					| "log"
					| "metric"
					| undefined) || undefined,
			source: filters.source || undefined,
			level:
				(filters.level as "info" | "error" | "warn" | "debug" | undefined) ||
				undefined,
			tags: filters.tags || undefined,
		},
		10000, // 10 second refresh
	);

	// Create event mutation
	const createEventMutation = useMutation({
		mutationFn: async (data: typeof createForm) => {
			let parsedPayload: EventPayload | undefined;
			if (data.payload) {
				try {
					parsedPayload = JSON.parse(data.payload);
				} catch (e) {
					throw new Error("Invalid JSON in payload field.");
				}
			}

			const payload = {
				event_type: data.event_type,
				source: data.source,
				message: data.message || undefined,
				level: data.level || undefined,
				tags: data.tags ? parseTagString(data.tags) : undefined,
				payload: parsedPayload,
			};
			return apiClient.createEvent(payload);
		},
		onSuccess: () => {
			toast.success("Event created successfully");
			setShowCreateForm(false);
			setCreateForm({
				event_type: "log",
				source: "",
				message: "",
				level: "info",
				tags: "",
				payload: "",
			});
			queryClient.invalidateQueries({ queryKey: ["monitoring", "events"] });
		},
		onError: (error) => {
			toast.error(`Failed to create event: ${error.message}`);
		},
	});

	// Parse tag string (key:value,key2:value2) to object
	const parseTagString = (tagString: string): EventTags => {
		const tags: EventTags = {};
		if (!tagString.trim()) return tags;

		for (const pair of tagString.split(",")) {
			const [key, value] = pair.split(":");
			if (key && value) {
				tags[key.trim()] = value.trim();
			}
		}
		return tags;
	};

	// Format tags object to display string
	const formatTags = (tags: EventTags): string => {
		return Object.entries(tags)
			.map(([key, value]) => `${key}:${value}`)
			.join(", ");
	};

	const getEventTypeColor = (type: string) => {
		switch (type) {
			case "log":
				return "bg-blue-100 text-blue-800";
			case "metric":
				return "bg-green-100 text-green-800";
			case "trace":
				return "bg-purple-100 text-purple-800";
			case "alert":
				return "bg-red-100 text-red-800";
			default:
				return "bg-gray-100 text-gray-800";
		}
	};

	const getLevelColor = (level: string) => {
		switch (level?.toLowerCase()) {
			case "error":
				return "text-red-600";
			case "warn":
				return "text-yellow-600";
			case "info":
				return "text-blue-600";
			case "debug":
				return "text-gray-600";
			default:
				return "text-gray-600";
		}
	};

	const clearFilters = () => {
		setFilters({
			event_type: "",
			source: "",
			level: "",
			tags: "",
			limit: 50,
		});
	};

	return (
		<AdminLayout>
			<div className="space-y-6">
				{/* Header */}
				<div className="flex items-center justify-between">
					<div>
						<h1 className="text-3xl font-bold tracking-tight">
							Events Dashboard
						</h1>
						<p className="text-muted-foreground">
							Real-time monitoring events with advanced tag filtering
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
						<span>Create Event</span>
					</Button>
					<Button variant="outline" onClick={() => refetch()}>
						<RefreshCw className="h-4 w-4 mr-2" />
						Refresh
					</Button>
				</div>

				{/* Create Event Form */}
				{showCreateForm && (
					<Card>
						<CardHeader>
							<CardTitle>Create New Event</CardTitle>
							<CardDescription>
								Submit a new monitoring event to the system
							</CardDescription>
						</CardHeader>
						<CardContent className="space-y-4">
							<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
								<div className="space-y-2">
									<Label htmlFor="event_type">Event Type</Label>
									<Select
										value={createForm.event_type}
										onValueChange={(
											value: "log" | "metric" | "trace" | "alert",
										) => setCreateForm({ ...createForm, event_type: value })}
									>
										<SelectTrigger>
											<SelectValue />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value="log">Log</SelectItem>
											<SelectItem value="metric">Metric</SelectItem>
											<SelectItem value="trace">Trace</SelectItem>
											<SelectItem value="alert">Alert</SelectItem>
										</SelectContent>
									</Select>
								</div>
								<div className="space-y-2">
									<Label htmlFor="source">Source</Label>
									<Input
										id="source"
										placeholder="e.g., payment-service, auth-service"
										value={createForm.source}
										onChange={(e) =>
											setCreateForm({ ...createForm, source: e.target.value })
										}
									/>
								</div>
								<div className="space-y-2">
									<Label htmlFor="level">Level</Label>
									<Select
										value={createForm.level}
										onValueChange={(value) =>
											setCreateForm({ ...createForm, level: value })
										}
									>
										<SelectTrigger>
											<SelectValue />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value="debug">Debug</SelectItem>
											<SelectItem value="info">Info</SelectItem>
											<SelectItem value="warn">Warn</SelectItem>
											<SelectItem value="error">Error</SelectItem>
										</SelectContent>
									</Select>
								</div>
								<div className="space-y-2">
									<Label htmlFor="tags">Tags</Label>
									<Input
										id="tags"
										placeholder="user_id:123,environment:production"
										value={createForm.tags}
										onChange={(e) =>
											setCreateForm({ ...createForm, tags: e.target.value })
										}
									/>
								</div>
							</div>
							<div className="space-y-2">
								<Label htmlFor="message">Message</Label>
								<Input
									id="message"
									placeholder="Event message or description"
									value={createForm.message}
									onChange={(e) =>
										setCreateForm({ ...createForm, message: e.target.value })
									}
								/>
							</div>
							<div className="space-y-2">
								<Label htmlFor="payload">Payload (JSON)</Label>
								<Textarea
									id="payload"
									placeholder='{"key": "value", "data": 123}'
									value={createForm.payload}
									onChange={(e) =>
										setCreateForm({ ...createForm, payload: e.target.value })
									}
								/>
							</div>
							<div className="flex space-x-2">
								<Button
									onClick={() => createEventMutation.mutate(createForm)}
									disabled={!createForm.source || createEventMutation.isPending}
								>
									{createEventMutation.isPending
										? "Creating..."
										: "Create Event"}
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

				{/* Filters */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<Filter className="h-5 w-5" />
							<span>Event Filters</span>
						</CardTitle>
						<CardDescription>
							Filter events by type, source, level, or tags
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
							<div className="space-y-2">
								<Label>Event Type</Label>
								<Select
									value={filters.event_type}
									onValueChange={(value) =>
										setFilters({ ...filters, event_type: value })
									}
								>
									<SelectTrigger>
										<SelectValue placeholder="All types" />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value="">All types</SelectItem>
										<SelectItem value="log">Log</SelectItem>
										<SelectItem value="metric">Metric</SelectItem>
										<SelectItem value="trace">Trace</SelectItem>
										<SelectItem value="alert">Alert</SelectItem>
									</SelectContent>
								</Select>
							</div>
							<div className="space-y-2">
								<Label>Source</Label>
								<Input
									placeholder="Filter by source"
									value={filters.source}
									onChange={(e) =>
										setFilters({ ...filters, source: e.target.value })
									}
								/>
							</div>
							<div className="space-y-2">
								<Label>Level</Label>
								<Select
									value={filters.level}
									onValueChange={(value) =>
										setFilters({ ...filters, level: value })
									}
								>
									<SelectTrigger>
										<SelectValue placeholder="All levels" />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value="">All levels</SelectItem>
										<SelectItem value="debug">Debug</SelectItem>
										<SelectItem value="info">Info</SelectItem>
										<SelectItem value="warn">Warn</SelectItem>
										<SelectItem value="error">Error</SelectItem>
									</SelectContent>
								</Select>
							</div>
							<div className="space-y-2">
								<Label>Tags</Label>
								<Input
									placeholder="user_id:123,env:prod"
									value={filters.tags}
									onChange={(e) =>
										setFilters({ ...filters, tags: e.target.value })
									}
								/>
							</div>
						</div>
						<div className="flex items-center space-x-2 mt-4">
							<Button variant="outline" onClick={clearFilters}>
								<X className="h-4 w-4 mr-2" />
								Clear Filters
							</Button>
							<span className="text-sm text-muted-foreground">
								{Object.values(filters).some((v) => v)
									? "Filters active"
									: "No filters applied"}
							</span>
						</div>
					</CardContent>
				</Card>

				{/* Events List */}
				<Card>
					<CardHeader>
						<CardTitle className="flex items-center space-x-2">
							<Activity className="h-5 w-5" />
							<span>Recent Events</span>
						</CardTitle>
						<CardDescription>
							Latest monitoring events matching your filters
						</CardDescription>
					</CardHeader>
					<CardContent>
						{isLoading ? (
							<div className="space-y-4">
								{Array.from({ length: 5 }, () => (
									<Skeleton
										key={`events-skeleton-${Math.random()}`}
										className="h-20"
									/>
								))}
							</div>
						) : events && Array.isArray(events) && events.length > 0 ? (
							<div className="space-y-4">
								{events.map((event: Event, index) => (
									<div
										key={event.id || index}
										className="border rounded-lg p-4 space-y-2"
									>
										<div className="flex items-start justify-between">
											<div className="flex items-center space-x-2">
												<Badge className={getEventTypeColor(event.event_type)}>
													{event.event_type}
												</Badge>
												<span className="font-medium">{event.source}</span>
												{event.level && (
													<span
														className={`text-sm ${getLevelColor(event.level)}`}
													>
														{event.level.toUpperCase()}
													</span>
												)}
											</div>
											<span className="text-sm text-muted-foreground">
												{event.recorded_at
													? formatDistanceToNow(new Date(event.recorded_at), {
															addSuffix: true,
														})
													: "Unknown time"}
											</span>
										</div>
										{event.message && (
											<p className="text-sm">{event.message}</p>
										)}
										{event.tags &&
										typeof event.tags === "object" &&
										event.tags !== null &&
										Object.keys(event.tags).length > 0 ? (
											<div className="flex items-center space-x-2">
												<span className="text-xs text-muted-foreground">
													Tags:
												</span>
												<span className="text-xs bg-gray-100 px-2 py-1 rounded">
													{formatTags(event.tags as EventTags)}
												</span>
											</div>
										) : null}
										{event.payload &&
										typeof event.payload === "object" &&
										event.payload !== null &&
										Object.keys(event.payload).length > 0 ? (
											<details className="text-xs">
												<summary className="cursor-pointer text-muted-foreground">
													Show payload
												</summary>
												<pre className="mt-2 bg-gray-50 p-2 rounded overflow-x-auto">
													{JSON.stringify(event.payload, null, 2)}
												</pre>
											</details>
										) : null}
									</div>
								))}
							</div>
						) : (
							<div className="text-center py-8">
								<Search className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
								<p className="text-muted-foreground">
									No events found matching your filters.
									<br />
									Try adjusting your filters or create a new event.
								</p>
							</div>
						)}
					</CardContent>
				</Card>
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/monitoring/events")({
	component: EventsDashboard,
});
