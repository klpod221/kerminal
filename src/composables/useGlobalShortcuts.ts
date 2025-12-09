import { onMounted, onBeforeUnmount, watch } from "vue";
import { useKeyboardShortcutsStore } from "../stores/keyboardShortcuts";
import { ShortcutAction, ActiveShortcut } from "../types/shortcuts";
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

  /**
   * Check if keyboard event should be ignored due to input focus
   */
  const shouldIgnoreInput = (event: KeyboardEvent): boolean => {
    const target = event.target as HTMLElement;
    const isInputElement =
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.contentEditable === "true";

    if (!isInputElement) return false;

    // Check if it's xterm textarea (terminal) or a modal/drawer input
    const isXterm = target.classList.contains("xterm-helper-textarea");
    const isModalInput =
      target.closest('[role="dialog"]') || target.closest(".drawer-content");

    // Only allow shortcuts in terminal, not in modal/drawer inputs
    if (!isXterm && isModalInput) return true;

    // For xterm, allow shortcuts but they might be handled by terminal
    // (Existing logic implies returning true to ignore, but original was "return" to stop)
    // Original logic:
    // if (!isXterm && isModalInput) return; // Stop
    // if (!isXterm) return; // Stop (so if it IS xterm, continue?? No wait)
    // The original logic:
    // if (isInputElement) {
    //   if (!isXterm && isModalInput) return;
    //   if (!isXterm) return;
    // }
    // Basically: if input AND NOT xterm -> return (ignore)
    // If input AND IS xterm -> continue (don't ignore)
    // Wait, original:
    // if (!isXterm) return; // This runs if (!isXterm && isModalInput) is FALSE.
    // So if isModalInput is false, but isXterm is false (generic input), it returns.
    // So essentially: If input is NOT xterm, return.
    // So: if input AND NOT xterm, return true (ignore).
    // If input AND xterm, return false (process).

    return !isXterm;
  };

  /**
   * Check if event matches shortcut configuration
   */
  const isShortcutMatch = (
    event: KeyboardEvent,
    shortcut: ActiveShortcut,
  ): boolean => {
    // Normalize event key
    let normalizedEventKey = event.key.toLowerCase();
    if (normalizedEventKey === " ") normalizedEventKey = "space";
    else if (normalizedEventKey.startsWith("arrow"))
      normalizedEventKey = normalizedEventKey.replace("arrow", "");

    // Normalize shortcut key
    let normalizedShortcutKey = shortcut.key.toLowerCase();
    if (normalizedShortcutKey === " ") normalizedShortcutKey = "space";
    else if (normalizedShortcutKey.startsWith("arrow"))
      normalizedShortcutKey = normalizedShortcutKey.replace("arrow", "");

    if (normalizedEventKey !== normalizedShortcutKey) return false;

    // Check modifiers
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

    return (
      ctrlOrMetaMatches && altMatches && shiftMatches && onlyOneModifier
    );
  };

  const handleKeydown = (event: KeyboardEvent): void => {
    if (shouldIgnoreInput(event)) return;

    // Check all active shortcuts
    for (const shortcut of shortcutsStore.activeShortcuts) {
      if (isShortcutMatch(event, shortcut)) {
        event.preventDefault();
        event.stopPropagation();
        executeShortcutAction(shortcut.id);
        break;
      }
    }
  };

  /**
   * Execute action for a shortcut
   */
  /**
   * Action handlers map
   */
  const actionHandlers: Partial<Record<ShortcutAction, () => void>> = {
    [ShortcutAction.SplitVertical]: () => {
      const activePanelId = workspaceStore.activePanelId;
      if (activePanelId) workspaceStore.splitVertical(activePanelId);
    },
    [ShortcutAction.SplitHorizontal]: () => {
      const activePanelId = workspaceStore.activePanelId;
      if (activePanelId) workspaceStore.splitHorizontal(activePanelId);
    },
    [ShortcutAction.HistorySearch]: () => {
      const activePanelId = workspaceStore.activePanelId;
      if (activePanelId) {
        const activePanel = workspaceStore.findPanelInLayout(activePanelId);
        if (activePanel?.activeTabId) {
          openOverlay("history-search-modal", {
            terminalId: activePanel.activeTabId,
          });
        }
      }
    },
    [ShortcutAction.NewTab]: () => {
      const activePanelId = workspaceStore.activePanelId;
      if (activePanelId) workspaceStore.addTab(activePanelId);
    },
    [ShortcutAction.CloseTab]: () => {
      const activePanelId = workspaceStore.activePanelId;
      if (activePanelId) {
        const activePanel = workspaceStore.findPanelInLayout(activePanelId);
        if (activePanel?.activeTabId) {
          workspaceStore.closeTab(activePanel.id, activePanel.activeTabId);
        }
      }
    },
    [ShortcutAction.NextTab]: () => {
      const activePanelId = workspaceStore.activePanelId;
      if (activePanelId) {
        const activePanel = workspaceStore.findPanelInLayout(activePanelId);
        if (activePanel && activePanel.tabs.length > 1) {
          const currentIndex = activePanel.tabs.findIndex(
            (t) => t.id === activePanel.activeTabId,
          );
          const nextIndex = (currentIndex + 1) % activePanel.tabs.length;
          workspaceStore.selectTab(activePanel.id, activePanel.tabs[nextIndex].id);
        }
      }
    },
    [ShortcutAction.FocusNextPanel]: () => {
      viewState.setActiveView("workspace");
      const panelIds = workspaceStore.collectPanelIds();
      if (panelIds.length > 0) {
        const currentIndex = panelIds.indexOf(workspaceStore.activePanelId);
        const nextIndex =
          currentIndex === -1 ? 0 : (currentIndex + 1) % panelIds.length;

        const nextPanelId = panelIds[nextIndex];
        workspaceStore.setActivePanel(nextPanelId);
        const nextPanel = workspaceStore.findPanelInLayout(nextPanelId);
        if (nextPanel?.activeTabId) {
          workspaceStore.selectTab(nextPanelId, nextPanel.activeTabId);
        }
      }
    },
    [ShortcutAction.FocusPreviousPanel]: () => {
      viewState.setActiveView("workspace");
      const panelIds = workspaceStore.collectPanelIds();
      if (panelIds.length > 0) {
        const currentIndex = panelIds.indexOf(workspaceStore.activePanelId);
        
        let prevIndex = currentIndex - 1;
        if (currentIndex === -1 || prevIndex < 0) {
            prevIndex = panelIds.length - 1;
        }

        const prevPanelId = panelIds[prevIndex];
        workspaceStore.setActivePanel(prevPanelId);
        const prevPanel = workspaceStore.findPanelInLayout(prevPanelId);
        if (prevPanel?.activeTabId) {
          workspaceStore.selectTab(prevPanelId, prevPanel.activeTabId);
        }
      }
    },
    [ShortcutAction.OpenWorkspace]: () => viewState.setActiveView("workspace"),
    [ShortcutAction.OpenFileBrowser]: () => viewState.setActiveView("sftp"),
    [ShortcutAction.OpenSSHProfiles]: () => {
      openOverlay("ssh-profile-drawer");
    },
    [ShortcutAction.OpenSavedCommands]: () => {
      openOverlay("saved-command-drawer");
    },
    [ShortcutAction.OpenCommandPalette]: () => {
      openOverlay("command-palette");
    },
  };

  /**
   * Execute action for a shortcut
   */
  const executeShortcutAction = (action: ShortcutAction): void => {
    const handler = actionHandlers[action];
    if (handler) {
      handler();
    } else {
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
