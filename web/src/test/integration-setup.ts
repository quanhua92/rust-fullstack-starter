// Integration test setup utilities
import { beforeAll } from "vitest";
import { waitForServer } from "./mocks";

const testServerConfig = {
	baseUrl: "http://localhost:3000/api/v1",
	timeout: 10000,
};

// Helper for integration tests that need a running server
export const setupIntegrationTest = () => {
	let serverReady = false;

	beforeAll(async () => {
		if (!serverReady) {
			console.log("Waiting for server to be ready...");
			await waitForServer(testServerConfig.baseUrl);
			serverReady = true;
			console.log("Server is ready!");
		}
	}, testServerConfig.timeout);

	return {
		baseUrl: testServerConfig.baseUrl,
	};
};

// Helper to run tests only if server is available
export const describeIntegration = (name: string, fn: () => void) => {
	const shouldSkip = process.env.SKIP_INTEGRATION === "true";

	// Use global describe from vitest
	const globalDescribe = (
		globalThis as unknown as {
			describe: {
				(name: string, fn: () => void): void;
				skip: (name: string, fn: () => void) => void;
			};
		}
	).describe;

	if (shouldSkip) {
		globalDescribe.skip(`${name} (INTEGRATION SKIPPED)`, fn);
	} else {
		globalDescribe(`${name} (INTEGRATION)`, fn);
	}
};
