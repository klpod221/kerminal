import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { OverlayConfig, OverlayState } from "../types/overlay";

export const useOverlayStore = defineStore("overlay", () => {
  // State
  const overlays = ref<Map<string, OverlayState>>(new Map());
  const activeOverlayId = ref<string | null>(null);
  const history = ref<string[]>([]);
  const baseZIndex = ref(1000);

  // Getters
  const activeOverlay = computed(() => {
    if (!activeOverlayId.value) return null;
    return overlays.value.get(activeOverlayId.value) || null;
  });

  const hasActiveOverlay = computed(() => activeOverlayId.value !== null);

  const getOverlayById = (id: string) => overlays.value.get(id);

  const getChildrenOverlays = (parentId: string) => {
    return Array.from(overlays.value.values()).filter(
      (overlay) => overlay.config.parentId === parentId,
    );
  };

  const getParentOverlay = (childId: string) => {
    const child = overlays.value.get(childId);
    if (!child?.config.parentId) return null;
    return overlays.value.get(child.config.parentId);
  };

  // Actions
  const register = (config: OverlayConfig): void => {
    if (overlays.value.has(config.id)) {

      return;
    }

    const state: OverlayState = {
      config,
      visible: false,
      transitioning: false,
      zIndex: baseZIndex.value + overlays.value.size,
      createdAt: Date.now(),
      lastAccessedAt: Date.now(),
    };

    overlays.value.set(config.id, state);

  };

  const unregister = (id: string): void => {
    const overlay = overlays.value.get(id);
    if (!overlay) return;

    // Close if currently active
    if (activeOverlayId.value === id) {
      close(id);
    }

    // Remove from history
    history.value = history.value.filter((historyId) => historyId !== id);

    // Remove from overlays
    overlays.value.delete(id);

  };

  const open = async (id: string, props?: Record<string, any>): Promise<void> => {
    const overlay = overlays.value.get(id);
    if (!overlay) {

      return;
    }

    // Check if already transitioning
    if (overlay.transitioning) {
      console.warn(`⚠️ Overlay ${id} is already transitioning`);
      return;
    }

    try {
      // Set transitioning state
      overlay.transitioning = true;

      // Call onBeforeOpen hook
      if (overlay.config.onBeforeOpen) {
        await overlay.config.onBeforeOpen();
      }

      // Clear existing props first, then set new ones
      overlay.config.props = {};

      // Update props if provided
      if (props) {
        overlay.config.props = { ...props };
      }

      // Close current overlay if exists
      if (activeOverlayId.value && activeOverlayId.value !== id) {
        const currentOverlay = overlays.value.get(activeOverlayId.value);
        if (currentOverlay) {
          currentOverlay.visible = false;
          // Add to history if not already there
          if (!history.value.includes(activeOverlayId.value)) {
            history.value.push(activeOverlayId.value);
          }
        }
      }

      // Open new overlay
      overlay.visible = true;
      overlay.lastAccessedAt = Date.now();
      activeOverlayId.value = id;

      // Wait for animation to complete
      setTimeout(() => {
        overlay.transitioning = false;

        // Call onOpened hook
        if (overlay.config.onOpened) {
          overlay.config.onOpened();
        }
      }, 300); // Match animation duration

    } catch (error) {
      console.error(`❌ Error opening overlay ${id}:`, error);
      overlay.transitioning = false;

      // Call onError hook
      if (overlay.config.onError) {
        overlay.config.onError(error as Error);
      }

      throw error;
    }
  };

  const close = async (id?: string, clearPropsDelay: number = 300): Promise<void> => {
    const targetId = id || activeOverlayId.value;
    if (!targetId) return;

    const overlay = overlays.value.get(targetId);
    if (!overlay) return;

    try {
      // Check if already transitioning
      if (overlay.transitioning) {
        console.warn(`⚠️ Overlay ${targetId} is already transitioning`);
        return;
      }

      // Set transitioning state
      overlay.transitioning = true;

      // Call onBeforeClose hook - can cancel close
      if (overlay.config.onBeforeClose) {
        const canClose = await overlay.config.onBeforeClose();
        if (canClose === false) {
          overlay.transitioning = false;
          console.log(`🚫 Close cancelled for overlay: ${targetId}`);
          return;
        }
      }

      // Hide the overlay
      overlay.visible = false;

      // If this is the active overlay, handle navigation
      if (activeOverlayId.value === targetId) {
        activeOverlayId.value = null;

        // Find parent to open
        const parentId = overlay.config.parentId;
        if (parentId && overlays.value.has(parentId)) {
          // Open parent overlay
          const parentOverlay = overlays.value.get(parentId)!;
          parentOverlay.visible = true;
          activeOverlayId.value = parentId;

        } else {
          // No parent, check history
          let nextId: string | undefined;
          do {
            nextId = history.value.pop();
          } while (nextId && !overlays.value.has(nextId));

          if (nextId) {
            const nextOverlay = overlays.value.get(nextId)!;
            nextOverlay.visible = true;
            activeOverlayId.value = nextId;

          }
        }
      }

      // Clear props after animation completes (lazy clearing)
      setTimeout(() => {
        const currentOverlay = overlays.value.get(targetId);
        if (currentOverlay && !currentOverlay.visible) {
          currentOverlay.config.props = {};
          currentOverlay.transitioning = false;

          // Call onClosed hook
          if (currentOverlay.config.onClosed) {
            currentOverlay.config.onClosed();
          }
        }
      }, clearPropsDelay);

    } catch (error) {
      console.error(`❌ Error closing overlay ${targetId}:`, error);
      overlay.transitioning = false;

      // Call onError hook
      if (overlay.config.onError) {
        overlay.config.onError(error as Error);
      }

      throw error;
    }

    // Remove from history
    history.value = history.value.filter((historyId) => historyId !== targetId);

    console.log(`🔒 Closed overlay: ${targetId}`, {
      activeNow: activeOverlayId.value,
      history: history.value,
    });
  };

  const closeAll = (): void => {
    overlays.value.forEach((overlay) => {
      overlay.visible = false;
      // Clear props when closing all overlays
      overlay.config.props = {};
    });
    activeOverlayId.value = null;
    history.value = [];

  };

  const isVisible = (id: string): boolean => {
    const overlay = overlays.value.get(id);
    return overlay?.visible || false;
  };

  const isTransitioning = (id: string): boolean => {
    const overlay = overlays.value.get(id);
    return overlay?.transitioning || false;
  };

  // ESC key handler
  const handleEscapeKey = (): void => {
    if (activeOverlayId.value) {
      close(activeOverlayId.value);
    }
  };

  return {
    // State
    overlays,
    activeOverlayId,
    history,
    baseZIndex,

    // Getters
    activeOverlay,
    hasActiveOverlay,
    getOverlayById,
    getChildrenOverlays,
    getParentOverlay,

    // Actions
    register,
    unregister,
    open,
    close,
    closeAll,
    isVisible,
    isTransitioning,
    handleEscapeKey,
  };
});
