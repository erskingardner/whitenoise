import { getContext, setContext } from "svelte";

export type Toast = {
    id: string;
    title: string;
    message: string;
    type: "error" | "success" | "info";
};

export class ToastState {
    toasts = $state<Toast[]>([]);
    toastTimeoutMap = new Map<string, number>();

    add(title: string, message: string, type: "error" | "success" | "info", durationMs = 10_000) {
        const id = crypto.randomUUID();
        this.toasts.push({ id, title, message, type });

        this.toastTimeoutMap.set(
            id,
            setTimeout(() => {
                this.remove(id);
            }, durationMs)
        );
    }

    remove(id: string) {
        const timeout = this.toastTimeoutMap.get(id);
        if (timeout) {
            clearTimeout(timeout);
            this.toastTimeoutMap.delete(id);
        }
        this.toasts = this.toasts.filter((toast) => toast.id !== id);
    }

    cleanup() {
        for (const timeout of this.toastTimeoutMap.values()) {
            clearTimeout(timeout);
        }
        this.toastTimeoutMap.clear();
    }
}

export const toastState = new ToastState();

const TOAST_KEY = Symbol("WhitenoiseToastKey");

export function setToastState() {
    return setContext(TOAST_KEY, new ToastState());
}

export function getToastState() {
    return getContext<ReturnType<typeof setToastState>>(TOAST_KEY);
}
