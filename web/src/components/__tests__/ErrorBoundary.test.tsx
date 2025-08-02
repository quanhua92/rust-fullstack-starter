import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, afterEach } from "vitest";
import { ErrorBoundary } from "../ErrorBoundary";

const ThrowError = ({ shouldThrow }: { shouldThrow: boolean }) => {
	if (shouldThrow) {
		throw new Error("Test error");
	}
	return <div>No error</div>;
};

const mockConsoleError = vi
	.spyOn(console, "error")
	.mockImplementation(() => {});

describe("ErrorBoundary", () => {
	afterEach(() => {
		mockConsoleError.mockClear();
	});

	it("renders children when no error occurs", () => {
		render(
			<ErrorBoundary>
				<ThrowError shouldThrow={false} />
			</ErrorBoundary>,
		);

		expect(screen.getByText("No error")).toBeDefined();
	});

	it("renders error UI when error occurs", () => {
		render(
			<ErrorBoundary>
				<ThrowError shouldThrow={true} />
			</ErrorBoundary>,
		);

		expect(screen.getByText("Something went wrong")).toBeDefined();
		expect(
			screen.getByText(
				"An unexpected error occurred while rendering this component.",
			),
		).toBeDefined();
		expect(screen.getByRole("button", { name: /try again/i })).toBeDefined();
	});

	it("logs error to console", () => {
		render(
			<ErrorBoundary>
				<ThrowError shouldThrow={true} />
			</ErrorBoundary>,
		);

		expect(mockConsoleError).toHaveBeenCalledWith(
			"ErrorBoundary caught an error:",
			expect.any(Error),
			expect.any(Object),
		);
	});

	it("allows retry functionality", () => {
		let shouldThrow = true;
		const DynamicThrowError = () => {
			if (shouldThrow) {
				throw new Error("Test error");
			}
			return <div>No error</div>;
		};

		render(
			<ErrorBoundary>
				<DynamicThrowError />
			</ErrorBoundary>,
		);

		expect(screen.getByText("Something went wrong")).toBeDefined();

		// Fix the error condition
		shouldThrow = false;

		// Click retry button
		fireEvent.click(screen.getByRole("button", { name: /try again/i }));

		// Component should now render without error
		expect(screen.getByText("No error")).toBeDefined();
	});

	it("uses custom fallback component when provided", () => {
		const CustomFallback = ({
			error,
			retry,
		}: { error: Error; retry: () => void }) => (
			<div>
				<h1>Custom Error: {error.message}</h1>
				<button type="button" onClick={retry}>
					Custom Retry
				</button>
			</div>
		);

		render(
			<ErrorBoundary fallback={CustomFallback}>
				<ThrowError shouldThrow={true} />
			</ErrorBoundary>,
		);

		expect(screen.getByText("Custom Error: Test error")).toBeDefined();
		expect(screen.getByRole("button", { name: /custom retry/i })).toBeDefined();
	});
});
