import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { getAvailableThemes } from "../utils/terminalTheme";
import { Store } from "@tauri-apps/plugin-store";
import { api } from "../services/api";
import * as settingsService from "../services/settings";
import { getSystemInfo as getSystemInfoService } from "../services/dashboard";
import type { TerminalTheme } from "../utils/terminalTheme";
import { handleError, type ErrorContext } from "../utils/errorHandler";
import { message } from "../utils/message";

export interface CustomTheme {
  id: string;
  name: string;
  colors: TerminalTheme;
  createdAt: string;
  updatedAt: string;
}

// Tauri store instance
let store: Store | null = null;

// Initialize Tauri store
const initStore = async () => {
  if (!store) {
    store = await Store.load("settings.json");
  }
  return store;
};

/**
 * Settings Store
 * Manages user preferences and application settings
 */
export const useSettingsStore = defineStore("settings", () => {
  // Terminal theme preference
  const terminalTheme = ref<string>("Default");
  const customThemes = ref<CustomTheme[]>([]);
  const isLoading = ref(false);

  // Terminal font preferences
  const fontFamily = ref<string>("FiraCode Nerd Font");
  const fontSize = ref<number>(13);

  // Available built-in themes
  const builtInThemes = getAvailableThemes();

  // All available themes (built-in + custom)
  const availableThemes = computed(() => [
    ...builtInThemes,
    ...customThemes.value.map((t) => t.name),
  ]);

  /**
   * Load settings from Tauri store with error handling
   */
  const loadSettings = async () => {
    const context: ErrorContext = {
      operation: "Load Settings",
    };

    try {
      isLoading.value = true;
      const storeInstance = await initStore();

      const savedCustomThemes =
        await storeInstance.get<CustomTheme[]>("custom-themes");
      if (savedCustomThemes) {
        customThemes.value = savedCustomThemes;
      }

      const savedTheme = await storeInstance.get<string>("terminal-theme");
      if (savedTheme) {
        terminalTheme.value = savedTheme;
      }

      const savedFontFamily = await storeInstance.get<string>("font-family");
      if (savedFontFamily) {
        fontFamily.value = savedFontFamily;
      }

      const savedFontSize = await storeInstance.get<number>("font-size");
      if (savedFontSize) {
        fontSize.value = savedFontSize;
      }
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Save settings to Tauri store with error handling
   */
  const saveSettings = async () => {
    const context: ErrorContext = {
      operation: "Save Settings",
    };

    try {
      const storeInstance = await initStore();
      await storeInstance.set("terminal-theme", terminalTheme.value);
      await storeInstance.set("font-family", fontFamily.value);
      await storeInstance.set("font-size", fontSize.value);
      await storeInstance.save();
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
    }
  };

  const setTerminalTheme = async (theme: string) => {
    terminalTheme.value = theme;
    await saveSettings();
  };

  const createCustomTheme = async (name: string, colors: TerminalTheme) => {
    const newTheme: CustomTheme = {
      id: `custom-${Date.now()}`,
      name,
      colors,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    customThemes.value.push(newTheme);
    await saveCustomThemes();
    return newTheme;
  };

  const updateCustomTheme = async (
    id: string,
    name: string,
    colors: TerminalTheme,
  ) => {
    const theme = customThemes.value.find((t) => t.id === id);
    if (theme) {
      theme.name = name;
      theme.colors = colors;
      theme.updatedAt = new Date().toISOString();
      await saveCustomThemes();

      if (terminalTheme.value === name) {
        terminalTheme.value = name;
      }
    }
  };

  const deleteCustomTheme = async (id: string) => {
    const theme = customThemes.value.find((t) => t.id === id);
    if (theme) {
      customThemes.value = customThemes.value.filter((t) => t.id !== id);
      await saveCustomThemes();

      if (terminalTheme.value === theme.name) {
        terminalTheme.value = "Default";
        await saveSettings();
      }
    }
  };

  const getCustomTheme = (name: string): CustomTheme | undefined => {
    return customThemes.value.find((t) => t.name === name);
  };

  const isCustomTheme = (name: string): boolean => {
    return customThemes.value.some((t) => t.name === name);
  };

  const saveCustomThemes = async () => {
    try {
      const storeInstance = await initStore();
      await storeInstance.set("custom-themes", customThemes.value);
      await storeInstance.save();
    } catch (error) {
      console.error("Failed to save custom themes:", error);
    }
  };

  const setFontFamily = async (font: string) => {
    fontFamily.value = font;
    await saveSettings();
  };

  const setFontSize = async (size: number) => {
    fontSize.value = size;
    await saveSettings();
  };

  const upsertCustomTheme = (theme: CustomTheme) => {
    if (!theme?.id) return;
    const i = customThemes.value.findIndex((t) => t.id === theme.id);
    if (i === -1) {
      customThemes.value = [...customThemes.value, theme];
    } else {
      customThemes.value[i] = { ...customThemes.value[i]!, ...theme };
    }
  };

  const removeCustomTheme = (id: string) => {
    const theme = customThemes.value.find((t) => t.id === id);
    if (theme) {
      customThemes.value = customThemes.value.filter((t) => t.id !== id);
      if (terminalTheme.value === theme.name) {
        terminalTheme.value = "Default";
        saveSettings();
      }
    }
  };

  let unsubscribeThemeRealtime: (() => void) | null = null;

  const startRealtime = async (): Promise<void> => {
    if (unsubscribeThemeRealtime) return;
    try {
      const u1 = await api.listen<CustomTheme>(
        "settings_custom_theme_created",
        (theme) => {
          upsertCustomTheme(theme);
          saveCustomThemes();
        },
      );
      const u2 = await api.listen<CustomTheme>(
        "settings_custom_theme_updated",
        (theme) => {
          upsertCustomTheme(theme);
          saveCustomThemes();
        },
      );
      const u3 = await api.listen<{ id: string }>(
        "settings_custom_theme_deleted",
        ({ id }) => {
          removeCustomTheme(id);
          saveCustomThemes();
        },
      );
      unsubscribeThemeRealtime = () => {
        u1();
        u2();
        u3();
      };
    } catch (e) {
      console.error("Failed to subscribe settings realtime events:", e);
    }
  };

  const stopRealtime = (): void => {
    if (unsubscribeThemeRealtime) {
      unsubscribeThemeRealtime();
      unsubscribeThemeRealtime = null;
    }
  };

  /**
   * Get system fonts with error handling
   * @returns Array of system font names
   */
  async function getSystemFonts(): Promise<string[]> {
    const context: ErrorContext = {
      operation: "Get System Fonts",
    };

    try {
      return await settingsService.getSystemFonts();
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  /**
   * Get system info with error handling
   * @returns System information
   */
  async function getSystemInfoFunc(): Promise<any> {
    const context: ErrorContext = {
      operation: "Get System Info",
    };

    try {
      return await getSystemInfoService();
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  // Initialize settings on store creation
  loadSettings();

  return {
    terminalTheme,
    customThemes,
    availableThemes,
    builtInThemes,
    fontFamily,
    fontSize,
    isLoading,
    setTerminalTheme,
    setFontFamily,
    setFontSize,
    loadSettings,
    createCustomTheme,
    updateCustomTheme,
    deleteCustomTheme,
    getCustomTheme,
    isCustomTheme,
    getSystemFonts,
    getSystemInfo: getSystemInfoFunc,

    startRealtime,
    stopRealtime,
  };
});
