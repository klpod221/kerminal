import type { Component } from "vue";

export interface OverlayConfig {
  id: string;
  type: "drawer" | "modal";
  component?: Component | string;
  props?: Record<string, any>;
  parentId?: string | null;
  title?: string;
  icon?: Component;
  metadata?: Record<string, any>;
}

export interface OverlayState {
  config: OverlayConfig;
  visible: boolean;
  zIndex: number;
  createdAt: number;
}

export interface OverlayManagerState {
  overlays: Map<string, OverlayState>;
  activeOverlayId: string | null;
  history: string[];
  baseZIndex: number;
}
