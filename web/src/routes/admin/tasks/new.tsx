import { AdminLayout } from "@/components/layout/AdminLayout";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Textarea } from "@/components/ui/textarea";
import { useToast } from "@/hooks/use-toast";
import { apiClient } from "@/lib/api/client";
import type { components } from "@/types/api";
import { useMutation, useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Link, useNavigate } from "@tanstack/react-router";
import { ArrowLeft, Code, Plus, Settings } from "lucide-react";
import { useState } from "react";

// Task payload type definitions
interface EmailTaskPayload {
	to: string;
	subject: string;
	template: string;
	data: Record<string, unknown>;
}

interface WebhookTaskPayload {
	url: string;
	method: string;
	headers: Record<string, string>;
	payload: Record<string, unknown>;
}

interface NotificationTaskPayload {
	user_id: string;
	title: string;
	body: string;
	type: string;
}

interface DataSyncTaskPayload {
	source: string;
	target: string;
	table: string;
	batch_size: number;
}

// Union type for all task payloads
type TaskPayload = EmailTaskPayload | WebhookTaskPayload | NotificationTaskPayload | DataSyncTaskPayload;

interface CreateTaskForm {
	taskType: string;
	data: string;
	priority: string;
	delay: string;
}

function NewTaskPage() {
	const navigate = useNavigate();
	const { toast } = useToast();

	const [formData, setFormData] = useState<CreateTaskForm>({
		taskType: "",
		data: "{}",
		priority: "normal",
		delay: "0",
	});

	const [errors, setErrors] = useState<Partial<CreateTaskForm>>({});

	// Fetch available task types
	const { data: taskTypesResponse, isLoading: isLoadingTypes } = useQuery({
		queryKey: ["taskTypes"],
		queryFn: async () => {
			const response = await apiClient.getTaskTypes();
			return response;
		},
	});

	const createTaskMutation = useMutation({
		mutationFn: async (taskData: {
			task_type: string;
			payload: TaskPayload;
			scheduled_at?: string;
		}) => {
			const response = await apiClient.createTask(taskData);
			return response;
		},
		onSuccess: (response) => {
			const data = response.data;
			toast({
				title: "Task created successfully",
				description: `Task ${data?.id?.slice(-8) || "Unknown"} has been queued for execution`,
			});
			navigate({ to: "/admin/tasks" });
		},
		onError: (error: Error) => {
			toast({
				title: "Failed to create task",
				description:
					error.message || "An error occurred while creating the task",
				variant: "destructive",
			});
		},
	});

	const taskTypes = taskTypesResponse?.data || [];

	const validateForm = (): boolean => {
		const newErrors: Partial<CreateTaskForm> = {};

		if (!formData.taskType) {
			newErrors.taskType = "Task type is required";
		}

		if (!formData.data.trim()) {
			newErrors.data = "Task data is required";
		} else {
			try {
				JSON.parse(formData.data);
			} catch {
				newErrors.data = "Task data must be valid JSON";
			}
		}

		if (formData.delay && Number.isNaN(Number(formData.delay))) {
			newErrors.delay = "Delay must be a valid number";
		}

		setErrors(newErrors);
		return Object.keys(newErrors).length === 0;
	};

	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault();

		if (!validateForm()) return;

		let parsedData: TaskPayload;
		try {
			parsedData = JSON.parse(formData.data) as TaskPayload;
		} catch {
			setErrors({ data: "Invalid JSON format" });
			return;
		}

		const taskPayload = {
			task_type: formData.taskType,
			payload: parsedData,
			priority: formData.priority,
			...(formData.delay &&
				Number(formData.delay) > 0 && {
					scheduled_at: new Date(
						Date.now() + Number(formData.delay) * 1000,
					).toISOString(),
				}),
		};

		createTaskMutation.mutate(taskPayload);
	};

	const handleInputChange = (field: keyof CreateTaskForm, value: string) => {
		setFormData((prev) => ({ ...prev, [field]: value }));
		// Clear error when user starts typing
		if (errors[field]) {
			setErrors((prev) => ({ ...prev, [field]: undefined }));
		}
	};

	const getTaskTypeExample = (taskType: string) => {
		const examples: Record<string, string> = {
			email: JSON.stringify(
				{
					to: "user@example.com",
					subject: "Welcome!",
					template: "welcome",
					data: { name: "John Doe" },
				},
				null,
				2,
			),
			webhook: JSON.stringify(
				{
					url: "https://api.example.com/webhook",
					method: "POST",
					headers: { "Content-Type": "application/json" },
					payload: { event: "user_created", user_id: "123" },
				},
				null,
				2,
			),
			notification: JSON.stringify(
				{
					user_id: "user_123",
					title: "New Message",
					body: "You have received a new message",
					type: "info",
				},
				null,
				2,
			),
			data_sync: JSON.stringify(
				{
					source: "database_a",
					target: "database_b",
					table: "users",
					batch_size: 100,
				},
				null,
				2,
			),
		};

		return (
			examples[taskType] ||
			JSON.stringify(
				{
					message: "Hello, World!",
					timestamp: new Date().toISOString(),
				},
				null,
				2,
			)
		);
	};

	const handleTaskTypeChange = (taskType: string) => {
		handleInputChange("taskType", taskType);
		// Auto-populate example data for the selected task type
		const exampleData = getTaskTypeExample(taskType);
		handleInputChange("data", exampleData);
	};

	return (
		<AdminLayout>
			<div className="max-w-2xl mx-auto space-y-6">
				{/* Header */}
				<div className="flex items-center space-x-4">
					<Button variant="ghost" size="sm" asChild>
						<Link to="/admin/tasks">
							<ArrowLeft className="h-4 w-4 mr-2" />
							Back to Tasks
						</Link>
					</Button>
					<div>
						<h1 className="text-3xl font-bold tracking-tight">
							Create New Task
						</h1>
						<p className="text-muted-foreground">
							Queue a new background task for execution
						</p>
					</div>
				</div>

				{/* Form */}
				<form onSubmit={handleSubmit} className="space-y-6">
					{/* Task Configuration */}
					<Card>
						<CardHeader>
							<CardTitle className="flex items-center">
								<Code className="h-5 w-5 mr-2" />
								Task Configuration
							</CardTitle>
						</CardHeader>
						<CardContent className="space-y-4">
							<div className="grid grid-cols-2 gap-4">
								<div className="space-y-2">
									<Label htmlFor="taskType">Task Type *</Label>
									{isLoadingTypes ? (
										<div className="h-10 bg-muted animate-pulse rounded-md" />
									) : (
										<Select
											value={formData.taskType}
											onValueChange={handleTaskTypeChange}
										>
											<SelectTrigger
												className={errors.taskType ? "border-red-500" : ""}
											>
												<SelectValue placeholder="Select task type" />
											</SelectTrigger>
											<SelectContent>
												{taskTypes.map((type) => (
													<SelectItem
														key={type.task_type}
														value={type.task_type}
													>
														<div className="flex flex-col items-start">
															<span className="font-medium">
																{type.task_type}
															</span>
															{type.description && (
																<span className="text-xs text-muted-foreground">
																	{type.description}
																</span>
															)}
														</div>
													</SelectItem>
												))}
											</SelectContent>
										</Select>
									)}
									{errors.taskType && (
										<p className="text-sm text-red-600">{errors.taskType}</p>
									)}
								</div>

								<div className="space-y-2">
									<Label htmlFor="priority">Priority</Label>
									<Select
										value={formData.priority}
										onValueChange={(value) =>
											handleInputChange("priority", value)
										}
									>
										<SelectTrigger>
											<SelectValue placeholder="Select priority" />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value="low">Low</SelectItem>
											<SelectItem value="normal">Normal</SelectItem>
											<SelectItem value="high">High</SelectItem>
											<SelectItem value="critical">Critical</SelectItem>
										</SelectContent>
									</Select>
								</div>
							</div>

							<div className="space-y-2">
								<Label htmlFor="delay">Delay (seconds)</Label>
								<Input
									id="delay"
									type="number"
									min="0"
									value={formData.delay}
									onChange={(e) => handleInputChange("delay", e.target.value)}
									placeholder="0 (execute immediately)"
									className={errors.delay ? "border-red-500" : ""}
								/>
								{errors.delay && (
									<p className="text-sm text-red-600">{errors.delay}</p>
								)}
								<p className="text-xs text-muted-foreground">
									Leave as 0 to execute immediately, or specify seconds to delay
									execution
								</p>
							</div>
						</CardContent>
					</Card>

					{/* Task Data */}
					<Card>
						<CardHeader>
							<CardTitle className="flex items-center">
								<Settings className="h-5 w-5 mr-2" />
								Task Data
							</CardTitle>
						</CardHeader>
						<CardContent className="space-y-4">
							<div className="space-y-2">
								<Label htmlFor="data">JSON Data *</Label>
								<Textarea
									id="data"
									value={formData.data}
									onChange={(e) => handleInputChange("data", e.target.value)}
									placeholder="Enter task data as JSON"
									className={`min-h-32 font-mono text-sm ${errors.data ? "border-red-500" : ""}`}
								/>
								{errors.data && (
									<p className="text-sm text-red-600">{errors.data}</p>
								)}
								<p className="text-xs text-muted-foreground">
									Enter the data that will be passed to the task handler as JSON
								</p>
							</div>

							{formData.taskType && (
								<div className="mt-4">
									<Separator />
									<div className="mt-4">
										<Label className="text-sm font-medium">
											Example Data Structure
										</Label>
										<p className="text-xs text-muted-foreground mb-2">
											Typical data structure for {formData.taskType} tasks:
										</p>
										<div className="rounded-md bg-muted p-3">
											<pre className="text-xs overflow-auto">
												{getTaskTypeExample(formData.taskType)}
											</pre>
										</div>
									</div>
								</div>
							)}
						</CardContent>
					</Card>

					{/* Actions */}
					<div className="flex items-center justify-end space-x-4">
						<Button type="button" variant="outline" asChild>
							<Link to="/admin/tasks">Cancel</Link>
						</Button>
						<Button type="submit" disabled={createTaskMutation.isPending}>
							<Plus className="h-4 w-4 mr-2" />
							{createTaskMutation.isPending ? "Creating..." : "Create Task"}
						</Button>
					</div>
				</form>
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/tasks/new")({
	component: NewTaskPage,
});
