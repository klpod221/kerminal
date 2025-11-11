import type { KeyboardShortcut } from "../types/shortcuts";
import { ShortcutAction } from "../types/shortcuts";

/**
 * Default keyboard shortcuts configuration
 * These are the built-in shortcuts that come with the application
 */
export const DEFAULT_SHORTCUTS: KeyboardShortcut[] = [
  // Terminal splitting
  {
    id: ShortcutAction.SplitVertical,
    label: "Split Terminal Vertically",
    category: "Terminal",
    defaultKey: "k",
    defaultModifiers: { ctrlKey: true }, // Ctrl on Windows/Linux, Cmd on Mac (handled by logic)
    customizable: true,
  },
  {
    id: ShortcutAction.SplitHorizontal,
    label: "Split Terminal Horizontally",
    category: "Terminal",
    defaultKey: "l",
    defaultModifiers: { ctrlKey: true }, // Ctrl on Windows/Linux, Cmd on Mac
    customizable: true,
  },

  // History
  {
    id: ShortcutAction.HistorySearch,
    label: "Search Command History",
    category: "Terminal",
    defaultKey: "h",
    defaultModifiers: { ctrlKey: true }, // Ctrl on Windows/Linux, Cmd on Mac
    customizable: true,
  },

  // Terminal operations
  {
    id: ShortcutAction.NewTab,
    label: "New Tab",
    category: "Terminal",
    defaultKey: "t",
    defaultModifiers: { ctrlKey: true }, // Ctrl on Windows/Linux, Cmd on Mac
    customizable: true,
  },
  {
    id: ShortcutAction.CloseTab,
    label: "Close Tab",
    category: "Terminal",
    defaultKey: "w",
    defaultModifiers: { ctrlKey: true }, // Ctrl on Windows/Linux, Cmd on Mac
    customizable: true,
  },
  {
    id: ShortcutAction.NextTab,
    label: "Next Tab",
    category: "Terminal",
    defaultKey: "Tab",
    defaultModifiers: { ctrlKey: true }, // Ctrl on Windows/Linux, Cmd on Mac
    customizable: true,
  },

  // Navigation
  {
    id: ShortcutAction.FocusNextPanel,
    label: "Focus Next Panel",
    category: "Navigation",
    defaultKey: "ArrowRight",
    defaultModifiers: { ctrlKey: true }, // Ctrl on Windows/Linux, Cmd on Mac
    customizable: true,
  },
  {
    id: ShortcutAction.FocusPreviousPanel,
    label: "Focus Previous Panel",
    category: "Navigation",
    defaultKey: "ArrowLeft",
    defaultModifiers: { ctrlKey: true }, // Ctrl on Windows/Linux, Cmd on Mac
    customizable: true,
  },
  {
    id: ShortcutAction.OpenWorkspace,
    label: "Open Terminal Workspace",
    category: "Navigation",
    defaultKey: "`",
    defaultModifiers: { ctrlKey: true }, // Ctrl+` on Windows/Linux, Cmd+` on Mac
    customizable: true,
  },
  {
    id: ShortcutAction.OpenFileBrowser,
    label: "Open File Browser",
    category: "Navigation",
    defaultKey: "f",
    defaultModifiers: { ctrlKey: true, shiftKey: true }, // Ctrl+Shift on Windows/Linux, Cmd+Shift on Mac
    customizable: true,
  },
  {
    id: ShortcutAction.OpenSSHProfiles,
    label: "Open SSH Profiles",
    category: "Navigation",
    defaultKey: "s",
    defaultModifiers: { ctrlKey: true, shiftKey: true }, // Ctrl+Shift on Windows/Linux, Cmd+Shift on Mac
    customizable: true,
  },
  {
    id: ShortcutAction.OpenSavedCommands,
    label: "Open Saved Commands",
    category: "Navigation",
    defaultKey: "c",
    defaultModifiers: { ctrlKey: true, altKey: true }, // Ctrl+Alt on Windows/Linux, Cmd+Alt on Mac
    customizable: true,
  },
];

/**
 * Get default shortcut by action ID
 */
export function getDefaultShortcut(
  action: ShortcutAction,
): KeyboardShortcut | undefined {
  return DEFAULT_SHORTCUTS.find((s) => s.id === action);
}

/**
 * Get shortcuts grouped by category
 */
export function getShortcutsByCategory(): Record<string, KeyboardShortcut[]> {
  const grouped: Record<string, KeyboardShortcut[]> = {};

  for (const shortcut of DEFAULT_SHORTCUTS) {
    if (!grouped[shortcut.category]) {
      grouped[shortcut.category] = [];
    }
    grouped[shortcut.category].push(shortcut);
  }

  return grouped;
}
