import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
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
import {
	getRoleDisplayName,
	getRoleColorClasses,
	type UserRole,
} from "@/lib/rbac/types";
import { Link, useLocation } from "@tanstack/react-router";
import {
	BarChart3,
	CheckSquare,
	Database,
	Heart,
	Home,
	LogOut,
	Users,
	UserCheck,
} from "lucide-react";

interface MenuItem {
	title: string;
	url?: string;
	icon: React.ComponentType<{ className?: string }>;
	visible?: boolean;
	items?: SubMenuItem[];
}

interface SubMenuItem {
	title: string;
	url: string;
	visible?: boolean;
}

const getMenuItems = (
	isModeratorOrHigher: boolean,
	isAdmin: boolean,
): MenuItem[] =>
	[
		{
			title: "Dashboard",
			url: "/admin",
			icon: Home,
		},
		{
			title: "Tasks",
			icon: CheckSquare,
			visible: true, // All authenticated users can see tasks
			items: [
				{
					title: "All Tasks",
					url: "/admin/tasks",
				},
				{
					title: "Create Task",
					url: "/admin/tasks/new",
				},
				{
					title: "Dead Letter Queue",
					url: "/admin/tasks/dead-letter",
					visible: isModeratorOrHigher, // Only moderator+ can see dead letter queue
				},
			],
		},
		{
			title: "Users",
			icon: Users,
			visible: isModeratorOrHigher, // Only moderator+ can see user management
			items: [
				{
					title: "All Users",
					url: "/admin/users",
				},
				{
					title: "Create User",
					url: "/admin/users/new",
					visible: isAdmin, // Only admin can create users
				},
				{
					title: "User Analytics",
					url: "/admin/users/analytics",
					visible: isAdmin, // Only admin can see analytics
				},
			],
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
			visible: isAdmin, // Only admin can see main analytics
		},
	].filter((item) => item.visible !== false);

export function AdminSidebar() {
	const location = useLocation();
	const { user, logout, isModeratorOrHigher, isAdmin } = useAuth();

	const menuItems = getMenuItems(isModeratorOrHigher(), isAdmin());

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
											{item.items
												.filter((subItem) => subItem.visible !== false)
												.map((subItem) => (
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
						<SidebarMenuButton asChild size="lg">
							<Link to="/admin">
								{/* Changed to existing route for now */}
								<Avatar className="h-8 w-8">
									<AvatarFallback>
										{user?.username?.charAt(0).toUpperCase() || "U"}
									</AvatarFallback>
								</Avatar>
								<div className="flex-1 min-w-0">
									<p className="text-sm font-medium truncate">
										{user?.username || "User"}
									</p>
									<div className="flex items-center gap-2">
										<Badge
											variant="outline"
											className={`${getRoleColorClasses(user?.role as UserRole).text} ${getRoleColorClasses(user?.role as UserRole).border} text-xs`}
										>
											{getRoleDisplayName(user?.role as UserRole)}
										</Badge>
									</div>
								</div>
								<UserCheck className="size-4" />
							</Link>
						</SidebarMenuButton>
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
