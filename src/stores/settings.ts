import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { getAvailableThemes } from "../utils/terminalTheme";
import { Store } from "@tauri-apps/plugin-store";
import type { TerminalTheme } from "../utils/terminalTheme";

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
    ...customThemes.value.map(t => t.name)
  ]);

  // Load settings from Tauri store
  const loadSettings = async () => {
    try {
      isLoading.value = true;
      const storeInstance = await initStore();

      // Load custom themes
      const savedCustomThemes = await storeInstance.get<CustomTheme[]>("custom-themes");
      if (savedCustomThemes) {
        customThemes.value = savedCustomThemes;
      }

      // Load selected theme
      const savedTheme = await storeInstance.get<string>("terminal-theme");
      if (savedTheme) {
        terminalTheme.value = savedTheme;
      }

      // Load font settings
      const savedFontFamily = await storeInstance.get<string>("font-family");
      if (savedFontFamily) {
        fontFamily.value = savedFontFamily;
      }

      const savedFontSize = await storeInstance.get<number>("font-size");
      if (savedFontSize) {
        fontSize.value = savedFontSize;
      }
    } catch (error) {
      console.error("Failed to load settings from Tauri store:", error);
    } finally {
      isLoading.value = false;
    }
  };

  // Save settings to Tauri store
  const saveSettings = async () => {
    try {
      const storeInstance = await initStore();
      await storeInstance.set("terminal-theme", terminalTheme.value);
      await storeInstance.set("font-family", fontFamily.value);
      await storeInstance.set("font-size", fontSize.value);
      await storeInstance.save();
    } catch (error) {
      console.error("Failed to save settings to Tauri store:", error);
    }
  };

  // Set terminal theme
  const setTerminalTheme = async (theme: string) => {
    terminalTheme.value = theme;
    await saveSettings();
  };

  // Create custom theme
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

  // Update custom theme
  const updateCustomTheme = async (id: string, name: string, colors: TerminalTheme) => {
    const theme = customThemes.value.find(t => t.id === id);
    if (theme) {
      theme.name = name;
      theme.colors = colors;
      theme.updatedAt = new Date().toISOString();
      await saveCustomThemes();

      // If this is the current theme, trigger a refresh
      if (terminalTheme.value === name) {
        terminalTheme.value = name;
      }
    }
  };

  // Delete custom theme
  const deleteCustomTheme = async (id: string) => {
    const theme = customThemes.value.find(t => t.id === id);
    if (theme) {
      customThemes.value = customThemes.value.filter(t => t.id !== id);
      await saveCustomThemes();

      // If this was the current theme, switch to default
      if (terminalTheme.value === theme.name) {
        terminalTheme.value = "Default";
        await saveSettings();
      }
    }
  };

  // Get custom theme by name
  const getCustomTheme = (name: string): CustomTheme | undefined => {
    return customThemes.value.find(t => t.name === name);
  };

  // Check if theme is custom
  const isCustomTheme = (name: string): boolean => {
    return customThemes.value.some(t => t.name === name);
  };

  // Save custom themes to store
  const saveCustomThemes = async () => {
    try {
      const storeInstance = await initStore();
      await storeInstance.set("custom-themes", customThemes.value);
      await storeInstance.save();
    } catch (error) {
      console.error("Failed to save custom themes:", error);
    }
  };

  // Set font family
  const setFontFamily = async (font: string) => {
    fontFamily.value = font;
    await saveSettings();
  };

  // Set font size
  const setFontSize = async (size: number) => {
    fontSize.value = size;
    await saveSettings();
  };

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
  };
});

