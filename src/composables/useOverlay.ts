import { useOverlayStore } from "../stores/overlay";
import type { OverlayConfig } from "../types/overlay";

export function useOverlay() {
  const overlayStore = useOverlayStore();

  const registerOverlay = (config: OverlayConfig) => {
    overlayStore.register(config);
  };

  const unregisterOverlay = (id: string) => {
    overlayStore.unregister(id);
  };

  const openOverlay = (id: string, props?: Record<string, any>) => {
    if (typeof overlayStore.open === "function") {
      overlayStore.open(id, props);
    } else {
      console.error("overlayStore.open is not a function");
    }
  };

  const closeOverlay = (id?: string) => {
    overlayStore.close(id);
  };

  const closeAllOverlays = () => {
    overlayStore.closeAll();
  };

  const isOverlayVisible = (id: string) => {
    return overlayStore.isVisible(id);
  };

  return {
    // Store access
    overlayStore,

    // Actions
    registerOverlay,
    unregisterOverlay,
    openOverlay,
    closeOverlay,
    closeAllOverlays,
    isOverlayVisible,

    // Computed
    activeOverlay: overlayStore.activeOverlay,
    hasActiveOverlay: overlayStore.hasActiveOverlay,
  };
}

// Helper function to create overlay configs
export function createOverlayConfig(
  id: string,
  type: "drawer" | "modal",
  options: Partial<OverlayConfig> = {},
): OverlayConfig {
  return {
    id,
    type,
    component: options.component,
    props: options.props || {},
    parentId: options.parentId,
    title: options.title,
    icon: options.icon,
    metadata: options.metadata || {},
  };
}
