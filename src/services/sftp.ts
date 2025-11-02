import { api } from "./api";
import type {
  FileEntry,
  TransferProgress,
  SyncOperation,
  DiffEntry,
} from "../types/sftp";

/**
 * Connect to SFTP server using SSH profile
 */
export async function connectSFTP(profileId: string): Promise<string> {
  return await api.call("sftp_connect", { profileId });
}

/**
 * Disconnect SFTP session
 */
export async function disconnectSFTP(sessionId: string): Promise<void> {
  return await api.call("sftp_disconnect", { sessionId });
}

/**
 * List directory contents
 */
export async function listSFTPDirectory(
  sessionId: string,
  path: string,
): Promise<FileEntry[]> {
  return await api.call("sftp_list_directory", { sessionId, path });
}

/**
 * Get file attributes (stat)
 */
export async function statSFTP(
  sessionId: string,
  path: string,
): Promise<FileEntry> {
  return await api.call("sftp_stat", { sessionId, path });
}

/**
 * Create directory
 */
export async function createSFTPDirectory(
  sessionId: string,
  path: string,
): Promise<void> {
  return await api.call("sftp_create_directory", { sessionId, path });
}

/**
 * Rename/move file or directory
 */
export async function renameSFTP(
  sessionId: string,
  oldPath: string,
  newPath: string,
): Promise<void> {
  return await api.call("sftp_rename", {
    sessionId,
    oldPath,
    newPath,
  });
}

/**
 * Delete file or directory
 */
export async function deleteSFTP(
  sessionId: string,
  path: string,
  recursive: boolean = false,
): Promise<void> {
  return await api.call("sftp_delete", { sessionId, path, recursive });
}

/**
 * Set file permissions (chmod)
 */
export async function setSFTPPermissions(
  sessionId: string,
  path: string,
  mode: number,
): Promise<void> {
  return await api.call("sftp_set_permissions", { sessionId, path, mode });
}

/**
 * Create symlink
 */
export async function createSFTPSymlink(
  sessionId: string,
  target: string,
  linkPath: string,
): Promise<void> {
  return await api.call("sftp_create_symlink", {
    sessionId,
    target,
    linkPath,
  });
}

/**
 * Read symlink target
 */
export async function readSFTPSymlink(
  sessionId: string,
  path: string,
): Promise<string> {
  return await api.call("sftp_read_symlink", { sessionId, path });
}

/**
 * Upload file from local to remote
 */
export async function uploadSFTPFile(
  sessionId: string,
  localPath: string,
  remotePath: string,
): Promise<string> {
  // Tauri v2 automatically injects AppHandle in commands
  return await api.call("sftp_upload_file", {
    sessionId,
    localPath,
    remotePath,
  });
}

/**
 * Download file from remote to local
 */
export async function downloadSFTPFile(
  sessionId: string,
  remotePath: string,
  localPath: string,
): Promise<string> {
  return await api.call("sftp_download_file", {
    sessionId,
    remotePath,
    localPath,
  });
}

/**
 * Get transfer progress
 */
export async function getSFTPTransferProgress(
  transferId: string,
): Promise<TransferProgress> {
  return await api.call("sftp_get_transfer_progress", { transferId });
}

/**
 * Cancel transfer
 */
export async function cancelSFTPTransfer(transferId: string): Promise<void> {
  return await api.call("sftp_cancel_transfer", { transferId });
}

/**
 * Pause transfer
 */
export async function pauseSFTPTransfer(transferId: string): Promise<void> {
  return await api.call("sftp_pause_transfer", { transferId });
}

/**
 * Resume interrupted transfer
 */
export async function resumeSFTPTransfer(transferId: string): Promise<void> {
  return await api.call("sftp_resume_transfer", { transferId });
}

/**
 * Compare local and remote directories
 */
export async function compareSFTPDirectories(
  sessionId: string,
  localPath: string,
  remotePath: string,
): Promise<DiffEntry[]> {
  return await api.call("sftp_compare_directories", {
    sessionId,
    localPath,
    remotePath,
  });
}

/**
 * Sync directories
 */
export async function syncSFTPDirectories(
  sessionId: string,
  operation: SyncOperation,
): Promise<void> {
  return await api.call("sftp_sync_directory", { sessionId, operation });
}

/**
 * Read file content as text
 */
export async function readSFTPFile(
  sessionId: string,
  path: string,
): Promise<string> {
  return await api.call("sftp_read_file", { sessionId, path });
}

/**
 * Write file content as text
 */
export async function writeSFTPFile(
  sessionId: string,
  path: string,
  content: string,
): Promise<void> {
  return await api.call("sftp_write_file", { sessionId, path, content });
}
