// Test setup file
import { afterEach, beforeEach } from "vitest";
import { mockLocalStorage } from "./mocks";

// Setup mock localStorage globally
const mockStorage = mockLocalStorage();
Object.defineProperty(window, "localStorage", {
	value: mockStorage,
	writable: true,
});

// Clear localStorage before each test
beforeEach(() => {
	mockStorage.clear();
});

// Clean up after each test
afterEach(() => {
	// Reset all mocks
	if (global.fetch && typeof global.fetch.mockClear === "function") {
		global.fetch.mockClear();
	}
});
