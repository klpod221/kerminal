import { check, Update } from "@tauri-apps/plugin-updater";
import { invoke } from "@tauri-apps/api/core";

export interface UpdateInfo {
  version: string;
  date: string;
  body: string;
  currentVersion: string;
}

export interface UpdateProgress {
  downloaded: number;
  total: number;
  percentage: number;
}

/**
 * Check if an update is available
 * @returns Update object if available, null otherwise
 */
export async function checkForUpdates(): Promise<Update | null> {
  try {
    const update = await check();
    return update || null;
  } catch (error) {
    console.error("Failed to check for updates:", error);
    return null;
  }
}

/**
 * Get current platform using navigator
 */
export function getPlatform(): string {
  const userAgent = navigator.userAgent.toLowerCase();
  if (userAgent.includes("win")) return "windows";
  if (userAgent.includes("mac")) return "macos";
  if (userAgent.includes("linux")) return "linux";
  return "unknown";
}

/**
 * Download and install an update
 * @param update The update to install
 * @param onProgress Callback for progress updates
 */
export async function downloadAndInstall(
  update: Update,
  onProgress?: (progress: UpdateProgress) => void
): Promise<void> {
  let downloaded = 0;
  let total = 0;

  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case "Started":
        total = event.data.contentLength || 0;
        if (onProgress) {
          onProgress({
            downloaded: 0,
            total,
            percentage: 0,
          });
        }
        break;
      case "Progress":
        downloaded += event.data.chunkLength;
        if (onProgress) {
          onProgress({
            downloaded,
            total,
            percentage: total > 0 ? (downloaded / total) * 100 : 0,
          });
        }
        break;
      case "Finished":
        if (onProgress) {
          onProgress({
            downloaded: total,
            total,
            percentage: 100,
          });
        }
        break;
    }
  });
}

/**
 * Restart the application
 */
export async function restartApp(): Promise<void> {
  // Use the process plugin through invoke
  await invoke("plugin:process|restart");
}

/**
 * Check for updates on Linux and return GitHub release URL if available
 */
export async function checkLinuxUpdate(): Promise<{
  available: boolean;
  version?: string;
  url?: string;
} | null> {
  try {
    const response = await fetch(
      "https://api.github.com/repos/klpod221/kerminal/releases/latest"
    );
    if (!response.ok) return null;

    const latestRelease = await response.json();
    const latestVersion = latestRelease.tag_name;

    // Get current version from package
    const currentVersion = "v2.5.1"; // This should be dynamically loaded

    if (latestVersion !== currentVersion) {
      return {
        available: true,
        version: latestVersion,
        url: latestRelease.html_url,
      };
    }

    return { available: false };
  } catch (error) {
    console.error("Failed to check Linux updates:", error);
    return null;
  }
}
