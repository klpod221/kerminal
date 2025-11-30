import { onMounted, onBeforeUnmount, watch } from "vue";
import { useKeyboardShortcutsStore } from "../stores/keyboardShortcuts";
import { ShortcutAction } from "../types/shortcuts";
import { useWorkspaceStore } from "../stores/workspace";
import { useViewStateStore } from "../stores/viewState";
import { useOverlay } from "./useOverlay";

// Singleton flag to ensure only one listener is registered
let isListenerRegistered = false;

/**
 * Global keyboard shortcuts manager
 * Registers and handles all application-wide shortcuts
 * Uses singleton pattern to prevent duplicate event listeners
 */
export function useGlobalShortcuts() {
  const shortcutsStore = useKeyboardShortcutsStore();
  const workspaceStore = useWorkspaceStore();
  const viewState = useViewStateStore();
  const { openOverlay } = useOverlay();

  const handleKeydown = (event: KeyboardEvent): void => {
    // Don't handle shortcuts when typing in input fields
    const target = event.target as HTMLElement;
    const isInputElement =
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.contentEditable === "true";

    // Allow shortcuts in terminal (xterm textarea)
    // But also check if we're in a modal/drawer input field
    if (isInputElement) {
      // Check if it's xterm textarea (terminal) or a modal/drawer input
      const isXterm = target.classList.contains("xterm-helper-textarea");
      const isModalInput =
        target.closest('[role="dialog"]') || target.closest(".drawer-content");

      // Only allow shortcuts in terminal, not in modal/drawer inputs
      if (!isXterm && isModalInput) {
        return;
      }

      // For xterm, allow shortcuts but they might be handled by terminal
      if (!isXterm) {
        return;
      }
    }

    // Normalize event key for comparison
    let normalizedEventKey = event.key.toLowerCase();
    if (normalizedEventKey === " ") {
      normalizedEventKey = "space";
    } else if (normalizedEventKey.startsWith("arrow")) {
      normalizedEventKey = normalizedEventKey.replace("arrow", "");
    }

    // Check all active shortcuts
    for (const shortcut of shortcutsStore.activeShortcuts) {
      // Normalize shortcut key for comparison
      let normalizedShortcutKey = shortcut.key.toLowerCase();
      if (normalizedShortcutKey === " ") {
        normalizedShortcutKey = "space";
      } else if (normalizedShortcutKey.startsWith("arrow")) {
        normalizedShortcutKey = normalizedShortcutKey.replace("arrow", "");
      }

      const keyMatches = normalizedEventKey === normalizedShortcutKey;

      // Handle cross-platform: ctrlKey OR metaKey (for Ctrl/Cmd)
      // If shortcut requires ctrlKey or metaKey, accept either Ctrl (Windows/Linux) or Cmd (Mac)
      const needsCtrlOrMeta =
        !!shortcut.modifiers.ctrlKey || !!shortcut.modifiers.metaKey;
      const hasCtrlOrMeta = event.ctrlKey || event.metaKey;
      const ctrlOrMetaMatches = !needsCtrlOrMeta || hasCtrlOrMeta;

      const altMatches =
        !shortcut.modifiers.altKey ||
        event.altKey === !!shortcut.modifiers.altKey;
      const shiftMatches =
        !shortcut.modifiers.shiftKey ||
        event.shiftKey === !!shortcut.modifiers.shiftKey;

      // Don't match if both Ctrl and Meta are pressed (should be one or the other)
      const onlyOneModifier = !(event.ctrlKey && event.metaKey);

      if (
        keyMatches &&
        ctrlOrMetaMatches &&
        altMatches &&
        shiftMatches &&
        onlyOneModifier
      ) {
        event.preventDefault();
        event.stopPropagation();

        // Execute action based on shortcut ID
        executeShortcutAction(shortcut.id);
        break;
      }
    }
  };

  /**
   * Execute action for a shortcut
   */
  const executeShortcutAction = (action: ShortcutAction): void => {
    switch (action) {
      case ShortcutAction.SplitVertical: {
        const activePanelId = workspaceStore.activePanelId;
        if (activePanelId) {
          workspaceStore.splitVertical(activePanelId);
        }
        break;
      }

      case ShortcutAction.SplitHorizontal: {
        const activePanelId = workspaceStore.activePanelId;
        if (activePanelId) {
          workspaceStore.splitHorizontal(activePanelId);
        }
        break;
      }

      case ShortcutAction.HistorySearch: {
        // Get current terminal ID from active panel
        const activePanelId = workspaceStore.activePanelId;
        if (activePanelId) {
          const activePanel = workspaceStore.findPanelInLayout(activePanelId);
          if (activePanel?.activeTabId) {
            const terminalId = activePanel.activeTabId;
            openOverlay("history-search-modal", {
              terminalId,
            });
          }
        }
        break;
      }

      case ShortcutAction.NewTab: {
        const activePanelId = workspaceStore.activePanelId;
        if (activePanelId) {
          workspaceStore.addTab(activePanelId);
        }
        break;
      }

      case ShortcutAction.CloseTab: {
        const activePanelId = workspaceStore.activePanelId;
        if (activePanelId) {
          const activePanel = workspaceStore.findPanelInLayout(activePanelId);
          if (activePanel?.activeTabId) {
            workspaceStore.closeTab(activePanel.id, activePanel.activeTabId);
          }
        }
        break;
      }

      case ShortcutAction.NextTab: {
        const activePanelId = workspaceStore.activePanelId;
        if (activePanelId) {
          const activePanel = workspaceStore.findPanelInLayout(activePanelId);
          if (activePanel && activePanel.tabs.length > 1) {
            const currentIndex = activePanel.tabs.findIndex(
              (t) => t.id === activePanel.activeTabId,
            );
            const nextIndex = (currentIndex + 1) % activePanel.tabs.length;
            workspaceStore.selectTab(
              activePanel.id,
              activePanel.tabs[nextIndex].id,
            );
          }
        }
        break;
      }

      case ShortcutAction.FocusNextPanel: {
        // Ensure we're in workspace view
        viewState.setActiveView("workspace");

        const panelIds = workspaceStore.collectPanelIds();
        if (panelIds.length > 1) {
          const currentIndex = panelIds.indexOf(workspaceStore.activePanelId);
          if (currentIndex !== -1) {
            const nextIndex = (currentIndex + 1) % panelIds.length;
            const nextPanelId = panelIds[nextIndex];
            workspaceStore.setActivePanel(nextPanelId);

            // Ensure the active tab in the new panel is selected
            const nextPanel = workspaceStore.findPanelInLayout(nextPanelId);
            if (nextPanel?.activeTabId) {
              workspaceStore.selectTab(nextPanelId, nextPanel.activeTabId);
            }
          } else if (panelIds.length > 0) {
            // If current panel not found, focus first panel
            const firstPanelId = panelIds[0];
            workspaceStore.setActivePanel(firstPanelId);
            const firstPanel = workspaceStore.findPanelInLayout(firstPanelId);
            if (firstPanel?.activeTabId) {
              workspaceStore.selectTab(firstPanelId, firstPanel.activeTabId);
            }
          }
        }
        break;
      }

      case ShortcutAction.FocusPreviousPanel: {
        // Ensure we're in workspace view
        viewState.setActiveView("workspace");

        const panelIds = workspaceStore.collectPanelIds();
        if (panelIds.length > 1) {
          const currentIndex = panelIds.indexOf(workspaceStore.activePanelId);
          if (currentIndex !== -1) {
            const prevIndex =
              currentIndex === 0 ? panelIds.length - 1 : currentIndex - 1;
            const prevPanelId = panelIds[prevIndex];
            workspaceStore.setActivePanel(prevPanelId);

            // Ensure the active tab in the new panel is selected
            const prevPanel = workspaceStore.findPanelInLayout(prevPanelId);
            if (prevPanel?.activeTabId) {
              workspaceStore.selectTab(prevPanelId, prevPanel.activeTabId);
            }
          } else if (panelIds.length > 0) {
            // If current panel not found, focus last panel
            const lastPanelId = panelIds[panelIds.length - 1];
            workspaceStore.setActivePanel(lastPanelId);
            const lastPanel = workspaceStore.findPanelInLayout(lastPanelId);
            if (lastPanel?.activeTabId) {
              workspaceStore.selectTab(lastPanelId, lastPanel.activeTabId);
            }
          }
        }
        break;
      }

      case ShortcutAction.OpenWorkspace: {
        viewState.setActiveView("workspace");
        break;
      }

      case ShortcutAction.OpenFileBrowser: {
        viewState.setActiveView("sftp");
        break;
      }

      case ShortcutAction.OpenSSHProfiles: {
        openOverlay("ssh-profile-drawer");
        break;
      }

      case ShortcutAction.OpenSavedCommands: {
        openOverlay("saved-command-drawer");
        break;
      }

      case ShortcutAction.OpenCommandPalette: {
        openOverlay("command-palette");
        break;
      }

      default:
        console.warn(`Unknown shortcut action: ${action}`);
    }
  };

  // Watch for shortcut changes and re-register
  watch(
    () => shortcutsStore.activeShortcuts,
    () => {
      // Shortcuts are handled by the same event listener
      // No need to re-register
    },
    { deep: true },
  );

  onMounted(async () => {
    // Load shortcuts on mount (only if not already loaded)
    if (shortcutsStore.activeShortcuts.length === 0) {
      await shortcutsStore.loadShortcuts();
    }

    // Only register listener once (singleton pattern)
    if (!isListenerRegistered) {
      document.addEventListener("keydown", handleKeydown, true);
      isListenerRegistered = true;
    }
  });

  onBeforeUnmount(() => {
    // Don't remove listener here - it's a singleton that should persist
    // The listener will be removed when the app closes or if explicitly needed
  });

  return {
    executeShortcutAction,
  };
}
