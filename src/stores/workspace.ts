import { defineStore } from "pinia";
import { ref, nextTick } from "vue";
import { debounce } from "../utils/helpers";
import type { PanelLayout, TerminalInstance, Panel, Tab } from "../types/panel";
import { useViewStateStore } from "./viewState";
import {
  createLocalTerminal,
  closeTerminal,
  getUserHostname,
  listenToTerminalTitleChanges,
  listenToTerminalExits,
} from "../services/terminal";
import type { TerminalTitleChanged, TerminalExited } from "../types/panel";

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

  /**
   * Find a panel in the layout tree by ID
   * @param layout - The layout to search in
   * @param panelId - The panel ID to find
   * @returns The panel if found, null otherwise
   */
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

  /**
   * Remove a panel from the layout tree
   * @param layout - The layout to remove from
   * @param panelId - The panel ID to remove
   * @returns The updated layout or null if the panel should be removed
   */
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
   * Close a tab in a panel
   * @param panelId - The panel ID
   * @param tabId - The tab ID to close
   */
  const closeTab = async (panelId: string, tabId: string): Promise<void> => {
    const panel = findPanelInLayout(panelLayout.value, panelId);
    if (!panel) return;

    const tabIndex = panel.tabs.findIndex((tab) => tab.id === tabId);
    const terminalIndex = terminals.value.findIndex(
      (terminal) => terminal.id === tabId
    );

    if (tabIndex !== -1) {
      // Check if terminal is already being closed to prevent double close
      const terminal = terminals.value[terminalIndex];
      if (terminal?.isClosing) {
        return; // Already being closed, skip
      }

      // Close backend terminal if it has one
      if (terminalIndex !== -1) {
        const backendTerminalId = terminal.backendTerminalId;

        // Mark terminal as closing to prevent race conditions
        if (terminal) {
          terminal.isClosing = true;
        }

        if (backendTerminalId) {
          try {
            await closeTerminal(backendTerminalId);
          } catch (error) {
            console.error("Failed to close backend terminal:", error);
          }
        }

        // Remove terminal instance
        terminals.value.splice(terminalIndex, 1);
      }

      // Remove tab from panel
      panel.tabs.splice(tabIndex, 1);

      // If closed tab was active, set new active tab
      if (panel.activeTabId === tabId) {
        if (panel.tabs.length > 0) {
          // Choose the next tab or previous if closing the last tab
          let newActiveIndex;
          if (tabIndex < panel.tabs.length) {
            // If there's a tab after the closed one, select it
            newActiveIndex = tabIndex;
          } else {
            // If closing the last tab, select the previous one
            newActiveIndex = panel.tabs.length - 1;
          }
          panel.activeTabId = panel.tabs[newActiveIndex].id;
        } else {
          // No tabs left, close the panel
          autoClosePanel(panelId);
          return;
        }
      }
    }
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
          shouldFocusOnReady: true // Mark this terminal to focus when ready
        };
        terminals.value.push(newTerminal);
      } else {
        const newTabId = tabCounter.toString();
        newTab = { id: newTabId, title: "Terminal" };
        const newTerminal: TerminalInstance = {
          id: newTabId,
          ready: false,
          shouldFocusOnReady: true
        };
        terminals.value.push(newTerminal);
      }
    } else {
      const newTabId = tabCounter.toString();
      newTab = { id: newTabId, title: "Terminal" };
      const newTerminal: TerminalInstance = {
        id: newTabId,
        ready: false,
        shouldFocusOnReady: true
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
          shouldFocusOnReady: true // Mark this terminal to focus when ready
        };
        terminals.value.push(newTerminal);
      } else {
        const newTabId = tabCounter.toString();
        newTab = { id: newTabId, title: "Terminal" };
        const newTerminal: TerminalInstance = {
          id: newTabId,
          ready: false,
          shouldFocusOnReady: true
        };
        terminals.value.push(newTerminal);
      }
    } else {
      const newTabId = tabCounter.toString();
      newTab = { id: newTabId, title: "Terminal" };
      const newTerminal: TerminalInstance = {
        id: newTabId,
        ready: false,
        shouldFocusOnReady: true
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
        (terminal) => terminal.id === tabId
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

  /**
   * Insert a tab into a panel at a specific position
   * @param panel - The panel to insert the tab into
   * @param tab - The tab to insert
   * @param targetTabId - The tab ID to insert before (optional)
   */
  const insertTabToPanel = (panel: Panel, tab: Tab, targetTabId?: string): void => {
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
    targetTabId?: string
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
    tabIndex: number
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
    targetTabId?: string
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
      shouldFocusOnReady: true // Focus duplicated tabs when ready
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
        try {
          // Find the tab to get the title
          let title = "Terminal";

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
          }

          const response = await createLocalTerminal(title);

          // Store the backend terminal ID
          terminal.backendTerminalId = response.terminal_id;
        } catch (error) {
          console.error("Failed to create terminal:", error);
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
      unlistenTitleChanges = await listenToTerminalTitleChanges(
        (titleChange: TerminalTitleChanged) => {
          // Find the terminal and update its tab title
          const findTabInLayout = (layout: PanelLayout): Tab | undefined => {
            if (layout.type === "panel" && layout.panel) {
              return layout.panel.tabs.find((t: Tab) => {
                const terminal = terminals.value.find((term) => term.id === t.id);
                return terminal?.backendTerminalId === titleChange.terminal_id;
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
        }
      );

      // Setup terminal exit listener
      unlistenTerminalExits = await listenToTerminalExits(
        (exitEvent: TerminalExited) => {
          console.log("Terminal exited:", exitEvent);
          // Find the tab that corresponds to this terminal and close it
          const findTabByBackendId = (layout: PanelLayout): { panel: Panel; tab: Tab } | undefined => {
            if (layout.type === "panel" && layout.panel) {
              for (const tab of layout.panel.tabs) {
                const terminal = terminals.value.find((term) => term.id === tab.id);
                if (terminal?.backendTerminalId === exitEvent.terminal_id) {
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
            const terminal = terminals.value.find((term) => term.id === result.tab.id);
            if (terminal?.isClosing) {
              return; // Already being closed, skip
            }

            console.log(`Auto-closing tab ${result.tab.id} due to terminal exit`);
            closeTab(result.panel.id, result.tab.id);
          }
        }
      );
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

    // Automatically create the first terminal when app starts
    await addTab("panel-1");
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
    closeTab,
    splitVertical,
    splitHorizontal,
    closePanel,
    moveTab,
    duplicateTab,
    moveTabToNewPanel,
    terminalReady,
    updateLayout,
    initialize,
    cleanup,

    // Getters/Computed
    findPanelInLayout: (panelId: string) => findPanelInLayout(panelLayout.value, panelId),
  };
});
