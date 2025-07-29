import { createFileRoute } from "@tanstack/react-router";
import { useMutation } from "@tanstack/react-query";
import { AdminLayout } from "@/components/layout/AdminLayout";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { useToast } from "@/hooks/use-toast";
import { apiClient } from "@/lib/api/client";
import { ArrowLeft, User, Lock, Shield } from "lucide-react";
import { Link, useNavigate } from "@tanstack/react-router";
import { useState } from "react";

interface CreateUserForm {
	username: string;
	email: string;
	password: string;
	confirmPassword: string;
	role: string;
	isActive: boolean;
}

function NewUserPage() {
	const navigate = useNavigate();
	const { toast } = useToast();
	
	const [formData, setFormData] = useState<CreateUserForm>({
		username: "",
		email: "",
		password: "",
		confirmPassword: "",
		role: "user",
		isActive: true,
	});

	const [errors, setErrors] = useState<Partial<CreateUserForm>>({});

	const createUserMutation = useMutation({
		mutationFn: async (userData: Omit<CreateUserForm, 'confirmPassword'>) => {
			const response = await apiClient.createUser(userData);
			return response.data;
		},
		onSuccess: (data) => {
			toast({
				title: "User created successfully",
				description: `User ${data.username} has been created with ID ${data.id}`,
			});
			navigate({ to: "/admin/users" });
		},
		onError: (error: any) => {
			toast({
				title: "Failed to create user",
				description: error.message || "An error occurred while creating the user",
				variant: "destructive",
			});
		},
	});

	const validateForm = (): boolean => {
		const newErrors: Partial<CreateUserForm> = {};

		if (!formData.username.trim()) {
			newErrors.username = "Username is required";
		} else if (formData.username.length < 3) {
			newErrors.username = "Username must be at least 3 characters";
		}

		if (!formData.email.trim()) {
			newErrors.email = "Email is required";
		} else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email)) {
			newErrors.email = "Please enter a valid email address";
		}

		if (!formData.password) {
			newErrors.password = "Password is required";
		} else if (formData.password.length < 6) {
			newErrors.password = "Password must be at least 6 characters";
		}

		if (formData.password !== formData.confirmPassword) {
			newErrors.confirmPassword = "Passwords do not match";
		}

		setErrors(newErrors);
		return Object.keys(newErrors).length === 0;
	};

	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault();
		
		if (!validateForm()) return;

		const { confirmPassword, ...userData } = formData;
		createUserMutation.mutate(userData);
	};

	const handleInputChange = (field: keyof CreateUserForm, value: string | boolean) => {
		setFormData(prev => ({ ...prev, [field]: value }));
		// Clear error when user starts typing
		if (errors[field]) {
			setErrors(prev => ({ ...prev, [field]: undefined }));
		}
	};

	return (
		<AdminLayout>
			<div className="max-w-2xl mx-auto space-y-6">
				{/* Header */}
				<div className="flex items-center space-x-4">
					<Button variant="ghost" size="sm" asChild>
						<Link to="/admin/users">
							<ArrowLeft className="h-4 w-4 mr-2" />
							Back to Users
						</Link>
					</Button>
					<div>
						<h1 className="text-3xl font-bold tracking-tight">Create New User</h1>
						<p className="text-muted-foreground">
							Add a new user account to the system
						</p>
					</div>
				</div>

				{/* Form */}
				<form onSubmit={handleSubmit} className="space-y-6">
					{/* Basic Information */}
					<Card>
						<CardHeader>
							<CardTitle className="flex items-center">
								<User className="h-5 w-5 mr-2" />
								Basic Information
							</CardTitle>
						</CardHeader>
						<CardContent className="space-y-4">
							<div className="grid grid-cols-2 gap-4">
								<div className="space-y-2">
									<Label htmlFor="username">Username *</Label>
									<Input
										id="username"
										value={formData.username}
										onChange={(e) => handleInputChange("username", e.target.value)}
										placeholder="Enter username"
										className={errors.username ? "border-red-500" : ""}
									/>
									{errors.username && (
										<p className="text-sm text-red-600">{errors.username}</p>
									)}
								</div>
								
								<div className="space-y-2">
									<Label htmlFor="email">Email Address *</Label>
									<Input
										id="email"
										type="email"
										value={formData.email}
										onChange={(e) => handleInputChange("email", e.target.value)}
										placeholder="Enter email address"
										className={errors.email ? "border-red-500" : ""}
									/>
									{errors.email && (
										<p className="text-sm text-red-600">{errors.email}</p>
									)}
								</div>
							</div>
						</CardContent>
					</Card>

					{/* Security */}
					<Card>
						<CardHeader>
							<CardTitle className="flex items-center">
								<Lock className="h-5 w-5 mr-2" />
								Security
							</CardTitle>
						</CardHeader>
						<CardContent className="space-y-4">
							<div className="grid grid-cols-2 gap-4">
								<div className="space-y-2">
									<Label htmlFor="password">Password *</Label>
									<Input
										id="password"
										type="password"
										value={formData.password}
										onChange={(e) => handleInputChange("password", e.target.value)}
										placeholder="Enter password"
										className={errors.password ? "border-red-500" : ""}
									/>
									{errors.password && (
										<p className="text-sm text-red-600">{errors.password}</p>
									)}
								</div>
								
								<div className="space-y-2">
									<Label htmlFor="confirmPassword">Confirm Password *</Label>
									<Input
										id="confirmPassword"
										type="password"
										value={formData.confirmPassword}
										onChange={(e) => handleInputChange("confirmPassword", e.target.value)}
										placeholder="Confirm password"
										className={errors.confirmPassword ? "border-red-500" : ""}
									/>
									{errors.confirmPassword && (
										<p className="text-sm text-red-600">{errors.confirmPassword}</p>
									)}
								</div>
							</div>
						</CardContent>
					</Card>

					{/* Permissions & Status */}
					<Card>
						<CardHeader>
							<CardTitle className="flex items-center">
								<Shield className="h-5 w-5 mr-2" />
								Permissions & Status
							</CardTitle>
						</CardHeader>
						<CardContent className="space-y-4">
							<div className="grid grid-cols-2 gap-4">
								<div className="space-y-2">
									<Label htmlFor="role">Role</Label>
									<Select
										value={formData.role}
										onValueChange={(value) => handleInputChange("role", value)}
									>
										<SelectTrigger>
											<SelectValue placeholder="Select role" />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value="user">User</SelectItem>
											<SelectItem value="moderator">Moderator</SelectItem>
											<SelectItem value="admin">Administrator</SelectItem>
										</SelectContent>
									</Select>
								</div>
								
								<div className="space-y-2">
									<Label htmlFor="isActive">Account Status</Label>
									<div className="flex items-center space-x-2 pt-2">
										<Switch
											id="isActive"
											checked={formData.isActive}
											onCheckedChange={(checked) => handleInputChange("isActive", checked)}
										/>
										<Label htmlFor="isActive" className="text-sm">
											{formData.isActive ? "Active" : "Inactive"}
										</Label>
									</div>
								</div>
							</div>
							
							<Separator />
							
							<div className="space-y-2">
								<Label className="text-sm font-medium">Role Permissions</Label>
								<div className="text-sm text-muted-foreground space-y-1">
									{formData.role === "admin" && (
										<p>• Full system access and user management</p>
									)}
									{formData.role === "moderator" && (
										<>
											<p>• Content moderation capabilities</p>
											<p>• Limited user management</p>
										</>
									)}
									{formData.role === "user" && (
										<p>• Basic user access and functionality</p>
									)}
								</div>
							</div>
						</CardContent>
					</Card>

					{/* Actions */}
					<div className="flex items-center justify-end space-x-4">
						<Button type="button" variant="outline" asChild>
							<Link to="/admin/users">Cancel</Link>
						</Button>
						<Button 
							type="submit" 
							disabled={createUserMutation.isPending}
						>
							{createUserMutation.isPending ? "Creating..." : "Create User"}
						</Button>
					</div>
				</form>
			</div>
		</AdminLayout>
	);
}

export const Route = createFileRoute("/admin/users/new")({
	component: NewUserPage,
});