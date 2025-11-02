import { api } from "./api";

/**
 * Get system fonts
 * @returns Array of system font names
 */
export async function getSystemFonts(): Promise<string[]> {
  return await api.callRaw("get_system_fonts");
}
