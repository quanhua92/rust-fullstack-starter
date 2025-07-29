import { createFileRoute, Navigate } from "@tanstack/react-router";
import { useAuth } from "@/lib/auth/context";

function Home() {
	const { isAuthenticated, isLoading } = useAuth();

	if (isLoading) {
		return (
			<div className="flex items-center justify-center min-h-screen">
				<div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
			</div>
		);
	}

	if (isAuthenticated) {
		return <Navigate to="/admin" />;
	}

	return <Navigate to="/auth/login" />;
}

export const Route = createFileRoute("/")({
	component: Home,
});
