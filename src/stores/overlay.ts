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
      console.warn(`Overlay with id "${config.id}" already exists`);
      return;
    }

    const state: OverlayState = {
      config,
      visible: false,
      zIndex: baseZIndex.value + overlays.value.size,
      createdAt: Date.now(),
    };

    overlays.value.set(config.id, state);
    console.log(`🔧 Registered overlay: ${config.id} (${config.type})`);
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
    console.log(`🗑️ Unregistered overlay: ${id}`);
  };

  const open = (id: string, props?: Record<string, any>): void => {
    const overlay = overlays.value.get(id);
    if (!overlay) {
      console.error(`Overlay with id "${id}" not found`);
      return;
    }

    // Update props if provided
    if (props) {
      overlay.config.props = { ...overlay.config.props, ...props };
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
    activeOverlayId.value = id;

    console.log(`🔓 Opened overlay: ${id}`, {
      type: overlay.config.type,
      parent: overlay.config.parentId,
      history: history.value,
      props: overlay.config.props,
    });
  };

  const close = (id?: string): void => {
    const targetId = id || activeOverlayId.value;
    if (!targetId) return;

    const overlay = overlays.value.get(targetId);
    if (!overlay) return;

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
        console.log(`⬆️ Opened parent overlay: ${parentId}`);
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
          console.log(`📖 Opened from history: ${nextId}`);
        }
      }
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
    });
    activeOverlayId.value = null;
    history.value = [];
    console.log("🔒 Closed all overlays");
  };

  const isVisible = (id: string): boolean => {
    const overlay = overlays.value.get(id);
    return overlay?.visible || false;
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
    handleEscapeKey,
  };
});
