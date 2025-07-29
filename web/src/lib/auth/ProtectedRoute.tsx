import { useAuth } from "@/lib/auth/context";
import { Navigate, useLocation } from "@tanstack/react-router";
import type { ReactNode } from "react";

interface ProtectedRouteProps {
	children: ReactNode;
	redirectTo?: string;
}

export const ProtectedRoute = ({
	children,
	redirectTo = "/auth/login",
}: ProtectedRouteProps) => {
	const { isAuthenticated, isLoading } = useAuth();
	const location = useLocation();

	// Show loading while checking authentication
	if (isLoading) {
		return (
			<div className="flex items-center justify-center min-h-screen">
				<div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
			</div>
		);
	}

	// Redirect to login if not authenticated
	if (!isAuthenticated) {
		return (
			<Navigate
				to={redirectTo}
				search={{
					redirect: location.pathname + location.search,
				}}
			/>
		);
	}

	return <>{children}</>;
};
