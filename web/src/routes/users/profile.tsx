import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { useToast } from "@/hooks/use-toast";
import { apiClient } from "@/lib/api/client";
import { useAuth } from "@/lib/auth/context";
import { RoleGuard } from "@/components/auth/RoleGuard";
import { getRoleDisplayName, getRoleColor } from "@/lib/rbac/types";
import { Badge } from "@/components/ui/badge";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { AlertTriangle, Key, Trash2, User } from "lucide-react";
import { useState } from "react";
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
	AlertDialogTrigger,
} from "@/components/ui/alert-dialog";

function UserProfilePage() {
	const { user, refreshUser } = useAuth();
	const { toast } = useToast();
	const queryClient = useQueryClient();

	// Profile update form state
	const [profileForm, setProfileForm] = useState({
		username: user?.username || "",
		email: user?.email || "",
	});

	// Password change form state
	const [passwordForm, setPasswordForm] = useState({
		current_password: "",
		new_password: "",
		confirm_password: "",
	});

	// Account deletion form state
	const [deleteForm, setDeleteForm] = useState({
		password: "",
		confirmation: "",
	});

	// Profile update mutation
	const updateProfileMutation = useMutation({
		mutationFn: (data: { username?: string; email?: string }) =>
			apiClient.updateOwnProfile(data),
		onSuccess: () => {
			toast({
				title: "Profile updated",
				description: "Your profile has been updated successfully.",
			});
			refreshUser();
			queryClient.invalidateQueries({ queryKey: ["user"] });
		},
		onError: (error: any) => {
			toast({
				title: "Failed to update profile",
				description: error.message || "An error occurred while updating your profile.",
				variant: "destructive",
			});
		},
	});

	// Password change mutation
	const changePasswordMutation = useMutation({
		mutationFn: (data: { current_password: string; new_password: string }) =>
			apiClient.changeOwnPassword(data),
		onSuccess: () => {
			toast({
				title: "Password changed",
				description: "Your password has been changed successfully.",
			});
			setPasswordForm({ current_password: "", new_password: "", confirm_password: "" });
		},
		onError: (error: any) => {
			toast({
				title: "Failed to change password",
				description: error.message || "An error occurred while changing your password.",
				variant: "destructive",
			});
		},
	});

	// Account deletion mutation
	const deleteAccountMutation = useMutation({
		mutationFn: (data: { password: string; confirmation: string }) =>
			apiClient.deleteOwnAccount(data),
		onSuccess: () => {
			toast({
				title: "Account deleted",
				description: "Your account has been deleted successfully.",
			});
			// Redirect will happen automatically when user is logged out
		},
		onError: (error: any) => {
			toast({
				title: "Failed to delete account",
				description: error.message || "An error occurred while deleting your account.",
				variant: "destructive",
			});
		},
	});

	const handleProfileUpdate = (e: React.FormEvent) => {
		e.preventDefault();
		updateProfileMutation.mutate(profileForm);
	};

	const handlePasswordChange = (e: React.FormEvent) => {
		e.preventDefault();
		
		if (passwordForm.new_password !== passwordForm.confirm_password) {
			toast({
				title: "Password mismatch",
				description: "New password and confirmation do not match.",
				variant: "destructive",
			});
			return;
		}

		if (passwordForm.new_password.length < 8) {
			toast({
				title: "Password too short",
				description: "Password must be at least 8 characters long.",
				variant: "destructive",
			});
			return;
		}

		changePasswordMutation.mutate({
			current_password: passwordForm.current_password,
			new_password: passwordForm.new_password,
		});
	};

	const handleAccountDeletion = () => {
		if (deleteForm.confirmation !== "DELETE") {
			toast({
				title: "Invalid confirmation",
				description: 'Please type "DELETE" to confirm account deletion.',
				variant: "destructive",
			});
			return;
		}

		deleteAccountMutation.mutate(deleteForm);
	};

	if (!user) {
		return (
			<div className="flex items-center justify-center min-h-screen">
				<p>Loading user profile...</p>
			</div>
		);
	}

	return (
		<div className="container mx-auto py-6 space-y-6">
			<div>
				<h1 className="text-3xl font-bold">My Profile</h1>
				<p className="text-muted-foreground">
					Manage your account settings and preferences
				</p>
			</div>

			{/* User Info Card */}
			<Card>
				<CardHeader>
					<CardTitle className="flex items-center gap-2">
						<User className="h-5 w-5" />
						Account Information
					</CardTitle>
				</CardHeader>
				<CardContent className="space-y-4">
					<div className="grid grid-cols-2 gap-4">
						<div>
							<Label className="text-sm font-medium text-muted-foreground">
								User ID
							</Label>
							<p className="font-mono text-sm">{user.id}</p>
						</div>
						<div>
							<Label className="text-sm font-medium text-muted-foreground">
								Role
							</Label>
							<div>
								<Badge variant="outline" className={`text-${getRoleColor(user.role)} border-${getRoleColor(user.role)}`}>
									{getRoleDisplayName(user.role)}
								</Badge>
							</div>
						</div>
					</div>
				</CardContent>
			</Card>

			{/* Profile Update Form */}
			<Card>
				<CardHeader>
					<CardTitle>Update Profile</CardTitle>
				</CardHeader>
				<CardContent>
					<form onSubmit={handleProfileUpdate} className="space-y-4">
						<div>
							<Label htmlFor="username">Username</Label>
							<Input
								id="username"
								type="text"
								value={profileForm.username}
								onChange={(e) =>
									setProfileForm({ ...profileForm, username: e.target.value })
								}
								required
							/>
						</div>
						<div>
							<Label htmlFor="email">Email</Label>
							<Input
								id="email"
								type="email"
								value={profileForm.email}
								onChange={(e) =>
									setProfileForm({ ...profileForm, email: e.target.value })
								}
								required
							/>
						</div>
						<Button 
							type="submit" 
							disabled={updateProfileMutation.isPending}
						>
							{updateProfileMutation.isPending ? "Updating..." : "Update Profile"}
						</Button>
					</form>
				</CardContent>
			</Card>

			{/* Password Change Form */}
			<Card>
				<CardHeader>
					<CardTitle className="flex items-center gap-2">
						<Key className="h-5 w-5" />
						Change Password
					</CardTitle>
				</CardHeader>
				<CardContent>
					<form onSubmit={handlePasswordChange} className="space-y-4">
						<div>
							<Label htmlFor="current_password">Current Password</Label>
							<Input
								id="current_password"
								type="password"
								value={passwordForm.current_password}
								onChange={(e) =>
									setPasswordForm({ ...passwordForm, current_password: e.target.value })
								}
								required
							/>
						</div>
						<div>
							<Label htmlFor="new_password">New Password</Label>
							<Input
								id="new_password"
								type="password"
								value={passwordForm.new_password}
								onChange={(e) =>
									setPasswordForm({ ...passwordForm, new_password: e.target.value })
								}
								required
								minLength={8}
							/>
						</div>
						<div>
							<Label htmlFor="confirm_password">Confirm New Password</Label>
							<Input
								id="confirm_password"
								type="password"
								value={passwordForm.confirm_password}
								onChange={(e) =>
									setPasswordForm({ ...passwordForm, confirm_password: e.target.value })
								}
								required
								minLength={8}
							/>
						</div>
						<Button 
							type="submit" 
							disabled={changePasswordMutation.isPending}
						>
							{changePasswordMutation.isPending ? "Changing..." : "Change Password"}
						</Button>
					</form>
				</CardContent>
			</Card>

			{/* Danger Zone - Account Deletion */}
			<Card className="border-destructive">
				<CardHeader>
					<CardTitle className="flex items-center gap-2 text-destructive">
						<AlertTriangle className="h-5 w-5" />
						Danger Zone
					</CardTitle>
				</CardHeader>
				<CardContent>
					<div className="space-y-4">
						<div>
							<h4 className="font-medium">Delete Account</h4>
							<p className="text-sm text-muted-foreground">
								Once you delete your account, there is no going back. Please be certain.
							</p>
						</div>
						<Separator />
						<AlertDialog>
							<AlertDialogTrigger asChild>
								<Button variant="destructive" className="flex items-center gap-2">
									<Trash2 className="h-4 w-4" />
									Delete Account
								</Button>
							</AlertDialogTrigger>
							<AlertDialogContent>
								<AlertDialogHeader>
									<AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
									<AlertDialogDescription>
										This action cannot be undone. This will permanently delete your
										account and remove your data from our servers.
									</AlertDialogDescription>
								</AlertDialogHeader>
								<div className="space-y-4">
									<div>
										<Label htmlFor="delete_password">Enter your password</Label>
										<Input
											id="delete_password"
											type="password"
											value={deleteForm.password}
											onChange={(e) =>
												setDeleteForm({ ...deleteForm, password: e.target.value })
											}
											placeholder="Enter your current password"
										/>
									</div>
									<div>
										<Label htmlFor="delete_confirmation">
											Type "DELETE" to confirm
										</Label>
										<Input
											id="delete_confirmation"
											type="text"
											value={deleteForm.confirmation}
											onChange={(e) =>
												setDeleteForm({ ...deleteForm, confirmation: e.target.value })
											}
											placeholder="DELETE"
										/>
									</div>
								</div>
								<AlertDialogFooter>
									<AlertDialogCancel>Cancel</AlertDialogCancel>
									<AlertDialogAction
										onClick={handleAccountDeletion}
										disabled={deleteAccountMutation.isPending}
										className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
									>
										{deleteAccountMutation.isPending ? "Deleting..." : "Delete Account"}
									</AlertDialogAction>
								</AlertDialogFooter>
							</AlertDialogContent>
						</AlertDialog>
					</div>
				</CardContent>
			</Card>
		</div>
	);
}

export const Route = createFileRoute("/users/profile")({
	component: UserProfilePage,
});