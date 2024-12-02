import type { Component } from "svelte";

export interface ModalView {
    component: Component;
    modalProps?: Record<string, unknown>;
}

export interface ViewProps {
    pushView?: PushView;
    popView?: PopView;
    closeModal?: CloseModal;
}

export type PushView = (component: Component, modalProps?: Record<string, unknown>) => void;
export type PopView = () => void;
export type CloseModal = () => void;
