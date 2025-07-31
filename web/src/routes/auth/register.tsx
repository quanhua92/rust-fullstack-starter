import { RegisterForm } from "@/components/auth/RegisterForm";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/auth/register")({
	component: RegisterForm,
});
