import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { OverlayConfig, OverlayState } from "../types/overlay";

export const useOverlayStore = defineStore("overlay", () => {
  const overlays = ref<Map<string, OverlayState>>(new Map());
  const activeOverlayId = ref<string | null>(null);
  const history = ref<string[]>([]);
  const baseZIndex = ref(1000);

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

    if (activeOverlayId.value === id) {
      close(id);
    }

    history.value = history.value.filter((historyId) => historyId !== id);

    overlays.value.delete(id);
  };

  const open = async (
    id: string,
    props?: Record<string, any>,
  ): Promise<void> => {
    const overlay = overlays.value.get(id);
    if (!overlay) {
      return;
    }

    if (overlay.transitioning) {
      console.warn(`⚠️ Overlay ${id} is already transitioning`);
      return;
    }

    try {
      overlay.transitioning = true;

      if (overlay.config.onBeforeOpen) {
        await overlay.config.onBeforeOpen();
      }

      overlay.config.props = {};

      if (props) {
        overlay.config.props = { ...props };
      }

      if (activeOverlayId.value && activeOverlayId.value !== id) {
        const currentOverlay = overlays.value.get(activeOverlayId.value);
        if (currentOverlay) {
          currentOverlay.visible = false;
          if (!history.value.includes(activeOverlayId.value)) {
            history.value.push(activeOverlayId.value);
          }
        }
      }

      overlay.visible = true;
      overlay.lastAccessedAt = Date.now();
      activeOverlayId.value = id;

      setTimeout(() => {
        overlay.transitioning = false;

        if (overlay.config.onOpened) {
          overlay.config.onOpened();
        }
      }, 300); // Match animation duration
    } catch (error) {
      console.error(`❌ Error opening overlay ${id}:`, error);
      overlay.transitioning = false;

      if (overlay.config.onError) {
        overlay.config.onError(error as Error);
      }

      throw error;
    }
  };

  const close = async (
    id?: string,
    clearPropsDelay: number = 300,
  ): Promise<void> => {
    const targetId = id || activeOverlayId.value;
    if (!targetId) return;

    const overlay = overlays.value.get(targetId);
    if (!overlay) return;

    try {
      if (overlay.transitioning) {
        console.warn(`⚠️ Overlay ${targetId} is already transitioning`);
        return;
      }

      overlay.transitioning = true;

      if (overlay.config.onBeforeClose) {
        const canClose = await overlay.config.onBeforeClose();
        if (canClose === false) {
          overlay.transitioning = false;
          console.log(`🚫 Close cancelled for overlay: ${targetId}`);
          return;
        }
      }

      overlay.visible = false;

      if (activeOverlayId.value === targetId) {
        activeOverlayId.value = null;

        const parentId = overlay.config.parentId;
        if (parentId && overlays.value.has(parentId)) {
          const parentOverlay = overlays.value.get(parentId)!;
          parentOverlay.visible = true;
          activeOverlayId.value = parentId;
        } else {
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

      setTimeout(() => {
        const currentOverlay = overlays.value.get(targetId);
        if (currentOverlay && !currentOverlay.visible) {
          currentOverlay.config.props = {};
          currentOverlay.transitioning = false;

          if (currentOverlay.config.onClosed) {
            currentOverlay.config.onClosed();
          }
        }
      }, clearPropsDelay);
    } catch (error) {
      console.error(`❌ Error closing overlay ${targetId}:`, error);
      overlay.transitioning = false;

      if (overlay.config.onError) {
        overlay.config.onError(error as Error);
      }

      throw error;
    }

    history.value = history.value.filter((historyId) => historyId !== targetId);

    console.log(`🔒 Closed overlay: ${targetId}`, {
      activeNow: activeOverlayId.value,
      history: history.value,
    });
  };

  const closeAll = (): void => {
    overlays.value.forEach((overlay) => {
      overlay.visible = false;
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

  const handleEscapeKey = (): void => {
    if (activeOverlayId.value) {
      close(activeOverlayId.value);
    }
  };

  return {
    overlays,
    activeOverlayId,
    history,
    baseZIndex,

    activeOverlay,
    hasActiveOverlay,
    getOverlayById,
    getChildrenOverlays,
    getParentOverlay,

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
