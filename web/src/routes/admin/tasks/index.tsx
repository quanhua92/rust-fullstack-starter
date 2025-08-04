import { AdminLayout } from "@/components/layout/AdminLayout";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { useToast } from "@/hooks/use-toast";
import { QUERY_KEYS, useTaskStats } from "@/hooks/useApiQueries";
import { apiClient } from "@/lib/api/client";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Link } from "@tanstack/react-router";
import {
	AlertTriangle,
	CheckCircle,
	Clock,
	Eye,
	MoreHorizontal,
	Pause,
	Play,
	Plus,
	RotateCcw,
	Search,
	Trash2,
	XCircle,
} from "lucide-react";
import { useState } from "react";

import type { components } from "@/types/api";

type TaskStatus = components["schemas"]["TaskStatus"];

function TasksPage() {
	const [searchTerm, setSearchTerm] = useState("");
	const [statusFilter, setStatusFilter] = useState<string>("all");
	const [typeFilter, setTypeFilter] = useState<string>("all");

	const queryClient = useQueryClient();
	const { toast } = useToast();

	// Fetch tasks with filters
	const { data: tasksResponse, isLoading } = useQuery({
		queryKey: ["tasks", searchTerm, statusFilter, typeFilter],
		queryFn: async () => {
			const params = new URLSearchParams();
			if (statusFilter !== "all") params.set("status", statusFilter);
			if (typeFilter !== "all") params.set("type", typeFilter);
			if (searchTerm) params.set("search", searchTerm);

			const query = params.toString();
			const response = await apiClient.getTasks({
				limit: 50,
				offset: 0,
				...(query && { extra: query }),
			});
			return response;
		},
	});

	// Fetch task types for filter dropdown
	const { data: taskTypesResponse } = useQuery({
		queryKey: ["taskTypes"],
		queryFn: async () => {
			const response = await apiClient.getTaskTypes();
			return response;
		},
	});

	// Fetch task statistics - now using consistent hook
	const { data: taskStatsData } = useTaskStats();

	// Task action mutations
	const cancelTaskMutation = useMutation({
		mutationFn: (taskId: string) => apiClient.cancelTask(taskId),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["tasks"] });
			queryClient.invalidateQueries({ queryKey: QUERY_KEYS.tasks.stats });
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
		mutationFn: (taskId: string) => apiClient.retryTask(taskId),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["tasks"] });
			queryClient.invalidateQueries({ queryKey: QUERY_KEYS.tasks.stats });
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
		mutationFn: (taskId: string) => apiClient.deleteTask(taskId),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["tasks"] });
			queryClient.invalidateQueries({ queryKey: QUERY_KEYS.tasks.stats });
			toast({ title: "Task deleted successfully" });
		},
		onError: (error: Error) => {
			toast({
				title: "Failed to delete task",
				description: error.message,
				variant: "destructive",
			});
		},
	});

	const tasks = tasksResponse?.data || [];
	const taskTypes = taskTypesResponse?.data || [];
	const taskStats = taskStatsData; // Already extracted by hook

	const getStatusBadge = (status: TaskStatus) => {
		const statusConfig = {
			Pending: { variant: "secondary" as const, icon: Clock, label: "Pending" },
			Running: { variant: "default" as const, icon: Play, label: "Running" },
			Completed: {
				variant: "default" as const,
				icon: CheckCircle,
				label: "Completed",
			},
			Failed: {
				variant: "destructive" as const,
				icon: XCircle,
				label: "Failed",
			},
			Cancelled: {
				variant: "secondary" as const,
				icon: XCircle,
				label: "Cancelled",
			},
			Retrying: {
				variant: "outline" as const,
				icon: RotateCcw,
				label: "Retrying",
			},
		};

		const config = statusConfig[status] || statusConfig.Pending;
		const Icon = config.icon;

		return (
			<Badge variant={config.variant} className="flex items-center gap-1">
				<Icon className="h-3 w-3" />
				{config.label}
			</Badge>
		);
	};

	const handleTaskAction = async (taskId: string, action: string) => {
		switch (action) {
			case "cancel":
				cancelTaskMutation.mutate(taskId);
				break;
			case "retry":
				retryTaskMutation.mutate(taskId);
				break;
			case "delete":
				if (confirm("Are you sure you want to delete this task?")) {
					deleteTaskMutation.mutate(taskId);
				}
				break;
		}
	};

	return (
		<AdminLayout>
			<div className="space-y-6">
				{/* Header */}
				<div className="flex items-center justify-between">
					<div>
						<h1 className="text-3xl font-bold tracking-tight">Tasks</h1>
						<p className="text-muted-foreground">
							Manage background tasks and monitor execution status
						</p>
					</div>
					<div className="flex items-center space-x-2">
						<Button variant="outline" asChild>
							<Link to="/admin/tasks/dead-letter">
								<AlertTriangle className="h-4 w-4 mr-2" />
								Dead Letter Queue
							</Link>
						</Button>
						<Button asChild>
							<Link to="/admin/tasks/new">
								<Plus className="h-4 w-4 mr-2" />
								Create Task
							</Link>
						</Button>
					</div>
				</div>

				{/* Stats Cards */}
				<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-5">
					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Total Tasks</CardTitle>
							<CheckCircle className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">{taskStats?.total || 0}</div>
							<p className="text-xs text-muted-foreground">All time</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Pending</CardTitle>
							<Clock className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">
								{taskStats?.pending || 0}
							</div>
							<p className="text-xs text-muted-foreground">Waiting to run</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Running</CardTitle>
							<Play className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">
								{taskStats?.running || 0}
							</div>
							<p className="text-xs text-muted-foreground">
								Currently executing
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Completed</CardTitle>
							<CheckCircle className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">
								{taskStats?.completed || 0}
							</div>
							<p className="text-xs text-muted-foreground">
								Successfully finished
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Failed</CardTitle>
							<XCircle className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">{taskStats?.failed || 0}</div>
							<p className="text-xs text-muted-foreground">Need attention</p>
						</CardContent>
					</Card>
				</div>

				{/* Filters */}
				<div className="flex items-center space-x-4">
					<div className="relative flex-1 max-w-sm">
						<Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
						<Input
							placeholder="Search tasks..."
							value={searchTerm}
							onChange={(e) => setSearchTerm(e.target.value)}
							className="pl-8"
						/>
					</div>

					<Select value={statusFilter} onValueChange={setStatusFilter}>
						<SelectTrigger className="w-32">
							<SelectValue placeholder="Status" />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="all">All Status</SelectItem>
							<SelectItem value="Pending">Pending</SelectItem>
							<SelectItem value="Running">Running</SelectItem>
							<SelectItem value="Completed">Completed</SelectItem>
							<SelectItem value="Failed">Failed</SelectItem>
							<SelectItem value="Cancelled">Cancelled</SelectItem>
							<SelectItem value="Retrying">Retrying</SelectItem>
						</SelectContent>
					</Select>

					<Select value={typeFilter} onValueChange={setTypeFilter}>
						<SelectTrigger className="w-40">
							<SelectValue placeholder="Task Type" />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="all">All Types</SelectItem>
							{taskTypes.map((type) => (
								<SelectItem key={type.task_type} value={type.task_type}>
									{type.task_type}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</div>

				{/* Tasks Table */}
				<Card>
					<CardContent className="p-0">
						<Table>
							<TableHeader>
								<TableRow>
									<TableHead>Task</TableHead>
									<TableHead>Type</TableHead>
									<TableHead>Status</TableHead>
									<TableHead>Created</TableHead>
									<TableHead>Updated</TableHead>
									<TableHead>Attempts</TableHead>
									<TableHead className="w-[70px]">Actions</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{isLoading ? (
									<TableRow>
										<TableCell colSpan={7} className="text-center py-4">
											Loading tasks...
										</TableCell>
									</TableRow>
								) : tasks.length === 0 ? (
									<TableRow>
										<TableCell colSpan={7} className="text-center py-4">
											{searchTerm ||
											statusFilter !== "all" ||
											typeFilter !== "all"
												? "No tasks match your filters."
												: "No tasks found."}
										</TableCell>
									</TableRow>
								) : (
									tasks.map((task) => (
										<TableRow key={task.id}>
											<TableCell>
												<div className="flex flex-col">
													<div className="font-medium">
														Task #{task.id?.slice(-8) || "Unknown"}
													</div>
													<div className="text-sm text-muted-foreground">
														{task.metadata && typeof task.metadata === "object"
															? Object.keys(task.metadata)
																	.slice(0, 2)
																	.join(", ")
															: "No metadata"}
													</div>
												</div>
											</TableCell>
											<TableCell>
												<Badge variant="outline">{task.task_type}</Badge>
											</TableCell>
											<TableCell>
												{getStatusBadge(task.status as TaskStatus)}
											</TableCell>
											<TableCell>
												<div className="text-sm">
													{task.created_at
														? new Date(task.created_at).toLocaleString()
														: "Unknown"}
												</div>
											</TableCell>
											<TableCell>
												<div className="text-sm">
													{task.updated_at
														? new Date(task.updated_at).toLocaleString()
														: "Not updated"}
												</div>
											</TableCell>
											<TableCell>
												<span className="font-mono text-sm">
													{task.current_attempt || 0}
												</span>
											</TableCell>
											<TableCell>
												<DropdownMenu>
													<DropdownMenuTrigger asChild>
														<Button variant="ghost" className="h-8 w-8 p-0">
															<MoreHorizontal className="h-4 w-4" />
														</Button>
													</DropdownMenuTrigger>
													<DropdownMenuContent align="end">
														<DropdownMenuItem asChild>
															<Link
																to="/admin/tasks/$taskId"
																params={{ taskId: task.id }}
															>
																<Eye className="mr-2 h-4 w-4" />
																View Details
															</Link>
														</DropdownMenuItem>

														{task.status === "Pending" && (
															<DropdownMenuItem
																onClick={() =>
																	handleTaskAction(task.id, "cancel")
																}
															>
																<Pause className="mr-2 h-4 w-4" />
																Cancel Task
															</DropdownMenuItem>
														)}

														{(task.status === "Failed" ||
															task.status === "Cancelled") && (
															<DropdownMenuItem
																onClick={() =>
																	handleTaskAction(task.id, "retry")
																}
															>
																<RotateCcw className="mr-2 h-4 w-4" />
																Retry Task
															</DropdownMenuItem>
														)}

														<DropdownMenuSeparator />
														<DropdownMenuItem
															onClick={() =>
																handleTaskAction(task.id, "delete")
															}
															className="text-red-600"
														>
															<Trash2 className="mr-2 h-4 w-4" />
															Delete Task
														</DropdownMenuItem>
													</DropdownMenuContent>
												</DropdownMenu>
											</TableCell>
										</TableRow>
									))
								)}
							</TableBody>
						</Table>
					</CardContent>
				</Card>
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/tasks/")({
	component: TasksPage,
});
