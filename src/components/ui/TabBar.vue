<template>
  <div
    class="flex items-center border-b border-gray-800 relative bg-bg-primary sm:h-[30px] sm:min-h-[30px] sm:max-h-[30px]"
    :class="
      isMobile ? 'h-9 min-h-9 max-h-9' : 'h-[30px] min-h-[30px] max-h-[30px]'
    "
    @dragover="onPanelDragOver"
    @drop="onPanelDrop"
    @dragenter="onPanelDragEnter"
    @dragleave="onPanelDragLeave"
  >
    <!-- Active panel TabBar overlay -->
    <div
      class="absolute inset-0 transition-opacity duration-200 pointer-events-none"
      :class="{
        'opacity-100 bg-linear-to-r from-blue-900/20 to-transparent': isActive,
        'opacity-0': !isActive,
      }"
    ></div>

    <!-- Drag drop indicator overlay -->
    <div
      class="absolute inset-0 transition-all duration-200 pointer-events-none z-20"
      :class="{
        'opacity-100 bg-blue-500/20 border-2 border-blue-500 border-dashed':
          isDragOver,
        'opacity-0': !isDragOver,
      }"
    ></div>

    <!-- Tabs Container -->
    <div
      class="flex items-center flex-1 h-full min-w-0 relative z-10 sm:max-h-[30px]"
      :class="isMobile ? 'max-h-9' : 'max-h-[30px]'"
    >
      <!-- Left scroll button -->
      <Button
        v-show="showScrollButtons && canScrollLeft && !isMobile"
        title="Scroll left"
        variant="ghost"
        size="sm"
        :icon="ChevronLeft"
        class="scroll-btn shrink-0 z-20"
        @click="scrollLeft"
      />

      <!-- Scrollable tabs container -->
      <div
        ref="tabsContainer"
        class="flex items-center h-full overflow-hidden flex-1 scrollable-tabs sm:max-h-[30px]"
        :class="isMobile ? 'max-h-9' : 'max-h-[30px]'"
        @wheel.prevent="onWheel"
      >
        <div
          ref="tabsContent"
          class="flex items-center h-full transition-transform duration-200 ease-out sm:max-h-[30px]"
          :class="isMobile ? 'max-h-9' : 'max-h-[30px]'"
          :style="{ transform: `translateX(${scrollOffset}px)` }"
        >
          <transition-group
            name="tab"
            tag="div"
            class="flex items-center h-full sm:max-h-[30px]"
            :class="isMobile ? 'max-h-9' : 'max-h-[30px]'"
            appear
          >
            <Tab
              v-for="tab in panel.tabs"
              :key="tab.id"
              :ref="(el) => setTabRef(tab.id, el)"
              :tab="tab"
              :panel-id="panel.id"
              :is-active="tab.id === panel.activeTabId"
              :is-connecting="getTerminalConnectingState(tab.id)"
              :is-connected="getTerminalConnectedState(tab.id)"
              :is-error="getTerminalErrorState(tab.id)"
              :disconnect-reason="getTerminalDisconnectReason(tab.id)"
              :backend-terminal-id="getBackendTerminalId(tab.id)"
              :latency="getTerminalLatency(tab.id)"
              :min-width="tabMinWidth"
              :max-width="tabMaxWidth"
              @select="selectTab(tab.id)"
              @close="closeTab(tab.id)"
              @duplicate="handleTabDuplicate"
              @close-others="handleCloseOthers"
              @close-to-right="handleCloseToRight"
              @move-to-new-panel="handleMoveToNewPanel"
              @drag-start="onTabDragStart"
              @drop="onTabDrop"
            />
          </transition-group>

          <!-- Add Tab Button - Inside scrollable area when not scrolling -->
          <Transition name="fade">
            <Button
              v-show="!showScrollButtons"
              title="Add new tab (Right-click for profiles)"
              variant="ghost"
              size="sm"
              :icon="Plus"
              class="add-tab-btn shrink-0 ml-1"
              @click="addTab"
              @contextmenu="onAddTabContextMenu"
            />
          </Transition>
        </div>
      </div>

      <!-- Recording Controls -->
      <div
        v-if="activeBackendTerminalId"
        class="shrink-0 flex items-center px-1"
      >
        <RecordingControls :terminal-id="activeBackendTerminalId" />
      </div>

      <!-- Right scroll button -->
      <Button
        v-show="showScrollButtons && canScrollRight && !isMobile"
        title="Scroll right"
        variant="ghost"
        size="sm"
        :icon="ChevronRight"
        class="scroll-btn shrink-0 z-20"
        @click="scrollRight"
      />

      <!-- Add Tab Button - Outside when scrolling is needed -->
      <Transition name="fade">
        <Button
          v-show="showScrollButtons && !isMobile"
          title="Add new tab (Right-click for profiles)"
          variant="ghost"
          size="sm"
          :icon="Plus"
          class="add-tab-btn shrink-0"
          @click="addTab"
          @contextmenu="onAddTabContextMenu"
        />
      </Transition>
    </div>

    <!-- Panel Controls -->
    <div
      class="flex items-center h-full shrink-0 relative z-10 sm:max-h-[30px]"
      :class="isMobile ? 'max-h-9' : 'max-h-[30px]'"
    >
      <!-- Split Horizontal Button - Hide on mobile -->
      <Button
        v-if="!isMobile"
        title="Split horizontal"
        variant="ghost"
        size="sm"
        :icon="SplitSquareHorizontal"
        @click="splitHorizontal"
      />

      <!-- Split Vertical Button - Hide on mobile -->
      <Button
        v-if="!isMobile"
        title="Split vertical"
        variant="ghost"
        size="sm"
        :icon="SplitSquareVertical"
        @click="splitVertical"
      />

      <!-- Add Tab Button on mobile -->
      <Button
        v-if="isMobile"
        title="Add new tab"
        variant="ghost"
        size="sm"
        :icon="Plus"
        class="add-tab-btn"
        @click="addTab"
        @contextmenu="onAddTabContextMenu"
      />

      <!-- Close Panel Button -->
      <Button
        title="Close panel"
        variant="ghost"
        size="sm"
        :icon="X"
        class="hover:bg-red-600/20 hover:text-red-400"
        @click="closePanel"
      />
    </div>

    <!-- Add Tab Context Menu -->
    <ContextMenu
      ref="addTabContextMenuRef"
      :items="addTabContextMenuItems"
      @item-click="handleAddTabContextMenuAction"
    />
  </div>
</template>

<script setup lang="ts">
import {
  computed,
  ref,
  nextTick,
  watch,
  onMounted,
  onBeforeUnmount,
} from "vue";
import {
  Plus,
  SplitSquareHorizontal,
  SplitSquareVertical,
  X,
  ChevronLeft,
  ChevronRight,
  Terminal,
} from "lucide-vue-next";
import Tab from "./Tab.vue";
import Button from "./Button.vue";
import RecordingControls from "../recording/RecordingControls.vue";
import { useWindowSize } from "../../composables/useWindowSize";
import { safeJsonParse } from "../../utils/helpers";
import type {
  Tab as TabType,
  Panel,
  TerminalInstance,
} from "../../types/panel";
import ContextMenu, { type ContextMenuItem } from "./ContextMenu.vue";

interface TabBarProps {
  panel: Panel;
  isActive: boolean;
  terminals?: TerminalInstance[];
}

interface TabBarEmits {
  selectTab: [panelId: string, tabId: string];
  closeTab: [panelId: string, tabId: string];
  addTab: [panelId: string];
  splitHorizontal: [panelId: string];
  splitVertical: [panelId: string];
  closePanel: [panelId: string];
  moveTab: [
    fromPanelId: string,
    toPanelId: string,
    tabId: string,
    targetTabId?: string,
  ];
  duplicateTab: [panelId: string, tabId: string];
  moveTabToNewPanel: [panelId: string, tabId: string];
  addTabWithProfile: [panelId: string, profile: any];
}

const props = withDefaults(defineProps<TabBarProps>(), {
  terminals: () => [],
});

const emit = defineEmits<TabBarEmits>();
const { width: windowWidth, isMobile } = useWindowSize();

const tabsContainer = ref<HTMLElement | null>(null);
const tabsContent = ref<HTMLElement | null>(null);
const tabRefs = ref<Record<string, HTMLElement>>({});
const scrollOffset = ref(0);
const maxScrollOffset = ref(0);

const isDragOver = ref(false);
const dragEnterCounter = ref(0);

const canScrollLeft = computed(() => scrollOffset.value < 0);
const canScrollRight = computed(
  () => scrollOffset.value > -maxScrollOffset.value,
);
const showScrollButtons = computed(() => maxScrollOffset.value > 0);

/**
 * Get backend terminal ID for a tab
 */
const getBackendTerminalId = (tabId: string): string | undefined => {
  const terminal = props.terminals.find((t) => t.id === tabId);
  return terminal?.backendTerminalId;
};

/**
 * Get backend terminal ID for the active tab
 */
const activeBackendTerminalId = computed(() => {
  if (!props.panel.activeTabId) return null;
  return getBackendTerminalId(props.panel.activeTabId);
});

/**
 * Set the ref for a tab element.
 * @param {string} tabId - The tab id.
 * @param {unknown} el - The ref value.
 */
const setTabRef = (tabId: string, el: unknown): void => {
  if (el && typeof el === "object" && "$el" in el) {
    tabRefs.value[tabId] = (el as { $el: HTMLElement }).$el;
  } else if (el && el instanceof HTMLElement) {
    tabRefs.value[tabId] = el;
  } else {
    delete tabRefs.value[tabId];
  }
};

/**
 * Calculate the maximum scroll offset based on content width.
 */
const updateScrollLimits = (): void => {
  if (!tabsContainer.value || !tabsContent.value) return;

  const containerWidth = tabsContainer.value.offsetWidth;
  const contentWidth = tabsContent.value.scrollWidth;

  const newMaxScrollOffset = Math.max(0, contentWidth - containerWidth);

  if (Math.abs(maxScrollOffset.value - newMaxScrollOffset) > 1) {
    maxScrollOffset.value = newMaxScrollOffset;

    if (scrollOffset.value < -maxScrollOffset.value) {
      scrollOffset.value = -maxScrollOffset.value;
    }
  }
};

/**
 * Scroll tabs to the left.
 */
const scrollLeft = (): void => {
  const scrollAmount = Math.min(200, Math.abs(scrollOffset.value));
  scrollOffset.value = Math.min(0, scrollOffset.value + scrollAmount);
};

/**
 * Scroll tabs to the right.
 */
const scrollRight = (): void => {
  const remainingScroll = maxScrollOffset.value + scrollOffset.value;
  const scrollAmount = Math.min(200, remainingScroll);
  scrollOffset.value = Math.max(
    -maxScrollOffset.value,
    scrollOffset.value - scrollAmount,
  );
};

/**
 * Handle mouse wheel scrolling on tabs.
 * @param {WheelEvent} event - The wheel event.
 */
const onWheel = (event: WheelEvent): void => {
  if (maxScrollOffset.value === 0) return;

  const delta = event.deltaX !== 0 ? event.deltaX : event.deltaY;
  const scrollAmount = delta > 0 ? -120 : 120;
  const newOffset = scrollOffset.value + scrollAmount;

  scrollOffset.value = Math.max(-maxScrollOffset.value, Math.min(0, newOffset));
};

/**
 * Scroll active tab into view if it's not visible.
 */
const scrollActiveTabIntoView = (): void => {
  const activeTab = tabRefs.value[props.panel.activeTabId];
  if (!activeTab || !tabsContainer.value || !tabsContent.value) return;

  const containerRect = tabsContainer.value.getBoundingClientRect();
  const tabRect = activeTab.getBoundingClientRect();
  const contentRect = tabsContent.value.getBoundingClientRect();

  const tabRelativeLeft = tabRect.left - contentRect.left;
  const tabRelativeRight = tabRelativeLeft + tabRect.width;

  const visibleLeft = -scrollOffset.value;
  const visibleRight = visibleLeft + containerRect.width;

  const padding = 20;

  if (tabRelativeLeft < visibleLeft) {
    scrollOffset.value = -(tabRelativeLeft - padding);
  } else if (tabRelativeRight > visibleRight) {
    scrollOffset.value = -(tabRelativeRight - containerRect.width + padding);
  }

  scrollOffset.value = Math.max(
    -maxScrollOffset.value,
    Math.min(0, scrollOffset.value),
  );
};

const debouncedScrollIntoView = (() => {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  return () => {
    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = setTimeout(() => {
      updateScrollLimits();
      scrollActiveTabIntoView();
    }, 100);
  };
})();

watch(
  () => props.panel.activeTabId,
  () => {
    nextTick(() => {
      debouncedScrollIntoView();
    });
  },
);

const debouncedUpdateLimits = (() => {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  return () => {
    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = setTimeout(() => {
      updateScrollLimits();
    }, 150);
  };
})();

watch(
  () => props.panel.tabs.length,
  () => {
    nextTick(() => {
      debouncedUpdateLimits();
    });
  },
);

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  if (tabsContainer.value) {
    resizeObserver = new ResizeObserver(() => {
      updateScrollLimits();
    });
    resizeObserver.observe(tabsContainer.value);
  }

  nextTick(() => {
    updateScrollLimits();
  });
});

onBeforeUnmount(() => {
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
});

const tabMinWidth = computed(() => {
  const tabCount = props.panel.tabs.length;
  const addButtonWidth = showScrollButtons.value ? 32 : 36; // Add button width (larger when inside scrollable area)
  const scrollButtonsWidth = showScrollButtons.value ? 64 : 0; // Scroll buttons
  const panelControlsWidth = 128; // Split + close buttons
  const padding = 16;
  const availableWidth =
    windowWidth.value -
    addButtonWidth -
    scrollButtonsWidth -
    panelControlsWidth -
    padding;

  const idealTabWidth = Math.floor(availableWidth / Math.max(tabCount, 1));

  if (tabCount <= 4 && idealTabWidth >= 180) return 180;
  if (tabCount <= 6 && idealTabWidth >= 150) return 150;
  if (tabCount <= 8 && idealTabWidth >= 120) return 120;
  if (tabCount <= 10 && idealTabWidth >= 100) return 100;

  return Math.max(idealTabWidth, 80); // Minimum readable space for scrollable tabs
});

const tabMaxWidth = computed(() => {
  const tabCount = props.panel.tabs.length;
  if (tabCount <= 4) return 180;
  return 200;
});

const getTerminalConnectingState = (tabId: string): boolean => {
  const terminal = props.terminals.find((t) => t.id === tabId);
  return terminal?.isSSHConnecting || false;
};

const getTerminalConnectedState = (tabId: string): boolean => {
  const terminal = props.terminals.find((t) => t.id === tabId);
  return terminal?.isConnected || false;
};

const getTerminalErrorState = (tabId: string): boolean => {
  const terminal = props.terminals.find((t) => t.id === tabId);
  return terminal?.hasError || false;
};

const getTerminalDisconnectReason = (tabId: string): string | undefined => {
  const terminal = props.terminals.find((t) => t.id === tabId);
  return terminal?.disconnectReason;
};

const getTerminalLatency = (tabId: string): number | undefined => {
  const terminal = props.terminals.find((t) => t.id === tabId);
  return terminal?.latency;
};

const selectTab = (tabId: string): void => {
  emit("selectTab", props.panel.id, tabId);
};

const closeTab = (tabId: string): void => {
  emit("closeTab", props.panel.id, tabId);
};

const addTab = (): void => {
  emit("addTab", props.panel.id);
};

const splitHorizontal = (): void => {
  emit("splitHorizontal", props.panel.id);
};

const splitVertical = (): void => {
  emit("splitVertical", props.panel.id);
};

const closePanel = (): void => {
  emit("closePanel", props.panel.id);
};

/**
 * Handle tab duplication from context menu
 */
const handleTabDuplicate = (tab: TabType): void => {
  emit("duplicateTab", props.panel.id, tab.id);
};

/**
 * Handle close other tabs from context menu
 */
const handleCloseOthers = (tab: TabType): void => {
  const otherTabs = props.panel.tabs.filter((t) => t.id !== tab.id);
  otherTabs.forEach((otherTab) => {
    emit("closeTab", props.panel.id, otherTab.id);
  });
};

/**
 * Handle close tabs to the right from context menu
 */
const handleCloseToRight = (tab: TabType): void => {
  const tabIndex = props.panel.tabs.findIndex((t) => t.id === tab.id);
  if (tabIndex !== -1) {
    const tabsToClose = props.panel.tabs.slice(tabIndex + 1);
    tabsToClose.forEach((tabToClose) => {
      emit("closeTab", props.panel.id, tabToClose.id);
    });
  }
};

/**
 * Handle move tab to new panel from context menu
 */
const handleMoveToNewPanel = (tab: TabType): void => {
  emit("moveTabToNewPanel", props.panel.id, tab.id);
};

const onTabDragStart = (tab: TabType): void => {
  console.log("onTabDragStart", tab);
};

const onTabDrop = (draggedTab: TabType, targetTab: TabType): void => {
  emit("moveTab", props.panel.id, props.panel.id, draggedTab.id, targetTab.id);
};

const onPanelDragOver = (event: DragEvent): void => {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = "move";
  }
};

const onPanelDragEnter = (event: DragEvent): void => {
  event.preventDefault();
  dragEnterCounter.value++;
  if (dragEnterCounter.value === 1) {
    isDragOver.value = true;
  }
};

const onPanelDragLeave = (event: DragEvent): void => {
  event.preventDefault();
  dragEnterCounter.value--;
  if (dragEnterCounter.value === 0) {
    isDragOver.value = false;
  }
};

const onPanelDrop = (event: DragEvent): void => {
  event.preventDefault();
  dragEnterCounter.value = 0;
  isDragOver.value = false;

  if (event.dataTransfer) {
    const draggedTabData = event.dataTransfer.getData("application/json");
    if (draggedTabData) {
      const dragData = safeJsonParse<{
        tab: TabType;
        sourcePanelId: string;
      } | null>(draggedTabData, null);
      if (!dragData) return;

      const draggedTab = dragData.tab;
      const sourcePanelId = dragData.sourcePanelId;

      if (sourcePanelId && sourcePanelId !== props.panel.id) {
        emit("moveTab", sourcePanelId, props.panel.id, draggedTab.id, "");
      }
    }
  }
};

// Terminal Profiles Context Menu
import { useTerminalProfileStore } from "../../stores/terminalProfile";
import { useOverlay } from "../../composables/useOverlay";
import { Settings } from "lucide-vue-next";

const terminalProfileStore = useTerminalProfileStore();
const { openOverlay } = useOverlay();
const addTabContextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null);

const addTabContextMenuItems = computed<ContextMenuItem[]>(() => {
  const items: ContextMenuItem[] = [
    {
      id: "default",
      label: "Default Terminal",
      icon: Plus,
      action: "default",
    },
    {
      id: "divider-profiles",
      type: "divider",
    },
  ];

  if (terminalProfileStore.profiles.length > 0) {
    terminalProfileStore.profiles.forEach((profile) => {
      items.push({
        id: `profile-${profile.id}`,
        label: profile.name,
        icon: Terminal, // You might need to import Terminal icon if not already imported
        action: "profile",
        // We can pass the profile object as a custom property if we extend the type,
        // but for now we'll use the ID to lookup
      });
    });

    items.push({
      id: "divider-manage",
      type: "divider",
    });
  }

  items.push({
    id: "manage-profiles",
    label: "Manage Profiles",
    icon: Settings,
    action: "manage",
  });

  return items;
});

const onAddTabContextMenu = (event: MouseEvent) => {
  event.preventDefault();
  if (addTabContextMenuRef.value) {
    addTabContextMenuRef.value.show(event.clientX, event.clientY);
  }
};

const handleAddTabContextMenuAction = (item: ContextMenuItem) => {
  if (item.action === "default") {
    addTab();
  } else if (item.action === "manage") {
    openOverlay("terminal-profile-modal");
  } else if (item.action === "profile") {
    const profileId = item.id.replace("profile-", "");
    const profile = terminalProfileStore.getProfile(profileId);
    if (profile) {
      emit("addTabWithProfile", props.panel.id, profile);
    }
  }
};
</script>

<style scoped>
/* Drag and drop styles */
.drag-over-indicator {
  position: absolute;
  inset: 0;
  background: linear-gradient(
    135deg,
    rgba(59, 130, 246, 0.1) 0%,
    rgba(99, 102, 241, 0.1) 100%
  );
  border: 2px dashed rgba(59, 130, 246, 0.5);
  border-radius: 4px;
  z-index: 20;
  pointer-events: none;
  transition: all 0.2s ease;
}

.drag-over-indicator::before {
  content: "Drop tab here";
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  color: rgba(59, 130, 246, 0.8);
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  background: rgba(13, 13, 13, 0.8);
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid rgba(59, 130, 246, 0.3);
}

/* Scroll button styles */
.scroll-btn {
  width: 24px;
  height: 24px;
  padding: 0;
  min-width: 0;
  opacity: 0.7;
  transition: opacity 0.2s ease;
}

.scroll-btn:hover {
  opacity: 1;
}

.scrollable-tabs {
  position: relative;
  overflow: hidden;
}

.scrollable-tabs::before,
.scrollable-tabs::after {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  width: 8px;
  pointer-events: none;
  z-index: 10;
  opacity: 0;
  transition: opacity 0.3s ease;
}

.scrollable-tabs::before {
  left: 0;
  background: linear-gradient(to right, rgba(13, 13, 13, 1), transparent);
}

.scrollable-tabs::after {
  right: 0;
  background: linear-gradient(to left, rgba(13, 13, 13, 1), transparent);
}

/* Show fade gradients when scrollable */
.scrollable-tabs:hover::before {
  opacity: 1;
}

.scrollable-tabs:hover::after {
  opacity: 1;
}

/* Tab transition animations */
.tab-enter-active {
  transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.tab-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.6, 1);
}

.tab-enter-from {
  opacity: 0;
  transform: scaleX(0.3) translateX(-30px);
  transform-origin: left center;
  max-width: 0;
}

.tab-enter-to {
  opacity: 1;
  transform: scaleX(1) translateX(0);
  transform-origin: left center;
}

.tab-leave-from {
  opacity: 1;
  transform: scaleX(1) translateX(0);
  transform-origin: right center;
}

.tab-leave-to {
  opacity: 0;
  transform: scaleX(0.3) translateX(30px);
  transform-origin: right center;
  max-width: 0;
}

.tab-move {
  transition: transform 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

/* Stagger effect for multiple tabs */
.tab-enter-active:nth-child(2) {
  transition-delay: 0.05s;
}

.tab-enter-active:nth-child(3) {
  transition-delay: 0.1s;
}

.tab-enter-active:nth-child(4) {
  transition-delay: 0.15s;
}

.tab-enter-active:nth-child(n + 5) {
  transition-delay: 0.2s;
}

/* Add Tab Button Animation */
:deep(.add-tab-btn) {
  position: relative;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

:deep(.add-tab-btn):hover {
  transform: scale(1.1);
}

:deep(.add-tab-btn):active {
  transform: scale(0.95);
}

:deep(.add-tab-btn):hover::before {
  content: "";
  position: absolute;
  top: 50%;
  left: 50%;
  width: 0;
  height: 0;
  background: radial-gradient(
    circle,
    rgba(59, 130, 246, 0.3) 0%,
    transparent 70%
  );
  border-radius: 50%;
  transform: translate(-50%, -50%);
  animation: ripple 0.6s ease-out;
}

/* Add fade transition for add button position change */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease-in-out;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.add-tab-btn {
  opacity: 1;
  transition: opacity 0.2s ease-in-out;
}

.add-tab-btn.v-enter-active,
.add-tab-btn.v-leave-active {
  transition: opacity 0.2s ease-in-out;
}

.add-tab-btn.v-enter-from,
.add-tab-btn.v-leave-to {
  opacity: 0;
}

@keyframes ripple {
  to {
    width: 40px;
    height: 40px;
  }
}
</style>
