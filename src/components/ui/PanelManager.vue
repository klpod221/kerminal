<template>
  <div class="flex-1 h-full overflow-hidden">
    <div v-if="layout.type === 'panel'" class="h-full">
      <Panel
        :panel="layout.panel!"
        :terminals="terminals"
        :is-active="layout.panel!.id === activePanelId"
        @select-tab="selectTab"
        @close-tab="closeTab"
        @add-tab="addTab"
        @split-horizontal="splitHorizontal"
        @split-vertical="splitVertical"
        @close-panel="closePanel"
        @move-tab="moveTab"
        @duplicate-tab="duplicateTab"
        @move-tab-to-new-panel="moveTabToNewPanel"
        @terminal-ready="terminalReady"
        @set-active-panel="setActivePanel"
        @split-panel-by-drop="splitPanelByDrop"
        @clone-tab-and-split="cloneTabAndSplit"
        @add-tab-with-profile="addTabWithProfile"
      />
    </div>

    <div v-else-if="layout.type === 'split'" class="h-full">
      <Splitpanes
        :horizontal="props.layout.direction === 'horizontal'"
        class="default-theme"
        @resize="onPaneResize"
      >
        <Pane
          v-for="(child, index) in layout.children"
          :key="child.id"
          :size="getPaneSize(index)"
          :min-size="10"
        >
          <PanelManager
            :layout="child"
            :terminals="terminals"
            :active-panel-id="activePanelId"
            @select-tab="selectTab"
            @close-tab="closeTab"
            @add-tab="addTab"
            @split-horizontal="splitHorizontal"
            @split-vertical="splitVertical"
            @close-panel="closePanel"
            @move-tab="moveTab"
            @duplicate-tab="duplicateTab"
            @move-tab-to-new-panel="moveTabToNewPanel"
            @terminal-ready="terminalReady"
            @set-active-panel="setActivePanel"
            @layout-updated="layoutUpdated"
            @split-panel-by-drop="splitPanelByDrop"
            @clone-tab-and-split="cloneTabAndSplit"
            @add-tab-with-profile="addTabWithProfile"
          />
        </Pane>
      </Splitpanes>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Splitpanes, Pane } from "splitpanes";
import Panel from "./Panel.vue";
import { debounce } from "../../utils/helpers";
import type { PanelLayout, TerminalInstance, Tab } from "../../types/panel";

interface PanelManagerProps {
  layout: PanelLayout;
  terminals: TerminalInstance[];
  activePanelId: string;
}

type SplitDirection = "top" | "bottom" | "left" | "right";

interface PanelManagerEmits {
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
  layoutUpdated: [layout: PanelLayout];
  duplicateTab: [panelId: string, tabId: string];
  moveTabToNewPanel: [panelId: string, tabId: string];
  splitPanelByDrop: [
    direction: SplitDirection,
    draggedTab: Tab,
    sourcePanelId: string,
    targetPanelId: string,
  ];
  cloneTabAndSplit: [direction: SplitDirection, tabId: string, panelId: string];
  addTabWithProfile: [panelId: string, profile: any];
}

const props = defineProps<PanelManagerProps>();
const emit = defineEmits<PanelManagerEmits>();

const getPaneSize = (index: number): number => {
  const size =
    props.layout.sizes?.[index] || 1 / (props.layout.children?.length || 1);
  return Math.max(10, Math.min(90, size * 100)); // Clamp between 10% and 90%
};

const selectTab = (panelId: string, tabId: string): void => {
  emit("selectTab", panelId, tabId);
};

const closeTab = (panelId: string, tabId: string): void => {
  emit("closeTab", panelId, tabId);
};

const addTab = (panelId: string): void => {
  emit("addTab", panelId);
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

const terminalReady = (terminalId: string): void => {
  emit("terminalReady", terminalId);
};

const setActivePanel = (panelId: string): void => {
  emit("setActivePanel", panelId);
};

const duplicateTab = (panelId: string, tabId: string): void => {
  emit("duplicateTab", panelId, tabId);
};

const moveTabToNewPanel = (panelId: string, tabId: string): void => {
  emit("moveTabToNewPanel", panelId, tabId);
};

const layoutUpdated = (layout: PanelLayout): void => {
  emit("layoutUpdated", layout);
};

const splitPanelByDrop = (
  direction: "top" | "bottom" | "left" | "right",
  draggedTab: Tab,
  sourcePanelId: string,
  targetPanelId: string,
): void => {
  emit("splitPanelByDrop", direction, draggedTab, sourcePanelId, targetPanelId);
};

const cloneTabAndSplit = (
  direction: "top" | "bottom" | "left" | "right",
  tabId: string,
  panelId: string,
): void => {
  emit("cloneTabAndSplit", direction, tabId, panelId);
};

const handlePaneResize = (paneComponents: { size: number }[]): void => {
  if (
    !props.layout.children ||
    paneComponents.length !== props.layout.children.length
  ) {
    return;
  }

  const newSizes = paneComponents.map((pane) => pane.size / 100);

  const updatedLayout = { ...props.layout, sizes: newSizes };
  emit("layoutUpdated", updatedLayout);

  setTimeout(() => {
    globalThis.dispatchEvent(new Event("resize"));
  }, 50);
};

const onPaneResize = debounce(handlePaneResize, 150);

const addTabWithProfile = (panelId: string, profile: any): void => {
  emit("addTabWithProfile", panelId, profile);
};
</script>

<style scoped>
/* Import splitpanes CSS */
@import "splitpanes/dist/splitpanes.css";

/* Dark theme customizations */
:deep(.splitpanes__splitter) {
  background-color: #374151;
  border: none;
  position: relative;
}

:deep(.splitpanes__splitter:hover) {
  background-color: #4b5563;
}

:deep(.splitpanes__splitter:before) {
  content: "";
  position: absolute;
  left: 0;
  top: 0;
  right: 0;
  bottom: 0;
  background-color: #6b7280;
  opacity: 0;
  transition: opacity 0.2s;
}

:deep(.splitpanes__splitter:hover:before) {
  opacity: 1;
}

/* Adjust splitter size and cursor */
:deep(.splitpanes--vertical > .splitpanes__splitter) {
  width: 4px;
  cursor: col-resize;
}

:deep(.splitpanes--horizontal > .splitpanes__splitter) {
  height: 4px;
  cursor: row-resize;
}
</style>
