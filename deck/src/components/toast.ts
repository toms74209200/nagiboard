import { signal } from "@preact/signals";

export type ToastKind = "info" | "error";
export type ToastMessage = { readonly text: string; readonly kind: ToastKind };

export const toastMessage = signal<ToastMessage | null>(null);

let timer: ReturnType<typeof setTimeout> | undefined;

export const showToast = (text: string, kind: ToastKind = "info"): void => {
	toastMessage.value = { text, kind };
	clearTimeout(timer);
	timer = setTimeout(() => {
		toastMessage.value = null;
	}, 3200);
};
