import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { ProtectedRoute } from "@/lib/auth/ProtectedRoute";
import type { ReactNode } from "react";
import { AdminHeader } from "./AdminHeader";
import { AdminSidebar } from "./AdminSidebar";

interface AdminLayoutProps {
	children: ReactNode;
}

export function AdminLayout({ children }: AdminLayoutProps) {
	return (
		<ProtectedRoute>
			<SidebarProvider>
				<AdminSidebar />
				<SidebarInset>
					<AdminHeader />
					<div className="flex flex-1 flex-col gap-4 p-4 pt-0">
						<main className="flex-1">{children}</main>
					</div>
				</SidebarInset>
			</SidebarProvider>
		</ProtectedRoute>
	);
}
