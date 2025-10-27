import { ref, computed, onMounted, onUnmounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

export const BREAKPOINTS = {
  mobile: 480,
  tablet: 768,
  desktop: 1024,
} as const;

export type DeviceType = "mobile" | "tablet" | "desktop";

/**
 * Composable for tracking window size and device type
 * @returns Window size, device type, and breakpoint utilities
 */
export function useWindowSize() {
  const width = ref(0);
  const height = ref(0);

  const updateSize = async () => {
    const window = getCurrentWindow();
    const size = await window.innerSize();

    width.value = size.width;
    height.value = size.height;
  };

  const isMobile = computed(() => width.value < BREAKPOINTS.mobile);
  const isTablet = computed(
    () =>
      width.value >= BREAKPOINTS.mobile && width.value < BREAKPOINTS.desktop,
  );
  const isDesktop = computed(() => width.value >= BREAKPOINTS.desktop);

  const deviceType = computed<DeviceType>(() => {
    if (isMobile.value) return "mobile";
    if (isTablet.value) return "tablet";
    return "desktop";
  });

  const isTouch = computed(() => isMobile.value || isTablet.value);

  onMounted(() => {
    updateSize();
    window.addEventListener("resize", updateSize);
  });

  onUnmounted(() => {
    window.removeEventListener("resize", updateSize);
  });

  return {
    width,
    height,
    isMobile,
    isTablet,
    isDesktop,
    deviceType,
    isTouch,
  };
}
