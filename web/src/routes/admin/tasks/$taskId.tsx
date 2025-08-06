import { AdminLayout } from "@/components/layout/AdminLayout";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useToast } from "@/hooks/use-toast";
import { apiClient } from "@/lib/api/client";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Link } from "@tanstack/react-router";
import {
	Activity,
	ArrowLeft,
	CheckCircle,
	Clock,
	Code,
	Pause,
	Play,
	RotateCcw,
	Trash2,
	XCircle,
} from "lucide-react";

import type { components } from "@/types/api";

type TaskStatus = components["schemas"]["TaskStatus"];

function TaskDetailPage() {
	const { taskId } = Route.useParams();
	const queryClient = useQueryClient();
	const { toast } = useToast();

	// Fetch task details
	const { data: taskResponse, isLoading } = useQuery({
		queryKey: ["task", taskId],
		queryFn: async () => {
			const response = await apiClient.getTask(taskId);
			return response;
		},
	});

	// Task action mutations
	const cancelTaskMutation = useMutation({
		mutationFn: () => apiClient.cancelTask(taskId),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["task", taskId] });
			toast({ title: "Task cancelled successfully" });
		},
		onError: (error: Error) => {
			toast({
				title: "Failed to cancel task",
				description: error.message,
				variant: "destructive",
			});
		},
	});

	const retryTaskMutation = useMutation({
		mutationFn: () => apiClient.retryTask(taskId),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["task", taskId] });
			toast({ title: "Task retry initiated successfully" });
		},
		onError: (error: Error) => {
			toast({
				title: "Failed to retry task",
				description: error.message,
				variant: "destructive",
			});
		},
	});

	const deleteTaskMutation = useMutation({
		mutationFn: () => apiClient.deleteTask(taskId),
		onSuccess: () => {
			toast({ title: "Task deleted successfully" });
			// Navigate back to tasks list
			window.history.back();
		},
		onError: (error: Error) => {
			toast({
				title: "Failed to delete task",
				description: error.message,
				variant: "destructive",
			});
		},
	});

	if (isLoading) {
		return (
			<AdminLayout>
				<div className="flex items-center justify-center min-h-[400px]">
					<div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900" />
				</div>
			</AdminLayout>
		);
	}

	const task = taskResponse?.data;

	if (!task) {
		return (
			<AdminLayout>
				<div className="text-center py-12">
					<h2 className="text-2xl font-semibold">Task not found</h2>
					<p className="text-muted-foreground mt-2">
						The task you're looking for doesn't exist.
					</p>
					<Button asChild className="mt-4">
						<Link to="/admin/tasks">
							<ArrowLeft className="h-4 w-4 mr-2" />
							Back to Tasks
						</Link>
					</Button>
				</div>
			</AdminLayout>
		);
	}

	const getStatusBadge = (status: TaskStatus) => {
		const statusConfig = {
			pending: { variant: "secondary" as const, icon: Clock, label: "Pending" },
			running: { variant: "default" as const, icon: Play, label: "Running" },
			completed: {
				variant: "default" as const,
				icon: CheckCircle,
				label: "Completed",
			},
			failed: {
				variant: "destructive" as const,
				icon: XCircle,
				label: "Failed",
			},
			cancelled: {
				variant: "secondary" as const,
				icon: XCircle,
				label: "Cancelled",
			},
			retrying: {
				variant: "outline" as const,
				icon: RotateCcw,
				label: "Retrying",
			},
		};

		const config = statusConfig[status] || statusConfig.pending;
		const Icon = config.icon;

		return (
			<Badge variant={config.variant} className="flex items-center gap-1">
				<Icon className="h-3 w-3" />
				{config.label}
			</Badge>
		);
	};

	const formatDuration = (startTime: string, endTime?: string) => {
		const start = new Date(startTime);
		const end = endTime ? new Date(endTime) : new Date();
		const duration = end.getTime() - start.getTime();
		const seconds = Math.floor(duration / 1000);
		const minutes = Math.floor(seconds / 60);
		const hours = Math.floor(minutes / 60);

		if (hours > 0) return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
		if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
		return `${seconds}s`;
	};

	const handleTaskAction = async (action: string) => {
		switch (action) {
			case "cancel":
				cancelTaskMutation.mutate();
				break;
			case "retry":
				retryTaskMutation.mutate();
				break;
			case "delete":
				if (confirm("Are you sure you want to delete this task?")) {
					deleteTaskMutation.mutate();
				}
				break;
		}
	};

	return (
		<AdminLayout>
			<div className="space-y-6">
				{/* Header */}
				<div className="flex items-center justify-between">
					<div className="flex items-center space-x-4">
						<Button variant="ghost" size="sm" asChild>
							<Link to="/admin/tasks">
								<ArrowLeft className="h-4 w-4 mr-2" />
								Back to Tasks
							</Link>
						</Button>
						<div>
							<h1 className="text-3xl font-bold tracking-tight">
								Task #{task.id?.slice(-8) || "Unknown"}
							</h1>
							<p className="text-muted-foreground">
								{task.task_type} • Created{" "}
								{task.created_at
									? new Date(task.created_at).toLocaleDateString()
									: "Unknown"}
							</p>
						</div>
					</div>
					<div className="flex items-center space-x-2">
						{getStatusBadge(task.status as TaskStatus)}
					</div>
				</div>

				{/* Action Buttons */}
				<div className="flex items-center space-x-2">
					{task.status === "pending" && (
						<Button
							variant="outline"
							onClick={() => handleTaskAction("cancel")}
							disabled={cancelTaskMutation.isPending}
						>
							<Pause className="h-4 w-4 mr-2" />
							Cancel Task
						</Button>
					)}

					{(task.status === "failed" || task.status === "cancelled") && (
						<Button
							variant="outline"
							onClick={() => handleTaskAction("retry")}
							disabled={retryTaskMutation.isPending}
						>
							<RotateCcw className="h-4 w-4 mr-2" />
							Retry Task
						</Button>
					)}

					<Button
						variant="destructive"
						onClick={() => handleTaskAction("delete")}
						disabled={deleteTaskMutation.isPending}
					>
						<Trash2 className="h-4 w-4 mr-2" />
						Delete Task
					</Button>
				</div>

				{/* Task Info Cards */}
				<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Task Type</CardTitle>
							<Code className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">{task.task_type}</div>
							<p className="text-xs text-muted-foreground">Handler type</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Attempts</CardTitle>
							<Activity className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">
								{task.current_attempt || 0}
							</div>
							<p className="text-xs text-muted-foreground">
								Execution attempts
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Duration</CardTitle>
							<Clock className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-xl font-bold">
								{task.created_at && task.status !== "pending"
									? formatDuration(task.created_at, task.updated_at)
									: "N/A"}
							</div>
							<p className="text-xs text-muted-foreground">
								{task.status === "running" ? "Running time" : "Total time"}
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Priority</CardTitle>
							<Activity className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">
								{task.priority || "normal"}
							</div>
							<p className="text-xs text-muted-foreground">
								Execution priority
							</p>
						</CardContent>
					</Card>
				</div>

				{/* Detailed Information Tabs */}
				<Tabs defaultValue="overview" className="space-y-4">
					<TabsList>
						<TabsTrigger value="overview">Overview</TabsTrigger>
						<TabsTrigger value="data">Task Data</TabsTrigger>
						<TabsTrigger value="logs">Execution Logs</TabsTrigger>
					</TabsList>

					<TabsContent value="overview">
						<div className="grid gap-4 md:grid-cols-2">
							<Card>
								<CardHeader>
									<CardTitle>Task Information</CardTitle>
								</CardHeader>
								<CardContent className="space-y-4">
									<div className="grid grid-cols-2 gap-4">
										<div>
											<span className="text-sm font-medium text-muted-foreground">
												Task ID
											</span>
											<p className="font-mono text-sm">{task.id}</p>
										</div>
										<div>
											<span className="text-sm font-medium text-muted-foreground">
												Task Type
											</span>
											<p>{task.task_type}</p>
										</div>
										<div>
											<span className="text-sm font-medium text-muted-foreground">
												Status
											</span>
											<p>{task.status}</p>
										</div>
										<div>
											<span className="text-sm font-medium text-muted-foreground">
												Attempts
											</span>
											<p>{task.current_attempt || 0}</p>
										</div>
									</div>
									<Separator />
									<div className="grid grid-cols-2 gap-4">
										<div>
											<span className="text-sm font-medium text-muted-foreground">
												Created At
											</span>
											<p>
												{task.created_at
													? new Date(task.created_at).toLocaleString()
													: "Unknown"}
											</p>
										</div>
										<div>
											<span className="text-sm font-medium text-muted-foreground">
												Updated At
											</span>
											<p>
												{task.updated_at
													? new Date(task.updated_at).toLocaleString()
													: "Not updated"}
											</p>
										</div>
									</div>
								</CardContent>
							</Card>

							<Card>
								<CardHeader>
									<CardTitle>Execution Details</CardTitle>
								</CardHeader>
								<CardContent className="space-y-4">
									<div>
										<span className="text-sm font-medium text-muted-foreground">
											Next Retry At
										</span>
										<p>No retry data available in API</p>
									</div>
									<div>
										<span className="text-sm font-medium text-muted-foreground">
											Error Message
										</span>
										<p
											className={
												task.last_error
													? "text-red-600"
													: "text-muted-foreground"
											}
										>
											{task.last_error || "No errors"}
										</p>
									</div>
									<div>
										<span className="text-sm font-medium text-muted-foreground">
											Result
										</span>
										<p className="text-muted-foreground">
											No result data available in API
										</p>
									</div>
								</CardContent>
							</Card>
						</div>
					</TabsContent>

					<TabsContent value="data">
						<Card>
							<CardHeader>
								<CardTitle>Task Input Data</CardTitle>
							</CardHeader>
							<CardContent>
								<div className="rounded-md bg-muted p-4">
									<pre className="text-sm overflow-auto">
										{task.metadata
											? JSON.stringify(task.metadata, null, 2)
											: "No input data"}
									</pre>
								</div>
							</CardContent>
						</Card>
					</TabsContent>

					<TabsContent value="logs">
						<Card>
							<CardHeader>
								<CardTitle>Execution Logs</CardTitle>
							</CardHeader>
							<CardContent>
								<div className="space-y-4">
									{task.last_error ? (
										<div className="rounded-md bg-red-50 border border-red-200 p-4">
											<h4 className="font-medium text-red-800 mb-2">
												Error Details
											</h4>
											<pre className="text-sm text-red-700 whitespace-pre-wrap">
												{task.last_error}
											</pre>
										</div>
									) : (
										<div className="text-center text-muted-foreground py-8">
											{task.status === "completed"
												? "Task completed successfully with no errors."
												: task.status === "running"
													? "Task is currently running. Logs will appear here if errors occur."
													: "No execution logs available."}
										</div>
									)}
								</div>
							</CardContent>
						</Card>
					</TabsContent>
				</Tabs>
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/tasks/$taskId")({
	component: TaskDetailPage,
});
