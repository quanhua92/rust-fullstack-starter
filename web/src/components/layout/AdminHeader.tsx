import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
	Breadcrumb,
	BreadcrumbItem,
	BreadcrumbLink,
	BreadcrumbList,
	BreadcrumbPage,
	BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { useAuth } from "@/lib/auth/context";
import { useLocation } from "@tanstack/react-router";
import { Bell, Search } from "lucide-react";

// Helper function to generate breadcrumbs from pathname
const generateBreadcrumbs = (pathname: string) => {
	const segments = pathname.split("/").filter(Boolean);
	const breadcrumbs = [];

	let currentPath = "";
	for (const segment of segments) {
		currentPath += `/${segment}`;

		// Skip the first 'admin' segment for cleaner breadcrumbs
		if (segment === "admin" && segments.length > 1) continue;

		// Convert segment to readable title
		const title = segment
			.split("-")
			.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
			.join(" ");

		breadcrumbs.push({
			title,
			path: currentPath,
		});
	}

	return breadcrumbs;
};

export function AdminHeader() {
	const { user, logout } = useAuth();
	const location = useLocation();

	const breadcrumbs = generateBreadcrumbs(location.pathname);

	const handleLogout = async () => {
		try {
			await logout();
		} catch (error) {
			console.error("Logout failed:", error);
		}
	};

	return (
		<header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
			<SidebarTrigger className="-ml-1" />
			<Separator orientation="vertical" className="mr-2 h-4" />

			{/* Breadcrumbs */}
			<Breadcrumb>
				<BreadcrumbList>
					<BreadcrumbItem>
						<BreadcrumbLink href="/admin">Admin</BreadcrumbLink>
					</BreadcrumbItem>
					{breadcrumbs.map((breadcrumb, index) => (
						<div key={breadcrumb.path} className="flex items-center">
							<BreadcrumbSeparator />
							<BreadcrumbItem>
								{index === breadcrumbs.length - 1 ? (
									<BreadcrumbPage>{breadcrumb.title}</BreadcrumbPage>
								) : (
									<BreadcrumbLink href={breadcrumb.path}>
										{breadcrumb.title}
									</BreadcrumbLink>
								)}
							</BreadcrumbItem>
						</div>
					))}
				</BreadcrumbList>
			</Breadcrumb>

			{/* Spacer */}
			<div className="flex-1" />

			{/* Search */}
			<div className="relative w-64">
				<Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
				<Input type="search" placeholder="Search..." className="pl-8" />
			</div>

			{/* Notifications */}
			<Button variant="ghost" size="icon">
				<Bell className="h-4 w-4" />
				<span className="sr-only">Notifications</span>
			</Button>

			{/* User Menu */}
			<DropdownMenu>
				<DropdownMenuTrigger asChild>
					<Button variant="ghost" className="relative h-8 w-8 rounded-full">
						<Avatar className="h-8 w-8">
							<AvatarFallback>
								{user?.username?.charAt(0).toUpperCase() || "U"}
							</AvatarFallback>
						</Avatar>
					</Button>
				</DropdownMenuTrigger>
				<DropdownMenuContent className="w-56" align="end" forceMount>
					<div className="flex items-center justify-start gap-2 p-2">
						<div className="flex flex-col space-y-1 leading-none">
							<p className="font-medium">{user?.username || "User"}</p>
							<p className="w-[200px] truncate text-sm text-muted-foreground">
								{user?.email || "user@example.com"}
							</p>
						</div>
					</div>
					<DropdownMenuSeparator />
					<DropdownMenuItem>Profile Settings</DropdownMenuItem>
					<DropdownMenuItem>Preferences</DropdownMenuItem>
					<DropdownMenuSeparator />
					<DropdownMenuItem onClick={handleLogout} className="text-red-600">
						Log out
					</DropdownMenuItem>
				</DropdownMenuContent>
			</DropdownMenu>
		</header>
	);
}
