import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { AlertTriangle, RefreshCw } from "lucide-react";
import React from "react";

interface ErrorBoundaryState {
	hasError: boolean;
	error: Error | null;
	errorInfo: React.ErrorInfo | null;
}

interface ErrorBoundaryProps {
	children: React.ReactNode;
	fallback?: React.ComponentType<{ error: Error; retry: () => void }>;
}

export class ErrorBoundary extends React.Component<
	ErrorBoundaryProps,
	ErrorBoundaryState
> {
	constructor(props: ErrorBoundaryProps) {
		super(props);

		this.state = {
			hasError: false,
			error: null,
			errorInfo: null,
		};
	}

	static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
		return {
			hasError: true,
			error,
		};
	}

	componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
		this.setState({
			error,
			errorInfo,
		});

		// Log the error to monitoring service
		console.error("ErrorBoundary caught an error:", error, errorInfo);
	}

	handleRetry = () => {
		this.setState({
			hasError: false,
			error: null,
			errorInfo: null,
		});
	};

	render() {
		if (this.state.hasError) {
			// Custom fallback UI
			if (this.props.fallback) {
				const FallbackComponent = this.props.fallback;
				return (
					<FallbackComponent
						error={this.state.error || new Error("Unknown error")}
						retry={this.handleRetry}
					/>
				);
			}

			// Default fallback UI
			return (
				<div className="flex items-center justify-center min-h-[400px] p-4">
					<Card className="w-full max-w-md">
						<CardHeader>
							<CardTitle className="flex items-center space-x-2 text-red-600">
								<AlertTriangle className="h-5 w-5" />
								<span>Something went wrong</span>
							</CardTitle>
							<CardDescription>
								An unexpected error occurred while rendering this component.
							</CardDescription>
						</CardHeader>
						<CardContent className="space-y-4">
							{process.env.NODE_ENV === "development" && this.state.error && (
								<div className="text-xs bg-gray-100 p-3 rounded border font-mono overflow-auto max-h-32">
									<div className="font-semibold text-red-600 mb-1">Error:</div>
									<div>{this.state.error.message}</div>
									{this.state.errorInfo?.componentStack && (
										<>
											<div className="font-semibold text-red-600 mt-2 mb-1">
												Component Stack:
											</div>
											<div className="whitespace-pre-wrap text-xs">
												{this.state.errorInfo.componentStack.slice(0, 500)}
												{this.state.errorInfo.componentStack.length > 500
													? "..."
													: ""}
											</div>
										</>
									)}
								</div>
							)}
							<Button onClick={this.handleRetry} className="w-full">
								<RefreshCw className="h-4 w-4 mr-2" />
								Try Again
							</Button>
						</CardContent>
					</Card>
				</div>
			);
		}

		return this.props.children;
	}
}

// Hook version for functional components (when React 18+ error boundaries support)
export const useErrorHandler = () => {
	return (error: Error, errorInfo: React.ErrorInfo) => {
		console.error("Error caught by useErrorHandler:", error, errorInfo);
	};
};
