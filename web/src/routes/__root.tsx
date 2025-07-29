import { Outlet, createRootRouteWithContext } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";

import TanStackQueryLayout from "../integrations/tanstack-query/layout.tsx";
import { AuthProvider } from "@/lib/auth/context";
import { Toaster } from "@/components/ui/sonner";

import type { QueryClient } from "@tanstack/react-query";

interface MyRouterContext {
	queryClient: QueryClient;
}

export const Route = createRootRouteWithContext<MyRouterContext>()({
	component: () => (
		<AuthProvider>
			<Outlet />
			<Toaster />
			<TanStackRouterDevtools />
			<TanStackQueryLayout />
		</AuthProvider>
	),
});
