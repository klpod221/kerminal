/*
 * Kerminal - Modern Terminal Emulator & SSH Manager
 * Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

import { check, Update } from "@tauri-apps/plugin-updater";
import { invoke } from "@tauri-apps/api/core";
import { api } from "./api";
import { version } from "../../package.json";

export interface UpdateInfo {
  version: string;
  date: string;
  body: string;
  currentVersion: string;
}

export interface LinuxUpdateInfo {
  available: boolean;
  version?: string;
  url?: string;
}

export interface TauriUpdateInfo {
  version: string;
  date?: string;
  body?: string;
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
  onProgress?: (progress: UpdateProgress) => void,
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
      "https://api.github.com/repos/klpod221/kerminal/releases/latest",
    );
    if (!response.ok) return null;

    const latestRelease = await response.json();
    const latestVersion = latestRelease.tag_name;

    // Get current version from package
    const currentVersion = `v${version}`;

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

/**
 * Listen to update available events
 */
export async function listenToUpdateEvents(
  callback: (data: LinuxUpdateInfo | TauriUpdateInfo) => void,
): Promise<() => void> {
  return await api.listen<LinuxUpdateInfo | TauriUpdateInfo>(
    "update-available",
    callback,
  );
}
