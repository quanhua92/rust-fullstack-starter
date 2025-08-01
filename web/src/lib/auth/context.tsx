import { apiClient, getAuthToken, setAuthToken } from "@/lib/api/client";
import type { components } from "@/types/api";
import type { UserRole } from "@/lib/rbac/types";
import { hasRoleOrHigher } from "@/lib/rbac/types";
import {
	type ReactNode,
	createContext,
	useContext,
	useEffect,
	useState,
} from "react";

type AuthUser = components["schemas"]["AuthUser"];

interface AuthContextType {
	user: AuthUser | null;
	isLoading: boolean;
	isAuthenticated: boolean;
	// Legacy aliases for compatibility
	loading: boolean;
	authenticated: boolean;
	login: (credentials: {
		username?: string;
		email?: string;
		password: string;
	}) => Promise<void>;
	register: (data: {
		username: string;
		email: string;
		password: string;
	}) => Promise<void>;
	logout: () => Promise<void>;
	logoutAll: () => Promise<void>;
	refreshUser: () => Promise<void>;
	refreshToken: () => Promise<boolean>;
	// RBAC helper methods
	hasRole: (requiredRole: UserRole) => boolean;
	isAdmin: () => boolean;
	isModerator: () => boolean;
	isModeratorOrHigher: () => boolean;
}

const AuthContext = createContext<AuthContextType | null>(null);

export const useAuth = () => {
	const context = useContext(AuthContext);
	if (!context) {
		throw new Error("useAuth must be used within an AuthProvider");
	}
	return context;
};

interface AuthProviderProps {
	children: ReactNode;
}

export const AuthProvider = ({ children }: AuthProviderProps) => {
	const [user, setUser] = useState<AuthUser | null>(null);
	const [isLoading, setIsLoading] = useState(true);
	const [tokenExpiration, setTokenExpiration] = useState<string | null>(null);

	const isAuthenticated = !!user;

	// Check if user is logged in on app start
	useEffect(() => {
		const initAuth = async () => {
			const token = getAuthToken();
			if (token) {
				try {
					const response = await apiClient.getCurrentUser();
					if (response.data) {
						setUser(response.data);
					}
				} catch (error) {
					console.error("Failed to fetch current user:", error);
					// Clear invalid token
					setAuthToken(null);
				}
			}
			setIsLoading(false);
		};

		initAuth();
	}, []);

	// Smart token refresh based on expiration time
	useEffect(() => {
		if (!isAuthenticated || !tokenExpiration) return;

		const scheduleRefresh = () => {
			const expirationTime = new Date(tokenExpiration).getTime();
			const currentTime = Date.now();
			const timeUntilExpiration = expirationTime - currentTime;

			// Refresh when 75% of the token lifetime has passed, with minimum 5 minutes before expiration
			const refreshTime = Math.max(
				timeUntilExpiration * 0.25, // 25% before expiration (75% of lifetime passed)
				5 * 60 * 1000, // At least 5 minutes before expiration
			);

			// Don't schedule if token is already expired or expires very soon
			if (timeUntilExpiration <= 60 * 1000) {
				// Less than 1 minute remaining
				console.log("Token expires very soon, forcing refresh now");
				refreshToken();
				return;
			}

			console.log(
				`Scheduling token refresh in ${Math.round(refreshTime / 1000 / 60)} minutes`,
			);

			const timeoutId = setTimeout(async () => {
				const success = await refreshToken();
				if (!success) {
					console.log("Token refresh failed, user will be logged out");
				}
			}, refreshTime);

			return timeoutId;
		};

		const timeoutId = scheduleRefresh();
		return () => {
			if (timeoutId) clearTimeout(timeoutId);
		};
	}, [isAuthenticated, tokenExpiration]);

	const login = async (credentials: {
		username?: string;
		email?: string;
		password: string;
	}) => {
		setIsLoading(true);
		try {
			const response = await apiClient.login(credentials);
			if (response.data?.user) {
				setUser(response.data.user);
				// Store token expiration for smart refresh scheduling
				if (response.data.expires_at) {
					setTokenExpiration(response.data.expires_at);
				}
			}
		} catch (error) {
			console.error("Login failed:", error);
			throw error;
		} finally {
			setIsLoading(false);
		}
	};

	const register = async (data: {
		username: string;
		email: string;
		password: string;
	}) => {
		setIsLoading(true);
		try {
			await apiClient.register(data);
			// After registration, user needs to login
			// Auto-login could be implemented here if the API returns a session token
		} catch (error) {
			console.error("Registration failed:", error);
			throw error;
		} finally {
			setIsLoading(false);
		}
	};

	const logout = async () => {
		setIsLoading(true);
		try {
			await apiClient.logout();
		} catch (error) {
			console.error("Logout failed:", error);
			// Continue with logout even if API call fails
		} finally {
			setUser(null);
			setAuthToken(null);
			setTokenExpiration(null);
			setIsLoading(false);
		}
	};

	const logoutAll = async () => {
		setIsLoading(true);
		try {
			await apiClient.logoutAll();
		} catch (error) {
			console.error("Logout all failed:", error);
			// Continue with logout even if API call fails
		} finally {
			setUser(null);
			setAuthToken(null);
			setTokenExpiration(null);
			setIsLoading(false);
		}
	};

	const refreshUser = async () => {
		try {
			const response = await apiClient.getCurrentUser();
			if (response.data) {
				setUser(response.data);
			}
		} catch (error) {
			console.error("Failed to refresh user:", error);
			// If refresh fails, user might need to login again
			setUser(null);
			setAuthToken(null);
			setTokenExpiration(null);
		}
	};

	const refreshToken = async (): Promise<boolean> => {
		try {
			const response = await apiClient.refreshToken();
			if (response.success && response.data) {
				// Token was successfully refreshed with new expiration
				console.log(
					`Token refreshed successfully. New expiration: ${response.data.expires_at}`,
				);
				// Update stored expiration time for smart refresh scheduling
				setTokenExpiration(response.data.expires_at);
				// Refresh user data to ensure consistency
				await refreshUser();
				return true;
			}
			return false;
		} catch (error) {
			console.error("Token refresh failed:", error);
			// Token is invalid, clear user data
			setUser(null);
			setAuthToken(null);
			setTokenExpiration(null);
			return false;
		}
	};

	// RBAC helper functions
	const hasRole = (requiredRole: UserRole): boolean => {
		if (!user) return false;
		return hasRoleOrHigher(user.role, requiredRole);
	};

	const isAdmin = (): boolean => {
		return hasRole("admin");
	};

	const isModerator = (): boolean => {
		return hasRole("moderator");
	};

	const isModeratorOrHigher = (): boolean => {
		return hasRole("moderator");
	};

	const value: AuthContextType = {
		user,
		isLoading,
		isAuthenticated,
		// Legacy aliases for compatibility
		loading: isLoading,
		authenticated: isAuthenticated,
		login,
		register,
		logout,
		logoutAll,
		refreshUser,
		refreshToken,
		// RBAC helper methods
		hasRole,
		isAdmin,
		isModerator,
		isModeratorOrHigher,
	};

	return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};
