import { Outlet, createRootRouteWithContext } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";

import { ErrorBoundary } from "@/components/ErrorBoundary";
import { Toaster } from "@/components/ui/sonner";
import { AuthProvider } from "@/lib/auth/context";
import TanStackQueryLayout from "../integrations/tanstack-query/layout.tsx";

import type { QueryClient } from "@tanstack/react-query";

interface MyRouterContext {
	queryClient: QueryClient;
}

export const Route = createRootRouteWithContext<MyRouterContext>()({
	component: () => (
		<ErrorBoundary>
			<AuthProvider>
				<Outlet />
				<Toaster />
				<TanStackRouterDevtools />
				<TanStackQueryLayout />
			</AuthProvider>
		</ErrorBoundary>
	),
});
