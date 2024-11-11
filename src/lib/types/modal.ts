import type { Component } from "svelte";

export interface ViewProps {
    pushView: PushView;
    popView: PopView;
    closeModal: CloseModal;
}

export type PushView = (component: Component, props?: Record<string, unknown>) => void;
export type PopView = () => void;
export type CloseModal = () => void;

export interface ModalView {
    component: Component;
    props: Record<string, unknown>;
}
