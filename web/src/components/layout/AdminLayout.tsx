import { SidebarProvider, SidebarInset } from "@/components/ui/sidebar";
import { AdminSidebar } from "./AdminSidebar";
import { AdminHeader } from "./AdminHeader";
import { ProtectedRoute } from "@/lib/auth/ProtectedRoute";
import type { ReactNode } from "react";

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
