import { defineStore } from "pinia";
import { ref, nextTick } from "vue";
import { debounce } from "../utils/helpers";
import { useViewStateStore } from "./viewState";
import {
  createLocalTerminal,
  createSSHTerminal,
  closeTerminal,
  getUserHostname,
  listenToTerminalTitleChanged,
  listenToTerminalExit,
} from "../services/terminal";
import { api } from "../services/api";
import type {
  TerminalTitleChanged,
  TerminalExited,
  PanelLayout,
  TerminalInstance,
  Panel,
  Tab,
} from "../types/panel";

/**
 * Workspace Store
 * Manages panel layouts, terminals, and workspace state
 */
export const useWorkspaceStore = defineStore("workspace", () => {
  const viewState = useViewStateStore();

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

// Event listener cleanup functions
let unlistenTitleChanges: (() => void) | null = null;
let unlistenTerminalExits: (() => void) | null = null;
let unlistenSSHConnected: (() => void) | null = null;  /**
   * Find a panel in the layout tree by ID
   * @param layout - The layout to search in
   * @param panelId - The panel ID to find
   * @returns The panel if found, null otherwise
   */
  const findPanelInLayout = (
    layout: PanelLayout,
    panelId: string,
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

  /**
   * Remove a panel from the layout tree
   * @param layout - The layout to remove from
   * @param panelId - The panel ID to remove
   * @returns The updated layout or null if the panel should be removed
   */
  const removePanelFromLayout = (
    layout: PanelLayout,
    panelId: string,
  ): PanelLayout | null => {
    if (layout.type === "panel" && layout.panel?.id === panelId) {
      return null; // This panel should be removed
    }

    if (layout.type === "split" && layout.children) {
      const filteredChildren = layout.children
        .map((child) => removePanelFromLayout(child, panelId))
        .filter((child) => child !== null);

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

  /**
   * Find the first panel in the layout tree
   * @param layout - The layout to search in
   * @returns The first panel found or null
   */
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

  /**
   * Auto close panel when it has no tabs left
   * @param panelId - The panel ID to close
   */
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

  /**
   * Set the active panel
   * @param panelId - The panel ID to activate
   */
  const setActivePanel = (panelId: string): void => {
    activePanelId.value = panelId;
  };

  /**
   * Select a tab in a panel
   * @param panelId - The panel ID
   * @param tabId - The tab ID to select
   */
  const selectTab = (panelId: string, tabId: string): void => {
    const panel = findPanelInLayout(panelLayout.value, panelId);
    if (panel) {
      panel.activeTabId = tabId;
      viewState.setActiveView("workspace");
      // Note: setActivePanel will be called separately by Panel component
    }
  };

  /**
   * Add a new tab to a panel
   * @param panelId - The panel ID to add the tab to
   */
  const addTab = async (panelId: string): Promise<void> => {
    const panel = findPanelInLayout(panelLayout.value, panelId);
    if (!panel) return;

    const newTabId = tabCounter.toString();

    // Get username and hostname from backend
    const defaultTitle = await getUserHostname();

    const newTab: Tab = {
      id: newTabId,
      title: defaultTitle,
    };

    const newTerminal: TerminalInstance = {
      id: newTabId,
      ready: false,
      shouldFocusOnReady: true, // Focus new tabs when ready
    };

    // Add tab to panel
    panel.tabs.push(newTab);
    panel.activeTabId = newTabId;

    // Add terminal instance
    terminals.value.push(newTerminal);

    // Switch to workspace (setActivePanel will be called separately by Panel component)
    viewState.setActiveView("workspace");

    // DON'T create backend terminal here - wait for terminal component to be ready

    tabCounter++;
  };

  /**
   * Add a new SSH terminal tab to a panel
   * @param panelId - The panel ID to add the tab to
   * @param profileId - The SSH profile ID to connect with
   * @param profileName - The SSH profile name for the tab title
   */
  const addSSHTab = async (
    panelId: string,
    profileId: string,
    profileName: string,
  ): Promise<void> => {
    const panel = findPanelInLayout(panelLayout.value, panelId);
    if (!panel) return;

    const newTabId = tabCounter.toString();

    const newTab: Tab = {
      id: newTabId,
      title: profileName,
      profileId: profileId, // Store SSH profile ID in the tab
    };

    const newTerminal: TerminalInstance = {
      id: newTabId,
      ready: false,
      shouldFocusOnReady: true,
      isSSHConnecting: true, // Mark as SSH connecting
      sshProfileId: profileId, // Store SSH profile ID for reconnection
      canReconnect: true, // SSH terminals can reconnect
    };

    // Add tab to panel
    panel.tabs.push(newTab);
    panel.activeTabId = newTabId;

    // Add terminal instance
    terminals.value.push(newTerminal);

    // Switch to workspace
    viewState.setActiveView("workspace");

    tabCounter++;
  };

  /**
   * Handle SSH connection error for a terminal
   * @param terminalId - The terminal ID that failed to connect
   * @param error - The error message
   */
  const handleSSHConnectionError = async (terminalId: string, error: string): Promise<void> => {
    // Find and update terminal state
    const terminal = terminals.value.find(t => t.id === terminalId);
    if (terminal) {
      terminal.isSSHConnecting = false;
      terminal.hasError = true;
      terminal.errorMessage = error;
      terminal.canReconnect = true; // Enable reconnect for error cases
    }

    // Find the tab that corresponds to this terminal
    const findTabByTerminalId = (layout: PanelLayout): { panel: Panel; tab: Tab } | undefined => {
      if (layout.type === "panel" && layout.panel) {
        for (const tab of layout.panel.tabs) {
          if (tab.id === terminalId) {
            return { panel: layout.panel, tab };
          }
        }
      } else if (layout.type === "split" && layout.children) {
        for (const child of layout.children) {
          const found = findTabByTerminalId(child);
          if (found) return found;
        }
      }
      return undefined;
    };

    const result = findTabByTerminalId(panelLayout.value);
    if (result) {
      // Update tab title to show error
      result.tab.title = `${result.tab.title} (Failed)`;
    }
  };

    /**
   * Handle SSH connection success for a terminal
   * @param terminalId - The terminal ID that successfully connected
   */
  const handleSSHConnectionSuccess = (terminalId: string): void => {
    const terminal = terminals.value.find(t => t.id === terminalId);
    if (terminal) {
      terminal.isSSHConnecting = false;
      terminal.disconnectReason = undefined; // Clear any previous disconnect reason
      terminal.hasError = false; // Clear any previous error
      terminal.errorMessage = undefined;
    }
  };

  /**
   * Reconnect SSH terminal
   * @param terminalId - The terminal ID to reconnect
   * @param profileId - The SSH profile ID to use (currently not used but may be needed for validation)
   */
  const reconnectSSH = async (terminalId: string, _profileId: string): Promise<void> => {
    const terminal = terminals.value.find(t => t.id === terminalId);
    if (!terminal) return;

    // Clear disconnected state and mark as connecting
    terminal.disconnectReason = undefined;
    terminal.hasError = false;
    terminal.errorMessage = undefined;
    terminal.isSSHConnecting = true;
    terminal.backendTerminalId = undefined;

    // Trigger terminal ready to recreate backend connection
    try {
      // The terminalReady handler will automatically create the backend terminal
      // because backendTerminalId is now undefined
      await terminalReady(terminalId);
    } catch (error) {
      console.error("Failed to reconnect SSH:", error);
      terminal.isSSHConnecting = false;
      terminal.disconnectReason = "connection-lost";
    }
  };

  /**
   * Helper: Remove terminal instance and close backend terminal if needed
   */
  const removeTerminalInstance = async (terminalId: string): Promise<void> => {
    const terminalIndex = terminals.value.findIndex(
      (terminal) => terminal.id === terminalId,
    );
    if (terminalIndex === -1) return;
    const terminal = terminals.value[terminalIndex];
    if (terminal?.isClosing) return;
    if (terminal) {
      terminal.isClosing = true;
      terminal.disconnectReason = "user-closed"; // Mark as user-closed
    }
    if (terminal?.backendTerminalId) {
      try {
        await closeTerminal(terminal.backendTerminalId);
      } catch (error) {
        console.error("Failed to close backend terminal:", error);
      }
    }
    terminals.value.splice(terminalIndex, 1);
  };

  /**
   * Helper: Update active tab after closing
   */
  const updateActiveTabAfterClose = (
    panel: Panel,
    tabId: string,
    tabIndex: number,
    panelId: string,
  ): boolean => {
    if (panel.activeTabId !== tabId) return false;
    if (panel.tabs.length > 0) {
      let newActiveIndex =
        tabIndex < panel.tabs.length ? tabIndex : panel.tabs.length - 1;
      panel.activeTabId = panel.tabs[newActiveIndex].id;
      return false;
    } else {
      autoClosePanel(panelId);
      return true;
    }
  };

  /**
   * Close a tab in a panel
   * @param panelId - The panel ID
   * @param tabId - The tab ID to close
   */
  const closeTab = async (panelId: string, tabId: string): Promise<void> => {
    const panel = findPanelInLayout(panelLayout.value, panelId);
    if (!panel) return;
    const tabIndex = panel.tabs.findIndex((tab) => tab.id === tabId);
    if (tabIndex === -1) return;

    await removeTerminalInstance(tabId);
    panel.tabs.splice(tabIndex, 1);

    // If closed tab was active, set new active tab or close panel
    if (updateActiveTabAfterClose(panel, tabId, tabIndex, panelId)) return;
  };

  /**
   * Split a panel in the layout tree
   * @param layout - The layout to split
   * @param panelId - The panel ID to split
   * @param newPanel - The new panel to add
   * @param direction - The split direction
   * @returns True if the split was successful
   */
  const splitPanelInLayout = (
    layout: PanelLayout,
    panelId: string,
    newPanel: Panel,
    direction: "horizontal" | "vertical",
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

  /**
   * Split a panel vertically
   * @param panelId - The panel ID to split
   */
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
        const newTerminal: TerminalInstance = {
          id: newTabId,
          ready: false,
          shouldFocusOnReady: true, // Mark this terminal to focus when ready
        };
        terminals.value.push(newTerminal);
      } else {
        const newTabId = tabCounter.toString();
        newTab = { id: newTabId, title: "Terminal" };
        const newTerminal: TerminalInstance = {
          id: newTabId,
          ready: false,
          shouldFocusOnReady: true,
        };
        terminals.value.push(newTerminal);
      }
    } else {
      const newTabId = tabCounter.toString();
      newTab = { id: newTabId, title: "Terminal" };
      const newTerminal: TerminalInstance = {
        id: newTabId,
        ready: false,
        shouldFocusOnReady: true,
      };
      terminals.value.push(newTerminal);
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
    setActivePanel(newPanelId);
    tabCounter++;
  };

  /**
   * Split a panel horizontally
   * @param panelId - The panel ID to split
   */
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
        const newTerminal: TerminalInstance = {
          id: newTabId,
          ready: false,
          shouldFocusOnReady: true, // Mark this terminal to focus when ready
        };
        terminals.value.push(newTerminal);
      } else {
        const newTabId = tabCounter.toString();
        newTab = { id: newTabId, title: "Terminal" };
        const newTerminal: TerminalInstance = {
          id: newTabId,
          ready: false,
          shouldFocusOnReady: true,
        };
        terminals.value.push(newTerminal);
      }
    } else {
      const newTabId = tabCounter.toString();
      newTab = { id: newTabId, title: "Terminal" };
      const newTerminal: TerminalInstance = {
        id: newTabId,
        ready: false,
        shouldFocusOnReady: true,
      };
      terminals.value.push(newTerminal);
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
    setActivePanel(newPanelId);
    tabCounter++;
  };

  /**
   * Close a panel
   * @param panelId - The panel ID to close
   */
  const closePanel = (panelId: string): void => {
    const panel = findPanelInLayout(panelLayout.value, panelId);
    if (!panel) return;

    // Close all tabs in the panel (this will also destroy their terminals)
    const tabIds = [...panel.tabs.map((tab) => tab.id)]; // Create a copy to avoid mutation during iteration

    for (const tabId of tabIds) {
      const terminalIndex = terminals.value.findIndex(
        (terminal) => terminal.id === tabId,
      );
      if (terminalIndex !== -1) {
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

  /**
   * Find the panel containing a specific tab
   * @param layout - The layout to search in
   * @param tabId - The tab ID to find
   * @returns The panel containing the tab or null
   */
  const findPanelContainingTab = (
    layout: PanelLayout,
    tabId: string,
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

  /**
   * Insert a tab into a panel at a specific position
   * @param panel - The panel to insert the tab into
   * @param tab - The tab to insert
   * @param targetTabId - The tab ID to insert before (optional)
   */
  const insertTabToPanel = (
    panel: Panel,
    tab: Tab,
    targetTabId?: string,
  ): void => {
    if (targetTabId) {
      const targetIndex = panel.tabs.findIndex((t) => t.id === targetTabId);
      if (targetIndex !== -1) {
        panel.tabs.splice(targetIndex, 0, tab);
        return;
      }
    }
    panel.tabs.push(tab);
  };

  /**
   * Reorder a tab within a panel
   * @param panelId - The panel ID
   * @param tabId - The tab ID to move
   * @param targetTabId - The tab ID to move before (optional)
   */
  const reorderTabWithinPanel = (
    panelId: string,
    tabId: string,
    targetTabId?: string,
  ): void => {
    const panel = findPanelInLayout(panelLayout.value, panelId);
    if (!panel || !targetTabId) return;

    const draggedIndex = panel.tabs.findIndex((tab) => tab.id === tabId);
    const targetIndex = panel.tabs.findIndex((tab) => tab.id === targetTabId);

    if (draggedIndex === -1 || targetIndex === -1) return;

    const [draggedTab] = panel.tabs.splice(draggedIndex, 1);
    panel.tabs.splice(targetIndex, 0, draggedTab);
  };

  /**
   * Update active tabs after moving a tab between panels
   * @param fromPanel - The source panel
   * @param toPanel - The destination panel
   * @param tabId - The tab ID that was moved
   * @param tabIndex - The original index of the tab
   */
  const updateActiveTabsAfterMove = (
    fromPanel: Panel,
    toPanel: Panel,
    tabId: string,
    tabIndex: number,
  ): void => {
    if (fromPanel.activeTabId === tabId) {
      if (fromPanel.tabs.length > 0) {
        fromPanel.activeTabId =
          fromPanel.tabs[Math.min(tabIndex, fromPanel.tabs.length - 1)].id;
      } else {
        fromPanel.activeTabId = "";
      }
    }
    toPanel.activeTabId = tabId;
    setActivePanel(toPanel.id);
  };

  /**
   * Move a tab between panels
   * @param fromPanelId - The source panel ID
   * @param toPanelId - The destination panel ID
   * @param tabId - The tab ID to move
   * @param targetTabId - The tab ID to move before (optional)
   */
  const moveTabBetweenPanels = (
    fromPanelId: string,
    toPanelId: string,
    tabId: string,
    targetTabId?: string,
  ): void => {
    const fromPanel = findPanelInLayout(panelLayout.value, fromPanelId);
    const toPanel = findPanelInLayout(panelLayout.value, toPanelId);

    if (!fromPanel || !toPanel) return;

    const tabIndex = fromPanel.tabs.findIndex((tab) => tab.id === tabId);
    if (tabIndex === -1) return;

    const [tab] = fromPanel.tabs.splice(tabIndex, 1);

    insertTabToPanel(toPanel, tab, targetTabId);

    updateActiveTabsAfterMove(fromPanel, toPanel, tabId, tabIndex);
  };

  /**
   * Move a tab (within panel or between panels)
   * @param fromPanelId - The source panel ID
   * @param toPanelId - The destination panel ID
   * @param tabId - The tab ID to move
   * @param targetTabId - The tab ID to move before (optional)
   */
  const moveTab = (
    fromPanelId: string,
    toPanelId: string,
    tabId: string,
    targetTabId?: string,
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

  /**
   * Duplicate a tab in a panel
   * @param panelId - The panel ID
   * @param tabId - The tab ID to duplicate
   */
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

    const newTerminal: TerminalInstance = {
      id: newTabId,
      ready: false,
      shouldFocusOnReady: true, // Focus duplicated tabs when ready
    };

    // Add tab to panel
    const sourceIndex = panel.tabs.findIndex((tab) => tab.id === tabId);
    panel.tabs.splice(sourceIndex + 1, 0, newTab);
    panel.activeTabId = newTabId;
    terminals.value.push(newTerminal);

    // Set active panel and switch to workspace
    setActivePanel(panelId);
    viewState.setActiveView("workspace");

    tabCounter++;
  };

  /**
   * Split a panel by dropping a tab
   * @param direction - The split direction
   * @param draggedTab - The dragged tab
   * @param sourcePanelId - The source panel ID
   * @param targetPanelId - The target panel ID
   */
  const splitPanelByDrop = (
    direction: "top" | "bottom" | "left" | "right",
    draggedTab: Tab,
    sourcePanelId: string,
    targetPanelId: string,
  ): void => {
    const sourcePanel = findPanelInLayout(panelLayout.value, sourcePanelId);
    const targetPanel = findPanelInLayout(panelLayout.value, targetPanelId);

    if (!sourcePanel || !targetPanel) return;

    // Remove tab from source panel
    const tabIndex = sourcePanel.tabs.findIndex(
      (tab) => tab.id === draggedTab.id,
    );
    if (tabIndex === -1) return;

    const [tab] = sourcePanel.tabs.splice(tabIndex, 1);

    // Update source panel's active tab if needed
    if (sourcePanel.activeTabId === draggedTab.id) {
      if (sourcePanel.tabs.length > 0) {
        sourcePanel.activeTabId =
          sourcePanel.tabs[Math.min(tabIndex, sourcePanel.tabs.length - 1)].id;
      } else {
        sourcePanel.activeTabId = "";
      }
    }

    // Create new panel with the moved tab
    const newPanelId = `panel-${panelCounter++}`;
    const newPanel: Panel = {
      id: newPanelId,
      activeTabId: tab.id,
      tabs: [tab],
    };

    // Determine split direction based on drop zone
    let splitDirection: "horizontal" | "vertical";
    if (direction === "top" || direction === "bottom") {
      splitDirection = "vertical";
    } else {
      splitDirection = "horizontal";
    }

    // Split the target panel
    const success = splitPanelInLayout(
      panelLayout.value,
      targetPanelId,
      newPanel,
      splitDirection,
    );

    if (success) {
      // If direction is 'top' or 'left', we need to swap the order of panels
      if (direction === "top" || direction === "left") {
        // Find the newly created split layout and swap its children
        const swapChildrenInLayout = (layout: PanelLayout): boolean => {
          if (
            layout.type === "split" &&
            layout.children &&
            layout.children.length === 2
          ) {
            // Check if one of the children is our new panel
            const hasNewPanel = layout.children.some(
              (child) =>
                child.type === "panel" && child.panel?.id === newPanelId,
            );
            const hasTargetPanel = layout.children.some((child) => {
              if (child.type === "panel") {
                return child.panel?.id === targetPanelId;
              } else if (child.type === "split") {
                return findPanelInLayout(child, targetPanelId) !== null;
              }
              return false;
            });

            if (hasNewPanel && hasTargetPanel) {
              // Swap the children
              [layout.children[0], layout.children[1]] = [
                layout.children[1],
                layout.children[0],
              ];
              return true;
            }
          }

          if (layout.type === "split" && layout.children) {
            return layout.children.some((child) => swapChildrenInLayout(child));
          }

          return false;
        };

        swapChildrenInLayout(panelLayout.value);
      }

      setActivePanel(newPanelId);

      // Auto-close source panel if it has no tabs left
      if (sourcePanel.tabs.length === 0) {
        autoClosePanel(sourcePanelId);
      }
    }
  };

  /**
   * Clone a tab and split panel (used when dropping tab to same panel edges)
   * @param direction - The split direction
   * @param tabId - The tab ID to clone
   * @param panelId - The panel ID
   */
  const cloneTabAndSplit = (
    direction: "top" | "bottom" | "left" | "right",
    tabId: string,
    panelId: string,
  ): void => {
    const panel = findPanelInLayout(panelLayout.value, panelId);
    if (!panel) return;

    const sourceTab = panel.tabs.find((tab) => tab.id === tabId);
    if (!sourceTab) return;

    // Create new tab (clone of the dragged tab)
    const newTabId = tabCounter.toString();
    const newTab: Tab = {
      id: newTabId,
      title: sourceTab.title,
      color: sourceTab.color,
      profileId: sourceTab.profileId,
      groupId: sourceTab.groupId,
    };

    // Create terminal for cloned tab
    const newTerminal: TerminalInstance = {
      id: newTabId,
      ready: false,
      shouldFocusOnReady: true,
    };
    terminals.value.push(newTerminal);

    // Create new panel with the cloned tab
    const newPanelId = `panel-${panelCounter++}`;
    const newPanel: Panel = {
      id: newPanelId,
      activeTabId: newTab.id,
      tabs: [newTab],
    };

    // Determine split direction based on drop zone
    let splitDirection: "horizontal" | "vertical";
    if (direction === "top" || direction === "bottom") {
      splitDirection = "vertical";
    } else {
      splitDirection = "horizontal";
    }

    // Split the panel
    const success = splitPanelInLayout(
      panelLayout.value,
      panelId,
      newPanel,
      splitDirection,
    );

    if (success) {
      // If direction is 'top' or 'left', we need to swap the order of panels
      if (direction === "top" || direction === "left") {
        // Find the newly created split layout and swap its children
        const swapChildrenInLayout = (layout: PanelLayout): boolean => {
          if (
            layout.type === "split" &&
            layout.children &&
            layout.children.length === 2
          ) {
            // Check if one of the children is our new panel
            const hasNewPanel = layout.children.some(
              (child) =>
                child.type === "panel" && child.panel?.id === newPanelId,
            );
            const hasTargetPanel = layout.children.some((child) => {
              if (child.type === "panel") {
                return child.panel?.id === panelId;
              } else if (child.type === "split") {
                return findPanelInLayout(child, panelId) !== null;
              }
              return false;
            });

            if (hasNewPanel && hasTargetPanel) {
              // Swap the children
              [layout.children[0], layout.children[1]] = [
                layout.children[1],
                layout.children[0],
              ];
              return true;
            }
          }

          if (layout.type === "split" && layout.children) {
            return layout.children.some((child) => swapChildrenInLayout(child));
          }

          return false;
        };

        swapChildrenInLayout(panelLayout.value);
      }

      setActivePanel(newPanelId);
      tabCounter++;
    }
  };

  /**
   * Move a tab to a new panel
   * @param panelId - The panel ID
   * @param tabId - The tab ID to move
   */
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
    setActivePanel(newPanelId);
  };

  /**
   * Handle terminal ready event
   * @param terminalId - The terminal ID that is ready
   */
  const terminalReady = async (terminalId: string): Promise<void> => {
    const terminal = terminals.value.find((t) => t.id === terminalId);
    if (terminal) {
      terminal.ready = true;

      // Clear the shouldFocusOnReady flag after use
      if (terminal.shouldFocusOnReady) {
        terminal.shouldFocusOnReady = false;
      }

      // Create backend terminal now that the frontend is ready
      if (!terminal.backendTerminalId) {
        // Find the tab to get the title and profile info
        let title = "Terminal";
        let profileId: string | undefined;

        const findTabInLayout = (layout: PanelLayout): Tab | undefined => {
          if (layout.type === "panel" && layout.panel) {
            return layout.panel.tabs.find((t: Tab) => t.id === terminalId);
          } else if (layout.type === "split" && layout.children) {
            for (const child of layout.children) {
              const found = findTabInLayout(child);
              if (found) return found;
            }
          }
          return undefined;
        };

        const tab = findTabInLayout(panelLayout.value);
        if (tab) {
          title = tab.title;
          profileId = tab.profileId;
        }

        try {
          let response;
          if (profileId) {
            // This is an SSH terminal
            response = await createSSHTerminal(profileId);
          } else {
            // This is a local terminal
            response = await createLocalTerminal(
              undefined,
              undefined,
              title,
            );
          }

          // Store the backend terminal ID
          terminal.backendTerminalId = response.terminalId;
        } catch (error) {
          console.error("Failed to create terminal:", error);
          // Clear the connecting state if it was an SSH connection
          if (terminal.isSSHConnecting && profileId) {
            // Handle SSH connection error - properly format error message
            let errorMessage: string;
            if (error instanceof Error) {
              errorMessage = error.message;
            } else if (typeof error === 'string') {
              errorMessage = error;
            } else if (error && typeof error === 'object') {
              // Handle object errors by stringifying or extracting message
              errorMessage = (error as any).message || JSON.stringify(error, null, 2);
            } else {
              errorMessage = 'Unknown connection error occurred';
            }
            await handleSSHConnectionError(terminalId, errorMessage);
          } else if (terminal.isSSHConnecting) {
            terminal.isSSHConnecting = false;
          }
        }
      }
    }
  };

  /**
   * Trigger terminal resize with debounce
   */
  const triggerTerminalResize = debounce((): void => {
    nextTick(() => {
      setTimeout(() => {
        window.dispatchEvent(new Event("resize"));
      }, 50);
    });
  }, 150);

  /**
   * Update the panel layout
   * @param newLayout - The new layout
   */
  const updateLayout = (newLayout: PanelLayout): void => {
    // Deep clone to ensure reactivity
    panelLayout.value = JSON.parse(JSON.stringify(newLayout));
    // Trigger terminal resize after layout update
    triggerTerminalResize();
  };

  /**
   * Initialize the workspace store
   */
  const initialize = async (): Promise<void> => {
    // Setup title change listener
    try {
      unlistenTitleChanges = await listenToTerminalTitleChanged(
        (titleChange: TerminalTitleChanged) => {
          // Find the terminal and update its tab title
          const findTabInLayout = (layout: PanelLayout): Tab | undefined => {
            if (layout.type === "panel" && layout.panel) {
              return layout.panel.tabs.find((t: Tab) => {
                const terminal = terminals.value.find(
                  (term) => term.id === t.id,
                );
                return terminal?.backendTerminalId === titleChange.terminalId;
              });
            } else if (layout.type === "split" && layout.children) {
              for (const child of layout.children) {
                const found = findTabInLayout(child);
                if (found) return found;
              }
            }
            return undefined;
          };

          const tab = findTabInLayout(panelLayout.value);
          if (tab) {
            tab.title = titleChange.title;
          }
        },
      );

      // Setup terminal exit listener
      unlistenTerminalExits = await listenToTerminalExit(
        (exitEvent: TerminalExited) => {
          console.log("Terminal exited:", exitEvent);
          // Find the tab that corresponds to this terminal
          const findTabByBackendId = (
            layout: PanelLayout,
          ): { panel: Panel; tab: Tab } | undefined => {
            if (layout.type === "panel" && layout.panel) {
              for (const tab of layout.panel.tabs) {
                const terminal = terminals.value.find(
                  (term) => term.id === tab.id,
                );
                if (terminal?.backendTerminalId === exitEvent.terminalId) {
                  return { panel: layout.panel, tab };
                }
              }
            } else if (layout.type === "split" && layout.children) {
              for (const child of layout.children) {
                const found = findTabByBackendId(child);
                if (found) return found;
              }
            }
            return undefined;
          };

          const result = findTabByBackendId(panelLayout.value);
          if (result) {
            // Check if the terminal is already being closed to prevent double close
            const terminal = terminals.value.find(
              (term) => term.id === result.tab.id,
            );
            if (terminal?.isClosing) {
              return; // Already being closed, skip
            }

            // If reason is user-closed, auto-close the tab
            if (exitEvent.reason === "user-closed" || terminal?.disconnectReason === "user-closed") {
              console.log(
                `Auto-closing tab ${result.tab.id} due to user-initiated close`,
              );
              closeTab(result.panel.id, result.tab.id);
            } else {
              // Connection lost - show reconnect UI
              console.log(
                `Connection lost for tab ${result.tab.id}, showing reconnect UI`,
              );
              if (terminal) {
                terminal.disconnectReason = "connection-lost";
                terminal.isSSHConnecting = false;
                terminal.backendTerminalId = undefined; // Clear backend ID for reconnect
              }
            }
          }
        },
      );

      // Setup SSH connected listener
      try {
        unlistenSSHConnected = await api.listen<{ terminalId: string }>(
          "ssh-connected",
          (data: { terminalId: string }) => {
            console.log("🎉 SSH Connected event received for terminal:", data.terminalId);

            // Find the terminal by backend ID or by connecting state if ID not yet set
            let terminal = terminals.value.find((t) => t.backendTerminalId === data.terminalId);

            if (!terminal) {
              // If not found by backend ID, look for SSH connecting terminals without backend ID
              terminal = terminals.value.find((t) => t.isSSHConnecting && !t.backendTerminalId);
              if (terminal) {
                console.log("🔗 Linking SSH terminal frontend ID", terminal.id, "with backend ID", data.terminalId);
                terminal.backendTerminalId = data.terminalId;
              }
            }

            if (terminal) {
              console.log("✅ Clearing SSH connecting state for terminal:", terminal.id);
              terminal.isSSHConnecting = false;
              handleSSHConnectionSuccess(terminal.id);
            } else {
              console.warn("⚠️ Terminal not found for SSH connected event:", data.terminalId);
              console.log("📋 Current terminals:", terminals.value.map(t => ({
                id: t.id,
                backendId: t.backendTerminalId,
                isSSHConnecting: t.isSSHConnecting
              })));
            }
          },
        );
      } catch (error) {
        console.error("Failed to setup SSH connected listener:", error);
      }
    } catch (error) {
      console.error("Failed to setup title change listener:", error);
    }

    // Use requestAnimationFrame to ensure UI is fully rendered
    await new Promise((resolve) => {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          setTimeout(resolve, 200);
        });
      });
    });

    // Automatically create the first terminal when app starts if no terminals exist
    if (terminals.value.length === 0) {
      const firstPanel = findFirstPanel(panelLayout.value);
      if (firstPanel) {
        await addTab(firstPanel.id);
      }
    }
  };

  /**
   * Cleanup the workspace store
   */
  const cleanup = (): void => {
    if (unlistenTitleChanges) {
      unlistenTitleChanges();
      unlistenTitleChanges = null;
    }
    if (unlistenTerminalExits) {
      unlistenTerminalExits();
      unlistenTerminalExits = null;
    }
    if (unlistenSSHConnected) {
      unlistenSSHConnected();
      unlistenSSHConnected = null;
    }
  };

  return {
    // State
    panelLayout,
    activePanelId,
    terminals,

    // Actions
    setActivePanel,
    selectTab,
    addTab,
    addSSHTab,
    closeTab,
    splitVertical,
    splitHorizontal,
    closePanel,
    moveTab,
    duplicateTab,
    moveTabToNewPanel,
    splitPanelByDrop,
    cloneTabAndSplit,
    terminalReady,
    updateLayout,
    initialize,
    cleanup,
    handleSSHConnectionError,
    handleSSHConnectionSuccess,
    reconnectSSH,

    // Getters/Computed
    findPanelInLayout: (panelId: string) =>
      findPanelInLayout(panelLayout.value, panelId),
  };
});
