import { computed, type ComputedRef } from "vue";
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

  const openOverlay = async (id: string, props?: Record<string, any>) => {
    if (typeof overlayStore.open === "function") {
      await overlayStore.open(id, props);
    } else {
      console.error("overlayStore.open is not a function");
    }
  };

  const closeOverlay = async (id?: string) => {
    await overlayStore.close(id);
  };

  const closeAllOverlays = () => {
    overlayStore.closeAll();
  };

  const isOverlayVisible = (id: string) => {
    return overlayStore.isVisible(id);
  };

  const isOverlayTransitioning = (id: string) => {
    return overlayStore.isTransitioning(id);
  };

  /**
   * Clear props for a specific overlay
   * @param id - The overlay ID to clear props for
   */
  const clearOverlayProps = (id: string) => {
    const overlay = overlayStore.getOverlayById(id);
    if (overlay) {
      overlay.config.props = {};
    }
  };

  /**
   * Get specific prop value with overlay precedence
   *
   * @param overlayId - The ID of the overlay to get props from
   * @param propName - Name of the prop to get
   * @param directValue - Direct prop value
   * @param defaultValue - Default value if neither overlay nor direct prop exists
   * @returns The prop value with overlay taking precedence
   */
  const getOverlayProp = <T>(
    overlayId: string,
    propName: string,
    directValue: T,
    defaultValue?: T,
  ) => {
    return computed(() => {
      const overlay = overlayStore.getOverlayById(overlayId);
      const overlayProps = overlay?.config.props || {};

      return overlayProps[propName] ?? directValue ?? defaultValue;
    });
  };

  /**
   * Automatically merge props from overlay system with direct component props
   * Overlay props take precedence over direct props
   *
   * @param overlayId - The ID of the overlay to get props from
   * @param directProps - Direct props passed to the component
   * @returns Merged props with overlay props taking precedence
   */
  const getOverlayProps = <T extends Record<string, any>>(
    overlayId: string,
    directProps: T,
  ) => {
    return computed(() => {
      const overlay = overlayStore.getOverlayById(overlayId);
      const overlayProps = overlay?.config.props || {};

      const mergedProps = { ...directProps };

      Object.keys(overlayProps).forEach((key) => {
        if (overlayProps[key] !== undefined && overlayProps[key] !== null) {
          mergedProps[key as keyof T] = overlayProps[key];
        }
      });

      return mergedProps;
    });
  };

  /**
   * Create reactive prop getters for easier usage
   *
   * @param overlayId - The ID of the overlay
   * @param propNames - Array of prop names to create getters for
   * @param directProps - Direct props object
   * @returns Object with reactive prop getters
   */
  const getOverlayPropGetters = <T extends Record<string, any>>(
    overlayId: string,
    propNames: (keyof T)[],
    directProps: T,
  ) => {
    const getters = {} as { [K in keyof T]: ComputedRef<T[K]> };

    propNames.forEach((propName) => {
      getters[propName] = computed(() => {
        const overlay = overlayStore.getOverlayById(overlayId);
        const overlayProps = overlay?.config.props || {};

        return (
          overlayProps[propName as string] ?? directProps[propName] ?? null
        );
      });
    });

    return getters;
  };

  return {
    overlayStore,

    registerOverlay,
    unregisterOverlay,
    openOverlay,
    closeOverlay,
    closeAllOverlays,
    isOverlayVisible,
    isOverlayTransitioning,
    clearOverlayProps,

    getOverlayProp,
    getOverlayProps,
    getOverlayPropGetters,

    activeOverlay: overlayStore.activeOverlay,
    hasActiveOverlay: overlayStore.hasActiveOverlay,

    updateOverlayProp: (overlayId: string, propName: string, value: any) => {
        const overlay = overlayStore.getOverlayById(overlayId);
        if (overlay) {
            if (!overlay.config.props) {
                overlay.config.props = {};
            }
            overlay.config.props[propName] = value;
        }
    }
  };
}

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
    onBeforeOpen: options.onBeforeOpen,
    onOpened: options.onOpened,
    onBeforeClose: options.onBeforeClose,
    onClosed: options.onClosed,
    onError: options.onError,
  };
}
