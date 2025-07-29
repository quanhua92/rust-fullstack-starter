import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Separator } from "@/components/ui/separator";
import {
	Sidebar,
	SidebarContent,
	SidebarFooter,
	SidebarGroup,
	SidebarGroupContent,
	SidebarGroupLabel,
	SidebarHeader,
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem,
	SidebarMenuSub,
	SidebarMenuSubButton,
	SidebarMenuSubItem,
	SidebarRail,
} from "@/components/ui/sidebar";
import { useAuth } from "@/lib/auth/context";
import { Link, useLocation } from "@tanstack/react-router";
import {
	BarChart3,
	CheckSquare,
	Database,
	Heart,
	Home,
	LogOut,
	Users,
} from "lucide-react";

const menuItems = [
	{
		title: "Dashboard",
		url: "/admin",
		icon: Home,
	},
	{
		title: "Tasks",
		icon: CheckSquare,
		items: [
			{
				title: "All Tasks",
				url: "/admin/tasks",
			},
			{
				title: "Create Task",
				url: "/admin/tasks/create",
			},
			{
				title: "Dead Letter Queue",
				url: "/admin/tasks/dead-letter",
			},
			{
				title: "Task Types",
				url: "/admin/tasks/types",
			},
		],
	},
	{
		title: "Users",
		url: "/admin/users",
		icon: Users,
	},
	{
		title: "Health & Monitoring",
		icon: Heart,
		items: [
			{
				title: "Health Dashboard",
				url: "/admin/health",
			},
		],
	},
	{
		title: "Analytics",
		url: "/admin/analytics",
		icon: BarChart3,
	},
];

export function AdminSidebar() {
	const location = useLocation();
	const { user, logout } = useAuth();

	const handleLogout = async () => {
		try {
			await logout();
		} catch (error) {
			console.error("Logout failed:", error);
		}
	};

	return (
		<Sidebar collapsible="icon">
			<SidebarHeader>
				<SidebarMenu>
					<SidebarMenuItem>
						<SidebarMenuButton size="lg" asChild>
							<Link to="/admin">
								<div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
									<Database className="size-4" />
								</div>
								<div className="grid flex-1 text-left text-sm leading-tight">
									<span className="truncate font-semibold">
										Rust Starter Admin
									</span>
									<span className="truncate text-xs">Admin Portal</span>
								</div>
							</Link>
						</SidebarMenuButton>
					</SidebarMenuItem>
				</SidebarMenu>
			</SidebarHeader>

			<SidebarContent>
				<SidebarGroup>
					<SidebarGroupLabel>Navigation</SidebarGroupLabel>
					<SidebarGroupContent>
						<SidebarMenu>
							{menuItems.map((item) => (
								<SidebarMenuItem key={item.title}>
									{item.items ? (
										<SidebarMenuButton className="w-full">
											<item.icon className="size-4" />
											<span>{item.title}</span>
										</SidebarMenuButton>
									) : (
										<SidebarMenuButton
											asChild
											isActive={location.pathname === item.url}
										>
											<Link to={item.url || "#"}>
												<item.icon className="size-4" />
												<span>{item.title}</span>
											</Link>
										</SidebarMenuButton>
									)}
									{item.items && (
										<SidebarMenuSub>
											{item.items.map((subItem) => (
												<SidebarMenuSubItem key={subItem.title}>
													<SidebarMenuSubButton
														asChild
														isActive={location.pathname === subItem.url}
													>
														<Link to={subItem.url}>
															<span>{subItem.title}</span>
														</Link>
													</SidebarMenuSubButton>
												</SidebarMenuSubItem>
											))}
										</SidebarMenuSub>
									)}
								</SidebarMenuItem>
							))}
						</SidebarMenu>
					</SidebarGroupContent>
				</SidebarGroup>
			</SidebarContent>

			<SidebarFooter>
				<SidebarMenu>
					<SidebarMenuItem>
						<div className="flex items-center gap-2 px-2 py-1">
							<Avatar className="h-8 w-8">
								<AvatarFallback>
									{user?.username?.charAt(0).toUpperCase() || "U"}
								</AvatarFallback>
							</Avatar>
							<div className="flex-1 min-w-0">
								<p className="text-sm font-medium truncate">
									{user?.username || "User"}
								</p>
								<p className="text-xs text-muted-foreground truncate">
									{user?.role || "user"}
								</p>
							</div>
						</div>
						<Separator className="my-2" />
						<SidebarMenuButton onClick={handleLogout} className="text-red-600">
							<LogOut className="size-4" />
							<span>Logout</span>
						</SidebarMenuButton>
					</SidebarMenuItem>
				</SidebarMenu>
			</SidebarFooter>

			<SidebarRail />
		</Sidebar>
	);
}
