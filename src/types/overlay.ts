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
  onBeforeOpen?: () => void | Promise<void>;
  onOpened?: () => void;
  onBeforeClose?: () => boolean | Promise<boolean>;
  onClosed?: () => void;
  onError?: (error: Error) => void;
}

export interface OverlayState {
  config: OverlayConfig;
  visible: boolean;
  transitioning: boolean;
  zIndex: number;
  createdAt: number;
  lastAccessedAt?: number;
}

export interface OverlayManagerState {
  overlays: Map<string, OverlayState>;
  activeOverlayId: string | null;
  history: string[];
  baseZIndex: number;
}
