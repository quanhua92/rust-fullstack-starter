import { toast } from "sonner";

interface ToastProps {
	title: string;
	description?: string;
	variant?: "default" | "destructive";
}

export const useToast = () => {
	return {
		toast: ({ title, description, variant = "default" }: ToastProps) => {
			if (variant === "destructive") {
				toast.error(title, {
					description,
				});
			} else {
				toast.success(title, {
					description,
				});
			}
		},
	};
};
