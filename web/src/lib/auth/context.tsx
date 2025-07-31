import { apiClient, getAuthToken, setAuthToken } from "@/lib/api/client";
import type { components } from "@/types/api";
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
	login: (credentials: {
		username_or_email: string;
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

	// Automatic token refresh every 15 minutes
	useEffect(() => {
		if (!isAuthenticated) return;

		const refreshInterval = setInterval(async () => {
			const success = await refreshToken();
			if (!success) {
				console.log("Token refresh failed, user will be logged out");
			}
		}, 15 * 60 * 1000); // 15 minutes

		return () => clearInterval(refreshInterval);
	}, [isAuthenticated]);

	const login = async (credentials: {
		username_or_email: string;
		password: string;
	}) => {
		setIsLoading(true);
		try {
			const response = await apiClient.login(credentials);
			if (response.data?.user) {
				setUser(response.data.user);
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
		}
	};

	const refreshToken = async (): Promise<boolean> => {
		try {
			const response = await apiClient.refreshToken();
			if (response.success) {
				// Token is still valid, refresh user data
				await refreshUser();
				return true;
			}
			return false;
		} catch (error) {
			console.error("Token refresh failed:", error);
			// Token is invalid, clear user data
			setUser(null);
			setAuthToken(null);
			return false;
		}
	};

	const value: AuthContextType = {
		user,
		isLoading,
		isAuthenticated,
		login,
		register,
		logout,
		logoutAll,
		refreshUser,
		refreshToken,
	};

	return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};
