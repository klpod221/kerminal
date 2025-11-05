/**
 * Keyboard shortcut action types
 * Each action represents a specific functionality in the application
 */
export enum ShortcutAction {
  // Terminal splitting
  SplitVertical = "splitVertical",
  SplitHorizontal = "splitHorizontal",

  // History
  HistorySearch = "historySearch",

  // Terminal operations
  NewTab = "newTab",
  CloseTab = "closeTab",
  NextTab = "nextTab",

  // Navigation
  FocusNextPanel = "focusNextPanel",
  FocusPreviousPanel = "focusPreviousPanel",
  OpenWorkspace = "openWorkspace",

  // File operations
  OpenFileBrowser = "openFileBrowser",
  OpenSSHProfiles = "openSSHProfiles",
  OpenSavedCommands = "openSavedCommands",
}

/**
 * Keyboard modifier keys
 */
export interface ShortcutModifiers {
  ctrlKey?: boolean;
  altKey?: boolean;
  shiftKey?: boolean;
  metaKey?: boolean; // Cmd on Mac, Windows key on Windows
}

/**
 * Keyboard shortcut configuration
 */
export interface KeyboardShortcut {
  /** Unique identifier for the shortcut */
  id: ShortcutAction;

  /** Human-readable label/description */
  label: string;

  /** Category for grouping in UI */
  category: string;

  /** Default key binding */
  defaultKey: string;

  /** Default modifier keys */
  defaultModifiers: ShortcutModifiers;

  /** Custom key binding (user-defined, optional) */
  customKey?: string;

  /** Custom modifier keys (user-defined, optional) */
  customModifiers?: ShortcutModifiers;

  /** Whether this shortcut can be customized */
  customizable: boolean;
}

/**
 * Active keyboard shortcut (merged defaults + custom)
 */
export interface ActiveShortcut {
  id: ShortcutAction;
  key: string;
  modifiers: ShortcutModifiers;
  label: string;
  category: string;
}

/**
 * Shortcut conflict information
 */
export interface ShortcutConflict {
  action: ShortcutAction;
  conflictingAction: ShortcutAction;
  key: string;
  modifiers: ShortcutModifiers;
}

/**
 * Format shortcut to display string
 */
export function formatShortcut(
  shortcut: ActiveShortcut | KeyboardShortcut,
): string {
  let modifiers: ShortcutModifiers;
  let key: string;

  if ("defaultKey" in shortcut) {
    // KeyboardShortcut
    modifiers = shortcut.customModifiers || shortcut.defaultModifiers;
    key = shortcut.customKey || shortcut.defaultKey;
  } else {
    // ActiveShortcut
    modifiers = shortcut.modifiers;
    key = shortcut.key;
  }

  const parts: string[] = [];

  if (modifiers.ctrlKey) {
    parts.push(navigator.platform.includes("Mac") ? "⌃" : "Ctrl");
  }
  if (modifiers.altKey) {
    parts.push(navigator.platform.includes("Mac") ? "⌥" : "Alt");
  }
  if (modifiers.shiftKey) {
    parts.push(navigator.platform.includes("Mac") ? "⇧" : "Shift");
  }
  if (modifiers.metaKey) {
    parts.push(navigator.platform.includes("Mac") ? "⌘" : "Win");
  }

  parts.push(key.toUpperCase());

  return parts.join(navigator.platform.includes("Mac") ? "" : "+");
}

/**
 * Check if two shortcuts conflict
 */
export function checkShortcutConflict(
  shortcut1: { key: string; modifiers: ShortcutModifiers },
  shortcut2: { key: string; modifiers: ShortcutModifiers },
): boolean {
  if (shortcut1.key.toLowerCase() !== shortcut2.key.toLowerCase()) {
    return false;
  }

  return (
    !!shortcut1.modifiers.ctrlKey === !!shortcut2.modifiers.ctrlKey &&
    !!shortcut1.modifiers.altKey === !!shortcut2.modifiers.altKey &&
    !!shortcut1.modifiers.shiftKey === !!shortcut2.modifiers.shiftKey &&
    !!shortcut1.modifiers.metaKey === !!shortcut2.modifiers.metaKey
  );
}
