import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { Store } from "@tauri-apps/plugin-store";
import type {
  KeyboardShortcut,
  ActiveShortcut,
  ShortcutConflict,
  ShortcutModifiers,
} from "../types/shortcuts";
import { ShortcutAction, checkShortcutConflict } from "../types/shortcuts";
import { DEFAULT_SHORTCUTS } from "../config/shortcuts";
import { handleError, type ErrorContext } from "../utils/errorHandler";
import { message } from "../utils/message";

// Tauri store instance
let store: Store | null = null;

// Initialize Tauri store
const initStore = async () => {
  store ??= await Store.load("settings.json");
  return store;
};

/**
 * Custom shortcuts data structure for storage
 */
interface CustomShortcutsData {
  [key: string]: {
    key?: string;
    modifiers?: ShortcutModifiers;
  };
}

/**
 * Keyboard Shortcuts Store
 * Manages keyboard shortcut configurations and customizations
 */
export const useKeyboardShortcutsStore = defineStore(
  "keyboardShortcuts",
  () => {
    const shortcuts = ref<Map<ShortcutAction, KeyboardShortcut>>(new Map());
    const isLoading = ref(false);
    const conflicts = ref<ShortcutConflict[]>([]);

    /**
     * Initialize shortcuts map from defaults
     */
    const initializeDefaults = () => {
      shortcuts.value.clear();
      for (const shortcut of DEFAULT_SHORTCUTS) {
        shortcuts.value.set(shortcut.id, { ...shortcut });
      }
    };

    /**
     * Normalize modifiers based on platform and custom data
     */
    const normalizeModifiers = (
      customModifiers: ShortcutModifiers,
    ): ShortcutModifiers => {
      // Normalize modifiers: only keep one of Ctrl or Meta
      const isMac = navigator.userAgent.includes("Mac");
      return {
        ctrlKey: isMac ? false : (customModifiers.ctrlKey ?? false),
        altKey: customModifiers.altKey ?? false,
        shiftKey: customModifiers.shiftKey ?? false,
        metaKey: isMac ? (customModifiers.metaKey ?? false) : false,
      };
    };

    /**
     * Apply custom shortcuts from storage
     */
    const applyCustomShortcuts = (savedShortcuts: CustomShortcutsData) => {
      for (const [actionId, customData] of Object.entries(savedShortcuts)) {
        const action = actionId as ShortcutAction;
        const shortcut = shortcuts.value.get(action);

        if (shortcut && customData) {
          if (customData.key) {
            shortcut.customKey = customData.key;
          }
          if (customData.modifiers) {
            shortcut.customModifiers = normalizeModifiers(customData.modifiers);
          }
        }
      }
    };

    /**
     * Load shortcuts from Tauri store
     */
    const loadShortcuts = async () => {
      const context: ErrorContext = {
        operation: "Load Keyboard Shortcuts",
      };

      try {
        isLoading.value = true;

        // Initialize with defaults first
        initializeDefaults();

        const storeInstance = await initStore();
        const savedShortcuts =
          await storeInstance.get<CustomShortcutsData>("keyboard-shortcuts");

        if (savedShortcuts) {
          applyCustomShortcuts(savedShortcuts);
        }

        // Detect conflicts after loading
        detectConflicts();
      } catch (error) {
        const errorMessage = handleError(error, context);
        message.error(errorMessage);
      } finally {
        isLoading.value = false;
      }
    };

    /**
     * Save shortcuts to Tauri store
     */
    const saveShortcuts = async () => {
      const context: ErrorContext = {
        operation: "Save Keyboard Shortcuts",
      };

      try {
        const storeInstance = await initStore();
        const customShortcuts: CustomShortcutsData = {};

        // Only save custom shortcuts (those that differ from defaults)
        for (const [action, shortcut] of shortcuts.value.entries()) {
          if (shortcut.customKey || shortcut.customModifiers) {
            customShortcuts[action] = {
              key: shortcut.customKey,
              modifiers: shortcut.customModifiers,
            };
          }
        }

        await storeInstance.set("keyboard-shortcuts", customShortcuts);
        await storeInstance.save();

        // Re-detect conflicts after saving
        detectConflicts();
      } catch (error) {
        const errorMessage = handleError(error, context);
        message.error(errorMessage);
      }
    };

    /**
     * Get active shortcut (merged default + custom)
     */
    const getActiveShortcut = (
      action: ShortcutAction,
    ): ActiveShortcut | null => {
      const shortcut = shortcuts.value.get(action);
      if (!shortcut) return null;

      return {
        id: shortcut.id,
        key: shortcut.customKey || shortcut.defaultKey,
        modifiers: shortcut.customModifiers || shortcut.defaultModifiers,
        label: shortcut.label,
        category: shortcut.category,
      };
    };

    /**
     * Get all active shortcuts
     */
    const activeShortcuts = computed(() => {
      const active: ActiveShortcut[] = [];
      for (const [action] of shortcuts.value.entries()) {
        const activeShortcut = getActiveShortcut(action);
        if (activeShortcut) {
          active.push(activeShortcut);
        }
      }
      return active;
    });

    /**
     * Update a shortcut's custom binding
     */
    const updateShortcut = async (
      action: ShortcutAction,
      key: string,
      modifiers: ShortcutModifiers,
    ) => {
      const shortcut = shortcuts.value.get(action);
      if (!shortcut?.customizable) {
        return;
      }

      shortcut.customKey = key;
      shortcut.customModifiers = modifiers;

      // Check for conflicts before saving
      detectConflicts();

      // If there's a conflict with this specific action, warn but still save
      const hasConflict = conflicts.value.some(
        (c) => c.action === action || c.conflictingAction === action,
      );

      if (hasConflict) {
        message.warning(
          "Shortcut conflict detected. Please resolve conflicts in settings.",
        );
      }

      await saveShortcuts();
    };

    /**
     * Reset a shortcut to default
     */
    const resetShortcut = async (action: ShortcutAction) => {
      const shortcut = shortcuts.value.get(action);
      if (!shortcut) return;

      shortcut.customKey = undefined;
      shortcut.customModifiers = undefined;

      await saveShortcuts();
      detectConflicts();
    };

    /**
     * Reset all shortcuts to defaults
     */
    const resetAllShortcuts = async () => {
      initializeDefaults();
      await saveShortcuts();
      detectConflicts();
      message.success("All shortcuts reset to defaults");
    };

    /**
     * Detect conflicts between shortcuts
     */
    const detectConflicts = () => {
      conflicts.value = [];
      const active = activeShortcuts.value;

      for (let i = 0; i < active.length; i++) {
        for (let j = i + 1; j < active.length; j++) {
          const shortcut1 = active[i];
          const shortcut2 = active[j];

          if (
            checkShortcutConflict(
              { key: shortcut1.key, modifiers: shortcut1.modifiers },
              { key: shortcut2.key, modifiers: shortcut2.modifiers },
            )
          ) {
            conflicts.value.push({
              action: shortcut1.id,
              conflictingAction: shortcut2.id,
              key: shortcut1.key,
              modifiers: shortcut1.modifiers,
            });
          }
        }
      }
    };

    /**
     * Get conflicts for a specific action
     */
    const getConflictsFor = (action: ShortcutAction): ShortcutConflict[] => {
      return conflicts.value.filter(
        (c) => c.action === action || c.conflictingAction === action,
      );
    };

    /**
     * Get shortcuts grouped by category
     */
    const shortcutsByCategory = computed(() => {
      const grouped: Record<string, KeyboardShortcut[]> = {};

      for (const shortcut of shortcuts.value.values()) {
        if (!grouped[shortcut.category]) {
          grouped[shortcut.category] = [];
        }
        grouped[shortcut.category].push(shortcut);
      }

      return grouped;
    });

    return {
      // State
      shortcuts: computed(() => shortcuts.value),
      activeShortcuts,
      shortcutsByCategory,
      conflicts: computed(() => conflicts.value),
      isLoading: computed(() => isLoading.value),

      // Actions
      loadShortcuts,
      saveShortcuts,
      getActiveShortcut,
      updateShortcut,
      resetShortcut,
      resetAllShortcuts,
      detectConflicts,
      getConflictsFor,
    };
  },
);
