<template>
  <PanelManager
    :layout="panelLayout"
    :terminals="terminals"
    :window-width="windowWidth"
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
    @layout-updated="updateLayout"
  />
</template>

<script setup lang="ts">
import { ref, nextTick } from "vue";
import PanelManager from "./ui/PanelManager.vue";
import { debounce } from "../utils/helpers";
import type { PanelLayout, TerminalInstance, Panel, Tab } from "../types/panel";
import { useWindowSize } from "../composables/useWindowSize";
import { useViewStateStore } from "../stores/viewState";

import { createTerminal } from "../services/terminal";

// Store and Composables
const viewState = useViewStateStore();
const { width: windowWidth } = useWindowSize();

// State variables
const panelLayout = ref<PanelLayout>({
  type: "panel",
  id: "panel-1",
  panel: {
    id: "panel-1",
    activeTabId: "",
    tabs: [],
  },
});
const activePanelId = ref<string>("panel-1");
const terminals = ref<TerminalInstance[]>([]);

// Counter variables
let tabCounter = 1;
let panelCounter = 2; // Start from 2 since panel-1 is already created

// Methods to manage panels and tabs
const findPanelInLayout = (
  layout: PanelLayout,
  panelId: string
): Panel | null => {
  if (layout.type === "panel" && layout.panel?.id === panelId) {
    return layout.panel;
  }
  if (layout.type === "split" && layout.children) {
    for (const child of layout.children) {
      const found = findPanelInLayout(child, panelId);
      if (found) return found;
    }
  }
  return null;
};

const removePanelFromLayout = (
  layout: PanelLayout,
  panelId: string
): PanelLayout | null => {
  if (layout.type === "panel" && layout.panel?.id === panelId) {
    return null; // This panel should be removed
  }

  if (layout.type === "split" && layout.children) {
    const filteredChildren = layout.children
      .map((child) => removePanelFromLayout(child, panelId))
      .filter((child) => child !== null) as PanelLayout[];

    if (filteredChildren.length === 0) {
      return null;
    }

    if (filteredChildren.length === 1) {
      // Collapse split with only one child
      return filteredChildren[0];
    }

    // Update sizes proportionally when a child is removed
    const newSizes = layout.sizes
      ? filteredChildren.map(() => 1 / filteredChildren.length)
      : undefined;

    return {
      ...layout,
      children: filteredChildren,
      sizes: newSizes,
    };
  }

  return layout;
};

const findFirstPanel = (layout: PanelLayout): Panel | null => {
  if (layout.type === "panel") {
    return layout.panel || null;
  }
  if (layout.type === "split" && layout.children) {
    for (const child of layout.children) {
      const found = findFirstPanel(child);
      if (found) return found;
    }
  }
  return null;
};

// Auto close panel when it has no tabs left
const autoClosePanel = (panelId: string): void => {
  // Remove panel from layout
  const newLayout = removePanelFromLayout(panelLayout.value, panelId);
  if (newLayout) {
    panelLayout.value = newLayout;
    // Find a new active panel if the closed panel was active
    if (activePanelId.value === panelId) {
      const firstPanel = findFirstPanel(panelLayout.value);
      if (firstPanel) {
        activePanelId.value = firstPanel.id;
      } else {
        // No panels left, show dashboard
        viewState.setActiveView("dashboard");
      }
    }
  } else {
    // All panels closed, show dashboard
    viewState.setActiveView("dashboard");
  }
};

const selectTab = (panelId: string, tabId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId);
  if (panel) {
    panel.activeTabId = tabId;
    activePanelId.value = panelId;
    viewState.setActiveView("workspace");
  }
};

const addTab = (panelId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId);
  if (!panel) return;

  const newTabId = tabCounter.toString();
  const newTab: Tab = {
    id: newTabId,
    title: "Terminal", // Default title, will be updated by terminal process
  };

  const newTerminal: TerminalInstance = {
    id: newTabId,
    ready: false,
  };

  // Add tab to panel
  panel.tabs.push(newTab);
  panel.activeTabId = newTabId;

  // Add terminal instance
  terminals.value.push(newTerminal);

  // Set active panel and switch to workspace
  activePanelId.value = panelId;
  viewState.setActiveView("workspace");

  // Request new terminal from main process
  createTerminal(newTabId);

  tabCounter++;
};

const closeTab = (panelId: string, tabId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId);
  if (!panel) return;

  const tabIndex = panel.tabs.findIndex((tab) => tab.id === tabId);
  const terminalIndex = terminals.value.findIndex(
    (terminal) => terminal.id === tabId
  );

  if (tabIndex !== -1) {
    // Remove terminal instance
    if (terminalIndex !== -1) {
      // TODO: Request terminal destruction from main process
      // window.api?.send('terminal.destroy', { terminalId: tabId })
      terminals.value.splice(terminalIndex, 1);

      // TODO: Clear local buffer for this terminal as it's being permanently closed
      // bufferManager.clearLocalBuffer(tabId)
    }

    // Remove the tab
    const wasActive = panel.activeTabId === tabId;
    panel.tabs.splice(tabIndex, 1);

    // AUTO-CLOSE PANEL: If this was the last tab in the panel, auto-close the panel
    if (panel.tabs.length === 0) {
      autoClosePanel(panelId);
      return;
    }

    // If closed tab was active, activate another tab
    if (wasActive) {
      const newActiveIndex = Math.min(tabIndex, panel.tabs.length - 1);
      panel.activeTabId = panel.tabs[newActiveIndex].id;
    }
  }
};

const splitPanelInLayout = (
  layout: PanelLayout,
  panelId: string,
  newPanel: Panel,
  direction: "horizontal" | "vertical"
): boolean => {
  if (layout.type === "panel" && layout.panel?.id === panelId) {
    // This is the panel want to split
    // We need to replace this layout with a split layout
    const originalPanel = layout.panel;

    // Update layout properties in-place for better reactivity
    layout.type = "split";
    layout.direction = direction;
    layout.children = [
      {
        type: "panel",
        id: originalPanel.id,
        panel: originalPanel,
      },
      {
        type: "panel",
        id: newPanel.id,
        panel: newPanel,
      },
    ];
    layout.sizes = [0.5, 0.5];

    // Clear panel property since we're now a split
    delete (layout as PanelLayout & { panel?: Panel }).panel;

    return true;
  }

  if (layout.type === "split" && layout.children) {
    for (const child of layout.children) {
      if (splitPanelInLayout(child, panelId, newPanel, direction)) {
        return true;
      }
    }
  }

  return false;
};

const splitVertical = (panelId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId);
  if (!panel) return;

  // Clone current active tab or create default tab
  let newTab: Tab;
  if (panel.activeTabId && panel.tabs.length > 0) {
    const activeTab = panel.tabs.find((tab) => tab.id === panel.activeTabId);
    if (activeTab) {
      const newTabId = tabCounter.toString();
      newTab = {
        id: newTabId,
        title: activeTab.title,
        color: activeTab.color,
        profileId: activeTab.profileId,
        groupId: activeTab.groupId,
      };
      // Create terminal for cloned tab
      const newTerminal: TerminalInstance = { id: newTabId, ready: false };
      terminals.value.push(newTerminal);

      if (activeTab.profileId) {
        // TODO: Clone SSH connection
        // window.api?.send('terminal.createSSH', {
        //   terminalId: newTabId,
        //   profileId: activeTab.profileId
        // })
      } else {
        // TODO: Clone regular terminal
        // window.api?.send('terminal.create', { terminalId: newTabId })
      }
    } else {
      newTab = { id: tabCounter.toString(), title: "Terminal" };
    }
  } else {
    newTab = { id: tabCounter.toString(), title: "Terminal" };
  }

  // Create new panel
  const newPanelId = `panel-${panelCounter++}`;
  const newPanel: Panel = {
    id: newPanelId,
    activeTabId: newTab.id,
    tabs: [newTab],
  };

  // Split the specific panel in the layout
  splitPanelInLayout(panelLayout.value, panelId, newPanel, "horizontal");
  activePanelId.value = newPanelId;
  tabCounter++;
};

const splitHorizontal = (panelId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId);
  if (!panel) return;

  // Clone current active tab or create default tab
  let newTab: Tab;
  if (panel.activeTabId && panel.tabs.length > 0) {
    const activeTab = panel.tabs.find((tab) => tab.id === panel.activeTabId);
    if (activeTab) {
      const newTabId = tabCounter.toString();
      newTab = {
        id: newTabId,
        title: activeTab.title,
        color: activeTab.color,
        profileId: activeTab.profileId,
        groupId: activeTab.groupId,
      };
      // Create terminal for cloned tab
      const newTerminal: TerminalInstance = { id: newTabId, ready: false };
      terminals.value.push(newTerminal);

      if (activeTab.profileId) {
        // TODO: Clone SSH connection
        // window.api?.send('terminal.createSSH', {
        //   terminalId: newTabId,
        //   profileId: activeTab.profileId
        // })
      } else {
        // TODO: Clone regular terminal
        // window.api?.send('terminal.create', { terminalId: newTabId })
      }
    } else {
      newTab = { id: tabCounter.toString(), title: "Terminal" };
    }
  } else {
    newTab = { id: tabCounter.toString(), title: "Terminal" };
  }

  // Create new panel
  const newPanelId = `panel-${panelCounter++}`;
  const newPanel: Panel = {
    id: newPanelId,
    activeTabId: newTab.id,
    tabs: [newTab],
  };

  // Split the specific panel in the layout
  splitPanelInLayout(panelLayout.value, panelId, newPanel, "vertical");
  activePanelId.value = newPanelId;
  tabCounter++;
};

const closePanel = (panelId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId);
  if (!panel) return;

  // Close all tabs in the panel (this will also destroy their terminals)
  const tabIds = [...panel.tabs.map((tab) => tab.id)]; // Create a copy to avoid mutation during iteration

  for (const tabId of tabIds) {
    const terminalIndex = terminals.value.findIndex(
      (terminal) => terminal.id === tabId
    );
    if (terminalIndex !== -1) {
      // TODO: Request terminal destruction from main process
      // window.api?.send('terminal.destroy', { terminalId: tabId })
      terminals.value.splice(terminalIndex, 1);
    }
  }

  // Remove panel from layout
  const newLayout = removePanelFromLayout(panelLayout.value, panelId);
  if (newLayout) {
    panelLayout.value = newLayout;
    // Find a new active panel if the closed panel was active
    if (activePanelId.value === panelId) {
      const firstPanel = findFirstPanel(panelLayout.value);
      if (firstPanel) {
        activePanelId.value = firstPanel.id;
      } else {
        // No panels left, show dashboard
        viewState.setActiveView("dashboard");
      }
    }
  } else {
    // All panels closed, show dashboard
    viewState.setActiveView("dashboard");
  }
};

const findPanelContainingTab = (
  layout: PanelLayout,
  tabId: string
): Panel | null => {
  if (layout.type === "panel" && layout.panel) {
    const hasTab = layout.panel.tabs.some((tab) => tab.id === tabId);
    if (hasTab) return layout.panel;
  }
  if (layout.type === "split" && layout.children) {
    for (const child of layout.children) {
      const found = findPanelContainingTab(child, tabId);
      if (found) return found;
    }
  }
  return null;
};

function insertTabToPanel(panel: Panel, tab: Tab, targetTabId?: string): void {
  if (targetTabId) {
    const targetIndex = panel.tabs.findIndex((t) => t.id === targetTabId);
    if (targetIndex !== -1) {
      panel.tabs.splice(targetIndex, 0, tab);
      return;
    }
  }
  panel.tabs.push(tab);
}

function reorderTabWithinPanel(
  panelId: string,
  tabId: string,
  targetTabId?: string
): void {
  const panel = findPanelInLayout(panelLayout.value, panelId);
  if (!panel || !targetTabId) return;

  const draggedIndex = panel.tabs.findIndex((tab) => tab.id === tabId);
  const targetIndex = panel.tabs.findIndex((tab) => tab.id === targetTabId);

  if (draggedIndex === -1 || targetIndex === -1) return;

  const [draggedTab] = panel.tabs.splice(draggedIndex, 1);
  panel.tabs.splice(targetIndex, 0, draggedTab);
}

function updateActiveTabsAfterMove(
  fromPanel: Panel,
  toPanel: Panel,
  tabId: string,
  tabIndex: number
): void {
  if (fromPanel.activeTabId === tabId) {
    if (fromPanel.tabs.length > 0) {
      fromPanel.activeTabId =
        fromPanel.tabs[Math.min(tabIndex, fromPanel.tabs.length - 1)].id;
    } else {
      fromPanel.activeTabId = "";
    }
  }
  toPanel.activeTabId = tabId;
  activePanelId.value = toPanel.id;
}

function moveTabBetweenPanels(
  fromPanelId: string,
  toPanelId: string,
  tabId: string,
  targetTabId?: string
): void {
  const fromPanel = findPanelInLayout(panelLayout.value, fromPanelId);
  const toPanel = findPanelInLayout(panelLayout.value, toPanelId);

  if (!fromPanel || !toPanel) return;

  const tabIndex = fromPanel.tabs.findIndex((tab) => tab.id === tabId);
  if (tabIndex === -1) return;

  const [tab] = fromPanel.tabs.splice(tabIndex, 1);

  insertTabToPanel(toPanel, tab, targetTabId);

  updateActiveTabsAfterMove(fromPanel, toPanel, tabId, tabIndex);
}

const moveTab = (
  fromPanelId: string,
  toPanelId: string,
  tabId: string,
  targetTabId?: string
): void => {
  // Auto-detect source panel if not provided
  let actualFromPanelId = fromPanelId;
  if (!actualFromPanelId) {
    actualFromPanelId =
      findPanelContainingTab(panelLayout.value, tabId)?.id || "";
  }

  if (!actualFromPanelId) {
    // Silently handle missing panel - not critical
    return;
  }

  if (actualFromPanelId === toPanelId) {
    reorderTabWithinPanel(actualFromPanelId, tabId, targetTabId);
  } else {
    moveTabBetweenPanels(actualFromPanelId, toPanelId, tabId, targetTabId);
  }
};

const duplicateTab = (panelId: string, tabId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId);
  if (!panel) return;

  const sourceTab = panel.tabs.find((tab) => tab.id === tabId);
  if (!sourceTab) return;

  const newTabId = tabCounter.toString();
  const newTab: Tab = {
    id: newTabId,
    title: `${sourceTab.title} (Copy)`,
    color: sourceTab.color,
    profileId: sourceTab.profileId,
    groupId: sourceTab.groupId,
  };

  const newTerminal: TerminalInstance = { id: newTabId, ready: false };

  // Add tab to panel
  const sourceIndex = panel.tabs.findIndex((tab) => tab.id === tabId);
  panel.tabs.splice(sourceIndex + 1, 0, newTab);
  panel.activeTabId = newTabId;
  terminals.value.push(newTerminal);

  // Set active panel and switch to workspace
  activePanelId.value = panelId;
  viewState.setActiveView("workspace");

  // Request new terminal from main process
  if (sourceTab.profileId) {
    // TODO: Clone SSH connection
    // window.api?.send('terminal.createSSH', {
    //   terminalId: newTabId,
    //   profileId: sourceTab.profileId
    // })
  } else {
    // TODO: Clone regular terminal
    // window.api?.send('terminal.create', { terminalId: newTabId })
  }

  tabCounter++;
};

const moveTabToNewPanel = (panelId: string, tabId: string): void => {
  const sourcePanel = findPanelInLayout(panelLayout.value, panelId);
  if (!sourcePanel) return;

  const tab = sourcePanel.tabs.find((t) => t.id === tabId);
  if (!tab) return;

  // Create new panel with the moved tab
  const newPanelId = `panel-${panelCounter++}`;
  const newPanel: Panel = {
    id: newPanelId,
    activeTabId: tab.id,
    tabs: [tab],
  };

  // Remove tab from source panel
  const tabIndex = sourcePanel.tabs.findIndex((t) => t.id === tabId);
  sourcePanel.tabs.splice(tabIndex, 1);

  // Update source panel's active tab if needed
  if (sourcePanel.activeTabId === tabId) {
    if (sourcePanel.tabs.length > 0) {
      sourcePanel.activeTabId =
        sourcePanel.tabs[Math.min(tabIndex, sourcePanel.tabs.length - 1)].id;
    } else {
      sourcePanel.activeTabId = "";
    }
  }

  // Split the source panel layout to add the new panel
  splitPanelInLayout(panelLayout.value, panelId, newPanel, "horizontal");
  activePanelId.value = newPanelId;
};

const terminalReady = (terminalId: string): void => {
  const terminal = terminals.value.find((t) => t.id === terminalId);
  if (terminal) {
    terminal.ready = true;
  }
};

const setActivePanel = (panelId: string): void => {
  activePanelId.value = panelId;
};

const triggerTerminalResize = debounce((): void => {
  nextTick(() => {
    setTimeout(() => {
      window.dispatchEvent(new Event("resize"));
    }, 50);
  });
}, 150);

const updateLayout = (newLayout: PanelLayout): void => {
  // Deep clone to ensure reactivity
  panelLayout.value = JSON.parse(JSON.stringify(newLayout));
  // Trigger terminal resize after layout update
  triggerTerminalResize();
};
</script>
