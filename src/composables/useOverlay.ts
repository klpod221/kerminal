import { onMounted, onUnmounted } from 'vue'
import { useOverlayStore } from '../stores/overlay'
import type { OverlayConfig } from '../types/overlay'

export function useOverlay() {
  const overlayStore = useOverlayStore()

  // Register global ESC key handler
  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      overlayStore.handleEscapeKey()
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeyDown)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeyDown)
  })

  const registerOverlay = (config: OverlayConfig) => {
    overlayStore.register(config)
  }

  const unregisterOverlay = (id: string) => {
    overlayStore.unregister(id)
  }

  const openOverlay = (id: string, props?: Record<string, any>) => {
    overlayStore.open(id, props)
  }

  const closeOverlay = (id?: string) => {
    overlayStore.close(id)
  }

  const closeAllOverlays = () => {
    overlayStore.closeAll()
  }

  const isOverlayVisible = (id: string) => {
    return overlayStore.isVisible(id)
  }

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
    hasActiveOverlay: overlayStore.hasActiveOverlay
  }
}

// Helper function to create overlay configs
export function createOverlayConfig(
  id: string,
  type: 'drawer' | 'modal',
  options: Partial<OverlayConfig> = {}
): OverlayConfig {
  return {
    id,
    type,
    component: options.component,
    props: options.props || {},
    parentId: options.parentId,
    title: options.title,
    icon: options.icon,
    metadata: options.metadata || {}
  }
}
