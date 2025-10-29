<template>
  <div
    class="flex flex-col h-full cursor-pointer relative bg-[#0D0D0D] panel"
    @click="handlePanelClick"
    @dragover="onDragOver"
    @dragenter="onDragEnter"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <!-- Active panel background overlay -->
    <div
      class="absolute inset-0 transition-opacity duration-200 pointer-events-none"
      :class="{
        'opacity-100 bg-linear-to-br from-[#141a20] to-[#0D0D0D]': isActive,
        'opacity-0 bg-[#0D0D0D]': !isActive,
      }"
    ></div>
    <!-- Active panel blue tint -->
    <div
      class="absolute inset-0 transition-opacity duration-200 pointer-events-none"
      :class="{
        'opacity-100 bg-blue-500/5': isActive,
        'opacity-0': !isActive,
      }"
    ></div>

    <!-- Drop Zones Overlay -->
    <DropZones
      :show-drop-zones="showDropZones"
      :panel-id="panel.id"
      @split-panel="onSplitPanel"
      @move-tab="onMoveTab"
    />

    <!-- Tab Bar -->
    <TabBar
      class="relative z-10"
      :panel="panel"
      :terminals="terminals"
      :is-active="isActive"
      @select-tab="selectTab"
      @close-tab="closeTab"
      @add-tab="addTab"
      @split-horizontal="splitHorizontal"
      @split-vertical="splitVertical"
      @close-panel="closePanel"
      @move-tab="moveTab"
      @duplicate-tab="duplicateTab"
      @move-tab-to-new-panel="moveTabToNewPanel"
    />

    <!-- Panel Content -->
    <div class="flex-1 overflow-hidden relative z-10">
      <!-- Terminal Manager -->
      <TerminalManager
        ref="terminalManagerRef"
        :terminals="activeTerminals"
        :active-terminal-id="panel.activeTabId"
        @terminal-ready="onTerminalReady"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  computed,
  ref,
  watch,
  nextTick,
  onMounted,
  onBeforeUnmount,
} from "vue";
import TabBar from "./TabBar.vue";
import TerminalManager from "./TerminalManager.vue";
import DropZones from "./DropZones.vue";
import type { Panel, TerminalInstance, Tab } from "../../types/panel";
import type { ComponentPublicInstance } from "vue";

interface TerminalManagerComponent extends ComponentPublicInstance {
  focusActiveTerminal: () => void;
}

interface PanelProps {
  panel: Panel;
  terminals: TerminalInstance[];
  isActive: boolean;
}

interface PanelEmits {
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
  terminalReady: [terminalId: string];
  setActivePanel: [panelId: string];
  duplicateTab: [panelId: string, tabId: string];
  moveTabToNewPanel: [panelId: string, tabId: string];
  splitPanelByDrop: [
    direction: "top" | "bottom" | "left" | "right",
    draggedTab: Tab,
    sourcePanelId: string,
    targetPanelId: string,
  ];
  cloneTabAndSplit: [
    direction: "top" | "bottom" | "left" | "right",
    tabId: string,
    panelId: string,
  ];
}

const props = defineProps<PanelProps>();

const emit = defineEmits<PanelEmits>();

const terminalManagerRef = ref<TerminalManagerComponent | null>(null);

const showDropZones = ref(false);
const dragEnterCounter = ref(0);
let hideDropZonesTimeout: ReturnType<typeof setTimeout> | null = null;

const activeTerminals = computed(() => {
  const tabIds = props.panel.tabs.map((tab) => tab.id);
  return props.terminals.filter((terminal) => tabIds.includes(terminal.id));
});

watch(
  () => props.isActive,
  async (isActive) => {
    if (isActive && terminalManagerRef.value) {
      await nextTick();
      setTimeout(() => {
        terminalManagerRef.value?.focusActiveTerminal();
      }, 50);
    }
  },
  { immediate: true },
);

const selectTab = (panelId: string, tabId: string): void => {
  emit("selectTab", panelId, tabId);
  emit("setActivePanel", panelId); // Also make this panel active
};

const closeTab = (panelId: string, tabId: string): void => {
  emit("closeTab", panelId, tabId);
};

const addTab = (panelId: string): void => {
  emit("addTab", panelId);
  emit("setActivePanel", panelId); // Make this panel active when adding tab
};

const splitHorizontal = (panelId: string): void => {
  emit("splitHorizontal", panelId);
};

const splitVertical = (panelId: string): void => {
  emit("splitVertical", panelId);
};

const closePanel = (panelId: string): void => {
  emit("closePanel", panelId);
};

const moveTab = (
  fromPanelId: string,
  toPanelId: string,
  tabId: string,
  targetTabId?: string,
): void => {
  emit("moveTab", fromPanelId, toPanelId, tabId, targetTabId);
};

const duplicateTab = (panelId: string, tabId: string): void => {
  emit("duplicateTab", panelId, tabId);
};

const moveTabToNewPanel = (panelId: string, tabId: string): void => {
  emit("moveTabToNewPanel", panelId, tabId);
};

const onTerminalReady = (terminalId: string): void => {
  emit("terminalReady", terminalId);
};

const handlePanelClick = (): void => {
  emit("setActivePanel", props.panel.id);
};

/**
 * Handle drag over event to show drop zones
 * @param {DragEvent} event - The drag event
 */
const onDragOver = (event: DragEvent): void => {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = "move";
  }
};

/**
 * Handle drag enter event
 * @param {DragEvent} event - The drag event
 */
const onDragEnter = (event: DragEvent): void => {
  event.preventDefault();

  if (
    event.dataTransfer &&
    event.dataTransfer.types.includes("application/json")
  ) {
    if (hideDropZonesTimeout) {
      clearTimeout(hideDropZonesTimeout);
      hideDropZonesTimeout = null;
    }

    dragEnterCounter.value++;
    if (dragEnterCounter.value === 1) {
      showDropZones.value = true;
    }
  }
};

/**
 * Handle drag leave event
 * @param {DragEvent} event - The drag event
 */
const onDragLeave = (event: DragEvent): void => {
  event.preventDefault();

  const panelElement = event.currentTarget as HTMLElement;
  const rect = panelElement.getBoundingClientRect();

  const isLeavingPanel =
    event.clientX < rect.left ||
    event.clientX > rect.right ||
    event.clientY < rect.top ||
    event.clientY > rect.bottom;

  if (isLeavingPanel) {
    dragEnterCounter.value--;
    if (dragEnterCounter.value <= 0) {
      dragEnterCounter.value = 0;
      showDropZones.value = false;
    }
  }
};

/**
 * Handle drop event on panel (fallback)
 * @param {DragEvent} event - The drag event
 */
const onDrop = (event: DragEvent): void => {
  event.preventDefault();
  showDropZones.value = false;
  dragEnterCounter.value = 0;
};

/**
 * Reset drop zones state
 */
const resetDropZones = (): void => {
  showDropZones.value = false;
  dragEnterCounter.value = 0;

  if (hideDropZonesTimeout) {
    clearTimeout(hideDropZonesTimeout);
    hideDropZonesTimeout = null;
  }
};

/**
 * Handle split panel from drop zones
 * @param {string} direction - The split direction
 * @param {Tab} draggedTab - The dragged tab
 * @param {string} sourcePanelId - The source panel ID
 */
const onSplitPanel = (
  direction: "top" | "bottom" | "left" | "right",
  draggedTab: Tab,
  sourcePanelId: string,
): void => {
  resetDropZones();

  if (sourcePanelId === props.panel.id) {
    emit("cloneTabAndSplit", direction, draggedTab.id, props.panel.id);
  } else {
    emit(
      "splitPanelByDrop",
      direction,
      draggedTab,
      sourcePanelId,
      props.panel.id,
    );
  }
};

/**
 * Handle move tab from drop zones
 * @param {Tab} draggedTab - The dragged tab
 * @param {string} sourcePanelId - The source panel ID
 */
const onMoveTab = (draggedTab: Tab, sourcePanelId: string): void => {
  resetDropZones();
  emit("moveTab", sourcePanelId, props.panel.id, draggedTab.id);
};

/**
 * Handle global drag end event to reset drop zones
 */
const onGlobalDragEnd = (): void => {
  hideDropZonesTimeout = setTimeout(() => {
    resetDropZones();
  }, 100);
};

/**
 * Handle ESC key to reset drop zones
 */
const onEscapeKey = (event: KeyboardEvent): void => {
  if (event.key === "Escape") {
    resetDropZones();
  }
};

onMounted(() => {
  document.addEventListener("dragend", onGlobalDragEnd);
  document.addEventListener("keydown", onEscapeKey);
});

onBeforeUnmount(() => {
  document.removeEventListener("dragend", onGlobalDragEnd);
  document.removeEventListener("keydown", onEscapeKey);
  if (hideDropZonesTimeout) {
    clearTimeout(hideDropZonesTimeout);
  }
});
</script>

<style scoped>
/* Panel entrance animation */
.panel-enter-active {
  transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.panel-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.6, 1);
}

.panel-enter-from {
  opacity: 0;
  transform: scale(0.95) translateY(-10px);
}

.panel-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(10px);
}

/* Active panel glow effect */
.panel:has(.opacity-100) {
  box-shadow: 0 0 20px rgba(59, 130, 246, 0.1);
  transition: box-shadow 0.3s ease;
}

/* Smooth background transitions */
.absolute.inset-0 {
  transition: all 0.3s ease;
}
</style>
