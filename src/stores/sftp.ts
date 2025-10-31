import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  FileEntry,
  SFTPSession,
  SFTPBrowserState,
  SyncOperation,
  DiffEntry,
} from "../types/sftp";
import * as sftpService from "../services/sftp";
import { api } from "../services/api";
import { useSSHStore } from "./ssh";
import { readDir, stat } from "@tauri-apps/plugin-fs";

/**
 * SFTP Store
 * Manages SFTP sessions, file browsing, and transfers
 */
export const useSFTPStore = defineStore("sftp", () => {
  // State
  const sessions = ref<Map<string, SFTPSession>>(new Map());
  const activeSessionId = ref<string | null>(null);
  const connecting = ref<boolean>(false);
  const browserState = ref<SFTPBrowserState>({
    activeSessionId: null,
    localPath: "",
    remotePath: "",
    localFiles: [],
    remoteFiles: [],
    selectedLocalFiles: new Set(),
    selectedRemoteFiles: new Set(),
    loading: {
      local: false,
      remote: false,
    },
    activeTransfers: new Map(),
  });

  // Computed
  const activeSession = computed(() => {
    if (!activeSessionId.value) return null;
    return sessions.value.get(activeSessionId.value) || null;
  });

  const activeTransfers = computed(() => {
    // Filter out cancelled transfers
    return Array.from(browserState.value.activeTransfers.values()).filter(
      (t) => t.status !== "cancelled",
    );
  });

  // Actions
  /**
   * Connect to SFTP server
   */
  async function connect(profileId: string): Promise<string> {
    connecting.value = true;
    try {
      const sessionId = await sftpService.connectSFTP(profileId);
      const sshStore = useSSHStore();
      const profile = sshStore.profiles.find((p) => p.id === profileId);

      const session: SFTPSession = {
        sessionId,
        profileId,
        profileName: profile?.name || "Unknown",
        connectedAt: new Date().toISOString(),
        lastUsed: new Date().toISOString(),
      };

      sessions.value.set(sessionId, session);
      activeSessionId.value = sessionId;
      browserState.value.activeSessionId = sessionId;

      // Start listening to transfer events
      startRealtime();

      return sessionId;
    } catch (error) {
      console.error("Failed to connect SFTP:", error);
      throw error;
    } finally {
      connecting.value = false;
    }
  }

  /**
   * Disconnect SFTP session
   */
  async function disconnect(sessionId: string): Promise<void> {
    try {
      await sftpService.disconnectSFTP(sessionId);
      sessions.value.delete(sessionId);

      if (activeSessionId.value === sessionId) {
        activeSessionId.value = null;
        browserState.value.activeSessionId = null;
        browserState.value.localFiles = [];
        browserState.value.remoteFiles = [];
      }
    } catch (error) {
      console.error("Failed to disconnect SFTP:", error);
      throw error;
    }
  }

  /**
   * List local directory
   */
  async function listLocalDirectory(path: string): Promise<void> {
    // Clear selection when navigating to a different directory
    if (browserState.value.localPath !== path) {
      browserState.value.selectedLocalFiles.clear();
    }

    browserState.value.loading.local = true;
    try {
      // Use Tauri fs plugin to read local directory
      const entries = await readDir(path);

      // Use Promise.allSettled to handle individual file errors gracefully
      const fileResults = await Promise.allSettled(
        entries.map(async (entry) => {
          // DirEntry doesn't have path property, construct it
          // Normalize path to avoid double slashes
          const normalizedPath = path.endsWith("/") ? path.slice(0, -1) : path;
          const entryPath = normalizedPath === "/"
            ? `/${entry.name}`
            : `${normalizedPath}/${entry.name}`;

          try {
            // Use stat to get metadata
            const meta = await stat(entryPath);

            let fileType: FileEntry["fileType"] = "file";
            if (meta.isDirectory) {
              fileType = "directory";
            } else if (entry.isSymlink || meta.isSymlink) {
              fileType = "symlink";
            }

            // Get permissions (Unix only)
            let permissions = 0o644;
            if (meta.mode) {
              permissions = meta.mode & 0o777;
            }

            return {
              name: entry.name,
              path: entryPath,
              fileType,
              size: fileType === "file" ? meta.size || null : null,
              permissions,
              modified: new Date(meta.mtime || Date.now()).toISOString(),
              accessed: meta.atime
                ? new Date(meta.atime).toISOString()
                : null,
              symlinkTarget: null, // Would need readlink to get this
              uid: null,
              gid: null,
            };
          } catch (error) {
            // Skip files that can't be stat'd (broken symlinks, deleted files, etc.)
            console.warn(`Failed to get metadata for ${entryPath}:`, error);
            return null;
          }
        }),
      );

      // Filter out null results (failed entries)
      const files: FileEntry[] = [];
      for (const result of fileResults) {
        if (result.status === "fulfilled" && result.value !== null) {
          files.push(result.value);
        } else if (result.status === "rejected") {
          console.warn("Failed to process directory entry:", result.reason);
        }
      }

      browserState.value.localFiles = files;
      browserState.value.localPath = path;
    } catch (error) {
      console.error("Failed to list local directory:", error);
      throw error;
    } finally {
      browserState.value.loading.local = false;
    }
  }

  /**
   * List remote directory
   */
  async function listRemoteDirectory(
    sessionId: string,
    path: string,
): Promise<void> {
    // Clear selection when navigating to a different directory
    if (browserState.value.remotePath !== path) {
      browserState.value.selectedRemoteFiles.clear();
    }

    browserState.value.loading.remote = true;
    try {
      const files = await sftpService.listSFTPDirectory(sessionId, path);
      browserState.value.remoteFiles = files;
      browserState.value.remotePath = path;

      // Update session last used
      const session = sessions.value.get(sessionId);
      if (session) {
        session.lastUsed = new Date().toISOString();
      }
    } catch (error) {
      console.error("Failed to list remote directory:", error);
      throw error;
    } finally {
      browserState.value.loading.remote = false;
    }
  }

  /**
   * Upload file
   */
  async function uploadFile(
    sessionId: string,
    localPath: string,
    remotePath: string,
  ): Promise<string> {
    try {
      const transferId = await sftpService.uploadSFTPFile(
        sessionId,
        localPath,
        remotePath,
      );
      // Initial transfer entry will be created when we get progress event
      return transferId;
    } catch (error) {
      console.error("Failed to upload file:", error);
      throw error;
    }
  }

  /**
   * Download file
   */
  async function downloadFile(
    sessionId: string,
    remotePath: string,
    localPath: string,
  ): Promise<string> {
    try {
      const transferId = await sftpService.downloadSFTPFile(
        sessionId,
        remotePath,
        localPath,
      );
      return transferId;
    } catch (error) {
      console.error("Failed to download file:", error);
      throw error;
    }
  }

  /**
   * Cancel transfer
   */
  async function cancelTransfer(transferId: string): Promise<void> {
    try {
      await sftpService.cancelSFTPTransfer(transferId);
      // Remove transfer from active transfers immediately
      browserState.value.activeTransfers.delete(transferId);
    } catch (error) {
      console.error("Failed to cancel transfer:", error);
      throw error;
    }
  }


  /**
   * Compare directories
   */
  async function compareDirectories(
    sessionId: string,
    localPath: string,
    remotePath: string,
  ): Promise<DiffEntry[]> {
    try {
      return await sftpService.compareSFTPDirectories(
        sessionId,
        localPath,
        remotePath,
      );
    } catch (error) {
      console.error("Failed to compare directories:", error);
      throw error;
    }
  }

  /**
   * Sync directories
   */
  async function syncDirectories(
    sessionId: string,
    operation: SyncOperation,
  ): Promise<void> {
    try {
      await sftpService.syncSFTPDirectories(sessionId, operation);
    } catch (error) {
      console.error("Failed to sync directories:", error);
      throw error;
    }
  }

  /**
   * Rename file or directory
   */
  async function renameFile(
    sessionId: string,
    oldPath: string,
    newPath: string,
  ): Promise<void> {
    try {
      await sftpService.renameSFTP(sessionId, oldPath, newPath);
      // Refresh current directory
      if (browserState.value.remotePath) {
        await listRemoteDirectory(sessionId, browserState.value.remotePath);
      }
    } catch (error) {
      console.error("Failed to rename file:", error);
      throw error;
    }
  }

  /**
   * Delete file or directory
   */
  async function deleteFile(
    sessionId: string,
    path: string,
    recursive: boolean = false,
  ): Promise<void> {
    try {
      await sftpService.deleteSFTP(sessionId, path, recursive);
      // Refresh current directory
      if (browserState.value.remotePath) {
        await listRemoteDirectory(sessionId, browserState.value.remotePath);
      }
    } catch (error) {
      console.error("Failed to delete file:", error);
      throw error;
    }
  }

  /**
   * Set file permissions
   */
  async function setPermissions(
    sessionId: string,
    path: string,
    mode: number,
  ): Promise<void> {
    try {
      await sftpService.setSFTPPermissions(sessionId, path, mode);
      // Refresh current directory
      if (browserState.value.remotePath) {
        await listRemoteDirectory(sessionId, browserState.value.remotePath);
      }
    } catch (error) {
      console.error("Failed to set permissions:", error);
      throw error;
    }
  }

  /**
   * Create directory
   */
  async function createDirectory(
    sessionId: string,
    path: string,
  ): Promise<void> {
    try {
      await sftpService.createSFTPDirectory(sessionId, path);
      // Refresh current directory
      if (browserState.value.remotePath) {
        await listRemoteDirectory(sessionId, browserState.value.remotePath);
      }
    } catch (error) {
      console.error("Failed to create directory:", error);
      throw error;
    }
  }

  let unsubscribeTransferRealtime: (() => void) | null = null;

  /**
   * Start listening to realtime events
   */
  async function startRealtime(): Promise<void> {
    if (unsubscribeTransferRealtime) return;
    try {
      // Listen to transfer progress
      const u1 = await api.listen<{
        transferId: string;
        transferredBytes: number;
        totalBytes: number;
      }>("sftp_transfer_progress", async (data) => {
        let existing = browserState.value.activeTransfers.get(data.transferId);
        if (!existing) {
          // Fetch transfer progress if not in map yet
          try {
            existing = await sftpService.getSFTPTransferProgress(data.transferId);
            browserState.value.activeTransfers.set(data.transferId, existing);
          } catch (error) {
            console.error("Failed to get transfer progress:", error);
            return;
          }
        }

        existing.transferredBytes = data.transferredBytes;
        // Update speed and ETA (simplified)
        const now = Date.now();
        const startTime = new Date(existing.startedAt).getTime();
        const elapsed = (now - startTime) / 1000;
        if (elapsed > 0) {
          existing.speedBytesPerSec = Math.round(
            data.transferredBytes / elapsed,
          );
          if (existing.speedBytesPerSec > 0) {
            const remaining = data.totalBytes - data.transferredBytes;
            existing.etaSeconds = Math.round(
              remaining / existing.speedBytesPerSec,
            );
          }
        }
      });

      // Listen to transfer complete
      const u2 = await api.listen<{ transferId: string }>(
        "sftp_transfer_complete",
        async (data) => {
          try {
            const progress = await sftpService.getSFTPTransferProgress(
              data.transferId,
            );
            browserState.value.activeTransfers.set(data.transferId, progress);
          } catch (error) {
            console.error("Failed to get transfer progress:", error);
          }
        },
      );

      // Listen to transfer errors
      const u3 = await api.listen<{ transferId: string; error: string }>(
        "sftp_transfer_error",
        async (data) => {
          try {
            const progress = await sftpService.getSFTPTransferProgress(
              data.transferId,
            );
            progress.error = data.error;
            browserState.value.activeTransfers.set(data.transferId, progress);
          } catch (error) {
            console.error("Failed to get transfer progress:", error);
          }
        },
      );

      unsubscribeTransferRealtime = () => {
        u1();
        u2();
        u3();
      };
    } catch (error) {
      console.error("Failed to start SFTP realtime:", error);
    }
  }

  /**
   * Stop listening to realtime events
   */
  function stopRealtime(): void {
    if (unsubscribeTransferRealtime) {
      unsubscribeTransferRealtime();
      unsubscribeTransferRealtime = null;
    }
  }

  return {
    // State
    sessions,
    activeSessionId,
    connecting,
    browserState,
    // Computed
    activeSession,
    activeTransfers,
    // Actions
    connect,
    disconnect,
    listLocalDirectory,
    listRemoteDirectory,
    uploadFile,
    downloadFile,
    cancelTransfer,
    compareDirectories,
    syncDirectories,
    renameFile,
    deleteFile,
    setPermissions,
    createDirectory,
    startRealtime,
    stopRealtime,
  };
});

