import {
  TERMINAL_THEMES,
  getAvailableThemeNames,
  isBuiltInTheme as isBuiltInThemeConfig,
  type TerminalTheme,
} from "../config/terminalThemes";

// Re-export type for backward compatibility
export type { TerminalTheme };

export function getTerminalTheme(
  themeName: keyof typeof TERMINAL_THEMES | string = "Default",
  customTheme?: TerminalTheme,
): TerminalTheme {
  // If custom theme is provided, use it
  if (customTheme) {
    return customTheme;
  }

  // Otherwise, look up in built-in themes
  return (
    TERMINAL_THEMES[themeName as keyof typeof TERMINAL_THEMES] ||
    TERMINAL_THEMES["Default"]
  );
}

export function getAvailableThemes(): (keyof typeof TERMINAL_THEMES)[] {
  return getAvailableThemeNames();
}

export function isBuiltInTheme(themeName: string): boolean {
  return isBuiltInThemeConfig(themeName);
}
