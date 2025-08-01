import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { useAuth } from "@/lib/auth/context";
import { zodResolver } from "@hookform/resolvers/zod";
import { useNavigate, useSearch } from "@tanstack/react-router";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";

const loginSchema = z.object({
	username: z.string().optional(),
	email: z.string().email().optional(),
	password: z.string().min(1, "Password is required"),
}).refine(
	(data) => data.username || data.email,
	{
		message: "Either username or email is required",
		path: ["username"],
	}
);

type LoginFormData = z.infer<typeof loginSchema>;

export const LoginForm = () => {
	const [error, setError] = useState<string>("");
	const [isLoading, setIsLoading] = useState(false);
	const { login } = useAuth();
	const navigate = useNavigate();
	const search = useSearch({ from: "/auth/login" });

	const form = useForm<LoginFormData>({
		resolver: zodResolver(loginSchema),
		defaultValues: {
			username: "",
			email: "",
			password: "",
		},
	});

	const onSubmit = async (data: LoginFormData) => {
		setIsLoading(true);
		setError("");

		try {
			await login(data);

			// Redirect to the intended page or dashboard
			const redirectTo =
				(search as Record<string, string>)?.redirect || "/admin";
			navigate({ to: redirectTo });
		} catch (err) {
			setError(err instanceof Error ? err.message : "Login failed");
		} finally {
			setIsLoading(false);
		}
	};

	return (
		<div className="flex items-center justify-center min-h-screen bg-gray-50">
			<Card className="w-full max-w-md">
				<CardHeader className="space-y-1">
					<CardTitle className="text-2xl font-bold text-center">
						Sign In
					</CardTitle>
					<CardDescription className="text-center">
						Enter your credentials to access the admin portal
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

							{error && (
								<Alert variant="destructive">
									<AlertDescription>{error}</AlertDescription>
								</Alert>
							)}

							<Button type="submit" className="w-full" disabled={isLoading}>
								{isLoading ? "Signing In..." : "Sign In"}
							</Button>
						</form>
					</Form>

					<div className="mt-4 text-center text-sm">
						<span className="text-gray-600">Don't have an account? </span>
						<Button
							variant="link"
							className="p-0 h-auto font-semibold"
							onClick={() => navigate({ to: "/auth/register" })}
						>
							Sign Up
						</Button>
					</div>
				</CardContent>
			</Card>
		</div>
	);
};
