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
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { useToast } from "@/hooks/use-toast";
import { apiClient } from "@/lib/api/client";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Link } from "@tanstack/react-router";
import {
	AlertTriangle,
	ArrowLeft,
	Eye,
	MoreHorizontal,
	RotateCcw,
	Search,
	Trash2,
	XCircle,
} from "lucide-react";
import { useState } from "react";

function DeadLetterQueuePage() {
	const [searchTerm, setSearchTerm] = useState("");

	const queryClient = useQueryClient();
	const { toast } = useToast();

	// Fetch dead letter queue tasks
	const { data: deadLetterResponse, isLoading } = useQuery({
		queryKey: ["deadLetterQueue", searchTerm],
		queryFn: async () => {
			const response = await apiClient.getDeadLetterQueue({
				limit: 50,
				offset: 0,
			});
			return response;
		},
	});

	// Task action mutations
	const retryTaskMutation = useMutation({
		mutationFn: (taskId: string) => apiClient.retryTask(taskId),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["deadLetterQueue"] });
			queryClient.invalidateQueries({ queryKey: ["tasks"] });
			queryClient.invalidateQueries({ queryKey: ["tasks", "stats"] });
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
			queryClient.invalidateQueries({ queryKey: ["deadLetterQueue"] });
			queryClient.invalidateQueries({ queryKey: ["tasks", "stats"] });
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

	const tasks = deadLetterResponse?.data || [];
	const filteredTasks = tasks.filter(
		(task) =>
			task.task_type?.toLowerCase().includes(searchTerm.toLowerCase()) ||
			task.id?.toLowerCase().includes(searchTerm.toLowerCase()),
	);

	const handleTaskAction = async (taskId: string, action: string) => {
		switch (action) {
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

	const bulkRetryAll = () => {
		if (
			confirm(
				`Are you sure you want to retry all ${filteredTasks.length} failed tasks?`,
			)
		) {
			for (const task of filteredTasks) {
				retryTaskMutation.mutate(task.id);
			}
		}
	};

	const bulkDeleteAll = () => {
		if (
			confirm(
				`Are you sure you want to delete all ${filteredTasks.length} failed tasks? This action cannot be undone.`,
			)
		) {
			for (const task of filteredTasks) {
				deleteTaskMutation.mutate(task.id);
			}
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
								Dead Letter Queue
							</h1>
							<p className="text-muted-foreground">
								Tasks that have permanently failed and require manual
								intervention
							</p>
						</div>
					</div>
					{filteredTasks.length > 0 && (
						<div className="flex items-center space-x-2">
							<Button
								variant="outline"
								onClick={bulkRetryAll}
								disabled={retryTaskMutation.isPending}
							>
								<RotateCcw className="h-4 w-4 mr-2" />
								Retry All
							</Button>
							<Button
								variant="destructive"
								onClick={bulkDeleteAll}
								disabled={deleteTaskMutation.isPending}
							>
								<Trash2 className="h-4 w-4 mr-2" />
								Delete All
							</Button>
						</div>
					)}
				</div>

				{/* Stats Card */}
				<div className="grid gap-4 md:grid-cols-3">
					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">
								Failed Tasks
							</CardTitle>
							<XCircle className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">{tasks.length}</div>
							<p className="text-xs text-muted-foreground">
								Tasks in dead letter queue
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">
								Avg Attempts
							</CardTitle>
							<AlertTriangle className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-2xl font-bold">
								{tasks.length > 0
									? Math.round(
											tasks.reduce(
												(sum, task) => sum + (task.current_attempt || 0),
												0,
											) / tasks.length,
										)
									: 0}
							</div>
							<p className="text-xs text-muted-foreground">
								Average retry attempts
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium">Oldest Task</CardTitle>
							<AlertTriangle className="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div className="text-lg font-bold">
								{tasks.length > 0 && tasks[0]?.created_at
									? `${Math.floor((Date.now() - new Date(tasks[0].created_at).getTime()) / (1000 * 60 * 60 * 24))} days`
									: "N/A"}
							</div>
							<p className="text-xs text-muted-foreground">
								Age of oldest failed task
							</p>
						</CardContent>
					</Card>
				</div>

				{/* Search */}
				<div className="flex items-center space-x-2">
					<div className="relative flex-1 max-w-sm">
						<Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
						<Input
							placeholder="Search failed tasks..."
							value={searchTerm}
							onChange={(e) => setSearchTerm(e.target.value)}
							className="pl-8"
						/>
					</div>
				</div>

				{/* Failed Tasks Table */}
				<Card>
					<CardContent className="p-0">
						<Table>
							<TableHeader>
								<TableRow>
									<TableHead>Task</TableHead>
									<TableHead>Type</TableHead>
									<TableHead>Failed At</TableHead>
									<TableHead>Attempts</TableHead>
									<TableHead>Error</TableHead>
									<TableHead className="w-[70px]">Actions</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{isLoading ? (
									<TableRow>
										<TableCell colSpan={6} className="text-center py-4">
											Loading failed tasks...
										</TableCell>
									</TableRow>
								) : filteredTasks.length === 0 ? (
									<TableRow>
										<TableCell colSpan={6} className="text-center py-8">
											{searchTerm ? (
												<>
													<AlertTriangle className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
													<p className="text-lg font-medium">
														No matching failed tasks
													</p>
													<p className="text-muted-foreground">
														Try adjusting your search criteria
													</p>
												</>
											) : (
												<>
													<AlertTriangle className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
													<p className="text-lg font-medium">No failed tasks</p>
													<p className="text-muted-foreground">
														Great! All tasks are executing successfully
													</p>
												</>
											)}
										</TableCell>
									</TableRow>
								) : (
									filteredTasks.map((task) => (
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
												<div className="text-sm">
													{task.updated_at
														? new Date(task.updated_at).toLocaleString()
														: "Unknown"}
												</div>
											</TableCell>
											<TableCell>
												<Badge variant="destructive" className="font-mono">
													{task.current_attempt || 0}
												</Badge>
											</TableCell>
											<TableCell>
												<div className="max-w-xs">
													<p
														className="text-sm text-red-600 truncate"
														title={task.last_error || undefined}
													>
														{task.last_error || "Unknown error"}
													</p>
												</div>
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

														<DropdownMenuSeparator />

														<DropdownMenuItem
															onClick={() => handleTaskAction(task.id, "retry")}
														>
															<RotateCcw className="mr-2 h-4 w-4" />
															Retry Task
														</DropdownMenuItem>

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

				{filteredTasks.length > 0 && (
					<div className="text-center text-sm text-muted-foreground">
						<p>
							Tasks in the Dead Letter Queue have exceeded their maximum retry
							attempts. You can manually retry them or delete them permanently.
						</p>
					</div>
				)}
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/tasks/dead-letter")({
	component: DeadLetterQueuePage,
});
