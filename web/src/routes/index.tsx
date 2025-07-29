import { useAuth } from "@/lib/auth/context";
import { Navigate, createFileRoute } from "@tanstack/react-router";

function Home() {
	const { isAuthenticated, isLoading } = useAuth();

	if (isLoading) {
		return (
			<div className="flex items-center justify-center min-h-screen">
				<div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900" />
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
