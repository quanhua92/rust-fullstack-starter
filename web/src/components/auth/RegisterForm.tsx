import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { useAuth } from "@/lib/auth/context";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { useNavigate } from "@tanstack/react-router";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "@/components/ui/form";

const registerSchema = z
	.object({
		username: z
			.string()
			.min(3, "Username must be at least 3 characters")
			.max(50, "Username must be at most 50 characters")
			.regex(
				/^[a-zA-Z0-9_-]+$/,
				"Username can only contain letters, numbers, hyphens, and underscores",
			),
		email: z
			.string()
			.email("Please enter a valid email address")
			.max(254, "Email must be at most 254 characters"),
		password: z
			.string()
			.min(8, "Password must be at least 8 characters")
			.max(128, "Password must be at most 128 characters"),
		confirmPassword: z.string(),
	})
	.refine((data) => data.password === data.confirmPassword, {
		message: "Passwords don't match",
		path: ["confirmPassword"],
	});

type RegisterFormData = z.infer<typeof registerSchema>;

export const RegisterForm = () => {
	const [error, setError] = useState<string>("");
	const [isLoading, setIsLoading] = useState(false);
	const [success, setSuccess] = useState(false);
	const { register } = useAuth();
	const navigate = useNavigate();

	const form = useForm<RegisterFormData>({
		resolver: zodResolver(registerSchema),
		defaultValues: {
			username: "",
			email: "",
			password: "",
			confirmPassword: "",
		},
	});

	const onSubmit = async (data: RegisterFormData) => {
		setIsLoading(true);
		setError("");
		setSuccess(false);

		try {
			await register({
				username: data.username,
				email: data.email,
				password: data.password,
			});

			setSuccess(true);

			// Redirect to login after successful registration
			setTimeout(() => {
				navigate({ to: "/auth/login" });
			}, 2000);
		} catch (err) {
			setError(err instanceof Error ? err.message : "Registration failed");
		} finally {
			setIsLoading(false);
		}
	};

	if (success) {
		return (
			<div className="flex items-center justify-center min-h-screen bg-gray-50">
				<Card className="w-full max-w-md">
					<CardContent className="pt-6">
						<Alert>
							<AlertDescription>
								Registration successful! Redirecting to login page...
							</AlertDescription>
						</Alert>
					</CardContent>
				</Card>
			</div>
		);
	}

	return (
		<div className="flex items-center justify-center min-h-screen bg-gray-50">
			<Card className="w-full max-w-md">
				<CardHeader className="space-y-1">
					<CardTitle className="text-2xl font-bold text-center">
						Create Account
					</CardTitle>
					<CardDescription className="text-center">
						Sign up to access the admin portal
					</CardDescription>
				</CardHeader>
				<CardContent>
					<Form {...form}>
						<form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
							<FormField
								control={form.control}
								name="username"
								render={({ field }) => (
									<FormItem>
										<FormLabel>Username</FormLabel>
										<FormControl>
											<Input
												{...field}
												placeholder="Enter your username"
												disabled={isLoading}
											/>
										</FormControl>
										<FormMessage />
									</FormItem>
								)}
							/>

							<FormField
								control={form.control}
								name="email"
								render={({ field }) => (
									<FormItem>
										<FormLabel>Email</FormLabel>
										<FormControl>
											<Input
												{...field}
												type="email"
												placeholder="Enter your email"
												disabled={isLoading}
											/>
										</FormControl>
										<FormMessage />
									</FormItem>
								)}
							/>

							<FormField
								control={form.control}
								name="password"
								render={({ field }) => (
									<FormItem>
										<FormLabel>Password</FormLabel>
										<FormControl>
											<Input
												{...field}
												type="password"
												placeholder="Enter your password"
												disabled={isLoading}
											/>
										</FormControl>
										<FormMessage />
									</FormItem>
								)}
							/>

							<FormField
								control={form.control}
								name="confirmPassword"
								render={({ field }) => (
									<FormItem>
										<FormLabel>Confirm Password</FormLabel>
										<FormControl>
											<Input
												{...field}
												type="password"
												placeholder="Confirm your password"
												disabled={isLoading}
											/>
										</FormControl>
										<FormMessage />
									</FormItem>
								)}
							/>

							{error && (
								<Alert variant="destructive">
									<AlertDescription>{error}</AlertDescription>
								</Alert>
							)}

							<Button type="submit" className="w-full" disabled={isLoading}>
								{isLoading ? "Creating Account..." : "Create Account"}
							</Button>
						</form>
					</Form>

					<div className="mt-4 text-center text-sm">
						<span className="text-gray-600">Already have an account? </span>
						<Button
							variant="link"
							className="p-0 h-auto font-semibold"
							onClick={() => navigate({ to: "/auth/login" })}
						>
							Sign In
						</Button>
					</div>
				</CardContent>
			</Card>
		</div>
	);
};
