import { apiClient } from "@/lib/api/client";
import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";

/**
 * Smart health monitoring hook that adjusts query intervals based on:
 * - Page visibility (pause when tab is not active)
 * - Connection status (slower intervals on poor connection)
 * - Error rates (back off when services are failing)
 */
export const useSmartHealthQueries = () => {
	const [isPageVisible, setIsPageVisible] = useState(true);
	const [connectionQuality, setConnectionQuality] = useState<
		"good" | "poor" | "offline"
	>("good");
	const [errorCount, setErrorCount] = useState(0);

	// Track page visibility
	useEffect(() => {
		const handleVisibilityChange = () => {
			setIsPageVisible(!document.hidden);
		};

		document.addEventListener("visibilitychange", handleVisibilityChange);
		return () =>
			document.removeEventListener("visibilitychange", handleVisibilityChange);
	}, []);

	// Monitor connection quality
	useEffect(() => {
		const updateConnectionQuality = () => {
			if (!navigator.onLine) {
				setConnectionQuality("offline");
			} else if ("connection" in navigator) {
				const connection = (
					navigator as typeof navigator & {
						connection?: { effectiveType: string };
					}
				).connection;
				// Slow connections: 2G, slow-2g
				if (
					connection &&
					(connection.effectiveType === "2g" ||
						connection.effectiveType === "slow-2g")
				) {
					setConnectionQuality("poor");
				} else {
					setConnectionQuality("good");
				}
			} else {
				setConnectionQuality("good");
			}
		};

		updateConnectionQuality();
		window.addEventListener("online", updateConnectionQuality);
		window.addEventListener("offline", updateConnectionQuality);

		return () => {
			window.removeEventListener("online", updateConnectionQuality);
			window.removeEventListener("offline", updateConnectionQuality);
		};
	}, []);

	// Calculate smart intervals
	const getSmartInterval = (baseInterval: number, critical = false) => {
		if (connectionQuality === "offline") return false; // Don't query when offline
		if (!isPageVisible) return baseInterval * 3; // Slow down when tab not visible

		let multiplier = 1;

		// Adjust for connection quality
		if (connectionQuality === "poor") multiplier *= 2;

		// Exponential backoff for errors (up to 4x slower)
		if (errorCount > 0) {
			multiplier *= Math.min(1.5 ** errorCount, 4);
		}

		// Critical queries (liveness) should have minimum intervals
		if (critical && multiplier > 2) multiplier = 2;

		return baseInterval * multiplier;
	};

	// Error tracking wrapper
	const createQueryWithErrorTracking = <T>(
		queryFn: () => Promise<T>,
		queryKey: string[],
	) => {
		return useQuery({
			queryKey,
			queryFn: async () => {
				try {
					const result = await queryFn();
					// Reset error count on success
					setErrorCount((prev) => Math.max(0, prev - 1));
					return result;
				} catch (error) {
					// Increment error count on failure
					setErrorCount((prev) => Math.min(prev + 1, 5)); // Cap at 5
					throw error;
				}
			},
			refetchInterval: getSmartInterval(30000), // 30s base interval
			refetchOnWindowFocus: isPageVisible,
			retry: (failureCount) => {
				// Retry less aggressively when connection is poor or offline
				if (connectionQuality === "offline") return false;
				if (connectionQuality === "poor") return failureCount < 2;
				return failureCount < 3;
			},
		});
	};

	// Health query hooks with smart intervals
	const basicHealthQuery = createQueryWithErrorTracking(
		() => apiClient.getHealth(),
		["health", "basic"],
	);

	const detailedHealthQuery = createQueryWithErrorTracking(
		() => apiClient.getDetailedHealth(),
		["health", "detailed"],
	);

	const livenessQuery = useQuery({
		queryKey: ["health", "liveness"],
		queryFn: () => apiClient.getLivenessProbe(),
		refetchInterval: getSmartInterval(10000, true), // Critical - 10s base, max 2x slower
		refetchOnWindowFocus: isPageVisible,
		retry: connectionQuality === "offline" ? false : 2,
	});

	const readinessQuery = useQuery({
		queryKey: ["health", "readiness"],
		queryFn: () => apiClient.getReadinessProbe(),
		refetchInterval: getSmartInterval(15000), // 15s base interval
		refetchOnWindowFocus: isPageVisible,
		retry: connectionQuality === "offline" ? false : 2,
	});

	const startupQuery = useQuery({
		queryKey: ["health", "startup"],
		queryFn: () => apiClient.getStartupProbe(),
		refetchInterval: getSmartInterval(20000), // 20s base interval
		refetchOnWindowFocus: isPageVisible,
		retry: connectionQuality === "offline" ? false : 2,
	});

	return {
		basicHealthQuery,
		detailedHealthQuery,
		livenessQuery,
		readinessQuery,
		startupQuery,
		// Expose state for debugging/monitoring
		meta: {
			isPageVisible,
			connectionQuality,
			errorCount,
		},
	};
};
