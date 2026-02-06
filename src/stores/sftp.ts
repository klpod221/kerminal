import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  FileEntry,
  SFTPSession,
  SFTPBrowserState,
  SyncOperation,
  DiffEntry,
  SearchResult,
} from "../types/sftp";
import * as sftpService from "../services/sftp";
import { api } from "../services/api";
import { useSSHStore } from "./ssh";
import { readDir, stat } from "@tauri-apps/plugin-fs";
import {
  withRetry,
  handleError,
  type ErrorContext,
} from "../utils/errorHandler";
import { message } from "../utils/message";

/**
 * SFTP Store
 * Manages SFTP sessions, file browsing, and transfers
 */

// Helper function to process directory entries
async function processFileEntry(
  entry: any,
  path: string,
): Promise<FileEntry | null> {
  const normalizedPath = path.endsWith("/") ? path.slice(0, -1) : path;
  const entryPath =
    normalizedPath === "/"
      ? `/${entry.name}`
      : `${normalizedPath}/${entry.name}`;

  try {
    const meta = await stat(entryPath);

    let fileType: FileEntry["fileType"] = "file";
    if (meta.isDirectory) {
      fileType = "directory";
    } else if (entry.isSymlink || meta.isSymlink) {
      fileType = "symlink";
    }

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
      accessed: meta.atime ? new Date(meta.atime).toISOString() : null,
      symlinkTarget: null, // Would need readlink to get this
      uid: null,
      gid: null,
    };
  } catch (error) { // NOSONAR
    // Failed to get metadata (expected for non-existent/inaccessible files)
    return null;
  }
}

/**
 * Upload file with error handling
 * @param sessionId - SFTP session ID
 * @param localPath - Local file path
 * @param remotePath - Remote file path
 * @returns Transfer ID
 * @throws Enhanced error if upload fails
 */
async function uploadFile(
  sessionId: string,
  localPath: string,
  remotePath: string,
): Promise<string> {
  const context: ErrorContext = {
    operation: "Upload File",
    context: { localPath, remotePath },
  };

  try {
    const transferId = await sftpService.uploadSFTPFile(
      sessionId,
      localPath,
      remotePath,
    );
    return transferId;
  } catch (error) {
    const errorMessage = handleError(error, context);
    message.error(errorMessage);
    throw new Error(errorMessage);
  }
}

/**
 * Download file with error handling
 * @param sessionId - SFTP session ID
 * @param remotePath - Remote file path
 * @param localPath - Local file path
 * @returns Transfer ID
 * @throws Enhanced error if download fails
 */
async function downloadFile(
  sessionId: string,
  remotePath: string,
  localPath: string,
): Promise<string> {
  const context: ErrorContext = {
    operation: "Download File",
    context: { remotePath, localPath },
  };

  try {
    const transferId = await sftpService.downloadSFTPFile(
      sessionId,
      remotePath,
      localPath,
    );
    return transferId;
  } catch (error) {
    const errorMessage = handleError(error, context);
    message.error(errorMessage);
    throw new Error(errorMessage);
  }
}

/**
 * Compare directories with retry logic
 * @param sessionId - SFTP session ID
 * @param localPath - Local directory path
 * @param remotePath - Remote directory path
 * @returns Array of differences
 * @throws Enhanced error if comparison fails
 */
async function compareDirectories(
  sessionId: string,
  localPath: string,
  remotePath: string,
): Promise<DiffEntry[]> {
  const context: ErrorContext = {
    operation: "Compare Directories",
    context: { localPath, remotePath },
  };

  try {
    return await withRetry(
      () =>
        sftpService.compareSFTPDirectories(sessionId, localPath, remotePath),
      { maxRetries: 1 },
      context,
    );
  } catch (error) {
    const errorMessage = handleError(error, context);
    // Failed to compare directories: errorMessage
    throw new Error(errorMessage);
  }
}

/**
 * Sync directories with error handling
 * @param sessionId - SFTP session ID
 * @param operation - Sync operation configuration
 * @throws Enhanced error if sync fails
 */
async function syncDirectories(
  sessionId: string,
  operation: SyncOperation,
): Promise<void> {
  const context: ErrorContext = {
    operation: "Sync Directories",
    context: {
      sessionId,
      direction: operation.direction,
      localPath: operation.localPath,
      remotePath: operation.remotePath,
    },
  };

  try {
    await sftpService.syncSFTPDirectories(sessionId, operation);
  } catch (error) {
    const errorMessage = handleError(error, context);
    // Failed to sync directories: errorMessage
    throw new Error(errorMessage);
  }
}

/**
 * Read file content as text (remote) with error handling
 * @param sessionId - SFTP session ID
 * @param path - Remote file path
 * @returns File content as string
 * @throws Enhanced error if read fails
 */
async function readFile(sessionId: string, path: string): Promise<string> {
  const context: ErrorContext = {
    operation: "Read File",
    context: { path },
  };

  try {
    return await sftpService.readSFTPFile(sessionId, path);
  } catch (error) {
    const errorMessage = handleError(error, context);
    // Failed to read file: errorMessage
    throw new Error(errorMessage);
  }
}

/**
 * Write file content as text (remote) with error handling
 * @param sessionId - SFTP session ID
 * @param path - Remote file path
 * @param content - File content to write
 * @throws Enhanced error if write fails
 */
async function writeFile(
  sessionId: string,
  path: string,
  content: string,
): Promise<void> {
  const context: ErrorContext = {
    operation: "Write File",
    context: { path, contentLength: content.length },
  };

  try {
    await sftpService.writeSFTPFile(sessionId, path, content);
  } catch (error) {
    const errorMessage = handleError(error, context);
    // Failed to write file: errorMessage
    throw new Error(errorMessage);
  }
}

/**
 * Pause transfer with error handling
 * @param transferId - Transfer ID to pause
 * @throws Enhanced error if pause fails
 */
async function pauseTransfer(transferId: string): Promise<void> {
  const context: ErrorContext = {
    operation: "Pause Transfer",
    context: { transferId },
  };

  try {
    await sftpService.pauseSFTPTransfer(transferId);
  } catch (error) {
    const errorMessage = handleError(error, context);
    message.error(errorMessage);
    throw new Error(errorMessage);
  }
}

/**
 * Resume transfer with error handling
 * @param transferId - Transfer ID to resume
 * @throws Enhanced error if resume fails
 */
async function resumeTransfer(transferId: string): Promise<void> {
  const context: ErrorContext = {
    operation: "Resume Transfer",
    context: { transferId },
  };

  try {
    await sftpService.resumeSFTPTransfer(transferId);
  } catch (error) {
    const errorMessage = handleError(error, context);
    message.error(errorMessage);
    throw new Error(errorMessage);
  }
}

/**
 * Set transfer priority (0-255, higher = higher priority)
 * @param transferId - Transfer ID
 * @param priority - Priority value
 * @throws Enhanced error if operation fails
 */
async function setTransferPriority(
  transferId: string,
  priority: number,
): Promise<void> {
  const context: ErrorContext = {
    operation: "Set Transfer Priority",
    context: { transferId, priority },
  };

  try {
    await sftpService.setTransferPriority(transferId, priority);
  } catch (error) {
    const errorMessage = handleError(error, context);
    message.error(errorMessage);
    throw new Error(errorMessage);
  }
}

/**
 * Get all transfers with optional status filter
 * @param statusFilter - Optional status filter
 * @returns List of transfers
 */
async function getAllTransfers(statusFilter?: string) {
  const context: ErrorContext = {
    operation: "Get All Transfers",
    context: { statusFilter },
  };

  try {
    return await sftpService.getAllTransfers(statusFilter);
  } catch (error) {
    const errorMessage = handleError(error, context);
    message.error(errorMessage);
    throw new Error(errorMessage);
  }
}

/**
 * Reorder transfer queue
 * @param transferIds - Array of transfer IDs in desired order
 * @throws Enhanced error if reorder fails
 */
async function reorderQueue(transferIds: string[]): Promise<void> {
  const context: ErrorContext = {
    operation: "Reorder Queue",
    context: { transferCount: transferIds.length },
  };

  try {
    await sftpService.reorderQueue(transferIds);
  } catch (error) {
    const errorMessage = handleError(error, context);
    message.error(errorMessage);
    throw new Error(errorMessage);
  }
}

/**
 * Retry failed transfer
 * @param transferId - Transfer ID to retry
 * @throws Enhanced error if retry fails
 */
async function retryTransfer(transferId: string): Promise<void> {
  const context: ErrorContext = {
    operation: "Retry Transfer",
    context: { transferId },
  };

  try {
    await sftpService.retryTransfer(transferId);
  } catch (error) {
    const errorMessage = handleError(error, context);
    message.error(errorMessage);
    throw new Error(errorMessage);
  }
}

/**
 * Search for text in files
 * @param sessionId - SFTP session ID
 * @param path - Path to search in
 * @param query - Search query
 * @returns Search results
 */
async function search(
  sessionId: string,
  path: string,
  query: string,
): Promise<SearchResult[]> {
  const context: ErrorContext = {
    operation: "Search SFTP",
    context: { path, query },
  };

  try {
    return await sftpService.searchSFTP(sessionId, path, query);
  } catch (error) {
    const errorMessage = handleError(error, context);
    // Failed to search: errorMessage
    throw new Error(errorMessage);
  }
}

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
    return Array.from(browserState.value.activeTransfers.values()).filter(
      (t) => t.status !== "cancelled",
    );
  });

  // Actions
  /**
   * Connect to SFTP server with retry logic
   * @param profileId - SSH profile ID to connect with
   * @returns Session ID
   * @throws Enhanced error with context if connection fails
   */
  async function connect(profileId: string): Promise<string> {
    connecting.value = true;
    const context: ErrorContext = {
      operation: "SFTP Connection",
      context: { profileId },
    };

    try {
      const sessionId = await withRetry(
        () => sftpService.connectSFTP(profileId),
        {
          maxRetries: 2,
          retryDelay: 2000,
        },
        context,
      );

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

      startRealtime();

      return sessionId;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      connecting.value = false;
    }
  }

  /**
   * Disconnect SFTP session with error handling
   * @param sessionId - Session ID to disconnect
   * @throws Enhanced error if disconnect fails
   */
  async function disconnect(sessionId: string): Promise<void> {
    const context: ErrorContext = {
      operation: "Disconnect SFTP",
      context: { sessionId },
    };

    try {
      await sftpService.disconnectSFTP(sessionId);
      sessions.value.delete(sessionId);

      if (activeSessionId.value === sessionId) {
        activeSessionId.value = null;
        browserState.value.activeSessionId = null;
        browserState.value.remoteFiles = [];
        browserState.value.remotePath = "";
      }
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  /**
   * List local directory with error handling
   * @param path - Local directory path
   * @throws Enhanced error if listing fails
   */
  async function listLocalDirectory(path: string): Promise<void> {
    if (browserState.value.localPath !== path) {
      browserState.value.selectedLocalFiles.clear();
    }

    browserState.value.loading.local = true;
    const context: ErrorContext = {
      operation: "List Local Directory",
      context: { path },
    };

    try {
      const entries = await readDir(path);

      const fileResults = await Promise.allSettled(
        entries.map((entry) => processFileEntry(entry, path)),
      );

      const files: FileEntry[] = [];
      for (const result of fileResults) {
        if (result.status === "fulfilled" && result.value !== null) {
          files.push(result.value);
        } else if (result.status === "rejected") {
          // Failed to process directory entry: result.reason
        }
      }

      browserState.value.localFiles = files;
      browserState.value.localPath = path;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      browserState.value.loading.local = false;
    }
  }

  /**
   * List remote directory with retry logic
   * @param sessionId - SFTP session ID
   * @param path - Remote directory path
   * @throws Enhanced error if listing fails
   */
  async function listRemoteDirectory(
    sessionId: string,
    path: string,
  ): Promise<void> {
    if (browserState.value.remotePath !== path) {
      browserState.value.selectedRemoteFiles.clear();
    }

    browserState.value.loading.remote = true;
    const context: ErrorContext = {
      operation: "List Remote Directory",
      context: { sessionId, path },
    };

    try {
      const files = await withRetry(
        () => sftpService.listSFTPDirectory(sessionId, path),
        { maxRetries: 2 },
        context,
      );
      browserState.value.remoteFiles = files;
      browserState.value.remotePath = path;

      const session = sessions.value.get(sessionId);
      if (session) {
        session.lastUsed = new Date().toISOString();
      }
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      browserState.value.loading.remote = false;
    }
  }

  /**
   * Upload file with error handling
   * @param sessionId - SFTP session ID
   * @param localPath - Local file path
   * @param remotePath - Remote file path
   * @returns Transfer ID
   * @throws Enhanced error if upload fails
   */


  /**
   * Download file with error handling
   * @param sessionId - SFTP session ID
   * @param remotePath - Remote file path
   * @param localPath - Local file path
   * @returns Transfer ID
   * @throws Enhanced error if download fails
   */


  /**
   * Cancel transfer with error handling
   * @param transferId - Transfer ID to cancel
   * @throws Enhanced error if cancel fails
   */
  async function cancelTransfer(transferId: string): Promise<void> {
    const context: ErrorContext = {
      operation: "Cancel Transfer",
      context: { transferId },
    };

    try {
      await sftpService.cancelSFTPTransfer(transferId);
      browserState.value.activeTransfers.delete(transferId);
    } catch (error) {
      const errorMessage = handleError(error, context);
      // Failed to cancel transfer: errorMessage
      throw new Error(errorMessage);
    }
  }

  /**
   * Compare directories with retry logic
   * @param sessionId - SFTP session ID
   * @param localPath - Local directory path
   * @param remotePath - Remote directory path
   * @returns Array of differences
   * @throws Enhanced error if comparison fails
   */


  /**
   * Sync directories with error handling
   * @param sessionId - SFTP session ID
   * @param operation - Sync operation configuration
   * @throws Enhanced error if sync fails
   */


  /**
   * Rename file or directory with error handling
   * @param sessionId - SFTP session ID
   * @param oldPath - Current file/directory path
   * @param newPath - New file/directory path
   * @throws Enhanced error if rename fails
   */
  async function renameFile(
    sessionId: string,
    oldPath: string,
    newPath: string,
  ): Promise<void> {
    const context: ErrorContext = {
      operation: "Rename File",
      context: { oldPath, newPath },
    };

    try {
      await sftpService.renameSFTP(sessionId, oldPath, newPath);
      if (browserState.value.remotePath) {
        await listRemoteDirectory(sessionId, browserState.value.remotePath);
      }
    } catch (error) {
      const errorMessage = handleError(error, context);
      // Failed to rename file: errorMessage
      throw new Error(errorMessage);
    }
  }

  /**
   * Delete file or directory with error handling
   * @param sessionId - SFTP session ID
   * @param path - File/directory path to delete
   * @param recursive - Whether to delete recursively (for directories)
   * @throws Enhanced error if delete fails
   */
  async function deleteFile(
    sessionId: string,
    path: string,
    recursive: boolean = false,
  ): Promise<void> {
    const context: ErrorContext = {
      operation: "Delete File",
      context: { path, recursive },
    };

    try {
      await sftpService.deleteSFTP(sessionId, path, recursive);
      if (browserState.value.remotePath) {
        await listRemoteDirectory(sessionId, browserState.value.remotePath);
      }
    } catch (error) {
      const errorMessage = handleError(error, context);
      // Failed to delete file: errorMessage
      throw new Error(errorMessage);
    }
  }

  /**
   * Set file permissions with error handling
   * @param sessionId - SFTP session ID
   * @param path - File path
   * @param mode - Permission mode (octal)
   * @throws Enhanced error if setting permissions fails
   */
  async function setPermissions(
    sessionId: string,
    path: string,
    mode: number,
  ): Promise<void> {
    const context: ErrorContext = {
      operation: "Set Permissions",
      context: { path, mode: `0o${mode.toString(8)}` },
    };

    try {
      await sftpService.setSFTPPermissions(sessionId, path, mode);
      if (browserState.value.remotePath) {
        await listRemoteDirectory(sessionId, browserState.value.remotePath);
      }
    } catch (error) {
      const errorMessage = handleError(error, context);
      // Failed to set permissions: errorMessage
      throw new Error(errorMessage);
    }
  }

  /**
   * Create directory with error handling
   * @param sessionId - SFTP session ID
   * @param path - Directory path to create
   * @throws Enhanced error if creation fails
   */
  async function createDirectory(
    sessionId: string,
    path: string,
  ): Promise<void> {
    const context: ErrorContext = {
      operation: "Create Directory",
      context: { path },
    };

    try {
      await sftpService.createSFTPDirectory(sessionId, path);
      if (browserState.value.remotePath) {
        await listRemoteDirectory(sessionId, browserState.value.remotePath);
      }
    } catch (error) {
      const errorMessage = handleError(error, context);
      // Failed to create directory: errorMessage
      throw new Error(errorMessage);
    }
  }

  /**
   * Read file content as text (remote) with error handling
   * @param sessionId - SFTP session ID
   * @param path - Remote file path
   * @returns File content as string
   * @throws Enhanced error if read fails
   */


  /**
   * Write file content as text (remote) with error handling
   * @param sessionId - SFTP session ID
   * @param path - Remote file path
   * @param content - File content to write
   * @throws Enhanced error if write fails
   */


  let unsubscribeTransferRealtime: (() => void) | null = null;

  /**
   * Start listening to realtime events
   */
  async function startRealtime(): Promise<void> {
    if (unsubscribeTransferRealtime) return;
    try {
      const u1 = await api.listen<{
        transferId: string;
        transferredBytes: number;
        totalBytes: number;
      }>("sftp_transfer_progress", async (data) => {
        let existing = browserState.value.activeTransfers.get(data.transferId);
        if (!existing) {
          try {
            existing = await sftpService.getSFTPTransferProgress(
              data.transferId,
            );
            browserState.value.activeTransfers.set(data.transferId, existing);
          } catch (error) {
            const errorMessage = handleError(error, {
              operation: "Get Transfer Progress",
              context: { transferId: data.transferId },
            });
            console.error("Failed to get transfer progress:", errorMessage);
            return;
          }
        }

        existing.transferredBytes = data.transferredBytes;
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

      const u2 = await api.listen<{ transferId: string }>(
        "sftp_transfer_complete",
        async (data) => {
          try {
            const progress = await sftpService.getSFTPTransferProgress(
              data.transferId,
            );
            browserState.value.activeTransfers.set(data.transferId, progress);
          } catch (error) {
            const errorMessage = handleError(error, {
              operation: "Get Transfer Progress (Complete)",
              context: { transferId: data.transferId },
            });
            console.error("Failed to get transfer progress:", errorMessage);
          }
        },
      );

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
      const errorMessage = handleError(error, {
        operation: "Start SFTP Realtime",
      });
      console.error("Failed to start SFTP realtime:", errorMessage);
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

  /**
   * Pause transfer with error handling
   * @param transferId - Transfer ID to pause
   * @throws Enhanced error if pause fails
   */


  /**
   * Resume transfer with error handling
   * @param transferId - Transfer ID to resume
   * @throws Enhanced error if resume fails
   */


  /**
   * Set transfer priority (0-255, higher = higher priority)
   * @param transferId - Transfer ID
   * @param priority - Priority value
   * @throws Enhanced error if operation fails
   */


  /**
   * Get all transfers with optional status filter
   * @param statusFilter - Optional status filter
   * @returns List of transfers
   */


  /**
   * Reorder transfer queue
   * @param transferIds - Array of transfer IDs in desired order
   * @throws Enhanced error if reorder fails
   */


  /**
   * Retry failed transfer
   * @param transferId - Transfer ID to retry
   * @throws Enhanced error if retry fails
   */


  /**
   * Search for text in files
   * @param sessionId - SFTP session ID
   * @param path - Path to search in
   * @param query - Search query
   * @returns Search results
   */


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
    readFile,
    writeFile,
    startRealtime,
    stopRealtime,
    pauseTransfer,
    resumeTransfer,
    setTransferPriority,
    getAllTransfers,
    reorderQueue,
    retryTransfer,
    search,
  };
});
